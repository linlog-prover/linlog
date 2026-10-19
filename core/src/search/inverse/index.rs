// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The indices of the sequents kept, for the questions subsumption asks of
//! them: is there a kept sequent with the same linear zone and an
//! unrestricted zone within the one given, or, of those that absorb or
//! weaken, one whose two zones both lie within the ones given? A trie over
//! the sorted members of the linear zone, repeats spelled out, a separator,
//! and the sorted members of the unrestricted zone: a query walks only the
//! paths whose members the zones given hold, instead of every sequent kept.

use super::super::memory::bytes_of;
use crate::hash::HashMap;
use crate::occurrences::OccId;

/// What separates the linear zone from the unrestricted one on a path: no
/// occurrence, since a forest has fewer than `u32::MAX` of them.
const SEPARATOR: OccId = OccId::new(u32::MAX);

/// A trie of pairs of sorted zones, each path ending in the ids of the
/// sequents it spells.
#[derive(Debug, Default)]
pub(super) struct Trie {
    /// The children of a node by the member that extends its path; the
    /// root is node 0.
    children: HashMap<(u32, OccId), u32>,
    /// The ids at a node, the sequents whose zones end there.
    ends: HashMap<u32, Vec<u32>>,
    /// The nodes made, the root left out.
    nodes: u32,
    /// The bytes of the lists in `ends`, which the table's own capacity
    /// does not count.
    lists: usize,
}

impl Trie {
    /// Returns the child of a node by a member, making it where it is
    /// missing, or `None` where every `u32` names a node already.
    fn child(&mut self, node: u32, o: OccId) -> Option<u32> {
        if let Some(&child) = self.children.get(&(node, o)) {
            return Some(child);
        }
        self.nodes = self.nodes.checked_add(1)?;
        self.children.insert((node, o), self.nodes);
        Some(self.nodes)
    }

    /// Adds a sequent's zones, sorted, under its id; returns false, adding
    /// nothing, where the trie has no name left for a node it needs, which
    /// only a search without a memory bound reaches.
    pub(super) fn insert(&mut self, gamma: &[OccId], theta: &[OccId], id: u32) -> bool {
        let mut node = 0;
        for &o in gamma.iter().chain([&SEPARATOR]).chain(theta) {
            match self.child(node, o) {
                Some(child) => node = child,
                None => return false,
            }
        }
        let list = self.ends.entry(node).or_default();
        let before = bytes_of(list);
        list.push(id);
        self.lists += bytes_of(list) - before;
        true
    }

    /// Calls `each` on the nodes reached from `start` along a path whose
    /// members lie within the sorted `zone`, as a multiset, until it says
    /// to stop; returns whether it did.
    fn walk(&self, start: u32, zone: &[OccId], mut each: impl FnMut(u32) -> bool) -> bool {
        let mut stack = vec![(start, 0usize)];
        while let Some((node, from)) = stack.pop() {
            if each(node) {
                return true;
            }
            for j in from..zone.len() {
                // A repeat of the member before it extends the paths that
                // member extended already, but for the one that takes both.
                if j > from && zone[j] == zone[j - 1] {
                    continue;
                }
                if let Some(&child) = self.children.get(&(node, zone[j])) {
                    stack.push((child, j + 1));
                }
            }
        }
        false
    }

    /// Whether some sequent at the end of a path from `node` whose members
    /// lie within `theta` has `hit` true.
    fn ends_within(&self, node: u32, theta: &[OccId], hit: &mut impl FnMut(u32) -> bool) -> bool {
        self.walk(node, theta, |n| {
            self.ends
                .get(&n)
                .is_some_and(|ids| ids.iter().any(|&id| hit(id)))
        })
    }

    /// Whether some sequent with the linear zone `gamma` and an
    /// unrestricted zone within `theta` has `hit` true.
    pub(super) fn same_within(
        &self,
        gamma: &[OccId],
        theta: &[OccId],
        mut hit: impl FnMut(u32) -> bool,
    ) -> bool {
        let mut node = 0;
        for &o in gamma.iter().chain([&SEPARATOR]) {
            match self.children.get(&(node, o)) {
                Some(&child) => node = child,
                None => return false,
            }
        }
        self.ends_within(node, theta, &mut hit)
    }

    /// Whether some sequent whose two zones lie within `gamma` and `theta`
    /// has `hit` true.
    pub(super) fn within(
        &self,
        gamma: &[OccId],
        theta: &[OccId],
        mut hit: impl FnMut(u32) -> bool,
    ) -> bool {
        self.walk(0, gamma, |node| {
            self.children
                .get(&(node, SEPARATOR))
                .is_some_and(|&split| self.ends_within(split, theta, &mut hit))
        })
    }

    /// The bytes the trie holds.
    pub(super) fn bytes(&self) -> usize {
        self.children.capacity() * (size_of::<((u32, OccId), u32)>() + 1)
            + self.ends.capacity() * (size_of::<(u32, Vec<u32>)>() + 1)
            + self.lists
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Within a zone the trie finds the zones inserted below it, and only
    /// those; a node past the last `u32` is refused, never wrapped onto
    /// the root.
    #[test]
    fn finds_zones_within_and_refuses_past_its_names() {
        let o = OccId::new;
        let mut trie = Trie::default();
        assert!(trie.insert(&[o(1), o(1), o(3)], &[o(7)], 0));
        assert!(trie.insert(&[o(2)], &[], 1));
        let found = |gamma: &[OccId], theta: &[OccId]| {
            let mut ids = Vec::new();
            trie.within(gamma, theta, |id| {
                ids.push(id);
                false
            });
            ids.sort_unstable();
            ids
        };
        assert_eq!(found(&[o(1), o(1), o(2), o(3)], &[o(7)]), vec![0, 1]);
        assert_eq!(
            found(&[o(1), o(2), o(3)], &[o(7)]),
            vec![1],
            "one 1 too few"
        );
        assert_eq!(found(&[o(1), o(1), o(3)], &[]), Vec::<u32>::new());
        assert!(trie.same_within(&[o(2)], &[o(5)], |_| true));
        assert!(!trie.same_within(&[o(1), o(1), o(3)], &[], |_| true));
        let mut full = Trie {
            nodes: u32::MAX - 1,
            ..Trie::default()
        };
        assert!(!full.insert(&[o(1)], &[], 2), "two nodes past the names");
        assert!(!full.within(&[o(1)], &[], |_| true));
    }
}
