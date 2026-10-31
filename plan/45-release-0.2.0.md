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

## What steps 40 to 44 left you (finished 2026-10-31, at the review of 44b)

- **The planned break, accounted by the reviews** (each in its Status
  entry of `plan/README.md`), for 45d's tool to confirm and the
  changelog to list: new variants of closed enums `Term::{Forall,
  Exists}`, `Kind::{Forall, Exists}`, `Node::{Forall, Exists}`,
  `Rule::{Forall, Exists}`, `FoTerm::{Eigen, Meta}`, `Fault::{Instance,
  Eigenvariable, Reintroduced}`, `Refusal::Terms`, `Space::Symbol`, and
  step 41's cut variants; `Placed::parent` an `Option` (41c);
  `Error::Quantifiers`, raised nowhere since 42k, to retire or keep with
  a reason. Every other change since 0.1.0 is said to be an addition;
  the tool decides.
- **Rocq**: certificates that 0.1.0 wrote with a record literal (one-sided
  Mix or affine proofs and their refutations) no longer compile, the
  library's record having gained `cut` (41a): the changelog says so. The
  library's first-order development (42g: `Terms.v`,
  `FirstOrderFormulas.v`, `FirstOrderCalculus.v`,
  `FirstOrderIntuitionistic.v` join the trusted base) and every kind of
  certificate 0.2.0 writes compile; its version is still 0.1.0, for 45a
  and 45d to raise.
- **The wire forms are at level 7** (quantifiers, witnesses, the
  session's instances and undo marks), none released before 0.2.0.
- **The manual** (44b): README a front page, the reference an mdBook
  manual under `docs/`, built into the Pages site at `/linlog/manual/`
  by the flake's `doc` package; a release's manual is its tag's `docs/`.
  45d prepares both for 0.2.0; the benchmark chapter takes 45b's figures
  and the fourth baseline's beside the old (the author, 2026-10-30).
- **Benchmarks**: timed runs need no yes from the author, only
  coordination with the other sessions through the planning session (the
  author, 2026-10-30); the night baseline still waits for the night the
  author gives. The latest target sets are `bench/targets/42k.csv` and
  `net-42k.csv`. LinearOne is in the comparison at first order
  (`bench/comparison/first-order/`). `chain-fo` above about 400 copies
  ends at the recursion limit (a follow-up in `later.md`): 45b chooses
  sizes that decide, or says where a family ends at a limit.
- **Order of the sessions** (the planning session, 2026-10-31): 45b, the
  growing families, runs first, since it needs nothing from 45a; 45a's
  repository is created through `gh`, which asks the author, so 45a's
  GitHub part waits for the author's presence while the rest of it is
  prepared; 45c on the night the author gives; 45d after 45a and 45c;
  45e after the author's tags.

## How the step runs

Five sessions on Opus 5.5 at `high`, each with its goal in its launch
prompt and reviewed by the planning session before the next; the
fifth (45e) after the author's tags. (45b was added on 2026-10-30 and
the later sessions' letters moved by one; the Status log keeps the old
ones.)

- **45a, the Rocq library in its own repository** (the author,
  2026-10-24; D20 and D22 foresaw it): `rocq/` becomes the repository
  `linlog-prover/rocq-linlog`, started fresh (one initial commit of the
  tree as it stands, no history carried over; linlog's history keeps the
  record, and both changelogs say where the library went), EUPL-1.2.
  - **What moves**: the library (`rocq/theories`, its `tests/` with the
    certificates that test it, `Makefile`, `_RocqProject`, the opam file,
    its README with the logo), the bridges to NanoYalla's `nanoll.v` and
    Yalla's `nanoill.v` with their flake inputs, the flake modules that
    build and check them (`modules/rocq-linlog.nix`,
    `modules/rocq-bridges.nix`: the library built as nixpkgs builds one,
    every certificate compiled, the theorems' assumptions closed under
    the global context), and `.claude/rules/rocq.md` as the new
    repository's own rules, with a CLAUDE.md, a flake in linlog's
    dendritic style, the licence check, and CI that runs its `nix flake
    check` and pulls from the Cachix cache `linlog-prover`.
  - **What stays in linlog**: the exporter and its second kernel
    (`--kernel linlog`), whose certificates linlog's flake still compiles,
    now against the library taken as a pinned flake input
    (`rocq-linlog`, as the client takes linlog), and the NanoYalla export
    check (`modules/rocq.nix`), which needs no library of linlog's. No
    certificate test is lost: each one either moves with the library or
    stays as linlog's exporter check, said in the report which.
  - **Everything that names `rocq/`** follows: CLAUDE.md (its crates
    list, the commands, the verification table), README, `CITATION.cff`
    where it names the library, the rules files, `plan/README.md`'s D20
    and D22 with a dated note, the guide for AI models (step 44, its
    rule in conduct.md), the client's About if it links the library.
  - **GitHub, by the planning session** (the author's yes of
    2026-10-24): the repository created public in the organisation, the
    branch ruleset and topics as the client's, Actions on; its
    `CACHIX_AUTH_TOKEN` is the author's to add. The new repository's
    `main` is pushed before linlog's change that takes it as an input.
  - Checked by both flakes' `nix flake check`: the library's at the new
    repository's head, linlog's (with the exporter's certificates against
    the pinned library) at its head.
- **45b, families that grow, and their graphs** (the author,
  2026-10-30; after 44b, before the baseline): problems of linlog's own
  beyond the libraries, chosen to show where each engine and each other
  prover is strong or weak, in sets that grow in size so that the growth
  of the time to solve can be seen, and graphs of it in the comparison
  and the manual.
  - **A family per claim**: each says what it shows and what is
    expected, and has its verdicts by construction: an engine's strength
    (the net engine on wide unit-free MLL, the Horn engine on counters,
    the inverse method on Mix, the memo on duplicated `&` subgoals, the
    first-order search's mechanisms of 42i and 42j), an engine's
    weakness (the copy bound's deepening on a long `!` chain, the
    focused engine's splits on wide tensors), and another prover's known
    limit (llprover's contraction cap of three, LinearOne's MILL1 only).
    Research first (sub-agents on Sonnet 5.5 at `medium`, conduct.md's
    rule): the literature's hard families for linear and intuitionistic
    logic, ILTP's and TPTP's parametrised classes, the other provers'
    own benchmark sets. linlog's seventeen families
    (`core/src/families.rs`) and 42h's first-order ones are reused
    where they show a claim; a new one is written only where none does.
  - **Sizes that grow geometrically**, each family up to the size where
    every engine reaches the time limit or a stated ceiling; the
    machine time the nights need estimated by arithmetic in the report
    before any run.
  - **The harness**: the growing sets selectable in `linlog-bench run`,
    each family run under the default and under every engine that admits
    it, forced; the other provers through step 29's harness
    (`bench/comparison.sh`) on the families their logic covers.
  - **The graphs**: time against size per family, a line per engine and
    per prover, log scales, the fitted growth (a polynomial's degree or
    an exponential's base) in a table beside each, a run past its limit
    drawn at the limit; drawn by `bench/plots.py` in both colour themes
    by the `dataviz` skill's rules, committed as SVG. Step 29's cactus and
    scatter plots (`bench/comparison/2026-10-15/figures/`), drawn but
    never shown, are shown beside them.
  - **What runs when**: this session runs only each family's smallest
    sizes, for the verdicts (no `MISMATCH`); the curves come from the
    baseline's night (45c) and the second night (45c); 45d puts the
    graphs and the fourth baseline's results into `bench/COMPARISON.md`
    and the manual's benchmark chapter, beside the older results and
    figures 44b put there (the author, 2026-10-30: the results with
    their figures, the old problems' and the new ones', in the manual),
    every figure drawn by `bench/plots.py` from the rows.
  Opus 5.5 at `high`; a fresh-context reviewer on Opus 5.5 at `high` of
  each family's verdicts by construction and of its claim against the
  engines' documented behaviour.
- **45c, the fourth baseline**, comparable with the third
  (`bench/results/2026-10-17/`; `bench/COMPARISON.md` gains a column),
  by `.claude/rules/bench.md` and the lessons of steps 14, 16 and 30: the
  estimate by arithmetic and a word to the supervisor before the start,
  the night the author's go, detached (`bench/baseline.sh --detach`),
  stopped by the author's word through the graceful-stop skill and
  resumed; the target sets against their latest labels; no probes
  beyond those named. It runs 45b's growing families over every engine
  that admits them. A second night, on a date the author gives (asked
  for on 2026-10-30), runs the other provers on 45b's families through
  step 29's harness, and step 29's library comparison with 0.2.0's
  engines if the night has room; without that rerun, README's "Beside
  the other provers" says the library comparison is of 0.1.0's engines.
- **45d, the release prepared**, so that the author's acts (the tag, and
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
    file in its own repository (and its tag, by the author, beside
    linlog's), the bindings' package, the changelog dated; a dry run of
    publishing both crates (`cargo publish --dry-run`) and the
    `package` check; what docs.rs builds.
  - **The faces**: README (a front page since session 44b) and the
    manual (`docs/`, 44b's) as a released tool's (what 0.2.0 adds: the
    two-sided surface, MELL nets, cut and its elimination, first-order
    logic, the logo); the guide for AI models (step 44) for 0.2.0, its
    tests passing; the web client's linlog input on the release's
    commit, after the author's tag, by the planning session.
  - **Installing on each system** (the author, 2026-10-25): README's
    "Install" (today `cargo install` and the flake) becomes instructions
    for the systems linlog's users have: Linux by its main families
    (Debian and Ubuntu, Fedora, Arch, NixOS and Nix elsewhere), macOS
    and Windows, each with how to get what it needs (a Rust toolchain of
    at least the `rust-version`, which a distribution's own `rustc` may
    not reach, so rustup where it does not; the build tools Windows
    needs) and how to update and remove it again.
    - **Research first**, by conduct.md's rule: how comparable Rust
      command-line tools tell users to install them today, and which
      channels each system's users expect (prebuilt binaries on the
      GitHub release and `cargo binstall`, Homebrew, the AUR, winget or
      Scoop, a NixOS or home-manager configuration through the flake),
      with the upkeep each costs. The report recommends which to offer
      for 0.2.0, and the session builds only what needs no account or
      outward act: instructions for what exists, and prebuilt binaries
      if it recommends them, made by the release workflow in the way
      current tools make them (checksums with them), so that the
      author's tag publishes them. A channel that needs an account, a
      package in someone else's repository or a tap of the
      organisation's is proposed to the author in the final message,
      not created (nixpkgs is 45e's).
    - **Native packages, where feasible** (the author, 2026-10-25): a
      `.deb` and an `.rpm` for Linux, and for macOS and Windows what
      their users install directly (a macOS `.pkg` or the archive
      current tools ship, a Windows `.msi`), each made by the release
      workflow from the tag with the tools current practice uses then
      (at the time of writing cargo-deb, cargo-generate-rpm, cargo-wix,
      or cargo-dist for several at once, each adopted through the
      `new-tool` skill), with the licences' texts inside, checksums
      beside them, and installed, run and removed again in a container
      or on a runner before the release relies on them. Feasibility is
      the session's to judge and the report's to say: what each format
      costs to keep, and what it needs that linlog lacks. Code signing,
      above all (Apple's notarisation needs a paid developer account, a
      Windows certificate costs too, and an unsigned package meets
      Gatekeeper's and SmartScreen's warnings), is the author's to
      decide; the session builds what is feasible unsigned, says on the
      download page what a user sees and does, and puts signing to the
      author in the final message.
      Flatpak and AppImage (asked by the author, 2026-10-25) are judged
      the same way, with the planning session's expectation to check
      against their documents of the day: both are made for desktop
      applications with a window; Flathub, as far as the planning
      session knows, does not take console-only programs, and a
      Flatpak's sandbox stands between a command and the files it reads
      and writes, while an AppImage adds nothing to a single
      self-contained binary. So the portable Linux form is likely a
      statically linked binary (musl) in an archive, which runs on any
      distribution; if the documents say otherwise, the session follows
      them and says so.
    - **Every instruction tried**: the commands for a Linux family run
      in that distribution's container where the machine allows it, the
      rest by a workflow that follows README's commands on GitHub's
      Linux, macOS and Windows runners, on a release and weekly, so a
      broken instruction shows; the report says which were tried how,
      and which only after the tag.
    - **The same everywhere it is said**: README, the page's About, and
      the guide for AI models (its rule in conduct.md) agree, the guide
      and README's examples kept true by their tests.
  - **The Rocq library published** (the author, 2026-10-25), once 45a
    has separated it and it holds everything 0.2.0 brings:
    - **Complete for 0.2.0 first**: every kind of certificate linlog
      0.2.0 writes (with cut from step 41, first-order from step 42, the
      two-sided calculi from step 40) compiles against the library and
      rests on its soundness theorems, their assumptions closed; what it
      lacks is listed in the report and its README, not glossed.
    - **By the Rocq community's current practice**, read that day
      (conduct.md's rule): the Rocq opam repository's contribution
      guide (`rocq-prover/opam`) and libraries it accepted in the last
      months, for the package name `rocq-linlog`, the opam file's fields
      (the `logpath:` tag, bounds on `rocq-core` and `rocq-stdlib` that
      the library is tested against, the licence, `dev-repo`,
      `bug-reports`, `doc`), and how such libraries publish their
      documentation.
    - **Its own release, in its own repository**: version 0.2.0 beside
      linlog's; a changelog, `CITATION.cff` and README (with how to
      install it by opam and by Nix, as for the command) of its own; a
      release workflow that makes the GitHub release from the tag, as
      linlog's does; its documentation built (`rocq doc` or what the
      practice of the day uses) and, if recommended, served on its
      GitHub Pages, whose switch is the author's yes; `opam lint` and an
      install from the opam file into a fresh switch for each Rocq
      version its bounds admit, by the flake or a container, before the
      tag.
    - **After the author's tag** (45e): the opam repository's file
      `released/packages/rocq-linlog/rocq-linlog.0.2.0/opam` with the
      release archive's checksum, `opam lint --check-upstream` passing,
      in a local clone, the pull request's text in the report; the pull
      request, the tag and a Zenodo DOI for the library are the
      author's. The Rocq Platform waits for evidence of use
      (`plan/notes/distribution.md`).
  - **The author's reminders**, collected by the planning session from
    `plan/later.md` and the reports, in the final message: what is
    theirs to send, set or decide before or after the tag.

- **45e, the package archives** (the author, 2026-10-25), after the
  author has tagged 0.2.0 in linlog and in `rocq-linlog`: first the Rocq
  library's submission to the Rocq opam repository, prepared as 45d's
  last bullet says; then nixpkgs, as follows: the command as
  the package `linlog`, and the Rocq library as `rocqPackages`' `linlog`
  (or the name nixpkgs' conventions give it), prepared as commits for a
  pull request to nixpkgs that the author opens.
  - **First, whether to submit at all: a decision of the author's**
    (2026-10-25). nixpkgs turns away packages that are "too immature" or
    used by "a niche of 5 people" (`plan/notes/distribution.md`), so the
    session first reads nixpkgs' inclusion criteria as they stand that
    day (`CONTRIBUTING.md`, `pkgs/README.md`, how recent pull requests
    for little-known packages were received) and gathers the evidence of
    use beyond the author that exists then, from public sources only:
    crates.io's downloads of each version, the repositories' stars,
    forks, issues and pull requests by others, citations of the DOI,
    links to the page or the repositories, mentions where users talk;
    the page collects no visitor data, so none is claimed. It sends the
    planning session a short assessment: the criteria quoted, the
    evidence with its sources, the likely reception, and its
    recommendation (submit now, wait for a stated sign of use, or offer
    the package elsewhere, such as linlog's own flake, which already
    serves `nix run`). The planning session gives it to the author, and
    nothing below starts before the author's answer; on "wait", the
    session ends with the assessment in the report and a note in
    `plan/later.md` of what would change the answer.
  - **Idiomatic, by nixpkgs' current practice, read, not remembered**:
    first read, at nixpkgs' `master` of the day, `CONTRIBUTING.md`,
    `pkgs/README.md`, `pkgs/by-name/README.md`, the manual's sections on
    Rust (`doc/languages-frameworks/rust.section.md`) and on Rocq, and
    the pull request template; then three Rust command-line packages
    and one Rocq library merged in the last months, as models. The
    package follows them, not this prompt, where they differ, and the
    report says what was read and what each choice follows.
  - **The command**: `pkgs/by-name/li/linlog/package.nix` in the form
    nixpkgs uses then (at the time of writing: `buildRustPackage` with
    `finalAttrs`, `fetchFromGitHub` at the tag, `cargoHash`, only the
    package `linlog-cli` built and tested, `versionCheckHook`,
    `nix-update-script`, and `meta` with the description, homepage,
    changelog, `mainProgram = "linlog"` and the licences
    `cli/Cargo.toml` names, the fonts' included, each checked against
    `lib.licenses`); the fonts' licence texts installed as linlog's flake
    installs them.
  - **The tests run in nixpkgs' sandbox**, on a loaded builder: the
    command's tests pass there, built as nixpkgs builds them. A test
    that cannot pass there (a time limit, a thread count, a file outside
    the source) is fixed in linlog at its cause, a linlog commit
    reviewed and pushed as usual; until a release carries the fix, the
    package skips it with `checkFlags` and a comment naming that commit.
    Nothing is skipped for convenience.
  - **The Rocq library**: its derivation under nixpkgs' Rocq packages
    from `rocq-linlog`'s tag (at the time of writing
    `mkRocqDerivation`, the release's hash, the Rocq versions it builds
    with), built against each Rocq version nixpkgs has that its opam
    file allows; the bridges stay out.
  - **The maintainer**: the author's entry in
    `maintainers/maintainer-list.nix`, with what they choose to give;
    the session asks the planning session, which asks the author, and
    prepares the entry from the answer.
  - **Checked**: each attribute built in the sandbox (`nix-build -A …`
    or what nixpkgs' docs name then), the formatter nixpkgs pins run
    over every changed file, `nixpkgs-review` over the commits, and on
    `x86_64-linux` only (the report says the other platforms are left
    to nixpkgs' CI).
  - **Where**: a local clone of nixpkgs in a scratch directory, through
    jj, never git; the commits as nixpkgs names them ("maintainers: add
    …", "linlog: init at 0.2.0", and the Rocq library's), each with the
    `Assisted-by:` trailer nixpkgs' `CONTRIBUTING.md` asks of work
    written with an LLM (`plan/notes/distribution.md`).
  - **The author's acts**: the fork, the push and the pull request, and
    whether to be the maintainer. The report holds the pull request's
    text, filled in from the template, saying who uses linlog (the web
    page, the guide for AI models, the citations), since nixpkgs does
    not want a package "of a niche of five people" (distribution.md).

## Deliverables

Thematic jj commits in linlog and in `rocq-linlog`; `bench/results/DAY/`;
the nixpkgs commits in a local clone (45e);
`plan/reports/45-release-0.2.0.md`. Pushing, tagging, publishing and
the nixpkgs pull request are the author's.
