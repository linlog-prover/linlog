# Walk-through of step 31 (the Rocq library) against `plan/notes/api.md`

Read: the design in full, `plan/31-rocq-library.md`, `research/31-rocq.md`,
`research/refutations.md`, the 39 register entries naming 31, and the present
`oracle.rs`, `proofs/mod.rs`, `export/rocq.rs`, `core-proofs.md`,
`core-forest.md`, `core-export.md`. Section numbers are the design's.
Result: **0 blocking, 9 friction, 4 note**. No type 31 needs is closed; the
frictions are missing placements and three conflicts between design text.

## 1. The first change, as a session would write it

Three commits, in the order the prompt's stages give, then the writer.

**Commit A: `rocq/` and the classical development (Rocq only, plus two Rust tests).**
The mirror, written against 3.7/3.8/9.2 (`M` is the member type; here `nat`):

```coq
Inductive formula := atom (n : nat) | datom (n : nat) | one | bot | top | zero
  | tens (a b : formula) | parr (a b : formula) | wth (a b : formula) | plus (a b : formula)
  | oc (a : formula) | wn (a : formula).                  (* Kind order; Term::Atom/DualAtom *)
Record params := { mix : bool; affine : bool }.           (* 34 adds cut; see W11 *)
Inductive node (M : Type) :=                              (* Node::TAGS order, 13 variants *)
  | n_ax (x y : M) | n_tens (o : M) (l r : nat) | n_par (o : M) (p : nat)
  | n_one (o : M) | n_bot (o : M) (p : nat) | n_with (o : M) (l r : nat)
  | n_plus (o : M) (s : side) (p : nat) | n_top (o : M) | n_bang (o : M) (p : nat)
  | n_quest (o : M) (p : nat) | n_copy (a : M) (p : nat) | n_weaken (o : M) (p : nat)
  | n_mix (l r : nat).
Definition forest_of (fs : list formula) : forest.        (* 3.3's preorder, recomputed *)
Definition check (p : params) (fs : list formula) (ns : list (node nat)) : bool.  (* oracle.rs, rule by rule *)
Theorem check_sound : check p fs ns = true -> ll p fs.    (* ll in Prop, exchange by Permutation *)
```

Rust tests of the same commit: `Node::TAGS` against the constructor list (R118)
and `core/tests/forest.rs`'s `id -> (kind, parent, atom)` table (3.3 item 1),
which the Rocq suite must read as `Example f_i : forest_table [...] = [...] := eq_refl.`

**Commit B: the writer (`core/src/export/rocq.rs`).**

```rust
pub const LIBRARY: &str = "0.1.0";  pub const FORMAT: u32 = 1;
#[non_exhaustive] #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Kernel { #[default] Auto, NanoYalla, Linlog }          // wire: "auto" | "nanoyalla" | "linlog"
#[non_exhaustive] pub struct Options { pub form: Form, pub lemma: Identifier,
                                       pub prelude: Option<String>, pub kernel: Kernel }
#[non_exhaustive] pub enum Unsupported {
    Open, Compact,                                              // kept
    Rule { kernel: Kernel, rule: Rule },                        // Mix, AffineWeakening
    Mode(Mode), Refutation { kernel: Kernel }, Quantifiers,     // 36, 31, 38
}
pub fn write_proof(proof: &Proof, mode: Mode, options: &Options, out: &mut impl fmt::Write,
                   stop: impl FnMut(Progress) -> bool) -> Result<(), Error>;
pub fn write_disproof(d: &Disproof, options: &Options, out: &mut impl fmt::Write,
                      stop: impl FnMut(Progress) -> bool) -> Result<(), Error>;
```

Output for `A, A -o B |- B` (7.3's example), `Kernel::Linlog`, `Form::Standalone`:

```coq
(* linlog 0.1.0, rocq-linlog 0.1.0, certificate format 1, mode classical *)
From Linlog Require Import Certificate.
Check Linlog.Certificate.format_1.
Lemma certificate (A B : formula) : ll (mk_params false false) [dual A; tens A (dual B); B].
Proof. exact (certify_sound (mk_params false false) [A; B]
  [datom 0; tens (atom 0) (datom 1); atom 1]               (* formulas, written order *)
  [n_ax 0 2; n_ax 3 4; n_tens 1 0 1] eq_refl). Qed.         (* nodes, arena order, shared once *)
```

Intuitionistic mode writes `ill (mk_aparams false) [hyps] goal` from `Reading::walk`
(3.6, public) and adds the positions as a list. `write_proof` never calls
`Proof::check` (R199), refuses `Proof::goal().is_some()` with `Error::GoalProof`
(3.7), and polls `stop` per node.

**Commit C: the refutations.** `Refutation::Classical(Assignment)` (3.12),
`Disproof::check` in `refutation.rs`, the first `Refute` row (8.7), then
`write_disproof`. Wire (7.3), a level-2 tag (W7):

```json
{"version": 2, "sequent": {...}, "mode": "classical",
 "refutation": {"kind": "classical", "assignment": [true, false]}}
```

## 2. Workarounds

**W1. `Kernel::Auto` has no function that resolves it. `friction`.**
9.2 gives `write(&Derivation)` (NanoYalla, as today) and `write_proof(&Proof,
Mode)` (Linlog), and says Auto "writes NanoYalla's script ... for a proof in
classical mode without Mix and affine weakening, and the library's term
certificate otherwise". Neither entry can honour Auto: NanoYalla needs a
`Derivation` (built `Compact::Never` under `derivation_bytes`), the library
needs the `Proof`. R139's `Kernel::for_mode` is not in the design. So the CLI,
`interact`, the batch, the harness and the web client each re-derive the rule
(D15 broken), and the wording is both mode-based and proof-based ("without Mix
used" or "mode without Mix"?). R139 also contradicts itself: `ill.v` (an
intuitionistic derivation) must stay byte for byte under `Options::default()`,
while `for_mode` sends intuitionistic to Linlog; and `prove -i --format rocq`
changes output under Auto. *Smallest change:* add to 9.2 `Kernel::resolve(self,
mode, &Proof) -> Kernel` (Auto: NanoYalla iff classical mode and no `Mix`/`wk`
of a non-`?` in the term; everything else Linlog) and one front-door
`rocq::certify(&Proof, Mode, &Options, out, &Limits, stop)` that resolves and
builds the derivation for NanoYalla. Say that `write(&Derivation)` is the
NanoYalla-only entry and refuses `Kernel::Linlog` with `InvalidOption`. Decide
whether `prove -i` keeps NanoYalla (then Auto is classical-only by mode).

**W2. `lemma: Identifier` is checked against one reserved set; R155 makes it per kernel. `friction`.**
6.4 types the lemma "outside the reserved names (F38)" at construction, and a
`Settings` file is read before the kernel is known. A name valid for NanoYalla
may clash with the library's constructors (`check`, `certify_sound`, `ll`) and
the union of all kernels' names would turn a stored lemma invalid when a kernel
is added (a break for a stored `Settings`). *Change:* `Identifier` checks only
the lexical form; the reserved check happens at write time per kernel
(`InvalidOption { key: "rocq.lemma" }`), as 9.2's `identifiers(atoms,
&reserved)` already does for atoms.

**W3. `Node::TAGS` has 14 tags for 13 variants. `friction`.**
3.7 defines `TAGS` as "the wire tags in variant order" and R118 compares it with
the Rocq constructors one to one; `Plus(Member, Side, NodeId)` is one variant
with two tags (`⊕₁`, `⊕₂`, 7.3). The session must choose one `n_plus` with a
`side` or two constructors, and the test cannot be "equal lists". *Change:* 3.7
adds `Node::VARIANTS: &[&str]` (13 names, `plus`) beside `TAGS`, and says the
Rocq test compares `VARIANTS` and, separately, the `Side` constructors.

**W4. No way to write the cross-check corpus and the forest fixture. `friction`.**
R199 needs `Example t_i : check ... = true/false := eq_refl.` for valid, mutant
and rejected terms; 3.3 needs the forest table as a Rocq-readable file. (a)
`Form` is closed (2.5) with `Fragment`/`Standalone` only, so a check-only form is
a 0.y bump; (b) `oracle::proofs()` is `pub(crate)` test-only, so a corpus writer
must live in `core/src`, not `core/tests`; (c) "`core/tests/forest.rs` ...
Rocq's test suite reads the same file" cannot be read: a Rust file is no data.
*Change:* make `Form` `#[non_exhaustive]` now (it is an options value; 2.5's
"closed on purpose" list does not need it) and reserve `Form::Check` ([31]); say
the forest fixture is `core/tests/fixtures/forest.txt` (kind/parent/atom per
line) read by both tests, with the Rocq side generated from it.

**W5. The two-sided statement is not specified where the Rocq function can be written against it. `friction`.**
9.2: positions are certificate data, checked locally ("each position agrees with
its parent's by the grammar; the goal is the written succedent"). (a) 3.8 has a
per-rule table for the classical checker but the position grammar (output: `⊗ ⊕ &
! 1 ⊤ 0`, atoms, `A⊸B` stored `A⊥⅋B`; input: the duals; the flip only at an
implication's antecedent) and the implication's factor rule exist only in
`core-forest.md` and 3.6 prose; R54 asks a "language-neutrally described
algorithm". (b) "The goal is the written succedent" holds only for `antecedents ==
Some(k)`; 3.1 lets `None` exist (JSON of the pre-release form, `Sequent::add`)
and 3.6 keeps today's choice there, including the symmetric reading `b ⅋ ~a` as
`a ⊸ b`, whose lowering `~a ⅋ b` swaps operand order, so the Rocq forest
recomputed from the stated `ill` formulas would not have the written ids. (c)
`Reading::new(&Forest)` can only use `antecedents` if `Forest` stores it; 3.3
and 3.1 ("the forest and the engines ignore it") do not say so. *Change:* 3.6
gets a normative position table and the implication rule, in the style of 3.8;
9.2 says `write_proof` in intuitionistic mode refuses `antecedents == None` with
`Unsupported::SidesUnknown` (or carries the goal member as data and states the
roots by `Permutation`); 3.3 says `Forest` keeps `antecedents`.

**W6. The refuter hook cannot say why it gave up, and has no switch. `friction`.**
8.7: `refute(...) -> Option<Refutation>`. R20/R153 want the Rocq writer to
"refuse with an `Unsupported` that names the bound" and R173 a verdict-line
comment saying none exists; the writer only sees a `Disproof` with `Exhausted`.
`None` conflates "the formula is a classical tautology" (a real case: `|- A ⊕ ~A`
is unprovable and valid), "the bound was hit" and "the refuter was off". Also:
the refuter runs after every `Unprovable` with `Exhausted` and only
`refute_unknown: bool` (6.2) is an option, so there is no way to turn the
first run off for a benchmark, and a bool cannot grow into a choice per refuter
(R72's phase and Kripke models). 5.3 says `Limits::work` bounds "the refuters"
but not whether each starts from zero (after `Unknown(WorkLimit)` the search
has spent it). 5.2 lists no unit of work for the refuter or `Disproof::check`.
*Change:* `Option<Refutation>` becomes `Refuted { Found(Refutation), None,
Gave(Refusal) }` crate-private; `Outcome` (non_exhaustive) records it in
`Statistics` (`refuted: bool`, `refute_work: u64`) so a front end can name the
bound; replace `refute_unknown: bool` by `refute: Refute { Off, Unprovable,
Always }` (`Unprovable` default, `Always` = also after `Unknown`); state that a
refuter's `work` budget is its own and name its unit (one DPLL decision or
propagation).

**W7. The classical refutation's wire form and its level. `friction`.**
(a) `Classical(Assignment)` (3.12) and `{assignment}` (7.3) give no shape: a
`Vec<bool>` over all of `sequent.atoms` (unmentioned atoms false) is the one a
checker can verify without guessing; 7.4's `refutation.kind` row omits
`classical`. (b) 7.1 raises the level for "a new tag" in a form read back, so
`classical` makes `wire::LEVEL` 2 at 31, and 8.7 runs the refuter by default, so
the same sequent's unprovable outcome is `version: 1, "exhausted"` in 0.1.0 and
`version: 2, "classical"` after 31, in a JSON Lines batch the `version` differs
per record. That contradicts P8 ("a propositional, cut-free value writes 1 in
every later release") and 7.5's "nothing else moves a pinned form". (c) At 38 a
quantified goal needs a model, not an assignment: `Classical` stays valid for
ground input, a new variant is additive (fine). *Change:* 7.1/10.3 state that 31
takes level 2, and 8.7 decides: refuter off by default after `Unprovable` (CLI
turns it on for `--format rocq`, R173), or the level rule says a refutation tag
read as an open enumeration does not raise the level (then 7.4 marks it open
and a 0.1.0 reader treats an unknown kind as `exhausted`-like).

**W8. `Disproof::check` has no error type, no code and no applicability predicate. `friction`.**
3.12 returns `Result<(), Error>` but 4.4 has no variant or code for an invalid
disproof (the table has `Check(CheckError)` for proofs only), and P5 says a
specific type stays where a caller matches data: `linlog check` needs to say
"not applicable in this mode", "numbers differ" or "assignment satisfies the
sequent". R70 asked for `Refutation::certifiable(&Sequent, Mode)`; the writer
and the check both need the conditions (exponential-free, no weakening, no
additives for `Equation`), and a third copy in the Rocq lemma exists already.
*Change:* 4.4 adds `Refutation(Box<RefutationFault>)` (code `invalid_refutation`,
kind `invalid`, `Refused` inside as for `CheckError`) with a non_exhaustive
`RefutationFault { NotApplicable, Mismatch, Satisfied, ... }`; 3.12 adds `pub fn
Refutation::applies(&self, &Sequent, goal, Mode) -> bool`, the one function
`check` and `write_disproof` call. For `Disproof::goal() == Some(_)` the
intuitionistic statement needs the session's reading, which a `Disproof` lacks:
`write_disproof` refuses it (`Unsupported::Goal`) unless the mode is classical.

**W9. R154's classical "not valid" is two incompatible things. `friction`.**
The prompt (item 8, third bullet) and R154: `~ (forall a b : Prop, F)` over the
user's ordinary formula, no library, from `export::rocq::ordinary`. 10.3:
"through the same refuter and `write_disproof` over the image", i.e. `~ ll p
[image]` in the library, which proves non-derivability of the image, not
non-validity of `F`. The design places neither the Prop writer (no
`rocq::ordinary_disproof`) nor the assignment in ordinary terms:
`ordinary::Outcome::NotValid` is payload-free (3.13), `Image` interns its own
atoms (3.13) with no map back, and 2.5 does not list `ordinary::Outcome` as
non-exhaustive, so R72's later Kripke model cannot be added without a break.
*Change:* 3.13: `Image::atom_of(ordinary atom) -> Atom` (or the image keeps the
ordinary atom order), `ordinary::Outcome` non_exhaustive with `countermodel:
Option<Countermodel>` (an assignment now, a Kripke model at R72), and 9.2 gets
`rocq::ordinary_disproof(&ordinary::Sequent, &Countermodel, ...)`. Say which
statement 31 owes; the prompt's is the useful one.

**W10. The writers are missing from 5.6 and from the error and mode tables. `note`.**
`write_proof` and `write_disproof` take no `&Limits` (linear in a proof and
forest already held, like `Derivation::write_text`) and 5.6 has no row for them;
it needs "none / `stop` per node (`Phase::Write`) / linear". The statement is
the unfolded forest (R156), as large as `limits.occurrences` allows: say so.

**W11. Certificates are a wire form with no compatibility rule. `note`.**
P8/7.1 govern JSON only. 9.2: 34 adds `params.cut`, a constructor and a new
`FORMAT`, and a library without the format "fails by name". A record literal
`{| mix := ..; affine := .. |}` fails on a library with a third field, so every
stored certificate (the snapshots too) breaks at 34. *Change:* 9.2 states the
.v form follows 7.1's rule: build `params` and `certify` through functions
(`mk_params`, `certify_sound`) whose old arities the library keeps, define
`format_1` forever, and let `FORMAT` count only incompatible changes.

**W12. A new mode must be a compile error in the writer. `note`.**
9.2 says NanoYalla refuses an ordered mode; `write_proof` for Linlog must also,
or a Lambek/cyclic proof gets a commutative `ll` statement (weaker than
claimed, no error). 3.5's private `Mode` fields make that checkable only if
the writer destructures the whole `Mode` as `Modes::take` does: say so in 9.2
and add `Unsupported::Mode(Mode)` for Linlog.

**W13. Smaller placements. `note`.** (a) `Settings`' pinned default JSON (7.3)
gains `"kernel": "auto"`; it is a key added to an options form (6.1: fine) but a
lock commit of 31's. (b) At 38 `Sequent::atom_name(a)` is the predicate symbol
(3.2), so ground atoms `p(a)`, `p(b)` give `identifiers` duplicate names
(it primes them: `p p'`); the writer should name an atom through the crate's one
atom writer, escaped. (c) `Equation.mix` duplicates `Disproof::mode`: `check`
compares them. (d) `ordinary::rocq` (positive and negative) ignores `kernel`:
document it on the field.

## 3. Register entries naming step 31

| entry | status |
|---|---|
| R3 refutations read back, no wildcard | placed 3.12, 7.3 ("derived on the types"), 7.5 |
| R6 compatibility policy | met 7.1; 31's level not numbered (W7) |
| R7 mode with the proof | met 3.7, 7.3 (optional `mode`) |
| R12 `LIBRARY` | placed 9.2 (`LIBRARY`, `FORMAT`, `format_1` check); opam compare is the flake's |
| R20 falsifying assignment in the library | placed 8.7, 10.3; payload, bound field names, why-not not (W6, W7) |
| R243 stop with work | met 5.2; refuter's unit missing (W6) |
| R54 two-sided statement from `Reading` | placed 3.1, 3.6; grammar and `None` not (W5) |
| R57 cut roots, lookup a parameter | placed 3.3, 3.7, 10.6; 31 must state the parameter in the Rocq checker |
| R68 kernel open to a second development | placed 9.2 (`Kernel` non_exhaustive, `M` parameter); per-kernel printer is code |
| R69 shared coordinates | met 3.3 item 1; fixture format not (W4) |
| R70 refutation data and conditions | placed 3.12 (payload, `StateEquation`); public predicate not (W8) |
| R71 assumption of a refutation / sufficient bound | placed 3.12 ("no refutation rests on a bound"); a "bound is sufficient" flag not placed (note) |
| R72 countermodels | placed only as a plug-in remark 8.7, 3.13; value types and the ordinary slot not (W9) |
| R244 member type | met 3.4 |
| R112 failure trace | reserved 8.2 (`Answer.trace`) |
| R113 ordinary decide | placed 3.13; `NotValid` payload-free (W9) |
| R117 term and checker specified, frozen | met 3.8 (table, oracle kept, outside-the-function list); reading half not (W5) |
| R118 variants = constructors | placed 3.7 (`TAGS`); 14 vs 13 (W3) |
| R119 `Cut` node | placed 3.7, 10.6 |
| R124 independent checker | placed 3.12 (`Disproof::check`); error type not (W8) |
| R125 witnesses out of line | placed 3.7, 3.14 |
| R139 kernel option, Auto | placed 9.2; resolution not (W1) |
| R140 prelude per kernel | met 6.4, 9.2 |
| R152 writer from `Proof` and `Mode` | placed 9.2 |
| R153 refutation writer | placed 9.2 (signature elided) |
| R154 classical not valid, ordinary | conflict between 10.3 and the register (W9) |
| R155 per-kernel reserved names | placed 9.2; clashes with `Identifier` (W2) |
| R156 statement printer, one walk | placed 9.2 (`Visit`), 3.6 (`Reading::walk`) |
| R157 open goals as hypotheses | not placed; optional, `Unsupported::Open` stays |
| R158 further target | met 9.1 |
| R159 NanoYalla byte-identical | met 9.2 (but W1 for `ill.v`) |
| R161, R242 first-order refusal and writer | placed 9.2, 10.10 (i) (`Unsupported::Quantifiers` is not in 9.2's enum list; add) |
| R162 opam file | not API (flake and docs) |
| R173 CLI refutation certificate | placed 9.2 + 10.3; the reason line needs W6 |
| R199 cross-check on mutants | not placed (W4) |
| R200, R240, R225 flake, bridges, rules file | not API; nothing in the design blocks them |

## 4. What fits well

- `Member` as every operand (3.4) and `node (M : Type)` let the first
  development stay propositional with `nat` and leave 38 a development beside it;
  `Cut(Member, ...)` last in `TAGS` order keeps old constructor positions.
- 3.8 is the right text to translate: rule table, absorbing flag, the list of
  what lies outside the verified function (`Surplus`, `Refused`), `oracle.rs`
  kept with a new arm per node.
- `Verdict::Unprovable(Box<Disproof>)` with sequent, goal and mode (3.12) is
  exactly what a refutation certificate needs; `Refutation` non_exhaustive and
  the crate-private `Refute` kind make the first refuter additive.
