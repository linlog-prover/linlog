// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The failures that wait on an open ancestor. A stable sequent whose
//! failure rests on a prune of the loop check is no fact about the
//! sequent: it holds only while the ancestor it repeated is on the
//! branch, and the memo never takes it. Searched again, such a sequent
//! was searched to the end every time; under Mix, where a part of a
//! sequent below a copy is often an ancestor, that was nearly every
//! failure (`|- ?((((!~a par c) + c) par a) par ~c) par ?((~c * ~c) * c)`
//! at a copy bound of 3: 5.8 million decisions of 603 sequents). So such
//! a failure waits here instead, under its *leader*, the shallowest
//! ancestor it rests on, which is open: it answers for its sequent while
//! the leader stays open, and goes into the memo as a fact once the
//! leader fails without a dependency of its own.
//!
//! The argument, which every rule below keeps. A failure of `S` at
//! budget `b` with dependency `d` says: every proof of `S` (within `b`
//! where the failure was cut by the budget, at any budget where it was
//! not) contains, strictly above its root, a proof of a member of a set
//! `W(S)` of stable sequents, within that member's budget where it is a
//! failure cut by the budget: an ancestor at depth `d` or deeper whose
//! repeat was pruned, or a sequent whose failure with a dependency the
//! search of `S` rested on (a premise searched, or a failure reused from
//! here). Budgets only shrink along a branch, and a failure cut by the
//! budget is reused only within its budget, which is why the members'
//! budgets hold. The close of a stable sequent `Y` keeps the failures
//! that wait true, in whatever branch they are reused:
//!
//! - `Y` proved: every failure decided below `Y` is dropped, since it may
//!   rest on a prune of `Y`.
//! - `Y` failed with a cut by the budget: every failure below it becomes
//!   one cut by the budget at its own budget, since it may rest on a
//!   prune of `Y`, whose failure says nothing beyond `Y`'s budget.
//! - `Y` failed with a dependency `d` above it: every failure below it
//!   that waits on a leader deeper than `d` waits on `d`, since it may
//!   rest on `Y`, which rests on `d`; on a worker of a pool whose `d`
//!   lies in the spawning engine's branch, they are dropped instead.
//! - `Y` failed with no dependency: the failures below it whose leader is
//!   `Y` or deeper go into the memo as facts. Take every failure decided
//!   below `Y` with a dependency on `Y` or deeper, after the rules above,
//!   and below no stable sequent that was proved, whether it waits here
//!   or not (one the cap kept out is among them: the rules above ran on
//!   the failures below it all the same); each of their `W` members is
//!   `Y`, one of them, or a failure the memo holds as a fact (a member
//!   below a proved sequent would have dropped them). Of these failures
//!   take the one with the smallest proof that it denies, if any had one:
//!   that proof contains a proof of a member within the member's budget,
//!   which is `Y` (unprovable), a fact, or one of them with a smaller
//!   proof. So none had one.
//!
//! The leader of every failure here is open, so it is an ancestor of
//! every sequent the search visits until it closes, and so is every
//! member of `W` still open; a member closed since failed, and its own
//! statement rests on members at the leader's depth or deeper (its
//! dependency moved the leader up), open or closed with a failure in
//! turn. So a proof that a reuse in another branch below the leader
//! denies would contain one of an open ancestor at that depth or deeper,
//! which the reuse's dependency, the leader's depth, says; the search
//! settles it no earlier than at the leader. The marks hold this up: a
//! close settles the failures from its stable sequent's mark on, and
//! `Run::relieve`, which drops every failure, sets the marks of the
//! branch to the start, since what waits after it was decided below all
//! of them. Below one leader each sequent is then decided about once per
//! budget: that sequent's search visited 17 179 stable sequents at the
//! copy bound of 3 instead of 76.5 million.

use crate::search::focus::memo::{Key, Zones};

/// A position in [`Waiting::entries`] that names no entry.
const NONE: u32 = u32::MAX;

/// A failure that waits on an open ancestor.
#[derive(Debug)]
pub(super) struct Waited {
    /// The stable sequent.
    pub(super) key: Key,
    /// Its hash.
    pub(super) hash: u64,
    /// The copies that were left when it failed.
    pub(super) budget: u32,
    /// Whether some branch below it was cut by the copy budget.
    pub(super) exhausted: bool,
    /// Whether no part of its linear zone is provable either, as
    /// [`Failure::Hereditary`](super::memo::Failure::Hereditary) says;
    /// never with `exhausted`.
    pub(super) hereditary: bool,
    /// The depth of the shallowest ancestor it rests on, on the branch.
    pub(super) leader: u32,
    /// The entry before it of the same hash, or [`NONE`].
    previous: u32,
}

/// What a failure that waits answers for its sequent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Reused {
    /// Whether it was cut by the copy budget.
    pub(super) exhausted: bool,
    /// Whether it is hereditary.
    pub(super) hereditary: bool,
    /// The depth of its leader.
    pub(super) leader: u32,
}

/// The failures that wait on an open ancestor, in the order they failed,
/// so that those decided below a stable sequent are the entries from the
/// length the list had when it was pushed on the branch.
#[derive(Debug, Default)]
pub(super) struct Waiting {
    /// The failures.
    entries: Vec<Waited>,
    /// Per hash, the last entry of that hash.
    index: crate::hash::HashMap<u64, u32>,
    /// Keys of entries gone, whose buffers the next ones reuse.
    spare: Vec<Key>,
    /// The entries a close keeps waiting while it records the others.
    held: Vec<Waited>,
    /// The bytes of the lists' capacity that [`Waiting::grown`] handed
    /// out to be charged.
    charged: usize,
}

impl Waiting {
    /// How many failures wait: the mark of a stable sequent pushed now.
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether no failure waits.
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// What a failure of the stable sequent that waits says with `budget`
    /// copies left: one not cut by the budget whatever the budget, else
    /// one cut by it within its budget, unless the memo holds such a
    /// failure (`cut_known`), which rests on no ancestor.
    pub(super) fn find(
        &self,
        key: Zones<'_>,
        hash: u64,
        budget: u32,
        cut_known: bool,
    ) -> Option<Reused> {
        let mut found = None;
        let mut at = self.index.get(&hash).copied().unwrap_or(NONE);
        while at != NONE {
            let entry = &self.entries[at as usize];
            if entry.hash == hash && key.of(&entry.key) {
                if !entry.exhausted {
                    return Some(Reused {
                        exhausted: false,
                        hereditary: entry.hereditary,
                        leader: entry.leader,
                    });
                }
                if !cut_known && budget <= entry.budget && found.is_none() {
                    found = Some(Reused {
                        exhausted: true,
                        hereditary: false,
                        leader: entry.leader,
                    });
                }
            }
            at = entry.previous;
        }
        found
    }

    /// A spare key of an entry gone, whose buffers a new entry reuses.
    pub(super) fn spare(&mut self) -> Option<Key> {
        self.spare.pop()
    }

    /// Lets the failure of the stable sequent `key`, of hash `hash`, wait
    /// on the ancestor at depth `leader`.
    pub(super) fn wait(
        &mut self,
        key: Key,
        hash: u64,
        budget: u32,
        cuts: (bool, bool),
        leader: u32,
    ) {
        let (exhausted, hereditary) = cuts;
        self.push(Waited {
            key,
            hash,
            budget,
            exhausted,
            hereditary: hereditary && !exhausted,
            leader,
            previous: NONE,
        });
    }

    /// Appends an entry, linking it into the index.
    fn push(&mut self, mut entry: Waited) {
        let at = u32::try_from(self.entries.len()).expect("fewer entries than the memo holds");
        entry.previous = self.index.insert(entry.hash, at).unwrap_or(NONE);
        self.entries.push(entry);
    }

    /// Removes the last entry, unlinking it from the index.
    fn pop(&mut self) -> Option<Waited> {
        let entry = self.entries.pop()?;
        if entry.previous == NONE {
            self.index.remove(&entry.hash);
        } else {
            self.index.insert(entry.hash, entry.previous);
        }
        Some(entry)
    }

    /// Drops every entry from `mark` on: they were decided below a stable
    /// sequent that is proved, or that rests on what this engine cannot
    /// settle.
    pub(super) fn drop_from(&mut self, mark: usize) {
        while self.entries.len() > mark {
            let entry = self.pop().expect("an entry above the mark");
            self.spare.push(entry.key);
        }
    }

    /// Makes every entry from `mark` on a failure cut by the budget at its
    /// own budget: they may rest on a prune of a stable sequent whose
    /// failure was cut by the budget.
    pub(super) fn exhaust_from(&mut self, mark: usize) {
        for entry in self.entries.iter_mut().skip(mark) {
            entry.exhausted = true;
            entry.hereditary = false;
        }
    }

    /// Lets every entry from `mark` on whose leader lies deeper than
    /// `leader` wait on `leader`: they may rest on a stable sequent that
    /// failed with that dependency.
    pub(super) fn lead_from(&mut self, mark: usize, leader: u32) {
        for entry in self.entries.iter_mut().skip(mark) {
            entry.leader = entry.leader.min(leader);
        }
    }

    /// Takes the entries from `mark` on whose leader lies at `depth` or
    /// deeper, the last first, for the caller to record and give back;
    /// the others wait on. Call [`Waiting::settled`] once it has.
    pub(super) fn take_settled(&mut self, mark: usize, depth: u32) -> Option<Waited> {
        while self.entries.len() > mark {
            let entry = self.pop().expect("an entry above the mark");
            if entry.leader >= depth {
                return Some(entry);
            }
            self.held.push(entry);
        }
        None
    }

    /// Gives back the key of an entry taken.
    pub(super) fn give(&mut self, key: Key) {
        self.spare.push(key);
    }

    /// Lets the entries a close kept wait again, in their order.
    pub(super) fn settled(&mut self) {
        while let Some(entry) = self.held.pop() {
            self.push(entry);
        }
    }

    /// Drops every entry, keeping the buffers.
    pub(super) fn clear(&mut self) {
        self.drop_from(0);
    }

    /// Returns by how many bytes the lists' capacity grew since the last
    /// call, for the caller to charge: the entries, those a close keeps,
    /// the index's buckets (a power of two, eight for every seven entries
    /// it can hold, of a slot and a control byte each, and a group of
    /// control bytes after them) and the places of the spare keys,
    /// whose buffers the caller charges itself. The vectors' capacities
    /// only grow until [`Waiting::release`]; the index's can shrink by a
    /// removal, so what was handed out is the most it ever counted.
    pub(super) fn grown(&mut self) -> usize {
        /// The control bytes a table has after its buckets.
        const GROUP: usize = 16;
        let index = match self.index.capacity() {
            0 => 0,
            held => {
                (held * 8).div_ceil(7).next_power_of_two() * (size_of::<(u64, u32)>() + 1) + GROUP
            }
        };
        let bytes = (self.entries.capacity() + self.held.capacity()) * size_of::<Waited>()
            + self.spare.capacity() * size_of::<Key>()
            + index;
        let grown = bytes.saturating_sub(self.charged);
        self.charged = self.charged.max(bytes);
        grown
    }

    /// Drops every entry and every spare key with the lists' memory, and
    /// returns how many keys went and the bytes [`Waiting::grown`] handed
    /// out, whose charges the caller releases.
    pub(super) fn release(&mut self) -> (usize, usize) {
        self.clear();
        let gone = (self.spare.len(), self.charged);
        *self = Self::default();
        gone
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use crate::search::{Bias, Engine, Options, Verdict, prove_within};
    use crate::{Limits, Mode, Sequent};

    /// Every rule by which a close settles the failures that wait, and
    /// the reuse of one, pinned by sequents whose verdict or counters
    /// change when that rule goes: a proof drops what was decided below
    /// it, a failure cut by the budget makes what is below it cut, one
    /// with a dependency moves the leaders below it up, one without
    /// records only what it or a deeper ancestor leads, and a reuse keeps
    /// within its budget and keeps a cut. Each rule dropped in turn
    /// changed hundreds of counters on 58 000 generated cases and the
    /// verdicts of a few, none of them to a wrong one; so what is pinned
    /// is the verdict, the stable sequents visited, the memo's hits (a
    /// reuse of a failure that waits is one) and its most entries, as the
    /// target sets pin counters, and a change of the search that moves
    /// them pins them again.
    #[test]
    fn a_close_settles_by_every_rule() {
        let mix = Mode::CLASSICAL.with_mix();
        let (rarer, factors) = (Bias::Rarer, Bias::Factors);
        for (text, mode, copies, bias, memo, proved, nodes, hits, entries) in [
            // Lost without the cut a failure below a cut inherits.
            (
                "|- ~c, ?~c, ?((1 * c) * ?b), (c * !?~c)",
                mix,
                6,
                rarer,
                Some(16),
                false,
                1552,
                434,
                16,
            ),
            (
                "|- ?!?(~c * 1), !?!b, ?!(!bot par bot)",
                Mode::CLASSICAL,
                3,
                factors,
                None,
                false,
                28,
                10,
                7,
            ),
            // Without the drop at a proof.
            (
                "|- !(?(bot par (?(1 * ~a) par a)) * ?(bot par (?(1 * ~a) par c))), ?bot",
                mix,
                4,
                factors,
                None,
                false,
                22,
                7,
                6,
            ),
            // Without the leaders moved up.
            (
                "|- (?~a par ?~c), !~c, (?~a par ?~c), !~c, ?(c * c), ?~a, ?a, (?a * ?a)",
                mix,
                12,
                factors,
                None,
                true,
                287,
                175,
                69,
            ),
            // With every failure below recorded, whatever its leader.
            (
                "|- ?(~b par ~b), ?b, !(b * ?!~c), ?(?(a par a) par !(a * b))",
                mix,
                2,
                rarer,
                None,
                true,
                166,
                92,
                57,
            ),
            // Lost with a reuse past its budget, or of a cut as complete.
            (
                "!a, b, (!((c + (a + top)) & b) + !((c + (a + top)) & b)) |- (((!top * b) + (b * \
                 (top & top))) * (((c + (b + top)) & b) * b))",
                mix,
                2,
                factors,
                None,
                true,
                313,
                202,
                71,
            ),
            (
                "|- ?c, !(c * b), ?~c, (?~a par ?(~b * a))",
                mix,
                3,
                factors,
                None,
                true,
                38,
                17,
                11,
            ),
        ] {
            let s: Sequent = text.parse().unwrap();
            let mut options = Options::default()
                .with_engine(Some(Engine::Focus))
                .with_copies(Some(copies))
                .with_bias(bias)
                .with_jobs(1);
            if let Some(memo) = memo {
                options = options.with_memo_limit(memo);
            }
            let outcome = prove_within(&s, mode, &options, &Limits::default(), |_| false).unwrap();
            let decided = match outcome.verdict {
                Verdict::Proved(_) => Some(true),
                Verdict::Unprovable(_) => Some(false),
                Verdict::Unknown(_) => None,
            };
            let statistics = &outcome.statistics;
            let counted = (
                statistics.nodes,
                statistics.memo_hits,
                statistics.memo_entries,
            );
            assert_eq!(
                (decided, counted),
                (Some(proved), (nodes, hits, entries)),
                "{text:?} in {mode} mode, copy bound {copies}, {bias:?}, memo {memo:?}"
            );
        }
    }

    /// The lists' memory is handed out to be charged as it grows, at least
    /// an entry and an index slot for each failure that waited at once and
    /// a place for each spare key, and given back whole by `release`.
    #[test]
    fn the_lists_are_charged_as_they_grow() {
        use super::{Waited, Waiting};
        use crate::occurrences::OccSet;
        use crate::search::focus::context::Context;
        use crate::search::focus::memo::Key;
        let mut waiting = Waiting::default();
        let mut charged = 0;
        for hash in 0..1000 {
            let key = Key {
                theta: OccSet::empty(64),
                gamma: Context::empty(64),
            };
            waiting.wait(key, hash, 0, (false, false), 1);
            charged += waiting.grown();
        }
        assert_eq!(waiting.grown(), 0, "nothing grew since");
        let entries = size_of::<Waited>() + size_of::<(u64, u32)>();
        assert!(charged >= 1000 * entries, "{charged} bytes");
        waiting.drop_from(0);
        let spares = waiting.grown();
        assert!(spares >= 1000 * size_of::<Key>(), "{spares} bytes");
        assert_eq!(waiting.release(), (1000, charged + spares));
        assert_eq!(waiting.grown(), 0, "released");
    }

    /// A search that relieves its memory while failures wait keeps the
    /// marks of the branch true: under a bound of a few kibibytes the memo
    /// is emptied over and over in this Mix search, where nearly every
    /// failure waits, and a mark left past the list's end once let a
    /// failure decided after it outlive its leader (the debug assertion in
    /// the close).
    #[test]
    fn a_relieved_search_settles_what_waits() {
        let s: Sequent = "|- ?((((!~a par c) + c) par a) par ~c) par ?((~c * ~c) * c)"
            .parse()
            .unwrap();
        let options = Options::default()
            .with_copies(Some(2))
            .with_bias(Bias::Rarer)
            .with_jobs(1);
        for kibibytes in [8, 12, 16, 24] {
            let limits = Limits::default().with_memory_bytes(Some(kibibytes << 10));
            let outcome =
                prove_within(&s, Mode::CLASSICAL.with_mix(), &options, &limits, |_| false).unwrap();
            assert!(
                matches!(outcome.verdict, Verdict::Unknown(_)),
                "{kibibytes} KiB: {:?}",
                outcome.verdict
            );
        }
    }
}
