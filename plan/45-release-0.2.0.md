# Step 45: the fourth baseline, and the second release, 0.2.0

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step comes after
every step before it (38e to 44) is done, reviewed and on `main`; it
needs the machine for a night, which the author gives. This prompt was
written on 2026-10-23 at the author's word and is finished at the
review of the last step before it. It says what is fixed.

Read `plan/30-baseline-release.md` and its report (how 0.1.0 was made:
the third baseline, the release prepared, the author's acts last),
`plan/notes/distribution.md` (where and how linlog is published, what
each registry asks), `.claude/rules/bench.md`, `ci.md` (the release
workflow, crates.io's Trusted Publishing), `rocq.md` (the Rocq library's
package), `CHANGELOG.md` and the reports of steps 39 to 44.

## Why 0.2.0

Since 0.1.0 a public item changed only by addition, with one exception
planned from the start (core.md, "Decisions"): a new variant of the
calculus's closed enums is "a planned 0.y bump", and steps 41 (cut) and
42 (the quantifiers) add such variants (`Rule`, `Node`, `Term`, `Kind`
and their kin), as step 40 may in its typed surface. So the release is
0.2.0, and it is where the API may break for the better, once, said
plainly.

## How the step runs

Two sessions on Opus 5.5 at `high`, each with its goal in its launch
prompt and reviewed by the planning session before the next.

- **45a, the fourth baseline**, comparable with the third
  (`bench/results/2026-10-17/`; `bench/COMPARISON.md` gains a column),
  by `.claude/rules/bench.md` and the lessons of steps 14, 16 and 30: the
  estimate by arithmetic and a word to the supervisor before the start,
  the night the author's go, detached (`bench/baseline.sh --detach`),
  stopped by the author's word through the graceful-stop skill and
  resumed; the target sets against their latest labels; no probes
  beyond those named. A second night of the comparison with other
  tools (step 29's) is the author's to ask for; without it, README's
  "Beside the other provers" says the comparison is of 0.1.0's engines.
- **45b, the release prepared**, so that the author's acts (the tag, and
  through it the release workflow's publication to crates.io, the GitHub
  release and Zenodo's new version of the DOI) are the last and smallest
  part:
  - **The break, accounted for**: every change since 0.1.0 that breaks a
    user of the library found by a tool, not by memory (cargo-semver-
    checks or what the session finds fit, adopted through the `new-tool`
    skill), and listed in the changelog's `## [0.2.0]` with what to do
    instead; the lists of marked and closed public types in `core.md`
    true; no deprecated item kept without a reason.
  - **The versions**: 0.2.0 in the manifests, `CITATION.cff` (its
    version and date; the DOI's concept stays), the Rocq library's opam
    file, the bindings' package, the changelog dated; a dry run of
    publishing both crates (`cargo publish --dry-run`) and the
    `package` check; what docs.rs builds.
  - **The faces**: README as a released tool's (what 0.2.0 adds: the
    two-sided surface, MELL nets, cut and its elimination, first-order
    logic, the logo); the guide for AI models (step 44) for 0.2.0, its
    tests passing; the web client's linlog input on the release's
    commit, after the author's tag, by the planning session.
  - **The author's reminders**, collected by the planning session from
    `plan/later.md` and the reports, in the final message: what is
    theirs to send, set or decide before or after the tag.

## Deliverables

Thematic jj commits; `bench/results/DAY/`;
`plan/reports/45-release-0.2.0.md`. Pushing, tagging and publishing
are the author's.
