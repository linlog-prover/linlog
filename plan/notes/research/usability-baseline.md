# Usability baseline: four strangers read the documentation

Written 2026-10-08 for step 28 from the four accounts in
`plan/notes/research/usability/` (`logician.md`, `rust-developer.md`,
`web-developer.md`, `rocq-user.md`). Each user had only README, the
command's `--help` and the published rustdoc; nothing was compiled, and
only the logician and the Rocq user ran the release binary of the code of 2026-10-08. The refactor's
last check repeats the exercise with a fresh reader: section 2 is the
ranking to beat, section 4 the stories to tick off. Counts are tasks, not
users; "(inference)" marks a conclusion of this note's own.

## 1. What each kind of user could and could not do

37 tasks in all: 14 done, 21 done with trouble, 2 stuck.

**The logician (command line; 3 done, 5 with trouble, 1 stuck).** Everything
README shows worked at first try: verdicts and trees for twelve textbook
sequents, every export format, proof-net drawings, LJ and LK proofs,
`interact`, `check` on a saved proof, a Rocq certificate. Trouble came
from what README does not show but rustdoc does: the typeable rule names
in `interact` (`+1`, `-oL`), the `--style` label markup and rule keys,
the binding order. `interact --help` says `show [latex|typst|svg]`, and
`show latex` wrote a file named `latex`. Goal numbers are never reused.
The advice on an `unknown` for `!(A + B) |- !A + !B` led from `--timeout`
to `--recursion-limit` without deciding anything. Stuck: no command
checks a linking a student proposes; `check` rejected `ProofStructure`'s
JSON. In the library, `OccId` has no `Display` and nothing prints a
sequent's occurrence numbering.

**The Rust developer (library; 4 done, 6 with trouble).** Reading the
outcome, serializing and re-checking a proof, driving `Interactive` and
building an ordinary-logic sequent from an AST went through on the
documentation alone. Nearest to stuck: the linear `Sequent` has no
builder, so an AST goes through generated text (renaming atoms that are
not identifiers) or a hand-built one-sided NNF arena, while
`ordinary::Formulas` has exactly that builder. Parsing re-sorts the
roots, so nothing maps a root back to the input. `search::Options` has no
serde; `batch::run` has no example; `png`/`pdf` take font bytes whose
source is never named.

**The web developer (library on wasm32; 3 done, 5 with trouble, 1 stuck).**
Inline parse errors, themed SVG with element ids for clicks, and
proof-net drawing were clean. Stuck: a settings panel kept as JSON cannot
become `Options` (private fields, no serde); `Engine` has no
`Deserialize` or `FromStr`, `Bias` neither. Nothing says which features
build for wasm32 or what stack a recursion level needs; the only
`prove_until` example uses `Instant`, which panics in the browser. Wire
strings are listed as "such as"; `Error` and `Refusal` are `Display`
only; `InfId` has no serde; no JSON form carries a version; `close()`'s
outcome serializes like a proof file that would not check.

**The Rocq user (command and library; 4 done, 5 with trouble).** README's
certificate section is the best-documented path of all four accounts:
kernel, version, no cut, no axiom, both install routes; Prop certificates
need no library. Trouble: nothing shows how to use a lemma from another
file (the statement is in NNF, so a two-sided goal needs
`cbn_sequent`/`bidual`); `--lemma` is not validated and `--prelude` is
ignored silently; batch files are named `modus ponens.v` and every lemma
`certificate`; a refused certificate drops the verdict line;
`ViewOptions::default()` can compact, which `rocq::derivation` refuses;
`Unsupported` and `WriteError` do not convert into `linlog::Error`.

## 2. Frictions that recur, ranked by the tasks they cost

1. **API values with no second form** (8 tasks, all four users). Types a
   front end or script must hold, send or print have `Display` or serde
   but not both, or neither: `search::Options` (no serde, private
   fields), `Engine` (no `FromStr`/`Deserialize`), `Bias` (nothing),
   `Labels` (no `FromStr`, unlike `Rule`), `InfId` and `OccId` (no serde,
   no `Display`), `Refusal` and `Error` (`Display` only),
   `Outcome`/`Verdict` (no `Display`), `Statistics`, `Reason`,
   `Refutation` (not `Serialize` alone). Every user rebuilt a table the
   command already has.
2. **Library defaults are not the command's** (4 tasks, all four users).
   `DEFAULT_COPIES` is 3 with no time limit; the command deepens for 2 s.
   Each user hit or foresaw "unknown: copy bound 3", and found the cure
   (`copies(None)` plus a deadline in `prove_until`) only on the
   constant's own page.
3. **The fact is written, but not where the reader stands** (7 tasks).
   ASCII rule names on `Rule`'s `FromStr` impl, label markup in
   `proofs::style`, binding order in `--help` only, `Compact::Never` for
   Rocq found by laying two pages side by side, `ProofStructure::
   from_proof` far from `svg::net`, `rocq.form=standalone` for `interact`
   guessed from `--style`'s help. README and `--help` do not point into
   rustdoc, and rustdoc pages do not point at the page the next step
   needs.
4. **Errors that name the fault but not the fix or the place** (5 tasks).
   Unknown rule key without the valid names; a rejected rule name without
   a suggestion; `Error` and `Refusal` with no code and no position;
   export errors outside `linlog::Error`, forcing `Box<dyn Error>`.
5. **Input accepted silently when it has no effect or is wrong** (4
   tasks). `--prelude` without `--standalone`, a jsonl `"lemma"` key,
   `--lemma 'my lemma'` and `--lemma Qed` written as given, `show latex`
   taken as a file name.
6. **The derivation ignores the mode** (3 tasks). `Proof::derivation()`
   is one-sided after an intuitionistic search; each library user
   branched on `outcome.mode` by hand.
7. **No way from the user's data into the sequent and back** (3 tasks).
   No linear `Sequent` builder, roots sorted on parse with no map to
   input position, no printed occurrence numbering outside `prove --net`.
8. **How to depend on the crate** (3 tasks). Not on crates.io, in a
   workspace's `core/`; two users guessed a git line, one a version;
   `serde_json` in the examples is the caller's to add, unsaid.
9. **Batch without a per-item limit** (2 tasks): no `run` example, no
   deadline hook, and for Rocq no per-record lemma or valid module name.
10. **The font** (2 tasks): Euler Math is needed for PNG, PDF and the SVG
    page and never located or licensed in the docs.
11. **WebAssembly unsaid** (2 tasks): features that build, stack per
    recursion level, a clock that is not `Instant`.
12. **Single-task frictions worth keeping**: "unknown" advice that reads
    as a cure; the verdict lost when a certificate is refused; `close`
    grafting a `!c`/`!w`/`!c` detour; four `Options` types; `svg::ordinary`
    writing while `svg::derivation` returns; duplicated first sentences on
    six module pages; the `Prf::check` typo; a crate-private trait named
    in public docs.

## 3. What the accounts say about the API and the documentation

**Names.** The names were found good and the collisions bad: `Options`
four times over, `ordinary::Sequent` beside `Sequent`, no `decide` for
ordinary logic beside `prove`. Rule names follow the formula syntax,
which nobody was told.

**Types.** What cost most is asymmetry: one half of a pair exists and the
other does not. `Rule: FromStr` but not `Labels`; `Engine: Serialize`
but not `Deserialize`; `ordinary::Formulas` as builder but no linear one;
`svg::Style` and `ViewOptions` as JSON but not `search::Options`;
`Outcome` written but never read; `derivation` and
`two_sided_derivation` where the mode is already known. Each was found
by a user who expected the other half.

**Errors.** Every error's text was praised; its shape was not. A front
end wants a code and a span, a script wants the verdict before the
refusal, a library caller wants `?` to work. `Reason::Stopped` needs a
neutral phrase: "the time limit was reached" misreads an interrupt.

**Options.** Each setter's "which engines read it" note was called
exemplary. Missing is the relation to the command: nothing states that
`linlog prove` is `copies(None)` plus a two-second deadline plus a pool
after 0.1 s. The command validates less than the library: atoms are
mangled into Rocq identifiers, lemma names are not.

**Outputs.** The per-type JSON sections were the most-cited strength and
their gaps the most-cited weakness: the full set of wire strings, a
version marker, a stability rule, `outcome.net`, a goal proof that looks
like a whole one. The SVG module is the model for the other outputs.

**Documentation.** README is a reliable transcript of the command and
nothing more; rustdoc is complete to the item and weak between items.
Missing outright: how to depend on the crate, how to match the command
from the library, how to use a certificate from another Rocq file. The
rest is placement: pointers from `--help` and README into rustdoc, and
from each rustdoc page to the page its next step needs.

## 4. User stories the API design should satisfy

Each check runs on the documentation or the binary, not the source.

1. *As any library user, I reproduce the command's verdict.* Check: the
   front page or `Options` states the recipe in one paragraph, and
   `!(A -o B), !(B -o C), !(C -o D), !(D -o E), A |- E` is proved by that
   recipe and by `linlog prove` alike.
2. *As a front end, I keep every option as JSON and read it back.* Check:
   `search::Options`, `Engine`, `Bias` and the batch options have a
   documented JSON form and `FromStr` where the command takes a string.
3. *As a front end, I type the whole wire protocol.* Check: one page
   lists every string of `Reason`, `Engine`, `Fragment`, `Mode`,
   `Verdict` and `Refutation`, and states which JSON forms are stable
   and how a version is carried.
4. *As a UI, I react to an error without matching its text.* Check:
   `Error`, `Refusal` and `ParseError` expose a code and a position, and
   `Unsupported`/`WriteError` convert into `linlog::Error`.
5. *As a Rust developer, I build a linear sequent from my own AST and
   map every root of the proof back to my input.* Check: a builder on the
   `Sequent` page with an example, and either written root order kept or
   a documented position map.
6. *As a library user, "the derivation" fits the mode.* Check: one call
   on `Outcome` or `Proof` picks one- or two-sided by mode, and the Rocq
   example says why it never compacts.
7. *As a web developer, I build for wasm32 from the docs alone.* Check:
   a WebAssembly paragraph names the features that build, the bytes per
   recursion level or a wasm-safe limit, and a `prove_until` example with
   a clock that is not `Instant`.
8. *As a logician, I type every rule `rules` shows and know where new
   goals get their numbers.* Check: `rules` or `interact --help` prints
   the ASCII names beside the Unicode ones, the `show` line in `--help`
   matches the session's `help`, and the goal-number rule is stated.
9. *As a logician, I hand in a linking and read the criterion's verdict
   and witness.* Check: `check` or `prove --net` accepts a linking and
   prints the switching cycle or the parts; `OccId` prints, and a command
   prints a sequent's occurrence ids.
10. *As a paper writer, I name rules in my house style.* Check: the
    `--style` help or README explains the label markup and lists the rule
    keys, the unknown-key error lists them too, and a raw-LaTeX label is
    possible or its absence stated.
11. *As a Rocq user, I close my own two-sided goal with a generated
    lemma, batch-certify named lemmas, and never get a file Rocq cannot
    load.* Check: README has the import-and-apply snippet; a batch names
    lemmas after their records and files as valid module names; `--lemma`
    rejects a non-identifier; `--prelude` warns when it has no effect; a
    refused certificate still prints the verdict.
12. *As a batch user, I give each sequent its own time limit.* Check:
    `batch::run` has a deadline example, or `batch::Options` a per-item
    stop.
13. *As anyone, I add the crate in one copy-paste.* Check: README has a
    "Using the library" paragraph with the dependency line, the features
    and `serde_json`, and `png`/`pdf` name the font file and its source.
14. *As a student, advice on an `unknown` tells me when more resources
    will not help.* Check: the `unknown` line for `!(A + B) |- !A + !B`
    after `--timeout 10s` does not recommend a limit as a cure.

## 5. How to use this note at the last check

A fresh reader repeats the four task lists with the inputs quoted in the
accounts and records the same three outcomes. Compare: tasks done without
trouble (14 of 37 here), tasks stuck (2), frictions of section 2 still
reported (12), stories of section 4 whose check passes (0 of 14, by
construction). (inference) The first two numbers measure the refactor;
the last two, whether its design was the right one.
