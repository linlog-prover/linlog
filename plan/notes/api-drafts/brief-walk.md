# Brief: walk one later step through linlog's API design

linlog (`/home/tux/Projects/own/linlog`; read `CLAUDE.md` first) is a
linear-logic proof-search suite in Rust, built step by step by separate
sessions. Step 28 puts the library in order before the first release
(0.1.0, step 30) and has written a design note for its public surface, its
data model and its wire forms:
`/home/tux/Projects/own/linlog/plan/notes/api.md`
(sections 11.5 and 12 are still placeholders: the spike's results and this walk-through). The fixes that implement it come later;
the design is to let every later step (29 to 38) enter without a second
rewrite and without a breaking change after 0.1.0, quantifiers above all,
while the propositional case pays nothing (D17).

You are the walk-through for **one later step**, named in your prompt. Your
job: sketch that step's first change (its first one or two commits) as a
session would write it against the design, and report every place where it
would have to work around the design: a type it cannot extend without a
breaking change, a signature that lacks a parameter it needs, a wire form
that cannot carry its data by an added key, a missing hook, a conflict with
another part of the design, a requirement of the register for that step
that the design does not place.

## What to read

1. The design draft, in full.
2. Your step's prompt, `plan/NN-*.md` (the file whose number is your step's),
   in full, and its research note `plan/notes/research/NN-*.md`; for step
   38 also `plan/notes/research/fo-linear.md` §4 and §5,
   `impact-quantifiers.md` §3, and `corpus-fo-linear.md` (pick three items
   of the corpus and state them in the design's types); for step 33 also
   `mell-nets-spec.md` §7 and `corpus-mell-nets.md` (pick three items).
3. The register's entries for your step:
   `grep -n "^### R\|Steps:\|steps:" plan/notes/requirements.md` and read the
   entries whose steps include yours (each entry names its steps).
4. The present code where the design changes it (`core/src/`), enough to
   know what the step's commit would touch.

## Your report

Write it to the file your prompt names, in Markdown, about 6 to 15 KB:

1. **The first change**, sketched: the types, signatures and wire keys it
   adds or changes, in Rust and JSON, as the step's session would write
   them against the design (not the present code).
2. **Workarounds**: each place the sketch had to work around the design, as
   a numbered item: what the step needs, which part of the design stands in
   the way (section and item), why it matters (a breaking change after
   0.1.0, a silent wrong answer, a cost to the propositional case, a
   rewrite), and the smallest change to the design that removes it. Mark
   each `blocking` (the step cannot be done without a breaking change or a
   rewrite), `friction` (it can, at a cost) or `note`.
3. **Register entries**: each entry naming your step, met or placed by the
   design (with the section) or not (with what is missing).
4. **What fits well**: two or three lines, so the synthesis knows what not
   to change.

Be concrete and brief; a workaround without the design's section and the
step's need is no finding. Do not redesign the step; report where the
design makes it harder than it should be.

## The rules of the machine (binding)

- **You write only your report file.** Never edit, create or delete
  anything in the repository or the design.
- **Never run `git`, and never run `jj`** (a jj command snapshots the
  working copy and tries to sign, which hangs now). Read files directly.
- No builds or runs are needed; if you want one to check a claim, ask in
  your final message instead.
- Nothing outward-facing: no network, no `gh`.

Your final message: in under 150 words, the file, the number of blocking
and friction items, and the most important one.
