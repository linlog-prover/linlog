# First-order linear logic in linlog's terms

Written 2026-10-08 from the snapshot of the repository and the sources of
section 8. It builds on `38-first-order.md` (the survey of provers,
complexity and the step's needs) and does not repeat it; where the two
differ, this note holds. "(inference)" marks a conclusion of this note's
own; every other result cites its source. Section 7 ends with what the
API design of step 28 must fix now.

## 1. Syntax

**Signature and terms.** A *signature* is a set of function symbols and
a set of predicate symbols with arities; a symbol is its name *and* its
arity, as in Prolog, so `p/0` and `p/1` are two symbols [BN98]. Terms
are variables, constants (nullary function symbols) and applications
`f(t₁, …, tₙ)`; a term without variables is *ground*. An *atom* is
`p(t₁, …, tₙ)` for a predicate `p/n`; today's atom `A` is the atom of
`A/0`, so every present sequent is a first-order sequent over nullary
predicates and no function symbol (inference: what lets D17's "pays
nothing" hold at the level of the data, section 5.5).

**Formulas.** First-order linear logic adds `∀x.A` and `∃x.A`, with
`(∀x.A)⊥ = ∃x.A⊥` and `(∃x.A)⊥ = ∀x.A⊥` [Gir87]. linlog keeps every
sequent one-sided in *negation normal form* (`core-sequents.md`): a
literal is `p(t̄)` or `~p(t̄)`, and negating a binder swaps it and
negates the body, which `Term::dual` does for the two new unary variants
as it swaps `!` and `?`; the parser's `finish` pass carries the negation
flag through a binder as through `!`.

**Free and bound variables, α-equivalence.** `∀x` and `∃x` bind `x` in
their body; an occurrence is bound by the innermost enclosing binder of
its name and free otherwise. Formulas differing only in bound names are
*α-equivalent*; provability is invariant under it and under renaming of
free variables, and the rules below are read up to it [BN98; Cha12].
linlog's input formulas are *closed*: an identifier in term position that
no binder binds is a constant (inference, the proposal; Prolog's implicit
universal closure is the alternative, section 7). Free variables then
exist only inside a search, as *metavariables* (logic, Herbrand or
existential variables) and *eigenvariables* (parameters).

**Two-sided ILL.** `Reading` reads `Γ ⊢ A` off the one-sided form (D1,
`core-forest.md`). Quantifiers do not flip the position, only an
implication's antecedent does, so a hypothesis `∀x.A` is a one-sided `∃`
over the negated body, both binders appear in both positions, and the
reading's grammar gains exactly that (inference from section 2.2).

**Proposed text syntax**, consistent with the grammar on `Sequent` and
README's syntax paragraph (the author decides, section 7):

| construct | ASCII | Unicode | rule |
|---|---|---|---|
| atom | `p(t₁, …, tₙ)`, `p` | | an identifier followed at once by `(` is a predicate with arguments; alone it is `p/0`, today's atom |
| term | `x`, `c`, `f(t₁, …)` | | inside argument lists only; bound by an enclosing quantifier: a variable, else a constant |
| universal | `forall x. A`, `forall x y. A` | `∀x. A` | binds looser than `-o`: the scope runs right to the closing `)`, the next `,`, the turnstile or the end |
| existential | `exists x. A` | `∃x. A` | as `forall` |
| negation | `~forall x. A`, `(forall x. A)^` | | read as `exists x. ~A` |

`forall` and `exists` are keywords only where a formula can start, as
`par` is a connective only where a connective can stand; inside an
argument list every identifier is a symbol. The dot is required.
Shadowing is allowed; a bound variable used as a predicate is an error.
No capitalisation convention: llprover's Prolog rule, capitals are
variables [Tam20], would make every present `A |- A` a sequent with a
free variable (inference). Numerals in argument position (`p(0)`) are an
open question. Printing restores bound names from the sequent's side
table (section 5.1), priming a clash: `⊢ ∃x. ~p(x), p(c)`.

## 2. The rules

### 2.1 One-sided CLL

With `a` an eigenvariable and `t` any term over the signature extended by
the eigenvariables so far [Gir87; LS94, Appendix B]:

```
  ⊢ Γ, A[a/x]                      ⊢ Γ, A[t/x]
  ───────────── ∀ (a not in Γ, A)   ───────────── ∃
  ⊢ Γ, ∀x.A                        ⊢ Γ, ∃x.A
```

The *eigenvariable condition*: `a` occurs free nowhere in the conclusion.
A derivation respecting it can be renamed so that every `∀` step has its
own eigenvariable. The identity `⊢ p(t̄), ~p(s̄)` demands equal argument
lists; the other rules are the propositional ones.

### 2.2 Two-sided ILL

Chaudhuri and Pfenning's four rules for the dyadic two-sided calculus,
with "the proviso that `a` may not occur in the conclusion" on `∀R` and
`∃L` [CP05, §2]:

```
  Γ;Δ ⇒ A[a/x]          Γ;Δ, A[t/x] ⇒ C        Γ;Δ ⇒ A[t/x]        Γ;Δ, A[a/x] ⇒ C
  ──────────── ∀Rᵃ      ──────────────── ∀L    ──────────── ∃R      ──────────────── ∃Lᵃ
  Γ;Δ ⇒ ∀x.A            Γ;Δ, ∀x.A ⇒ C          Γ;Δ ⇒ ∃x.A          Γ;Δ, ∃x.A ⇒ C
```

Under D1's lowering `∀R` and `∃L` are the one-sided `∀` (eigenvariable),
`∀L` and `∃R` the one-sided `∃` (term). The quantifier rules keep the
goal where it is, so the checker's one-succedent rules R1 to R3
(`core-proofs.md`) need no new case (inference). Lolli is the fragment
with `∀` in goals and clauses and `!` [HM94]; linlog's two-sided engine
has all four rules.

### 2.3 The focused forms

`∀` is asynchronous (negative, invertible: a fresh eigenvariable at once)
and `∃` synchronous (positive: the witness is a choice) in Andreoli's
classification as Liang and Miller restate it [And92; LM09]; "positive
universal and negative existential formulas can only ever be instantiated
with parameters in a cut-free backward sequent derivation" [CP05, §4].
In the engine's phases (`focus/mod.rs`): the asynchronous phase takes
`∀` like `⅋` and `&`, binding a fresh eigenvariable to the occurrence's
binder; `focus` takes `∃` like `⊕`, binding a fresh metavariable and
going on along the body; a focus that reaches a `∀` is released. With
the exponentials: `?∀x.A` puts `∀x.A` into `Θ`, and each *copy* focuses
and releases at once; `?∃x.A` in `Θ` gives every copy its own
metavariable; `!∀x.A` under focus needs `Γ` empty, then releases.
`Kind::polarity` is `Negative` for `Forall`, `Positive` for `Exists`;
a literal's polarity stays the per-predicate bias. The identity under
focus on `p(t̄)` looks not for *the* dual occurrence but for a member of
`Γ` or `Θ` whose instance *unifies* with `~p(t̄)`; several candidates
are a choice point the propositional engine never has (inference).

## 3. Proof search

### 3.1 Metavariables and unification

Backward search replaces the `∃` witness by a fresh metavariable and
decides it at the identities by first-order unification with the *occurs
check* (a variable is never bound to a term containing it), which keeps
the solution a finite term and, below, detects eigenvariable violations
[MM82; LS94, §2]. Unification is linear in the worst case [PW78],
near-linear with Martelli and Montanari's algorithm [MM82] and linear on
average for its variants [ACF]; the hash-consed arena of section 5.1
makes terms DAGs, so the occurs check marks visited nodes once per call
or is exponential on shared terms (inference). Bindings live in a
substitution beside the forest, never in the formulas (`plan/later.md`).

### 3.2 The eigenvariable condition

**Why it needs care.** A `∀` step's eigenvariable must not occur in its
conclusion, but with metavariables the conclusion is known only once
unification has bound them. Classical logic *Skolemizes* once, soundly
by the Herbrand theorem, which "cannot be applied directly to most
logics, including intuitionistic, linear, and modal logics" [LS94, §1;
Nad93]. **Static Skolemization is unsound for linear logic**: `⊢
(∃x.p⊥ ⅋ q⊥(x)), (∀y.q(y)) ⊗ p` is not provable, yet its Skolemized form
`⊢ p⊥ ⅋ q⊥(v), q(c) ⊗ p` is, with `v := c` [LS94, §2]. The two-sided
counterexamples `∀x.A ⊗ B(x) ⊢ A ⊗ ∀u.B(u)` and `∀x.!A(x) ⊢ !∀u.A(u)`
lose that `∀L` must precede `∀R` through `⊗L`, and through `!R`'s empty
context [BRS24, §1–2]. Lincoln and Shankar's quantifier
impermutabilities are `∃/&` (`⊢ (A(t)⊥ & A(u)⊥), ∃x.A(x)`: the witness
differs per branch) and `∀/∃` (`⊢ ∀y.(A(y) ⊗ B), ∃x.A(x)⊥`), the latter
"present in most first-order logics" [LS94, Appendix A].

**Dynamic Skolemization (LLV).** The condition is kept inside the
search: a `∀` step introduces `h(T)`, `h` new and `T` *all*
metavariables introduced so far on the branch, so that binding one of
them to a term containing `h` fails the occurs check, "indicating a
potential transgression of the eigenvariable condition" [LS94, §2–3].
LLV is sound and complete for LL (Theorem 3.3), and the soundness
argument "applies to any cut-free sequent calculus with the subformula
property and conventional quantifier rules" [LS94, §4]; its cost is
backtracking over the order of quantifier steps. **LLO** reduces `T` to the variables that *govern* the
formula through the three impermutable rules `!`, `⅋`, `&`, so that "in
the pure multiplicative fragment, all quantifier rules can be applied
immediately" [LS94, §4]; BRS24 reach the same for focused ILL with Skolem
terms annotated by branch and world, checked as admissibility of the
unifier [BRS24, §3]. Neither reports an implementation beyond LS94's
Prolog prototypes [LS94, §5; BRS24]. **Raising** is the dual: the
metavariable becomes "a new existential variable of higher type" over
the eigenvariables above it, as Isabelle lifts a rule over a goal's
parameters [Mil92, §1, §5; Pau89, §6]; it needs higher-order terms and is
not the device here (inference).

**Levels: LLV with an integer.** Nadathur, Jayaraman and Kwon implement
the same dependency in λProlog with a *universe index*: each `∀` raises
the index and tags its new constant with it, a metavariable gets the
index of its creation, and binding a variable of tag `i` to `t` requires,
"in addition to the usual occurs-check", that `t` contain no constant of
a greater tag; the variables in `t` with a greater tag are lowered to `i`
[NJK95, pp. 15–16; Nad93]. This is `38-first-order.md`'s recommendation
made precise (inference): a level per metavariable and per eigenvariable,
the eigenvariable's one more than the branch's, which it raises; check
and lowering ride on the occurs check's walk; the lowering is trailed.
Soundness and completeness are LLV's [LS94, Theorem 3.3], since a
variable is older than an eigenvariable exactly when LLV would have put
it under `h`; proofs keep real terms, and the propositional path never
reads a level. LLO's finer dependency, which one integer cannot express,
is the measured follow-up.

### 3.3 The trail

Bindings are a `Vec<Option<TermRef>>` indexed by metavariable; every
binding and every lowering pushes its variable (and the old level) on a
*trail*; a choice point records the trail length, the search arena's
length and the two counters; undoing pops to the mark and truncates the
arena. That is the WAM's trail [War83; AK91], where only bindings older
than the choice point need recording [AK91]; linlog may trail everything
first and measure.

### 3.4 Sharing between premises, and step 15's prunes

A metavariable introduced below a binary rule occurs in both premises
(`∃x.(p(x) ⊗ q(x))` gives `p(X) ⊗ q(X)`), so the premises of `⊗` and `&`
are *not* independent: the right runs under the left's bindings, and a
failure on the right backtracks into the left for another unifier, as a
Prolog conjunction does [AK91]. For `core-focus.md`'s invariants
(inference, each): the *count* prunes stay sound keyed by predicate
symbol; *interchangeable occurrences* rest on "no rule looks at an
occurrence's id", so a class is `(term, frame, position)` and framed
members get no canonical key; the *loop check* compares by instance; the
*parallel* `&` and the cubes split only premises sharing no unbound
metavariable, which with section 5.3's memo means ground subgoals.

### 3.5 Decidable and undecidable fragments

| fragment | status | source |
|---|---|---|
| MLL1 | NP-complete: linearly many axioms, each sequent at most quadratic, unification linear | [LS94, Thm 3.5] |
| ALL1 | NP-complete | [HH15] |
| MALL1 | NEXPTIME-complete: depth bounded by the conclusion's connectives | [LS94, Thm 3.4; LSc94] |
| MELL1, LL1 | "full linear logic (with !, ?) … do not exhibit a finite depth bound on sequent proofs, due to the contraction rule" | [LS94, §4.1] |
| LL1 | undecidable, as propositional LL is | [LMSS92] |
| MELL (propositional) | open; Bimbó's decidability claim refuted | [Str19] |
| MELL1 | undecidable (inference): first-order Horn programs under `!` embed in IMELL1 with provability preserved [HM94; Mil21], and Horn clause provability is Turing-complete [Tär77] |
| MLL with first- and second-order quantifiers | undecidable | [LSS95] |

So the copy bound and the three-valued answer carry over unchanged to
every fragment with exponentials, and MLL1 and MALL1 are decided by the
bounded search alone (D17; `plan/later.md`).

## 4. Proof terms

### 4.1 What a proof records

Two variants keep `Node` at 16 bytes: `Forall(OccId, Eigen, NodeId)` with
a `u32` eigenvariable id, and `Exists(OccId, Witness, NodeId)` with a
`u32` index into a *witness arena* the `Proof` owns: terms over the
sequent's symbols, the proof's eigenvariables and, if the author allows
them, *open* metavariables the search left unbound (section 7).
Witnesses are recorded, not recovered: Hughes shows that a canonical
first-order proof object must leave them implicit (unification nets, with
local linear-time cut elimination), explicit witnesses making Girard's
first-order nets non-canonical [Hug18]; for first-order ALL witness nets
and unification nets are the two choices [HHS19]. linlog's term is a
sequent proof, so two proofs equal up to witnesses are two terms; the
net engine's later first-order form can be a unification net
(inference). Eigenvariables are named by index, not bound in the term:
binders with freshness as α-equivalence [HHP93] would not fit a 16-byte
node (inference).

### 4.2 How a checker verifies it

Two passes replace the propositional one when the forest has binders,
and only then (D17).

*Top-down frames.* The bottom-up pass (`check.rs`, `Pass`) derives each
node's sequent from its premises, but a member's *instance* depends on
the binder nodes *below* it towards the root, unseen when the pass
reaches a leaf. So a first pass in reverse arena order assigns every
node its members' *frames*: the roots get the empty frame; `Forall(o, a,
p)` gives the body `o+1` the frame of `o` extended by `a`, `Exists(o, w,
p)` by `w`; every other rule passes frames on unchanged. A node reached
from two parents (a memo hit) must receive equal frames, else
`Problem::Frames` (inference; section 5.3's ground-only memo never
produces the other case). Frames are interned per check, so the
bottom-up pass compares `(OccId, FrameId)` pairs as it compares ids
today: `Ax` needs the two instances syntactically equal with
eigenvariables as constants, `&` equal zones by instance; `⊗`, Mix, `!`,
`⊤` and `any` are unchanged.

*The eigenvariable condition.* It cannot be checked on the derived
conclusion: a `⊤` above absorbs members the pass never sees
(`core-proofs.md`, the `any` flag). The claim for the panel, from
`38-first-order.md`: a proof is valid only if every eigenvariable `a` is
introduced by exactly one `Forall` node `N`, occurs in no root, and
occurs only in witnesses of `Exists` nodes reachable from `N`. *Sound*
(inference): a member of `N`'s conclusion, visible or absorbed, has a
frame built by nodes between the root and `N`, so `a` could enter it only
through a witness chosen outside `N`'s subtree, which the rule forbids.
*Complete up to renaming* (inference): an `∃` outside `N`'s subtree
whose witness mentions `a` lies on a path where `a` is a free constant,
and renaming it there keeps the derivation valid. Memo-shared subproofs
unfold `N` twice with the same `a`; the condition is local to each
unfolded `N`, so the check counts introductions by node, not by path.
Implemented as a set of eigenvariables flowing up with the `State`:
`Exists` adds its witness's, `Forall` removes its own, the root ends
empty. *Witnesses* are over the sequent's symbols with their arities,
the proof's eigenvariables and, if allowed, open metavariables, which
the checker treats as constants outside the signature (sound: a proof
over new constants is a proof; inference).

### 4.3 Cut (step 34)

With `Cut(OccId, NodeId, NodeId)` on dual trees after the roots
(`34-cut.md`), the key case `∀`/`∃` substitutes the witness `w` for the
eigenvariable `a` throughout the `∀` premise, by the substitution lemma
on derivations [Gir87]; Pfenning's structural proof for linear sequent
calculi is the form a Rocq mirror can follow [Pfe95]. For linlog's term
that is a substitution over the witness arena and the frames of a
subproof, well defined because `a` is introduced once and used only
inside `N`'s subtree; a shared subproof under the cut is copied before
it is substituted, or the other parent sees `w` for `a` (inference). A
substitution adds no node, so bounded elimination stays bounded.

## 5. Consequences for the data model

### 5.1 Binders and terms in the arena

*Formulas.* `Term` gains `Pred(Atom, Args)`, `DualPred(Atom, Args)`,
`Forall(TermId)` and `Exists(TermId)`; two `u32` payloads keep its size;
`Var`/`DualVar` stay as the nullary case or become `Pred` with empty
`Args` (the spike decides); `Kind` gains the four tags, `arity` 0, 0, 1,
1, `is_literal` true for the predicates.

*First-order terms.* A second arena per `Sequent`: `FoTerm::Bound(u32)`
(a de Bruijn index, innermost binder 0 [dB72]) and `FoTerm::App(Symbol,
Args)` (constants with empty `Args`), `Args` an index into a CSR table of
argument lists, `Symbol` into a table of `(name, arity)`. Bound variables
nameless, free ones named by id: the *locally nameless* representation
[Cha12]. A closed subformula is α-canonical, so `Sequent::optimize`
hash-conses `∀x.p(x)` and `∀y.p(y)` into one term with no renaming,
maximal sharing by construction [CF06]; bound names live in a side table
from binder `TermId` to name, for printing only, the first name winning
a merge. `add` offsets and merges the symbol table as it merges atoms;
`verify_integrity` checks the new indices; `sizes` counts a binder as
one node.

*Instantiation never rewrites.* Opening a binder binds the occurrence's
binder slot to a term in the *search arena*; the instance of an
occurrence is its term with `Bound(i)` read through its *frame*, the
terms its enclosing binders are bound to. The sequent's arenas are
read-only during search.

### 5.2 Occurrences, copies and zones

The forest is untouched by binders as unary occurrences (preorder,
`left(o) = o+1`, CSR literal lists by predicate symbol and sign); a
binder-depth array is added only when the sequent has quantifiers. Under
contraction one body occurrence is instantiated several times: each copy
from `Θ` opens its binder afresh, so a zone member is `(OccId, FrameId)`
with frames hash-consed in the search arena, `Γ` a multiset and `Θ` a
set of such pairs. Today's `Context` (bitset plus extra copies) and
`OccSet` cannot carry a frame per member. The requirement (inference):
the engine generic over its zone type, `Engine<Z: Zone>`, `Z = Context`
propositionally and a framed zone (a bitset over the occurrences of
empty frame plus a sorted vector of framed members) otherwise; the
memo's `Key<Z>` and `Classes` follow. Step 37's lift of `Context` and
`Classes` to `search/` is the same move (these notes' `README.md`,
conflict 6).

### 5.3 Memo keys up to renaming

A memo entry is a fact only if it holds for every continuation. A
`Complete` failure of a stable sequent with an *unbound* metavariable is
not one: a sibling may bind it later and the key then names another
sequent. Rule for the first version (`38-first-order.md`, made precise):
memoize only *ground* stable sequents, keyed by instance; ground keys are
level-independent, since inside the subtree every new eigenvariable is
younger than every variable present (inference). Two ground sequents
differing by a renaming of eigenvariables are *variants*, terms that
"can be made identical through variable renaming", tabled logic
programming's default check [CW96; XSB]; a `Complete` failure may be shared under the
variant key (eigenvariables renamed in order of first occurrence), a
`Proved` entry may not, since its subproof names them, mirroring today's
rule for interchangeable members (`core-focus.md`). Every propositional
key is ground, so nothing changes there.

### 5.4 JSON

Additive keys under `#[serde(default)]`, skipped when empty, so a
propositional file reads and writes as before (D18; these notes'
wire-form policy). Sequent: tags `P` and `N` for a predicate and its
dual, `[atom, [fo-term ids]]`; `∀` and `∃` with the body's index; keys
`fo_terms` (`{"B": i}` bound, `{"S": [symbol, [ids]]}` application),
`symbols` (`[name, arity]`), `arities` for `var_dict` (absent: all zero)
and `binders` (binder index to name). Proof: `{"∀": [o, a, p]}`, `{"∃":
[o, w, p]}`, `witnesses` as `fo_terms` plus `{"E": k}` (eigenvariable)
and `{"M": k}` (open metavariable). Interactive state: members as `[id,
frame]` and a `bindings` key. `Fragment` gets a sixth flag `QUANTIFIERS`
and the names `MLL1`, `MALL1`, `MELL1`, `LL1` (`IMLL1`, … intuitionistic).
A test reads a pre-step file of each form unchanged.

### 5.5 What the propositional case pays (D17)

Nothing in the data: `Term`, `Kind` (a `u8`), `Node`, `OccId`, `OccSet`
and the forest's arrays keep their sizes; the first-order arenas, the
symbol table, the binder depths and the witness arena are empty;
`Proof::new` takes the witness arena and `Answer::of_arena` passes it.
Nothing in the hot loop: the engine is monomorphised at `Z = Context`,
with no frame, level or unification; `Decide::admits` sends a
`QUANTIFIERS` sequent to the focused engines' framed instance and every
other engine refuses it; the checker's frame pass and eigenvariable set
run only with binders; the parser's binder stack is consulted at no
identifier while empty. What this does *not* settle is code size and the
instruction cache, which the spike (stage 2, item 4) measures: target-set
counters identical, pinned CPU time within a few percent.

## 6. Worked examples

Occurrence ids are preorder over the written root order (the design
stops sorting roots, these notes' conflict 2); `o+1` is a binder's body.
Terms are listed premises first, root last, as `Proof` stores them;
`wₖ` are witnesses, `a` an eigenvariable, `X`, `Y` metavariables with
their levels as superscripts.

**E1. `forall x. p(x) |- p(c)`**, `⊢ ∃x. ~p(x), p(c)`: 0 `∃x.~p(x)`, 1
`~p(x)`, 2 `p(c)`. Stable at once; focus 0 with `X⁰`; `Ax` binds `X :=
c`. Term: `n0 = Ax(1, 2)`, `n1 = Exists(0, w0 = c, n0)`. The frame pass
gives 1 the frame `[c]`, so `Ax` compares `~p(c)` with `p(c)`. JSON:
`{"proof": [{"ax": [1, 2]}, {"∃": [0, 0, 0]}], "witnesses": [{"S": [1,
[]]}]}` with `symbols: [["p", 1], ["c", 0]]`.

**E2. `forall x. p(x) |- forall y. p(y)`**, `⊢ ∃x. ~p(x), ∀y. p(y)`: 0
`∃x.~p(x)`, 1 `~p(x)`, 2 `∀y.p(y)`, 3 `p(y)`. Asynchronous: `∀` at 2
with `a¹`; stable `⊢ 0, p(a)`; focus 0 with `X¹`; `Ax` binds `X := a`,
allowed since `X` is not older than `a`. Term: `n0 = Ax(1, 3)`, `n1 =
Exists(0, w0 = a, n0)`, `n2 = Forall(2, a, n1)`: `a` introduced by `n2`
only, used inside its subtree, in no root. Opening the `∃` first would
ask `X⁰ := a¹`, refused: that is the condition.

**E3. `forall x. (p(x) -o p(s(x))), p(z) |- p(s(z))`** (MLL1 with a
function symbol), `⊢ ∃x. (p(x) ⊗ ~p(s(x))), ~p(z), p(s(z))`: 0 `∃x.(…)`,
1 `⊗`, 2 `p(x)`, 3 `~p(s(x))`, 4 `~p(z)`, 5 `p(s(z))`. Focus 0 with
`X⁰`; `⊗` splits `{4, 5}`: left `⊢ p(X), ~p(z)` binds `X := z` (trailed);
right `⊢ ~p(s(X)), p(s(z))` closes under it. Term: `n0 = Ax(2, 4)`, `n1
= Ax(3, 5)`, `n2 = Tensor(1, n0, n1)`, `n3 = Exists(0, w0 = z, n2)`.

**E4. `!forall x. (p(x) -o p(s(x))), p(z) |- p(s(s(z)))`** (two copies,
two frames of one body), `⊢ ?∃x. (p(x) ⊗ ~p(s(x))), ~p(z), p(s(s(z)))`:
0 `?`, 1 `∃x.(…)`, 2 `⊗`, 3 `p(x)`, 4 `~p(s(x))`, 5 `~p(z)`, 6
`p(s(s(z)))`. `?` moves 1 into `Θ`; copy 1 with `X₁`: left `⊢ p(X₁),
~p(z)`, `X₁ := z`; right `⊢ ~p(s(z)), p(s(s(z)))` is stable, copy 1 again
with `X₂`: left `⊢ p(X₂), ~p(s(z))`, `X₂ := s(z)`; right closes. Term:
`n0 = Ax(3, 5)`, `n1 = Ax(3, 4)`, `n2 = Ax(4, 6)`, `n3 = Tensor(2, n1,
n2)`, `n4 = Exists(1, w1 = s(z), n3)`, `n5 = Copy(1, n4)`, `n6 =
Tensor(2, n0, n5)`, `n7 = Exists(1, w0 = z, n6)`, `n8 = Copy(1, n7)`, `n9
= Quest(0, n8)`. The frame pass gives `n1` the members `(3, [s(z)])` and
`(4, [z])`, the body occurrences `n0` and `n2` hold under other frames;
`Ax(3, 4)` is `p(s(z))` against `~p(s(z))`. A propositional zone could
not state this sequent.

**E5. `p(c) + p(d) |- exists x. p(x)`**, `⊢ ~p(c) & ~p(d), ∃x. p(x)`: 0
`&`, 1 `~p(c)`, 2 `~p(d)`, 3 `∃x.p(x)`, 4 `p(x)`. `&` first, then each
branch focuses 3 with its own metavariable. Term: `n0 = Ax(4, 1)`, `n1 =
Exists(3, w0 = c, n0)`, `n2 = Ax(4, 2)`, `n3 = Exists(3, w1 = d, n2)`,
`n4 = With(0, n1, n3)`. One occurrence, two witnesses, no contraction:
witnesses belong to nodes, not occurrences; this is the impermutability
`∃/&` [LS94, Appendix A].

**E6. `forall x. (p(x) -o q(x)), forall x. p(x) |- forall y. q(y)`**
(two-sided `∀R`, `⊸L`, `∀L` twice), `⊢ ∃x. (p(x) ⊗ ~q(x)), ∃x. ~p(x),
∀y. q(y)`: 0 `∃x.(…)`, 1 `⊗`, 2 `p(x)`, 3 `~q(x)`, 4 `∃x.~p(x)`, 5
`~p(x)`, 6 `∀y.q(y)`, 7 `q(y)`. `∀` at 6 with `a¹`; focus 0 with `X¹`;
`⊗`: right `⊢ ~q(X), q(a)` binds `X := a`; left `⊢ p(a), 4` is stable,
focus 4 with `Y¹ := a`. Term: `n0 = Ax(5, 2)`, `n1 = Exists(4, w1 = a,
n0)`, `n2 = Ax(3, 7)`, `n3 = Tensor(1, n1, n2)`, `n4 = Exists(0, w0 = a,
n3)`, `n5 = Forall(6, a, n4)`. The two-sided view prints `∀R` on `n5`,
`∀L` on `n4` and `n1`, `⊸L` on `n3`; the goal `q(a)` stays in the
`⊸L`'s right premise, as the one-succedent condition asks.

**U. `⊢ ∃x. (~p ⅋ ~q(x)), (∀y. q(y)) ⊗ p`** is unprovable [LS94, §2].
The `⅋` must come before the `⊗` can separate `~p` from `~q(x)`, and the
`∃` before the `⅋` (subformula property), so the `∀` lies above the `⊗`
in the branch `⊢ ~q(X⁰), ∀y. q(y)`, introduces `a¹`, and the identity
demands `X := a`, which the level check (LLV's occurs check on `h(X)`)
refuses; every other split fails on a count: `Unprovable`, a `Complete`
failure. Static Skolemization, `⊢ ~p ⅋ ~q(X), q(c) ⊗ p`, succeeds with
`X := c`. (The drinker's formula `⊢ ∃x. ∀y. (~p(x) ⅋ p(y))` fails the
same way in MLL1, where classical logic proves it by contraction;
inference.)

## 7. Open questions

1. **The text syntax** (section 1): `forall`/`exists` with `∀`/`∃` and a
   required dot; closed formulas with unbound identifiers as constants,
   or Prolog's implicit closure; numerals in argument position; whether
   `p` and `p(x)` coexist as `p/0` and `p/1`. The author decides.
2. **Sorts.** Untyped terms (the Lambek embedding needs none [MP01]), the
   proposal, or a sorted signature later.
3. **Open metavariables in a found proof**: a fresh constant, as LS94
   do, or `{"M": k}` treated as a constant by the checker (section 4.2).
4. **Levels first, LLO's dependency later**: the panel judges the
   reduction to LLV's `h(T)` (section 3.2); the families measure the
   finer dependency's gain.
5. **The eigenvariable rule** of section 4.2, a claim to prove or
   refute, with shared subproofs and the `any` flag in view.
6. **Memo**: ground-only first; variant keys for `Complete` failures as
   a measured option.
7. **Several unifiable candidates at the identity**: a choice point, or
   a prune by the atom bias; measured.
8. **Interactive witnesses**: how a witness given by hand is parsed (new
   constants?), and an open witness as a metavariable that `close`
   extends.
9. **Exports**: `rocq` answers `Unsupported` until step 31's library has
   binders, which it should have from the start (D20); `ViewOptions`
   decide whether witnesses show and how variables are named.

**What the API design (stage 2) must fix now**, so that step 38 adds
without reshaping: (a) `Term` with two `u32` payloads and room for four
variants, `Kind` the per-occurrence tag, both exhaustive with a 0.y
bump; (b) a second arena and a symbol table on `Sequent`, empty in the
propositional case; (c) the zone type as a parameter of the focused
engine and the memo key, `Context` one instance, `(OccId, frame)` members
the other; (d) `Node` at 16 bytes with two variants reserved, `Proof::new`
taking a witness arena; (e) the checker as two passes, the frame pass
only with binders, `State` carrying a set of eigenvariables; (f)
`Fragment` with a sixth flag and the `…1` names; (g) `Task` and the
interactive inference sequents naming a frame beside an id; (h) additive
JSON keys with a pre-step-file test per form; (i) a public `Walk` with a
binder stop and a term walk; (j) `Decide::admits` refusing `QUANTIFIERS`
in every engine but the focused ones.

## 8. Sources

- [And92] J.-M. Andreoli, *Logic programming with focusing proofs in linear logic*, J. Logic and Computation 2(3), 1992, 297–347. DOI 10.1093/logcom/2.3.297; the quantifier polarities are taken from its restatement in [LM09].
- [LM09] C. Liang, D. Miller, *Focusing and polarization in linear, intuitionistic, and classical logics*, Theoretical Computer Science 410, 2009. <https://www.lix.polytechnique.fr/~dale/papers/tcs09fixed.pdf>
- [Gir87] J.-Y. Girard, *Linear logic*, Theoretical Computer Science 50(1), 1987, 1–102. DOI 10.1016/0304-3975(87)90045-4
- [HM94] J. S. Hodas, D. Miller, *Logic programming in a fragment of intuitionistic linear logic*, Information and Computation 110(2), 1994, 327–365. <https://lics.siglog.org/1991/HodasMiller-Logicprogrammingina.html> (LICS 1991 version)
- [CP05] K. Chaudhuri, F. Pfenning, *A focusing inverse method theorem prover for first-order linear logic*, CADE-20, LNCS 3632, 2005. <https://www.cs.cmu.edu/~fp/papers/cade05.pdf>
- [LS94] P. Lincoln, N. Shankar, *Proof search in first-order linear logic and other cut-free sequent calculi*, LICS 1994, 282–291. <https://csl.sri.com/papers/lics94/lics94.pdf>
- [LSc94] P. Lincoln, A. Scedrov, *First-order linear logic without modalities is NEXPTIME-hard*, Theoretical Computer Science 135(1), 1994, 139–153. <https://www.csl.sri.com/~lincoln/papers/mall1-hard.pdf>
- [LSS95] P. Lincoln, N. Shankar, A. Scedrov, *Decision problems for second-order linear logic*, LICS 1995, 476–485. <https://lics.siglog.org/1995/LincolnShankarScedr-DecisionProblemsFor.html>
- [LMSS92] P. Lincoln, J. Mitchell, A. Scedrov, N. Shankar, *Decision problems for propositional linear logic*, Annals of Pure and Applied Logic 56, 1992, 239–311. <https://www.csl.sri.com/papers/lmss90/lmss90.pdf>
- [HH15] W. Heijltjes, D. Hughes, *Complexity bounds for sum-product logic via additive proof nets and Petri nets*, LICS 2015. <https://lics.siglog.org/2015/HeijltjesHughes-ComplexityBoundsfor.html>
- [HHS19] W. Heijltjes, D. Hughes, L. Straßburger, *Proof nets for first-order additive linear logic*, FSCD 2019, LIPIcs 131, 22:1–22:22. <https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.FSCD.2019.22>
- [Hug18] D. J. D. Hughes, *Unification nets: canonical proof net quantifiers*, LICS 2018, 540–549. DOI 10.1145/3209108.3209159; <https://arxiv.org/abs/1802.03224>
- [Str19] L. Straßburger, *On the decision problem for MELL*, Theoretical Computer Science 768, 2019, 91–98; INRIA RR-9203, 2018. <https://www.lix.polytechnique.fr/~lutz/papers/RR-9203.pdf>
- [Tär77] S.-Å. Tärnlund, *Horn clause computability*, BIT 17, 1977, 215–226. DOI 10.1007/BF01932293
- [Mil21] D. Miller, *A survey of the proof-theoretic foundations of logic programming*, arXiv 2109.01483, 2021. <https://arxiv.org/pdf/2109.01483>
- [BRS24] A. Bruni, E. Ritter, C. Schürmann, *Skolemisation for intuitionistic linear logic*, IJCAR 2024, LNCS 14740. <https://arxiv.org/abs/2405.01375>
- [Mil92] D. Miller, *Unification under a mixed prefix*, J. Symbolic Computation 14, 1992. <https://www.lix.polytechnique.fr/~dale/papers/jsc92.pdf>
- [Pau89] L. C. Paulson, *The foundation of a generic theorem prover*, J. Automated Reasoning 5(3), 1989, 363–397. <https://arxiv.org/abs/cs/9301105>
- [Nad93] G. Nadathur, *A proof procedure for the logic of hereditary Harrop formulas*, J. Automated Reasoning 11(1), 1993, 115–145. DOI 10.1007/BF00881902
- [NJK95] G. Nadathur, B. Jayaraman, K. Kwon, *Scoping constructs in logic programming: implementation problems and their solution*, J. Logic Programming, 1995. <https://arxiv.org/abs/cs/9809016>
- [MM82] A. Martelli, U. Montanari, *An efficient unification algorithm*, ACM TOPLAS 4(2), 1982, 258–282. <https://lara.epfl.ch/w/_media/sav08/unification-p258-martelli.pdf>
- [PW78] M. S. Paterson, M. N. Wegman, *Linear unification*, J. Computer and System Sciences 16(2), 1978, 158–167. DOI 10.1016/0022-0000(78)90043-0
- [ACF] L. Albert, R. Casas, F. Fages, *Average-case analysis of unification algorithms*, Theoretical Computer Science 113(1), 1993, 3–34 (STACS 1991 version: LNCS 480). <https://researchportal.ip-paris.fr/en/publications/average-case-analysis-of-unification-algorithms-2/>
- [BN98] F. Baader, T. Nipkow, *Term Rewriting and All That*, Cambridge University Press, 1998. <https://www.proof.cit.tum.de/~nipkow/TRaAT>
- [War83] D. H. D. Warren, *An abstract Prolog instruction set*, SRI Technical Note 309, 1983. <https://www.sri.com/publication/cyber-formal-methods-pubs/an-abstract-prolog-instruction-set/>
- [AK91] H. Aït-Kaci, *Warren's Abstract Machine: a tutorial reconstruction*, MIT Press, 1991. <https://lara.epfl.ch/w/_media/cc09/wambook.pdf>
- [CW96] W. Chen, D. S. Warren, *Tabled evaluation with delaying for general logic programs*, J. ACM 43(1), 1996, 20–74. DOI 10.1145/227595.227597
- [XSB] *The XSB system, volume 1: programmer's manual*, "Variant-based tabled evaluation". <https://xsb.sourceforge.net/shadow_site/manual1/node49.html>
- [Pfe95] F. Pfenning, *Structural cut elimination*, LICS 1995, 156–166. <https://lics.siglog.org/1995/Pfenning-StructuralCutElimin.html>
- [HHP93] R. Harper, F. Honsell, G. Plotkin, *A framework for defining logics*, J. ACM 40(1), 1993, 143–184. DOI 10.1145/138027.138060
- [dB72] N. G. de Bruijn, *Lambda calculus notation with nameless dummies*, Indagationes Mathematicae 34, 1972, 381–392. DOI 10.1016/1385-7258(72)90034-0
- [Cha12] A. Charguéraud, *The locally nameless representation*, J. Automated Reasoning 49, 2012, 363–408. <https://chargueraud.org/softs/ln>
- [CF06] S. Conchon, J.-C. Filliâtre, *Type-safe modular hash-consing*, ACM SIGPLAN Workshop on ML, 2006. <https://researchportal.ip-paris.fr/fr/publications/type-safe-modular-hash-consing/>
- [MP01] R. Moot, M. Piazza, *Linguistic applications of first order intuitionistic linear logic*, J. Logic, Language and Information 10(2), 2001, 211–232. <https://dc2.philarchive.org/rec/MOOLAO>
- [Tam20] N. Tamura, *A linear logic prover (llprover)*, Prolog, last changed December 2020. <https://cspsat.gitlab.io/llprover/>

Sources checked 2026-10-08: [LS94], [CP05], [Mil92] and [NJK95] read
from their PDFs, [BRS24] from its arXiv HTML, the others confirmed
bibliographically. Girard's 1991 first-order nets are known only through
[Hug18] and `38-first-order.md`.
