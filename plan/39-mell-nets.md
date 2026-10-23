# Step 39: MELL proof nets with exponential boxes

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes two
sessions; this prompt was finished on 2026-10-19, after steps 34 to 37. It says what is
fixed.

Read `plan/later.md` ("MELL proof nets with exponential boxes"),
`plan/reports/05-proof-nets.md`, `11-svg.md`, `17-assessment.md` (3.4),
and `plan/README.md` (D6a, D12, D15).

## Goal

Proof nets for MELL as a representation: `!`-boxes and the `?` nodes
over the occurrence forest, a correctness criterion independent of any
engine (Danos–Regnier at each box depth with boxes contracted; Guerrini
and Masini, TCS 254, 2001, for the parsing view), sequentialization
through boxes, desequentialization of a proof, a JSON form, and the
drawing with boxes as rectangles, in the command and the web front end. The drawing is step 36's: every formula occurrence written, the
conclusions at the bottom, the compact drawing an option; a box
encloses the nodes of its contents and its `!` and `?` doors.
Search stays with the focused engine. The second session also mends an
older fault of the compact drawing (the author, 2026-10-23): a net's
edges begin a tenth of an em below each literal, inside every font's
descenders, Euler Math's included; they begin at each font's own depth,
as the drawing with formulas does since step 38b.

## Fixed now

First session the structure, the criterion and both conversions, with a
fresh-context reviewer comparing the criterion with switching
enumeration on small structures, as step 5 had; second session the
drawing and the front ends. The choice of `?` nodes (dereliction,
contraction and weakening as nodes, or one generalised node) is the
session's, argued from what makes desequentialization canonical and
weakening checkable. Opus 5.5 at `high`; every change of a search or a
criterion is reviewed by the panel step 26's prompt describes, its
argument read on Fable 5.1 at `high`.

## What steps 31 to 37 left you

- **The net module** (`core/src/nets/`, `.claude/rules/core-nets.md`):
  `ProofStructure` with `is_correct`, `is_essential` and `is_planar`,
  each independent of any search with a witness of failure; the search
  loop generic over its criterion (`Linker<C: Correctness>`: the
  classical `Switching`, `Essential` for IMLL, and `Planar<C>` wrapping
  either for the ordered modes); three prunes of step 35a's, each with
  its argument, off where they do not carry over. A box is a new kind
  of vertex and a criterion that contracts boxes: say where it plugs
  into `Correctness` or why it stands apart.
- **The drawing** (step 36): every formula occurrence written, the
  links between them, the conclusions at the bottom, `svg.net=compact`
  the old circles; ids `o<n>` on each occurrence's formula and
  `l<m>-<n>` on links, which the web page's buttons use. A box is a
  rectangle around its contents with its `!` door and `?` doors on its
  border, in both drawings, its snapshot rendered and looked at.
- **The ordered modes** (cyclic MLL and the Lambek calculus, step 35)
  are unit-free MLL and refuse exponentials, so boxes do not reach
  them; the essential net is IMLL's, and an intuitionistic MELL net is
  left out unless it comes for free, said so.
- **The web front end** draws a finished proof's net through the
  bindings' `draw_net` (`web/`, `plan/web/README.md`), greyed with its
  reason where no net exists; a MELL net makes that tab live for MELL,
  and the client's repository (`/home/tux/Projects/own/linlog-prover.github.io`)
  takes it by moving its linlog input, a change the planning session
  makes after this step.
- **The wire forms** are at level 3 since step 37 (a saturation
  refutation); a net document that holds boxes raises the level only as
  `wire.rs` says.

## How the step runs

Two sessions on Opus 5.5 at `high`, each with its goal in its launch
prompt: the first the structure, the criterion and both conversions;
the second the drawing, the JSON form and the command; the panel of
step 26's prompt for the criterion and for sequentialization, its
argument on Fable 5.1 at `high`, the counterexamples and the integers
on Opus 5.5 and Sonnet 5.5 at `high`, each on a frozen tree. The net
target set (`bench/net-targets.sh` against
`bench/targets/net-after-cyclic.csv`) keeps every counter of MLL.

## The machine

Session 38e runs beside the first session, in the web client's
repository alone. This step's timed runs (`bench/net-targets.sh`, which
pins core 2) on cores 2 and 3; its builds, tests, the gate and the
panel's runs on cores 4 to 7 in capped scopes (the gate as CLAUDE.md
gives it, with `taskset -c 4-7`); 38e's on cores 8 to 11. `nix flake
check`, `nix build` and every timed run under `flock
/tmp/linlog-machine.lock`, since the Nix daemon builds on any core.

Deliverable: thematic jj commits; `plan/reports/39-mell-nets.md`.

## Session 39c: intuitionistic MELL nets (the author, 2026-10-23)

Session 39a left intuitionistic MELL out (`NetError::NotMell`), as the
plan allowed; the author asked for it. One session, Opus 5.5 at `high`,
after 38i and before step 40 (both touch the net module and the
drawing), linlog first and then the page, as 38h runs:

1. **Research first**: how the literature gives intuitionistic MELL
   proof nets (Lamarche's essential nets and how they take the
   exponentials, polarized nets, the IMELL nets built on Guerrini and
   Masini's boxes, or what else exists), each source read, never from
   memory, and the choice argued in the report: a criterion
   independent of any engine, with a witness of failure, that accepts
   exactly the nets of IMELL proofs in linlog's intuitionistic and
   affine-intuitionistic modes where it applies, said precisely where it
   does not. If no sound criterion can be had, say why in the report
   and in the message to planning, and build nothing.
2. **The structure and its conversions**: `MellStructure` or the
   essential net (`Essential`, `core-nets.md`) extended to boxes in
   intuitionistic mode, whichever the research supports; the criterion,
   sequentialization into a term the checker accepts in the sequent's
   mode, desequentialization; the panel of step 26's prompt for the
   criterion and for sequentialization (the argument on Fable 5.1 at
   `high`, the counterexamples on Opus 5.5 at `high`, the integers on
   Sonnet 5.5 at `high`), each on a frozen tree; the net target set
   against `bench/targets/net-after-39b.csv` unchanged.
3. **Drawn, in JSON, in the command and on the page**: the essential
   net's drawing with boxes in both drawings and 38i's options; the
   JSON form; `prove --net` and `check --net` in intuitionistic mode;
   the bindings, and the page's net tab live for a finished
   intuitionistic MELL proof after the client's lock moves.

The report as a section of `plan/reports/39-mell-nets.md`.
