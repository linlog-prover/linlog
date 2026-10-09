# Impact map: ordinary first-order logic through first-order linear logic

Written 2026-10-08 from the snapshot of the repository (its code read
directly: `core/src/`, `cli/src/`, `bench/src/`, `modules/bench.nix`, the
tests and README), for stage 2 of `plan/28-audit-and-refactor.md`. It
builds on `38-first-order.md` (the linear side: terms, levels, the trail,
`(OccId, frame)` members, the prunes) and on `plan/later.md` ("Ordinary
logic through its embeddings", "First-order linear logic", "Follow-ups:
ordinary logic"). It does not repeat their surveys. What it adds is a
list of places by file and line, grouped as the task asks, each tagged:

- **open now**: cheap at step 28 and costly later. Examples are a
  `#[non_exhaustive]`, a version field, a type named in `api.md`, or an
  exhaustive arm where a wildcard now hides a new variant.
- **at step**: local and cheap at the feature's own step (38, or the
  ordinary first-order step after it).

"(inference)" marks a conclusion of this note's own.

## 0. The feature, and three facts that change current claims

The path: a TPTP `fof` problem or the native syntax is read into an
ordinary first-order arena. It is translated into a first-order linear
image and decided by step 38's focused engine. The linear proof, which
carries witnesses, is read back as LK or LJ with the rules `∀L ∀R ∃L
∃R`, checked by `Derivation::check`, drawn by the exports and certified
over `Prop`.

1. **Classical first-order logic is not decided in affine MALL without
   copies.** The propositional claim (`ordinary/mod.rs:17-25`, the docs
   of `Translation::Affine` at `mod.rs:112-116`, the CLI help at
   `argument_parsing.rs:674-677`, README 815-817) holds because the
   invertible propositional rules need no contraction. A γ-formula
   (`∃` on the right, `∀` on the left) needs contraction. The drinker
   formula `∃x.(p(x) → ∀y.p(y))` has no proof that uses its `∃` once
   (inference). So the classical image needs `?` on every γ-formula. It
   then deepens a copy bound and can answer `unknown`, since classical
   first-order validity is only semi-decidable. `Translation::target()`
   ("affine MALL", `mod.rs:186-192`) and the README verdict lines that
   `cli/tests/readme.rs` pins become wrong for first-order input.
2. **Ordinary first-order semantics assumes a non-empty domain**, as
   TPTP and LK/LJ do: `⊢ ∃x.⊤` and `∀x.p(x) ⊢ ∃x.p(x)` are valid. A
   witness for a problem without a closed term must then be a fresh
   constant. That is open question 3 of `38-first-order.md`: for the
   ordinary layer the answer is forced (a constant), and the Rocq
   statement must take an inhabitant.
3. **First-order certificates of ordinary logic need no library.** The
   linear ones wait for step 31's kernel. The ordinary certificate
   (`ordinary/rocq.rs`) is a plain Rocq term over `Prop`. It extends
   with a domain `D : Type`, an inhabitant, `forall`/`exists`,
   `ex_intro` and a `match` on `ex` (inference; `31-rocq.md`:303-304
   says the ordinary certificates stay as they are).

## 1. Data model

| place | assumes | first-order needs | when |
|---|---|---|---|
| `ordinary/mod.rs:237-255` `Node` (`Atom(u32)`, `True`, `False`, `Not`, `And`, `Or`, `Implies`, `Iff`) | atoms are nullary, no binder | `Pred(sym, args)` with the argument list by index (so `Node` stays 12 bytes), `Forall(body)` and `Exists(body)` with the bound variable nameless; `Atom(u32)` kept for nullary predicates | open now: `#[non_exhaustive]` (§10, item 5); the variants at step |
| `ordinary/mod.rs:258-267` `Node::operands` | at most two children | binders are unary and fit; arguments are terms, not formulas | at step |
| `ordinary/mod.rs:270-283` `Formulas` (`nodes`, `atoms`, `names`, `ids: HashMap<Node, NodeId>`) | hash-consing makes "equal formula ⇔ equal id", which `same` (`derivation.rs:594-605`) and the `ids.get(&Node::…)` look-ups (`derivation.rs:451,459,464-465`; `rocq.rs:304,329-341`) rely on | equality up to α-renaming, so bound variables must be nameless (locally nameless, as `38-first-order.md` recommends for `Sequent`); a term arena; a symbol table with arities, function symbols and constants | open now: one representation for both arenas, written in `api.md` (§10, item 1) |
| `ordinary/mod.rs:304-345` `atom_name(u32)`, `atom_names()`, `atom(name)` | a name is an atom | `p/0` and `p/1` are different symbols; constants and functions need their own tables | at step (additive methods) |
| `ordinary/mod.rs:317-331` `Formulas::add` | callers pass operands of this arena (unchecked) | binder well-formedness: no loose index outside a binder, arities | open now: validate, since it already returns `Result` (item 7) |
| `ordinary/mod.rs:476-510` `Sequent` and `Sequent::new(formulas, left, right) -> Self` | infallible; ids unchecked; every formula is closed by construction | the formulas must be closed; there must be a policy on free variables of the native syntax | open now: `Result<Self, Error>` (item 7) |
| `ordinary/mod.rs:393-424` `Symbols` (pub(crate)) | seven connective spellings | `∀`, `∃`, the binder separator, application syntax and a bound-variable naming | at step |
| `ordinary/translate.rs:18-53` `Core`, `Operand`, `Pattern { bangs, core, operands }` | a pattern puts only `!` on top of the positive image (`bangs`); no binder core | `Forall` and `Exists` cores. The classical γ-formula needs `?` on top of the image that stands right of `⊢`, which `bangs` cannot express (the negative side gets `?`, `translate.rs:391-396`) | at step (all private to the module) |
| `ordinary/translate.rs:55-59,312-317` `FALSE` and its collision loop over `atom_names()` | atom names are the whole namespace | collision by `(name, arity)` | at step |
| `ordinary/translate.rs:72-97` `Extra` (the two implications per `↔`, and false) | the only extra nodes are propositional | equality axioms if `=` is read as a predicate (§9), and the inhabitant constant (§0.2) | at step |
| `ordinary/translate.rs:434-438` `crate::Sequent { terms, roots, atoms }` struct literal; `462-505` `Builder` | the linear sequent has three fields; `Builder` repeats the arena's hash-consing | the linear first-order tables (`38-first-order.md` §3). With one shared term type the translation copies term ids unchanged | open now: a crate-internal constructor instead of the struct literals (`translate.rs:434`, `parse/mod.rs:407`, `serialize/sequents.rs:124`, the test helpers), cheap |
| `sequents/term.rs:27-30` `Atom` ("a propositional variable"); `52-77` `Term`, 12 bytes with no size assertion | — | `Pred`/`DualPred` with an argument index (38 note), `Forall`/`Exists` | open now: `assert!(size_of::<Term>() == 12)` beside `Node`'s (item 10) |
| `sequents/term.rs:196-253` `subterms`, `operands`, `map_subterms`, `offset`, `check_bounds` | only subterm and atom indices | argument ids to offset (`Sequent::add`, `mod.rs:361-379`) and bound-check (`verify_integrity`, `mod.rs:176-201`) | at step |
| `sequents/mod.rs:281-356` `optimize_atoms`, `merge_atoms`; `serialize/sequents.rs:120-131` (reading merges atoms by name) | "a name is an atom" | the atom is `(name, arity)`, else `p/0` and `p/1` merge | at step, but `api.md` should say the identity is `(name, arity)` |
| `sequents/mod.rs:137-143` `Sequent::atom(name)` | name lookup is unique | ambiguous across arities | at step (additive) |
| `sequents/mod.rs:204-219` `optimize_roots` sorts roots; `translate.rs:441-445` pairs roots by sorted term id | roots sorted by term | not first-order, but the pairing changes in the commit that makes the written order canonical (README of these notes, conflict 2) | in that commit |
| `occurrences/mod.rs:155-180` `Forest` (`literals` grouped by atom and sign) | atom and sign find every axiom partner | candidates by symbol, then unification (38 note) | at step |
| `fragment.rs:13-42` `Fragment(u8)`, five bits used | a fragment is a set of connective classes | a sixth bit for first-order (binders, or any predicate with arguments); see §5 for why "with arguments" | open now: reserve the bit and the naming (item 3) |
| `fragment.rs:95-125` `name`, `name_in`; `serialize/search.rs:10-42` `NAMED` and the `I`-prefix rule | six names, intuitionistic ones by an `I` prefix | names that keep the prefix rule (for example a suffix `1`: `MLL1`, `IMLL1`) | open now: decide the scheme (item 3); add the names at step |
| `fragment.rs:220-229` `Mode` | — | unchanged | — |
| `lltp.rs:59-66` `Status { Theorem, NonTheorem }`, exhaustive | two statuses | ILTP's `Unsolved` and `Open`, if the harness is to report them | open now: `#[non_exhaustive]` (item 5) |

## 2. Exhaustive matches, visitors, and the wildcards that hide them

Exhaustive today: a new variant fails to compile at each of these, which
is what the feature wants.

- `Term` and `Kind`: `term.rs:111-208`, `occurrences/mod.rs:102-109`,
  `fragment.rs:163-173`, `reading.rs:206-213`, `sequents/fmt.rs:145-156`,
  `notation.rs:63-77` and `102-114` (the `Enter` arms),
  `export/rocq.rs:265-282`, `proofs/size.rs:205-217`,
  `serialize/sequents.rs:62-102`.
- `Node` (linear): `proofs/mod.rs:163-230,463-478`, `check.rs:1156ff`
  (`Pass::rule`), `serialize/proofs.rs:65-110`.
- Ordinary: `translate.rs:123-209` (`pattern`, per translation),
  `derivation.rs:134-221` (`name`, `markup`), `ordinary/rocq.rs:158-322`
  (`tasks`).

**Wildcards.** Each of these sends a new variant down an existing branch
with no compile error. That gives a wrong answer, a wrong print or a
runtime panic.

| place | the wildcard | what a `∀`/`∃`/predicate would do there |
|---|---|---|
| `search/focus/mod.rs:840-866` (`asynchronous`) | `_ => gamma.insert(o)` | a `∀` (negative) enters the stable sequent unopened |
| `search/focus/mod.rs:878-882` | `_ => Node::Bot(o, node)` | a new invertible rule is recorded as `⊥` |
| `search/focus/mod.rs:1304-1310` | `kind => unreachable!` | an `∃` in a stable sequent panics |
| `search/focus/mod.rs:1698-1709` (`focus_class`) | `_ => 2` | `∃` is ordered like `⊕` |
| `search/focus/mod.rs:1723-1775` (`focus_on`) | `_ => release` | an `∃` (positive) is released as negative and then lands in the stable sequent again (line 865) |
| `search/focus/split.rs:54-62`, `schedule.rs:246-250`, `horn/mod.rs:398-400,433-438,611-615`, `horn/proof.rs:216-219,242-249`, `search/net.rs:143-146`, `nets/graph.rs:144-147,460-463` | `_ => None`, `_ => {}`, `_ => return None`, `_ => unreachable!()` | mostly refusals by luck; `graph.rs:147` treats every other kind as a literal with a free axiom slot |
| `occurrences/reading.rs:252-267,312-321,371-375` | positions, implication, `_ => " ⊕ "` | position right by luck for unary binders; printing wrong |
| `sequents/fmt.rs:158-163`, `export/notation.rs:81-86,119-124` | `_ => " ⊕ "`, `_ => self.plus` | a binder would never reach `Between`, but a new binary would print as `⊕` |
| `proofs/derivation.rs:230-252` `Rule::classical`, `253-277` `Rule::intuitionistic` | `rule => rule`, `(rule, _) => rule` | `∀`/`∃` silently get no two-sided name (`∀L`/`∀R`) |
| `proofs/interactive.rs:1049-1098` | `_ => unreachable!("classical rules only")` | runtime panic in `proof()` |
| `export/rocq.rs:552-557` | deny-list `_ => {}` | a quantifier rule reaches NanoYalla output instead of `Unsupported` |
| `export/latex.rs:263-281`, `export/typst.rs:228-246`; `proofs/style.rs:93-95` (`symbol`) | `_ => NOTATION.zero`, `_ => "0"` | `∀`/`∃` in a label are set as `0` once added to `symbol` |
| `ordinary/mod.rs:365-381` (`Formulas::write`) | `_ if nested => '('`, `_ => {}`, `_ => symbols.iff` | a binder prints `(` or nothing |
| `ordinary/translate.rs:358-389` | `_ => &falsity` (the atom's name), `_ => (Term::Par(na, pb), Term::Tensor(pa, nb))` | a predicate is named `false`; a new core becomes `⊸` |
| `ordinary/derivation.rs:708-711` (`flips`) | `_ => false` | correct for binders; kept explicit anyway |

**Recommendation, open now.** Replace each by the explicit list of the
variants it means, and put `#![deny(clippy::wildcard_enum_match_arm)]`
on `search/focus`, `search/horn`, `export` and `ordinary`. The change is
mechanical and costs nothing at run time (D17). The checker matches
`Node` exhaustively already. The engines and the printers do not.
Leaving this means auditing every engine at step 38 for branches the
compiler does not show (item 2).

## 3. Invariants: occurrence sets, memo keys, hashing, orderings

| place | invariant | first-order |
|---|---|---|
| `ordinary/derivation.rs:594-605` `same`, `is` | a multiset of formulas is a sorted `Vec<NodeId>` | holds when instances are nodes of the derivation's arena and the arena is α-canonical. Otherwise members become `(NodeId, substitution)`. Decide in `api.md` (item 1) |
| `ordinary/derivation.rs:303-485` `Derivation::check` | the rules look at the principal formula's node only | `∀R`/`∃L` need the eigenvariable condition: free parameters per node, computed in index order without recursion |
| `ordinary/derivation.rs:616-626` `Tag { node, function, bangs, side }`; `core-ordinary.md`'s last bullet ("the read-back's tags carry no more than the node") | an occurrence of the image stands for one ordinary formula | under a binder an occurrence stands for many instances. The tag stays per occurrence, but the instance comes from the frame of the linear member. The rules file's bullet is then wrong and needs correcting |
| `search/focus/memo.rs:4-21,32-37` (`Key { theta: OccSet, gamma: Context }`; "a proved sequent is a fact … regardless of how the search reached it") | facts are ground | memoize ground stable sequents only (38 note) |
| `search/focus/context.rs:20-54`, `occurrences/set.rs:24` | a zone is a bitset plus extra copies over `OccId` | members `(OccId, frame)`; generic zone (38 note; the README of these notes, item 2 of §4) |
| `search/focus/classes.rs:4-48` | "every rule looks at an occurrence's connective, its subformulas and … its position"; classes by term | the class needs the frame too; skip it under binders |
| `search/focus/counts.rs:12-45`, `bias.rs:20-55`, `search/mod.rs:636-645` | balance, bias and multiplicity per atom | sound per predicate symbol (an instance keeps its symbol) |
| **Axiom partners by `Atom` equality**: `proofs/check.rs:1167`, `proofs/oracle.rs:271`, `search/additive.rs:191`, `proofs/interactive.rs:679`, `nets/mod.rs:289`, `search/net.rs:437`, `search/focus/mod.rs:1584-1620` (`2 * atom + sign`), the Horn engine's places (`horn/mod.rs:283-296`) | equal atom and opposite sign make an axiom | `p(a)` and `~p(b)` have equal atoms. If predicates reuse the literal kinds, the checker accepts that axiom and the Horn engine merges the places. Every such site must unify, or the goal must be refused (§5). Open now: route the eight sites through one `Forest` predicate ("dual literals") so that first-order changes one function (item 2) |
| `search/focus/parallel.rs`, `search/parallel.rs` | `&` premises and cubes are independent | sequential under shared metavariables (38 note) |

## 4. The proof term and the checker

| place | assumes | first-order needs | when |
|---|---|---|---|
| `proofs/mod.rs:116-158` `Node`, 16 bytes asserted | — | `Forall(OccId, Eigen, NodeId)`, `Exists(OccId, Witness, NodeId)` (38 note) | at step; the size assertion stays |
| `proofs/mod.rs:308-369` `Proof { forest, nodes }`, `Proof::new(forest, nodes, root)`; `search/mod.rs:444-462` `Answer::of_arena` | a proof is nodes over a forest | a witness arena the `Proof` owns, which the ordinary read-back and the certificate read | open now: one constructor that takes the side tables (README item "`Proof` carries what rebuilds its forest") (item 8) |
| `proofs/derivation.rs:543-560` `Inference { sequent: Vec<OccId>, rule, principal, premises, times }`, pub fields | a member is an occurrence | members carry an instance (a frame). The ordinary read-back reads exactly this (`ordinary/derivation.rs:734-739`) | open now: `#[non_exhaustive]` and a member type in `api.md`, so a parallel `frames` field is additive (item 6) |
| `proofs/check.rs:63-345` `Zone`, `Bag`, `State`; `349-358` `Dyadic` (pub fields `theta`, `gamma`, `any`); `448-460` `CheckError` (pub fields) | zones of `OccId` | `(OccId, frame)` members; the `Dyadic` that reports them | open now: `#[non_exhaustive]` on `Dyadic` (item 6); the checker at step |
| `proofs/derivation.rs:62-131,136,300-352` linear `Rule`, `ALL`, `from_str` | 34 rules | `∀`, `∃`, two-sided `∀L ∀R ∃L ∃R` (README of these notes, "`Rule` grows by one checklist") | at step |
| `ordinary/derivation.rs:45-131` `Rule` (exhaustive, `ALL: [Self; 25]`, `rule as usize` indexes the label tables through `Drawn::RULES`, `521-592`) | 25 rules of LK/LJ | `∀L ∀R ∃L ∃R` | open now: `#[non_exhaustive]` (item 5); the variants at step |
| `ordinary/derivation.rs:231-247` `Inference { left, right, rule, principal, premises }`, pub fields; a test builds one by literal (`961`) | a rule is determined by its principal formula | `∀R`/`∃L` name an eigenvariable and `∀L`/`∃R` a witness term: a field | open now: `#[non_exhaustive]` (item 5) |
| `ordinary/derivation.rs:665-806` `Image::read_back(&self, linear: &crate::Derivation)` | the unfolded linear derivation carries all the read-back needs | witnesses live in the proof term. `later.md` (follow-ups of ordinary logic) wants a read-back on the term (a DAG) anyway, and step 28 adds a stop to this signature | open now: when 28 changes the signature for the stop, take the `Proof` or a `Derivation` that carries frames (item 8) |
| `ordinary/derivation.rs:828-856` `rule_of` | a linear connective rule on an image is one ordinary rule | rows for `∀`/`∃`. A classical γ-copy is `?c`, which maps to `CL`/`CR` (`754-755`) already | at step |
| `ordinary/rocq.rs:371-378` statement `forall atoms : Prop,`; `23-38` `USED`; `76-94` brackets only for `¬`; `121-324` `tasks` | atoms are `Prop`s | `forall (D : Type) (d : D) (p : D -> Prop) (f : D -> D), …`; `ex_intro`, `ex` in `USED`; brackets around a quantifier as an argument | at step |

## 5. The engines and the dispatch

| place | assumes | first-order needs | when |
|---|---|---|---|
| `search/mod.rs:488-495` `dispatch` with `.expect("the last two rows take every goal")`; `501-532` `DISPATCH` (last rows `Fragment::LL`) | every goal lies in `LL` | a goal with the first-order bit lies in no row: a panic | open now: `dispatch` returns an error that names the fragment (item 3) |
| `search/focus/mod.rs:133-146` `admits` (mode only); `search/horn/mod.rs:38-43` (`Program::read` only, by shape) | every goal of the mode, or every goal of Horn shape, is decidable | the Horn engine would read `p(a)` and `p(b)` as one place, an unsound refutation or proof under a forced `--engine horn` | open now: each engine declares the largest fragment it decides as data (README of these notes, "registering an engine is a list"), checked once in `prepare` (`search/mod.rs:359-386`) before `admits` (item 3) |
| `search/net.rs:50-61`, `search/additive.rs:39-47` | refuse by `Fragment::contains` | refuse the first-order bit with no change, if the bit exists | — |
| `search/mod.rs:327-339` `fragment_of`; `goal_fragment` (`613-621`) | the fragment comes from the kinds | if predicates reuse `Kind::Var`/`DualVar` (attractive: every literal path stays), the kinds do not show arguments. The first-order bit must then come from the arena (any predicate with arguments), not from `Kind::fragment` | open now: say in `api.md` where the bit comes from (item 3) |
| `search/focus/mod.rs:268-310` `refutation` (`Unbalanced`, `Equation`) | — | sound per symbol without exponentials; quantifiers count nothing | — |
| `search/mod.rs:851-886` `Options` (private fields), `1417-1437` `Reason`, `1462-1491` `Statistics` (`non_exhaustive`) | — | a witness-depth bound, `unifications`, `bindings`: additive | at step |
| `cli/src/prove.rs:1019` `deepens` from `has_exponentials()` | — | true for a classical first-order image (it has `?`), so the notice is right | — |

## 6. JSON wire forms and their stability

| place | form | first-order | when |
|---|---|---|---|
| `serialize/sequents.rs:9-59` tags `V D 1 ⊥ ⊤ 0 ⊗ ⅋ & ⊕ ! ?`, proxy `{terms, ids, var_dict}` | no version | tags for predicates and binders, keys for terms and symbols under `serde(default)` (38 note §3) | open now: the `version` key (README of these notes); the keys at step |
| `serialize/proofs.rs:11-63` `Step` tags, `{sequent, proof}` | no version | `∀`, `∃` steps and `witnesses` | open now: `version` (item 8); the steps at step |
| `serialize/search.rs:10-42` fragment by name | six names | names with the first-order scheme (§1) | open now: the scheme (item 3) |
| `serialize/interactive.rs` | members as id lists | `[id, frame]` under binders | at step, if the member type is settled (item 6) |
| `ordinary/mod.rs:66-79,108-135,202-212` `Logic`, `Translation`, `Options` (serde, `deny_unknown_fields`) | — | probably an `equality` or `domain` option (§9) | open now: `#[non_exhaustive]` on `Options` (item 5) |
| **`ordinary::Sequent` and `ordinary::Derivation` have no JSON form**; `cli/src/ordinary.rs:136` writes the text tree for `--format json` | step 28 creates them (`plan/28-…md`, "From step 25"; `32-web.md`:171-172) | a symbol table with arities, a term arena, binder tags, the inference's term slot | open now: design them with that room and a version from the start (item 4) |
| `cli/src/batch.rs:540-551` JSON lines with `--logic` | the image's outcome, no logic or translation key | unchanged | — |

## 7. The exports

| place | assumes | first-order | when |
|---|---|---|---|
| `export/notation.rs:20-55` `Notation` (`atom: fn(&mut String, &str)`, `ordinary: Symbols`) | an atom is a name | an atom is a name applied to terms. The term printer must walk without recursing (`core.md`, "Nothing recurses over a formula", must extend to terms) | at step |
| `export/notation.rs:143-196` `Notation::sequent` over `&[OccId]` | a member prints from its term | from its instance | at step, with item 6 |
| `export/latex.rs:83`, `export/typst.rs:95`, `export/svg/mod.rs:229` the `ordinary` symbol tables; `export/svg/font.rs:89` `ADVANCES` (208 characters, fallback 650) | no `∀ ∃` | the symbols and their Euler widths | at step |
| `proofs/style.rs:31-44` `Labels` (exhaustive; `Table(BTreeMap<Rule, String>)` keyed by the linear `Rule`) | ordinary labels cannot be retabled (documented at `ordinary/derivation.rs:499-501`) | the same, with four more rules | open now: `#[non_exhaustive]` on `Labels` (item 5) |
| `export/rocq.rs:107-127` `Unsupported` (exhaustive) | four reasons | `Quantifiers` until step 31's kernel has them (38 note) | open now: `#[non_exhaustive]` (item 5) |
| `export/rocq.rs:242-252` `identifiers(atoms, lemma)` | identifiers for atom names | also for functions, constants and bound variables; `ordinary/rocq.rs:356` reuses it | at step |
| `proofs/size.rs:190-230` text width per term | an occurrence's width is its term's | an instance's width depends on its witnesses | at step (an upper bound, or per instance) |
| `sequents/fmt.rs:52-125` `Walk` (pub(crate); `operands: Fn(T) -> (Option<T>, Option<T>)`) | binary trees | n-ary terms and binder stops. The README of these notes wants `Walk` public at 28 | open now: if it goes public, give it n-ary children (`Between(T, index)`) and a binder stop; else keep it crate-private (item 9) |

## 8. The command

| place | assumes | first-order | when |
|---|---|---|---|
| `cli/src/argument_parsing.rs:11-27` `SYNTAX` | propositional connectives | quantifier and term syntax | at step |
| `argument_parsing.rs:599-602` `InputFormat::Tptp` docs; `665-707` `LogicArgs`, `LogicArg` ("propositional", "affine MALL, with no copy bound") | §0.1 | rewritten | at step |
| `cli/src/ordinary.rs:22-33` `sequent_in(text, format)`; `io.rs:140-148`; `batch.rs:489-497` (`Source::File(path, …)` read to text, then the path dropped) | a problem is one text | TPTP `include('…')` needs the file's directory or a root such as TPTP's `$TPTP` | at step (pass the path; a flag for the root) |
| `cli/src/ordinary.rs:48-66` `verdict_line`, via `Translation::target()` | the target depends on the translation alone | it also depends on whether the input is first-order | at step: `Image::target()` (additive) |
| `cli/src/prove.rs:1120-1126` `ordinary_mode`, `cli/src/lib.rs:222-232` `seq print --logic`, `batch.rs:130-140` (`takes`: `.p` files) | — | unchanged; `Axioms/*.ax` are not taken, which is right | — |
| `interact` | no `--logic` (`later.md`, follow-ups) | a witness given, or left open, in the session | at step |

## 9. The harness, the families and the TPTP reader

| place | assumes | first-order | when |
|---|---|---|---|
| `lltp.rs:122-234` `clauses` (shared by `read_tptp`) | only `fof(`; roles `axiom`, `hypothesis`, `conjecture` (`210-218`); no `include`; a `%` ends a line even inside quotes; `-` and `.` inside names mangled for LLTP (`152-167`) | `include` with a resolver; the roles `definition`, `lemma`, `theorem`, `assumption`; quoted atoms; the mangling kept off for TPTP | at step: `read_tptp_with(text, resolve)` is additive |
| `ordinary/parse.rs:55-64` `Pending`; `141-153,231-253` identifiers as atoms; `186-197` the TPTP operators | no binder, no term, no `=` | `![X,Y]:` and `?[X]:` as prefix operators on a unit formula (TPTP's scope), variables by case, terms, `=`/`!=` | at step. Open question: equality as a predicate plus axioms, as intuitionistic provers on ILTP do (inference) |
| `ordinary/parse.rs:346-387` `read_tptp` docs ("propositional"); `errors/mod.rs:57-62` `Error::Tptp` ("of propositional logic") | — | reworded | at step |
| `bench/src/problems.rs:4-55,240-` sources `family`, `lltp`, `spec`, `file` | the harness never reads ordinary logic. Step 25's ILTP run went through `linlog prove`'s batch | a `tptp:LOGIC:TRANSLATION:PATH` source, the expected verdict from the status | at step |
| `bench/src/run.rs:29-36` `HEADER` | no logic or translation column | columns at the end (the CSV columns are the interface, `bench.md`) | at step |
| `modules/bench.nix:43-46` `iltp` | the propositional tarball only | the first-order one, with its axiom files | at step (`new-tool` skill) |
| `families.rs:33-74` `Instance { sequent, mode, provable, copies }`, `non_exhaustive` | linear sequents only | first-order linear generators (38 note §2); ordinary families need an ordinary field | at step (additive) |

## 10. Tests and docs

- Tests that pin the propositional layer and grow:
  `ordinary/mod.rs:557-666`, `ordinary/parse.rs:389-447`,
  `ordinary/derivation.rs:916-997`, `core/tests/depth.rs:149-178`,
  `core/tests/export.rs:145-176` (snapshots `ordinary.{tex,typ,svg}`),
  `cli/tests/cli.rs:1258-` and README 778-860 through
  `cli/tests/readme.rs`. The depth test must add quantifier towers and
  terms nested 100 000 deep.
- `core/tests/serialize.rs` has no ordinary form. When step 28 adds
  the forms, add the test that reads a pre-step file unchanged (the
  README of these notes, "a version on every form").
- Docs that say "propositional": `lib.rs:12-14,116-119`,
  `ordinary/mod.rs:4-25,66,448-474`, `ordinary/parse.rs:4-7,346-355`,
  `errors/mod.rs:57-66`, README 778-860 and 1217-1223, and
  `.claude/rules/core-ordinary.md` (its first bullet, and its last,
  "Where quantifiers go", whose claim about tags fails, §3). Also
  `core.md`'s depth rule, which should name terms.

## 11. Decisions for the author

1. Equality: a predicate with axioms added to the hypotheses, or
   refused.
2. Free variables in the native syntax: refused, or read as constants.
3. Classical first-order: `?` on γ-formulas (the read-back stays LK), or
   static Skolemization of the ordinary input first (classically sound,
   but the read-back is then of the Skolemized sequent).
4. Bound-variable names: nameless in the arena with names invented when
   printed, or a name table beside the binder node, kept out of the hash
   key so that α-equal formulas stay one node.

## 12. Open now: the ten that cost most to leave, ranked

1. **One term representation for both arenas, written in `api.md`.**
   That means one first-order term type, nameless bound variables (one
   binder discipline), and what `Atom` means (`(name, arity)`), shared by
   `ordinary::Formulas` and `Sequent`. `translate`
   (`translate.rs:319-399`) builds images bottom-up by id.
   `read_back` maps occurrences to ordinary nodes by tag. The LK/LJ
   checker equates formulas by id (`derivation.rs:594-605`). Two
   representations chosen apart would need a renaming layer in both
   directions and would break equality by id. Stage 2's spike measures
   only what it is told to build.
2. **Explicit arms instead of the wildcards of §2** (about thirty sites),
   with the lint on the engine, export and ordinary modules, and the
   eight axiom-partner tests routed through one function (§3). Without
   it, a `∀` is silently a stable member (`focus/mod.rs:865`), an `∃` is
   released as negative (`1769`), a predicate is named `false`
   (`translate.rs:362`) and a label symbol prints as `0`
   (`latex.rs:280`). The compiler finds none of these. Now the change is
   mechanical and free at run time.
3. **The first-order fragment bit, its naming scheme, and an engine's
   largest fragment checked centrally, with `dispatch` returning an
   error.** The Horn engine admits by shape (`horn/mod.rs:38-43`) and
   would merge `p(a)` with `p(b)`. The focused engines admit by mode.
   `dispatch` panics on a goal no row takes (`search/mod.rs:494`). The
   fragment's names are a JSON, CLI and CSV interface
   (`serialize/search.rs:10-42`).
4. **The JSON forms of `ordinary::Sequent` and `ordinary::Derivation`,
   which step 28 creates, designed with room and a version**: a symbol
   table with arities, a term arena key, binder tags, and the
   inference's term slot. They are new, so the room costs nothing now.
   After 0.1.0 the web client reads them (`32-web.md`), and a change is
   a format version.
5. **`#[non_exhaustive]` on `ordinary::Node`, `ordinary::Rule`,
   `ordinary::Inference`, `ordinary::Options`, `lltp::Status`,
   `export::rocq::Unsupported` and `proofs::Labels`.** Each gains
   variants or fields for first-order. Marking one later is itself a
   breaking change. Matches inside the crate stay exhaustive. The README
   of these notes keeps the linear `Term`, `Kind`, `Node` and `Rule`
   exhaustive. These are peripheral and seldom matched exhaustively
   outside the crate.
6. **Members beyond bare `OccId` in public structs**:
   `proofs::Inference::sequent` (`derivation.rs:543-560`),
   `check::Dyadic` (`check.rs:349-358`) and `Interactive::goal`. Mark the
   structs non-exhaustive and name the member and frame types in
   `api.md`. The ordinary read-back, the exports, the interactive JSON
   and the CLI all read these, and a public field cannot change type
   after 0.1.0.
7. **A fallible `ordinary::Sequent::new`, and `Formulas::add` that
   checks its operands.** First-order needs closedness and arity checks
   at construction. Today an id from another arena passes and panics
   later. Changing an infallible public constructor later is breaking.
8. **Side tables in the proof and the read-back from the term.**
   `Proof::new` should take the witness arena (with cut formulas and
   mode, as the README of these notes lists). The proof JSON should carry
   a version and an optional `witnesses` key. When step 28 adds the stop
   to `Image::read_back`, it should read the `Proof` or a framed
   derivation, not the plain `crate::Derivation`, because witnesses live
   in the term. That also answers `later.md`'s follow-up of a read-back
   on the DAG.
9. **`Walk` n-ary with a binder stop if it goes public**, and the rule
   against recursion extended to terms. A public walk limited to two
   children freezes that limit. First-order terms are n-ary and can be
   nested as deep as formulas.
10. **Size assertions on `Term` (12 bytes) and `ordinary::Node`
    (12 bytes)** beside `Node`'s 16. They are D17's guard for the new
    variants (an argument index, not an inline list) and cost one line
    each. Without them a variant that grows every arena and the focused
    engine's hot loops compiles without a warning.
