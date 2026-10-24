# Step 40: the two-sided surface, intuitionistic and classical

## Session 40a: the intuitionistic surface

### The research

A research sub-agent (Sonnet 5.5 at `medium`) read the sources below on
2026-10-24 and reported what they say with links; it marked what it read
([S]) apart from what it inferred ([I]). What follows keeps that
distinction. Where it could not open a source it said so, and nothing is
claimed from those.

**What exists.**

- **Yalla** (Olivier Laurent's Coq library) keeps intuitionistic linear
  logic as an inductive of its own, `iformula`, and lowers it by a
  function, `ill2ll`, which sends `ilmap A B` to `parr (dual (ill2ll A))
  (ill2ll B)` and the units to their classical namesakes; the embedding
  theorem `ill_to_ll` states that an ILL proof of `l ⊢ C` gives an LL
  proof of `ill2ll C :: rev (map dual (map ill2ll l))` [S:
  <https://raw.githubusercontent.com/olaure01/yalla/master/yalla/ill_vs_ll.v>,
  the file's README
  <https://github.com/olaure01/yalla/blob/master/yalla/README.md>]. The
  sides are never recovered from the one-sided image: the typed surface
  is the source of truth and the lowering a function of it. Conservativity
  is a theorem only under side conditions there (`nonzerospos`, no `0` in
  the consequent of an implication), and `counter_ex_ill` is a
  counterexample without them [S, same file].
- **Click & coLLecT** (Etienne Callies and Olivier Laurent, TLLA 2021) is
  one-sided; its wiki lists a two-sided sequent calculus and ILL only as
  future directions [S: <https://github.com/etiennecallies/click-and-collect/wiki>].
  Its paper on HAL could not be opened.
- **llprover** (Tamura) and the MLL prover built on it are natively
  two-sided, with `->` a primitive connective and a left rule `L->`, and
  keep no one-sided core [S:
  <https://www.inf.unibz.it/~ntroquard/SOFTWARES/MLLPROVER/mllprover.html>;
  Tamura's own site did not resolve]. **APLL** reads two-sided sequents
  of LL and, with `-ill`, ILL, and emits Yalla certificates [S:
  <https://github.com/wujuihsuan2016/LL_prover>]; its README does not say
  how it represents them inside.
- **The literature.** Laurent, *Around classical and intuitionistic linear
  logics* (LICS 2018) [S: <https://perso.ens-lyon.fr/olivier.laurent/acaill.pdf>]:
  an ILL formula is an LL formula by unfolding `A ⊸ B` into `A⊥ ⅋ B`, an
  ILL sequent `Γ ⊢ A` becomes `⊢ Γ⊥, A`; following Schellinx (1991),
  conservativity of LL over ILL fails when `⊸` and `0` occur together and
  holds without either. The LL Handbook draft [S:
  <https://ll-handbook.pages.math.cnrs.fr/book/ll-handbook-draft.pdf>,
  §3.7.2.1] defines ILL as the two-sided system with one formula right of
  `⊢`, without `_⊥`, `⅋`, `⊥` and `?`, which is exactly the set of symbols
  the strict parser refuses, and gives an unprovable ILL formula whose LL
  image is provable. Its §3.3 (Exercise 3.3.11, Theorem 3.3.12) states the
  equivalence of two-sided LL with explicit negation rules and the
  one-sided calculus at the level of provability [S]. Lamarche's,
  Schellinx's own, Danos, Joinet and Schellinx's and Girard's LU texts
  were not opened.
- **Keeping what was written.** rust-analyzer's lossless trees (rowan)
  store every token, and its book says that memoised red nodes would more
  than double the memory [S:
  <https://rust-analyzer.github.io/book/contributing/syntax.html>]. Lean 4
  recomputes notation from the term by delaborators and unexpanders and
  records nothing in the term [S:
  <https://lean-lang.org/doc/reference/latest/Notations-and-Macros/Extending-Lean___s-Output/>].

**What fits linlog** [I, the sub-agent's and this session's]: inside a
formula of the one-sided image the only writings that collide are the
units (an output `⊤` and an input `0` lower alike, and so do `0` and an
input `⊤`), and between formulas which root is the goal; a written ILL
sequent fixes both, so a side per occurrence recorded at lowering is
exact and costs a byte an occurrence beside the forest's 25. A lossless
tree is far heavier and buys spacing and comments, which nobody asks
for. The recomputing reading stays, for input that arrives one-sided
(a JSON sequent, an LLTP problem, the permissive text).

**Does it change the design?** No. The typed surface as the source of
truth is Yalla's design (`iformula` and `ill2ll`), the strict parser's
refusals are the Handbook's definition of ILL, and recording the sides
at lowering is what every source that has a typed surface does; no
source recomputes them from the image. For the second session the
sources confirm that one notation bit and one side bit per occurrence
suffice and that the two-sided classical view is an insertion of
negation steps where the author wrote one. One caution carries over:
nothing may assume that an LL proof of an ILL sequent's image makes the
sequent ILL-provable when `⊸` and `0` occur together, which the
checker's one-succedent condition already accounts for.

### Outcome

An intuitionistic sequent now means what its author wrote wherever
linlog shows it. `IllSequent` (`core/src/occurrences/ill.rs`, a new
public type) is the two-sided sequent of ILL: its hypotheses in the
order written and exactly one goal. It is built by a strict parser
(`IllSequent::parse_within` and `FromStr`) that refuses `~`, `^`, `⅋`,
`par`, `|`, `?`, `⊥` and `bot` with a hint naming what to write instead
(`ParseError::hint`, a new field), by its own JSON form (wire level 5,
`serialize/ill.rs`), and by `IllSequent::read_one_sided`, the explicit
intuitionistic reading of a one-sided sequent. Building it lowers it once
to the forest and records every occurrence's side; `IllSequent::reading`
is the `Reading` over those sides, which a test pins equal to
`Reading::new`'s on every lowered sequent. Every intuitionistic output is
raised: the refutations' phrases, the checker's reports through
`describe`, `SequentMismatch`, `Interactive::formula`,
`Derivation::formula`, `Proof::formula`, the nets' text and drawings, the
command's lines and the bindings' texts; three property tests (library,
command, bindings) assert that none writes a classical symbol, and two
property tests pin the round trip. The command reads intuitionistic text
strictly by default; `--ill-syntax one-sided` keeps the old reading.

### What was built

- **The type** (`IllSequent`): the lowered `Forest`, a `Side` per
  occurrence and the goal. `lowered` (crate-private) takes a sequent
  whose sides were written and whose formulas are ILL as written, checks
  exactly one formula right of `⊢` (`ShapeError::Succedents` otherwise),
  builds the forest within the limits and passes the sides down from the
  roots, flipping at the left factor of a `⅋` on the goal's side or a
  `⊗` on the hypotheses' side. A `debug_assert` compares the result with
  `Reading::new`. `read_one_sided` is `Reading::new` with its positions
  kept (`Reading::into_parts`, no longer behind the feature
  `interactive`). `reading()` lends `Reading::of_parts` in constant time;
  `Display` prints `Γ ⊢ A` through it; equality is occurrence by
  occurrence (connective, atom name, side), so the parsed sequent and its
  JSON, whose arenas share differently, are equal.
- **The strict parser**: the commutative parser, untouched, then
  `parse::classical`, a scan of the text for the classical tokens. None
  of them is ILL syntax anywhere, so the scan needs no parser state; a
  parse error before the first classical token is reported first. The
  command prints the hint under the caret and names `--ill-syntax
  one-sided`.
- **The JSON form**: `{"version": 5, "terms": […], "atoms": […],
  "hypotheses": […], "goal": n}` with the tags `V`, `1`, `0`, `⊤`, `⊗`,
  `⊸`, `&`, `⊕`, `!`; a classical tag is an unknown variant. It is
  written from the reading in the written order, and read by lowering
  each term once per side it is needed on, the occurrences counted
  against the limits before the forest is built, an arena that could pass
  a `u32` of lowered terms refused. `core/tests/wire/level-5.txt` and
  `the_fifth_level_reads` keep it readable; `wire::LEVEL` is 5.
- **The options** (D15, D16): `InputOptions` with `intuitionistic: IllSyntax`
  (`strict`, the default `InputOptions::DEFAULT_INTUITIONISTIC`, or
  `one-sided`), in `Settings::input`, JSON `{"input": {"intuitionistic":
  "one-sided"}}`. The command maps `--ill-syntax` onto it (on `prove`,
  `interact` and every `seq` command, a flag of the sequent input), the
  bindings read it from the page's settings, and a wrapper sets the field.
- **The command**: `io::sequent_in` takes the mode and the options; text
  of a commutative intuitionistic mode, single or a batch's entry, goes
  through the strict parser unless `one-sided` is asked; JSON may be
  either form (told by the `hypotheses` key); `seq json -i` writes the
  intuitionistic form. LLTP and `.spec` problems are read as before.
- **The bindings**: `start` and `prove` read a typed sequent of an
  intuitionistic mode by the settings; `read_as(input, mode, settings)` is
  `read` for a mode, its `text` two-sided; a goal formula's `text` is the
  reading's in intuitionistic mode, as `reading` is.
- **Raising**: `Described` takes a mode from its owner (the sealed
  `Owner::mode`: a proof's recorded mode, a two-sided derivation, a
  session) or from `Described::in_mode` (new), builds the reading and
  passes it to every writer through `Names`; the checker's report writes a
  derived sequent `Θ ; Γ ⊢ C` and names rules two-sided; `Disproof`'s
  phrase reads its mode (an atom occurring positively more often than
  negatively, the count equation over `⊗` and `⊸` on either side, the
  classical reading with `⊸` as implication); `SequentMismatch` writes
  its formulas under the proof's mode; `ProofStructure::written_in` and
  `MellStructure::written_in` (new) write a net's text under the reading;
  an essential net's SVG title and description are two-sided, and the
  command and the bindings draw an intuitionistic net as its essential
  net.

### The round trip and the property tests

- `a_parsed_sequent_prints_back_to_itself`: 240 generated provable
  sequents (`search::generate::ill`, every rule set), parsed strictly,
  printed, parsed again: equal, sides included.
- `every_inference_prints_as_an_intuitionistic_sequent`: the proofs of 72
  generated sequents, every inference's sequent printed two-sided
  (`TEXT.sequent` under the reading), parsed strictly, its hypotheses and
  goal the same formulas as the inference's; more than 200 inferences.
- `the_recorded_sides_are_the_readings`: the generated sequents and five
  with `⊤`, `0` and `1` on either side and under implications.
- `no_intuitionistic_output_writes_a_classical_symbol` (library): the
  sequent, a session's goal formulas, the text tree, LaTeX, Typst, SVG
  and Rocq of each proof, its formulas, its net's text and essential
  drawing, a refused proof's report, a refutation's phrase and
  certificate, over 72 generated sequents and their mutants. A probe that
  fed it the one-sided print made it fail, so it can.
- The command's test of the same name runs every format, `--net` as text
  and SVG, `seq print -i` in four formats and a session over 24 random
  sequents; the bindings' `no_intuitionistic_text_writes_a_classical_symbol`
  runs `read_as`, the goal formulas before and after `close_all`, every
  export and the net over 16.

### Decisions

- **The permissive input (D16): refused with a hint by default, kept
  behind an explicit option.** A newcomer who types `~(A * ~B), A |- B`
  in intuitionistic mode meant a classical sequent or made a slip; the
  refusal says what to write (`A -o B, A |- B`) and names the flag.
  `--ill-syntax one-sided` (`input.intuitionistic: "one-sided"`) reads
  the classical syntax as before, through the explicit reading, for
  users with files in it. README ("`--intuitionistic`"), the changelog
  ("Changed") and the command's help (`--ill-syntax`, `-i`) say so.
- **The strict parser is the parser plus a scan**, not a parser of its
  own: the classical tokens are classical wherever they stand, so a scan
  finds them without the parser's state, and the parser, its journey
  (`read-text`) and its fuzzing stay as they were; the brief also asks
  this session to leave the classical parser alone.
- **The typed formula is the reading's view.** `IllSequent` is the typed
  sequent; its formulas are `IllFormula` values (`Reading::formula`)
  whose connective is an `IllConnective` (ILL's connectives only, a
  non-exhaustive enum with room for `∀` and `∃`), and on the wire the
  arena of `serialize/ill.rs`. No tree of ILL terms is kept beside the
  forest: the brief asks for no second representation where the reading
  serves, and the recorded sides are what the written form adds.
- **A JSON one-sided sequent stays accepted in intuitionistic mode**, by
  the explicit reading: it is what `prove --format json` and `check`
  exchange, and it was always one-sided. The intuitionistic form is
  accepted beside it, told by its `hypotheses` key.
- **LLTP and `.spec` problems stay one-sided readers**: their files are
  another syntax with their own sides, read by `Reading::new` as before;
  the target sets read the LLTP library and must not move.
- **An intuitionistic net is drawn as its essential net** in the command
  and the bindings, whatever `svg.essential` says there: the plain
  drawing labels the one-sided formulas. The key still chooses for a
  classical sequent that has a reading, and the library's `svg::net`
  keeps its contract (a structure records no mode).
- **The outputs that stay one-sided are data or explicit choices**: the
  JSON documents' arenas (sequent, proof, outcome, session, structure),
  which `check` and the page read back; `Sequent`'s and the structures'
  plain `Display` (no mode); a Rocq certificate under `--kernel
  nanoyalla` (`ll`, one-sided by the kernel's choice; the default kernel
  writes `ill`); and a `ShapeError`'s report, which names a formula that
  has no intuitionistic reading.

### Departures from the design

- **The engines and the checker read `Reading::new`, not the recorded
  sides (departs from the design).** "The recorded sides and the Reading
  … are one thing" is kept as an equality, not a single value: the
  search's front door, the checker and `Interactive` take a `Sequent` or
  a `Proof` (public, released), so they compute `Reading::new` on the
  lowered forest, and `the_recorded_sides_are_the_readings` with the
  `debug_assert` in `lowered` pin that it is the recorded reading.
  Handing them the recorded reading would change the search's public
  entry points, which 0.1.0 forbids but by addition, for no change of
  any answer.

### Options, front ends and quantifiers (D15, D16, D17)

- One options value, `InputOptions { intuitionistic: IllSyntax }` in
  `Settings::input` (serde, `deny_unknown_fields`, the default a named
  constant): the command maps `--ill-syntax` onto it, the bindings read
  it from the page's settings JSON, a wrapper sets the field.
- Quantifiers: `IllConnective` and the JSON form's tags gain `∀` and `∃`;
  `parse::classical` needs nothing (the reserved words are refused by the
  parser already); `lowered`'s pass gives a quantifier's body its side.

### What steps 41 and 42 must know

- **A cut's formulas get their sides where the cut is made** (step 41):
  the user types the cut formula as an ILL formula (parsed strictly, as
  a goal `|- C` would be), and the cut adds two occurrences, `C` as the
  goal of the left premise and `C` as a hypothesis of the right; their
  sides are written, so they are recorded as `lowered` records a root's
  (output for the left, input for the right), never read off the
  one-sided `C` and `C⊥`. The recorded-sides test extends to a cut's
  occurrences.
- **An instance's substitution enters the raising at the atom** (step
  42): `Reading::formula` writes every formula through `TEXT.ill`, whose
  atom arm is the one place an atom is written; an instance's
  substitution maps the atom there, the sides being the instance's
  occurrence's, so the raising needs no second walk.

### The review

One fresh-context reviewer (Opus 5.5 at `high`) judged the recorded
sides against the checker's reading, the strict parser's refusals, the
arithmetic of the JSON reader and the scan, and the front doors, asked to
report only what breaks soundness or a stated requirement. It found three
things; all three held and were fixed before the commit.

1. **`-obot` passed the strict parser** (medium). The scan started an
   identifier at the `o` of an unspaced `-o`, read `obot` and let it
   pass, while the parser read `-o` and then the constant `bot`; so `A
   -obot |- B` was accepted, a debug build hit the `debug_assert` in
   `lowered`, and a release build printed and serialized a sequent (`⊥`
   written as `1`) other than the one the engines would read (the
   engines, which call `Reading::new`, refused it, so no verdict was
   wrong). **Answer**: the scan now cuts tokens where the parser does,
   `|-` and `-o` each one token, and scans the whole text, reporting only
   a token before the parser's error; `A -obot |- B` and `(A -obot)
   -obot |- A` are refused at the `bot`, `A |-obot` is the atom `obot`
   (both in `the_strict_parser_refuses_classical_symbols`).
2. **The command read an intuitionistic JSON document past
   `--occurrence-limit`** (medium). `io::sequent_in` read it with
   unbounded limits and the reading builds the forest, so a 700-byte
   document of 2³⁰ occurrences exhausted a 3 GiB scope where the
   one-sided form is refused at once. **Answer**: it is read within the
   bound, and `an_intuitionistic_json_sequent_is_read_within_the_limit`
   (`cli/tests/cli.rs`) pins the refusal. The bindings read within
   their settings' limits already.
3. **A misplaced turnstile was reported as a `par`** (low): the scan of
   the text cut at the parser's error saw the `|` of a `|-` whose `-` had
   been cut. **Answer**: fixed by the same whole-text scan; `A |- B |- C`
   and `(A |- B)` are the parser's error at the `-`, without a hint.

The reviewer found the rest sound: the recorded sides equal
`Reading::new`'s whenever the input has no classical construct (the same
flip rule, `antecedents` always set, `⊤` and `0` taking either side,
shared subterms and merged atoms per occurrence, H9 and H10 refused);
every `IllSequent` has one goal; the JSON reader's bounds and the scan's
slicing cannot wrap or panic; and every text path of a commutative
intuitionistic mode under `strict` goes through the strict parser.

### Public items added, for step 45

All by addition. New: `IllSequent` (with `parse_within`, `FromStr`,
`read_one_sided`, `sequent`, `forest`, `into_sequent`, `goal`,
`reading`, `Display`, `PartialEq`, serde and `wire::Readable`),
`IllSyntax` (non-exhaustive), `InputOptions` (non-exhaustive,
`DEFAULT_INTUITIONISTIC`, `with_intuitionistic`), all re-exported at the
crate root; `Settings::input` and `Settings::with_input`;
`ParseError::hint` (a field of a non-exhaustive struct);
`Described::in_mode`; `ProofStructure::written_in`,
`MellStructure::written_in`; `wire::LEVEL` is 5. No closed enum gained a
variant. Behaviour that changed on released items, each in the
changelog: `Interactive::formula`, `Derivation::formula` and
`Proof::formula` write under the reading in an intuitionistic mode,
`Disproof`'s `Display` reads its mode, `describe` reads its owner's
mode, `Error::SequentMismatch`'s strings, an essential SVG net's title
and description. In the command: `--ill-syntax`, `seq json -i`, the
strict default. In the bindings: `read_as`, `start` and `prove` reading
by the settings, `text` raised.

### Gating

Each linlog code commit was gated (`gate` on cores 4 to 11, capped): the
first at its own tree (passed, 7 min 54 s); the second and the lock's
blessing together at the tree of both (passed), since the code commit
alone changes what the behaviour lock pins and the lock is blessed in a
commit of its own, as earlier steps do. A gate run between them failed on
two SVG snapshots whose title and description moved as intended
(blessed, `core/tests/snapshots/essential*.svg`, `lambek-net.svg`), and
one ran on a tree the review's fixes then changed; neither counts.

### The target sets

At the commit "Bless the behaviour lock for the intuitionistic surface",
the focused set (`bench/targets.sh ill-surface`, cores 2 and 3) and the
net set (`bench/net-targets.sh ill-surface`, core 2) ran in one detached
unit under the machine lock, both from one release build:
`linlog-bench summary --against bench/targets/deps-foldhash.csv
bench/targets/ill-surface.csv --counters` reports no verdict that
differs and, of the 145 problems both decide alike on one thread, 0 that
differ in `nodes`, `splits`, `memo_hits`, `memo_entries`, `links` or
`tests`; against `bench/targets/net-after-39c.csv` the net set reports
the same of its 64. The searches did not change, as expected: the
lowering of a strict text is the parser's own `Sequent`.
