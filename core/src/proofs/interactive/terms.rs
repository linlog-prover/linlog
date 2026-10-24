// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! A finished session's derivation translated into a proof term.

use super::Interactive;
use crate::Error;
use crate::limits::{Limits, Progress};
use crate::occurrences::{Member, OccId};
use crate::proofs::Rule;
use crate::proofs::derivation::InfId;
use crate::proofs::{Branch, Node, NodeId, Proof};
use crate::sequents::Kind;

impl Interactive {
    /// Returns the arena's inferences in postorder, premises before
    /// conclusions, the order of [`derivation`](Self::derivation).
    pub(super) fn postorder(&self) -> Vec<InfId> {
        let mut order = Vec::with_capacity(self.inferences.len());
        // Postorder without recursion: a frame is an inference and whether
        // its premises were visited.
        let mut stack = vec![(InfId::new(0), false)];
        while let Some((id, visited)) = stack.pop() {
            if visited {
                order.push(id);
            } else {
                stack.push((id, true));
                let premises = &self.inferences[id.index()].premises;
                stack.extend(premises.iter().rev().map(|&p| (p, false)));
            }
        }
        order
    }

    /// Translates the finished derivation into a proof term and checks it,
    /// returning the proof. Fails while goals are open
    /// ([`Error::OpenGoals`]) or if the checker rejects the term
    /// ([`Error::Check`]), which it never does for a derivation built
    /// through this interface, or gives the check up within `limits` or
    /// when `stop` fires ([`Error::Check`] with a
    /// [`CheckError::Refused`](crate::CheckError::Refused)).
    ///
    /// # Errors
    ///
    /// [`Error::OpenGoals`] while goals are open, and [`Error::Check`] for a
    /// term the checker rejects or a check given up within `limits` or by
    /// `stop`.
    pub fn proof(
        &self,
        limits: &Limits,
        stop: impl FnMut(Progress) -> bool,
    ) -> Result<Proof, Error> {
        let open = self.goals().count();
        if open > 0 {
            return Err(Error::OpenGoals { count: open });
        }
        let mut terms = Terms {
            state: self,
            nodes: Vec::with_capacity(self.inferences.len()),
        };
        let mut root = terms.term(InfId::new(0));
        for &r in self.forest.roots() {
            if self.forest.kind(r) == Kind::Quest {
                root = terms.push(Node::Quest(Member::from(r), root));
            }
        }
        let proof = Proof::new(self.forest.clone(), terms.nodes, root)?.with_mode(self.mode);
        proof.check_within(self.mode, limits, stop)?;
        Ok(proof)
    }
}

/// The translation of a finished derivation into a proof term, bottom-up
/// over the tree: every rule is its node; a dereliction is a copy, a
/// contraction and a `?w` are nothing, since a `?` formula lives in the
/// unrestricted zone from the point where it enters the derivation, which
/// is where its `?` node goes: below the rule that introduces it as a
/// subformula, or below the root for a `?` formula of the sequent.
pub(super) struct Terms<'a> {
    /// The finished derivation.
    state: &'a Interactive,
    /// The arena being built, premises before conclusions.
    nodes: Vec<Node>,
}

impl Terms<'_> {
    /// Appends a node and returns its id.
    pub(super) fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Builds the term of the subtree at `root`, without recursion: a
    /// derivation read from a file can be as high as its formulas are
    /// deep. The steps still to take wait on a stack and the terms of the
    /// subtrees done on another, in the order a recursion would take
    /// them, premises left to right and a rule after its premises.
    pub(super) fn term(&mut self, root: InfId) -> NodeId {
        use Rule::*;
        /// What is left to do.
        enum Step {
            /// Build the term of the subtree at an inference.
            Visit(InfId),
            /// Put the `?` nodes of the `?` formulas among these, which
            /// the rule below introduced, on the term just built.
            Quests([Option<OccId>; 2]),
            /// Put an inference's own node on the terms of its premises.
            Build(InfId),
        }
        let state = self.state;
        let f = &state.forest;
        let mut steps = vec![Step::Visit(root)];
        let mut done: Vec<NodeId> = Vec::new();
        while let Some(step) = steps.pop() {
            let id = match step {
                Step::Visit(id) | Step::Build(id) => id,
                Step::Quests(introduced) => {
                    let mut node = done.pop().expect("the premise's term");
                    for x in introduced.into_iter().flatten() {
                        if f.kind(x) == Kind::Quest {
                            node = self.push(Node::Quest(Member::from(x), node));
                        }
                    }
                    done.push(node);
                    continue;
                }
            };
            let inference = &state.inferences[id.index()];
            let sequent = &inference.sequent;
            let premises = &inference.premises;
            let o = || {
                sequent[inference
                    .principal
                    .expect("every rule but ax and mix has one")]
            };
            let a = || f.left(o().occ()).expect("a connective with a subformula");
            let b = || f.right(o().occ()).expect("a binary connective");
            let rule = inference.rule.rule;
            if matches!(step, Step::Visit(_)) {
                // What each premise's sequent gains from the rule.
                let introduced: [[Option<OccId>; 2]; 2] = match rule {
                    Ax | One | Top => {
                        #[expect(
                            clippy::wildcard_enum_match_arm,
                            reason = "the arm holds three rules"
                        )]
                        let leaf = match rule {
                            Ax => Node::Ax(sequent[0], sequent[1]),
                            One => Node::One(o()),
                            _ => Node::Top(o()),
                        };
                        let leaf = self.push(leaf);
                        done.push(leaf);
                        continue;
                    }
                    // Neither is a node: the subtree's term is the
                    // premise's.
                    Contraction | Weakening => {
                        steps.push(Step::Visit(premises[0]));
                        continue;
                    }
                    Par => [[Some(a()), Some(b())], [None; 2]],
                    Bot | AffineWeakening | Mix => [[None; 2]; 2],
                    PlusLeft | Promotion | Dereliction => [[Some(a()), None], [None; 2]],
                    PlusRight => [[Some(b()), None], [None; 2]],
                    With | Tensor => [[Some(a()), None], [Some(b()), None]],
                    Cut => {
                        let a = inference.cut.expect("a cut names its formula").occ();
                        [[Some(a), None], [f.dual(a), None]]
                    }
                    Open => unreachable!("no goal is open"),
                };
                steps.push(Step::Build(id));
                for (&premise, introduced) in premises.iter().zip(introduced).rev() {
                    steps.push(Step::Quests(introduced));
                    steps.push(Step::Visit(premise));
                }
                continue;
            }
            let mut premise = || done.pop().expect("a premise's term");
            let node = match rule {
                Par => Node::Par(o(), premise()),
                Bot => Node::Bot(o(), premise()),
                PlusLeft => Node::Plus(o(), Branch::Left, premise()),
                PlusRight => Node::Plus(o(), Branch::Right, premise()),
                Promotion => Node::Bang(o(), premise()),
                Dereliction => Node::Copy(Member::from(a()), premise()),
                AffineWeakening => Node::Weaken(o(), premise()),
                With => {
                    let (r, l) = (premise(), premise());
                    Node::With(o(), l, r)
                }
                Tensor => {
                    let (r, l) = (premise(), premise());
                    Node::Tensor(o(), l, r)
                }
                Mix => {
                    let (r, l) = (premise(), premise());
                    Node::Mix(l, r)
                }
                Cut => {
                    let (r, l) = (premise(), premise());
                    Node::Cut(inference.cut.expect("a cut names its formula"), l, r)
                }
                Ax | One | Top | Contraction | Weakening | Open => {
                    unreachable!("a rule that is no node over its premises")
                }
            };
            let node = self.push(node);
            done.push(node);
        }
        done.pop().expect("the root's term")
    }
}
