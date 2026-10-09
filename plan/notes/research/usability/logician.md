# A logician tries linlog from its documentation alone

Persona: I teach and research linear logic. I am at home in a terminal and
new to Rust. I want to check sequents, look at proofs, put proofs into a
paper and step through proofs with students. I had never used linlog. I
used only README.md (snapshot `ws-research`), `linlog --help`,
`linlog <command> --help` and the published rustdoc, starting from
<https://linlog-prover.github.io/linlog/linlog/index.html> and following its
links. I ran the release binary of the code of 2026-10-08 (always as
`nice -n 19 taskset -c 0,1 timeout 60 …`, through a wrapper `ll`). I compiled nothing,
and the Rust below is what I would write, not tested. The files I made were
scratch and are not kept.

## The tasks I would try first

1. Decide textbook sequents (distributivity, currying, the exponential
   isomorphism) and read the proofs.
2. Find out why a sequent fails, including a sequent with exponentials I
   know to be unprovable.
3. Export a derivation to LaTeX for a paper, with my own rule names.
4. Step through a proof with students in `interact`, including a dead
   end and `undo`.
5. Draw a proof net for a slide, and check a linking a student proposes.
6. Ordinary logic: Peirce's law and ¬¬(a ∨ ¬a) in LJ, exported to LaTeX.
7. Save a proof, check it independently and get a Rocq certificate.
8. Library: prove a sequent with a time limit and write a standalone
   LaTeX file of the two-sided derivation.
9. Library: decide whether a given axiom linking is a proof net and print
   why not.

## 1. Decide textbook sequents and read the proofs: done

I followed README's first examples. Twelve sequents gave exactly the
verdicts I expected. They included `A * (B + C) |- (A * B) + (A * C)` (provable),
its converse with `&`, which is unprovable, `!(A & B) |- !A * !B` and
its converse, `?A par ?B |- ?(A + B)` and `1 |- bot`. Exit statuses 0/1
matched the README. The trees are clean and easy to read.

What worked well:
- The verdict line names the fragment and the engine. When a count rules
  the sequent out, the unprovable line says why. For `1 |- bot` it
  showed the MLL count equation with the numbers filled in.
- `linlog --help` has a syntax card with the binding strengths.

What cost time:
- Precedence. `*` binds tighter than `|`, `|` tighter than `&` and `&`
  tighter than `+`, so `A par B & C` is `(A ⅋ B) & C` and `A + B & C`
  is `A ⊕ (B & C)`. It is documented, but it is not the convention
  of most papers. I checked it with `seq print`, which is the right
  tool, but README's syntax paragraph does not give the binding order;
  only `--help` and the `Sequent` rustdoc give it.
- Classical proofs are always one-sided. The hypothesis
  `A -o (B -o C)` shows up as `A ⊗ (B ⊗ ~C)`. For a researcher that is
  fine. Students need a moment, and no option draws a classical
  derivation two-sided (Γ ⊢ Δ). Only `-i` gives two-sided trees.
- In the message for `1 |- bot` ("#⊥ … = 4, and this one has 2"),
  the one-sided sequent `⊢ ⊥, ⊥` that the counts refer to is never
  printed.

What would have helped: one sentence in README's syntax paragraph on the
binding order, with an example such as `A par B & C` = `(A ⅋ B) & C`. The
unprovable message could also print the one-sided sequent it counts.

## 2. Why a sequent fails, with exponentials: done with trouble

`!A -o !B |- !(A -o B)` and `?(A * B) |- ?A * ?B` came back
"unprovable … the search was exhaustive", which is correct and quick.
The exam classic `!(A + B) |- !A + !B` (unprovable) came back
`unknown (LL, classical, focus engine): the time limit of 2s was reached
at a copy bound of 511; --timeout DURATION gives the search longer`, also
with `-i`. I followed the advice and ran `--timeout 10s`. It then stopped
after 2.3 s with `the recursion limit of 2048 was reached … raise it with
--recursion-limit N`. Each piece of advice led to another limit, and I
had no idea whether a larger limit could ever help. README explains
honestly that MELL/LL is three-valued, so "unknown" is fair. Still, the
advice reads as if more resources would decide the sequent. Here they
almost surely won't, and I spent time finding that out.

What would have helped: after deepening that ran out of time, a hint that
more time may not decide the sequent, for example "every copy bound up to
511 left a branch at its bound; the search may not end on this sequent".
The recursion-limit advice should not read as a cure.

## 3. LaTeX for a paper, with my own rule names: done with trouble

`--format latex` gives clean ebproof code, `--standalone` gives a
document that compiles, and `-i` gives aligned two-sided trees. I could
not compile anything because there is no pdflatex on this machine, but
the output looks right for ebproof/cmll. README says exactly which
packages are needed and that no font is chosen, which is the right
choice for a paper.

What cost time:
- I wanted "der" instead of `?d`. `--help` mentions
  `labels.table.⊸L=⊸_L`. I first wrote `--style
  'labels.table.?d=\mathrm{der}'`, and it was escaped into
  `\mbox{\textbackslash}mathrm…`. Then `--style
  'labels={"table":{"foo":"bar"}}'` said `"foo" is not the name of a
  rule` without listing the valid names. The label markup (`_x`
  subscripts, connective characters as symbols, the rest upright text)
  is explained only in the rustdoc of the module `proofs::style`. The
  rule names are listed only on `Rule`. Neither the CLI help nor README
  points to them.
- There is no way to put raw LaTeX in a label, such as
  `\textsc{der}` or `\mathsf{d}`, short of the label markup. For
  matching a paper's house style that is a real limit.

What would have helped: one line in `--style`'s help, or in README, that
explains the label markup and lists the rule keys. The unknown-rule
error could also list the valid names.

## 4. Stepping through a proof with students: done with trouble

The README transcript works as printed. I proved
`A * (B + C) |- (A * B) + (A * C)` by hand. On a wrong `⊕₁` before
the `&`, `close` correctly reported the remaining goal unprovable, a
good teaching moment. `undo` reopened the goals. Then `proof`,
`proof --latex`, `proof --rocq` and `proof proof.json` all worked. In
`-i`, `rules` lists `!L !c !w` and `⊗R (with a split)`, which is very
readable.

What cost time:
- Rule names when typing. `rules` prints `⊕₁ ⊕₂`, which I cannot type.
  I tried `⊕1`, `plus1` and `oplus1`, and all were rejected without a
  suggestion. The ASCII spellings `+1`, `*`, `par`, `-oL`, `&L1`,
  `topR` are documented only in the rustdoc of `Rule`'s `FromStr`
  impl, inside the trait implementations. README shows `-oL` once.
  `with`, `tensor`, `der` and `prom` are not accepted. After probing I
  saw that the names follow the formula syntax, but nothing said so.
- Goal numbers never come back. Applying a rule to goal 0 closes goal 0
  and opens goal 1, and so on. My first scripted session failed step
  after step with "there is no open goal 0". The README transcript shows
  this, but nothing states it, and it is the first thing a student
  trips on.
- The help text is wrong in one place. `linlog interact --help` says
  ``show [latex|typst|svg]``. I typed `show latex`, and it wrote the
  derivation to a file named `latex` in my directory
  (`derivation so far written to latex`). The in-session `help` and
  README correctly say `show --latex`.
- The search's proofs are not minimal. After I applied `!c` by hand on
  `!A |- !A * !A`, `close` grafted a proof that weakens one copy and
  contracts again (`!c`, `!w`, `!c` in a row). It is valid, but odd to
  show students.
- Small: `the proof is not finished: 1 goals are open`.

What would have helped: README or the interact help could list the ASCII
rule names, or `rules` could print them beside the Unicode names. Fix the
`show [latex|typst|svg]` line in the interact help. Add a sentence that
every applied step opens new goal numbers and closed numbers are not
reused.

## 5. Proof nets: drawing done, checking a student's linking stuck on the command line

`prove --net` printed the links with occurrence numbers, and `--output
net.png` / `net.pdf` / `net.svg` gave a clean drawing (dashed blue ⅋
premises, arcs over literals) ready for a slide. Asking for a net of an
MELL sequent gives a clear error.

Stuck: I wanted to give a linking myself (a student's wrong one) and see
the switching cycle or the disconnection named, which README advertises
for the criterion. `check` reads only proofs. A `{"sequent", "mix",
"links"}` file, which is `ProofStructure`'s documented JSON, was
rejected with `missing field 'proof'`. On the command line I found no
way in, and only the library has it (task 9).

What would have helped: `check --net FILE` reading a proof structure's
JSON, or `prove --net --links "1-5,2-4"`, which would print the
criterion's verdict and its witness.

## 6. Ordinary logic: done

`--logic intuitionistic` refuted Peirce's law and proved ¬¬(a ∨ ¬a) with
a readable LJ tree (with `CL`). `--format latex` gave LJ in
`\to`/`\lnot`. `--translation 01` and `--logic minimal` behave as
described. I would use this in class as it is. The only open question
was how `⊥L` is drawn (`⊥ ⊢ ⊥`): that is an instance, fine.

## 7. Check a saved proof and certify it: done

`proof proof.json` in `interact`, then `linlog check --quiet <
proof.json`, answered `valid proof of ⊢ … (classical)`. The proof JSON
over occurrence ids is not something I would write by hand, but I do not
need to. `--format rocq` writes a readable NanoYalla script. I could not
run Rocq here, but README says exactly which kernel version and how to
compile.

## 8. Library: prove with a time limit, export LaTeX: done with trouble

From the front page, `prove_until`, `Options`, `Verdict` and
`export::latex`, I would write:

```rust
use std::time::{Duration, Instant};
use linlog::export::{Form, latex};
use linlog::{Mode, Options, Sequent, Verdict, prove_until};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sequent: Sequent = "!(A & B) |- !A * !B".parse()?;
    // The library's default copy bound is 3, unlike the command's; lift it
    // and stop on a clock instead, as the command does.
    let options = Options::default().copies(None);
    let deadline = Instant::now() + Duration::from_secs(5);
    let outcome = prove_until(&sequent, Mode::INTUITIONISTIC, &options, || {
        Instant::now() >= deadline
    })?;
    match &outcome.verdict {
        Verdict::Proved(proof) => {
            let derivation = proof.two_sided_derivation()?;
            println!("{derivation}");
            let mut tex = latex::Options::default();
            tex.form = Form::Standalone;
            tex.labels = linlog::Labels::Subscript;
            std::fs::write("proof.tex", latex::derivation(&derivation, &tex))?;
        }
        Verdict::Unprovable(why) => println!("unprovable: {why}"),
        Verdict::Unknown(reason) => println!("unknown: {reason}"),
    }
    Ok(())
}
```

What worked well: the front page's two examples are exactly the common
path. `prove_until` explains the stop condition and gives an `Instant`
example. `Options` constants document every default and why.
`Refutation` and `Reason` implement `Display`. The latex module's
example shows `two_sided_derivation`.

What cost time:
- How to depend on the crate. Neither the front page nor README says
  what to put in `Cargo.toml`: it is not on crates.io ("a first release"
  is planned), and the package is in `core/` of a workspace. As a Rust
  newcomer I would guess `linlog = { git =
  "https://github.com/linlog-prover/linlog" }` and hope. The front
  page example also uses `serde_json` without saying so.
- Defaults differ from the command. `Options::DEFAULT_COPIES` is 3, the
  command's is none with a 2 s limit. It is documented on the constant,
  but someone who first used the command gets "unknown, copy bound 3"
  from the library on sequents the command proves.
- `Labels` has no `FromStr` (unlike `Rule`), so the `labels=subscript`
  I know from the command is `linlog::Labels::Subscript` here; my first
  guess, `"subscript".parse()`, would not compile.
- An intuitionistic proof has to be drawn with `two_sided_derivation()`.
  `derivation()` silently gives the one-sided tree even though I proved
  in `Mode::INTUITIONISTIC`.
- Building formulas programmatically: `Sequent` has `new`, `add` and
  `parse`, but no formula constructors, so generating a family of
  formulas means generating strings. That is fine, but worth saying.
- Several module pages repeat their first sentence twice ("Export of
  sequents and derivations to LaTeX, … Export of sequents and
  derivations: to LaTeX …"; the same on `nets`, `families`, `lltp`,
  `mist` and `proofs::style`). `proofs::style` also mentions a
  "crate-private trait Drawn", which a reader cannot follow.

## 9. Library: is this linking a proof net? Done with trouble

```rust
use linlog::{Forest, OccId, ProofStructure, Sequent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ⊢ ~A ⅋ ~B, B ⊗ A: 0 ~A⅋~B, 1 ~A, 2 ~B, 3 B⊗A, 4 B, 5 A (preorder).
    let sequent: Sequent = "A * B |- B * A".parse()?;
    let forest = Forest::new(&sequent)?;
    for o in forest.ids() {
        println!("{}: {}", o.get(), forest.formula(o));
    }
    let o = OccId::new;
    let net = ProofStructure::from_links(forest, false, &[(o(1), o(5)), (o(2), o(4))])?;
    match net.is_correct() {
        Ok(()) => println!("{}", net.sequentialize()?.derivation()?),
        Err(why) => println!("not a proof net: {}", why.describe(net.forest())),
    }
    Ok(())
}
```

What worked well: `Forest` documents the preorder numbering precisely
with an example, and `ProofStructure` documents the Danos–Regnier
criterion, the error witnesses (`SwitchingCycle`, `Disconnected`) and
`describe`. This is exactly what I want for teaching.

What cost time: `OccId` has no `Display`, so it needs `.get()`. To see the
numbering I had to work it out myself or print it as above, while the
command's `--net` prints it (`~A[1] — A[5]`) only for nets it found.

## Summary

The command is excellent for checking textbook sequents and exporting
proofs. Verdicts, trees, LaTeX/Typst/PNG/proof-net output and LJ/LK
proofs all worked first time from README's examples. What cost the most
time:
- ASCII rule names in `interact` are documented only in rustdoc.
- `interact --help` says `show latex`, which wrote a file named `latex`.
- Goal numbers that are never reused.
- Label markup and rule keys for `--style` are documented only in
  rustdoc.
- No command checks a user-given linking.
- The advice on "unknown" for `!(A + B) ⊢ !A + !B` leads from one limit
  to the next.
- The library docs never say how to add the crate as a dependency.
