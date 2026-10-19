# Step 40: proof nets drawn with their formulas

You are working in the linlog repository. CLAUDE.md applies throughout: jj
only (never git), thematic commits as soon as a unit is done, doc comments on
every item, the verification table, no pushing. The step takes one
session; this prompt was written on 2026-10-19 at the author's word. It
says what is fixed.

Read `plan/README.md` (D12, D15, D16), `plan/reports/11-svg.md` (the net's
layout), `plan/reports/35-mll-engines.md` (session 35b: the essential
net's drawing), `plan/reports/36-lambek.md` (the planar and Lambek
drawings), `.claude/rules/core-export.md`, and `core/src/export/svg/net.rs`.

## Goal

A proof net drawn by linlog shows its formulas, as the literature draws
nets. Today the SVG net draws the literals at the top, every `⊗` and `⅋`
as a circled connective, and the conclusion edges ending blank at the
bottom: neither the sequent's formulas nor any compound subformula is
written anywhere (the author, 2026-10-19: "the original formulas in the
sequent are not displayed in the net"). README's "the conclusions at the
bottom" says more than the drawing does.

## What is fixed

1. **Every formula occurrence written, by default** (the author,
   2026-10-19, on the planning session's recommendation): the
   presentation of Girard's 1987 paper and of the lecture notes that
   follow it, where every subformula occurrence of the conclusions is a
   node written as its formula, an axiom link joins `A` and `A⊥` over
   the top, a `⊗` or `⅋` link joins its two premises to the conclusion
   written below them with the connective small at the joint, and the
   sequent's formulas close the drawing at the bottom in the sequent's
   order. The layout stays D12's: the formula trees drawn downwards from
   the conclusions, the axiom links as arcs, deterministic, no general
   graph tool, widths from the Euler Math advance table, `textLength` on
   every text run.
2. **A compact presentation as an option** (D15, D16): the conclusions
   written at the bottom and the inner links as today's circled
   connectives, for nets too wide to label every node. One field of the
   SVG options value with serde, a `--style` key in the command, a line
   of documentation; the default is item 1's.
3. **Every net drawing at once**: the classical net, the essential net
   (`svg::Style::essential`, its direction, polarities and witness kept
   legible beside the labels), the planar nets of the cyclic mode and of
   the Lambek calculus, with and without Mix; where `--format svg --net`,
   `seq print` or the web bindings' `draw_net` draws a net, it is this
   drawing. Ids for click targets (`o<n>`, `l<m>-<n>`) stay as they are,
   so the web page's buttons over the drawing keep working; a formula's
   node gets the id its occurrence has.
4. **The snapshots** change in a commit of their own that says why
   (`core/tests/snapshots/net.svg`, `essential*.svg`, `planar.svg`,
   `lambek-net.svg`), each looked at as an image, not only diffed (render
   it with `rsvg-convert` or the PNG export and read the picture); the
   flake's `export` check renders them. README's net paragraph and its
   examples say what the drawing shows, and the changelog's
   `[Unreleased]` has the line.
5. **The web client** is the planning session's: once this is on
   `main`, it moves the client's pinned linlog input and checks the
   page's net view. Name in the report anything the page must change
   (a new option it should expose in Settings, a wider default view).

## How the step runs

One session on Opus 5.5 at `high`. No panel: nothing a search or a
criterion decides changes. A fresh-context reviewer on Opus 5.5 at
`high` judges the drawings against item 1 on the rendered images of
small nets (one per drawing of item 3), briefed with conduct.md's rules
on cores and memory and told to report only what breaks a stated
requirement. Builds and tests on the cores the brief names, in capped
scopes; `nix flake check` under `/tmp/linlog-machine.lock`.

Deliverable: thematic jj commits; `plan/reports/40-net-drawings.md`.
