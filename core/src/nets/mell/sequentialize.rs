// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! Sequentialization: from a proof net with boxes to a proof term of the
//! dyadic calculus, by the splitting tensor lemma at every box depth. A
//! sub-net is given by its conclusions, all at one depth. While one is a
//! `⅋`, it is replaced by its premises and a `⅋` rule goes below the rest;
//! while one is a `?` vertex, a `?` rule moves its subformula into the
//! unrestricted zone and its copies at its depth become conclusions, each
//! brought into the linear zone by a copy rule above it, so that the
//! copies inside boxes are in reach when their box is proved; inside a box
//! the doors are such conclusions from the start, their `?` rules lying
//! below the promotion. With Mix allowed, a sub-net in several parts is
//! proved part by part. Two literals alone are an axiom, a `!` vertex alone
//! the promotion of its box's contents. Otherwise some `⊗` conclusion is
//! splitting, which the bridge of its premise edge in the plain graph
//! says, as in the multiplicative case: the depth's graph is a
//! multiplicative structure in which a `?` vertex or a door is a `⅋` of
//! any arity, a `!` vertex with its box a hypothesis whose conclusions are
//! the `!` formula and its wires, and a link with the weakenings that hang
//! on it an axiom with more conclusions.

use super::graph::Scratch;
use super::{MellStructure, SCRATCH_BYTES, VERTEX_BYTES};
use crate::Error;
use crate::fragment::Mode;
use crate::limits::{Limits, Phase, Progress};
use crate::nets::{NONE, STOPPED, VertexId, afford};
use crate::occurrences::{Member, OccId};
use crate::proofs::{Node, NodeId, Proof};
use crate::sequents::Kind;

/// A rule a stage applies below what proves the rest, on an occurrence.
#[derive(Clone, Copy, Debug)]
enum Open {
    /// A `⅋` rule.
    Par(OccId),
    /// A `?` rule.
    Quest(OccId),
    /// A copy.
    Copy(OccId),
}

/// A step of a sequentialization.
enum Step {
    /// Prove the sub-net with these conclusions, graph vertices of one
    /// depth.
    Prove(Vec<u32>),
    /// Join the last two sub-nets proved with Mix.
    Mix,
    /// Apply this `⊗` to the last two sub-nets proved.
    Tensor(OccId),
    /// Apply this promotion to the last sub-net proved.
    Bang(OccId),
    /// Apply these rules to the last sub-net proved, the last first.
    Opened(Vec<Open>),
}

impl MellStructure {
    /// Turns a proof net with boxes into a proof term of its sequent, which
    /// the proof checker accepts in classical mode, with Mix where the
    /// criterion allows it, the mode the proof records: `⅋`, `⊗`, axiom,
    /// promotion, `?`, copy and, where the structure falls apart, Mix
    /// nodes, a weakening a `?` rule without copies. It takes time
    /// quadratic in the graph at most, holds within `limits.memory_bytes`
    /// (estimated before anything is made) and asks `stop` as
    /// [`is_correct`](Self::is_correct) does and once per stage.
    ///
    /// # Errors
    ///
    /// The criterion's [`NetError`](crate::NetError) if the structure is
    /// not a proof net; [`NetError::Refused`](crate::NetError::Refused)
    /// when the bound or the stop ended it.
    #[expect(
        clippy::missing_panics_doc,
        reason = "the expect states that a sequentialization keeps premises before conclusions, which the stages establish"
    )]
    pub fn sequentialize(
        &self,
        limits: &Limits,
        mut stop: impl FnMut(Progress) -> bool,
    ) -> Result<Proof, Error> {
        let vertices = self.graph.vertex_count() as u64;
        let bytes = vertices.saturating_mul(VERTEX_BYTES + SCRATCH_BYTES + 8);
        afford(bytes, limits)?;
        self.is_correct(|progress| {
            stop(Progress {
                held_bytes: bytes,
                ..progress
            })
        })?;
        let mut run = Sequentialization {
            net: self,
            scratch: Scratch::new(&self.graph),
            nodes: Vec::with_capacity(self.len()),
            steps: Vec::new(),
            proved: Vec::new(),
        };
        let mut stages = 0;
        let mut stop = |stage: u64| {
            stages += stage;
            stop(Progress {
                held_bytes: bytes,
                ..Progress::new(Phase::Net, stage, stages)
            })
        };
        let roots = (0..self.forest.roots().len() as u32).collect();
        let Some(root) = run.sequentialize(roots, &mut stop) else {
            return Err(STOPPED.into());
        };
        let mode = if self.criterion.mix {
            Mode::CLASSICAL.with_mix()
        } else {
            Mode::CLASSICAL
        };
        let proof = Proof::new(self.forest.clone(), run.nodes, root)
            .expect("premises precede conclusions and every occurrence is the forest's")
            .with_mode(mode);
        debug_assert_eq!(proof.check(mode), Ok(()));
        Ok(proof)
    }
}

/// A sequentialization in progress.
struct Sequentialization<'a> {
    /// The proof net.
    net: &'a MellStructure,
    /// Working memory for the searches; conclusions handled are deleted
    /// in it, so a sub-net is what its conclusions reach.
    scratch: Scratch,
    /// The proof's arena so far.
    nodes: Vec<Node>,
    /// What is yet to do, the next step last.
    steps: Vec<Step>,
    /// The nodes concluding the sub-nets proved and not yet used.
    proved: Vec<NodeId>,
}

impl Sequentialization<'_> {
    /// Adds a node and returns its id.
    fn push(&mut self, node: Node) -> NodeId {
        self.nodes.push(node);
        NodeId::new(self.nodes.len() as u32 - 1)
    }

    /// Takes the node concluding the last sub-net proved.
    fn premise(&mut self) -> NodeId {
        self.proved
            .pop()
            .expect("a sub-net proved for every premise")
    }

    /// Returns the occurrence of a structure vertex as a member.
    fn member(&self, v: u32) -> Member {
        Member::from(self.net.occurrence[v as usize])
    }

    /// Proves the sub-net with the given conclusions and returns the node
    /// concluding it, asking `stop` before each stage; `None` when it
    /// fired.
    fn sequentialize(
        &mut self,
        gamma: Vec<u32>,
        stop: &mut dyn FnMut(u64) -> bool,
    ) -> Option<NodeId> {
        self.steps.push(Step::Prove(gamma));
        while let Some(step) = self.steps.pop() {
            let node = match step {
                Step::Prove(gamma) => {
                    if stop(1) {
                        return None;
                    }
                    self.stage(gamma);
                    continue;
                }
                Step::Mix => {
                    let other = self.premise();
                    let node = self.premise();
                    self.push(Node::Mix(node, other))
                }
                Step::Tensor(t) => {
                    let right = self.premise();
                    let left = self.premise();
                    self.push(Node::Tensor(Member::from(t), left, right))
                }
                Step::Bang(b) => {
                    let premise = self.premise();
                    self.push(Node::Bang(Member::from(b), premise))
                }
                Step::Opened(rules) => {
                    let mut node = self.premise();
                    for &rule in rules.iter().rev() {
                        node = self.push(match rule {
                            Open::Par(o) => Node::Par(Member::from(o), node),
                            Open::Quest(o) => Node::Quest(Member::from(o), node),
                            Open::Copy(o) => Node::Copy(Member::from(o), node),
                        });
                    }
                    node
                }
            };
            self.proved.push(node);
        }
        Some(self.premise())
    }

    /// Opens the copies of a `?` vertex or door at its depth: each becomes
    /// a conclusion, brought in by a copy rule.
    fn copies(&self, source: u32, gamma: &mut Vec<u32>, rules: &mut Vec<Open>) {
        let graph = &self.net.graph;
        for w in graph.wires(source) {
            if w < graph.vertex_offset() && graph.parent_edge(w) == source {
                rules.push(Open::Copy(self.net.occurrence[w as usize]));
                gamma.push(w);
            }
        }
    }

    /// One stage, on the sub-net with the given conclusions: opens its
    /// `⅋` and `?` conclusions and its doors, then proves it by an axiom,
    /// or leaves the steps that prove it: its parts with a Mix after each
    /// but the first, the contents of its box and the promotion, or the two
    /// sides of its splitting `⊗` and the `⊗` after them.
    fn stage(&mut self, mut gamma: Vec<u32>) {
        let net = self.net;
        let graph = &net.graph;
        let n = graph.vertex_offset();
        let mut rules = Vec::new();
        // A conclusion opened without a premise in its place is marked
        // `NONE` and dropped at the end, so that each costs constant time.
        let mut i = 0;
        while i < gamma.len() {
            let c = gamma[i];
            if c >= n {
                gamma[i] = NONE;
                i += 1;
                self.scratch.delete(c);
                self.copies(c, &mut gamma, &mut rules);
                continue;
            }
            match net.kind(c) {
                Kind::Par => {
                    let mut premises = net.premises(VertexId::new(c));
                    let (l, r) = (premises.next().unwrap(), premises.next().unwrap());
                    self.scratch.delete(c);
                    rules.push(Open::Par(net.occurrence[c as usize]));
                    gamma[i] = l.get();
                    gamma.push(r.get());
                }
                Kind::Quest => {
                    gamma[i] = NONE;
                    i += 1;
                    self.scratch.delete(c);
                    rules.push(Open::Quest(net.occurrence[c as usize]));
                    self.copies(c, &mut gamma, &mut rules);
                }
                Kind::Atom
                | Kind::DualAtom
                | Kind::Tensor
                | Kind::Bang
                | Kind::One
                | Kind::Bot
                | Kind::Top
                | Kind::Zero
                | Kind::With
                | Kind::Plus => i += 1,
            }
        }
        gamma.retain(|&c| c != NONE);
        if !rules.is_empty() {
            self.steps.push(Step::Opened(rules));
        }

        let parts = graph.search(&mut self.scratch, gamma.iter().copied());
        if parts > 1 {
            debug_assert!(net.criterion.mix, "a proof net without Mix is connected");
            // The parts in the order of their first conclusion, by a sort of
            // the labels, so that a stage of many parts stays near linear.
            let mut first: Vec<(u32, usize)> = gamma
                .iter()
                .enumerate()
                .map(|(i, &c)| (self.scratch.component(c), i))
                .collect();
            first.sort_unstable();
            first.dedup_by_key(|&mut (label, _)| label);
            let labels: Vec<u32> = first.iter().map(|&(label, _)| label).collect();
            first.sort_unstable_by_key(|&(_, i)| i);
            let mut parts: Vec<Vec<u32>> = vec![Vec::new(); labels.len()];
            for &c in &gamma {
                let label = self.scratch.component(c);
                let j = labels.binary_search(&label).expect("every label is listed");
                parts[j].push(c);
            }
            for (i, &(label, _)) in first.iter().enumerate().rev() {
                if i > 0 {
                    self.steps.push(Step::Mix);
                }
                let j = labels.binary_search(&label).expect("every label is listed");
                self.steps.push(Step::Prove(std::mem::take(&mut parts[j])));
            }
        } else if gamma
            .iter()
            .all(|&c| net.forest.is_literal(net.occurrence[c as usize]))
        {
            debug_assert!(
                gamma.len() == 2 && net.partner[gamma[0] as usize] == gamma[1],
                "a connected sub-net of literals is one link"
            );
            let (x, y) = (self.member(gamma[0]), self.member(gamma[1]));
            let axiom = self.push(Node::Ax(x.min(y), x.max(y)));
            self.proved.push(axiom);
        } else if let [bang] = gamma[..]
            && net.kind(bang) == Kind::Bang
        {
            self.scratch.delete(bang);
            let premise = net.premises(VertexId::new(bang)).next().unwrap().get();
            let mut contents = vec![premise];
            contents.extend(graph.doors_in(bang));
            self.steps.push(Step::Bang(net.occurrence[bang as usize]));
            self.steps.push(Step::Prove(contents));
        } else {
            // The splitting ⊗ conclusion with the smallest id: one whose
            // premise edge is a bridge.
            let t = gamma
                .iter()
                .copied()
                .filter(|&c| net.kind(c) == Kind::Tensor)
                .filter(|&c| {
                    let l = net.premises(VertexId::new(c)).next().unwrap().get();
                    self.scratch.is_bridge(c, l)
                })
                .min()
                .expect("a connected proof net that is no link and no box has a splitting ⊗");
            let mut premises = net.premises(VertexId::new(t));
            let (l, r) = (
                premises.next().unwrap().get(),
                premises.next().unwrap().get(),
            );
            self.scratch.delete(t);
            graph.search(&mut self.scratch, [l]);
            let (mut left, mut right) = (vec![l], vec![r]);
            for &c in &gamma {
                if c == t {
                    continue;
                }
                if self.scratch.reached(c) {
                    left.push(c);
                } else {
                    right.push(c);
                }
            }
            self.steps.push(Step::Tensor(net.occurrence[t as usize]));
            self.steps.push(Step::Prove(right));
            self.steps.push(Step::Prove(left));
        }
    }
}
