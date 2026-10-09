---
paths:
  - "core/src/ordinary/**"
---

# linlog core: ordinary logic through its embeddings

Loaded, beside `core.md`, when a file of `core/src/ordinary/` is read:
classical, intuitionistic and minimal propositional logic decided by
translating into linear logic, and the linear proof read back as LK or LJ.

- **Layout.** `mod.rs`: `Logic`, `Translation` (`Affine`, `CallByName`,
  `CallByValue`, `ZeroOne`; short names `affine`, `cbn`, `cbv`, `01`),
  `Options`, the hash-consed arena `Formulas` (`Node`, `NodeId`; equal
  formulas are one id, which the checker's multiset comparisons rely on),
  `Sequent` (two-sided, its syntax documented on it) and the printers;
  `parse.rs` (behind `parse`): the precedence parser in two dialects,
  this crate's (`->`, `/\`, `\/`, `~`, `<->`, `true`, `false` and their
  Unicode symbols) and TPTP's `fof` (`read_tptp`, through
  `lltp::clauses`, which `lltp::read` shares); `translate.rs`: the
  pattern table, `translate`, `Image`; `derivation.rs`: `Rule`,
  `Inference`, `Derivation`, the read-back (`Image::read_back`) and the
  checker (`Derivation::check`); `rocq.rs` (behind `rocq`): the
  certificate over `Prop`, public as `export::rocq::ordinary`.
- **The ordinary syntax refuses the linear symbols** (`&`, `|`, `*`,
  `-o`): later.md's requirement that nobody writes `&` and gets the
  wrong connective. TPTP's `&` and `|` are read only by `read_tptp`.
- **One table is both the translation and the read-back's map**
  (`translate::pattern`): per node and translation function (classical:
  the side, `RIGHT`/`LEFT`; 0/1: `T0`/`T1`; the others one), the `!`s on
  top, the linear connective, and each operand with its function and
  added `!`s. `translate` builds every image bottom-up from it (both the
  image and its dual, so a hypothesis of ILL is the dual), and the
  read-back walks the forest of the image in preorder with the same table
  to tag every occurrence with the ordinary formula, function, `!`s left
  and side. A change to a translation is a change to the table alone;
  never build an image any other way.
- **The 0/1 translation is LLTP's** (`tptpparser/translation.ml`: `t₀`
  for hypotheses under `!`, `t₁` for the goal, `¬A` as `A → false`,
  `A ↔ B` as `(A → B) ∧ (B → A)`), so that this crate's images equal
  LLTP's translated files wherever LLTP's parser read the problem right
  (its `~` takes the widest scope, `plan/notes/lltp-headers.md`), and
  `cbn` writes true as `⊤` where LLTP wrote the atom `T`.
- **Extra nodes**: `translate` adds to its copy of the arena the
  implications `A → B` and `B → A` of every `A ↔ B` and the node false,
  which `¬A` is `A → false` of in ILL; `false` is made first in the build
  order, since a negation anywhere refers to it. The derivation's arena is
  that copy, so its rules name those nodes (`↔R`'s premises, the second
  premise of LJ's `¬L`).
- **The roots are matched by term.** The linear `Sequent` sorts its roots
  by term id; `translate` runs `optimize_atoms` and `optimize_terms`
  (pub(crate) for this), pairs each ordinary root with its term, sorts,
  then `optimize_roots`. Equal terms are interchangeable (an image equal
  as a term is equal as a tree), so any pairing among them is right.
- **Minimal logic**: false is the atom `FALSE` (`false`, which the ordinary
  syntax cannot name, but a TPTP problem can: then `_`s are appended until
  the name is free, and `false_is_no_atom_of_the_sequent` pins it), so no `0` exists and `⊥L` cannot arise; the checker
  refuses `⊥L` in minimal logic anyway. An intuitionistic or minimal
  sequent with nothing right of `⊢` is decided as `Γ ⊢ ⊥`, and `Image::ordinary`
  is that sequent; more than one formula right is `Error::Succedents`.
- **The read-back** reads the linear derivation that
  `Image::linear_derivation` builds (two-sided for ILL, which keeps the
  goal on the premise without absorbed hypotheses at a `⊗`; one-sided
  for affine MALL; never compact) inference by inference in its order,
  mapping each to zero or more ordinary inferences: `!`/`?d` none (the
  ordinary sequents coincide), `?c` `CL`, `?w`/`wk` `WL`/`WR`, a
  connective rule the rule of the ordinary connective under its tag
  (`rule_of`, by node, side and the classical rule name), and below it
  the `¬` rules a classical negation needs, since it has no image of its
  own (`unwind`). LJ's `→L` is Gentzen's, context split; `∧R` and `∨L`
  share it under cbn and 01, split it under cbv (`⊗`).
- **The checker trusts nothing of the read-back**: per inference, the
  rule against the principal formula (a position given), the premises as
  multisets either sharing the context or splitting it (G1c/G1i/G1m of
  Troelstra and Schwichtenberg with additive variants, all derivable),
  at most one formula right in LJ, exactly one in minimal logic (G1m: an
  empty right side is `⊥` to LK's `¬L` and anything to `WR` or a split
  `∨L`, which together are ex falso; a review forged `a, ¬a ⊢ b` that way,
  `minimal_logic_refuses_an_empty_right_side` pins it), no `⊥L` in minimal logic, the root
  concluding the image's ordinary sequent. `¬L` and `¬R` take both
  forms, LK's (one premise; `Γ, A ⊢` for `¬R`) and that of `A → ⊥`. It has
  no counter that can wrap: it compares sorted vectors.
- **The certificate** (`rocq.rs`) is a term, not a tactic script: an LJ
  sequent `Γ ⊢ C` is a term of type `C` (`False` for none), an LK sequent
  a term of `False` with a continuation `k : ~ D` per formula `D` right,
  and the rules that move a formula right use `NNPP` (standalone files
  import `Stdlib`'s `Classical_Prop`). Variables are `h'n`/`k'n`, which no
  atom identifier contains; atoms go through `export::rocq::identifiers`
  plus the names the terms use (`USED`). Every `match` has a `return`,
  and `↔` goes through a cast to the conjunction it unfolds to. A
  negation is bracketed like a binary formula, since it is an
  application. Minimal logic's `⊥` is Rocq's `False`, so a certificate
  of minimal logic proves the intuitionistic statement only (minimality
  rests on the checker).
- **Deciding** is the caller's: `prove(image.sequent(), image.mode(), …)`
  with the command's defaults (no copy bound, the time limit), affine
  classical mode for `Affine`, intuitionistic mode otherwise. The
  default translation for intuitionistic and minimal logic is
  `Translation::DEFAULT_INTUITIONISTIC`, chosen by the ILTP run
  (`plan/reports/25-ordinary-logic.md`).
- **Where quantifiers go**: `Node` gains `Forall`/`Exists` with a bound
  variable and `Atom` gains terms; the pattern table gets their rows
  (Girard: `∀` as `∀`, `∃` as `∃!`), and the read-back's tags carry no
  more than the node.

## Decisions

The author's answers for the release (`plan/notes/api.md` §14), which
the fixes implement and later rounds judge against. Where a bullet above
still describes code that a decision changes, the decision holds, and
the commit that lands it rewrites that bullet.

- **A TPTP file with several conjectures is refused**, naming the
  second, as the LLTP reader refuses one (`core-inputs.md`).
