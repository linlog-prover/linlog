// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Interchangeable occurrences. Two occurrences of the same formula are
//! distinct members of a sequent, but nothing a proof does tells them
//! apart: every rule looks at an occurrence's connective, its subformulas
//! and, two-sided, its position, and all of these are the same for both.
//! So a sequent stays as provable, by a proof of the same shape, when a
//! member is replaced by another occurrence of the same formula in the
//! same position, and the search need only try one of each kind.

use super::context::Context;
use super::memo::{REPEATED, decoded};
use crate::occurrences::{Forest, OccId, OccSet, Reading, Side};

/// The class of every occurrence of a forest: the occurrences of the same
/// term, and under an intuitionistic reading in the same position, are
/// *interchangeable* and share a class, which is named by its first
/// occurrence. A position below an occurrence is determined by the
/// occurrence's term and position, so the subformulas of two
/// interchangeable occurrences are interchangeable pairwise.
#[derive(Clone, Debug)]
pub(crate) struct Classes {
    /// Per occurrence, the first occurrence of its class.
    class: Box<[OccId]>,
    /// The occurrences that are not the first of their class.
    renamed: OccSet,
    /// Whether no two occurrences are interchangeable.
    distinct: bool,
}

/// Writes a class with its number of copies as a key's words at
/// `at`: the class alone for one copy, else marked and followed by the
/// number ([`REPEATED`]). Returns the words written.
fn mark(words: &mut [OccId], at: usize, class: OccId, copies: u32) -> usize {
    if copies == 1 {
        words[at] = class;
        1
    } else {
        words[at] = OccId::new(class.get() | REPEATED);
        words[at + 1] = OccId::new(copies);
        2
    }
}

impl Classes {
    /// Computes the classes of a forest, by term alone or, given the
    /// reading, by term and position.
    pub(crate) fn new(forest: &Forest, reading: Option<&Reading>) -> Self {
        /// No occurrence yet.
        const NONE: u32 = u32::MAX;
        // Per term, the first occurrence in input and in output position.
        let mut first = vec![[NONE; 2]; forest.sequent().terms().len()];
        let class: Box<[OccId]> = forest
            .ids()
            .map(|o| {
                let output = reading.is_some_and(|r| r.position(o) == Side::Output);
                let slot = &mut first[forest.term(o).index()][usize::from(output)];
                if *slot == NONE {
                    *slot = o.get();
                }
                OccId::new(*slot)
            })
            .collect();
        let mut renamed = OccSet::empty(forest.len());
        renamed.extend(forest.ids().filter(|&o| class[o.index()] != o));
        let distinct = renamed.is_empty();
        Self {
            class,
            renamed,
            distinct,
        }
    }

    /// Returns whether no two occurrences of the forest are
    /// interchangeable, so that every occurrence is its own class.
    #[cfg(test)]
    pub(crate) fn distinct(&self) -> bool {
        self.distinct
    }

    /// Returns the bytes the classes allocate.
    pub(crate) fn bytes(&self) -> usize {
        self.class.len() * size_of::<OccId>() + size_of_val(self.renamed.words())
    }

    /// Returns whether a member of `gamma` is not the first of its class,
    /// so that its canonical form ([`canonical`](Self::canonical)) differs
    /// from it: a test of the zone's words alone.
    pub(crate) fn renames(&self, gamma: &Context) -> bool {
        !self.distinct && gamma.meets(&self.renamed)
    }

    /// Returns whether a member a key lists ([`decoded`]) is not the first
    /// of its class, as [`renames`](Self::renames) on a zone of them.
    pub(crate) fn renames_any(&self, key: &[OccId]) -> bool {
        !self.distinct && decoded(key).any(|(o, _)| self.renamed.contains(o))
    }

    /// Makes `canonical` the key's words ([`decoded`]) with each member
    /// replaced by the first occurrence of its class, the copies of a
    /// class added up: the same for two keys exactly when one is the
    /// other up to interchangeable members. `repeats` says whether a
    /// member of the key is marked as present more than once.
    pub(crate) fn canonical(&self, key: &[OccId], repeats: bool, canonical: &mut Vec<OccId>) {
        canonical.clear();
        if !repeats {
            // Each member once: the classes sorted, and a run of one
            // class written as the class marked and its length.
            canonical.extend(key.iter().map(|&o| self.of(o)));
            canonical.sort_unstable();
            let words = canonical.as_mut_slice();
            let (mut written, mut at) = (0, 0);
            while at < words.len() {
                let class = words[at];
                let mut end = at + 1;
                while end < words.len() && words[end] == class {
                    end += 1;
                }
                written += mark(words, written, class, (end - at) as u32);
                at = end;
            }
            canonical.truncate(written);
            return;
        }
        for (o, copies) in decoded(key) {
            canonical.extend([self.of(o), OccId::new(copies)]);
        }
        canonical
            .as_chunks_mut::<2>()
            .0
            .sort_unstable_by_key(|[class, _]| *class);
        // Merged and written in place, a class in one word or two, never
        // past the pairs still to read.
        let pairs = canonical.len() / 2;
        let words = canonical.as_mut_slice();
        let (mut written, mut at) = (0, 0);
        while at < pairs {
            let class = words[2 * at];
            let mut copies = 0u32;
            while at < pairs && words[2 * at] == class {
                copies = copies
                    .checked_add(words[2 * at + 1].get())
                    .expect("fewer than 2³² copies of a class, as of a member");
                at += 1;
            }
            written += mark(words, written, class, copies);
        }
        canonical.truncate(written);
    }

    /// Returns the class of an occurrence: its first interchangeable
    /// occurrence.
    pub(crate) fn of(&self, o: OccId) -> OccId {
        self.class[o.index()]
    }

    /// Returns whether two occurrences are interchangeable.
    pub(crate) fn same(&self, x: OccId, y: OccId) -> bool {
        self.of(x) == self.of(y)
    }
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;
    use crate::Sequent;
    use crate::search::focus::context::Context;

    /// Equal formulas share a class wherever they occur, and a reading
    /// separates the positions: here the `⊤` that is the hypothesis `0`
    /// from the `⊤` that is the goal.
    #[test]
    fn classes() {
        use crate::sequents::Kind;
        let s: Sequent = "0, a, a |- top".parse().unwrap();
        let forest = Forest::new(&s).unwrap();
        let roots = |kind| -> Vec<OccId> {
            let roots = forest.roots().iter().copied();
            roots.filter(|&r| forest.kind(r) == kind).collect()
        };
        let (tops, literals) = (roots(Kind::Top), roots(Kind::DualAtom));
        assert_eq!((tops.len(), literals.len()), (2, 2));
        let classes = Classes::new(&forest, None);
        assert!(classes.same(tops[0], tops[1]));
        assert!(classes.same(literals[0], literals[1]));
        assert!(!classes.same(tops[0], literals[0]));
        assert_eq!(classes.of(literals[1]), literals[0]);
        assert!(!classes.distinct());
        // A zone's canonical form has the first of each class, as often as
        // the zone has members of the class, and is its own.
        let mut gamma = Context::empty(forest.len());
        for o in [literals[1], literals[1], literals[0], tops[1]] {
            gamma.insert(o);
        }
        let mut words = Vec::new();
        gamma.key_into(&mut words);
        assert!(classes.renames(&gamma) && classes.renames_any(&words));
        let mut canonical = Vec::new();
        classes.canonical(&words, true, &mut canonical);
        let three = [OccId::new(literals[0].get() | REPEATED), OccId::new(3)];
        let expected = if literals[0] < tops[0] {
            [three.as_slice(), &[tops[0]]].concat()
        } else {
            [[tops[0]].as_slice(), &three].concat()
        };
        assert_eq!(canonical, expected);
        assert!(!classes.renames_any(&canonical));
        let mut again = Vec::new();
        classes.canonical(&canonical, true, &mut again);
        assert_eq!(again, canonical);
        // Members each once that share a class make a run, written as
        // the class marked and its length.
        let pair = [literals[0], literals[1]];
        classes.canonical(&pair, false, &mut again);
        assert_eq!(
            again,
            [OccId::new(literals[0].get() | REPEATED), OccId::new(2)]
        );
        let reading = Reading::new(&forest).unwrap();
        let classes = Classes::new(&forest, Some(&reading));
        assert!(!classes.same(tops[0], tops[1]));
        assert!(classes.same(literals[0], literals[1]));
    }
}
