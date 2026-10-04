# Step 25: ordinary logic through its embeddings

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. Read before you start:

- `plan/later.md`: "Ordinary logic through its embeddings" (its layer,
  items 1 to 5, is the requirement; its termination work is not this
  step's).
- `plan/reports/17-assessment.md`: 2.2, 3.13, and the author's answer 7.
- `plan/reports/21-defaults.md`, `24-batch.md`.
- `plan/notes/lltp-headers.md`: "The causes" and "Twenty-one further
  files": LLTP's translator reads `~` with the widest scope, so 101 of
  the 362 ILTP and KLE problems are translated as other formulas than
  TPTP means.
- `plan/README.md`: D15, D16, D19.
- `.claude/rules/core.md` (the index, and "The rustdoc is the library's
  manual"), `core-batch.md`, `core-search.md`, `core-export.md`; and
  `cli.md`.

## Goal

A user with a formula of classical, intuitionistic or minimal
propositional logic gets it decided by the linear engines and sees how:
the translation by its name, the image as a linear sequent, the linear
proof, and the proof read back with the rules of LK or LJ. For teaching,
the embeddings are worth seeing by themselves.

## What is fixed now

1. A type and a syntax for ordinary formulas and sequents (`->`, `/\`,
   `\/`, `~`, `<->`, true, false), apart from the linear ones, and
   names that do not collide with `--intuitionistic`, which means ILL.
2. The translations as public functions, each by its name: classical
   logic into affine MALL without exponentials; Girard's translation,
   the call-by-value one and Liang and Miller's 0/1 translation into ILL;
   minimal logic with false as an atom. The image is printable from the
   command.
3. Deciding: classical through affine mode, which terminates without a
   copy bound; intuitionistic and minimal through ILL under step 21's
   deepening default, answering "unknown" where it is unknown. Which
   translation is the default is decided by a run of the ILTP
   propositional problems (274 problems, fetched by the flake at a pinned
   version as LLTP is, not committed: no licence is stated), read in
   their own syntax.
4. The proof read back as LK or LJ with their rule names, drawn by the
   outputs that take a derivation, and what checks the read-back.
5. A certificate over `Prop` for the ordinary statement, which needs no
   library.

## What the earlier steps left you

Step 24 made the command decide many sequents in one call
(`search::batch` in the library: `Problem`s in, results out in order,
one options value; `linlog prove --file DIR`, `--files-from LIST`,
`--input-format`, `--format json` as JSON Lines, a directory for
drawings). The ILTP run goes through it: the ILTP files are one more
input of the batch, chosen by a flag or their extension and never by
their text, read in their own syntax, with the logic and the
translation as options of the batch's problems. Its rules are in
`core-batch.md` and `cli.md`.

The LLTP library's translated files of these problems are not a
substitute for translating the originals here: its translator gave `~`
the widest scope, and the header report lists the problems it misread.
On those, this step's translation and LLTP's differ, and the report
says so where the ILTP table meets one of them; on the others they
should agree up to the translation's name, which is a check worth
running.

What the library's users read is its rustdoc (`core.md`): the ordinary
syntax is documented on its type, as the linear one is on `Sequent`,
every JSON form on its type, and a feature-gated item says which
feature it needs. A new module gets a rules file of its own with its
paths and a row in `core.md`'s table. README's examples run as a test
(`cli/tests/readme.rs`), so the new commands are shown there and
checked.

The names of the API as they stand are kept (step 28 puts them in
order). Termination on dyadic sequents (a loop check, or a bound
proved enough for an image) is engine research: it is assessed in this
step's report from what the ILTP run leaves undecided, and built only
as a step of its own.

## Verification

The checks of CLAUDE.md's table and `nix flake check`; the ILTP run
per translation through the batch, two pinned cores, detached; every
read-back checked as item 4 says, on the run's proofs.

## Deliverables

Thematic jj commits; README's usage for ordinary logic; the rules file
of the new module; `plan/reports/25-ordinary-logic.md` with the ILTP
table per translation and its comparison with LLTP's translated files.
