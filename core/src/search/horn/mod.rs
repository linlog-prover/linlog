// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The Horn engine: a goal that is a Horn program, clauses that may be
//! used any number of times under `!`, clauses used once, atoms and one
//! goal of atoms, is a Petri net with a marking to reach, and is decided
//! by a search over the markings instead of over sequents. A firing of a
//! clause is a copy of it, a split of the linear zone that hands the
//! clause's body exactly the tokens it consumes, and a decomposition of
//! its head into new tokens, so the firing sequence found is the proof.

/// The proof read off a firing sequence.
mod proof;
/// The search over markings.
mod reach;
#[cfg(test)]
mod tests;

use super::memory::Account;
use super::{Answer, Decide, Engine, Options, Refutation, Statistics, Task};
use crate::Error;
use crate::hash::HashMap;
use crate::occurrences::{Forest, OccId, Position, Sign};
use crate::sequents::Kind;

/// The engine, as [`Engine::Horn`] names it.
pub(crate) struct Horn;

impl Decide for Horn {
    /// Refuses affine mode, and a goal that is no Horn program.
    fn admits(&self, task: &Task<'_>) -> Result<(), Error> {
        if task.mode.affine {
            return Err(Error::EngineMode {
                engine: Engine::Horn,
                mode: task.mode,
            });
        }
        if Program::read(task).is_none() {
            return Err(Error::NotHorn);
        }
        Ok(())
    }

    /// Searches the markings on the calling thread, whatever
    /// [`Options::jobs`] says, and reads the proof off the firing sequence
    /// found.
    fn decide(
        &self,
        task: &Task<'_>,
        _options: &Options,
        account: &Account,
        stop: &mut dyn FnMut() -> bool,
    ) -> Result<Answer, Error> {
        // What the counts of the goal's literals rule out needs no
        // search: a goal that never balances an atom, whose net may be
        // infinite and searched until the memory bound otherwise.
        let counted = super::focus::refutation(
            task.forest,
            task.goal,
            task.fragment,
            task.mode,
            account,
            stop,
        );
        if counted != Refutation::Exhausted {
            return Ok(Answer::of_arena(
                task.forest,
                (Ok(None), Vec::new(), Statistics::default()),
            ));
        }
        let program = Program::read(task).expect("the engine admitted the goal");
        let (found, statistics) = reach::search(&program, account, reach::MOST_MARKINGS, stop);
        let (result, nodes) = match found {
            Ok(Some(firings)) => {
                match proof::build(task.forest, &program, &firings, account, proof::MOST_NODES) {
                    Ok((root, nodes)) => (Ok(Some(root)), nodes),
                    Err(reason) => (Err(reason), Vec::new()),
                }
            }
            Ok(None) => (Ok(None), Vec::new()),
            Err(reason) => (Err(reason), Vec::new()),
        };
        Ok(Answer::of_arena(task.forest, (result, nodes, statistics)))
    }
}

/// Whether a goal is a Horn program the engine decides: the feature of
/// its row in the dispatch.
pub(crate) fn is_program(task: &Task<'_>) -> bool {
    Program::read(task).is_some()
}

/// A list of arcs: places with their weights.
type Arcs = Vec<(u32, u32)>;

/// What a clause fired is: one under a `?`, whose occurrence is copied at
/// every firing, or one of a class of interchangeable clauses used once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Clause {
    /// The occurrence of the clause under its `?`.
    Reusable(OccId),
    /// The index of the class in [`Program::once`].
    Once(u32),
}

/// A transition of the net: the clause it fires and its arcs.
#[derive(Clone, Copy, Debug)]
struct Transition {
    /// The clause.
    clause: Clause,
    /// Where its inputs start in [`Program::arcs`].
    inputs: u32,
    /// Where its outputs start, and its inputs end.
    outputs: u32,
    /// Where its outputs end.
    end: u32,
}

/// A Horn program read off a goal, as a Petri net. Every count below is
/// at most the forest's occurrences, which are fewer than 2³², so none
/// of them overflows its `u32`: a weight counts literals of one clause, a
/// marking's count literals of the goal, a place an atom or a class of
/// clauses.
pub(super) struct Program {
    /// The places: one per atom of the goal, then one per class of clauses
    /// used once, whose token is the class's unused clauses.
    places: usize,
    /// The place of each atom of the sequent, `u32::MAX` for an atom the
    /// goal does not name.
    place_of: Vec<u32>,
    /// The transitions: the clauses under `?` with distinct arcs, then
    /// the classes of clauses used once.
    transitions: Vec<Transition>,
    /// The arcs of the transitions, each a place and a weight, sorted by
    /// place within the inputs and within the outputs of a transition.
    arcs: Vec<(u32, u32)>,
    /// The marking the search starts from.
    initial: Vec<u32>,
    /// The marking to reach: the goal's atoms, and no clause left unused.
    target: Vec<u32>,
    /// The sign of the atoms in bodies and in the goal; heads and markings
    /// have the other.
    body: Sign,
    /// The goal's members under a `?`.
    quests: Vec<OccId>,
    /// The goal's members that are markings: `⅋` and `⊥` over head
    /// literals.
    markings: Vec<OccId>,
    /// The goal's member to reach: a tensor of body literals and `1`.
    goal: OccId,
    /// The occurrences of each class of clauses used once.
    once: Vec<Vec<OccId>>,
}

impl Program {
    /// Reads the Horn program a goal is, with the atoms of bodies written
    /// `a` or else with all of them written `~a`, or `None` when it is
    /// none: every member under a `?` a clause with exactly one head,
    /// every other member a marking, a clause with exactly one head, used
    /// once, or the goal, a clause without one, of which there is exactly
    /// one. A clause is a tensor of body literals and `1` with at most one
    /// factor a head, a `⅋` of head literals and `⊥`. In intuitionistic
    /// mode the reading must put the goal and the clauses' bodies on the
    /// right of `⊢` and everything else on the left, as the proof built
    /// for a firing sequence does.
    fn read(task: &Task<'_>) -> Option<Self> {
        [Sign::Var, Sign::DualVar]
            .into_iter()
            .find_map(|body| Self::read_with(task, body))
    }

    /// [`read`](Self::read) with the sign of the bodies' atoms given.
    fn read_with(task: &Task<'_>, body: Sign) -> Option<Self> {
        let forest = task.forest;
        let atoms = forest.sequent().atom_names().len();
        let mut reader = Reader {
            forest,
            body,
            place_of: vec![u32::MAX; atoms],
            places: 0,
        };
        let mut reusable = Vec::new();
        let mut once = Vec::new();
        let mut quests = Vec::new();
        let mut markings = Vec::new();
        let mut goal = None;
        let mut tokens = Vec::new();
        for &member in task.goal {
            if forest.kind(member) == Kind::Quest {
                let clause = forest.left(member)?;
                let (inputs, outputs, head) = reader.clause(clause)?;
                head?;
                reusable.push((clause, inputs, outputs));
                quests.push(member);
            } else if reader.head(member) {
                tokens.extend(reader.literals(member));
                markings.push(member);
            } else {
                let (inputs, outputs, head) = reader.clause(member)?;
                if head.is_some() {
                    once.push((member, inputs, outputs));
                } else if goal.replace(member).is_some() {
                    return None;
                }
            }
        }
        let goal = goal?;
        if let Some(reading) = task.reading
            && !task.goal.iter().all(|&member| {
                let head = if member == goal {
                    None
                } else if forest.kind(member) == Kind::Quest {
                    Some(reader.head_of(forest.left(member).unwrap()))
                } else {
                    Some(reader.head_of(member))
                };
                forest.subtree(member).all(|x| {
                    // An occurrence is on the left exactly when it holds
                    // the head or lies in it; a marking is its own head.
                    let left = head.is_some_and(|h| {
                        (x <= h && h.index() < x.index() + forest.size(x) as usize)
                            || (h <= x && x.index() < h.index() + forest.size(h) as usize)
                    });
                    reading.position(x)
                        == if left {
                            Position::Input
                        } else {
                            Position::Output
                        }
                })
            })
        {
            return None;
        }
        let target_tokens = reader.clause(goal)?.0;
        Some(reader.program(
            Parts {
                reusable,
                once,
                tokens,
                target: target_tokens,
            },
            quests,
            markings,
            goal,
        ))
    }
}

/// The members of a goal read as a net, before the transitions are
/// gathered: each clause under `?` and used once with its input and
/// output places, the tokens of the markings and the goal's.
struct Parts {
    /// The clauses under `?`.
    reusable: Vec<(OccId, Vec<u32>, Vec<u32>)>,
    /// The clauses used once.
    once: Vec<(OccId, Vec<u32>, Vec<u32>)>,
    /// A place per head literal of the markings.
    tokens: Vec<u32>,
    /// A place per body literal of the goal.
    target: Vec<u32>,
}

/// What reads the members of a goal: the forest, the sign of the bodies,
/// and the places the atoms read so far got.
struct Reader<'a> {
    /// The forest.
    forest: &'a Forest,
    /// The sign of the bodies' atoms.
    body: Sign,
    /// The place of each atom, `u32::MAX` before it is met.
    place_of: Vec<u32>,
    /// The places given out.
    places: u32,
}

impl Reader<'_> {
    /// Whether a subformula is a head: `⅋` and `⊥` over literals of the
    /// heads' sign.
    fn head(&self, o: OccId) -> bool {
        self.forest.subtree(o).all(|x| match self.forest.kind(x) {
            Kind::Par | Kind::Bot => true,
            _ => self.forest.sign(x) == Some(!self.body),
        })
    }

    /// The place of a literal's atom, given out when the atom is new.
    fn place(&mut self, literal: OccId) -> u32 {
        let atom = self.forest.atom(literal).expect("a literal").index();
        if self.place_of[atom] == u32::MAX {
            self.place_of[atom] = self.places;
            self.places += 1;
        }
        self.place_of[atom]
    }

    /// The places of the literals of a subformula, one per literal.
    fn literals(&mut self, o: OccId) -> Vec<u32> {
        let forest = self.forest;
        forest
            .subtree(o)
            .filter(|&x| forest.is_literal(x))
            .map(|x| self.place(x))
            .collect()
    }

    /// Reads a clause: the places of its body literals, of its head's
    /// literals, and its head, or `None` when it is no clause: a factor
    /// that is neither a body literal, `1`, a tensor nor a head, or two
    /// heads.
    fn clause(&mut self, clause: OccId) -> Option<(Vec<u32>, Vec<u32>, Option<OccId>)> {
        let forest = self.forest;
        let (mut inputs, mut head) = (Vec::new(), None);
        let mut factors = vec![clause];
        while let Some(x) = factors.pop() {
            match forest.kind(x) {
                Kind::Tensor => factors.extend(forest.children(x)),
                Kind::One => {}
                _ if forest.sign(x) == Some(self.body) => inputs.push(self.place(x)),
                _ if head.is_none() && self.head(x) => head = Some(x),
                _ => return None,
            }
        }
        let outputs = head.map_or_else(Vec::new, |h| self.literals(h));
        Some((inputs, outputs, head))
    }

    /// The head of a clause read before, or the clause itself for a
    /// marking.
    fn head_of(&self, clause: OccId) -> OccId {
        head_of(self.forest, self.body, clause).unwrap_or(clause)
    }

    /// Gathers the transitions: the clauses under `?` with distinct arcs
    /// that change the marking, and the clauses used once by class of
    /// equal arcs, each class with a place of its own that holds a token
    /// per clause, which a firing takes.
    fn program(
        self,
        parts: Parts,
        quests: Vec<OccId>,
        markings: Vec<OccId>,
        goal: OccId,
    ) -> Program {
        /// The arcs of a list of places, one per literal: each place with
        /// its multiplicity, sorted.
        fn weights(mut places: Vec<u32>) -> Vec<(u32, u32)> {
            places.sort_unstable();
            let mut arcs: Vec<(u32, u32)> = Vec::new();
            for p in places {
                match arcs.last_mut() {
                    Some((q, w)) if *q == p => *w += 1,
                    _ => arcs.push((p, 1)),
                }
            }
            arcs
        }
        let Reader {
            place_of,
            places,
            body,
            ..
        } = self;
        let mut seen: HashMap<(Arcs, Arcs), u32> = HashMap::default();
        let mut transitions = Vec::new();
        let mut arcs = Vec::new();
        let mut push = |clause, inputs: &[(u32, u32)], outputs: &[(u32, u32)]| {
            let at = |arcs: &Vec<(u32, u32)>| u32::try_from(arcs.len()).expect("arcs are literals");
            let inputs_at = at(&arcs);
            arcs.extend_from_slice(inputs);
            let outputs_at = at(&arcs);
            arcs.extend_from_slice(outputs);
            transitions.push(Transition {
                clause,
                inputs: inputs_at,
                outputs: outputs_at,
                end: at(&arcs),
            });
        };
        for (clause, inputs, outputs) in parts.reusable {
            let (inputs, outputs) = (weights(inputs), weights(outputs));
            if inputs != outputs && !seen.contains_key(&(inputs.clone(), outputs.clone())) {
                push(Clause::Reusable(clause), &inputs, &outputs);
                seen.insert((inputs, outputs), 0);
            }
        }
        seen.clear();
        let mut once: Vec<Vec<OccId>> = Vec::new();
        let mut classes = Vec::new();
        for (clause, inputs, outputs) in parts.once {
            let key = (weights(inputs), weights(outputs));
            let class = *seen.entry(key.clone()).or_insert_with(|| {
                once.push(Vec::new());
                classes.push(key);
                once.len() as u32 - 1
            });
            once[class as usize].push(clause);
        }
        let ticket = |class: usize| places + class as u32;
        for (class, (inputs, outputs)) in classes.iter().enumerate() {
            let mut inputs = inputs.clone();
            inputs.push((ticket(class), 1));
            push(Clause::Once(class as u32), &inputs, outputs);
        }
        let width = places as usize + once.len();
        let mut initial = vec![0; width];
        for p in parts.tokens {
            initial[p as usize] += 1;
        }
        for (class, clauses) in once.iter().enumerate() {
            initial[ticket(class) as usize] = clauses.len() as u32;
        }
        let mut target = vec![0; width];
        for p in parts.target {
            target[p as usize] += 1;
        }
        Program {
            places: width,
            place_of,
            transitions,
            arcs,
            initial,
            target,
            body,
            quests,
            markings,
            goal,
            once,
        }
    }
}

/// The head of a clause: its one factor that is neither a tensor, `1`
/// nor a body literal, if it has one.
fn head_of(forest: &Forest, body: Sign, clause: OccId) -> Option<OccId> {
    let mut factors = vec![clause];
    while let Some(x) = factors.pop() {
        match forest.kind(x) {
            Kind::Tensor => factors.extend(forest.children(x)),
            Kind::One => {}
            _ if forest.sign(x) == Some(body) => {}
            _ => return Some(x),
        }
    }
    None
}
