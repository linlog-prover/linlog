# Research note for step 37: the focused inverse method

Written 2026-10-08 from the repository snapshot and the sources of section 5.
Citations are in brackets; "(inference)" marks a conclusion of this note's own.

## 1. The problem and the state of the art

**The method.** The inverse method (Maslov) proves a goal by forward
saturation: from initial sequents on the goal's atoms it applies the rules
forward, keeps every derived sequent unless a stronger one is there
(subsumption), and stops when a derived sequent subsumes the goal or
nothing new comes [Degtyarev and Voronkov 2001, as cited in Chaudhuri and
Pfenning 2005b, §3]. The subformula property limits the rules to instances
on the goal's signed subformulas [Chaudhuri and Pfenning 2005b, §2].

**Forward sequents and resource management.** The multiplicative split is
free forward: the premises' linear zones are inputs and the conclusion
joins them. What is hard is the context a conclusion does not get from its
premises: `init`'s unrestricted zone, and `⊤`'s both zones. The solution:
a forward sequent `Γ ; Δ →w C` has `Γ` a set standing for any superset and
a *weak flag*; a strong sequent (`w = 0`) stands for exactly `Δ`, a weak one
(`w = 1`, from `⊤R` or `0L`) for any `Δ' ⊇ Δ`. Initial sequents are strong;
`⊗R` joins the zones and ors the flags; `!R` must have a strong conclusion;
`&R` on two strong premises needs equal zones, on a strong and a weak one
the weak zone within the strong, on two weak ones the least upper bound of
the multiplicities [Chaudhuri and Pfenning 2005b, §3, Def. 3, Fig. 1;
Chaudhuri 2006, §3.2.2, Defs. 3.5–3.7]. A strong derived sequent is
provable as it stands, a weak one with any larger zone; a provable goal has
a derived sequent that is the goal or a stronger form of it [Chaudhuri and
Pfenning 2005b, Thms. 5, 6]. Rules whose conclusion a premise subsumes are
made *irredundant* by negative-existence side conditions: `⊗L` on a weak
sequent takes both operands when both are there, one only when the other is
absent [Chaudhuri 2006, §3.4].

**Subsumption.** `Γ ; [Δ]0 → C ≺ Γ' ; [Δ]0 → C` iff `Γ ⊆ Γ'`; a weak
`Γ ; [Δ]1 → γ` subsumes any `Γ' ; [Δ']w → γ'` with `Γ ⊆ Γ'`, `Δ ⊆ Δ'`,
`γ ⊆ γ'` [Chaudhuri 2006, Def. 3.7]; implemented as hierarchical tests
that fail early: flags, goal, `Δ`'s multiplicities, `Γ`'s, then the full
test [Chaudhuri 2006, Def. 4.11]. The propositional provers index nothing
(sequents are maps from labels to multiplicities, labels per subformula
occurrence except atoms) and name indexing as future work [Chaudhuri
2006, §4.1.2, §8.1]; the first-order prover uses substitution trees and
forward subsumption only [Chaudhuri and Pfenning 2005b, §5]. `Γ` is a set
by eager factoring, and the goal's own unrestricted hypotheses are
*globalized*, never recorded in derived sequents [Chaudhuri 2006, §4.1.4,
§6.3.1].

**Big-step rules from focusing.** Only *neutral* (stable) sequents are
kept; each *frontier* proposition (where a phase changes, computed by two
functions over the goal's signed subformulas) gets one derived rule whose
premises are the neutral sequents of the simulated backward focal-then-active
phase [Chaudhuri 2006, §6.2.2, §6.3, Def. 6.24, Lemma 6.25]. The forward
derived rules are complete with respect to the backward focused calculus,
but the proof is delicate, since forward finds *stronger* sequents and a
backward proof's premises need not be matched as they are [Chaudhuri 2006,
§6.2.2, Cor. 6.23]. The search is a lazy OTTER loop with an active and a
kept set; activating a sequent applies every rule to it, partially applied
rules are kept and *percolated* over the active set, so no rule is matched
twice against one sequent; conclusions are forward-subsumed before they are
kept [Chaudhuri and Pfenning 2005b, §5, Def. 14; Chaudhuri 2006, Defs.
4.17, 5.29, 5.30]. Which atoms are positive decides whether derived rules
chain forward (hyperresolution on Horn clauses) or backward (SLD
resolution); either choice is complete [Chaudhuri, Pfenning and Price 2008].

**Completeness and termination.** For propositional MALL the spec says the
space of sequents is finite, so saturation terminates; with exponentials
the method is complete for provable sequents only
[proof-search-specifications.md, "Alternative: focused inverse method"].
The thesis gives no termination theorem; an unprovable affine case did not
saturate and was cut at 1 000 iterations [Chaudhuri and Pfenning 2005b, §6].
(inference) In linlog's representation finiteness is exact: a cut-free
proof without exponentials uses every occurrence at most once, so a derived
sequent repeating an occurrence can never contribute and may be dropped,
leaving at most `2^(n+1)` sequents over `n` occurrences, weak flag included.

**Measured against backward search.** The only cross-prover numbers: the
propositional focused prover LPF against Gandalf's resolution (Gr) and
tableau (Gt) provers on eight problems. Focusing sped the propositional
prover up by 1.33 (basic) to 597.5 (affine2) times, the first-order one by
1.57 to 408.7; Gr fails five of the eight, Gt six; LinTAP does not
terminate on the two MELL problems; llprover solves none [Chaudhuri and
Pfenning 2005b, §6, Table 1]. Blocks world without focusing ran to memory
exhaustion after about 400 000 iterations; with focusing 45 iterations and
0.12 s right-biased, 26 and 0.04 s left-biased, whose derived rules have
one premise [Chaudhuri 2006, §7.2.1]. Imogen solved 261 of ILTP's 274
propositional problems (PITPINV 262, six others 175 to 238), and does
better on non-theorems [McLaughlin and Pfenning 2008, §5]; first-order,
784 (857 with a polarity retry) against iLeanCoP's 690 [McLaughlin and
Pfenning 2009, §5]. Imogen adds *recursive backward subsumption* and
*rule subsumption*, found path indexing faster than substitution trees,
and rebuilds a checkable proof term from a descendant graph [McLaughlin
and Pfenning 2009, §4.3–4.4].

**Implementations.**

| prover | URL | licence | last release | language | source |
|---|---|---|---|---|---|
| Sympli (Chaudhuri's propositional provers, 2003–2004) | https://github.com/chaudhuri/sympli | BSD-2-Clause | none; last push 2018-07-27, archived | Standard ML | [GitHub API, sympli]; [chaudhuri.info, systems] |
| Imogen | https://github.com/seanmcl/imogen | Apache-2.0 | none; last push 2019-05-31 | Standard ML | [GitHub API, imogen]; [McLaughlin and Pfenning 2009, §5] |
| Gandalf "nonclassical" 0.2 (Gr, Gt) | not found | not found | 0.2 as used in 2005 | Scheme (Hobbit) | [Chaudhuri and Pfenning 2005b, §6] |
| LinTAP; llprover | not found live; llprover was `bach.istc.kobe-u.ac.jp/llprover` | not found | — | —; Prolog | [Chaudhuri and Pfenning 2005b, refs. 20, 21] |

Chaudhuri's first-order prover was on his website in 2005 [Chaudhuri and
Pfenning 2005b, §6] and is not listed now [chaudhuri.info, systems]. No
publication on the inverse method for LLF proper was found; the nearest
are CADE 2005's motivation by CLF and Imogen's planned LF extension
[Chaudhuri and Pfenning 2005b, §1; McLaughlin and Pfenning 2008, §6]. Two
later papers bear on the step and were not read [Chaudhuri 2010; Chaudhuri
2015].

## 2. What the step needs

**Where it can win, today.** The prompt leaves it two cases: many
hypotheses with a small goal, and Mix [plan/37-inverse.md]. The nets are
gone: the Horn engine decides 3 026 of 3 137 LLTP Petri nets, 3 071 with
its refinements [core-search.md, "Why the Horn row takes only programs
with `!`"]. What remains is the intuitionistic library beyond the nets,
727 problems ending at the copy bound [later.md, "The focused inverse
method"], and `mix` at ten pairs (146 s) and eleven (over 1 200 s)
[bench/RESULTS.md, § mix]. (inference) The first is the forward method's
natural case: with the goal's `!` hypotheses globalized, a chain needs no
copy bound and no deepening, which is what Chaudhuri's QBF and
intuitionistic translations show. Mix is the weaker bet (item 6).

**Algorithms, costs, choices.**

1. *Frontier and rules.* One pass over the forest in index order computes
   the frontier and specializes one derived rule per frontier occurrence
   [Chaudhuri 2006, §6.3]. Recommendation: rules as data (premise schemas
   over occurrence ids), not closures, so the proof term is emitted while
   matching (section 3) and D17's terms have a slot.
2. *Sequent representation.* `(Θ, Γ, w)` over occurrence ids: a bitset for
   `Γ`, a sorted list of extra copies where an occurrence repeats, a set
   for `Θ`, the focused engine's `Context` and `OccSet` [core-focus.md,
   "Dyadic sequents"], costing the forest's width per sequent as the memo's
   keys do [core-focus.md, "The memo's layout"]. Recommendation: that
   layout, and drop any conclusion repeating an occurrence while the
   fragment has no exponentials (inference, section 1).
3. *Subsumption and index.* Strong-by-strong subsumption with `Θ`
   globalized is equality on `Γ` plus `Θ ⊆ Θ'`; weak sequents need subset
   queries both ways. Recommendation: a hash table on `(Θ, Γ)` for exact
   hits (the memo's record-in-chunks layout, no allocation per entry
   [core-focus.md, "The memo's layout"]) and feature-vector indexing for
   the subset queries [Schulz 2013]: a trie over subsumption-compatible
   counts (members of `Γ`, per root subtree, of `Θ`, the flag), forward
   queries walking feature values `≤`, backward `≥`; it adapts to set
   subsumption by the choice of features, cut E's subsumption attempts by
   97 % and its subsumption time by more than 22 times [Schulz 2013, §4,
   §6]. The hierarchical tests [Chaudhuri 2006, Def. 4.11] are the leaf
   test. Backward subsumption is worth having: Imogen saw half the database
   go when a strong sequent arrived [McLaughlin and Pfenning 2008, §6].
4. *Saturation loop.* The lazy OTTER loop with partially applied rules and
   percolation [Chaudhuri 2006, Defs. 5.29–5.30]; a round costs
   `O(rules + partial rules × active)`, and a `k`-premise rule is met in
   every order. Recommendation: keep it, with the engine's `Bias` choosing
   the atom polarity (the thesis's tenfold from single-premise left-biased
   rules on blocks world [Chaudhuri 2006, §7.2.1]).
5. *Exponentials.* `Θ` as a set, the goal's `?` members globalized
   [Chaudhuri 2006, §4.1.4, §6.3.1]. Linear multiplicities can grow forever
   and refutation needs a saturation that may never come [Chaudhuri and
   Pfenning 2005b, §6]. Recommendation: no multiplicity bound in the first
   session, `Unknown(Stopped)` under the time limit; the thesis's future
   work, a bound on linear multiplicities [Chaudhuri 2006, §8.1] (not in the source), as a
   measured option later, with a `Reason` of its own.
6. *Mix.* Forward, Mix is the disjoint union of two derived sequents, and
   is needed below a `⅋` (`⊢ a ⅋ b, ~a, ~b` with Mix), so it cannot wait
   for the end (inference). (inference) On `mix(k)` the derived sequents
   per pair are four, none the pair's two roots, so a forward engine still
   forms `5^k` unions; forward Mix is not obviously below the backward
   `3^n`. Recommendation: Mix as a way of meeting a premise (a disjoint
   union of active sequents), and measure `mix` first; if it does not win
   the row stays with the focused engine, as the prompt allows.
7. *Weak sequents and intuitionistic mode.* The checker's `any` flag is the
   weak flag (section 3). Every sequent of an intuitionistic proof has one
   output occurrence [core-proofs.md, "Intuitionistic mode"], so a
   conclusion with two outputs may be dropped; a weak one with none may
   still gain one (inference).

## 3. What linlog's library must offer for it

**Data model.** Formulas: the hash-consed NNF arena `Sequent`
(`core/src/sequents/mod.rs`), one-sided (D1); nothing to add. Occurrences:
`Forest` (`core/src/occurrences/mod.rs`: `kind`, `left`, `right`,
`children`, `subtree`, `parent`, `literals(atom, sign)`, `all_literals`),
ids stable across runs [core-forest.md]; the frontier functions are one
pass over it, as "nothing recurses over a formula" requires [core.md].
Sequents in search: `OccSet` (`occurrences/set.rs`: `is_subset`,
`is_disjoint`, `union_with`, `Hash`) gives the subsumption and disjointness
tests; `Context` (`search/focus/context.rs`: `insert`, `count`, `extra`,
`canonical_from`) holds the repeats MELL needs but is private to `focus`,
as are `Classes` (`search/focus/classes.rs`: `of`, `same`, `distinct`),
whose lemma lets one initial sequent stand per class pair and keys the
database up to relatives [core-focus.md, "Interchangeable occurrences"].
In the way: both live under `focus/`; the refactor should lift `Context`
and `Classes` to `search/` as shared types (D7 forbids two engines of one
algorithm, not shared data). Terms: a forward sequent carries no
substitution; D17 asks for the place of one (inference: a per-member slot
that is `()` propositionally).

**Proof term and checker.** `Node` (`core/src/proofs/mod.rs`, 14 variants
over `OccId`s, 16 bytes) is the dyadic calculus; `Top(o)` records no
context and the checker derives every sequent with an `any` flag: `⊗` sums
zones and ors the flags, `&` needs equal zones, or the absorbing side's
within the exact side's, or the pointwise maximum when both absorb, `!`
needs an empty zone and resets the flag [core-proofs.md, "The checker"].
That is Chaudhuri's weak-flag calculus rule for rule [Chaudhuri and
Pfenning 2005b, Def. 3, Fig. 1], so a forward derivation needs no new node
and no recorded context, and its multiplicities are the checker's `Bag`
(inference). A forward derivation appends a sequent's proof before any
sequent built from it, so "premise precedes conclusion" holds by
construction, and `Proof::new(forest, nodes, root)` drops the nodes of
backward-subsumed sequents [core-proofs.md, "Proofs are terms"];
`Answer::of_arena` (`search/mod.rs:444`) wraps it. Missing: a big-step rule
must emit its small steps as it matches, each kept sequent must carry its
root `NodeId`, and nothing collects dead nodes before the end (the focused
engine has `Arena::collect` [core-focus.md, "The kept arena is
collected"]), so the arena is bounded only by the `Account`. Every proof
passes `Proof::check` in `prove_goal` [core-search.md].

**Engine interface and dispatch.** `Decide` (`search/mod.rs:466`):
`admits(&Task)` and `decide(&Task, &Options, &Account, stop) ->
Result<Answer, Error>`; `Task` is forest, goal, fragment, mode, reading,
`roots`; `Answer` is `Result<Option<Proof>, Reason>`, `Statistics`, an
optional net and an optional `Refutation` (`search/mod.rs:408–440`). The
Horn engine (`search/horn/mod.rs:36`) is the model: `focus::refutation` on
a forked account first, then the search, then `of_arena`. A new engine is a
variant of the `#[non_exhaustive]` `Engine` with `Display` `"inverse"`, a
line in `Engine::implementation` (`search/mod.rs:768`), a `Feature`
variant and a `Row` in `DISPATCH` (`search/mod.rs:501`, read first row
down), and `Engine::parallel` must return `false` for it until it runs on a
pool [core-search.md, "One interface every engine implements"]. The stop is
polled wherever time can pass: per activation and every few thousand rule
applications, plus the set-up polls on large forests [core-search.md,
"Where it is polled"]. Memory: every table charges `memory::Account` by
capacity (`Charged`, `Account::fork`) and answers `Reason::MemoryLimit`;
the database cannot be emptied as the memo is without losing completeness,
so over the bound it is `Unknown` (inference). `Statistics`
(`search/mod.rs:1462`, `#[non_exhaustive]`): `nodes` as activations,
`memo_entries` as the database's size, `memo_hits` as forward-subsumed
conclusions, or new fields `generated` and `subsumed`, each with its serde
proxy line and CLI `statistics` arm. An exhausted saturation gets
`Refutation::Exhausted` from `prove_goal`; a `Saturated { sequents }`
variant is possible (`Refutation` is `#[non_exhaustive]`, `search/mod.rs:1258`).

**JSON wire forms.** `Engine` serializes as its name
(`serialize/search.rs:265`), so the outcome gains `"engine": "inverse"`;
`Outcome` writes `verdict`, `reason`, `fragment`, `mode`, `engine`,
`statistics` and the proof's keys; a new `Reason`, `Refutation` or
`Statistics` field needs its tag in the proxy and its line in
`core/tests/serialize.rs` [core-sequents.md, "Serialization"]. The proof's
JSON is unchanged: the same `Node` tags.

**Options.** `Options` has private fields and setters (`search/mod.rs:851`).
Reusable: `bias` (`Rarer`, `Factors`, `Auto`) as the engine's atom polarity,
Chaudhuri's left/right bias [Chaudhuri, Pfenning and Price 2008];
`memory_limit`, `check`, `occurrence_limit`. Without meaning here:
`copies`, `forward_copies`, `recursion_limit`, `test_period`, `memo_limit`
(documented, never refused [core-search.md]). Missing if item 5's bound is
taken: an option and a `Reason` for the multiplicity bound, since
`CopyBound` means something else; each default a named constant with a flag
(D16).

**Command, tests, harness.** `EngineArg` (`cli/src/argument_parsing.rs:1162`)
and its `From` impl; a `statistics` arm in `prove.rs` if the engine has
counters of its own; README's console blocks run under
`cli/tests/readme.rs` [cli.md]. The reference prover's `configurations`
(`search/reference.rs:675`) takes the engine. The harness takes its name in
`--engines` [bench.md]; `bench/targets.sh` pins the focused engine's
counters, which moving `Context` or `Classes` must leave identical
[CLAUDE.md, verification table].

## 4. Risks, open questions, and what the prompt should add

Risks.

- *Completeness of the forward derived rules* is the delicate theorem
  [Chaudhuri 2006, §6.2.2]; linlog's one-sided classical calculus with Mix
  and affine weakening is not Chaudhuri's two-sided intuitionistic one, so
  the panel's argument must be written for linlog's rules, with
  `search/reference.rs` as judge.
- *Memory*: the database is bitsets of the forest's width, and the
  intuitionistic library has forests of millions of occurrences
  [core-search.md, "Where it is polled"]; a sparse key is a follow-up the
  memo also names [core-focus.md, "The memo's layout"].
- *Mix* may not pay (section 2, item 6).
- *Non-saturation* with exponentials: `Unprovable` only from a saturation
  that ended, never from a bound (D9).
- *Affine mode*: weak sequents model affine logic [Chaudhuri and Pfenning
  2005b, §3], and linlog's affine mode is no decision procedure today
  [core-focus.md, "The spec's affine prune is wrong"]; whether the forward
  calculus with every sequent weak decides affine MALL is unverified here.

Open questions.

1. Does dropping sequents with a repeated occurrence stay sound with Mix
   and in affine mode? (inference: with weakening a repeat is weakened
   away, so it stays useless, but the argument must be made.)
2. What is the dispatch feature for "many hypotheses, small goal", and
   measured where? The prompt names the third baseline's undecided rows;
   the ILTP translations at the copy bound are the candidates.
3. Is `Bias` one choice for both engines, or does the forward engine want
   dynamic polarity [Chaudhuri 2010]?
4. Should `Options::memo_limit` cap the database at all, given that a
   capped database cannot refute?
5. Can the engine run one-sided in intuitionistic mode with the one-output
   prune, as the focused engine does [core-focus.md, "Two-sided is one
   constraint"]?

What the prompt should add: lifting `Context` and `Classes` out of `focus/`
as a separate, counter-neutral commit before the engine; the measurement
list (the undecided intuitionistic rows, `qbf`, `mix`, the families, each
engine forced, within `bench/` scopes); a reference check of the forward
rules on generated sequents with exponentials; the `Statistics` and JSON
additions named above, with README in the same commit; and the explicit
permission to leave the engine as `--engine inverse` without a row.

## 5. Sources

- Kaustuv Chaudhuri, *The Focused Inverse Method for Linear Logic*, PhD
  thesis, Carnegie Mellon University, 2006, technical report CMU-CS-06-162.
  https://www.csd.cmu.edu/sites/default/files/phd-thesis/CMU-CS-06-162.pdf
- Kaustuv Chaudhuri and Frank Pfenning, "Focusing the inverse method for
  linear logic", CSL 2005, LNCS 3634, pp. 200–215 (2005a).
  https://kilthub.cmu.edu/articles/journal_contribution/Focusing_the_Inverse_Method_for_Linear_Logic/6605744
- Kaustuv Chaudhuri and Frank Pfenning, "A focusing inverse method theorem
  prover for first-order linear logic", CADE-20, LNCS 3632, pp. 69–83
  (2005b). https://www.cs.cmu.edu/~fp/papers/cade05.pdf
- Kaustuv Chaudhuri, Frank Pfenning and Greg Price, "A logical
  characterization of forward and backward chaining in the inverse method",
  Journal of Automated Reasoning 40(2–3), pp. 133–177, 2008 (IJCAR 2006,
  LNCS 4130, pp. 97–111).
  https://researchportal.ip-paris.fr/en/publications/a-logical-characterization-of-forward-and-backward-chaining-in-th/
- Kaustuv Chaudhuri, "Magically constraining the inverse method using
  dynamic polarity assignment", LPAR-17, LNCS 6397, 2010; and "Disproving
  using the inverse method by iterated refinement of finite
  approximations", TABLEAUX 2015, LNCS 9323; both as listed at
  http://chaudhuri.info/research/papers/ (not read).
- chaudhuri.info, "Systems" page (Sympli entry), read 2026-10-08.
  http://chaudhuri.info/systems/
- GitHub API, repository metadata read 2026-10-08:
  https://api.github.com/repos/chaudhuri/sympli and
  https://api.github.com/repos/seanmcl/imogen; the repositories
  https://github.com/chaudhuri/sympli and https://github.com/seanmcl/imogen
- Sean McLaughlin and Frank Pfenning, "Imogen: focusing the polarized
  inverse method for intuitionistic propositional logic", LPAR 2008, LNCS
  5330, pp. 174–181. https://www.cs.cmu.edu/~fp/papers/lpar08.pdf
- Sean McLaughlin and Frank Pfenning, "Efficient intuitionistic theorem
  proving with the polarized inverse method", CADE-22, 2009, pp. 230–244.
  https://www.cs.cmu.edu/~fp/papers/cade09.pdf
- Anatoli Degtyarev and Andrei Voronkov, "The inverse method", in
  *Handbook of Automated Reasoning*, vol. I, Elsevier and MIT Press, 2001,
  pp. 179–272 (not read; cited through Chaudhuri and Pfenning 2005b and
  https://www.cs.cmu.edu/~fp/courses/atp/lectures/13-invcomplete.html).
- Stephan Schulz, "Simple and efficient clause subsumption with feature
  vector indexing", in *Automated Reasoning and Mathematics*, LNCS 7788,
  Springer, 2013, pp. 45–67.
  https://wwwlehre.dhbw-stuttgart.de/~sschulz/PAPERS/Schulz2013-FVI.pdf
- Tanel Tammet, "Proof strategies in linear logic", Journal of Automated
  Reasoning 12, pp. 273–304, 1994. https://link.springer.com/doi/10.1007/BF00885763
- Grigori Mints, "Resolution calculus for the first order linear logic",
  Journal of Logic, Language and Information 2(1), pp. 59–83, 1993.
  https://doi.org/10.1007/BF01051768
- Heiko Mantel and Jens Otten, "LinTAP: a tableau prover for linear logic",
  TABLEAUX 1999, LNAI 1617, pp. 217–231; Naoyuki Tamura, llprover,
  http://bach.istc.kobe-u.ac.jp/llprover (both as cited in Chaudhuri and
  Pfenning 2005b; URL not verified live).
- Repository files, snapshot of 2026-10-08: `plan/37-inverse.md`;
  `plan/later.md` ("The focused inverse method"); `plan/README.md` (D1,
  D7, D9, D16, D17, D19); `plan/reports/17-assessment.md` (2.1, 3.6);
  `proof-search-specifications.md` ("Alternative: focused inverse
  method"); `.claude/rules/core.md`, `core-search.md`, `core-focus.md`,
  `core-proofs.md`, `core-sequents.md`, `core-forest.md`, `core-horn.md`,
  `cli.md`, `bench.md`; `CLAUDE.md`; `bench/RESULTS.md` (§ mix);
  `core/src/search/mod.rs`, `search/horn/mod.rs`, `search/focus/context.rs`,
  `search/focus/classes.rs`, `search/reference.rs`, `proofs/mod.rs`,
  `occurrences/mod.rs`, `occurrences/set.rs`, `serialize/search.rs`,
  `cli/src/argument_parsing.rs`.

Sources checked 2026-10-08: 17 checked, 0 corrected, 0 removed, 1 claims marked.
