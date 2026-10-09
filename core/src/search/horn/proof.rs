// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The proof of a Horn program read off a firing sequence. Bottom up it
//! moves the clauses under `?` into the unrestricted zone, decomposes the
//! markings into tokens, and fires: a copy of the clause (or the clause
//! used once itself), whose tensors hand each body literal the token it
//! consumes by an axiom and the rest of the linear zone to the head, whose
//! `⅋` and `⊥` leave its literals as new tokens beside it. At the top the
//! goal's tensors meet the tokens left, an axiom each, and in affine mode
//! the tokens beyond the goal's and the clauses used once that were not
//! are weakened there.

use super::{Clause, Program, head_of};
use crate::occurrences::{Forest, Member, OccId, Sign};
use crate::proofs::{Node, NodeId};
use crate::search::Reason;
use crate::search::memory::{Account, Charged};
use crate::sequents::Kind;

/// The most nodes a proof may have: its ids are `u32`. A proof larger
/// than that is [`Reason::IndexLimit`]; with a memory bound the bound
/// comes first, since a node takes sixteen bytes.
pub(super) const MOST_NODES: u64 = u32::MAX as u64;

/// Builds the proof of the program's goal that fires `firings` from the
/// initial marking, which reaches the target, or in affine mode covers
/// it, into an arena of at most `most` nodes, the arena charged to
/// `account`; returns its root and the arena, or why it is not built.
pub(super) fn build(
    forest: &Forest,
    program: &Program,
    firings: &[u32],
    affine: bool,
    account: &Account,
    most: u64,
) -> Result<(NodeId, Vec<Node>), Reason> {
    let mut charged = Charged::new(account);
    // The clause each firing fires, and its size: an occurrence of the
    // clause is at most one node, and a copy one more.
    let mut once = program.once.clone();
    let clauses: Vec<(OccId, bool)> = firings
        .iter()
        .map(|&t| match program.transitions[t as usize].clause {
            Clause::Reusable(clause) => (clause, true),
            Clause::Once(class) => (
                once[class as usize]
                    .pop()
                    .expect("the target leaves no clause unused, and a firing takes one"),
                false,
            ),
        })
        .collect();
    // In affine mode the clauses used once that the firings left.
    let unused: Vec<OccId> = once.into_iter().flatten().collect();
    debug_assert!(affine || unused.is_empty());
    let size = |o: OccId| u64::from(forest.size(o));
    let nodes = clauses.iter().map(|&(c, _)| size(c) + 1).sum::<u64>()
        + size(program.goal)
        + program.markings.iter().map(|&m| size(m)).sum::<u64>()
        + program.quests.len() as u64;
    if nodes > most {
        return Err(Reason::IndexLimit);
    }
    // A weakening for each token left beside the goal's: the tokens given
    // and made less those taken, each a literal and so a node counted
    // above, so that none of these sums passes twice the bound.
    let atoms = program.places - program.once.len();
    let literals = |arcs: &[(u32, u32)]| {
        arcs.iter()
            .filter(|&&(p, _)| (p as usize) < atoms)
            .map(|&(_, w)| u64::from(w))
            .sum::<u64>()
    };
    let (mut made, mut taken) = (
        literals_of(&program.initial[..atoms]),
        literals_of(&program.target[..atoms]),
    );
    for &t in firings {
        let transition = &program.transitions[t as usize];
        taken += literals(&program.arcs[transition.inputs as usize..transition.outputs as usize]);
        made += literals(&program.arcs[transition.outputs as usize..transition.end as usize]);
    }
    let nodes = nodes + (made - taken) + unused.len() as u64;
    if nodes > most {
        return Err(Reason::IndexLimit);
    }
    // The nodes, and a pair of a body literal and its token for each, at
    // most a node each; the tokens the replay keeps, a literal each; and
    // the clauses fired, with where each one's pairs start.
    let per_node = size_of::<Node>() + size_of::<(OccId, OccId)>() + size_of::<OccId>();
    let per_firing = size_of::<(OccId, bool)>() + size_of::<usize>();
    let bytes = usize::try_from(nodes)
        .ok()
        .and_then(|nodes| nodes.checked_mul(per_node))
        .and_then(|bytes| bytes.checked_add(clauses.len() * per_firing))
        .filter(|&bytes| account.fits(bytes));
    let Some(bytes) = bytes else {
        return Err(Reason::MemoryLimit(account.limit()));
    };
    charged.charge(bytes);
    let (pairs, starts, left) = replay(forest, program, &clauses, nodes as usize);
    let mut builder = Builder {
        forest,
        body: program.body,
        nodes: Vec::with_capacity(nodes as usize),
        values: Vec::new(),
    };
    // The goal's pairs come last in the replay, each firing's in its place.
    let mut goal = &pairs[starts[clauses.len()]..];
    let mut root = builder.clause(program.goal, None, &mut goal, NodeId::new(0));
    debug_assert!(goal.is_empty());
    for &weakened in left.iter().chain(&unused) {
        root = builder.push(Node::Weaken(Member::from(weakened), root));
    }
    for (i, &(clause, reusable)) in clauses.iter().enumerate().rev() {
        let head = head_of(forest, program.body, clause);
        let mut fired = &pairs[starts[i]..starts[i + 1]];
        root = builder.clause(clause, head, &mut fired, root);
        debug_assert!(fired.is_empty());
        if reusable {
            root = builder.push(Node::Copy(Member::from(clause), root));
        }
    }
    for &marking in &program.markings {
        root = builder.head(marking, root);
    }
    for &quest in &program.quests {
        root = builder.push(Node::Quest(Member::from(quest), root));
    }
    Ok((root, builder.nodes))
}

/// The tokens of a marking.
fn literals_of(marking: &[u32]) -> u64 {
    marking.iter().map(|&c| u64::from(c)).sum()
}

/// Replays the firings on the tokens, which are occurrences of head
/// literals, from those of the markings: every body literal of a clause
/// fired takes a token of its atom, and its head's literals become
/// tokens. Returns the pairs of a body literal and its token, each
/// firing's and then the goal's, each in decreasing order of the body
/// literals' ids, which is the order the proof takes them in; where each
/// firing's pairs start, and the goal's last; and the tokens the goal
/// leaves, which only a firing sequence that covers the target and does
/// not reach it leaves.
fn replay(
    forest: &Forest,
    program: &Program,
    clauses: &[(OccId, bool)],
    capacity: usize,
) -> (Vec<(OccId, OccId)>, Vec<usize>, Vec<OccId>) {
    let place = |literal: OccId| {
        program.place_of[forest.atom(literal).expect("a literal").index()] as usize
    };
    let mut tokens: Vec<Vec<OccId>> = vec![Vec::new(); program.places];
    for &marking in &program.markings {
        for x in forest.subtree(marking).filter(|&x| forest.is_literal(x)) {
            tokens[place(x)].push(x);
        }
    }
    // Each firing's pairs and the goal's, and where they start.
    let mut pairs = Vec::with_capacity(capacity);
    let mut starts = Vec::with_capacity(clauses.len() + 1);
    let take =
        |clause: OccId, head: Option<OccId>, tokens: &mut Vec<Vec<OccId>>, pairs: &mut Vec<_>| {
            for x in forest.subtree(clause).rev() {
                if !head.is_some_and(|h| forest.is_below(x, h)) && forest.is_literal(x) {
                    let token = tokens[place(x)].pop().expect("the firing is enabled");
                    pairs.push((x, token));
                }
            }
        };
    for &(clause, _) in clauses {
        starts.push(pairs.len());
        let head = head_of(forest, program.body, clause);
        take(clause, head, &mut tokens, &mut pairs);
        if let Some(h) = head {
            for x in forest.subtree(h).filter(|&x| forest.is_literal(x)) {
                tokens[place(x)].push(x);
            }
        }
    }
    starts.push(pairs.len());
    take(program.goal, None, &mut tokens, &mut pairs);
    (pairs, starts, tokens.into_iter().flatten().collect())
}

/// What builds the nodes: the forest, the sign of the bodies, the arena,
/// and a stack of the nodes that prove the factors of a tensor.
struct Builder<'a> {
    /// The forest.
    forest: &'a Forest,
    /// The sign of the bodies' atoms.
    body: Sign,
    /// The arena.
    nodes: Vec<Node>,
    /// The proofs of the factors of a clause whose tensors are not built
    /// yet.
    values: Vec<NodeId>,
}

impl Builder<'_> {
    /// Pushes a node and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Wraps `above`, whose conclusion holds the literals of a head or a
    /// marking, in that formula's `⅋` and `⊥`: in decreasing order of
    /// ids, so that both subformulas of a `⅋` are there when it comes.
    fn head(&mut self, head: OccId, mut above: NodeId) -> NodeId {
        for x in self.forest.subtree(head).rev() {
            match self.forest.kind(x) {
                Kind::Par => above = self.push(Node::Par(Member::from(x), above)),
                Kind::Bot => above = self.push(Node::Bot(Member::from(x), above)),
                _ => {}
            }
        }
        above
    }

    /// Builds the proof of a clause fired, or of the goal, whose head's
    /// literals the proof `above` takes as tokens: an axiom for each body
    /// literal with its token, the next of `pairs`, `1` for each `1`, and
    /// the tensors over them, in decreasing order of ids, so that both
    /// factors of a tensor are proved when it comes.
    fn clause(
        &mut self,
        clause: OccId,
        head: Option<OccId>,
        pairs: &mut &[(OccId, OccId)],
        above: NodeId,
    ) -> NodeId {
        let forest = self.forest;
        for x in forest.subtree(clause).rev() {
            let node = match head {
                Some(h) if x == h => self.head(h, above),
                Some(h) if forest.is_below(x, h) => continue,
                _ => match forest.kind(x) {
                    Kind::Tensor => {
                        let left = self.values.pop().expect("the left factor is proved");
                        let right = self.values.pop().expect("the right factor is proved");
                        self.push(Node::Tensor(Member::from(x), left, right))
                    }
                    Kind::One => self.push(Node::One(Member::from(x))),
                    _ => {
                        let ((literal, token), rest) =
                            pairs.split_first().expect("a token per body literal");
                        debug_assert_eq!(*literal, x);
                        debug_assert_eq!(forest.sign(x), Some(self.body));
                        *pairs = rest;
                        self.push(Node::Ax(x.into(), (*token).into()))
                    }
                },
            };
            self.values.push(node);
        }
        self.values.pop().expect("the clause is proved")
    }
}
