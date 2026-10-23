# Step 40: the intuitionistic surface, two-sided sequents in and out

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes one session
in linlog and a short part in the web client's repository; this prompt was
written on 2026-10-23 from the author's request and the planning session's
design, which the author accepted. It says what is fixed.

Read `.claude/rules/core-forest.md` ("The intuitionistic reading"),
`core-sequents.md` (the parser, `antecedents`, the JSON forms),
`core-proofs.md` (the one-succedent condition), `core-derivations.md`
(the two-sided view, interactive proving), `cli.md` and `web.md`; the
reports of steps 8 (intuitionistic mode) and 9 (interactive proving), and
the sections 38c and 38d of `plan/reports/38b-zoom-fonts-dismiss.md`;
`plan/README.md` (D15 to D18).

## Goal

An intuitionistic sequent means what its author wrote, wherever linlog
shows it. The public face of intuitionistic linear logic is the two-sided
sequent: hypotheses and exactly one goal, in the syntax of ILL. Inside,
everything stays one-sided as it is: the engines, the checker, the nets
and the proof terms do not change. Every place that writes an
intuitionistic formula or sequent (the library, the command, the
exports, the JSON forms, the bindings, the page) goes through one
raising from the one-sided form, so the one-sided form never leaks, and
a round trip pins it.

## Fixed now (the author, 2026-10-23)

- **The two-sided sequent is the source of truth, the one-sided form
  compiled from it.** A typed surface: an ILL formula with ILL's
  connectives only (`⊗ ⊸ & ⊕ !`, `1 ⊤ 0`, atoms) and an ILL sequent
  (the hypotheses in the order written, exactly one goal). It is built
  by a strict two-sided parser that takes ILL syntax only (a `~`, `⅋`,
  `?`, `⊥` or `^` is a parse error that names what to write instead),
  by the JSON form, and by an explicit reading of a one-sided sequent
  as intuitionistic (today's inference of the sides, refusing what it
  cannot decide). The names and shapes of these types are the session's.
- **Lowering records the sides, raising reads them.** Building the
  sequent lowers it once to the forest the engines take and records
  every occurrence's side from what was written; nothing is guessed for
  two-sided input, `⊤` and `0` included. The recorded sides and the
  `Reading` the checker, the engines and the views read are one thing:
  engine, checker and view keep reading one reading of one forest
  (core-forest.md), and a test pins that the recorded sides equal
  `Reading::new`'s on every lowered sequent, since the checker's
  one-succedent condition rests on them (the audit's H9 and H10 were
  sides guessed wrongly). Raising is `Reading::formula` and
  `Derivation::two_sided`, which exist; a second representation where
  they serve is not built.
- **Every intuitionistic output is raised**: the library's printing of
  an intuitionistic sequent and its `describe`, `Interactive`'s goals
  and formulas, the refutations' explanations (today `⊢ A ⊸ B` is
  refused with "~A occurs 1 more time than A in the one-sided
  sequent"), the command's output, the text tree, LaTeX, Typst, SVG,
  the Rocq comments where they name formulas, the JSON forms where they
  write text, the bindings (step 38d's `reading` field becomes the rule)
  and the page (its fallback, the dual written as `~(…)`, goes).
- **Exactly one succedent** on every public intuitionistic sequent; the
  derivation's inferences have it already by the checker's condition
  and the top-down instantiation of what a `⊤` absorbs.
- **The round trip**: over generated intuitionistic sequents (the
  intuitionistic generator of `search/generate.rs`), printing a parsed
  sequent gives it back (up to spacing and brackets); every inference of
  every proof found prints as an intuitionistic sequent that parses back
  to the same; and no classical-only symbol appears in any
  intuitionistic output of the library, the command, the exports or the
  bindings. A property test each.
- **Additive** (0.1.0 is released): public items change only by
  addition, the classical surface and parser stay as they are. What the
  permissive intuitionistic input of today becomes (`~(A * ~B), A |- B`
  is read as `A ⊸ B, A ⊢ B`) is the session's to decide with D16, a
  sensible default and a way to change it: refused with a hint, or kept
  behind an explicit option; the README, the changelog and the command's
  help say which. Options a user may vary are plain data with serde,
  as D15 asks, which the command maps a flag onto and the bindings carry.
- **The places for what comes** (D17): the ILL formula type keeps the
  place for `∀` and `∃`; the report says where a cut's formulas get
  their sides (step 41) and where an instance's substitution enters the
  raising (step 42).
- **The searches do not change**: the focused target set against its
  latest label (`bench/targets/deps-foldhash.csv`) and the net set
  against `net-deps-foldhash.csv`, every decided row's counters equal
  (`linlog-bench summary --against … --counters`).

## What step 38 left you

- `Reading::formula(o)` is public and writes an occurrence as its
  position makes it (an input `⊗` as `⊸`, `⊥` as `1`, an input `⊤` as
  `0`); `Derivation::two_sided` names the rules `⊸L`, `!R` and so on and
  writes `Γ ⊢ A`; `Interactive` keeps its reading (O(1)); the two-sided
  view, the interactive state and the checker read one `Reading::new`.
- `Sequent::antecedents` holds the written sides of a sequent's roots;
  `the_written_sides_decide` pins the refusals of H9 and H10.
- The bindings' view gives each goal formula `reading` in intuitionistic
  mode (step 38d), which the web client writes in a goal opened in a
  new tab, the split chooser's lists, the rules popover's heading and the
  formula buttons' names, falling back to the dual of the one-sided text.
- The web client (`/home/tux/Projects/own/linlog-prover.github.io`)
  takes linlog by its pinned input, which may name only a commit on
  linlog's `main`: linlog's part is gated and pushed (by the planning
  session, after its review) before the client moves its lock and
  drops its fallback; its examples and practice in intuitionistic mode
  must parse under the strict surface, which a client test pins.

## How the step runs

One session on Opus 5.5 at `high`, its goal in its launch prompt; a
fresh-context reviewer on Opus 5.5 at `high` for the recorded sides
against the checker's reading and for the strict parser's refusals,
told to report only what breaks soundness or a stated requirement. Then
the client's part, after linlog's `main` holds this step's linlog part.

Deliverable: thematic jj commits in both repositories;
`plan/reports/40-ill-surface.md`.
