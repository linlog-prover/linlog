// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The zones of a derived sequent: the unrestricted zone as a set, the
//! linear zone as a multiset, and the `State` a subproof proves.

use super::error::Dyadic;
use crate::hash::{HashMap, HashSet};
use crate::occurrences::Member;
use crate::sequents::Eigen;

/// Returns the bytes of a hash table whose entries take `entry` bytes and
/// which held `most` of them when it was fullest. The standard library's
/// table has a power of two of slots, four at least and seven in eight
/// taken at most, each with a byte beside its entry, and sixteen bytes at
/// the end, the width of its SSE2 group; it never gives slots back, and a
/// copy has as many as its original. A table that entries came and went
/// from can have twice or four times the slots, so this is the least the
/// table takes and within that factor of what it does. Where the group is
/// eight bytes (wasm32, and aarch64's NEON) the table takes eight bytes
/// fewer than counted, which only makes a bound stricter.
///
/// Nothing overflows: `most` counts distinct occurrence ids, 2³² at most,
/// so there are 2³³ slots at most and fewer than 2³⁸ bytes, counted in
/// `u64` on every target; on a 32-bit one `most` is below 2³² anyway.
pub(super) fn table_bytes(most: usize, entry: usize) -> u64 {
    if most == 0 {
        return 0;
    }
    let slots = (most as u64 * 8).div_ceil(7).next_power_of_two().max(4);
    slots * (entry as u64 + 1) + 16
}

/// The unrestricted zone: a set of members.
#[derive(Clone, Debug, Default)]
pub(super) struct Zone {
    /// The members.
    pub(super) members: HashSet<Member>,
    /// The most members the table has held, which its memory follows:
    /// distinct occurrence ids, so 2³² at most, and a table's length on a
    /// 32-bit target, so below 2³² there.
    pub(super) most: usize,
}

impl Zone {
    /// Returns the number of members.
    pub(super) fn len(&self) -> usize {
        self.members.len()
    }

    /// Returns whether the set has no member.
    pub(super) fn is_empty(&self) -> bool {
        self.members.is_empty()
    }

    /// Returns the members, in no particular order.
    pub(super) fn iter(&self) -> impl Iterator<Item = Member> + '_ {
        self.members.iter().copied()
    }

    /// Adds `o` and returns whether it was not a member.
    pub(super) fn insert(&mut self, o: Member) -> bool {
        // The table makes room before it looks a member up, so adding one
        // that is there already would double a full table, which the
        // count of its bytes would not see.
        if self.members.contains(&o) {
            return false;
        }
        self.members.insert(o);
        self.most = self.most.max(self.members.len());
        true
    }

    /// Removes `o` and returns whether it was a member.
    pub(super) fn remove(&mut self, o: Member) -> bool {
        self.members.remove(&o)
    }

    /// Returns the bytes the table takes.
    pub(super) fn bytes(&self) -> u64 {
        table_bytes(self.most, size_of::<Member>())
    }
}

/// How many copies of each occurrence a linear zone holds: a multiset
/// whose insertions and removals cost the same whatever its size.
///
/// The counters saturate and never wrap: a zone of more than
/// [`MOST`](Self::MOST) members is refused by the pass before any rule
/// reads it, so every count a rule sees is the true one.
#[derive(Clone, Debug, Default)]
pub(super) struct Bag {
    /// The copies of every member, never zero. A count saturates at
    /// `u32::MAX`, and then the zone has that many members at least, more
    /// than [`MOST`](Self::MOST): a count that a rule reads is exact.
    pub(super) counts: HashMap<Member, u32>,
    /// The number of members, with repeats. The sum of two zones within
    /// [`MOST`](Self::MOST) is below 2³³, so on a 64-bit target it is
    /// exact until the pass refuses it; where a `usize` has 32 bits it
    /// saturates at `u32::MAX`, which is over the bound as well.
    pub(super) len: usize,
    /// The most distinct members the table has held, which its memory
    /// follows: distinct occurrence ids, so 2³² at most, and a table's
    /// length on a 32-bit target, so below 2³² there.
    pub(super) most: usize,
}

impl PartialEq for Bag {
    /// Returns whether both hold the same copies of the same members,
    /// whatever their tables held before.
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.counts == other.counts
    }
}

impl Eq for Bag {}

impl Bag {
    /// The most members a zone may hold: fewer than a counter saturates
    /// at, so that a saturated counter is always over it. It fits a
    /// `usize` of 32 bits.
    pub(super) const MOST: usize = (u32::MAX - 1) as usize;

    /// Returns the multiset of the given ids.
    pub(super) fn of(ids: impl IntoIterator<Item = Member>) -> Self {
        let mut bag = Self::default();
        for o in ids {
            bag.insert(o);
        }
        bag
    }

    /// Returns whether the multiset has no member.
    pub(super) fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the members with their numbers of copies, in no particular
    /// order.
    pub(super) fn counts(&self) -> impl Iterator<Item = (Member, u32)> + '_ {
        self.counts.iter().map(|(&o, &n)| (o, n))
    }

    /// Returns the members in ascending order, with repeats.
    pub(super) fn sorted(&self) -> Vec<Member> {
        let mut ids: Vec<Member> = self
            .counts()
            .flat_map(|(o, n)| std::iter::repeat_n(o, n as usize))
            .collect();
        ids.sort_unstable();
        ids
    }

    /// Adds one copy of `o`.
    pub(super) fn insert(&mut self, o: Member) {
        let n = self.counts.entry(o).or_insert(0);
        *n = n.saturating_add(1);
        self.len = self.len.saturating_add(1);
        self.most = self.most.max(self.counts.len());
    }

    /// Removes one copy of `o` and returns whether there was one.
    pub(super) fn remove(&mut self, o: Member) -> bool {
        let Some(n) = self.counts.get_mut(&o) else {
            return false;
        };
        // A count in the table is one at least, and the length is no less
        // than any count.
        *n -= 1;
        if *n == 0 {
            self.counts.remove(&o);
        }
        self.len -= 1;
        true
    }

    /// Adds every copy of every member of `other`: the multiset sum, the
    /// smaller table poured into the larger.
    pub(super) fn add(&mut self, mut other: Self) {
        if other.counts.len() > self.counts.len() {
            std::mem::swap(self, &mut other);
        }
        for (o, n) in other.counts {
            let own = self.counts.entry(o).or_insert(0);
            *own = own.saturating_add(n);
        }
        self.len = self.len.saturating_add(other.len);
        self.most = self.most.max(self.counts.len());
    }

    /// Raises every member's copies to those `other` has of it: the
    /// multiset union.
    pub(super) fn unite(&mut self, mut other: Self) {
        if other.counts.len() > self.counts.len() {
            std::mem::swap(self, &mut other);
        }
        for (o, n) in other.counts {
            let own = self.counts.entry(o).or_insert(0);
            if n > *own {
                self.len = self.len.saturating_add((n - *own) as usize);
                *own = n;
            }
        }
        self.most = self.most.max(self.counts.len());
    }

    /// Returns the bytes the table takes.
    pub(super) fn bytes(&self) -> u64 {
        table_bytes(self.most, size_of::<(Member, u32)>())
    }

    /// Returns whether no id has more copies here than in `other`.
    pub(super) fn is_subset(&self, other: &Self) -> bool {
        self.len <= other.len
            && self
                .counts()
                .all(|(o, n)| other.counts.get(&o).is_some_and(|&m| n <= m))
    }
}

/// What a subproof proves, as the checker derives it from the node's
/// premises: the dyadic sequent `⊢ Θ ; Γ`, or with `any` set `⊢ Θ ; Γ, Δ` for
/// every `Δ`, because a `⊤` leaf above absorbs whatever context it is given.
///
/// `Θ` is the least unrestricted zone the subproof needs, the occurrences it
/// copies without moving them into `Θ` itself; since weakening and
/// contraction on `Θ` are implicit, any larger zone does as well. So a proof
/// is correct when its root needs an empty `Θ` and derives the sequent's
/// formulas, and the checker never has to know the actual zone.
///
/// Both zones are tables of their members, so a sequent takes memory for
/// what it holds and none for the width of the forest.
///
/// A sequent that a rule or an observer reads has passed
/// [`Pass::within`](super::pass::Pass::within), so its linear zone has [`Bag::MOST`] members at
/// most, fewer than 2³² − 1. The sums below are exact for such a sequent;
/// while a rule builds one they saturate, which only a longer zone can
/// make them do, and that zone is refused before anything reads them.
#[derive(Clone, Debug)]
pub(crate) struct State {
    /// The unrestricted zone the subproof needs.
    pub(super) theta: Zone,
    /// The linear zone.
    pub(super) gamma: Bag,
    /// The members of the linear zone in output position, under a reading:
    /// no more than the zone has members, and saturating with its length,
    /// at `usize::MAX` as it does, which on a 32-bit target is `u32::MAX`
    /// and over [`Bag::MOST`] as well.
    pub(super) outputs: usize,
    /// The weights of the linear zone's members, added up: fewer than
    /// 2³² − 1 members of a weight below 2³² each, so below 2⁶⁴.
    pub(super) linear: u64,
    /// The weights of the `?` formulas of the unrestricted zone's members,
    /// added up. It is the sum over the set as it stands at every moment,
    /// and a set of occurrences has fewer than 2³² − 1 members (a forest
    /// has no more occurrences), each of a weight below 2³²: below 2⁶⁴.
    pub(super) unrestricted: u64,
    /// The weights of the linear zone's members in output position: no
    /// more than `linear`.
    pub(super) goal: u64,
    /// Whether a `⊤` above absorbs any further linear context.
    pub(super) any: bool,
}

/// A set of eigenvariables, as the pass carries it from a node to the
/// root: those the witnesses and cut frames above name and no `∀` step
/// above introduces.
#[derive(Clone, Debug, Default)]
pub(super) struct Eigens {
    /// The members.
    pub(super) set: HashSet<Eigen>,
    /// The most members the table has held, which its memory follows:
    /// fewer than a proof's terms, so a table's length.
    pub(super) most: usize,
}

impl Eigens {
    /// Adds `a`.
    pub(super) fn insert(&mut self, a: Eigen) {
        if self.set.insert(a) {
            self.most = self.most.max(self.set.len());
        }
    }

    /// Adds every member of `other`, the smaller set poured into the
    /// larger.
    pub(super) fn add(&mut self, mut other: Self) {
        if other.set.len() > self.set.len() {
            std::mem::swap(self, &mut other);
        }
        self.set.extend(other.set);
        self.most = self.most.max(self.set.len());
    }

    /// Returns the bytes the set takes: its table and its box.
    pub(super) fn bytes(&self) -> u64 {
        table_bytes(self.most, size_of::<Eigen>()) + size_of::<Self>() as u64 + 16
    }
}

impl State {
    /// What a sequent takes besides its tables: the value, and what the
    /// allocator keeps for it.
    pub(super) const OWN: u64 = size_of::<Self>() as u64 + 16;

    /// Returns the bytes the sequent takes in memory, at the least: the
    /// value and its two tables as they were when fullest. Fewer than 2³⁹.
    pub(super) fn bytes(&self) -> u64 {
        Self::OWN + self.theta.bytes() + self.gamma.bytes()
    }

    /// Returns the entries of the sequent's two tables, which a rule that
    /// copies, pours or compares it handles: distinct occurrence ids, so
    /// fewer than 2³³, which a `u64` holds; each table's length is taken
    /// to `u64` before the sum, which on a 32-bit target a `usize` might
    /// not hold.
    pub(super) fn entries(&self) -> u64 {
        self.theta.len() as u64 + self.gamma.counts.len() as u64
    }

    /// Returns the sequent as ids for an error report.
    pub(super) fn to_dyadic(&self) -> Dyadic {
        let mut theta: Vec<Member> = self.theta.iter().collect();
        theta.sort_unstable();
        Dyadic {
            theta,
            gamma: self.gamma.sorted(),
            any: self.any,
        }
    }

    /// Returns the unrestricted occurrences the subproof needs, in no
    /// particular order.
    pub(crate) fn unrestricted(&self) -> impl Iterator<Item = Member> + '_ {
        self.theta.iter()
    }

    /// Returns the linear zone's members with their numbers of copies, in
    /// no particular order.
    pub(crate) fn linear(&self) -> impl Iterator<Item = (Member, u32)> + '_ {
        self.gamma.counts()
    }

    /// Returns how many unrestricted occurrences the subproof needs.
    pub(crate) fn needs(&self) -> usize {
        self.theta.len()
    }

    /// Returns whether a `⊤` above absorbs any further linear context.
    pub(crate) fn absorbs(&self) -> bool {
        self.any
    }

    /// Returns the weight of the standard sequent `⊢ ?Θ, Γ`: its formulas'
    /// weights added up, or `u64::MAX` when that is less. Each zone's sum
    /// is below 2⁶⁴ and the two together need not be.
    pub(crate) fn weight(&self) -> u64 {
        self.linear.saturating_add(self.unrestricted)
    }

    /// Returns the weight of the linear zone's members in output position.
    pub(crate) fn goal_weight(&self) -> u64 {
        self.goal
    }
}
