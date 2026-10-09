# The design's drafts and their judgements

Evidence for `plan/notes/api.md`, written by step 28's design stage
(session `step-28c`, 2026-10-09). Three drafts from three angles, each in
a fresh context (Opus 5.5 at `xhigh`), under the brief
`brief-draft.md`:

- `draft-a.md`: the web client and the wire forms first;
- `draft-b.md`: the proof term, the checker and the Rocq library first;
- `draft-c.md`: new engines, calculi and quantifiers first.

Two judges scored them independently under `brief-judge.md` (whose last
section adds two questions: an atom as an interned atomic formula, and
H9/H10): `judge-opus.md` (Opus 5.5 at `high`: A 56, C 52, B 49) and
`judge-fable.md` (Fable 5.1 at `high`: A 56, B 56, C 54, ranked B, A,
C). Both proposed the same synthesis (A's frame, B's trusted core, C's
engines, later steps and spike), both chose C's rule for the written
sides and the interned atomic formula. `api.md` says where it follows
them and where not. These files are not kept current.

The walk-through (`brief-walk.md`): ten agents (Sonnet 5.5 at `high`),
one per step 29 to 38, each sketched its step's first change against
`api.md` as committed in ab0b27e5 and reported where it had to work
around it (`walk-29.md` … `walk-38.md`); `api.md` section 12 answers each
report. The spike (`brief-spike.md`, Opus 5.5 at `xhigh`) is reported in
`spike-report.md` and answered in `api.md` section 11.
