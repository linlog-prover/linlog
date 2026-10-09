# Walk-through: step 38 (first-order linear and ordinary logic) against `api.md`

Read: the brief, `api.md` in full, `plan/38-first-order.md`, the research
notes `38-first-order.md`, `fo-linear.md`, `impact-quantifiers.md`,
`impact-fo-ordinary.md`, `fo-embeddings.md`, `corpus-fo-linear.md`, the 48
register entries naming 38, and the present `sequents/`, `occurrences/`,
`Cargo.toml`. Section numbers are `api.md`'s. Nothing run or edited.

## 1. The first change, sketched

10.10 makes commit 1 "the data model alone, with its refusal everywhere".
Written against the design:

```rust
// sequents::fo (public types, private tables in Sequent)
pub struct SymbolId(u32);  pub struct ArgsId(u32);  pub struct FoTermId(u32);  pub struct Eigen(u32);
pub enum FoTerm { Bound(u32), App(SymbolId, ArgsId), Eigen(Eigen), Meta(u32) }   // 12 bytes, closed
// sequents
Term::{Forall(TermId), Exists(TermId)}     Kind::{Forall, Exists}     // 0.3.0; dual swaps; Forall negative
Fragment::QUANTIFIERS = 32                  // set by a binder only; names MLL1 .. LL1
Sequent { terms, roots, atoms, antecedents, fo: Option<Box<Fo>> }
//   Fo { symbols: Vec<(String,u32,SymKind)>, atom_of: Vec<(SymbolId,ArgsId)>, terms: Vec<FoTerm>,
//        args: CSR, binders: Vec<(TermId,String)> }       // None for propositional input
Sequent::fo_size() -> u64 (saturating)      Limits::terms: Option<u64>
Builder::{predicate, function, bind}        // 3.1, additive
// every engine's admits: NotTaken::Fragment on the bit; Proof::check -> Fault::Instance on a forest with binders
```

Corpus items in these types (occurrence ids by the forest's preorder):

**fo-linear-44** `!forall x. p(x) |- p(a) * p(b)`, one-sided `⊢ ?∃x.~p(x), p(a)⊗p(b)`,
`antecedents: 1`. Occurrences 0 `?`, 1 `∃`, 2 `~p(#0)`, 3 `⊗`, 4 `p(a)`, 5 `p(b)`
(`forest.len() = 6`).

```json
{"version": 3, "terms": [{"D":0},{"∃":0},{"?":1},{"V":1},{"V":2},{"⊗":[3,4]}], "roots": [2,5],
 "atoms": [[0,[0]],[0,[1]],[0,[2]]], "symbols": [["p",1],["a",0],["b",0]],
 "fo_terms": [{"B":0},{"S":[1,[]]},{"S":[2,[]]}], "binders": [[1,"x"]], "antecedents": 1}
```

Proof, with members 6 = (2,[a]) and 7 = (2,[b]) in the proof's `Instances`:
`ax[6,4]; ∃[1,1,0]; copy[1,1]; ax[7,5]; ∃[1,2,3]; copy[1,4]; ⊗[3,2,5]; ?[0,6]`
(`Node::Exists(Member, FoTermId, NodeId)`). Checker by 3.8 plus 10.10(h):
`∃` takes the body member (2,[a]) and puts 1; `Copy` moves 1 to Θ; the second
`∃` uses the same principal member 1 with witness `b`; `?` drops 1 from Θ.
This confirms 10.10(c): one body occurrence, two members, no signature change.

**fo-linear-03** `forall x. p(x) |- exists x. p(x)`: no constant exists. `⊢ ∃x.~p(x), ∃x.p(x)`;
`fo_terms [{"B":0}]`, one atom `p(#0)`, **both literals are the same `Atom`**. The
proof needs a witness term over nothing: either `Meta(0)` left open (the proof's `terms`
extension gets `{"M":0}`, the checker treats it as a constant) or a fresh constant.
`∃[2,w,n0]` over `ax[m(1,[w]), m(3,[w])]`, then `∃[0,w,n1]`. See items 2 and 3.

**fo-linear-36** `exists x. q(x), forall x. p(x) |- exists y.(p(y) * top)`:
`⊢ ∀x.~q(x), ∃x.~p(x), ∃y.(p(y)⊗⊤)`. Eigenvariable `e` from the `∀`; `∃x`
and `∃y` use witness `e`; `⊤` absorbs the body member (1,[e]), which no node names.
`Forall[0,e,n4]` takes (1,[e]) absorbed (3.8's `take`), removes `e` from the set that
the two `Exists` nodes added, root set empty. The checker must recompute members that
the table does not list (absorbed ones): fine as a pair lookup, see item 3.

Further commits (not sketched): parser and printer, JSON level, `Proof::with_instances`,
the checker's frame pass, `Step::witness`, the focused engine over `Z: Zone`, the
ordinary layer, Rocq refusal.

## 2. Workarounds

Numbered; **blocking** 1, **friction** 8, **note** 7.

**1. blocking. What `atoms: Vec<String>`, `atom_name` and `atom_names()` hold for `p(a)` and `p(b)`.**
Needs: ground first-order input is "accepted by every engine from commit 1 as
propositional input with structured atoms" (3.2, 10.10), so the exporters, the
translators (29, R74/R245), the harness and the Rocq writer see `p(a)` and `p(b)` as
two atoms. In the way: 3.1 keeps `atoms: Vec<String>` "one distinct name per atom"; 3.2
documents `atom_name(a)` as "the name of the atom's predicate symbol". Today
`atom_names() -> &[String]` is public and used as both the atom count and the display
key (33 sites: `rocq.rs:560` `identifiers`, `horn/mod.rs:198`, `focus/mod.rs:289`
`Unbalanced.name`, `proofs/size.rs:196` widths, bench `run.rs:826`). With the symbol-name
contract these give duplicates: `identifiers` declares `Parameter p` twice, so the
NanoYalla certificate states a collapsed (weaker) sequent, a silent wrong certificate;
a translator for an external prover sends `p` for both atoms, a silent wrong problem,
and R75 routes ground first-order input to propositional-only tools because its fragment
stays `MLL`. Changing the meaning of a public accessor after 0.1.0 is a break.
Smallest change: at 28 fix the contract as "`atoms` holds one distinct *key* per atom;
`atom_name(a)` is it (the name for a nullary atom, [38] the canonical text `p(f(a), b)`
for a ground one, `p(#0)` for an open one)", add `atom_count()` for the `len()` uses, and
let 38 add `atom_symbol(a)` for the predicate symbol. Then 3.2's "one crate-private writer"
covers display, and `identifiers` mangles unique keys.

**2. friction. Open atoms pair by `Atom` id; "soundness by construction" does not hold for them.**
Needs: `exists x. p(x) |- forall x. p(x)` (corpus 04) is `⊢ ∀x.~p(#0), ∀x.p(#0)`: the
two literals are the *same* `Atom` with opposite signs, so the eight pairing sites call
it an axiom. 3.2 says open atoms "exist only under a binder, whose kinds are new
variants, so every exhaustive `match` fails to compile"; but none of the eight sites
matches on a binder kind, they compare literals. The protection is the `QUANTIFIERS` bit
plus `admits` plus the checker's `Fault::Instance` (10.10), which is exactly the guard the
decision rejects, though here an exact one (an open atom exists only under a binder). Why
it matters: a forced engine or the checker accepting a false axiom is a silent wrong
answer. Smallest change: say in 3.2 that the guard is the bit, add `Sequent::atom_is_open(a)`,
make `Forest::dual_literals` (3.3 item 3) `debug_assert!` both atoms closed and document
"candidates, not partners" for open atoms; the commit-1 test forces every engine and the
checker on corpus 04 and expects the refusal.

**3. friction. `Instances` (3.14) lacks the tables an instance and a witness need.**
Needs: (a) a witness `s(s(z))` or a fresh constant (corpus 03, 62) is `App(sym, ArgsId)`:
the proof extends `args` (and `symbols` for a constant) as well as `terms`; (b) comparing
members at `Ax` requires the *substituted atom* `(SymbolId, ArgsId)` of `(o,F)`, interned,
and mapped back to a sequent `Atom` when it equals one (`p(a)` from frame `[a]` equals
the sequent's atom `p(a)`): that needs the atom reverse map and a substitution walk over a
shared argument DAG, neither named in 3.14 or 3.8; (c) `Atom` ids cannot key the
engine's counts, memo or loop check for instances (an instance atom is not in the table):
the seam must speak `(SymbolId, ArgsId)`; (d) P4 says the checker shares no code with the
engines, but `Instances` is both the engines' product and the checker's input: the
checker's interning must be its own overlay (the proof's table read-only, absorbed members
not in it). Smallest change: add `args` and `symbols` extensions and a named
`Instances::atom_of(member) -> (SymbolId, ArgsId)` contract to 3.14, and a sentence in
3.8 that the checker builds its own overlay.

**4. friction. Frames as flat lists are quadratic; R43 asks for 100 000 nested binders.**
3.14: a frame is "a list of FoTermIds, one per binder above" in a flat CSR. A tower
`∀x₁…∀x_D.` proved with `Forall` at each level has a member of frame length k at level k:
Σk = D²/2 entries (5·10⁹ at D = 10⁵, 20 GB), unbounded by `Limits::occurrences` or
`terms`, and the wire `frames` key would carry it. Same for the checker's eigenvariable set
if each `State` copies it. Smallest change: frames are hash-consed cons cells
`(parent: FrameId, term: FoTermId)` (wire `frames: [[parent|null, term]]`), the set
flows as a persistent structure; state both in 3.14/3.8, and make readers refuse
`instances` longer than three per node and entries nobody references (3.8's "bounded by the
nodes" needs a reader rule).

**5. friction. `Error::describe(&Forest)` cannot print a framed member (4.1, 3.8 `CheckError`).**
Needs: `CheckError::Invalid.premises: Vec<Dyadic>` of members past `forest.len()` shown as
formulas. `describe<'a>(&'a self, forest: &'a Forest)` and `ShapeError::describe` take only a
forest; members need the proof's table. Smallest change: at 28 make the parameter a
sealed trait `Members` implemented by `Forest`, `Proof`, `Derivation`, `Interactive`
(3.4 already gives all four `formula(m)`); otherwise 38 adds a second entry point next to
"one `describe`" (F48).

**6. friction. A `Member` is owner-relative, but three public hand-overs cross owners.**
3.4 says a member past `forest.len()` indexes "an instance table the owner keeps". Crossing
points with no table in the signature: `Proof::goal() -> &[Member]` compared by
`close_with` against the session's goal as a multiset (3.10, `GoalMismatch`); `Disproof {
goal: Option<Box<[Member]>> }` (3.12) for an unprovable framed goal (no `Instances`
field); `Goal::framed(forest, &instances, members)` (3.1/8.1) carries no *bindings* of the
session's open metavariables in, and `Outcome` carries none out, so a `close` that binds
`X := a` for goal 1 cannot tell the session (goal 2 shares `X`). Smallest change: say in
3.4 that equality of framed members across owners is by `(OccId, resolved frame)` and the
crate's `Instances::import` does it; give `Disproof` a private `instances` with an additive
`with_instances` (as `Proof` has); give `Outcome` (non-exhaustive) a `bindings` field and
`Goal::framed` a `&Bindings`.

**7. friction. `Step::witness(self, w: Witness)` has no payload type that fits the features and P9 (3.10).**
Needs: a witness given by hand, or `Open`. `Witness` is undefined; the `interactive` feature
has no `parse` (Cargo.toml), so text cannot be the payload; a nested public term value
recurses (P9, R43); a `FoTermId` is meaningless to a caller. `Inference::datum() ->
Option<Binding>` and the eigenvariable/metavariable *display names* (3.14 has no name table
for `Eigen`, `Meta`) are also undefined, and `Witness` text `0` would collide with a split
position in the command's `apply G P RULE [P…]` if numerals are constants (Q1). Smallest
change: `Witness::{Open, Term(TermBuf)}` with `TermBuf` a flat topological value of
`fo_terms`' wire shape, `Binding` named, an `Instances::names` table; `serialize` maps text
only in the front end.

**8. friction. New counters versus "every propositional JSON byte" (10.10 l, 7.1, 8.5).**
Needs: `unifications`, `bindings` (8.5 "added" row, R9). 7.3 says `statistics` writes
"every counter, always" and 10.10 (l) promises propositional JSON bytes unchanged; an
appended `"unifications": 0` changes every outcome, the README blocks and the lock.
Smallest change: appended counters are written only when non-zero (or under a nested
`"first_order"` key present for first-order runs), stated in 8.5/7.3; the CSV columns still
append.

**9. friction. The ordinary layer keeps the drafts' `Pred(u32, ArgsId)` (3.13, 10.10 j).**
Needs: one atom notion for both layers (3.13: "`translate` copies term ids"). 3.13 reserves
`ordinary::Node::{Pred(u32, ArgsId), Forall, Exists}` beside `Atom(u32)`, the two-node
representation decision 1 set aside: a nullary `p` is `Atom` or `Pred(p, EMPTY)`, breaking
"equal formula ⇔ equal id" unless forbidden; translation must intern `(sym, args)` into the
linear atom table anyway. Smallest change: ordinary `Node::Atom(u32)` indexes the same
interned atom table, so 38 adds `Forall`, `Exists` only (12 bytes kept), `translate` copies
atom ids, `Formulas::add` checks arity and closedness (R249), and 3.13/10.10 (j) say so.

**10. note. No public read access to symbols, arguments, binders (R212, R245, 29).**
3.14's types are listed in `sequents::fo` but 2.1/2.2 export neither the module nor
accessors, and `Visit` has no argument or binder-name stop. A LinearOne writer
(R212), 29's translators and 32's goal view cannot print an atom with arguments except
through `Display`. Smallest change: reserve in 2.1 `Sequent::{symbols, atom_symbol,
atom_arguments, fo_term, binder_name}` and a flat term walk, all iterative.

**11. note. Wire details of 7.3 under-specified.** `symbols: [[name, arity]]` omits the
kind (3.14 has predicate vs function): `p/1` as predicate and function collide; mixed `atoms`
entries (`string | [symbol, ids]`) are hard to type; the nullary symbols are implicit.
Pick: one kind namespace (refuse a name used as both) or write the kind.

**12. note. Canonical form of the empty tables.** `Sequent: Eq, Hash` is structural (3.1);
`add`/builder can leave `fo: Some(trivial)` for a sequent equal to a propositional one, and
the writer must still pick level 1. State the invariant `fo.is_none()` iff all atoms are
nullary and there is no binder, enforced by `optimize` and `Sequent::check`.

**13. note. `Limits::terms` does not bound instances (R250).** The unfolded `fo_size` bounds
terms; the instance table, frames and the checker's overlay are bounded only through
`memory_bytes` (5.4) and by item 4's reader rule.

**14. note. `Refutation::Unbalanced { atom }` for open atoms (10.10 f).** The count is per
predicate symbol, so `atom` alone is wrong for 31's `Disproof::check`. Add a field `by_symbol`
or a variant (`Refutation` and `Unbalanced` are non-exhaustive, so additive).

**15. note. First-order nets (R108).** 7.3 reserves a `substitution` key; 3.11's `Criterion`
comment and `NetError` list name no first-order criterion (eigenvariable dependency) or
error; both are non-exhaustive, so additive, but the engine row in 10.10 (f) refuses.

**16. note. Fragment names.** "append `1`" gives `MLL with units1`; only two bits (64, 128) are
left before 36. Choose the spelling now (`MLL1+units` or names by bit set).

## 3. Register entries naming step 38

Met or placed: R1 (6.2, 7.2), R6 (7.1), R8 (7.3 substitution key; criterion not placed, item 15),
R9 (8.5; byte clash, item 8), R16 (3.14, 7.3, 10.10 a, b), R17 (7.3 tags `∀ ∃`, members through
`instances`; "unifier never implicit" holds), R38 (6.2, 10.10 k; the `Reason` variants unnamed),
R39 (5.4, 10.10 d to f), R42 (P9, 5.3), R50 (2.5), R51 (3.5), R52 (amended by decision 1; ground
first-order is `MLL`), R59 (3.1 builder, additive), R62 (3.2, 3.14, size assertions P10), R63 (P10, 3.3),
R64 (3.14, 10.10 c), R65 (3.6: binders keep their position; no new `ShapeError` case is needed),
R66 (10.10 i, `Inference::datum`; `Binding` undefined, item 7), R68 (9.2 `node (M : Type)`),
R88 (8.4), R97 (3.10; payload, item 7), R106 and R107 (10.10 d to f), R109 (8.2 `Task::instances`,
the literal seam named only in 10.10 f), R117 and R125 and R126 (3.7, 3.8), R135 (8.4, 8.6),
R136 (4.4 reserved codes), R170 (3.2 writer, 3.3 `Forest::formula`), R209 (10.10 h: a first-order
reference prover), R211 (3.1, families additive), R216 (10.10 l, 11), R230 (this note), R237 (3.1),
R244 (3.4), R248 (P3 lint), R249 (3.13), R251 (3.2).

Partly or not: **R43** (stack safe, but quadratic frames, item 4); **R67** (rows placed, `Pred`
inconsistent, item 9); **R161** (refusal keyed to the bit only; ground first-order reaches Rocq,
item 1); **R212** and **R245** (no accessors or term walk, item 10); **R250** (terms bounded, not
instances, items 4, 13); **R108** (item 15); **R180** (the command's area; the `apply` witness
token is open, item 7); **R242** (second Rocq development: 9.2 names a new `FORMAT` only).

## 4. What fits well

- `Member` as one `u32` by offset (P2, 3.4) carried every corpus item without changing `ax[x,y]`,
  `Node` (16 bytes: `Exists(Member, FoTermId, NodeId)`) or the session's `sequent` key; ground
  first-order needs no `Member` table at all.
- The staging of 10.10 (data model with refusals first, one layer per commit), the
  `#[non_exhaustive]` set (`Fault`, `Refusal`, `Limits`, `Step`, `Needs`, `Reason`, `Disproof`'s
  private fields) and the level-per-content rule made every other addition additive; the interned
  atom is right for ground first-order, as long as item 1 fixes the names.
