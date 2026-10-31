# Architecture

Four crates: the library `linlog` in `core/` holds all the logic; the
command line program `linlog` (package `linlog-cli`) in `cli/` is a thin
front end; the benchmark harness `linlog-bench` in `bench/` runs the
library's engines on problem sets; and the bindings `linlog-web` in
`web/` compile the library to WebAssembly for a web page, so the library
uses no clock, and threads only behind its `parallel` feature, which the
bindings leave off (the flake builds them for `wasm32-unknown-unknown`
with every other feature).

The library keeps a sequent as a compact arena of subformulas in negation
normal form, one-sided (`Γ ⊢ Δ` becomes `⊢ Γ^⊥, Δ`); an intuitionistic
sequent is written two-sided, lowered once to the same arena with the
side of every subformula recorded, and read back two-sided through those
sides, with no second representation; a classical sequent's text keeps,
beside the arena, a byte per subformula occurrence saying how it was
written, from which its derivations are drawn two-sided. On top of that it detects the
fragment a sequent lives in and builds the occurrence forest, the
numbering of subformula occurrences that every proof-search engine, proof
checker and proof net works on. A proof is a compact term over those
occurrences, one node per rule instance; an independent checker decides
whether it proves its sequent, and a derivation view unfolds it into the
tree of explicit sequents of the standard sequent calculus, one-sided or
two-sided. A proof net is the same forest with axiom links, checked by its
own criterion and convertible to and from a proof term. Proof search
decides a sequent, or any goal within one, with the engine its fragment
and mode call for, sequent search, net search, the additive recursion or
the search of a Petri net's markings,
and returns a checked proof, that there is none, or why it could not tell.
Interactive proving holds a derivation with open goals over the same
forest, with the same inferences as the derivation view, and turns it back
into a term for the checker once it is finished. The exports write
sequents and derivations, finished or not, as LaTeX and Typst source, one
inference at a time, draw them and proof nets as SVG from layouts of
their own: a tree by subtree widths, a net by its formula trees under the
axiom links, each subformula written where it stands in its conclusion,
and write a finished derivation as a Rocq proof script for
the NanoYalla kernel, tracking the order of each goal's formulas so that
one exchange per `⊗` suffices.
