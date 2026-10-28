// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The count invariants proof search prunes with: per occurrence, an
//! interval of the balance every atom can reach below it, and the weight
//! the `MLL` count equation sums. Both are necessary conditions on a
//! provable sequent, computed once per forest and summed per sequent.

use super::Switches;
use super::split::Forced;
use crate::fragment::{Fragment, Mode};
use crate::occurrences::{Forest, OccId, Sign};
use crate::proofs::Branch;
use crate::search::Bias;
use crate::search::memory::{Account, bytes_of};
use crate::search::{Equation, OneSided, Reason, Refutation, Unbalanced, set_up_stopped};
use crate::sequents::{Atom, Kind};

#[cfg(test)]
thread_local! {
    /// Whether an atom whose literals below a `!` or `?` all have one sign
    /// keeps its rows, on this thread: off only where a test compares the
    /// counts with and without them.
    static ONE_SIDED: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

/// Returns what `f` returns, run with the counts of [`Counts::new_until`]
/// on this thread giving no rows to an atom with any literal below a `!`
/// or `?`, as if both signs lay there.
#[cfg(test)]
pub(crate) fn without_one_sided<T>(f: impl FnOnce() -> T) -> T {
    /// Turns the rows back on when dropped, after a panic too.
    struct Restore;
    impl Drop for Restore {
        fn drop(&mut self) {
            ONE_SIDED.set(true);
        }
    }
    ONE_SIDED.set(false);
    let _restore = Restore;
    f()
}

/// Returns whether the interval check runs in `mode`, where a `⊤` lies
/// below an exponential or not (`absorbs_from_copies`): weakening discards
/// any imbalance, and a copy of a `⊤` absorbs one.
pub(crate) const fn intervals_run(mode: Mode, absorbs_from_copies: bool) -> bool {
    !mode.affine && !absorbs_from_copies
}

/// What a formula occurrence can contribute to the per-atom balance of a
/// sequent, and to the `MLL` count equation, precomputed for every
/// occurrence of a forest.
///
/// **Intervals.** For an atom `a`, the balance of a sequent is the number of
/// `a` literals minus the number of `~a` literals it decomposes into; every
/// axiom consumes one of each, so a provable sequent has balance zero for
/// every atom. Under `&` and `⊕` only one side of the connective is
/// decomposed, so a formula's contribution is an interval `lo ..= hi`:
/// a literal is `±1`, `⊗` and `⅋` sum their sides, `&` and `⊕` take the
/// hull of theirs, and units contribute nothing. A sequent whose summed
/// interval excludes zero for some atom is unprovable. `⊤` proves any
/// sequent it appears in, so an occurrence with a `⊤` below it *absorbs*:
/// its row is meaningless and any sequent containing it passes. Rows are
/// sparse, over the atoms that occur below the occurrence.
///
/// A literal below a `?` or `!` can be copied any number of times, none
/// included. An atom with literals of both signs there gets no entry in
/// any row: copies can balance it either way. Where the interval check
/// runs ([`intervals_run`]: in linear mode, with no `⊤` below an
/// exponential), an atom whose literals there all have one sign keeps its
/// entries, each literal's, while the rows of a `!` or `?` stay empty, and
/// a set is ruled out by one bound alone ([`excluded`](Self::excluded)).
/// The invariant, for every provable dyadic sequent `Θ ; Γ`, whatever `Θ`
/// holds, unless a member of `Γ` absorbs: for an atom with every literal
/// below an exponential positive, the least balance of `Γ`'s rows is at
/// most zero (negative: the greatest at least zero; none below one: zero
/// lies between them). By induction on the proof: an axiom's `Γ` is a
/// literal and its dual, and the initial rule from `Θ` leaves a lone
/// literal whose dual, a copy, has the copied sign; `⊗`, `⅋` and Mix add
/// their premises' rows, `&` and `⊕` take hulls, which hold the premise's
/// least; `1`, `⊥`, a promotion's `!A` (with `Γ` empty above it) and a
/// weakening of a `?` formula count nothing; `?A` moves `A` into `Θ`,
/// which no row counts, and its own row is empty; a copy from `Θ` adds a
/// member whose entries all have the copied sign, so its least is at
/// least zero and `Γ`'s least was at most the premise's. A `⊤` below a `?`
/// or `!` makes the check useless altogether, since a copy of it absorbs
/// any imbalance; [`absorbs_from_copies`](Self::absorbs_from_copies) says
/// so.
///
/// **Weight.** In the multiplicative fragment with units, every cut-free
/// proof of a sequent of `c` formulas with `t` tensors, `p` pars, `u` ones
/// and `b` bottoms below them satisfies `c = t − p − u + b + 2`, or `≥` with
/// Mix, because every leaf is an axiom or a `1` and every `⊗` or Mix joins
/// two subproofs. The weight of an occurrence is `t − p − u + b` over its
/// subtree, so the equation is a sum over the members of a sequent. It is
/// unsound as soon as additives or exponentials appear: `⊢ a ⊕ b, ~a` is
/// provable with `b` unbalanced, and a `⊤` closes any sequent.
#[derive(Clone, Debug)]
pub(crate) struct Counts {
    /// Where the rows end in the entry arrays, in the order the rows were
    /// computed, the last occurrence's first: the row of occurrence `o` of
    /// `n` lies between the entries `n − 1 − o` and `n − o`. Every bound
    /// fits a `u32`: a set-up whose rows have more entries is refused.
    bound: Box<[u32]>,
    /// The row entries, one after another. A balance is a sum of ±1 over
    /// the literals below an occurrence, of which a forest that gets counts
    /// has fewer than 2³¹: within an `i32`.
    entries: Box<[RowEntry]>,
    /// Per occurrence, whether a `⊤` lies at or below it.
    absorbs: Box<[bool]>,
    /// Per occurrence, `t − p − u + b` over its subtree: at most its
    /// connectives in absolute value, fewer than 2³¹.
    weight: Box<[i32]>,
    /// The number of atoms that have rows, the width of a [`Tally`]: those
    /// with no literal below a `!` or `?`, and, where the interval check
    /// runs, those whose literals there all have one sign.
    row_atoms: usize,
    /// The atoms that have rows, by their rank.
    ranked: Box<[Atom]>,
    /// Per rank, the sign of the atom's literals below a `!` or `?`, all
    /// of which have it: copies can add any number of them, so the
    /// atom's balance can grow without bound that way, and only the other
    /// way excludes it. `None` for an atom with no literal there.
    copied: Box<[Option<Sign>]>,
    /// Whether some atom has copies of one sign (`copied`): without one,
    /// the loops over a row test a balance against zero alone and read no
    /// sign.
    one_sided: bool,
    /// Whether a `⊤` lies below a `?` or `!` somewhere in the problem, so
    /// that a copy can absorb any imbalance and the intervals prune
    /// nothing.
    absorbs_from_copies: bool,
    /// Per occurrence, what it forces on its side of a split as a factor
    /// of a `⊗`, without weakening: [`forced`](Self::forced).
    forced: Box<[Option<Forced>]>,
    /// Per atom, the sign of its positive literal in this search.
    positive: Box<[Sign]>,
}

/// An atom's rank among the atoms that have rows, in the order of the
/// sequent's atoms: its column in a [`Tally`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Rank(u32);

impl Rank {
    /// The rank as an index into a tally's columns.
    pub(crate) const fn index(self) -> usize {
        self.0 as usize
    }
}

/// One entry of a row: the atom's rank and the interval of its balance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RowEntry {
    /// The atom's rank.
    pub(crate) rank: Rank,
    /// The least balance.
    pub(crate) lo: i32,
    /// The greatest balance.
    pub(crate) hi: i32,
}

/// The rows of the occurrences computed so far, one after another.
#[derive(Default)]
struct Rows {
    /// The entries.
    entries: Vec<RowEntry>,
}

impl Rows {
    /// The entry at a place.
    fn at(&self, i: usize) -> RowEntry {
        self.entries[i]
    }

    /// Appends an entry.
    fn push(&mut self, e: RowEntry) {
        self.entries.push(e);
    }

    /// The bytes the entries' allocation takes.
    fn bytes(&self) -> usize {
        bytes_of(&self.entries)
    }

    /// Appends the merge of two rows that lie among the entries, each
    /// sorted by atom: the sum of the intervals for `⊗` and `⅋`, their
    /// hull for `&` and `⊕`, with an atom missing from one side
    /// contributing `0 ..= 0` there. Returns how many entries it read.
    fn merge(&mut self, a: std::ops::Range<usize>, b: std::ops::Range<usize>, sum: bool) -> usize {
        let (mut i, mut j) = (a.start, b.start);
        let zero = |rank| RowEntry { rank, lo: 0, hi: 0 };
        while i < a.end || j < b.end {
            let (x, y) = match (i < a.end, j < b.end) {
                (true, true) if self.at(i).rank == self.at(j).rank => {
                    i += 1;
                    j += 1;
                    (self.at(i - 1), self.at(j - 1))
                }
                (true, other) if !other || self.at(i).rank < self.at(j).rank => {
                    i += 1;
                    (self.at(i - 1), zero(self.at(i - 1).rank))
                }
                _ => {
                    j += 1;
                    (zero(self.at(j - 1).rank), self.at(j - 1))
                }
            };
            self.push(RowEntry {
                rank: x.rank,
                lo: if sum { x.lo + y.lo } else { x.lo.min(y.lo) },
                hi: if sum { x.hi + y.hi } else { x.hi.max(y.hi) },
            });
        }
        a.len() + b.len()
    }
}

impl Counts {
    /// Computes the rows and weights of every occurrence of a forest, and
    /// the literals that are positive under the bias given: those of a
    /// linear mode, which differ from an affine mode's only in rows the
    /// interval check does not read there.
    #[cfg(any(all(test, feature = "parse"), feature = "interactive"))]
    pub(crate) fn new(forest: &Forest, bias: Bias) -> Self {
        Self::new_until(
            forest,
            bias,
            Mode::CLASSICAL,
            &Account::new(None),
            &mut |_| false,
        )
        .expect("nothing stops it")
    }

    /// Computes the rows and weights of every occurrence of a forest, and
    /// the literals that are positive under the bias given, polling `stop`
    /// on a large forest once every so many occurrences and row entries
    /// visited, as the passes around it are polled between them: this one
    /// is several, and the longest of a search's set-up. The rows together
    /// can be quadratic in the forest (a nest of `⊗` and `⅋` over distinct
    /// atoms has a row as long as its subtree at every level), so their
    /// memory is charged to `account` as it grows. Returns the reason the
    /// set-up gave up: the condition fired, the account is over its bound,
    /// or the forest or the rows are more than the counts can index.
    pub(crate) fn new_until(
        forest: &Forest,
        bias: Bias,
        mode: Mode,
        account: &Account,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Result<Self, Reason> {
        /// How many occurrences are visited between two polls.
        const PERIOD: usize = 1 << 16;
        let mut visited = 0;
        let mut stopped = |steps: usize| {
            visited += steps;
            visited >= PERIOD && {
                visited = 0;
                set_up_stopped(forest, stop)
            }
        };
        let n = forest.len();
        // The balances and weights are sums over an occurrence's subtree.
        if n > i32::MAX as usize {
            return Err(Reason::IndexLimit);
        }
        let positive = super::bias::signs(forest, bias);
        // A literal below a `?` or `!` anywhere in the problem can be
        // copied or discarded any number of times: an atom with both signs
        // there can be balanced whatever its other literals, and gets no
        // row entries at all, and an atom with one sign there only lets
        // copies push its balance that way (`copied`).
        let num_atoms = forest.sequent().atom_names().len();
        let fixed = n * (size_of::<u32>() + size_of::<i32>() + 2 * size_of::<bool>())
            + num_atoms
                * (2 * size_of::<bool>()
                    + 2 * size_of::<u32>()
                    + size_of::<Sign>()
                    + size_of::<Option<Sign>>());
        account.charge(fixed);
        if account.over() {
            return Err(Reason::MemoryLimit {
                limit_bytes: account.limit(),
            });
        }
        // Per atom and sign, whether a literal of it lies below a `!` or `?`.
        let mut exponential = vec![[false; 2]; num_atoms];
        let mut absorbs_from_copies = false;
        // An occurrence's subtree is the ids that follow it, so one pass
        // that remembers where the outermost `!` or `?` it is inside ends
        // knows of every occurrence whether it lies below one: a walk of
        // each one's subtree was quadratic in a nest of them.
        let mut inside = 0;
        for o in forest.ids() {
            if o.index() < inside {
                if let (Some(atom), Some(sign)) = (forest.atom(o), forest.sign(o)) {
                    exponential[atom.index()][sign as usize] = true;
                }
                absorbs_from_copies |= forest.kind(o) == Kind::Top;
            }
            if matches!(forest.kind(o), Kind::Bang | Kind::Quest) {
                inside = inside.max(o.index() + forest.size(o) as usize);
            }
            if stopped(1) {
                return Err(Reason::Stopped);
            }
        }
        // Copies of one sign bound a balance only where the interval check
        // runs. Elsewhere such an atom gets no rows, which would only
        // reorder the split search, which sorts members by their rows.
        let one_sided = intervals_run(mode, absorbs_from_copies);
        #[cfg(test)]
        let one_sided = one_sided && ONE_SIDED.get();
        if !one_sided {
            for signs in &mut exponential {
                if signs.contains(&true) {
                    *signs = [true; 2];
                }
            }
        }
        // The atoms that have rows, numbered in their own order: a tally
        // is as wide as they are many, and a Petri net has tens of
        // thousands of atoms, nearly all of both signs below a `!`, which
        // have none: copies can move their balance either way.
        let mut rank = vec![0u32; num_atoms];
        let mut ranked = Vec::new();
        let mut copied = Vec::new();
        for (atom, rank) in rank.iter_mut().enumerate() {
            *rank = ranked.len() as u32;
            let [positive, negative] = exponential[atom];
            if !(positive && negative) {
                ranked.push(Atom::new(atom as u32));
                copied.push(match (positive, negative) {
                    (true, _) => Some(Sign::Atom),
                    (_, true) => Some(Sign::Dual),
                    _ => None,
                });
            }
        }
        let row_atoms = ranked.len();
        // The literals that a `?` can put into the unrestricted zone: those
        // directly under one.
        let mut unrestricted = vec![[false; 2]; num_atoms];
        for o in forest.ids() {
            if forest.kind(o) == Kind::Quest
                && let Some(below) = forest.left(o)
                && let (Some(atom), Some(sign)) = (forest.atom(below), forest.sign(below))
            {
                unrestricted[atom.index()][sign as usize] = true;
            }
        }
        // A positive literal whose dual only ever lies in the linear zone.
        let linear = |o: OccId| match (forest.atom(o), forest.sign(o)) {
            (Some(atom), Some(sign)) => {
                sign == positive[atom.index()] && !unrestricted[atom.index()][!sign as usize]
            }
            _ => false,
        };
        let mut forced = vec![None; n];
        // Every descendant has a larger id than its ancestor, so a pass from
        // the last id down sees the children before the parent, and a row
        // is merged from two that are written already.
        let mut rows = Rows::default();
        let mut bound = Vec::with_capacity(n + 1);
        bound.push(0u32);
        let row = |bound: &[u32], o: OccId| {
            let at = n - 1 - o.index();
            bound[at] as usize..bound[at + 1] as usize
        };
        let mut absorbs = vec![false; n];
        let mut weight = vec![0i32; n];
        let mut charged = 0;
        for o in forest.ids().rev() {
            use Kind::*;
            let kind = forest.kind(o);
            let mut steps = 1;
            let (absorb, w) = match kind {
                Atom | DualAtom => {
                    let sign = if kind == Atom { 1 } else { -1 };
                    let atom = forest.atom(o).unwrap();
                    // A literal below an exponential has its entry too: a
                    // copy of it in the linear zone is used once, and the
                    // rows of a `!` or `?` itself stay empty.
                    if !exponential[atom.index()].iter().all(|&below| below) {
                        rows.push(RowEntry {
                            rank: Rank(rank[atom.index()]),
                            lo: sign,
                            hi: sign,
                        });
                    }
                    (false, 0)
                }
                One => (false, -1),
                Bot => (false, 1),
                Top => (true, 0),
                Zero => (false, 0),
                Forall | Exists => {
                    unreachable!("the focused engine refuses a sequent with a binder")
                }
                Bang | Quest => {
                    // No row: the literals below it are copied any number
                    // of times, which `copied` accounts for.
                    let c = forest.left(o).unwrap();
                    (absorbs[c.index()], weight[c.index()])
                }
                Tensor | Par | With | Plus => {
                    let (l, r) = (forest.left(o).unwrap(), forest.right(o).unwrap());
                    let (li, ri) = (l.index(), r.index());
                    let absorb = absorbs[li] || absorbs[ri];
                    if !absorb {
                        let sum = matches!(kind, Tensor | Par);
                        steps += rows.merge(row(&bound, l), row(&bound, r), sum);
                    }
                    let w = weight[li]
                        + weight[ri]
                        + if kind == Tensor {
                            1
                        } else if kind == Par {
                            -1
                        } else {
                            0
                        };
                    (absorb, w)
                }
            };
            let Ok(end) = u32::try_from(rows.entries.len()) else {
                return Err(Reason::IndexLimit);
            };
            bound.push(end);
            absorbs[o.index()] = absorb;
            weight[o.index()] = w;
            forced[o.index()] = match kind {
                Zero => Some(Forced::Nothing),
                One | Bang => Some(Forced::Empty),
                Atom | DualAtom
                    if forest.sign(o) == Some(positive[forest.atom(o).unwrap().index()]) =>
                {
                    Some(Forced::Dual)
                }
                Tensor
                    if forest
                        .children(o)
                        .all(|c| linear(c) || forced[c.index()] == Some(Forced::Duals)) =>
                {
                    Some(Forced::Duals)
                }
                Atom | DualAtom | Bot | Top | Quest | Tensor | Par | With | Plus => None,
                Forall | Exists => {
                    unreachable!("the focused engine refuses a sequent with a binder")
                }
            };
            if rows.bytes() != charged {
                account.resize(charged, rows.bytes());
                charged = rows.bytes();
                if account.over() {
                    return Err(Reason::MemoryLimit {
                        limit_bytes: account.limit(),
                    });
                }
            }
            if stopped(steps) {
                return Err(Reason::Stopped);
            }
        }
        let counts = Self {
            bound: bound.into_boxed_slice(),
            entries: rows.entries.into_boxed_slice(),
            absorbs: absorbs.into_boxed_slice(),
            weight: weight.into_boxed_slice(),
            row_atoms,
            ranked: ranked.into_boxed_slice(),
            one_sided: copied.iter().any(Option::is_some),
            copied: copied.into_boxed_slice(),
            absorbs_from_copies,
            forced: forced.into_boxed_slice(),
            positive,
        };
        account.resize(charged, counts.entries.len() * size_of::<RowEntry>());
        Ok(counts)
    }

    /// The places of an occurrence's row in the entry arrays.
    fn span(&self, o: OccId) -> (usize, usize) {
        let at = self.absorbs.len() - 1 - o.index();
        (self.bound[at] as usize, self.bound[at + 1] as usize)
    }

    /// Returns whether the occurrence is a positive literal: the literal
    /// of its atom that the bias of the search makes positive.
    pub(crate) fn positive(&self, forest: &Forest, o: OccId) -> bool {
        match (forest.atom(o), forest.sign(o)) {
            (Some(atom), Some(sign)) => sign == self.positive[atom.index()],
            _ => false,
        }
    }

    /// Returns what the occurrence forces on its side of a split as a
    /// factor of a `⊗`, without weakening: nothing at all for `0`, the
    /// empty context for `1` and `!`, the dual literal alone for a positive
    /// literal, and one dual per literal for a tensor of positive literals
    /// none of whose duals can lie in the unrestricted zone (none is
    /// directly under a `?`), which in focus is proved by exactly one dual
    /// per literal, each from the linear zone.
    pub(super) fn forced(&self, o: OccId) -> Option<Forced> {
        self.forced[o.index()]
    }

    /// Returns the row of an occurrence: its interval per atom occurring
    /// below it, in ascending atom order. Empty when the occurrence absorbs.
    pub(crate) fn row(&self, o: OccId) -> impl Iterator<Item = RowEntry> + '_ {
        let (start, end) = self.span(o);
        self.entries[start..end].iter().copied()
    }

    /// Returns whether a `⊤` lies at or below the occurrence, so that any
    /// sequent containing it passes the interval check.
    pub(crate) fn absorbs(&self, o: OccId) -> bool {
        self.absorbs[o.index()]
    }

    /// Returns `t − p − u + b` over the occurrence's subtree.
    pub(crate) fn weight(&self, o: OccId) -> i32 {
        self.weight[o.index()]
    }

    /// Returns whether a `⊤` lies below a `?` or `!` somewhere in the
    /// problem: a copy from the unrestricted zone can then absorb any
    /// imbalance, so the interval check is unsound.
    pub(crate) fn absorbs_from_copies(&self) -> bool {
        self.absorbs_from_copies
    }

    /// Returns the rank of the first atom of the occurrence's row, or
    /// `u32::MAX` when the row is empty: members of a context sorted by it
    /// have every atom's members next to each other.
    pub(crate) fn first_rank(&self, o: OccId) -> u32 {
        let (start, end) = self.span(o);
        if start == end {
            u32::MAX
        } else {
            self.entries[start].rank.0
        }
    }

    /// Returns the number of atoms in the occurrence's row.
    pub(crate) fn row_len(&self, o: OccId) -> u32 {
        let (start, end) = self.span(o);
        (end - start) as u32
    }

    /// Returns the bytes a [`Tally`] of this forest allocates.
    pub(crate) fn tally_bytes(&self) -> usize {
        self.row_atoms * size_of::<Sum>()
    }

    /// Returns the bytes the counts of a [`Split`] of this forest take.
    pub(crate) fn split_bytes(&self) -> usize {
        size_of::<Split>() + self.row_atoms * size_of::<Column>()
    }

    /// Returns the counts of a split with no member yet.
    pub(crate) fn split(&self) -> Split {
        Split {
            columns: vec![Column::default(); self.row_atoms],
            bad: [0; 2],
            absorbers: [0; 2],
            open_absorbers: 0,
            slack: [-2; 2],
            slack_below: 0,
            slack_above: 0,
            inert: false,
            touched: Vec::new(),
        }
    }

    /// Returns whether the occurrence's row has an interval that excludes
    /// zero by itself: only such a member can make a set fail the interval
    /// check.
    pub(crate) fn tight(&self, o: OccId) -> bool {
        self.row(o)
            .any(|e| self.excluded(e.rank.index(), e.lo, e.hi))
    }

    /// Returns whether the balance of the atom of rank `rank`, between
    /// `lo` and `hi` over a set's rows, rules a proof of the set out: zero
    /// lies outside it, and the copies of the atom's literals below an
    /// exponential, which only add literals of their one sign, cannot
    /// bring it there.
    pub(crate) fn excluded(&self, rank: usize, lo: i32, hi: i32) -> bool {
        match self.copied[rank] {
            None => lo > 0 || hi < 0,
            Some(Sign::Atom) => lo > 0,
            Some(Sign::Dual) => hi < 0,
        }
    }

    /// Returns what [`excluded`](Self::excluded) does, `PLAIN` saying that
    /// no atom has copies of one sign, so that a loop over a row decides
    /// once whether to read their signs at all.
    #[inline(always)]
    fn excluded_as<const PLAIN: bool>(&self, rank: usize, lo: i32, hi: i32) -> bool {
        if PLAIN {
            lo > 0 || hi < 0
        } else {
            self.excluded(rank, lo, hi)
        }
    }

    /// Returns an empty tally, a column per atom that has rows.
    pub(crate) fn tally(&self) -> Tally {
        Tally {
            sums: vec![Sum::default(); self.row_atoms],
            touched: Vec::new(),
            bad: 0,
            absorbers: 0,
            len: 0,
            weight: 0,
        }
    }
}

/// The running sums of a set of occurrences: the summed interval per atom,
/// how many atoms' intervals exclude zero, how many members absorb, and the
/// member count and summed weight the `MLL` equation compares. Members are
/// added and removed one at a time, at the cost of their row.
#[derive(Clone, Debug)]
pub(crate) struct Tally {
    /// Per atom, the summed interval.
    sums: Vec<Sum>,
    /// The atoms whose sums a row found at zero since the tally was last
    /// cleared: the only atoms whose sums can be other than zero, each
    /// listed again only after a row left its sums at zero. An atom listed
    /// at every row would make the list grow, uncharged, with the copies
    /// of a formula in the linear zone, a branch's worth of them.
    touched: Vec<u32>,
    /// The number of atoms whose summed interval excludes zero.
    bad: u32,
    /// The number of members that absorb.
    absorbers: u32,
    /// The number of members.
    len: u32,
    /// The summed weight of the members.
    weight: i32,
}

impl Tally {
    /// Removes every member, at the cost of the rows that came in since the
    /// last time and not of the sequent's atoms.
    pub(crate) fn clear(&mut self) {
        for a in self.touched.drain(..) {
            self.sums[a as usize] = Sum::default();
        }
        self.bad = 0;
        self.absorbers = 0;
        self.len = 0;
        self.weight = 0;
    }

    /// Adds a member.
    pub(crate) fn add(&mut self, counts: &Counts, o: OccId) {
        self.apply(counts, o, 1);
    }

    /// Removes a member.
    #[cfg(all(test, feature = "parse"))]
    pub(crate) fn remove(&mut self, counts: &Counts, o: OccId) {
        self.apply(counts, o, -1);
    }

    /// Adds (`sign` 1) or removes (`sign` −1) a member's row, weight and
    /// absorption.
    fn apply(&mut self, counts: &Counts, o: OccId, sign: i32) {
        if counts.one_sided {
            self.apply_as::<false>(counts, o, sign);
        } else {
            self.apply_as::<true>(counts, o, sign);
        }
    }

    /// Does what [`apply`](Self::apply) does, `PLAIN` as in
    /// [`Counts::excluded_as`].
    #[inline(always)]
    fn apply_as<const PLAIN: bool>(&mut self, counts: &Counts, o: OccId, sign: i32) {
        for e in counts.row(o) {
            let a = e.rank.index();
            let sum = &mut self.sums[a];
            if *sum == Sum::default() {
                self.touched.push(a as u32);
            }
            let was_bad = counts.excluded_as::<PLAIN>(a, sum.lo, sum.hi);
            sum.lo += sign * e.lo;
            sum.hi += sign * e.hi;
            let is_bad = counts.excluded_as::<PLAIN>(a, sum.lo, sum.hi);
            if is_bad && !was_bad {
                self.bad += 1;
            } else if was_bad && !is_bad {
                self.bad -= 1;
            }
        }
        if counts.absorbs(o) {
            self.absorbers = self.absorbers.wrapping_add_signed(sign);
        }
        self.len = self.len.wrapping_add_signed(sign);
        self.weight += sign * counts.weight(o);
    }

    /// Returns an atom whose summed interval rules a proof out
    /// ([`Counts::excluded`]), the first in the sequent's order, with that
    /// interval and the sign its copies add, unless a member absorbs.
    pub(crate) fn unbalanced(&self, counts: &Counts) -> Option<(Atom, i32, i32, Option<Sign>)> {
        if self.absorbs() {
            return None;
        }
        let rank = self
            .touched
            .iter()
            .map(|&a| a as usize)
            .filter(|&a| counts.excluded(a, self.sums[a].lo, self.sums[a].hi))
            .min()?;
        Some((
            counts.ranked[rank],
            self.sums[rank].lo,
            self.sums[rank].hi,
            counts.copied[rank],
        ))
    }

    /// Returns whether a member absorbs, that is, has a `⊤` below it.
    pub(crate) fn absorbs(&self) -> bool {
        self.absorbers > 0
    }

    /// Returns whether the members pass the interval check: one of them
    /// absorbs, or zero lies in the summed interval of every atom.
    pub(crate) fn balanced(&self) -> bool {
        self.absorbs() || self.bad == 0
    }

    /// Returns whether the members pass the `MLL` count equation
    /// `c = t − p − u + b + 2`, as an equality or, with Mix, as `≥`. Only
    /// meaningful for members without additives or exponentials below them.
    pub(crate) fn equation(&self, mix: bool) -> bool {
        let c = i64::from(self.len);
        let rhs = i64::from(self.weight) + 2;
        if mix { c >= rhs } else { c == rhs }
    }

    /// Returns whether the `MLL` count equation leaves room for a Mix among
    /// the members: `c > t − p − u + b + 2`, since every Mix adds two to the
    /// left side.
    pub(crate) fn admits_mix(&self) -> bool {
        i64::from(self.len) > i64::from(self.weight) + 2
    }

    /// Returns the number of members.
    #[cfg(all(test, feature = "parse"))]
    pub(crate) fn len(&self) -> u32 {
        self.len
    }
}

/// The counts of a split of a context in the making: the sums of the
/// members each side has so far, and what the members not yet assigned to
/// a side can still add to either. A side whose sums no assignment of the
/// open members can bring to pass the interval check or the count equation
/// makes the partial split infeasible, so a search over the members cuts
/// it there; with no member open, [`feasible`](Self::feasible) is the test
/// of a whole split: both sides pass the interval check and the equation.
///
/// The bounds are per atom and per side, each on its own: an open member
/// can lower a side's least balance of an atom by the negative part of its
/// own, and raise the greatest by the positive part of its own. They are
/// necessary conditions, never sufficient.
#[derive(Clone, Debug)]
pub(crate) struct Split {
    /// Per atom, the sides' sums and what the open members can add.
    columns: Vec<Column>,
    /// Per side, the number of atoms whose interval excludes zero whatever
    /// the open members do.
    bad: [u32; 2],
    /// Per side, the number of members that absorb.
    absorbers: [u32; 2],
    /// The number of open members that absorb.
    open_absorbers: u32,
    /// Per side, `c − (t − p − u + b) − 2` over its members, which the
    /// count equation wants zero, or at least zero with Mix.
    slack: [i64; 2],
    /// The sum of the negative contributions of the open members to a
    /// side's slack.
    slack_below: i64,
    /// The sum of their positive contributions.
    slack_above: i64,
    /// Whether no prune can cut any assignment of these members, so that
    /// the counts need not follow the search.
    inert: bool,
    /// The atoms whose sums the row of a member placed or opened found at
    /// zero since the counts were last cleared: the only atoms whose sums
    /// can be other than zero, listed as the tally's are.
    touched: Vec<u32>,
}

/// An atom's summed interval in a [`Tally`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Sum {
    /// The summed least balance.
    lo: i32,
    /// The summed greatest balance.
    hi: i32,
}

/// An atom's sums in a [`Split`], kept together since a step reads and
/// writes them all.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Column {
    /// Per side, the summed least balance of the side's members.
    lo: [i32; 2],
    /// Per side, the summed greatest balance of the side's members.
    hi: [i32; 2],
    /// The sum of the negative least balances of the open members: the
    /// most they can lower a side's least balance.
    below: i32,
    /// The sum of the positive greatest balances of the open members: the
    /// most they can raise a side's greatest balance.
    above: i32,
}

impl Column {
    /// Per side, whether no assignment of the open members can bring the
    /// side's interval of the atom of rank `atom` to contain zero, or the
    /// atom's copies to balance it ([`Counts::excluded`]).
    #[inline(always)]
    fn excludes<const PLAIN: bool>(&self, counts: &Counts, atom: usize) -> [bool; 2] {
        [0, 1].map(|side| {
            counts.excluded_as::<PLAIN>(
                atom,
                self.lo[side] + self.below,
                self.hi[side] + self.above,
            )
        })
    }
}

impl Split {
    /// Brings the sides' counts of excluded atoms up to date after a step
    /// moved an atom's sums from excluding as `was` to excluding as `is`.
    #[inline(always)]
    fn count_bad(&mut self, was: [bool; 2], is: [bool; 2]) {
        for side in 0..2 {
            match (was[side], is[side]) {
                (false, true) => self.bad[side] += 1,
                (true, false) => self.bad[side] -= 1,
                _ => {}
            }
        }
    }

    /// Removes every member, at the cost of the rows that came in since the
    /// last time and not of the sequent's atoms: a Petri net has thousands
    /// of atoms and a split touches a few.
    pub(crate) fn clear(&mut self) {
        for a in self.touched.drain(..) {
            self.columns[a as usize] = Column::default();
        }
        self.bad = [0; 2];
        self.absorbers = [0; 2];
        self.open_absorbers = 0;
        self.slack = [-2; 2];
        self.slack_below = 0;
        self.slack_above = 0;
        self.inert = false;
    }

    /// Says, once every member is placed or open, whether no prune in
    /// force can cut any assignment of them: [`feasible`](Self::feasible)
    /// then answers yes without the counts, which stop following the
    /// members' moves.
    pub(crate) fn set_inert(&mut self, inert: bool) {
        self.inert = inert;
    }

    /// Moves a member between the open ones and a side, or adds it to
    /// either, in one pass over its row: `side` gains it `settled` times
    /// (1, 0 or −1) and the open members gain it `opened` times, and the
    /// sides' counts of excluded atoms are brought up to date.
    fn shift(&mut self, counts: &Counts, o: OccId, side: Branch, settled: i32, opened: i32) {
        if counts.one_sided {
            self.shift_as::<false>(counts, o, side, settled, opened);
        } else {
            self.shift_as::<true>(counts, o, side, settled, opened);
        }
    }

    /// Does what [`shift`](Self::shift) does, `PLAIN` as in
    /// [`Counts::excluded_as`].
    #[inline(always)]
    fn shift_as<const PLAIN: bool>(
        &mut self,
        counts: &Counts,
        o: OccId,
        side: Branch,
        settled: i32,
        opened: i32,
    ) {
        let side = side as usize;
        for e in counts.row(o) {
            let a = e.rank.index();
            let column = &mut self.columns[a];
            let was = column.excludes::<PLAIN>(counts, a);
            column.lo[side] += settled * e.lo;
            column.hi[side] += settled * e.hi;
            column.below += opened * e.lo.min(0);
            column.above += opened * e.hi.max(0);
            let is = column.excludes::<PLAIN>(counts, a);
            self.count_bad(was, is);
        }
        if counts.absorbs(o) {
            self.absorbers[side] = self.absorbers[side].wrapping_add_signed(settled);
            self.open_absorbers = self.open_absorbers.wrapping_add_signed(opened);
        }
        let slack = 1 - i64::from(counts.weight(o));
        self.slack[side] += i64::from(settled) * slack;
        self.slack_below += i64::from(opened) * slack.min(0);
        self.slack_above += i64::from(opened) * slack.max(0);
    }

    /// Adds a member to a side for good: a subformula of the `⊗`, or a
    /// member the search does not move.
    pub(crate) fn place(&mut self, counts: &Counts, o: OccId, side: Branch) {
        self.touch(counts, o);
        self.shift(counts, o, side, 1, 0);
    }

    /// Adds a member that a search will assign to a side.
    pub(crate) fn open(&mut self, counts: &Counts, o: OccId) {
        self.touch(counts, o);
        self.shift(counts, o, Branch::Left, 0, 1);
    }

    /// Lists the atoms of a member's row whose sums are all zero, before
    /// the member comes in.
    fn touch(&mut self, counts: &Counts, o: OccId) {
        for e in counts.row(o) {
            if self.columns[e.rank.index()] == Column::default() {
                self.touched.push(e.rank.0);
            }
        }
    }

    /// Assigns an open member to a side.
    pub(crate) fn assign(&mut self, counts: &Counts, o: OccId, side: Branch) {
        if !self.inert {
            self.shift(counts, o, side, 1, -1);
        }
    }

    /// Moves an assigned member from the other side to this one.
    pub(crate) fn flip(&mut self, counts: &Counts, o: OccId, side: Branch) {
        if self.inert {
            return;
        }
        if counts.one_sided {
            self.flip_as::<false>(counts, o, side);
        } else {
            self.flip_as::<true>(counts, o, side);
        }
    }

    /// Does what [`flip`](Self::flip) does, `PLAIN` as in
    /// [`Counts::excluded_as`].
    #[inline(always)]
    fn flip_as<const PLAIN: bool>(&mut self, counts: &Counts, o: OccId, side: Branch) {
        let (to, from) = (side as usize, 1 - side as usize);
        for e in counts.row(o) {
            let a = e.rank.index();
            let column = &mut self.columns[a];
            let was = column.excludes::<PLAIN>(counts, a);
            column.lo[from] -= e.lo;
            column.hi[from] -= e.hi;
            column.lo[to] += e.lo;
            column.hi[to] += e.hi;
            let is = column.excludes::<PLAIN>(counts, a);
            self.count_bad(was, is);
        }
        if counts.absorbs(o) {
            self.absorbers[from] -= 1;
            self.absorbers[to] += 1;
        }
        let slack = 1 - i64::from(counts.weight(o));
        self.slack[from] -= slack;
        self.slack[to] += slack;
    }

    /// Takes an assigned member back from its side: it is open again.
    pub(crate) fn unassign(&mut self, counts: &Counts, o: OccId, side: Branch) {
        if !self.inert {
            self.shift(counts, o, side, -1, 1);
        }
    }

    /// Returns whether some assignment of the open members may still pass
    /// the prunes in force on both sides: the interval check (a side
    /// passes with an absorbing member, which each open one can give to
    /// one side only) and the count equation, `≥` with Mix. With no open
    /// member, whether the split passes them.
    pub(crate) fn feasible(&self, intervals: bool, equation: bool, mix: bool) -> bool {
        if self.inert {
            return true;
        }
        if intervals {
            let needy = (0..2)
                .filter(|&side| self.absorbers[side] == 0 && self.bad[side] > 0)
                .count();
            if needy > self.open_absorbers as usize {
                return false;
            }
        }
        !equation
            || self.slack.iter().all(|slack| {
                slack + self.slack_above >= 0 && (mix || slack + self.slack_below <= 0)
            })
    }
}

/// Returns why a goal that the search refuted is unprovable, as far as its
/// counts tell: an atom whose literals cannot pair up, or the count
/// equation that fails, each under the rules the engine searched the
/// fragment and the mode with; else that the search was exhaustive. The
/// asynchronous phase keeps the sums of the goal's members (a `⅋` or `⊥`
/// keeps the count equation, a premise of `&` lies within its hull), so
/// a goal that fails a test makes every stable sequent above it fail it,
/// where the engine checks it. Counting is a pass over the forest under
/// the search's limits; a pass given up says only what the verdict does.
pub(crate) fn refutation(
    forest: &Forest,
    goal: &[OccId],
    fragment: Fragment,
    mode: Mode,
    account: &Account,
    stop: &mut dyn FnMut(u64) -> bool,
) -> Refutation {
    let Ok(counts) = Counts::new_until(forest, Bias::Rarer, mode, account, stop) else {
        return Refutation::Exhausted;
    };
    let rules = Switches::new(fragment, mode, &counts);
    let mut tally = counts.tally();
    for &o in goal {
        tally.add(&counts, o);
    }
    if rules.intervals
        && let Some((atom, least, most, copied)) = tally.unbalanced(&counts)
    {
        return match copied {
            None => Refutation::Unbalanced(Unbalanced { atom, least, most }),
            Some(copied) => Refutation::OneSided(OneSided {
                atom,
                least,
                most,
                copied,
            }),
        };
    }
    if rules.equation && !tally.equation(rules.mix) {
        let (mut tensors, mut pars, mut ones, mut bottoms) = (0, 0, 0, 0);
        for &o in goal {
            for x in forest.subtree(o) {
                match forest.kind(x) {
                    Kind::Tensor => tensors += 1,
                    Kind::Par => pars += 1,
                    Kind::One => ones += 1,
                    Kind::Bot => bottoms += 1,
                    _ => {}
                }
            }
        }
        // Each count is below 2³², so the sum is exact.
        let needed = tensors as i64 - pars as i64 - ones as i64 + bottoms as i64 + 2;
        return Refutation::Equation(Equation {
            formulas: goal.len() as u64,
            needed,
            tensors,
            pars,
            ones,
            bottoms,
            mix: rules.mix,
        });
    }
    Refutation::Exhausted
}

/// Whether a split of a goal into the two premises of a `⊗`, each given
/// with its subformula of the `⊗`, or of a Mix passes the count prunes the
/// engine applies to every split: the interval check per atom and, in the
/// multiplicative fragments, the count equation. A split that fails cannot
/// close; one that passes may still fail. `fragment` is the goal's.
#[cfg(feature = "interactive")]
pub(crate) fn split_passes(
    forest: &Forest,
    fragment: Fragment,
    mode: Mode,
    left: &[OccId],
    right: &[OccId],
) -> bool {
    let counts = Counts::new(forest, Bias::Auto);
    let rules = Switches::new(fragment, mode, &counts);
    let mut split = counts.split();
    for (members, side) in [(left, Branch::Left), (right, Branch::Right)] {
        for &m in members {
            split.place(&counts, m, side);
        }
    }
    split.feasible(rules.intervals, rules.equation, rules.mix)
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;

    /// Builds the forest and counts of `input`.
    fn counts(input: &str) -> (Forest, Counts) {
        let s: Sequent = input.parse().unwrap_or_else(|e| panic!("{input:?}: {e}"));
        let f = Forest::new(&s).unwrap();
        let c = Counts::new(&f, Bias::Auto);
        (f, c)
    }

    /// Tallies every root of the sequent.
    fn roots(input: &str) -> (Forest, Counts, Tally) {
        let (f, c) = counts(input);
        let mut t = c.tally();
        for &r in f.roots() {
            t.add(&c, r);
        }
        (f, c, t)
    }

    /// A tally and a split list an atom once while its sums are off zero,
    /// however often a member of its row comes in, as the copies of one
    /// formula do.
    #[test]
    fn copies_list_their_atoms_once() {
        let (f, c) = counts("|- ?(a & 1), ~a");
        let copy = f.left(f.roots()[0]).unwrap();
        let mut tally = c.tally();
        let mut split = c.split();
        for _ in 0..100 {
            tally.add(&c, copy);
            split.place(&c, copy, Branch::Left);
        }
        assert_eq!((tally.touched.len(), split.touched.len()), (1, 1));
    }

    /// An atom whose literals below the exponentials all have one sign
    /// gets rows only where the interval check runs: in linear mode and
    /// with no `⊤` below an exponential.
    #[test]
    fn rows_of_one_sign_only_where_the_check_runs() {
        let ranked = |input: &str, mode: Mode| {
            let s: Sequent = input.parse().unwrap();
            let f = Forest::new(&s).unwrap();
            let c = Counts::new_until(&f, Bias::Auto, mode, &Account::new(None), &mut |_| false)
                .unwrap();
            c.ranked.len()
        };
        assert_eq!(ranked("|- ?a, a", Mode::CLASSICAL), 1);
        assert_eq!(ranked("|- ?a, a", Mode::INTUITIONISTIC), 1);
        assert_eq!(ranked("|- ?a, a", Mode::CLASSICAL.with_affine()), 0);
        assert_eq!(ranked("|- ?(a & top), a", Mode::CLASSICAL), 0);
        assert_eq!(ranked("|- ?a, ?~a, a", Mode::CLASSICAL), 0);
    }

    /// Rows sum under the multiplicatives, take the hull under the
    /// additives, and are sparse over the atoms below.
    #[test]
    fn rows() {
        // ⊢ (a ⊗ ~a) ⅋ b, (a & ~b) ⊕ ~a, 1
        let (f, c) = counts("|- (a * ~a) par b, (a & ~b) + ~a, 1");
        // Without exponentials every atom has rows, ranked as it is
        // numbered.
        let rank = |name| Rank(f.sequent().atom(name).unwrap().index() as u32);
        let (a, b) = (rank("a"), rank("b"));
        let row = |o: u32| c.row(OccId::new(o)).collect::<Vec<_>>();
        let e = |rank, lo, hi| RowEntry { rank, lo, hi };
        assert_eq!(row(0), [e(a, 0, 0), e(b, 1, 1)], "the ⅋");
        assert_eq!(row(1), [e(a, 0, 0)], "the ⊗");
        assert_eq!(row(2), [e(a, 1, 1)]);
        assert_eq!(row(3), [e(a, -1, -1)]);
        assert_eq!(row(5), [e(a, -1, 1), e(b, -1, 0)], "the ⊕");
        assert_eq!(row(6), [e(a, 0, 1), e(b, -1, 0)], "the &");
        assert_eq!(row(10), [], "the 1");
        assert!(c.tight(OccId::new(2)), "a literal's interval excludes zero");
        assert!(!c.tight(OccId::new(5)), "the ⊕ can be balanced by itself");
        assert!(!c.absorbs(OccId::new(0)));
        assert_eq!(c.weight(OccId::new(0)), 0, "one ⊗, one ⅋");
        assert_eq!(c.weight(OccId::new(10)), -1);
    }

    /// The interval check accepts the balanced, rejects the unbalanced, and
    /// lets `⊢ a ⊕ b, ~a` through where the plain balance would not.
    #[test]
    fn intervals() {
        for (input, balanced) in [
            ("|- a, ~a", true),
            ("|- a, a", false),
            ("|- a, ~a, b", false),
            ("|- a + b, ~a", true),
            ("|- a & b, ~a", true),
            ("|- (a * b) + (a * c), ~a, ~b", true),
            ("|- (a * b) + (a * c), ~a, ~c, ~c", false),
            ("|- 1, bot, 0", true),
            ("|- a", false),
            ("|-", true),
        ] {
            let (_, _, t) = roots(input);
            assert_eq!(t.balanced(), balanced, "{input:?}");
        }
    }

    /// A `⊤` anywhere below a member absorbs any imbalance.
    #[test]
    fn top_absorbs() {
        for input in [
            "|- top, a",
            "|- top * a, b",
            "|- (a & top) + c, ~b",
            "|- top par a, b",
            "|- a & top, ~b",
            "|- top & a, ~b",
        ] {
            let (f, c, t) = roots(input);
            assert!(c.absorbs(f.roots()[0]), "{input:?}");
            assert!(t.balanced(), "{input:?}");
        }
    }

    /// The `MLL` equation accepts exactly the spec's unit checks, as an
    /// equality without Mix and as an inequality with it.
    #[test]
    fn equation() {
        for (input, holds, holds_with_mix) in [
            ("|- 1", true, true),
            ("|- bot, 1", true, true),
            ("|- 1 * 1", true, true),
            ("|- bot par 1", true, true),
            ("|- bot par bot", false, false),
            ("|- a, ~a", true, true),
            ("|- a * b, ~a, ~b", true, true),
            ("|- a par b, ~a, ~b", false, true),
            ("|- a, ~a, b, ~b", false, true),
            ("|- 1, a, ~a", false, true),
            ("|-", false, false),
        ] {
            let (_, _, t) = roots(input);
            assert_eq!(t.equation(false), holds, "{input:?}");
            assert_eq!(t.equation(true), holds_with_mix, "{input:?}");
        }
    }

    /// Members go in and out of a tally, and the counts follow.
    #[test]
    fn tally_moves() {
        let (f, c, mut t) = roots("|- a * b, ~a, ~b");
        assert!(t.balanced());
        assert_eq!(t.len(), 3);
        t.remove(&c, f.roots()[1]);
        assert!(!t.balanced(), "~a alone is unbalanced");
        assert_eq!(t.len(), 2);
        t.remove(&c, f.roots()[0]);
        t.remove(&c, f.roots()[2]);
        assert!(t.balanced(), "the empty set is balanced");
        assert!(!t.equation(false));
        t.add(&c, f.roots()[1]);
        t.add(&c, f.roots()[2]);
        assert!(!t.balanced());
        t.clear();
        assert!(t.balanced());
        assert_eq!(t.len(), 0);
    }
}
