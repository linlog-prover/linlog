# A Rocq user tries linlog's certificates

Persona: I work in Rocq and want linlog to find proofs of linear-logic
sequents, check its certificates in Rocq and use the lemmas in a
development of my own. I had never used linlog. I read only README.md
in the snapshot and the published rustdoc (front page, `export`,
`export::rocq`, `proofs::Proof`, `ViewOptions`, `Compact`, `Error`,
`search::Options`, `Outcome`, the `ordinary` module). I ran the release
binary (`linlog-cli 0.1.0`) at low priority on cores 0 and 1. Rocq is
not installed on this machine and I was told to build nothing, so I could
not run `rocq compile`. Instead I read NanoYalla 1.1.3's `nanoll.v` and
`macroll.v`, which the README links, and checked the scripts' rule
names and argument shapes against them by hand. Every script I read
matched the signatures of `tens_r_ext`, `parr_r_ext`, `oc_r_ext`,
`de_r_ext`, `wk_r_ext`, `co_r_ext`, `plus_r1_ext`, `top_r_ext`,
`ex_perm_r`, `ax_expansion` and `cbn_sequent`. A "done" below therefore
means that the documentation got me to a script I expect Rocq to
accept. The README says a flake check of the project runs Rocq on the
certificates, which is the main reason I trust that expectation.

## The tasks

1. Get a certificate for a first MLL sequent and know how to check it.
2. Fit the file into my development: my own lemma name, my own imports.
3. Use the lemma from my own proofs by instantiating it and matching a
   goal I stated myself.
4. Certify beyond classical linear logic: intuitionistic, affine, Mix.
5. Get a certificate over `Prop` for ordinary logic.
6. Certify a whole list of lemmas in one call (batch).
7. Reach a certificate by other routes: a proof built step by step in
   `interact`, and a JSON proof file through `check`.
8. Library: write the Rust that produces one `.v` file with several
   named lemmas.
9. Library: produce an ordinary-logic `Prop` certificate from Rust.

## 1. First certificate, and how to check it: done, with trouble from the environment only

`linlog prove --format rocq "A * B |- B * A"` gave exactly the README's
lemma. `--standalone` adds `From NanoYalla Require Import macroll.`. The
README's paragraph on checking a certificate is the best part of the
documentation for me. It names the kernel and its exact version (also the
constant `export::rocq::NANOYALLA = "1.1.3"`), the Rocq version (9), says
that Yalla is not needed, gives two ways to build NanoYalla and the
`rocq compile -R … NanoYalla proof.v` line, and states that the script
uses no cut and no axiom. That last point matters to me, because
NanoYalla's Yalla-free installation provides cut as an axiom. The
`--help` text repeats all of this.

What cost time:
- After `make install` the `-R path/to/nanoyalla NanoYalla` flag is
  presumably unnecessary. The README presents the two routes together
  and leaves that unclear.
- NanoYalla's own README says "tested with Coq >= 8.10", while linlog
  says Rocq 9. The README does not say whether the linear certificates
  also work on Coq 8.x. (The classical `Prop` certificates do not,
  because they use `From Stdlib`; see 5.)
- I did not know how to confirm "no axiom" myself. A line suggesting
  `Print Assumptions certificate.` would help.

## 2. My own lemma name and imports: done with trouble

`--lemma tens_comm` and `--prelude '…'` work, and so does
`--style-file` with `{"rocq": {"lemma": …, "form": "standalone",
"prelude": …}}`. The README mentions `--lemma` and `--prelude` in only
half a sentence, but `--help` covers them. Error messages for a bad style
value were good: ``unknown variant `Standalone`, expected `fragment` or
`standalone` ``.

Problems:
- `--prelude` without `--standalone` is silently ignored: the output
  has no prelude and no warning. Only `--help`'s "the lines a
  standalone Rocq file starts with" hints at this.
- `--lemma` is not validated. `--lemma 'my lemma'` writes
  `Lemma my lemma (A : formula) : …` and `--lemma Qed` writes
  `Lemma Qed …`, and both files are broken. Atom names, by contrast, are
  carefully made into identifiers: `formula`, `ll`, `dual`, `Lemma`
  and `certificate` became `formula'`, `ll'`, … as `export::rocq`
  documents.
- There is a prelude but no epilogue. With
  `--prelude 'From NanoYalla Require Import macroll.\nSection S.'` I
  cannot close the section, so I have to post-process the file anyway.

What would help: reject a lemma name that is not a Rocq identifier, and
warn when `--prelude` has no effect.

## 3. Use the lemma in my own development: done with trouble

Worked well: atoms become variables of type `formula`, not `var n`.
The lemma `certificate (A B : formula) : ll [parr (dual A) (dual B); tens
B A]` therefore holds for every formula, and I can apply it to
`tens X Y` or `var 0` at once. The `export::rocq` page explains this,
the occurrence order of the list and the role of the exchanges well.

What cost time: the statement is the one-sided sequent in negation
normal form, with the hypotheses' duals pushed inward and double
negations removed. For `!A, !(A -o B), !(B -o C) |- C` it is
`ll [wn (dual A); wn (tens A (dual B)); wn (tens B (dual C)); C]`.
My own statement would be `ll (map dual [oc A; oc (parr (dual A) B);
…] ++ [C])`. After unfolding, that contains `dual (dual A)`, which does
not reduce for a variable `A`. So `exact (certificate A B C)` fails, and
I first have to run NanoYalla's `cbn_sequent` (`cbn …; rewrite
?bidual`). I worked this out by reading NanoYalla's `dual`. (I first
thought NanoYalla's `dual` reversed `tens`/`parr` as Yalla's does. It
does not, so the shallow cases are convertible and only double
negations are the snag.) Neither the README nor the rustdoc says that
the statement is in NNF, how it relates to `map dual Γ ++ Δ`, or how to
use the lemma from another file.

What would help: one short Rocq snippet in the README that imports a
generated `proof.v`, states the two-sided goal and closes it with
`cbn_sequent; apply certificate.`, or whatever the intended idiom is.

## 4. Intuitionistic, affine, Mix: done

The documentation is clear and honest on this. An intuitionistic proof is
certified as the classical proof it is. `-i "A -o B -o C, A * B |- C"`
gave a classical `ll [...]` lemma, as the README says. Mix gives `error:
no certificate: NanoYalla has no Mix rule, …` and affine weakening gives
`… NanoYalla weakens ? formulas only …`, both with exit status 2. An
affine proof that weakens only `?` formulas (`-a "!A, B |- B"`) is
certified with `wk_r_ext`. That is a nice touch, and
`export::rocq::Unsupported` documents it.

Small costs:
- On a refusal, the verdict line (provable) is lost. The command prints
  only the error and exits 2, so a script cannot tell "provable, no
  certificate" from a parse error without running a second time.
- As an ILL user I get no ILL statement. That is listed as planned ("A
  Rocq library of linlog's own"), so this is fair.

## 5. Ordinary logic over `Prop`: done

This was the best experience. `--logic intuitionistic --format rocq
'a \/ b -> b \/ a'` gives a closed term proof over `forall a b :
Prop`, with no library needed. A classical proof uses `NNPP`, and
`--standalone` adds `From Stdlib Require Import Classical_Prop.`. A
sequent with several succedents becomes their disjunction, as
`export::rocq::ordinary` documents. Atoms that clash with Rocq names are
renamed (`Prop'`, `fun'`, `NNPP'`).

Costs:
- My Rocq habit made me write `True`, which linlog reads as an atom, so
  `X -> True` came out "not valid". The README does say `true`/`⊤`, so
  this was my mistake, but a note for Rocq users would prevent it.
- `--prelude` is silently ignored for ordinary certificates, even
  intuitionistic standalone ones, which have no import at all. The doc of
  `rocq::ordinary` ("a standalone file imports [NNPP] instead of
  options.prelude") reads as if this applies only to classical ones.
- `From Stdlib` ties the classical certificates to Rocq 9. That is
  consistent with the README, but it is not stated next to this
  example.

## 6. Many lemmas in one call: done with trouble

`prove --input-format lines --file lemmas.txt --format rocq --standalone
--output certs` writes one `.v` per proved line. Without `--output` the
error says what to do: `a batch writes each Rocq into a directory: give
it with --output DIR`.

Problems for Rocq:
- The file names come from the line names: I got `modus ponens.v` and
  `lemmas.txt:4.v`. Neither is a valid Rocq module name, so `rocq
  compile` cannot take them.
- Every lemma is called `certificate`. The line's name (`tens_comm:`)
  does not become the lemma's name, a JSON Lines record cannot name its
  lemma (an extra `"lemma"` key is silently ignored), and `--lemma`
  names all of them alike.
- Without `--standalone`, each file is a bare fragment that does not
  compile on its own. The README's batch section does not warn about
  this.

What would help: for `--format rocq`, name the lemma after the record's
name and make file names valid module names, or offer one combined `.v`
file with one import and many lemmas.

## 7. Other routes: interact and check: done

In `interact`, `help` lists `proof --rocq`, and `proof cert.v` picks Rocq
from the extension. `interact` takes `--lemma` and `--prelude` but not
`--standalone`. I got a standalone file only by guessing
`--style rocq.form=standalone` from the `--style` help. `linlog check
--format rocq --lemma swap < p.json` turned a JSON proof into a
certificate on the first try, although the README names only LaTeX and
Typst for `check`.

## 8. Library: one `.v` with several named lemmas: done with trouble (not compiled)

The `export::rocq` page has a complete example (`prove`,
`proof.derivation()`, `rocq::derivation`). `rocq::Options` has public
fields and a documented `Default` ("the lemma certificate alone, and
the import of NanoYalla's macroll as the prelude"). The example avoids
the clash between the two `Options` types with `Options as Search`. What
I would write:

```rust
use std::error::Error;
use std::time::{Duration, Instant};
use linlog::export::{Form, rocq};
use linlog::{Compact, Mode, Options as Search, Sequent, Verdict, ViewOptions, prove_until};

fn certify(lemmas: &[(&str, &str)]) -> Result<String, Box<dyn Error>> {
    let mut file = rocq::Options::default().prelude; // the NanoYalla import
    file.push_str("\n\n");
    // A compact derivation has no certificate (Unsupported::Compact).
    let view = ViewOptions::default().compact(Compact::Never);
    for &(name, text) in lemmas {
        let sequent: Sequent = text.parse()?;
        let deadline = Instant::now() + Duration::from_secs(2);
        let outcome = prove_until(&sequent, Mode::CLASSICAL,
            &Search::default().copies(None), || Instant::now() >= deadline)?;
        let Verdict::Proved(proof) = &outcome.verdict else {
            return Err(format!("{name}: {:?}", outcome.verdict).into());
        };
        let derivation = proof.derivation_with(&view, || false)?;
        let options = rocq::Options {
            form: Form::Fragment,
            lemma: name.to_owned(), // must be a Rocq identifier: not checked
            ..rocq::Options::default()
        };
        rocq::write(&derivation, &options, &mut file, || false)?;
        file.push_str("\n\n");
    }
    Ok(file)
}
```

What cost time:
- `ViewOptions::default()` turns on the compact view where a derivation
  would pass a bound, and `rocq::derivation` refuses a compact
  derivation (`Unsupported::Compact`). For a large proof, the module's
  own example (`proof.derivation()?`) would therefore fail with
  "compact" instead of a size message. I found this only by putting the
  `Compact` and `Unsupported` pages side by side. The `rocq` page should
  say to use `Compact::Never`.
- `rocq::derivation` returns `Unsupported` and `rocq::write` returns
  `WriteError`. Neither converts into `linlog::Error` (its `From`
  impls are `CheckError`, `NetError`, `Refusal` and `ViewError`), so a
  function returning `linlog::Error` cannot use `?` on them. I fell
  back to `Box<dyn Error>`.
- The library's default copy bound is 3 (`DEFAULT_COPIES`), while the
  command deepens it under a two-second limit. To match the command I
  had to combine `copies(None)` with a clock-based `prove_until`, which
  the front page's `prove_until` example suggests.
- The exact default prelude string is not printed in the docs. I relied
  on `Options::default().prelude` and on the CLI output.

## 9. Library: ordinary-logic certificate: done (not compiled)

The `ordinary` module's example goes from text to a checked LJ
derivation, and `rocq::ordinary` takes it from there:

```rust
use linlog::export::{Form, rocq};
use linlog::ordinary::{Logic, Sequent, Translation, translate};
use linlog::{Options, Verdict, ViewOptions, prove};

let sequent: Sequent = "a \\/ b -> b \\/ a".parse()?;
let image = translate(&sequent, Logic::Intuitionistic, Translation::default_for(Logic::Intuitionistic))?;
let outcome = prove(image.sequent(), image.mode(), &Options::default())?;
let Verdict::Proved(proof) = &outcome.verdict else { panic!("valid") };
let linear = image.linear_derivation(proof, &ViewOptions::default(), || false)?;
let lj = image.read_back(&linear)?;
lj.check()?; // rocq::ordinary requires a checked derivation
let mut out = String::new();
let options = rocq::Options { form: Form::Standalone, lemma: "or_swap".into(), ..Default::default() };
rocq::ordinary(&lj, &options, &mut out, || false)?;
```

The only doubt was what `Form::Standalone` with the default prelude (the
NanoYalla import) writes for a `Prop` certificate. The CLI shows that the
prelude is dropped, but the docs do not say so plainly.

## What worked well

- The README's certificate section and `export::rocq`'s module doc: the
  kernel, version, rule names, exchange strategy, identifier mangling
  and the cases with no certificate are all stated precisely.
- Certificates are small and readable, are generic over `formula`, use
  no cut and no axiom, and match NanoYalla's `macroll` signatures.
- Ordinary-logic certificates over `Prop` need no library and are a
  real convenience.
- Error messages for refused certificates and bad options name the
  reason and the fix.

## What would help most

1. A Rocq snippet that uses a generated certificate from another file
   to close a goal stated as `ll (map dual Γ ++ Δ)` (the NNF and
   `bidual` step).
2. Validate `--lemma` and warn when `--prelude` is ignored. For batches,
   name each lemma after its record and make file names valid Rocq
   module names.
3. In `export::rocq`, warn that a derivation must not be compact
   (`ViewOptions::default().compact(Compact::Never)`), and give
   `Unsupported`/`WriteError` a `From` into `linlog::Error`.
4. Keep the verdict line when a certificate is refused.
