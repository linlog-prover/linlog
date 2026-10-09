# Design constraints for stage 2: what the API must fix now

Written 2026-10-08 for `plan/notes/api.md` (stage 2 of
`plan/28-audit-and-refactor.md`), from the specifications
(`fo-linear.md`, `fo-embeddings.md`, `mell-nets-spec.md`), the impact
maps (`impact-*.md`), the corpora (`corpus-*.md`) and this directory's
`README.md`. It merges their ranked lists into one list of decisions,
each with its options, a recommendation, the evidence, what the
propositional case pays now (D17: nothing measurable) and how a
reviewer checks the design against it. "(inference)" marks this note's
own conclusions; corpus items are cited by id.

The ranking is by the cost of a wrong choice: what breaks after 0.1.0
(D18), how many of steps 29 to 38 build on it before 38 arrives, and
whether the failure is a compiler error, a format break or a silent
wrong answer. Section 2 says what may wait for the steps.

## 1. The decisions, ranked

### D-1. What a member of a sequent is

*Options.* (a) `OccId` stays the member and step 38 adds a parallel
`frames` field to every struct that lists members; (b) a `Member(u32)`
newtype, equal to the occurrence id when the frame is empty and an index
into a table the `Proof` or the state owns otherwise; (c) a pair
`(OccId, FrameId)` everywhere.

*Recommendation.* (b), in every public signature that lists members:
`Inference.sequent`, `Dyadic`, `Interactive::goal`, `prove_goal`,
`engine_for`, `Drawn::sequent`, `Notation::sequent`, the interactive
JSON and the CLI's goal line, the structs `#[non_exhaustive]`. The
term's leaves (`Ax`, `Copy`) name members too: the bottom-up checker
cannot otherwise fix a leaf's instance under `?` copies, and `Node`
stays 16 bytes only with a one-`u32` operand (`impact-quantifiers.md`
finding 2, items 1–2). `fo-linear.md` §4.2's top-down frame pass is
compatible: the table is what it interns (inference).

*Evidence.* `fo-linear.md` §4.2, §5.2; `impact-fo-ordinary.md` §12 item
6; `README.md` §4 item 2.

*Cost now.* None: the same `u32`, and `ax: [x, y]` stays two integers.

*Check.* fo-linear-44 (`!forall x. p(x) |- p(a) * p(b)`: one body
occurrence, frames `[a]` and `[b]` in one `Θ`) and fo-linear-62 (three
copies of one clause) are statable as an `Inference.sequent` and a
`Dyadic` without a type change; `api.md` names `Member` and `FrameId`;
no public field lists a sequent as `Vec<OccId>`.

### D-2. The forest's contract

*Options.* (a) the forest is exactly the conclusion and step 34 builds
cuts on a second structure; (b) `roots()` is the conclusion in written
order, ids may continue past it with extra trees the forest records
(`cut_pairs()`, empty today), `sequent()` is the conclusion, whose arena
may hold terms the roots do not reach, and the preorder numbering is a
promise a foreign checker may recompute.

*Recommendation.* (b), with root sorting stopped before 0.1.0 in the
commit that regenerates the snapshots, and `is_roots` meaning "the goal
is every tree of the forest" (`impact-boxes.md` §11 items 1, 10;
`README.md` §3 conflicts 2–3).

*Evidence.* `impact-boxes.md` §0 (three meanings of `roots()`, `ids()`
and `sequent()` across some sixty call sites), §3 (the dual by offset),
§5 (the net engine `expect`s a net on an extended forest: a panic);
`mell-nets-spec.md` §1 (the cut kind "must exist in the model now").

*Cost now.* None at run time; one regeneration of the snapshots and of
`translate.rs`'s root pairing.

*Check.* mell-nets-18 (conclusion `⊢ ?X⊥, X`, cut on `!X`/`?X⊥`) is a
forest with roots `q, x2` and two extra trees of equal shape in recorded
order; the Rocq checker (31) recomputes the snapshot's ids from the
written sequent; `Forest`'s doc states the three meanings apart.

### D-3. The wire-form policy

*Options.* (a) additive keys only, no version: old readers fail on the
first unknown tag; (b) `"version"` on every form read back, missing
read as 1, every new key under `serde(default)` and skipped when empty,
and the rule that no first-order or net meaning rides on an old tag
through a new key; (c) `deny_unknown_fields` on the data proxies.

*Recommendation.* (b). No proxy denies unknown keys, so under (a) a
0.1.0 reader drops `cuts`, `vertices` or `witnesses` and fails later on
an id out of range, or reads `{"V": a}` plus an argument key as a
propositional atom (`impact-quantifiers.md` §3 item 8; `impact-boxes.md`
§11 item 3); (c) breaks the additive discipline. Also codes, not
English, for `Error` and `Refusal`, one rule for numbers above 2⁵³,
serde on `search::Options` (`README.md` §2).

*Evidence.* `fo-linear.md` §5.4 (the new tags and keys);
`mell-nets-spec.md` §7 (`vertices`, `boxes`, `jumps`);
`impact-fo-ordinary.md` §12 item 4 (the ordinary forms are born at 28).

*Cost now.* None at run time; the `version` key changes README's pinned
JSON once.

*Check.* A test per form reads a pre-change file unchanged; a
propositional `Sequent` writes byte-identically but for `version`; E1
of `fo-linear.md` §6 and mell-nets-05 are expressible by added keys
only; a reader given `version: 2` refuses by name; the ordinary forms
reserve a symbol table with arities, a term arena, binder tags and an
inference datum (`impact-fo-ordinary.md` §6).

### D-4. Atoms, terms and the fragment bit

*Options.* (a) `Atom` stays "a propositional variable" and step 38 adds
a predicate layer beside it; (b) `Atom` is documented as a predicate
symbol identified by `(name, arity)`, `Term` gets a 12-byte assertion
with two `u32` payloads and four variants reserved, `Sequent` gets a
second arena (locally nameless first-order terms) and a symbol table,
both empty propositionally, and `Fragment` reserves a sixth bit set by a
predicate argument as well as by a binder, named `MLL1`, `IMLL1`, ….

*Recommendation.* (b), with one first-order term type shared by
`sequents` and `ordinary::Formulas`, so the translation copies term ids
and the LK/LJ checker keeps "equal formula ⇔ equal id"
(`impact-fo-ordinary.md` §12 item 1); the "variable" vocabulary
(`Term::Var`, `Kind::Var`, `Sign::Var`) renamed before 0.1.0, `var_dict`,
`V`, `D` kept on the wire (`impact-quantifiers.md` §3 item 9).

*Evidence.* `fo-linear.md` §1, §5.1, §5.5, §7 (a), (b), (f);
`fo-embeddings.md` §4; `impact-quantifiers.md` finding 1: a ground
sequent with arguments sets no binder flag, yet `p(a)` against `~p(b)`
is an axiom at eight sites that pair literals by atom, so the bit comes
from the arena, not from `Kind`.

*Cost now.* None: two `const` assertions, docs, and empty tables.

*Check.* fo-linear-58 (`p(x, f(y), y)`: ternary predicate, function
symbol, nested terms) and fo-linear-03 (no constant: open
metavariables, `fo-linear.md` Q3) fit the reserved tables; `p(a) |-
p(b)` is refused by every engine and by `Proof::check` until `Ax`
compares instances; `Fragment::LL`'s doc says "propositional"; the
eight axiom-partner sites call one `Forest` predicate
(`impact-fo-ordinary.md` §3).

### D-5. The proof term, `Proof::new` and the checker's shape

*Options.* (a) `Proof::new(forest, nodes, root)` stays and 38 adds a
constructor; (b) `Proof::new` takes the side tables now (witness arena,
D-1's member table, the `Mode` the exporter needs; cut formulas through
D-2's forest), `Answer::of_arena` passes them, `Node` keeps 16 bytes
with `Forall(o, Eigen, p)`, `Exists(o, Witness, p)` and `Cut(o, l, r)`
reserved, and the variant policy is written down.

*Recommendation.* (b): `Term`, `Kind`, `Node`, `Rule` exhaustive with
the version plan in the changelog (0.2.0 at 34, 0.3.0 at 38;
`README.md` §3.1), `Rule::ALL` a slice, the checker described as two
passes, the frame pass and the eigenvariable set running only when the
forest has binders (`fo-linear.md` §4.2), step 31's Rocq `node`
parameterized over the member type, and the interactive `apply` taking
a value `{position, rule, split}` with room for a `witness`
(`impact-quantifiers.md` §3 items 2, 4, 10).

*Evidence.* `fo-linear.md` §4.1 (witnesses belong to nodes, not
occurrences), §4.3; `impact-boxes.md` §4; `impact-fo-ordinary.md` §12
item 8 (`Image::read_back` reads the `Proof`, where the witnesses are).

*Cost now.* None: empty tables, the same `Node` size.

*Check.* fo-linear-55 (one `∃` occurrence, two witnesses in two `&`
branches) needs the witness on the node; fo-linear-36 (an eigenvariable
inside what `⊤` absorbs) needs the eigenvariable set beside `any` in
`State`; mell-nets-21 (a cut inside a box) needs `Cut` under `Bang`;
fo-linear-57 (witness `f(f(c))` by hand) fits the `apply` value; the
`Node` size assertion and a `Rule::ALL` round trip are tests.

### D-6. The net's vertex, criterion and `from_proof`

*Options.* (a) `ProofStructure` is documented as the MLL structure
indexed by `OccId` and MELL nets are a second type; (b) one type whose
API speaks a `VertexId` distinct from `OccId` (the occurrence itself in
MLL, no table), `new(forest, &Criterion)` in place of `mix: bool`, edge
kinds as data (`Axiom`, `Cut`, `Jump`, collector and door edges), a box
table, `NetError` non-exhaustive with witnesses typed by vertex and
naming the box, and `from_proof` exhaustive over `Node`, refusing the
rest, with a bound and a stop.

*Recommendation.* (b): `mell-nets-spec.md` §4, §7 recommend the tree of
boxes with explicit membership, and 33, 34, 35, 36 each extend the one
flag (`README.md` §3 conflict 4). The MLL hot path stays by vertex =
occurrence, gated by a counter-exact target list for the net engine
taken before the change (`impact-boxes.md` §11 items 2, 9). SVG ids
`o<n>` are vertex ids, with prefixes reserved for boxes, doors, jumps
and cuts (item 8).

*Evidence.* `mell-nets-spec.md` §1, §9 Q1 (net equality is isomorphism,
not `links == links`); `impact-boxes.md` §11 items 4–6 (`from_proof`
keeping `Ax` only returns a wrong net for a cut proof; unfolding shared
subproofs per path is exponential, so the bound belongs in the first
signature).

*Cost now.* None on the MLL path if the identity holds; the `links` and
`tests` counters of the net engine's list are the measure.

*Check.* mell-nets-04 and 05 (three and two instances of one `X⊥`) are
representable, which `OccId` indexing cannot do; mell-nets-17 against
30 (one structure, correct with Mix only) is one `Criterion` value;
mell-nets-09, 18 and 06 are edge and table kinds; mell-nets-33 needs an
error naming the box; `ProofStructure` has no `PartialEq`.

### D-7. The zone as a type parameter of the focused engine

*Options.* (a) `Context`, `OccSet` and `Key` stay concrete and 38
duplicates the engine; (b) `Engine<Z: Zone>`, `Z = Context`
propositionally and a framed zone later, `Key<Z>` and `Classes`
following, after step 37's counter-neutral lift of `Context` and
`Classes` to `search/`.

*Recommendation.* (b), decided by the spike (stage 2, item 4): adopted
only if the target set's columns stay equal and pinned CPU time within
a few percent; otherwise D17's duplicated fast path, and `api.md` says
which (`README.md` §3 conflict 6).

*Evidence.* `fo-linear.md` §5.2, §5.5 (monomorphised at `Z = Context`;
code size is what it does not settle); `impact-quantifiers.md` §3 item
3.

*Cost now.* The one decision whose cost is not zero by construction: it
is measured, and the measurement is the acceptance test.

*Check.* `bench/targets.sh` columns `verdict`, `nodes`, `splits`,
`memo_hits`, `memo_entries` equal to `after-bias.csv`; fo-linear-44 and
62 statable in the framed instance's `Θ`; `api.md` names the `Zone`
trait and the frame type.

### D-8. The extensibility policy

*Options.* (a) nothing marked, every addition a major version; (b)
`#[non_exhaustive]` before the tag on `Error`, `NetError`,
`rocq::Unsupported`, `Dyadic`, `Inference`, `CheckError`, `Size`, every
options value and `svg::Style`, `Mode` (constants and `with_*`
builders), `ordinary::{Node, Rule, Inference, Options}`, `lltp::Status`,
`proofs::Labels`; the four core enums exhaustive.

*Recommendation.* (b): marking later is itself breaking, and each of
these gains a variant or field at 33, 34, 36 or 38 (`README.md` §4
item 1; the three impact maps' last items).

*Cost now.* None.

*Check.* A grep over the list; callers write `Options::default()` plus
assignments; the changelog names the exhaustive enums and each bump.

### D-9. One stop and one account on every long entry

*Options.* (a) `FnMut() -> bool` stops and `Forest::DEFAULT_LIMIT` with
no way to pass another; (b) `Stop::poll(progress)`, kept for closures
by a blanket impl and polled by searches, checks, exports, elimination
and refuters alike, and a `within` constructor charging
`memory::Account` on every entry that builds a forest or a table.

*Recommendation.* (b), at 28 (`README.md` §2, §3 conflict 7;
`impact-boxes.md` §11 item 4; `fo-embeddings.md` §1.5: the classical
first-order image is bounded by the copy bound only, so `Unknown`
reaches the classical path).

*Cost now.* None measurable: existing poll sites change type.

*Check.* fo-ordinary-01 (the drinker, classically) and fo-ordinary-13
(an infinite countermodel) end in `Unknown` under a stop, not a hang;
the Horn engine's Petri-net proofs pass `from_proof` inside a limit
(`impact-boxes.md` §9); every `Deserialize` has a `_within` twin.

### D-10. A public, non-recursive walk with n-ary children and binders

*Options.* (a) `Walk` stays crate-private and binary; (b) public, with
`Enter`, `Between(index)`, `Exit` and a binder stop, a term walk inside
a predicate's `Enter`, `Forest::formula(o)` printing with the names of
the binders above `o`, "nothing recurses" extended to terms, and a
bound on unfolded terms beside the occurrence bound.

*Recommendation.* (b): 29's translators, 31's Rocq formulas, 32's goal
view and 36's printer are built on it before 38 (`README.md` §2;
`impact-quantifiers.md` §3 item 5; `impact-fo-ordinary.md` §12 item 9).

*Cost now.* None: one stop no printer uses.

*Check.* fo-linear-58's ternary predicate needs `Between(index)`;
`core/tests/depth.rs` gains binder towers and terms nested 100 000
deep; the checker compares witnesses by hash-consed id, never by
unfolding.

### D-11. Exhaustive arms where wildcards hide new variants

*Options.* (a) leave the `_ =>` arms; (b) explicit lists at the roughly
thirty sites, `clippy::wildcard_enum_match_arm` denied on
`search/focus`, `search/horn`, `export`, `ordinary` and `nets` with
allows on the shape readers that refuse, and `const fn` helpers
(`Rule::splits`, `Rule::has_principal`) for the `matches!` sites.

*Recommendation.* (b). The failures are silent: a `∀` enters the stable
sequent unopened, an `∃` is released without a witness, a binder body
keeps its old index in `optimize_terms`, a `Cut` builds a Mix node
(`impact-quantifiers.md` §2.2; `impact-fo-ordinary.md` §2;
`impact-boxes.md` §11 item 7).

*Cost now.* None; the target set's columns stay equal.

*Check.* The lint passes; fo-linear-05 (`∀` is asynchronous, so `∀R`
precedes `∀L`) is what the `decompose` wildcard would have lost.

### D-12. The ordinary layer's own constraints

*Recommendation.* Beyond D-4's shared term type: `ordinary::Sequent::new`
and `Formulas::add` fallible (closedness and arity checks at
construction); `Translation::target()` a value (`affine MALL`
propositionally, `affine LL` with a quantifier) and the docs stop
implying the classical path terminates; `Inference` with a datum slot
(`Witness`/`Eigen`, `None` today); `Image::read_back` over the `Proof`;
TPTP options on `read_tptp`, not on `ordinary::Options`
(`fo-embeddings.md` §2, §4; `impact-fo-ordinary.md` §0, §12 items 7–8).

*Cost now.* None.

*Check.* fo-ordinary-39 (witness `f(a)`) is an `Inference` datum;
fo-ordinary-01's read-back starts `CR, ∃R(c), …` with `c` a fresh
constant the Rocq statement takes as an inhabitant; fo-ordinary-43
(minimal: `⊥` an atom) still holds; fo-ordinary-49 (an ILTP file) is
readable by the reserved options.

## 2. What the design may leave to the steps

Everything whose cost is the same whenever it is paid and which fixes
no public signature or wire form:

- The variants themselves, the parser's binder stack and the text
  syntax (the author decides: `fo-linear.md` §7 Q1).
- Levels, the trail, unification, answer enumeration across `⊗`
  (`impact-quantifiers.md` finding 3), the ground-only memo, counts and
  bias per symbol: internals behind D-7's parameter.
- The frame pass and the eigenvariable rule's proof (`fo-linear.md`
  §4.2, Q5), for the panel at 38.
- The box table's fields, sequentialization with boxes, the jump
  criterion's exactness and `void` (`mell-nets-spec.md` §3, §9): behind
  D-6's `Criterion` and `VertexId`.
- Cut elimination's working representation (`impact-boxes.md` §1:
  promise no O(1) arbitrary unlink on `ProofStructure`).
- Equality in TPTP, the roles, the first-order default translation
  (`fo-embeddings.md` §6); the Rocq kernel and `Prop` against `Type`
  (the design fixes only that `rocq::Unsupported` can grow);
  first-order families, a first-order Horn engine, a second reference
  prover (`impact-quantifiers.md` §2.5, §2.9).

(Inference) Section 1's order is also the order in which a wrong choice
is noticed latest: D-1 to D-4 fail silently or in files that cannot be
recalled, D-5 to D-8 as version bumps, D-9 to D-12 as rework inside
this repository.
