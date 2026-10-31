# Ordinary logic

`--logic classical`, `intuitionistic` or `minimal` reads a formula or a
sequent of ordinary propositional logic instead, written with `->`/`→`,
`/\`/`∧`, `\/`/`∨`, `~`/`¬`, `<->`/`↔`, `true`/`⊤` and `false`/`⊥`
(`~` binds tightest, then `/\`, `\/`, `->`, and last `<->`; `->` and
`<->` group to the right). It is decided by its translation into linear
logic, and the proof is read back as a derivation of LK or LJ, which is
checked by the rules of the logic before it is shown:

```console
$ linlog prove --logic classical '((a -> b) -> a) -> a'
valid (classical logic by the affine translation into affine MALL, focus engine)
  ───── ax
  a ⊢ a
 ──────── WR
 a ⊢ b, a
────────── →R   ───── ax
⊢ a → b, a      a ⊢ a
───────────────────── →L
   (a → b) → a ⊢ a
 ─────────────────── →R
 ⊢ ((a → b) → a) → a
$ linlog prove --logic intuitionistic 'a \/ b -> b \/ a'
valid (intuitionistic logic by the cbn translation into ILL, two-sided engine)
  ───── ax        ───── ax
  a ⊢ a           b ⊢ b
───────── ∨R₂   ───────── ∨R₁
a ⊢ b ∨ a       b ⊢ b ∨ a
───────────────────────── ∨L
      a ∨ b ⊢ b ∨ a
   ─────────────────── →R
   ⊢ (a ∨ b) → (b ∨ a)
$ linlog prove --logic intuitionistic '~~a -> a'
not valid (intuitionistic logic by the cbn translation into ILL, two-sided engine): the image is unprovable: the search was exhaustive
```

Classical logic goes into affine MALL without exponentials (negation
normal form, `∧` as `&`, `∨` as `⅋`), where the search needs no copy
bound. Intuitionistic logic goes into ILL by `--translation cbn`
(Girard's call-by-name translation, the default), `cbv` (his call-by-value
one) or `01` (Liang and Miller's 0/1 translation); minimal logic is the
same with false an atom. There the search deepens its copy bound under
the time limit, so an intuitionistic formula may be answered unknown
(exit status 3). `seq print` shows the image, and `--linear` the linear
proof of it:

```console
$ linlog seq print --logic intuitionistic --translation cbv 'a, a -> b |- b'
!a, !(!a ⊸ !b) ⊢ !b
$ linlog seq print --logic minimal '~a |- a -> b'
!(!a ⊸ false) ⊢ !a ⊸ b
$ linlog prove --logic intuitionistic --linear 'a, a -> b |- b'
valid (intuitionistic logic by the cbn translation into ILL, two-sided engine)
───── ax
a ⊢ a
────── !L
!a ⊢ a
─────── !R   ───── ax
!a ⊢ !a      b ⊢ b
────────────────── ⊸L
  !a, !a ⊸ b ⊢ b
 ───────────────── !L
 !a, !(!a ⊸ b) ⊢ b
```

A `.p` file under `--logic` is a problem in TPTP's syntax, as the ILTP
library writes them (`nix build .#iltp` fetches its 274 propositional
problems), and the batch takes a directory of them. `--format latex`,
`typst`, `svg`, `png` and `pdf` draw the derivation of LK or LJ, and
`--format rocq` writes a certificate over `Prop` that needs no library
(a classical one the standard library's excluded middle), and for a
sequent that is not valid, where its image's refutation is a classical
assignment, the lemma that the formula does not hold for all
propositions, proved by instantiating the atoms with `True` and `False`:

```console
$ echo 'fof(pel1, conjecture, (p => q) <=> (~ q => ~ p)).' > pel1.p
$ linlog prove --logic intuitionistic --quiet --file pel1.p
not valid (intuitionistic logic by the cbn translation into ILL, two-sided engine): the image is unprovable: the search was exhaustive
$ linlog prove --logic classical --quiet --file pel1.p
valid (classical logic by the affine translation into affine MALL, focus engine)
$ linlog prove --logic classical --format rocq 'a \/ ~a'
(* valid (classical logic by the affine translation into affine MALL, focus engine) *)
Lemma certificate : forall a : Prop, (a \/ ~ a).
Proof.
  exact (fun (a : Prop) => NNPP (a \/ ~ a) (fun k'1 : ~ (a \/ ~ a) => (let k'2 : ~ a := fun x' => k'1 (or_introl x') in let k'3 : ~ (~ a) := fun x' => k'1 (or_intror x') in (k'3 (fun h'4 : a => (k'2 h'4)))))).
Qed.
$ linlog prove --logic classical --format rocq 'a -> b, b |- a'
(* not valid (classical logic by the affine translation into affine MALL, focus engine): the image is unprovable: the classical reading is false where b is true and every other atom false (! and ? erased, ⊗ and & as and, ⅋ and ⊕ as or, 1 and ⊤ true, ⊥ and 0 false), and every provable sequent reads as a tautology *)
Lemma certificate : ~ (forall a b : Prop, (a -> b) -> b -> a).
Proof.
  intro H. specialize (H False True). tauto.
Qed.
```
