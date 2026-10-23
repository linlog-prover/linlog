# Step 46: research, what to add after 0.2.0

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), no pushing. This step writes no code. It comes after
the release of 0.2.0 (step 45), so that it looks at what was released.
It has two parts with the author between them, as step 17's had: the
research, which ends in a ranked list of candidates and the questions
only the author can answer; and, once the author has chosen, the
prompts of the steps that follow. This prompt was written on 2026-10-23
at the author's word. It says what is fixed.

Read `plan/17-assessment.md` and its report (how the plan from step 18
on was made), `plan/later.md` (every candidate and follow-up still
unassigned), the scope notes of `plan/notes/research/scope/`
(applications, education, engines, interop, logics, proof theory,
semantics and decision, higher types), README (what linlog promises),
the reports of steps 38 to 45, `bench/TOOLS.md` and the comparison's
gap study (`plan/notes/research/comparison-gaps.md`).

## Goal

The candidates for linlog's next year, each with what it brings and to
whom (research, teaching, the AI models step 44 wrote for), what it
costs, what it risks and what it needs first, ranked; and the steps the
author then chooses, with their prompts.

## Fixed now (the author, 2026-10-23)

- **Read in sources, never from memory**: research sub-agents (Sonnet
  5.5 at `medium`), one per area, each reading the literature, the other
  provers and tools, and the standards it names, with links, and each
  briefed with what linlog has today (README and the reports), so that
  nothing it proposes is built already. The areas are at least the
  scope notes' eight, updated for what was built since they were
  written, and these beside them: the web page as students and teachers
  use it, the interfaces of other tools (proof assistants, TPTP and
  ILTP, editors), what models and agents need of linlog beyond step 44's
  guide (an MCP server, if step 44 left it to the author), and
  maintenance (dependencies, toolchains, wasm, accessibility).
- **Correctness verification, and its feasibility** (the author,
  2026-10-23), an area of its own. What linlog trusts today: the Rust
  checker and the checks of refutations (`Disproof::check`), the
  intuitionistic reading (and step 40's recorded sides), the parser,
  tested by differential tests, mutation runs and fuzzing; what is
  verified: the Rocq library's checkers with their soundness theorems,
  which the exported certificates of classical proofs rest on, and the
  invariants behind some refutations. The research weighs, with what
  each would cost here and what it would prove: verifying the Rust
  checker itself (Verus, Creusot, Prusti, Kani's bounded model
  checking, Aeneas or hax translating Rust to Lean, Rocq or F*), proving
  it refines the Rocq checker, running a checker extracted from Rocq
  beside or instead of it (in the command, in the browser's wasm),
  certificates for more verdicts (intuitionistic proofs, every kind of
  refutation) checked by the library, and which part of the trusted base
  each would remove. It says what is feasible in a step or two and what
  would be a project of its own, and ranks it with the rest. The
  author's preference among the tools (2026-10-23): where verifying the
  Rust code is feasible and Prusti is as good a fit as the others,
  Prusti; another tool only where it is clearly better suited, with the
  reasons said. The comparison is made on linlog's own
  code (the checker's loops, its arena indices and integers, the
  features of Rust it uses), each tool's state and maintenance read in
  its sources.
- **What users say, from what is public**: the repositories' issues and
  discussions, citations of the Zenodo DOI, crates.io's download counts,
  read through the public web and GitHub's public API (no `gh` task,
  which asks the author first), and nothing else, since the page tracks
  no one.
- **The synthesis** on Fable 5.1 at `high`, the plan's model for a
  session whose decisions every later one builds on: the research notes
  under `plan/notes/research/next/`, one per area, and the ranked list
  in the report, every candidate with its sources; the decisions for
  the author in one message through the planning session, each with a
  recommendation, as the author asked on 2026-10-23 ("concise plain
  language, a recommendation each, and ask me to decide").
- **Then the prompts**, after the author's answers: the chosen steps,
  numbered from 47, in the plan's table with their models, sessions and
  dependencies, and `plan/later.md` true (each candidate assigned, kept
  for later with the reason, or dropped with it).

## How the step runs

Two sessions with the author between them: the research and its
synthesis (Fable 5.1 at `high`, the sub-agents on Sonnet 5.5 at
`medium`), then the prompts (Opus 5.5 at `high`). No code, no benchmark
run.

Deliverables: `plan/notes/research/next/`, `plan/reports/46-research-next.md`,
the prompts of the chosen steps.
