# Applications of linear logic, and where linlog's scope could grow

Written 2026-10-09 from a web search, after reading `plan/later.md`,
`plan/notes/research/README.md` ("the research README") and `practice.md` beside it. Sources are cited by key (list
at the end); "(inference)" marks a conclusion of this note's own. The
question is not which benchmark sets to fetch, which `practice.md`
settles, but which uses of linear logic a prover with linlog's abilities
could serve as a tool, and what that asks of the API now.

## 1. The field as it stands

**Linear logic programming.** Lolli gave a fragment of intuitionistic
linear logic (`⊤ & ⊸ ⊃ ∀`, without `⊗` and `!`) a goal-directed reading, and solved the
context-splitting problem by search that consumes part of a context and
returns the rest, the input-output model [HM94]. LolliMon added
forward-chaining, committed-choice computation inside a monad taken from
CLF, with Lolli's backward chaining outside it [LPPW05]. Celf implements
CLF and runs its signatures as concurrent logic programs [SS08];
Simmons's SLS treats forward-chaining substructural programs as rewriting
rules [Sim12]. Ceptre is the current descendant aimed at designers: rules
`A -o B` rewrite a state, programs are split into stages, a stage ends
when it is quiescent (no rule applies), a non-interactive stage picks
among enabled transitions at random, an `#interactive` stage lets the
player pick, and a run prints a trace that GraphViz can draw [Mar15,
CepT]. Interactive narrative is the application that drove it: Bosser,
Cavazza and Champagnat proposed linear implication as a model of
narrative causality and formal validation of scenarios [BCC10]; Martens
et al. generated stories as Celf proof terms and turned each into a
causal graph by resource-flow analysis [MBFC13], then moved to forward
chaining for emergent story worlds [MFBC14].

**Planning.** Masseron, Tollu and Vauzeilles read plans as proofs
[MTV93]; Kanovich tied the `!`-Horn fragment to Petri nets [Kan95], and
Kanovich and Vauzeilles characterised classical AI planning in Horn
linear logic with its complexity [KV01]. Proof-carrying plans make PDDL
plans into Agda programs whose types are the planning problem [Sch19],
and a later resource logic for them is proved sound against the
possible-worlds semantics of planning and formalised in Agda [HKP20].
On the planners' side, unsolvability certificates check a "no plan"
answer independently [ERH17], and the Petri-net state equation is an
admissible heuristic for SAS+ planning [Bon13]. Web-service composition
was cast as linear-logic theorem proving, the composite process read off
the proof [RKM03].

**Resources in programming and contracts.** Move has linear resource
types; a resource is created or destroyed only by its declaring module,
and its bytecode language is proved to enjoy resource safety, "a
conservation property … analogous to conservation of mass" [Bla20]. Its
functional properties are checked by the Move Prover through Boogie
(unsourced). Nomos combines session types from linear logic, amortised
resource analysis and linear assets for digital contracts [Das21];
Typecoin attaches affine-logic propositions to Bitcoin transactions for
proof-carrying authorization and executable contracts [CS15]. Linear
Haskell puts linearity on the arrow in GHC [Ber18]; synthesis from linear
types is proof search, in Granule with graded types [HO20] and in a GHC
type-hole plugin for linear Haskell [MT23].

**Session types.** Caires and Pfenning typed the π-calculus by dual
intuitionistic linear logic [CP10], with cut reduction matching process
reduction in the journal version [CPT16]; Wadler's CP does the same for
classical linear logic [Wad12].

**Security.** A linear logic of authorization and knowledge models
consumable resources and single-use authorizations, with cut elimination
[Gar06]; consumable credentials (open a door once, a concert ticket) were
implemented in a deployed system [Bow07]. Multiset rewriting grounded in
linear logic expresses the Dolev-Yao model, with existentials for fresh
nonces [Cer99]; Tamarin's states are multisets of facts, linear unless
prefixed `!`, which persist [Mei13, TamM].

**Chemistry and biology.** Reaction networks are Petri nets; persistence
follows when every siphon contains the support of a P-semiflow, the net's
form of a conservation law [ADS07]. Hybrid linear logic indexes truth by
constraints (time, rates) for biological transition systems [CD13, DC16];
a logical framework for systems biology witnesses reachability as
entailment, with proofs in Coq helped by a λProlog prover [dMDF14]; SELL
with subexponentials gives spatial and temporal dependencies [Ola16].
Despeyroux names "using automatic provers" as future work [Des16]. Coecke,
Fritz and Spekkens frame chemistry, thermodynamics and entanglement as
resource theories, asking which resources convert into which [CFS16].

**Quantum programs.** Selinger and Valiron type a quantum λ-calculus by
affine intuitionistic linear logic [SV06]; QWIRE gives circuit wires
linear types [PRZ17]; Silq's type system rejects unphysical programs
[Bic20]. Categorically, no-cloning is the absence of a natural diagonal
beside the structure entanglement needs [Abr09].

**What the map shows** (inference): proof search is the workhorse where
the answer is a witness: a program run, a plan, a story, an access proof,
a synthesised term, a reaction pathway. Where linearity is a type
discipline (Move, Silq, QWIRE, Linear Haskell), a type checker decides it
and there is no search problem for linlog, except synthesis.

## 2. What linlog already reaches

The Horn engine decides Kanovich's `!`-Horn programs: reachability in
linear mode, coverability in affine mode, refutation by exhausted
markings or by state-equation weights (`.claude/rules/core-horn.md`,
`Refutation::StateEquation`). The focused engine decides ILL and CLL with
the copy bound; interactive proving, the four exports, Rocq certificates
and the ordinary-logic layer exist; steps 31 to 38 add a Rocq library,
the web, cut, Lambek and first-order logic (the research README, §1). Planning
domains, MCC nets and programming sequents are already ranked as
benchmark sources (`practice.md` §2). So the gap is not inputs but
*outputs and front ends*: answers in the user's vocabulary, programs run
rather than goals proved, and witnesses an application can use.

## 3. Candidate extensions

### applications-1: Plans and their certificates from the Horn engine

**What.** Print a Horn proof as what a planner prints: the firing
sequence as a plan with action names, its partial order (which firing
consumed which token, the resource-flow graph of [MBFC13]), and for a
refutation the state-equation weights as an unsolvability certificate.
**Value.** Planning researchers get a plan checked by linlog's checker or
Rocq, the proof-carrying-plans goal [Sch19, HKP20] by search rather than
by hand, and a "no plan" answer with a certificate, which is what
[ERH17] asks of planners (inference). The weights are the state-equation
argument [Bon13] turned into a proof of impossibility (inference).
**Cost.** Small to medium: the firing sequence exists inside the engine;
the partial order is a pass over it; a PDDL plan writer and the
`.sas` reader of `practice.md` §5. **Relation.** Extends step 27, the
refutations note and `practice.md` rows 4 and 5. **Room now.** Names
that survive from input to output: a clause, hypothesis or root may carry
a label (an action, a reaction, a credential) that the proof, the
derivation view and the JSON repeat; the Horn engine's firing sequence
kept as data on the answer rather than only lowered into a `Proof`.

### applications-2: A rule-system front end: run, explore, play

**What.** A file of propositional (grounded) rewrite rules with an
initial state, as in Ceptre, and three commands beside `prove`: `run` to
quiescence with a seeded random choice, `explore` to list or count the
reachable states and dead ends, and a forward `interact` where the user
picks the next rule; stages as ordinary atoms that rules consume and
produce (inference: Ceptre's stage switch can be encoded so). **Value.**
Game and narrative designers, teaching of concurrency and Petri nets;
Ceptre's goal is analysis of generative systems [Mar15], and linlog adds
decided questions on the same rules: can this state ever be reached, is
this ending avoidable (coverability, refutation). Ceptre itself is not
open source (`practice.md` §2), so a maintained tool has room
(inference). **Cost.** Medium: a format and reader, a forward-step API
over the Horn engine's markings, a seeded generator (no clock, so wasm
stays possible, D11), a trace output reusing the SVG work. **Relation.**
New; beside step 9 (interactive proving is backward) and step 32 (the
web is where play happens). **Room now.** `Program` and a marking as
public, serialisable types with `enabled(marking)` and `fire`; `Stop`
polling a run that need not end; the SVG id scheme able to name states
and firings, as the research README's §4 item 6 already asks for state ids.

### applications-3: Many answers: enumerate and count proofs

**What.** Return every proof, or every distinct one up to a stated
equivalence (proof nets for MLL, firing multisets or partial orders for
Horn programs), lazily, with a count. **Value.** Narrative generation
draws many stories from one query [MBFC13]; synthesis wants alternative
programs [HO20]; teaching shows that Lolli's formula 17 has eight proofs
(`practice.md` §3). **Cost.** Medium to high: the engines stop at the
first proof, the memo tables assume one answer, and distinctness needs a
canonical key per fragment. **Relation.** New; uses step 35's nets and
step 34's identity of proofs under cut elimination. **Room now.** An
answer stream rather than one `Outcome` in the engine interface, with
the existing `Stop` and memory account, so a second answer is a call,
not a new entry point.

### applications-4: Proofs as programs and processes

**What.** Two exports over the proof term: an ILL proof as a linear
λ-term (or a Haskell function using linear arrows), a CLL proof as a CP
process [Wad12] or a DILL process [CP10]. **Value.** Synthesis: the proof
search is the synthesiser [MT23, HO20]; teaching Curry-Howard and
session types, where step 34's cut elimination becomes process reduction
[CPT16]. **Cost.** Medium for the propositional exports (a term
assignment per rule, fresh names); useful synthesis needs data types and
polymorphism [MT23], which is beyond propositional logic and waits on
step 38. **Relation.** Extends the exports (steps 10 to 12, 22) and step
34. **Room now.** The export entry over the term (the research README's §4 item 6)
general enough for a fifth and sixth target; a two-sided ILL view of a
proof with stable hypothesis names, which the labels of applications-1
give.

### applications-5: A linear logic programming mode after quantifiers

**What.** Lolli-style queries: a program of first-order clauses, a goal
with logic variables, and the answer substitutions, by backward chaining
with the input-output context model [HM94]. **Value.** Logic programming
in linear logic has no maintained batch implementation in linlog's
comparison (`29-comparison.md`, "Not batch provers": Lolli and Celf);
teaching of resource-sensitive programs (inference). **Cost.** High: all
of step 38, then answers as data and a goal-directed operational order
the user can predict, which a complete search does not promise.
**Relation.** Extends step 38. **Room now.** 38's witness arena readable
from the answer, so an answer substitution is a projection of it, not a
new structure.

### applications-6: Reaction networks and conservation laws

**What.** An SBML reader (and the PNML reader `practice.md` §5 proposes)
mapping reactions to `!`-Horn clauses with stoichiometric multiplicities;
questions: can a species be produced from a pool (coverability), is a
state reachable, with the state-equation weights reported as a
conservation law that forbids it. **Value.** Systems-biology and
chemistry researchers who want automatic provers for untimed models
[Des16, dMDF14]; resource-convertibility questions of finitely presented
resource theories are of the same shape (inference, from [CFS16]). The
weights are a conservation-law certificate in the sense of P-semiflows
[ADS07] (inference: a weighting no firing raises is a sub-invariant).
**Cost.** Medium: the reader, stoichiometry as counts, BioModels has no
questions (`practice.md` row 11), so problems must be posed. Time, rates
and space (HyLL, SELL) are out of scope. **Relation.** Extends step 27
and `practice.md` rows 1 and 11. **Room now.** A multiplicity on a
literal (`a^k`) in the Horn program, not `k` copies in the arena, with
counts under the 2⁵³ rule (the research README's §2, "Numbers above 2⁵³").

### applications-7: Consumable credentials and proof-carrying authorization

**What.** Access checks as ILL search: reusable credentials under `!`,
single-use ones linear, the request as the goal; the proof, checked by
linlog's checker or a Rocq certificate, is the authorization evidence,
listing which credentials it consumes. **Value.** Security research and
teaching on [Gar06, Bow07]; a small, fast decision procedure fits a
reference monitor (inference). **Cost.** Low for principals folded into
atom names; a faithful logic needs the affirmation and knowledge
modalities of [Gar06], new rules and a checker arm, which is new theory
for linlog. **Relation.** New. **Room now.** The labels of applications-1
(which credential), and `Kind`/`Rule` growth by the one checklist the
research README names (§2, "`Rule` grows by one checklist") so that a modality is
an added variant, not a redesign.

### applications-8: Bounded exponentials and budgets

**What.** `!ₙA`, a formula usable at most `n` times, as in bounded linear
logic [GSS92] and graded types [HO20], and a budget atom with `cⁿ` in
planning (`practice.md` §3, row 5). **Value.** Tickets valid three times
(not in the source), plans within a cost, graded-type synthesis problems; makes the
global copy bound a per-formula statement the user writes (inference).
**Cost.** Medium: a `Kind`, a rule, a checker arm, the Rocq kernel; for
a numeral `n`, `!ₙA` unfolds to `n` linear copies under `&`/`1`
choices (inference), so the gain is a compact representation and a
search that counts rather than copies. **Relation.** New; beside the copy
bound (D9's three-valued answer). **Room now.** The copy count of a
`Copy` node and of the dyadic zone as a number, not implied by repeated
nodes, so a bound per formula fits.

## 4. What the API design should leave room for now

Collected from the candidates, in order of how many need it:

1. **Labels from input to output** (1, 2, 4, 6, 7): an optional name on a
   root, hypothesis or clause, kept through `Forest`, `Proof`, the
   derivation view, the exports and the JSON, under `serde(default)`.
2. **Forward stepping as public API** (1, 2, 6): the Horn `Program`, its
   marking and the firing sequence as types of their own.
3. **An answer stream** (3, 5): the engine interface able to yield again
   under the same `Stop` and account.
4. **Counts as numbers** (6, 8): multiplicities on literals and copy
   counts, under the 2⁵³ rule.
5. **The export entry and `Rule` checklist extensible** (4, 7, 8):
   already the research README's §4 items 1 and 6; these candidates add two exports and
   up to three rule kinds.

## 5. Considered and rejected

- **Smart-contract verification (Move, Nomos).** Move's resource safety
  is a property of its bytecode language [Bla20], its specifications go
  to Boogie (unsourced), Nomos checks by typing [Das21]: no linear-logic
  search problem remains (inference). A contract's asset rules modelled
  as rewrite rules are served by applications-2.
- **Quantum programs.** Linearity of qubits is a type check [SV06,
  PRZ17, Bic20], not proof search; no-cloning as a sequent (`a ⊢ a ⊗ a`
  unprovable) is a teaching example linlog already proves unprovable
  (inference). Entanglement as a resource theory [CFS16] folds into
  applications-6.
- **Security-protocol analysis by multiset rewriting.** Needs fresh names
  (existentials) and unbounded terms [Cer99]; Tamarin [Mei13] is the
  maintained tool. Outside propositional scope and dominated.
- **Scheduling with time.** Time-indexed encodings were dropped
  (`practice.md`, PSPLIB and BPPLIB rows); timed LL (HyLL [CD13]) is a
  different logic; constraint solvers own the field (inference).
- **Hybrid and subexponential linear logics.** Natural for biology
  [DC16, Ola16], but new logics without a problem set; keep `Unary`
  kinds extensible rather than plan them.
- **Web-service composition.** The encoding is planning's [RKM03]; no
  maintained corpus was found, so applications-1 covers it.

## 6. Sources

- [Abr09] S. Abramsky, "No-Cloning in Categorical Quantum Mechanics", arXiv, 2009. https://arxiv.org/abs/0910.2401
- [ADS07] D. Angeli, P. De Leenheer, E. Sontag, "A Petri net approach to the study of persistence in chemical reaction networks", Mathematical Biosciences 210, 2007. https://arxiv.org/abs/q-bio/0608019
- [BCC10] A.-G. Bosser, M. Cavazza, R. Champagnat, "Linear logic for non-linear storytelling", ECAI 2010. https://research.tees.ac.uk/en/publications/linear-logic-for-non-linear-storytelling/
- [Ber18] J.-P. Bernardy et al., "Linear Haskell: practical linearity in a higher-order polymorphic language", POPL 2018. https://arxiv.org/abs/1710.09756
- [Bic20] B. Bichsel, M. Baader, T. Gehr, M. Vechev, "Silq: A High-Level Quantum Language with Safe Uncomputation and Intuitive Semantics", PLDI 2020. https://pldi20.sigplan.org/details/pldi-2020-papers/47/Silq-A-High-Level-Quantum-Language-with-Safe-Uncomputation-and-Intuitive-Semantics
- [Bla20] S. Blackshear, D. L. Dill, S. Qadeer, C. W. Barrett, J. C. Mitchell, O. Padon, Y. Zohar, "Resources: A Safe Language Abstraction for Money", arXiv, 2020. https://arxiv.org/abs/2004.05106
- [Bon13] B. Bonet, "An admissible heuristic for SAS+ planning obtained from the state equation", IJCAI 2013. https://www.ijcai.org/Abstract/13/335
- [Bow07] K. D. Bowers, L. Bauer, D. Garg, F. Pfenning, M. K. Reiter, "Consumable Credentials in Logic-Based Access-Control Systems", NDSS 2007. https://www.mpi-sws.org/~dg/papers/NDSS07Final.pdf
- [CD13] K. Chaudhuri, J. Despeyroux, "A Hybrid Linear Logic for Constrained Transition Systems with Applications to Molecular Biology", INRIA report, arXiv 2013. https://arxiv.org/abs/1310.4310
- [CepT] C. Martens, Ceptre tutorial, GitHub `chrisamaphone/interactive-lp`. https://github.com/chrisamaphone/interactive-lp/blob/master/tutorial.md
- [Cer99] I. Cervesato, N. Durgin, P. Lincoln, J. Mitchell, A. Scedrov, "A Meta-notation for Protocol Analysis", CSFW 1999. https://www.csl.sri.com/papers/csfw99/csfw99.pdf
- [CFS16] B. Coecke, T. Fritz, R. W. Spekkens, "A mathematical theory of resources", Information and Computation 250, 2016. https://arxiv.org/abs/1409.5531
- [CP10] L. Caires, F. Pfenning, "Session Types as Intuitionistic Linear Propositions", CONCUR 2010. https://www.springerprofessional.de/session-types-as-intuitionistic-linear-propositions/3352308
- [CPT16] L. Caires, F. Pfenning, B. Toninho, "Linear logic propositions as session types", Mathematical Structures in Computer Science 26, 2016. https://www.cambridge.org/core/journals/mathematical-structures-in-computer-science/article/linear-logic-propositions-as-session-types/810338DAF92DDBDA77C95DEB12FD1057
- [CS15] K. Crary, M. J. Sullivan, "Peer-to-peer Affine Commitment using Bitcoin", PLDI 2015. https://www.cs.cmu.edu/~crary/papers/2015/typecoin.pdf
- [Das21] A. Das, S. Balzer, J. Hoffmann, F. Pfenning, I. Santurkar, "Resource-Aware Session Types for Digital Contracts", CSF 2021. https://arxiv.org/abs/1902.06056
- [DC16] J. Despeyroux, K. Chaudhuri, "A Hybrid Linear Logic for Constrained Transition Systems", arXiv 2016. https://arxiv.org/abs/1603.02641
- [Des16] J. Despeyroux, "(Mathematical) Logic for Systems Biology", invited paper, CMSB 2016. https://arxiv.org/abs/1701.05063
- [dMDF14] E. de Maria, J. Despeyroux, A. Felty, "A Logical Framework for Systems Biology", FMMB 2014. https://arxiv.org/abs/1404.5439
- [ERH17] S. Eriksson, G. Röger, M. Helmert, "Unsolvability Certificates for Classical Planning", ICAPS 2017. https://ojs.aaai.org/index.php/ICAPS/article/view/13818
- [Gar06] D. Garg, L. Bauer, K. D. Bowers, F. Pfenning, M. K. Reiter, "A Linear Logic of Authorization and Knowledge", ESORICS 2006. https://www.mpi-sws.org/~dg/papers/aff-know.pdf
- [GSS92] J.-Y. Girard, A. Scedrov, P. J. Scott, "Bounded linear logic: a modular approach to polynomial-time computability", Theoretical Computer Science 97(1), 1992. https://www.site.uottawa.ca/~phil/papers/BLL.1992.pdf
- [HKP20] A. Hill, E. Komendantskaya, R. Petrick, "Proof-Carrying Plans: a Resource Logic for AI Planning", PPDP 2020. https://arxiv.org/abs/2008.04165
- [HM94] J. Hodas, D. Miller, "Logic programming in a fragment of intuitionistic linear logic", Information and Computation, 1994 (LICS 1991). https://www.lix.polytechnique.fr/~dale/papers/ic94.pdf
- [HO20] J. Hughes, D. Orchard, "Resourceful Program Synthesis from Graded Linear Types", LOPSTR 2020. https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7880237/
- [Kan95] M. Kanovich, "Petri nets, Horn programs, Linear Logic and vector games", Annals of Pure and Applied Logic 75, 1995. https://www2.informatik.uni-hamburg.de/tgi/pnbib/k/kanovich_m_i1.html
- [KV01] M. Kanovich, J. Vauzeilles, "The classical AI planning problems in the mirror of Horn linear logic: semantics, expressibility, complexity", Mathematical Structures in Computer Science 11(6), 2001. https://www.cambridge.org/core/journals/mathematical-structures-in-computer-science/article/classical-ai-planning-problems-in-the-mirror-of-horn-linear-logic-semantics-expressibility-complexity/DCF6386A35BAB48FF754400DAA0BE590
- [LPPW05] P. López, F. Pfenning, J. Polakow, K. Watkins, "Monadic concurrent linear logic programming", PPDP 2005. https://doi.org/10.1145/1069774.1069778
- [Mar15] C. Martens, "Ceptre: A Language for Modeling Generative Interactive Systems", AIIDE 2015. https://ojs.aaai.org/index.php/AIIDE/article/view/12784
- [MBFC13] C. Martens, A.-G. Bosser, J. F. Ferreira, M. Cavazza, "Linear Logic Programming for Narrative Generation", LPNMR 2013. https://research.tees.ac.uk/en/publications/linear-logic-programming-for-narrative-generation/
- [Mei13] S. Meier, B. Schmidt, C. Cremers, D. Basin, "The TAMARIN Prover for the Symbolic Analysis of Security Protocols", CAV 2013. https://link.springer.com/10.1007/978-3-642-39799-8_48
- [MFBC14] C. Martens, J. F. Ferreira, A.-G. Bosser, M. Cavazza, "Generative Story Worlds as Linear Logic Programs", INT7 2014. https://webarchive.di.uminho.pt/haslab.uminho.pt/jff/files/2014-generativestoryworldsllp.pdf
- [MT23] R. Mesquita, B. Toninho, "Functional Program Synthesis from Linear Types", INForum 2023. https://web.tecnico.ulisboa.pt/bernardo.toninho/papers/inforum23-synth.pdf
- [MTV93] M. Masseron, C. Tollu, J. Vauzeilles, "Generating plans in linear logic I. Actions as proofs", Theoretical Computer Science 113(2), 1993. https://hal.archives-ouvertes.fr/hal-00003530
- [Ola16] C. Olarte, D. Chiarugi, M. Falaschi, D. Hermith, "A proof theoretic view of spatial and temporal dependencies in biochemical systems", Theoretical Computer Science 641, 2016. https://www.sciencedirect.com/science/article/pii/S0304397516300184
- [PRZ17] J. Paykin, R. Rand, S. Zdancewic, "QWIRE: A Core Language for Quantum Circuits", POPL 2017. https://rand.cs.uchicago.edu/files/popl_2017.pdf
- [RKM03] J. Rao, P. Küngas, M. Matskin, "Application of Linear Logic to Web Service Composition", ICWS 2003. https://www.cs.cmu.edu/~jinghai/papers/icws03.pdf
- [Sch19] C. Schwaab, E. Komendantskaya, A. Hill, F. Farka, R. Petrick, J. Wells, K. Hammond, "Proof-Carrying Plans", PADL 2019. https://research-repository.st-andrews.ac.uk/handle/10023/16855
- [Sim12] R. J. Simmons, "Substructural Logical Specifications", PhD thesis, Carnegie Mellon University, 2012. https://www.cs.cmu.edu/~rwh/students/simmons.pdf
- [SS08] A. Schack-Nielsen, C. Schürmann, "Celf – A Logical Framework for Deductive and Concurrent Systems", IJCAR 2008. https://www.itu.dk/~carsten/papers/ijcar08.pdf
- [SV06] P. Selinger, B. Valiron, "A lambda calculus for quantum computation with classical control", Mathematical Structures in Computer Science 16(3), 2006. https://arxiv.org/abs/cs/0404056
- [TamM] Tamarin manual, "Protocol Specification: Rules". https://tamarin-prover.com/manual/master/book/005_protocol-specification-rules.html
- [Wad12] P. Wadler, "Propositions as Sessions", ICFP 2012; JFP 24(2–3), 2014. https://www.research.ed.ac.uk/en/publications/propositions-as-sessions/

Sources checked 2026-10-09: 43 checked, 1 corrected, 0 removed, 3 claims marked.
