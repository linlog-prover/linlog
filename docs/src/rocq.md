# Rocq certificates

`--format rocq` writes the derivation as a Rocq certificate for one of
two kernels, which `--kernel` names: `nanoyalla`, the default for
classical linear logic, or `linlog`, linlog's own Rocq library, the
default for Mix, affine and intuitionistic logic and for a sequent with
a quantifier. The verdict becomes a
comment, and `--standalone` adds the import lines.

For `nanoyalla` it is a proof script that the Rocq kernel
[NanoYalla](https://github.com/ComputerAidedLL/click-and-collect/tree/master/nanoyalla)
checks, the kernel of Click & coLLecT: a lemma stating the one-sided
sequent over the atoms as `formula` variables, proved by one derived rule
of the kernel per inference, with an exchange before a `⊗` where the
kernel's list sequents need one, and closed by `Qed`, so Rocq accepts the
file only if the kernel accepts the proof. A proof with Mix or with the
weakening of affine mode has no certificate there, since the kernel has
no such rule; an intuitionistic proof is certified as the classical proof
it is.

```console
$ linlog prove --format rocq "A * B |- B * A"
(* provable (MLL, classical, focus engine) *)
Lemma certificate (A B : formula) : ll [parr (dual A) (dual B); tens B A].
Proof.
apply (parr_r_ext []); cbn_sequent.
apply (ex_perm_r [2; 0; 1] [dual B; tens B A; dual A]).
apply (tens_r_ext [dual B]); cbn_sequent.
{
  ax_expansion.
}
{
  ax_expansion.
}
Qed.
```

To check a certificate, install NanoYalla 1.1.3 (the `nanoyalla`
directory of the Click & coLLecT repository) with Rocq 9 and its standard
library: `./configure && make && make install` there, or
`rocq compile -R . NanoYalla nanoll.v` and the same for `macroll.v`, then
compile the certificate with the kernel on the load path:

```console
$ linlog prove --format rocq --standalone --output proof.v "!A, !B |- !(A * A)"
$ rocq compile -R path/to/nanoyalla NanoYalla proof.v
```

Rocq prints nothing for a certificate it accepts. The kernel needs no
Yalla installation, and the certificate uses no cut and no axiom.

For `linlog` it is a lemma stating the sequent over the library's own
calculus, with the atoms as strings: one-sided in the calculus of the
mode (`ll classical`, or with Mix or weakening), and two-sided for an
intuitionistic proof (`ill intuitionistic` or `ill affine`, hypotheses
and one goal). It is proved by the library's soundness theorem from the
derivation written as a proof term, which the library's checker accepts
by computation, so Rocq accepts the file only if the checker accepts the
term; every rule has its term, Mix and weakening included:

```console
$ linlog prove -i --format rocq "A, A -o B |- B"
(* provable (IMLL, intuitionistic, two-sided engine) *)
Lemma certificate : ill (atom := string) intuitionistic [ivar "A"; ilmap (ivar "A") (ivar "B")] (ivar "B").
Proof.
  apply (soundness_ill String.eqb String.eqb_spec intuitionistic
    (ILmapL (ivar "A") (ivar "B") (IAx "A") (IAx "B"))).
  vm_compute. reflexivity.
Qed.
$ linlog prove --format rocq --kernel linlog "A * B |- B * A"
(* provable (MLL, classical, focus engine) *)
Lemma certificate : ll (atom := string) classical [parr (covar "A") (covar "B"); tens (var "B") (var "A")].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Par (covar "A") (covar "B") (Tensor (var "B") (var "A") (Ax "B") (Ax "A")))).
  vm_compute. reflexivity.
Qed.
```

A first-order proof gets a certificate of the library's first-order
development, in every mode: the predicates applied to terms
(`tapp "f" [...]`), a bound variable as a de Bruijn index (`bound 0`,
the innermost quantifier's), an eigenvariable by its number
(`eigen 0`), every instance with its quantifiers' terms substituted,
and the calculus's optional rules named by their module
(`Calculus.classical`, `Intuitionistic.affine`). A `∀` step names the
body it opens and its eigenvariable, which occurs nowhere in its
conclusion, an `∃` step its witness, here a constant that no `∀` step
introduces, as a domain is never empty:

```console
$ linlog prove --format rocq "forall x. p(x) |- exists y. p(y)"
(* provable (MLL1, classical, focus engine) *)
Lemma certificate : ll (symbol := string) Calculus.classical [fex (covar "p" [bound 0]); fex (var "p" [bound 0])].
Proof.
  apply (soundness_classical String.eqb String.eqb_spec
    (Exists (covar "p" [bound 0]) (eigen 0) (Exists (var "p" [bound 0]) (eigen 0) (Ax "p" [eigen 0])))).
  vm_compute. reflexivity.
Qed.
```

An unprovable sequent gets a certificate too where its refutation is one
of the three that `check` verifies (an unbalanced atom, the count
equation, a classical assignment): a lemma that the mode's calculus does
not derive the sequent, proved by the library's theorem of that invariant
from the atom or the assignment's true atoms, which the library's test
accepts by computation. A refutation that rests on the search has none,
which a line on standard error says:

```console
$ linlog prove --format rocq "A |- B"
(* unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms *)
Lemma certificate : ~ ll (atom := string) classical [covar "A"; var "B"].
Proof.
  apply (unbalanced_refutation String.eqb "A" _).
  vm_compute. reflexivity.
Qed.
$ linlog prove -i --format rocq "A + B |- A"
(* unprovable (IALL, intuitionistic, additive engine): the classical reading is false where B is true and every other atom false (! erased, ⊗ and & as and, ⊕ as or, ⊸ as implies, 1 and ⊤ true, 0 false), and every provable sequent reads as a tautology *)
Lemma certificate : ~ ill (atom := string) intuitionistic [iplus (ivar "A") (ivar "B")] (ivar "A").
Proof.
  apply (iclassical_refutation _ (fun a => existsb (String.eqb a) ["B"])).
  vm_compute. reflexivity.
Qed.
$ linlog prove --format rocq "A & B |- A * B"
no certificate: a refutation of kind `exhausted` rests on the search and has no check
(* unprovable (MALL, classical, focus engine): the search was exhaustive *)
```

The library is `rocq/` of this repository, on Rocq 9 and its standard
library alone (`nix build .#rocq-linlog` builds it, or `make` in `rocq/`
with Rocq on the path); compile a certificate with its theories on the
load path:

```console
$ linlog prove --mix --format rocq --standalone --output mix.v "A, B |- A, B"
$ rocq compile -R path/to/rocq/theories Linlog mix.v
```
