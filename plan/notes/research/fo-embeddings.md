# Research note: ordinary first-order logic through its embeddings

Written 2026-10-08 from the snapshot of the repository, `38-first-order.md`
(cited as [38]) and the sources of section 7. It specifies what the
ordinary layer (`core/src/ordinary/`) must become when step 38 gives
linear logic its quantifiers, so that the design of step 28 leaves the
room. Everything not cited is marked "(inference)". `A*`, `A°`,
`A^0`/`A^1` and `A^Q`/`A^T` are the sources' names for the translations;
linlog's names `cbn`, `cbv`, `01` and `affine` stand beside them.

## 1. The translations extended to the quantifiers

**The calculi.** First-order linear logic adds `∀x` and `∃x` to the
one-sided calculus: `⊢ Γ, A` gives `⊢ Γ, ∀x A` when `x` is not free in
`Γ`, and `⊢ Γ, A[t/x]` gives `⊢ Γ, ∃x A`; `(∀x A)⊥ = ∃x A⊥` [Gir95,
§1.2.2]. LJ and LK have `∀L`/`∃R` with a witness term and `∀R`/`∃L` with
an eigenvariable not free in the conclusion [TS00, ch. 3]; the G3
variants keep the principal formula of `∀L` and `∃R` in the premise,
which is where contraction hides in first-order logic [TS00, ch. 3;
LM09, Fig. 2]. Intuitionistic negation is `A → ⊥` throughout [LM09, §3].

### 1.1 Girard's call-by-name translation (`cbn`)

Girard's translation of intuitionistic into linear logic, as Schellinx
states it from Girard 1987 [Sch94, Def. 1.3.2; Gir87]:

```
p* = p      ⊥* = 0      (A ∧ B)* = A* & B*      (A ∨ B)* = !A* ⊕ !B*
(A → B)* = !A* ⊸ B*     (∀x A)* = ∀x A*         (∃x A)* = ∃x !A*
```

and a sequent `Γ ⊢ A` goes to `!Γ* ⊢ A*`; the image lies in
`{0, &, ⊕, ⊸, ∀, ∃, !}` [Sch94, Def. 1.3.2]. This is linlog's `cbn` with
the two rows `core-ordinary.md` already names ("`∀` as `∀`, `∃` as
`∃!`"); true as `⊤`, `¬A` as `!A ⊸ 0` and `↔` as a `&` of implications
are definitional (inference).

*Faithfulness.* `IL ⊢ Γ ⇒ A` iff `DIL ⊢ !Γ* ⇒ A*`: correctness by
induction on LJ derivations with the additive `∧`/`∨` rules, faithfulness
because "the skeleton of a DIL-derivation of `!Γ* ⇒ A*` is an
IL-derivation of `Γ ⇒ A`" [Sch94, Prop. 1.3.3]. The statement is for the
first-order language: the fragment has `∀` and `∃`, and the sharper
theorem into *classical* linear logic treats `R∀` beside `R⊸` [Sch94,
Lemma 2.1.3, Thm 2.1.4: "Girard's translation faithfully embeds
intuitionistic logic into classical linear logic"]. Its detailed proof is
Schellinx 1991, whose abstract is exactly that [Sch91]. The subtlety is
`0`: with `0` present a cut-free CLL derivation of an ILL sequent can
pass through sequents with several succedents [Sch94, §2.1; DD, Rem.
2.2.7]. linlog proves the image in intuitionistic mode (`Image::mode`),
where the one-succedent discipline is the calculus itself, so the
skeleton argument [Sch94, Prop. 1.3.3] is the one that applies
(inference).

### 1.2 The call-by-value translation (`cbv`)

The second decomposition, `A → B` as `!(A ⊸ B)` with atoms as `!p`, is
the linear analogue of Gödel's translation of IL into S4 [Sch94,
§2.2–2.3]. Its first-order form, as Ferreira, Oliva and Protin state it:

```
(∀x A)* = !∀x A*          (∃x A)* = ∃x A*
```

with `!p` for atoms and `!(A* ⊸ B*)` for implication [FOP24, Def. 15];
LLTP writes `∧` as `⊗`, `∨` as `⊕`, true as `1`, false as `0`, `¬A` as
`!(A ⊸ 0)` [LLTP], which is linlog's `cbv`. Every image is positive (a
`!`, `⊗`, `⊕`, `∃`, `1` or `0` on top), so a hypothesis needs no further
`!` [Sch94, §2.2] (inference for the quantifier rows: `∀`, negative,
takes the `!` as `→` does; `∃`, positive, does not, as `⊗` does not).

*Faithfulness.* Schellinx derives it from Girard's translation and
`S4 ⊢ A° ↔ !A*`: "Girard's translation is a correct and faithful
embedding of second order propositional intuitionistic into linear
logic", likewise for the linear analogue of Gödel's translation, and
"adding first order quantifiers will be unproblematic. We refrain from
verifying the details" [Sch94, Cor. 2.3.4 and the remark after it]. The
proof-by-proof theory of both translations as *decorations* of LJ
derivations (the `t`- and `q`-decorations, which Schellinx relates to the
`T`- and `Q`-translations of §1.4) is Danos, Joinet and Schellinx
[DJS95a; Sch94, §3.2]; that paper was not read here.

### 1.3 The 0/1 translation (`01`)

Liang and Miller's translation of LJ proofs into focused linear logic
proofs, `A^1` for a formula right of `⊢` and `A^0` left of it [LM09,
Table 1]:

```
         A        A^1                  A^0
   atom  Q        Q                    Q
   true           1                    ⊤
   false          0                    0
   P ∧ Q          !(P^1 & Q^1)         !P^0 & !Q^0
   P ∨ Q          !P^1 ⊕ !Q^1          !P^0 ⊕ !Q^0
   P ⊃ Q          !(!P^0 ⊸ Q^1)        !P^1 ⊸ !Q^0
   ∃x P           ∃x !P^1              ∃x !P^0
   ∀x P           !∀x P^1              ∀x !P^0
```

The propositional rows are those of LLTP's `translation.ml` and of
linlog's table (`translate.rs`, `T0`/`T1`); LLTP's quantifier cases are
commented out ("Propositional, for now") [LLTP], so there is no LLTP
file to match and linlog takes the two rows from the paper.

*Faithfulness.* "The focused proofs of `⊢ (Γ^0)⊥ :⇑ R^1` are in bijective
correspondence with the proofs of `Γ ⊢_I R`" [LM09, Prop. 2], where
`⊢_I` is the G3i-like calculus of their Figure 2 with `∀L`, `∀R`, `∃L`,
`∃R` and "`y` is not free in the conclusion" [LM09, Fig. 2]: a bijection
gives both directions, for first-order LJ. The paper's lighter `−1/+1`
translation, which induces LJF (`(∀x A)^{±1} = ∀x A^{±1}`,
`(∃x P)^{−1} = ∃x P^{−1}`, `(∃x N)^{−1} = ∃x !N^{−1}`,
`(∃x A)^{+1} = ∃x A^{+1}`), is sound and complete [LM09, Fig. 5, Thm 8];
it depends on a polarity assignment to atoms and is a candidate fifth
translation, not a requirement.

### 1.4 Classical logic: the T- and Q-decorations (LKT, LKQ)

The two linear decompositions of classical implication, `!?A ⊸ ?B` (the
T-translation, "tête") and `!A ⊸ ?!B` (the Q-translation, "queue"), give
the calculi LKT and LKQ [DJS95b; Sch94, §2.4, §3.4]. Schellinx's
quantifier clauses are second-order propositional:

```
(∀p A)^Q = ∀p ?!A^Q      (∃p A)^Q = ∃p !A^Q
(∀p A)^T = ∀p ?A^T       (∃p A)^T = ∃p !?A^T
```

[Sch94, Table 2.1]; the thesis says twice that the first-order clauses
are "completely straightforward" and "treated completely analogous"
[Sch94, §2.4 fn. 5, §3.4], so the first-order rows are these with `x` for
`p` (inference). Both directions: "`CLL ⊢ !Γ^Q ⇒ ?!Δ^Q` iff
`CL ⊢ Γ ⇒ Δ` iff `CLL ⊢ !?Γ^T ⇒ ?Δ^T`" [Sch94, Thm 2.4.1], faithfulness
"trivial" by skeleton, correctness by induction. The decoration of
Girard's LC places exponentials by polarity (`∀x ?!P`, `∀x ?N`, `∃x !P`,
`∃x !?N`) [Sch94, §3.5]. All of these are "for showing, not for
deciding" (`plan/later.md`): with `?` on every quantifier the search is
bounded only by the copy bound.

### 1.5 The affine translation and its first-order form

linlog's `affine` translation of classical propositional logic into
affine MALL (negation normal form, `∧` as `&`, `∨` as `⅋`, true as `⊤`,
false as `⊥`, by side; `plan/reports/25-ordinary-logic.md`) rests on the
one-sided calculus with invertible rules being complete without
contraction, which fails in first-order logic: `∃R` and `∀L` need the
principal formula kept, as G3c keeps it [TS00, ch. 3]. The first-order
form (inference):

```
(∀x A)^a = ∀x A^a          (∃x A)^a = ?∃x A^a
```

on formulas in negation normal form, a hypothesis translated as the
negation normal form of its negation. Soundness: a cut-free affine LL
proof of the image reads back to LK by its skeleton, `?d` as nothing,
`?c` as `CR`, `?w` and affine weakening as `WL`/`WR`, the `∀` rule's
eigenvariable condition being LK's. Completeness: a one-sided G3c proof
(axioms with context, `∃R` repeating its principal formula) maps rule by
rule, `∃R` to `?c`, `?d`, `∃`, axioms to the linear axiom with affine
weakening. Neither direction is in a source; both are the skeleton
argument of [Sch94, Prop. 1.3.3] for a translation with one `?`
(inference, for the panel). The image is not closed under the linear
dual (the dual of `?∃x A` is `!∀x A⊥`, not `∀x (¬A)^a`), which is why the
table must stay indexed by side as today (`RIGHT`/`LEFT`). Only `∃`
carries a `?`, so the search is unbounded exactly where first-order
classical logic is undecidable, and `Verdict::Unknown` reaches the
classical path for the first time.

### 1.6 Minimal logic

Minimal logic is Johansson's intuitionistic logic without ex falso
[Joh37]; linlog translates false as the atom `FALSE` and the checker
refuses `⊥L` (`core-ordinary.md`). With quantifiers nothing changes:
`∀` and `∃` have the same rules in G1m as in G1i [TS00, ch. 3], and the
faithfulness of `cbn`, `cbv` and `01` holds over the language in which
`⊥` is an atom with no rule (inference: the three proofs are inductions
over the connectives and skeleton arguments that never use `⊥L`;
without `0` no empty succedent arises in a cut-free ILL derivation of
the image).

## 2. The read-back and its checker

**What the linear proof carries** [38, §3]: `Node::Forall(OccId, Eigen,
NodeId)` and `Node::Exists(OccId, Witness, NodeId)` with a witness arena
the `Proof` owns; the derivation's members as `(OccId, frame)`, a frame
listing the terms the binders above the occurrence are bound to; the
eigenvariables as fresh `Free` symbols; a metavariable still free in a
found proof made a fresh constant or left named ([38, Q3]).

**Rule mapping.** The read-back walks the linear derivation's inferences
as today (`Image::read_back`), each tagged by the pattern table with the
ordinary formula, function, `!`s left and side. The quantifier rules map
by the tag's side:

| linear rule on the image of | ordinary rule | datum |
|---|---|---|
| `∀` on a right formula `∀x A` | `∀R` | the eigenvariable |
| `∃` on a right formula `∃x A` | `∃R` | the witness |
| `∃` on the dual of a hypothesis `∀x A` (two-sided: `∀L`) | `∀L` | the witness |
| `∀` on the dual of a hypothesis `∃x A` (two-sided: `∃L`) | `∃L` | the eigenvariable |

The exponentials between a quantifier and its body (`∃x !A` in `cbn` and
`01`, `!∀x A` in `cbv` and `01`, `?∃x A` in `affine`) pass as the `bangs`
count passes them now: `!`/`?d` no inference, `?c` `CL`/`CR`, `?w`
`WL`/`WR` by side. One table row per translation and quantifier is all
the read-back needs.

**Instances.** A member of an ordinary sequent is a *closed* formula, the
instance `A[t₁/x₁, …]` of the tagged subformula under its frame. The
read-back computes it by opening binders in the ordinary arena
(`Formulas::instantiate(node, frame)`), adding nodes as it goes; the
arena is hash-consed, so equal instances are one `NodeId` and the
checker's multiset comparisons (`same`, `is`) stay comparisons of sorted
ids. The translation keeps predicate and function symbols and argument
lists verbatim, so a linear witness maps back by the identity on symbols
(inference; the invariant to state: the two symbol tables agree, or the
`Image` holds the map).

**The checker** (`Derivation::check`, trusting nothing of the read-back)
gains, with `Inference` carrying the rule's term or eigenvariable
explicitly: for `∀L`/`∃R`, the premise is the context plus `A[t/x]`,
recomputed by `instantiate` and compared by id; for `∀R`/`∃L`, the
premise is the context plus `A[y/x]` and `y` occurs free in no member of
the conclusion [TS00, ch. 3]. The second needs the free symbols of a
node, computed bottom-up once per node and cached in the arena. Carrying
the term explicitly keeps the check a recomputation and lets the Rocq
printer name the witness (inference). A derivation unfolded from a proof
DAG repeats an eigenvariable on two paths; the per-inference condition
does not mind, but the planned read-back on the proof term
(`plan/later.md`, "Follow-ups: ordinary logic") must count introductions
per node, as [38, §3] says for the linear checker.

## 3. The inputs

### 3.1 TPTP's FOF, the part needed

From the TPTP syntax BNF, v9.3.1.4 [TPTP-BNF]:

```
<fof_annotated>          ::= fof(<name>,<formula_role>,<fof_formula><annotations>).
<fof_logic_formula>      ::= <fof_binary_formula> | <fof_unary_formula> | <fof_unitary_formula>
<fof_binary_pair>        ::= <fof_unit_formula> <nonassoc_connective> <fof_unit_formula>
<fof_or_formula>         ::= <fof_unit_formula> <vline> <fof_unit_formula> | <fof_or_formula> <vline> <fof_unit_formula>
<fof_and_formula>        ::= <fof_unit_formula> & <fof_unit_formula> | <fof_and_formula> & <fof_unit_formula>
<fof_unary_formula>      ::= <unary_connective> <fof_unit_formula> | <fof_infix_unary>
<fof_unitary_formula>    ::= <fof_quantified_formula> | <fof_atomic_formula> | (<fof_logic_formula>)
<fof_quantified_formula> ::= <fof_quantifier> [<fof_variable_list>] : <fof_unit_formula>
<fof_quantifier>         ::= ! | ?
<fof_plain_term>         ::= <constant> | <functor>(<fof_arguments>)
<fof_term>               ::= <fof_function_term> | <variable>
<variable>               ::= <upper_word>
<atomic_word>            ::= <lower_word> | <single_quoted> | <back_quoted>
<nonassoc_connective>    ::= <=> | => | <= | <~> | ~| | ~&
<include>                ::= include(<file_name><include_optionals>).
```

Equality is `<fof_term> = <fof_term>` (`fof_defined_infix_formula`) and
`!=` (`fof_infix_unary`); `$true`/`$false` are the defined propositions;
numbers and `"distinct objects"` are terms of their own [TPTP-BNF].
"Negation has higher precedence than quantification, which in turn has
higher precedence than the binary connectives", "no precedence is
specified between the binary connectives; brackets are used", and "every
variable in a Formula must be bound by a preceding quantification"
[TPTP-Lang]. The roles are `axiom, hypothesis, definition, assumption,
lemma, theorem, corollary, conjecture, negated_conjecture, …` [TPTP-BNF].

linlog's `read_tptp` reads the propositional fragment through
`lltp::clauses`, which splits `fof(name, role, formula).` clauses and
reads the `Status` comments (`core/src/lltp.rs`). The first-order part
adds: the quantified formula with its variable list; variables as
`upper_word`, bound to the nearest enclosing quantifier (an unbound one
is an error, by the rule above); terms `constant | functor(args)` with
`lower_word`, single-quoted and back-quoted functors;
`include('Axioms/X.ax').` (every ILTP first-order problem with axioms has
them, as `GEJ003+1.p` shows [ROK07, Fig. 1]); the roles `definition`,
`lemma`, `theorem`, `corollary` as hypotheses beside `axiom` and
`hypothesis`. It refuses, with a named error: equality and inequality
(or, as an option, reads them as the predicate `equal` with the equality
axioms added, which is ILTP's own `tptp2X -t add_equality` for provers
without equality [ILTP-readme]); numbers, distinct objects and
`$`-defined symbols other than `$true`/`$false`; a `negated_conjecture`
outside classical logic (classically a hypothesis with the goal false;
intuitionistically `Γ, ¬C ⊢ ⊥` is weaker than `Γ ⊢ C`, inference). The
library reads no files, so `include` takes a resolver the caller supplies
(the CLI's resolves against the problem's directory).

### 3.2 ILTP's first-order problems

Release v1.1.2, statistics dated 1 December 2006: "2550 problems (2480
non-propositional, 70 propositional)" in 24 domains; intuitionistic
status `Theorem` 667, `Non-Theorem` 96, `Unsolved` 424, `Open` 1363;
classical status "1950 valid, 553 invalid, 47 unknown"; "185
pure-equality problems" [ILTP-stats; ILTP-readme]. Of release v1.1.1,
"Number of problems with equality 1730 (63%)" [ROK07, Table I], so about
820 first-order problems are equality-free (inference from the counts).
The propositional part is the 274 problems linlog already runs
(`modules/bench.nix`).

*How intuitionistic status is recorded.* Each file's header has
`% Status (intuit.) : Theorem` and `% Rating (intuit.) : 0.75 v1.1.1`
beside the TPTP fields `Status`, `Rating`, `Syntax` of the classical
problem it came from [ROK07, §2.3, Fig. 1]; `lltp::clauses` already
prefers the `(intuit.)` line. The status "is either Theorem,
Non-Theorem, Unsolved or Open. Problems with Unsolved status have not
been solved under the test condition by any state-of-the-art ATP
system. A problem has an Open status if its abstract problem has not
been solved at all. No theoretical investigations into the
intuitionistic validity of the problems in the TPTP library were done.
Instead the intuitionistic status of a problem is marked as Theorem or
Non-Theorem if any intuitionistic ATP system was able to prove or refute
the problem" [ROK07, §2.2]; "In the TPTP library the term Unknown is
used instead of Unsolved" [ROK07, fn. 4]. So a `Theorem` or
`Non-Theorem` is a prover's claim and the 1787 `Unsolved`/`Open` are no
verdict: the harness compares against the 763 decided problems only and
must not count a disagreement with `Open` as a mismatch (inference).
Problems whose classical status was Satisfiable or Unsatisfiable were
negated by tptp2X [ROK07, §2.2].

*Licence.* The site, the readmes and the paper state none; the paper
gives conditions for presenting results: state the release number, refer
to problems by name, modify nothing but the syntax, do not exploit the
header, document the system's version and settings [ROK07, §1.1]. The
flake fetches the propositional archive at a pinned version for that
reason (`modules/bench.nix`); the first-order archive
(`ILTP-v1.1.2-firstorder.tar.gz`, 1.8 MB [ILTP-site]) would come the same way.

### 3.3 Sensible classical first-order sets

- The ILTP first-order set itself under its classical `Status` line: the
  same equality-free files, 1950 valid against 553 invalid overall, with
  the intuitionistic verdicts on the same problems for the comparison of
  the two logics [ILTP-readme].
- Pelletier's seventy-five problems [Pel86]: 1–17 propositional, 18–34
  monadic predicate logic, 35–70 full predicate logic in several tiers
  (equality enters in the later ones; which, the TPTP headers say),
  71–75 for complexity. They are in TPTP's SYN domain with the header
  `% Problem : Pelletier Problem NN` (`SYN054+1` is problem 24; found
  through a web search, not re-verified on tptp.org). The equality-free
  ones are the right first classical set: small, famous, of known status.
- TPTP FOF problems chosen by header: `+` in the name (FOF), no equality
  in the `% Syntax` counts, small. TPTP states no licence on the pages
  consulted; its guidelines are the release number, the unambiguous
  problem name, no changes to the formulae, settings recorded, "the
  header information in each problem may not be used by the ATP system
  without explicit notice" [TPTP-Use]; the site asks that the TPTP World
  be cited by [Sut24], the library by [Sut17]. Not sensible: anything
  CASC-sized (`plan/later.md`, "not competitive").

## 4. Consequences for linlog's ordinary layer

**The formula type.** `Node` (`ordinary/mod.rs`) gains
`Pred(Symbol, Args)` for an atom with arguments, `Forall(NodeId)` and
`Exists(NodeId)` with the bound variable locally nameless (a de Bruijn
index inside, a name hint in a table for printing) [Cha12, as in 38];
`Formulas` gains a first-order term arena (`Bound(u32)`, `Free(Symbol)`,
`App(Symbol, Args)`), a symbol table with arities, `instantiate` and
`free_symbols`. Hash-consing stays the invariant the checker relies on
("equal formulas are one id"): closed instances compare by id, open
subformulas are α-canonical by construction. Whether this is *the same*
term type as the linear sequent's (`sequents/term.rs` after 38) or a
mirror is a decision for the design: one type saves the symbol map of §2
and a second printer; two keep the layers independent.

**The native syntax.** `forall x. A`, `exists x. A` and `∀x. A`, `∃x. A`
with scope as far right as possible, predicates `p(x, f(y))`,
identifiers bound by the nearest binder, unbound identifiers constants
(inference; the same question as [38, Q1], to be decided once for both
syntaxes). The TPTP dialect keeps the same `Parser`, with a binder entry
in `pending` and a term reader.

**Translations over binders.** `pattern` gets two rows per translation
and function: `cbn` `Forall ↦ (0, Forall, (a, f, 0))`,
`Exists ↦ (0, Exists, (a, f, 1))`; `cbv` `(1, Forall, (a, 0, 0))`,
`(0, Exists, (a, 0, 0))`; `01` under `T1` `(1, Forall, (a, T1, 0))`,
`(0, Exists, (a, T1, 1))` and under `T0` `(0, Forall, (a, T0, 1))`,
`(0, Exists, (a, T0, 1))`; `affine` `Forall` right and `Exists` left as
`Forall`, `Exists` right and `Forall` left as `?Exists`. `Core` gains the
two unary cores, and a `Pattern`'s exponential prefix (`bangs: u8`,
meaning `!` right of `⊢` and `?` on the dual) must also express a `?` on
the formula as it stands, which only `affine` needs: a short sequence
over `{!, ?}`, or a second count. The builder's `operand` closure wraps
the body before the binder, so `translate` stays bottom-up over the
arena with no renaming. `Translation::target()` becomes a value, not a
string: `affine MALL` for a propositional classical sequent, `affine LL`
once a quantifier is present; `Translation::decides` keeps the meaning
"sound and faithful" and the docs stop implying "terminates" for
classical logic. `Image::mode` is unchanged; the image's `Fragment`
gains `quantifiers`, which [38] routes to the focused engines.

**`Logic`, `Translation`, `Options`.** No new variant is required: the
three logics and the four translations all extend to quantifiers (§1).
Both enums and `ordinary::Options` take `#[non_exhaustive]` now (the
research README's rule), so that a polarized translation [LM09, Fig. 5]
or a showing-only LKT/LKQ can land in 0.y.
`Translation::DEFAULT_INTUITIONISTIC` was chosen on the propositional
ILTP run; the first-order default is a measurement to make, and the
constant's doc should say "propositional" until then. New reader
options, `Equality { Refuse, Axioms }` and the include resolver, belong to
`read_tptp`, not to `ordinary::Options`.

**Read-back and checker.** `Rule` gains `ForallLeft`, `ForallRight`,
`ExistsLeft`, `ExistsRight` (`∀L ∀R ∃L ∃R`, markup `∀_L` and so on), the
same four in LK and LJ: `Rule::ALL` goes from 25 to 29 and `Drawn::RULES`
follows. `Inference` gains one field for the rule's datum (`Option<
Binding>` with `Witness(TermId)` and `Eigen(Symbol)`), `None` for every
other rule, so the propositional JSON is unchanged. `Derivation` owns the
grown arena and its free-symbol table. `Derivation::check` adds the two
checks of §2 and a stop, as step 28 plans. The read-back takes members as
`(OccId, frame)` (the linear `Inference.sequent` after 38) and reaches the
`Proof`'s witness arena through the `Derivation`.

**Rocq certificates** (`ordinary/rocq.rs`). The lemma is stated over a
domain: a `Section` with `Variable D : Type`, a `Variable` per predicate
(`p : D -> D -> Prop`) and function symbol (`f : D -> D`), `∀x A` as
`forall x : D, A`, `∃x A` as `exists x : D, A` (`ex`/`ex_intro` of the
standard library). LJ: `∀R` is `fun y : D => …`, `∀L` the application
`h t`, `∃R` is `ex_intro _ t …`, `∃L` a `match h with ex_intro _ y h' =>
… end` with a `return` as every match has. LK keeps the continuation
style: `∃R` makes `k' : ~ A[t/x] := fun h => k (ex_intro _ t h)`; `∀R`
makes `∀x A` from a `False`-term with `k' : ~ A[y/x]` by `NNPP`
[Rocq-CP], as the certificates do today. A witness
or parameter not in the sequent (the fresh constants of [38, Q3]) is a
`Variable d : D` of the section, so a classical certificate of
`∀x p(x) ⊢ ∃x p(x)` is relative to an inhabited domain, which is
first-order logic's own assumption (inference). The identifier escaping
(`USED`, `export::rocq::identifiers`) extends to function and variable
names. Minimal logic's certificate stays the intuitionistic statement.

**JSON forms.** `ordinary::Sequent` and `ordinary::Derivation` have no
form yet; step 28 adds them, and the design must make the quantifiers
additive from the start: node tags `P`/`∀`/`∃` beside the propositional
ones, keys `terms`, `symbols` (with arities) and `variables` under
`#[serde(default)]` and skipped when empty, a `version` read as 1 when
missing, the inference's datum as an optional key, so that a file written
before step 38 reads unchanged, pinned by a test as the research README's
wire-form policy asks. The CLI's `--logic` paths need no new command;
`--equality` and `--include-dir` are the new flags, and the verdict line
says `affine LL` where it says `affine MALL`.

## 5. Worked examples

**(a) The drinker, classically.** `⊢ ∃x (p(x) → ∀y p(y))`. Negation
normal form `∃x (¬p(x) ∨ ∀y p(y))`; affine image
`⊢ ?∃x (p(x)⊥ ⅋ ∀y p(y))`. A proof: `?c` doubles the formula; `?d`, `∃`
with a parameter `c` (no term of the sequent fits; the metavariable
stays free and is made a constant), `⅋`, `∀` with eigenvariable `b`:
`⊢ p(c)⊥, p(b), ?∃x (p(x)⊥ ⅋ ∀y p(y))`; then `?d`, `∃` with witness `b`,
`⅋`: `⊢ p(c)⊥, p(b), p(b)⊥, ∀y p(y)`, closed by the axiom `p(b), p(b)⊥`
with affine weakening of `p(c)⊥` and `∀y p(y)`. Read back in LK: `CR` on
`∃x (p(x) → ∀y p(y))`, `∃R` with `c`, `→R`, `∀R` with `b`, `∃R` with `b`,
`→R`, `WL`, `WR`, `ax` on `p(b)`. Without the `?` the formula is
unprovable, since the `∃` is instantiated once (inference).
Intuitionistically, the `cbn` image `⊢ ∃x !(!p(x) ⊸ ∀y p(y))` fails:
after `∃` with a metavariable `X`, `!`, `⊸R` and `∀R` with eigenvariable
`b` the goal is `!p(X) ⊢ p(b)`, and `X := b` is forbidden because `b` is
younger than `X` ([38, §2], "levels, not Skolem terms"); the remaining
choices are finite, so the answer is `Unprovable` (inference).

**(b) An intuitionistic theorem in the four translations.**
`∀x (p(x) → q(x)), ∃x p(x) ⊢ ∃x q(x)`:

```
cbn     !∀x (!p(x) ⊸ q(x)),  !∃x !p(x)  ⊢  ∃x !q(x)
cbv     !∀x !(!p(x) ⊸ !q(x)), ∃x !p(x)  ⊢  ∃x !q(x)
01      !∀x !(!p(x) ⊸ !q(x)), !∃x !p(x) ⊢  ∃x !q(x)
affine  ⊢ ?∃x (p(x) & q(x)⊥), ∀x p(x)⊥, ?∃x q(x)
```

(`cbv` and `01` differ only in the `!` on the second hypothesis.) The
`cbn` proof in two-sided ILL: `!L` on `!∃x !p(x)` (no inference), `∃L`
with eigenvariable `a`, `!L` on `!p(a)` (none), `∃R` with `a` on the
goal, `!R` (none; the context is all `!`), `!L` and `∀L` with `a` on the
first hypothesis, `⊸L` (`→L`) with premises `!p(a) ⊢ !p(a)` and
`q(a) ⊢ q(a)`, `CL`/`WL` where the dyadic context copies and drops
hypotheses. Read back, structural rules omitted:

```
p(a) ⊢ p(a)    q(a) ⊢ q(a)
────────────────────────── →L
p(a), p(a) → q(a) ⊢ q(a)
──────────────────────────── ∀L(a)
p(a), ∀x (p(x) → q(x)) ⊢ q(a)
──────────────────────────────── ∃R(a)
p(a), ∀x (p(x) → q(x)) ⊢ ∃x q(x)
─────────────────────────────────── ∃L(a)
∀x (p(x) → q(x)), ∃x p(x) ⊢ ∃x q(x)
```

The checker recomputes `(p(x) → q(x))[a/x]` and `q(x)[a/x]` by
`instantiate` and finds their ids in the premises, and checks that `a`
is free in no member of `∃L`'s conclusion.

**(c) A classical non-intuitionistic formula with quantifiers.** The
constant-domain principle `∀x (p(x) ∨ q) → (∀x p(x) ∨ q)` is an LK
theorem (its affine image is proved as in (a)) and no LJ theorem. Its
`cbn` image `⊢ !∀x (!p(x) ⊕ !q) ⊸ (!∀x p(x) ⊕ !q)` is unprovable, but the
hypothesis `!∀x (…)` can be derelicted with a new witness each time, so
the search meets the copy bound and answers `Unknown`: refutations will
be rare, as ILTP's 96 `Non-Theorem` against 1363 `Open` foretell (§3.2).

## 6. Open questions

1. The native quantifier syntax and the variable convention (one answer
   for the linear and the ordinary syntax; the author, as [38, Q1]).
2. One term type for both layers, or a mirror with a symbol map (§4).
3. The exponential prefix of a `Pattern`: a `{!, ?}` sequence or two
   counts; and whether `affine` keeps its name when its target grows.
4. Equality: refuse, or `equal` with axioms as ILTP's tptp2X does, and
   whether the congruence axioms are generated per symbol and arity.
   Numbers and distinct objects stay refused.
5. `negated_conjecture` and the roles beyond `axiom`/`hypothesis`/
   `conjecture`: which are hypotheses, which are refused, per logic.
6. The classical certificate's inhabited domain: a `Variable d : D` only
   when the derivation uses a fresh parameter, or always.
7. Whether first-order ILTP statuses (prover claims, §3.2) enter the
   harness as ground truth or as a column beside linlog's verdicts.
8. The first-order default translation for intuitionistic logic: a
   measurement on ILTP's equality-free first-order problems, after 38.
9. Whether the `−1/+1` (LJF) translation [LM09, Fig. 5] is worth a fifth
   `Translation` once atoms carry a bias (the focused engine's atom bias
   suggests the knob exists).
10. The read-back on the proof DAG rather than the unfolded derivation
    (`plan/later.md`): with eigenvariables it must count introductions
    per node.

## 7. Sources

- [38] `plan/notes/research/38-first-order.md`, 2026-10-08 (this directory).
- [Gir87] J.-Y. Girard, *Linear logic*, Theoretical Computer Science 50(1), 1987, pp. 1–101. <https://girard.perso.math.cnrs.fr/linear.pdf> (a scan without text layer; its clauses are quoted here from [Sch94] and [FOP24], which attribute them to it).
- [Gir95] J.-Y. Girard, *Linear logic: its syntax and semantics*, in Advances in Linear Logic, LMS Lecture Notes 222, CUP 1995. <https://girard.perso.math.cnrs.fr/Synsem.pdf> (§1.2.2 the one-sided first-order rules).
- [Sch91] H. Schellinx, *Some syntactical observations on linear logic*, Journal of Logic and Computation 1(4), 1991, pp. 537–559. DOI 10.1093/logcom/1.4.537; preprint ILLC ML-1990-08, <https://eprints.illc.uva.nl/id/eprint/1299/>.
- [Sch94] H. Schellinx, *The Noble Art of Linear Decorating*, PhD thesis, ILLC Dissertation Series DS-1994-01, University of Amsterdam, 1994. <https://eprints.illc.uva.nl/id/eprint/1964/> (Def. 1.3.2, Prop. 1.3.3, Lemma 2.1.3, Thm 2.1.4, Cor. 2.3.4, Table 2.1, Thm 2.4.1, §3.4, §3.5).
- [DJS95a] V. Danos, J.-B. Joinet, H. Schellinx, *On the linear decoration of intuitionistic derivations*, Archive for Mathematical Logic 33(6), 1995, pp. 387–412. DOI 10.1007/BF02390456 (not read; known through [Sch94]).
- [DJS95b] V. Danos, J.-B. Joinet, H. Schellinx, *LKQ and LKT: sequent calculi for second order logic based upon dual linear decompositions of classical implication*, in Advances in Linear Logic, LMS Lecture Notes 222, CUP 1995, pp. 211–224 (not read; known through [Sch94] and [FOP24]).
- [DJS97] V. Danos, J.-B. Joinet, H. Schellinx, *A new deconstructive logic: linear logic*, Journal of Symbolic Logic 62(3), 1997, pp. 755–807. <https://www.cambridge.org/core/journals/journal-of-symbolic-logic/article/new-deconstructive-logic-linear-logic/A1FEC09E489CBA5B2868589CA92F8FEA> (not read).
- [LM09] C. Liang, D. Miller, *Focusing and polarization in linear, intuitionistic, and classical logics*, Theoretical Computer Science 410(46), 2009, pp. 4747–4768. Preprint <https://www.lix.polytechnique.fr/~dale/papers/tcs09.pdf> (Table 1, Fig. 2, Prop. 2, Fig. 5, Thm 8).
- [FOP24] G. Ferreira, P. Oliva, C. L. Protin, *On the various translations between classical, intuitionistic and linear logic*, arXiv:2409.02249, 2024 (v3 2025). <https://arxiv.org/abs/2409.02249> (Def. 15 call-by-value, Def. 16 call-by-name).
- [DD] V. Danos, R. Di Cosmo, *The Linear Logic Primer*, course notes, Université Paris VII, undated. <https://www.dicosmo.org/CourseNotes/LinLog/CorsoPisa.pdf> (Rem. 2.2.7, §2.3.3).
- [TS00] A. S. Troelstra, H. Schwichtenberg, *Basic Proof Theory*, 2nd ed., Cambridge Tracts in Theoretical Computer Science 43, CUP 2000 (ch. 3: the G1 and G3 calculi, the quantifier rules and their variable conditions).
- [Joh37] I. Johansson, *Der Minimalkalkül, ein reduzierter intuitionistischer Formalismus*, Compositio Mathematica 4, 1937, pp. 119–136.
- [ROK07] T. Raths, J. Otten, C. Kreitz, *The ILTP problem library for intuitionistic logic*, Journal of Automated Reasoning, 2007. DOI 10.1007/s10817-006-9060-z; preprint <https://www.iltp.de/download/iltp_jar06.pdf> (§1.1, §2.2, §2.3, Tables I–II).
- [ILTP-site] J. Otten, T. Raths, *The ILTP Library*, <https://www.iltp.de/>, <https://www.iltp.de/formulae.html> (v1.1.2, page dated 2017-03-01).
- [ILTP-readme] <https://www.iltp.de/download/readme-ILTP-v1.1.2-fof.txt>, <https://www.iltp.de/download/readme-ILTP-v1.1.2-prop.txt>.
- [ILTP-stats] <https://www.iltp.de/download/ILTP-v1.1.2-fof-statistics.txt> (1 Dec 2006).
- [TPTP-BNF] G. Sutcliffe, *TPTP syntax BNF*, v9.3.1.4. <https://tptp.org/UserDocs/TPTPLanguage/SyntaxBNF.html>
- [TPTP-Lang] *The TPTP Language*. <https://tptp.org/UserDocs/TPTPLanguage/TPTPLanguage.shtml>
- [TPTP-Use] *TPTP problem library manual: getting and using*. <https://tptp.org/UserDocs/ProblemLibraryManual/GettingAndUsing.shtml>
- [Sut17] G. Sutcliffe, *The TPTP problem library and associated infrastructure: from CNF to TH0, TPTP v6.4.0*, Journal of Automated Reasoning 59(4), 2017, pp. 483–502. DOI 10.1007/s10817-017-9407-7.
- [Sut24] G. Sutcliffe, *Stepping stones in the TPTP world*, IJCAR 2024, LNAI 14739, pp. 30–50 (the citation <https://www.tptp.org/> asks for).
- [Pel86] F. J. Pelletier, *Seventy-five problems for testing automatic theorem provers*, Journal of Automated Reasoning 2(2), 1986, pp. 191–216. DOI 10.1007/BF02432151.
- [LLTP] meta-logic/lltp, `tptpparser/translation.ml`. <https://github.com/meta-logic/lltp>
- [Rocq-CP] Rocq standard library, `Stdlib.Logic.Classical_Prop` (`classic`, `NNPP`). <https://rocq-prover.org/doc/V9.0.0/stdlib/Stdlib.Logic.Classical_Prop.html>
- [Cha12] A. Charguéraud, *The locally nameless representation*, JAR 49, 2012, pp. 363–408 (as cited in [38]).
- Not consulted: A. S. Troelstra, *Lectures on Linear Logic*, CSLI Lecture Notes 29, 1992, the textbook treatment of the embeddings.
