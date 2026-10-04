# Step 25 report: ordinary logic through its embeddings

## Outcome

A user can now decide a formula or sequent of classical, intuitionistic or
minimal propositional logic with `linlog prove --logic classical`,
`intuitionistic` or `minimal`. The verdict line names the translation and
the logic its image lies in, and the proof is drawn read back as a
derivation of LK or LJ. A checker of its own validates every read-back
before it is shown. `seq print --logic` prints the image, and `--linear`
shows the linear proof of the image instead. Every output that draws a
derivation also draws LK and LJ: the text tree, LaTeX, Typst, SVG, PNG and
PDF. `--format rocq` writes a certificate over `Prop` that needs no library
(a classical one imports the standard library's excluded middle). The
ILTP library's propositional problems are read in their own TPTP syntax, by
`--input-format tptp` or as a `.p` file under `--logic`, and the flake
fetches them at a pinned version (`nix build .#iltp`).

The library has all of it under `linlog::ordinary`:
- `Sequent`, with its syntax documented on it, and `read_tptp`;
- `translate`, which turns a sequent, a `Logic` and a `Translation` into an
  `Image`;
- `Image::linear_derivation` and `Image::read_back`, which give a
  `Derivation` of LK or LJ, and `Derivation::check`;
- the writers `latex::ordinary`, `typst::ordinary`, `svg::ordinary` and
  `rocq::ordinary`.

On the 274 ILTP problems, at the default time limit of 2 s on one thread
per problem:
- The call-by-name translation decides 96 problems, call-by-value 88 and
  0/1 68. Together they decide 108, and no decided verdict contradicts
  ILTP's status or another translation.
- The call-by-name translation stays the default
  (`Translation::DEFAULT_INTUITIONISTIC`).
- Classical logic through affine MALL decides 156 problems.
- Each run's proofs were read back and checked, and every read-back passed
  except those too large to build (see below).

This step's images equal LLTP's translated files on 199 of the 274
problems (200 under cbv and 01). The others differ:
- exactly on the 72 problems the header note lists as misread (LLTP's `~`
  takes the widest scope);
- on SYN915+1 under cbn, where LLTP writes true as the atom `T`;
- on SYN977+1, where only the grouping of `a | b | c` differs;
- SYN007+1.014 has no LLTP file.

Commits, in order:
1. "Decide ordinary propositional logic through its embeddings into linear
   logic"
2. "Fetch the ILTP library's propositional problems at a pinned version"
3. "Draw derivations of LK and LJ in every output that draws a derivation"
4. "Decide ordinary logic from the command: --logic, --translation, TPTP
   problems"
5. "Show ordinary logic in README's usage"
6. "Give the ordinary connectives their Euler Math advances in SVG"
7. "Bracket a negation that a certificate over Prop passes as an argument"
8. "Hold minimal logic to one formula right of ⊢, and keep its false
   apart from an atom named false"
9. this report.

## The options, and how each front end sets them (D15, D16)

| option | library | default | the command | the web front end, another wrapper |
|---|---|---|---|---|
| the logic | `ordinary::Options::logic`, `Logic` (serde, lower case) | intuitionistic in the library | `--logic classical\|intuitionistic\|minimal`; without it the input is linear logic | a field of the request |
| the translation | `ordinary::Options::translation`, `Option<Translation>` (serde `affine`, `cbn`, `cbv`, `01`) | `None`, meaning `Translation::default_for(logic)`: `Affine` for classical logic, `DEFAULT_INTUITIONISTIC` (cbn) for the others | `--translation affine\|cbn\|cbv\|01` | a field; `Translation::decides(logic)` says which are offered |
| what is shown | none: a proof read back by the caller, or not | the read-back | `--linear` shows the image's proof | both are calls of the library |
| the input syntax | `ordinary::Sequent::from_str`, `read_tptp` | text | `--input-format text\|lines\|tptp`, a `.p` file under `--logic` being TPTP | the page sends text |
| the search, the bounds of the derivation, the drawings, the certificate's name | the existing values: `search::Options`, `ViewOptions`, `TextOptions`, `latex::Options`, `typst::Options`, `svg::Style`, `rocq::Options` | as for linear logic | the existing flags and `--style` | as for linear logic |

The certificate over `Prop` takes `rocq::Options`' `lemma` and `form`. In a
standalone file it writes `From Stdlib Require Import Classical_Prop.` for
a classical certificate in place of `prelude`, which names NanoYalla. That
line is a constant (follow-up 4). `Options` (`ordinary`) has serde; the
command maps its flags onto `Logic` and `Translation` and never builds the
value, since it holds the two flags.

**Quantifiers (D17).** They go into `ordinary::Node`, as `Forall` and
`Exists` over a bound variable, with `Atom` taking terms, and into one row
each of the pattern table (Girard's `∀` as `∀`, `∃` as `∃!`). The
read-back's tags carry only the node, so they need nothing more. The
checker gains the four quantifier rules with their eigenvariable
condition. `Formulas` stays hash-consed, which with binders needs a
nameless representation (de Bruijn indices) so that equality stays id
equality.

## Decisions

- **Names.** The flag is `--logic`, which never collides with
  `--intuitionistic`: that flag means ILL, and the two cannot be combined,
  since the translation decides the mode. In the library the types live
  under `ordinary`: `ordinary::Sequent` beside the linear `Sequent`,
  `Logic` and `Translation`. The translations' short names are LLTP's
  directory names: `cbn`, `cbv`, `01`, and `affine` for the classical one.
- **Syntax.** The connectives are `->`, `/\`, `\/`, `~`, `<->`, `true` and
  `false`, and their Unicode symbols `→ ∧ ∨ ¬ ↔ ⊤ ⊥` (the author asked
  mid-step that both be read, as for linear sequents). `~` binds tightest,
  then `∧`, then `∨`; `→` and `↔` group to the right. A text without a
  turnstile is one formula to prove. The linear symbols `&`, `|`, `*` and
  `-o` are refused, so that a linear connective is never read as an
  ordinary one; TPTP's `&` and `|` are read only from TPTP files.
- **The translations are one table** (`translate::pattern`). For each
  connective and translation function it gives the `!`s on top, the linear
  connective, and each operand with its function and the `!`s added above
  it. The image is built bottom-up from the table. The read-back tags every
  occurrence of the image's forest with the same table, so the two cannot
  drift apart. The 0/1 translation is the one LLTP's translator implements
  (`t₀` for hypotheses under `!`, `t₁` for the goal). That makes this
  step's images comparable file for file; whether it is literally Liang
  and Miller's is the next point.
- **Assumption: "Liang and Miller's 0/1 translation" is LLTP's.** The
  plan names the translation and LLTP's collection `ILLTP-*-01` as the
  same thing. I implemented the latter's definition from its source and
  checked it against the files; I did not check it against the paper.
- **Classical logic is negation normal form into affine MALL.** It uses
  no exponentials, and `→` and `↔` are expanded. The classical negation
  has no image of its own, so the read-back emits a `¬L` or `¬R` wherever
  the proof takes apart a formula under a negation (`unwind`).
- **The read-back works on the standard derivation, not on the proof
  term.** For ILL it uses the two-sided derivation, so that at a `⊗` the
  goal goes to the premise that has no absorbed hypotheses; it never uses
  the compact view. Each linear inference becomes zero or more LK or LJ
  inferences:
  - dereliction and promotion become nothing, since the ordinary sequents
    coincide;
  - `?c` becomes `CL`;
  - `?w` and `wk` become `WL` and `WR`;
  - a connective rule becomes the rule of the ordinary connective.

  The rules are those of Gentzen's calculi with explicit structural rules.
  A two-premise rule shares its context (`∧R` under cbn) or splits it
  (`→L`, and `∧R` under cbv). In LJ, `¬` has the form of `→ ⊥`.
- **The read-back is checked, not trusted.** `Derivation::check` validates
  every inference by itself from the formulas as they stand:
  - the rule against its principal formula;
  - the premises as multisets, either sharing or splitting the context
    (the calculi G1c, G1i and G1m of Troelstra and Schwichtenberg, with
    additive variants, all derivable);
  - at most one formula on the right in LJ;
  - no `⊥L` in minimal logic;
  - the root concluding the sequent.

  It shares nothing with the read-back and has no counter that could
  wrap: it compares sorted vectors.
- **The certificate is a term, not a tactic script.**
  - An LJ sequent `Γ ⊢ C` is a term of type `C`.
  - An LK sequent `Γ ⊢ Δ` is a term of `False`, with a continuation
    `~ D` for each `D` in `Δ`.
  - The rules that move a formula to the right use `NNPP`.
  - Every `match` names its return type, and `↔` goes through a cast to
    the conjunction it unfolds to.

  The term is written by a walk with a stack of its own.
- **Minimal logic** translates false as the atom `false`, which the
  ordinary syntax cannot name. An intuitionistic sequent with nothing on
  the right is decided as `Γ ⊢ ⊥`.
- **The library's batch is unchanged.** "The logic and the translation as
  options of the batch's problems" is met in the command: `--logic` and
  `--translation` apply to every entry, a `.p` file under `--logic` is a
  TPTP problem, and `lines` are ordinary sequents. The library's
  `batch::run` takes a closure, and a wrapper decides ordinary problems
  in it by calling `translate`, `prove` and `read_back`. A field on
  `batch::Problem` would have duplicated those three calls for one
  convenience function.
- **The default translation**: cbn, which decides the most problems (96).
  Of the problems one translation alone decides, cbn has 19 (the larger
  SYJ204, SYJ206+1.007 and SYJ212 instances), cbv 12 (larger SYJ201,
  SYJ202 and SYJ205 instances) and 01 none.

## The ILTP run

Each translation decided the 274 problems of ILTP v1.1.2 through `linlog
prove`'s batch. Every run used these flags: `--file bench/iltp/Problems`,
`--workers 2`, `--cores across` (one thread per problem), `--stats` and
`--output DIR` (every proof read back, checked and written). Each problem
had the default time limit of 2 s. The runs were pinned to cores 4 and 5
(`taskset`) and ran detached, as a systemd unit capped at 8 GB, with the
release build of the step's last code commit. Theorem, Non-Theorem and
Unsolved are ILTP's statuses: 128, 109 and 37.

| run | valid | not valid | unknown | error | Theorems proved | Non-Theorems refuted | Unsolved decided | against ILTP's status | time of the decided (sum, median) |
|---|--:|--:|--:|--:|--:|--:|--:|--:|---|
| intuitionistic, cbn | 68 | 28 | 177 | 1 | 68 | 28 | 0 | 0 | 7.0 s, 0.28 ms |
| intuitionistic, cbv | 72 | 16 | 185 | 1 | 72 | 16 | 0 | 0 | 12.4 s, 0.26 ms |
| intuitionistic, 01 | 56 | 12 | 205 | 1 | 56 | 12 | 0 | 0 | 8.2 s, 0.37 ms |
| classical, affine | 155 | 1 | 116 | 2 | 80 | (1) | 10 | — | 25.2 s, 0.16 ms |
| minimal, cbn | 66 | 30 | 177 | 1 | 66 | 28 | 0 | — | 6.8 s, 0.39 ms |
| minimal, cbv | 70 | 18 | 185 | 1 | 70 | 16 | 0 | — | 12.5 s, 0.39 ms |
| minimal, 01 | 54 | 13 | 206 | 1 | 54 | 12 | 0 | — | 8.2 s, 0.34 ms |

ILTP's statuses are intuitionistic, so a minimal non-theorem may be an
intuitionistic Theorem. Both cases here are: SYN041+1 (`~(p => q) => (q
=> p)`, which needs ex falso) under every translation, and SYJ106+1 under
cbn and cbv. No sequent is minimally valid and intuitionistically not.

- Together, the three intuitionistic translations decide 108 problems.
  No two disagree on a problem, and none contradicts ILTP's status.
- No problem is intuitionistically valid and classically not.
- The 37 Unsolved problems stay undecided intuitionistically; classically
  10 of them are proved.
- The errors:
  - SYN007+1.014 is `p₁ ↔ (p₂ ↔ … )`, 28 deep. Its image unfolds to
    between 0.8 and 1.5 billion occurrences, because `↔` doubles both
    operands at every level, and `--occurrence-limit` refuses it with
    exit status 2. LLTP has no file for it either.
  - In the classical run, the checker gave up on the proof of
    SYJ211+1.018 at its memory bound, without a verdict.
- The classical search terminates without a copy bound, but its cost is
  exponential: 116 problems are unknown at 2 s, all of them large
  instances of the SYJ2xx families.

**Every read-back was checked.**
- **Proofs read back during the runs**: 61 of cbn's 68, 70 of cbv's 72,
  all 56 of 01's, and 123 of the classical 155. Each was read back,
  checked and written to its file.
- **Proofs left out by the default `--derivation-limit` of 64 MiB**: 7
  under cbn, 2 under cbv and 32 classical (their unfolded derivations run
  to millions of inferences).
- **Those 41, checked again** by a scratch program (in the session's
  scratch directory, not committed): it proves each again and reads it
  back with no bound on the derivation's size and a memory bound of
  6 GiB, capped at 8 GB on two other cores. It
  read back and checked 34 of them, the largest of 11.6 million
  inferences (SYJ206+1.005, classical). The other 7 are all classical
  (SYJ201+1.004, SYJ204+1.013, SYJ205+1.016 and .017, SYJ207+1.005,
  SYJ210+1.013, SYJ211+1.017; 8 to 32 million inferences). Their linear
  derivations are estimated over the 6 GiB bound, so they were refused
  before being built and not read back (follow-up 6).
- So every proof of the runs was read back and passed the checker, except
  those 7, which no check reached.

**The certificates.**
- The 34 certificates of the read-back test's sequents compile (again
  after the bracket fix), every
  logic and translation among them (7 classical, 27 intuitionistic and
  minimal). They were compiled with the flake's Rocq 9.1 and its standard
  library, and Rocq printed nothing for any of them.
- The `rocq` flake check now compiles a classical and an intuitionistic
  certificate over `Prop` made by the command.

## The comparison with LLTP's translated files

**The images.** A scratch program translated every ILTP problem by each
of cbn, cbv and 01 and read LLTP's file of the same name with
`lltp::read`. It compared the multisets of root formulas (the atoms'
numbering differs, because LLTP lists the axioms in reverse).

| translation | same | different | no LLTP file |
|---|--:|--:|--:|
| cbn | 199 | 74 | 1 |
| cbv | 200 | 73 | 1 |
| 01 | 200 | 73 | 1 |

The differences are of three kinds:
- The 72 problems `plan/notes/lltp-headers.md` lists as read with `~`
  taking the widest scope, in every translation: LCL181+1; SYJ103+1,
  SYJ105+1.003 and .004, SYJ106+1; SYJ209+1, SYJ211+1 and SYJ212+1 .001
  to .020; SYN001+1, SYN040+1, SYN041+1, SYN046+1, SYN047+1, SYN391+1 and
  SYN392+1.
- SYN915+1 under cbn, where LLTP writes true as the atom `T`.
- SYN977+1, `(a <=> b) | a | b`, which this reader groups to the left as
  TPTP's grammar does and LLTP's to the right. The two images are
  equivalent.

The missing file is SYN007+1.014's in every translation. So the
translations agree with LLTP's wherever LLTP read the problem as TPTP
means it, up to the translation's name.

**The verdicts.** The same batch, with the same flags and cores, ran on
LLTP's translated files in intuitionistic mode (`-i`). Each file was read as LLTP's
format, in intuitionistic mode (`-i`).

| translation | on LLTP's files: provable, unprovable, unknown | on this step's images: valid, not valid, unknown | decided by both | the verdicts differ |
|---|---|---|--:|--:|
| cbn | 65, 48, 160 | 68, 28, 177 | 92 | 4 |
| cbv | 70, 38, 165 | 72, 16, 185 | 87 | 6 |
| 01 | 54, 13, 206 | 56, 12, 205 | 64 | 3 |

Every problem on which the verdicts differ is one whose image differs.
On each of them this step's verdict agrees with ILTP's status, and LLTP's
file contradicts it:
- SYJ212+1.001 and SYN001+1, Non-Theorems, are proved from LLTP's files;
- SYN041+1, a Theorem, is refuted from them;
- under cbv, SYJ103+1 and SYJ105+1.003 and .004 are refuted from them too;
- under cbn, SYN915+1, a Theorem, is refuted from LLTP's file, whose
  conjecture is the atom `T`.

All 21 problems that only LLTP's files decide under cbn (21 under cbv, 3
under 01) are misread ones. Most are refutations of files that are not
even classically valid, which a classical countermodel ends early. The
problems that only this step decides (4, 1 and 4) are misread ones as
well. Where the images agree, the verdicts agree. LLTP's recorded Maude
results are not set beside these, since the step's question was the
translations: `plan/reports/17-assessment.md` 2.2 compares Maude's
results with linlog's on LLTP's files.

## Termination on dyadic sequents, assessed from what the run leaves undecided

Of the 274 problems, 166 are undecided by every intuitionistic
translation: 81 Non-Theorems, 48 Theorems and 37 Unsolved. The copy bound
each search had reached at its time limit sorts them into two groups.

- **35 problems deepened past a copy bound of 30** in some translation,
  some of them to over a million. They are SYJ210 (19 problems), SYJ208
  (6), SYJ209 (4), SYJ207 (2), SYJ211 (2), SYN392 and SYN393, and all 35
  are Non-Theorems. On them each level of the search is cheap, no level
  has a proof, and the deepening never ends: exactly what a loop check
  on the branch, or a bound proved sufficient for the image, would turn
  into refutations.
- **131 problems stayed below a copy bound of 30**: 48 Theorems, 46
  Non-Theorems, 37 Unsolved. Most stopped between 5 and 10, and 23 never
  finished the first level (the large SYJ206 and SYJ201 to SYJ203
  instances). Here each level is too expensive. A loop check does not make
  a level cheaper, and would refute at most the Non-Theorems among them
  once their levels can be completed.

So termination on dyadic sequents would settle about 35 of the 166 (13 %
of the library), all of them refutations. The rest is the cost of the
search, which the terminating calculi of the literature (Dyckhoff's LJT)
address on the intuitionistic side by other means. The engine work is
worth its own step if refuting intuitionistic non-theorems matters for
teaching. It is not worth one for the theorems: none of the 48 undecided
Theorems deepened far.

## The review of the checker

A fresh-context sub-agent reviewed `Derivation::check` for soundness. It
found the checker sound for LK and for intuitionistic LJ, but not for
minimal logic, and it found a second defect beside it. Both are fixed,
each with a test that pins the refusal.

- **Minimal logic let ex falso through an empty right side.** LK's
  one-premise `¬L` reads an empty right side as `⊥`, while `WR` and a
  split `∨L` read it as anything. Together they derive `a, ¬a ⊢ b` and
  `a ∨ b, ¬b ⊢ a`, which are not valid in minimal logic, and the checker
  accepted both when forged. The checker now asks for exactly one formula
  right of `⊢` in every minimal-logic inference (G1m), and
  `minimal_logic_refuses_an_empty_right_side` pins it. No read-back
  produced such a derivation: the ILL translations give `¬` the form of
  `→ ⊥`, and their read-backs keep a goal throughout.
  `decides_and_reads_back` passes unchanged under the stricter check. The
  minimal runs' read-backs were checked before the fix; by that argument
  they pass the new check too, but they were not run again.
- **A TPTP atom named `false` was minimal logic's false.**
  - **Cause:** this crate's syntax cannot name an atom `false`, but a TPTP
    problem can, and in minimal logic false is translated as the atom
    `false`.
  - **Effect:** `p, ~p ⊢ false`, with `false` an atom, was proved by every
    translation. The read-back's check refused the derivation, but under
    `--tree never` no read-back is made, so the command would have said
    "valid".
  - **Fix:** false's atom now gets `_`s appended until its name is free.
    `false_is_no_atom_of_the_sequent` pins it.
  - **Effect on the run:** none. No ILTP problem has such an atom (they
    write `$false`).
- **Noted, not changed:** a certificate of minimal logic writes `⊥` as
  Rocq's `False`, so it proves the intuitionistic statement only, and
  minimality rests on the checker. This is now recorded in
  `core-ordinary.md`.

## Deviations and assumptions

- The verdict words for ordinary logic are `valid`, `not valid` and
  `unknown`, not `provable` and `unprovable`. The exit statuses are the
  same. A "not valid" carries the image's refutation ("the image is
  unprovable: …"), which speaks of the linear sequent.
- The JSON output under `--logic` is the search's outcome on the image,
  unchanged. Ordinary sequents and derivations have no JSON form yet;
  nothing asked for one, and the web front end will need it (follow-up 1).
- `--logic` and the mode flags are refused together by a check in
  `prove` (exit 2), not by clap. A clap conflict would have to name the
  flags `-a` and `--mix`, which `seq print` lacks, and clap's debug
  assertion panics on that.
- `interact` has no `--logic`: proving ordinary logic step by step is not
  required here.
- The `ordinary` module's doc example pins the cbn image of `a → b, b → c
  ⊢ a → c`. The export sub-agent corrected my first version of it, which
  passed the proof where the linear derivation belongs.

## Open questions and follow-ups

1. JSON forms of `ordinary::Sequent` and `ordinary::Derivation`, for the
   web front end.
2. `interact --logic`: a session over the image whose display is read
   back.
3. A loop check for dyadic sequents (previous section): 35 of the
   library's problems, all refutations.
4. The classical certificate's import line is a constant in
   `ordinary::rocq`. A field of `rocq::Options` would make it an option,
   which a front end wanting `From Coq` for an older Rocq would need.
5. Classical logic through affine MALL is exponential on the larger SYJ2xx
   instances (116 unknown at 2 s), as later.md foresaw: not a SAT solver.
6. A derivation read back from a proof that shares subproofs unfolds
   exponentially. Seven classical proofs could not be checked within 6 GiB.
   A read-back on the proof term itself (a DAG) would avoid that, together
   with a checker on terms.

## Verification

- `cargo clippy --workspace --all-targets -- --deny warnings`: clean.
- `cargo test --workspace`: all pass, including the new tests:
  - `ordinary::tests::decides_and_reads_back` (20 sequents, every logic
    and translation, every proof read back and checked);
  - the parser and TPTP tests;
  - `ordinary::derivation::tests::text_tree`;
  - the export snapshots `ordinary.{tex,typ,svg}`;
  - the depth test on ordinary formulas 100 000 deep;
  - `cli/tests/cli.rs::ordinary_logic`;
  - README's new examples.
- `cargo hack check --each-feature -p linlog` and `--feature-powerset
  --depth 2`: clean.
- The `unreachable_pub`/`unnameable_types` check: clean.
- `nix build .#checks.x86_64-linux.export` passed (run by the sub-agent
  before the font change).
- `nix flake check`: all checks passed, on the third run. The first
  failed in the `rocq` check: the classical certificate wrote `NNPP ~ (a
  /\ ~ b)`, an unbracketed negation as an argument, which the 34 sample
  certificates had never produced. Fixed by "Bracket a negation that a
  certificate over Prop passes as an argument". The second failed in
  README's test, whose certificate example the fix changed (`~ (~ a)`),
  and README went into the same commit. A fourth run, after the review's fixes,
  passed as well.
- Every cargo command ran in a scope capped at 8 GB on cores 4 to 9, or 6
  to 9 while the runs held 4 and 5.
