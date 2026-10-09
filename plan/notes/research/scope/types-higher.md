# Scope: types and higher-order logic

Written 2026-10-09 from the web and from `plan/later.md`, this
directory's `README.md`, `design-constraints.md` and `practice.md`. The
question is how linlog's scope could grow toward type systems: linear
logical frameworks, dependent and quantitative type theory, linear and
affine languages, session types, and what a prover offers a type checker.
Each candidate says whether it extends a saved step, sits beside one, or
is new. Sources are cited as [Sn]; "(inference)" marks my own conclusions.

## 1. The field as it stands

**Linear logical frameworks and linear logic programming.** Lolli (Hodas
and Miller, LICS 1991) made a fragment of intuitionistic linear logic a
logic programming language, with clauses that may be used freely or
exactly once [S1]; Forum (Miller, TCS 1996) did the same for all of
higher-order linear logic [S2]. LLF (Cervesato and Pfenning, LICS 1996)
added linear types to the dependently typed framework LF to represent
state, and ran as a logic programming language with type inference [S3];
its unification is linear higher-order pre-unification (LICS 1997) [S4].
CLF (Cervesato, Pfenning, Walker, Watkins 2002) added a monad for
concurrency, and Celf (Schack-Nielsen and Schürmann, IJCAR 2008) is its
implementation in Standard ML: type checking by judgements-as-types,
execution as multiset rewriting, compatible with Twelf signatures [S5].
Celf's repository is GPL-3.0, with no release and no maintenance
statement [S6]; a CMU course page calls its documentation "spotty" [S5].
LolliMon (López, Pfenning, Polakow, Watkins, PPDP 2005) mixes backward
and forward chaining under a monad [S7]; Ceptre (Martens, AIIDE 2015) is
forward-chaining linear logic for game mechanics [S8]. The management of
the linear context in all these systems rests on the input-output model
of Cervesato, Hodas and Pfenning (TCS 2000) [S9].

**Dependent, graded and quantitative type theory.** Benton's LNL (1994)
puts a linear and a non-linear world side by side through an adjunction
[S10]; Krishnaswami, Pradic and Benton (POPL 2015) added type dependency
to it [S11]. McBride's "I Got Plenty o' Nuttin'" (2016) and Atkey's
Quantitative Type Theory (LICS 2018) annotate every variable of a
dependent judgement with a usage from a semiring [S12]; Idris 2 (Brady,
ECOOP 2021) is built on QTT with the quantities 0, 1 and unrestricted,
and uses them for erasure and for session-typed concurrency [S13].
Orchard, Liepelt and Eades's graded modal types (ICFP 2019) generalise
bounded linear logic to an arbitrary resource algebra, implemented in
Granule [S14]; Moon, Eades and Orchard (ESOP 2021) carried grades into a
dependent theory [S15]. Granule is BSD-3-Clause Haskell and needs Z3 to
build, with a `--solver-timeout` option on the checker [S16]; its
uniqueness extension (Marshall, Vollmer, Orchard, ESOP 2022) is a third
modality beside coeffects and effects [S17].

**Linear and affine type checking in practice.** Linear Haskell (Bernardy
et al., POPL 2018) attaches linearity to the function arrow [S18]; GHC's
`LinearTypes` has been experimental since 9.0.1, with multiplicity
polymorphism incomplete [S19], and Tweag's `linear-base` is its standard
library [S20]. Rust's ownership is an affine discipline, as Move's
authors state when contrasting it with Move's resources, which can be
neither copied nor dropped and are checked by a bytecode verifier
[S21]; Oxide (Weiss et al.) gives borrow checking a type-system account
[S22], RustBelt (Jung et al., POPL 2018) a machine-checked safety proof
over separation logic (not in the source) [S23], and Polonius an origin-tracking borrow
checker now heading for stabilisation (not in the source) [S24]. None of these checkers
searches for proofs: a program is a derivation candidate, and the
checker verifies it (inference from [S18], [S22]).

**Sessions.** Caires and Pfenning (CONCUR 2010) read intuitionistic
linear propositions as session types for the π-calculus [S25]; Toninho,
Caires and Pfenning made them dependent (PPDP 2011) [S26] and embedded
them in a functional language through a linear contextual monad (SILL,
ESOP 2013) [S27]. Wadler's CP and GV (ICFP 2012, JFP 2014) do the same
for classical linear logic, with deadlock freedom from the
correspondence [S28]; Lindley and Morris gave GV a semantics and HGV
[S29]; Kokke, Montesi and Peressotti's HCP (POPL 2019) uses
hyperenvironments, with ⊗ as separation between environments [S30];
Dardha and Gay's Priority CP admits cyclic topologies [S31]; Carbone,
Montesi, Schürmann and Yoshida (CONCUR 2015) replace duality by an n-ary
coherence for multiparty sessions [S32], generalising Honda, Yoshida and
Carbone's global types (POPL 2008) [S33]. Rast (Das and Pfenning, LMCS
2022) checks resource-aware session types with Presburger arithmetic
[S34]; Ferrite and `sesh` embed session types in Rust [S35].

**What a prover offers a type checker.** Under Curry–Howard, a closed
term of a type is a proof of the proposition, so inhabitation is
provability: PSPACE-complete for simple types (Statman) [S36],
NP-complete for the Horn and multiplicative fragments of linear logic
(Kanovich, LICS 1992) [S37], even with constants only (Lincoln and
Winkler, TCS 1994) [S38]; with exponentials, weakening alone makes it
TOWER-complete and contraction alone ACKERMANN-complete (Lazić and
Schmitz) [S39]. Uniqueness of inhabitants is a coherence theorem: Komori
and Hirokawa (JSL 1993) characterise the BCK-formulas with one normal
proof [S40]. The use that has been built is synthesis: Hughes and
Orchard's LOPSTR 2020 tool adapts Hodas and Miller's input-output context
to graded linear types [S41], and their ESOP 2024 paper uses grades to
prune type-driven search in Granule and GHC [S42]; Mesquita and Toninho
synthesise functional programs from linear types (INFORUM 2023) [S43];
ReSyn (PLDI 2019) guides synthesis by amortised resource bounds instead
[S44].

## 2. Candidates

### types-higher-1. Inhabitation: enumerate, count, decide uniqueness

*What.* An entry `inhabitants(sequent, bound)` that yields every proof
of an ILL or IMLL sequent as a `Proof`, distinct up to the permutations
the focused calculus already quotients, with a count and a `unique`
verdict; Lolli's example file has a formula with eight proofs and one
with two [S1, as `practice.md` verified] (not in the source). *Who and why.* Synthesis tools
want all inhabitants, not one [S42], and coherence questions (is this
type's inhabitant unique?) are what [S40] answers for BCK; teaching
shows why `a ⊸ a ⊸ a` has two. *Cost.* The engines decide; enumeration
is a different control flow (continue after a proof, share failures but
not successes), and the memo's "proved" entries must be re-expanded.
Canonicity: two sequent proofs of one MLL net are the same inhabitant,
so MLL counts should go through nets (step 5) and the rest through the
focused calculus, whose permutations are not all identifications
(inference). *Relation.* New; beside step 35 (MLL engines) and 25 (the
read-back). *Room now.* `Stop::poll` with progress on the enumeration
(D-9); `search::Options` with a `max_proofs` field under
`#[non_exhaustive]` (D-8); an `Answer` that can carry several proofs,
or an iterator type reserved beside it.

### types-higher-2. Proofs as programs: term printers

*What.* The λ-term of an ILL proof (abstraction for `⊸R`, application
for `⊸L`, pairs for `⊗`, `let` for `!`) printed in GHC `LinearTypes`
syntax (`%1 ->`), Granule and a neutral linear λ-calculus, as formats
beside LaTeX and Typst. *Who and why.* The sequents of `practice.md` row
3 (Lolli, Granule, GHC signatures) get an answer a programmer reads, and
a type class's method can be synthesised from its signature as [S42]
does; a round trip through GHC is a test of the checker. *Cost.* One
read-back pass over the term (the derivation view has the data), one
printer per language, and a decision for the classical side (CP
processes, below). Two-sided proofs only; a one-sided classical proof
has no λ-term. *Relation.* Extends step 22's one-value-per-export
discipline and `core-export.md`'s checklist; beside 25's LK/LJ
read-back. *Room now.* `Rule::ALL` as the one list (D-5); `Walk` public
with binder stops, since a term printer binds variables at `⊸R` and
`∀R` (D-10); the per-root member order kept (D-2), since argument order
is root order.

### types-higher-3. Sessions: types, duality, composition, a process

*What.* A `sessions` layer like `ordinary`: session-type syntax
(`!A.B`, `?A.B`, `⊕{…}`, `&{…}`, `end`, `!`/`?` for shared and
replicated channels) parsed into CLL formulas by Wadler's table [S28];
`dual` by negation; compatibility of two endpoints as a cut (step 34);
and a closed process inhabiting a type as the CP term read off a proof,
drawn as a derivation or, for the multiplicative part, a net. Mix
corresponds to HCP's hyperenvironments, where ⊗ separates environments
[S30], so `--mix` is the HCP reading (inference from [S30]; a panel
should confirm). *Who and why.* Teaching propositions-as-sessions with
a tool that checks duality, finds the deadlock-free process and shows
why a cyclic topology needs Priority CP [S31]; researchers testing a
calculus's rules on examples. *Cost.* A syntax and a printer, the cut of
step 34 for composition, a CP-term read-back (the classical counterpart
of candidate 2), and a decision how far to go: coherence for multiparty
sessions is a judgement of its own [S32], not provability in CLL, so it
is a second checker or out of scope. *Relation.* Extends 34 (cut as
composition) and 32 (the web as the teaching surface); sits beside 25.
*Room now.* `Forest::cut_pairs()` and extra trees after the roots
(D-2); the `Proof` carrying its cut formulas (D-5); the written root
order canonical, since process composition is by named channel.

### types-higher-4. A certificate back end for linear type checkers

*What.* A public, documented builder for `Proof` (nodes over `Rule`,
members, side tables) so that another program's typing derivation can
be handed to linlog as an ILL proof term, checked by `check`, and
certified through the Rocq kernel of step 31. A type checker for a
linear language elaborates its derivation into the term; linlog never
searches. *Who and why.* The checkers above verify, they do not prove
[S18, S22]; what they lack is a trusted base, which [S23] supplies for
Rust at great cost. For a research language whose usage rules are ILL's
(Granule's linear core, Idris 2's 1-quantity, SILL's channels), a
certificate of the usage part is cheap this way (inference). *Cost.*
Low in code: API and docs, a JSON import path that already exists, and
the mapping, which is the user's. The risk is API freeze: the builder's
shape is public from 0.1.0. *Relation.* Extends 30 (the API) and 31
(the Rocq library's `node` over a member type). *Room now.* Exactly
D-1, D-3 and D-5: `Member` not `OccId`, versioned wire forms, `Proof::new`
taking its tables; and `Rule` names and `from_str` as a stable contract.

### types-higher-5. Graded and sub-exponential modalities

*What.* Per-root copy bounds first (`!A` with at most `n` uses: the
`--copies` bound per hypothesis, which Granule's `A [n]` and QTT's
quantities ask for [S14, S13]), then a modality `!_g` indexed by a grade
from a semiring with weakening and contraction flags, which is also what
subexponentials give Forum-style specifications [S45]. *Who and why.*
Synthesis from graded types prunes by grades [S42]; `practice.md` row 3
expands `A [n]` to `n` copies by hand, which loses the bound's meaning
in affine readings. *Cost.* The first stage is an engine option and a
`Reason`. The second is a new `Kind`, a checker rule, the Rocq mirror,
exports and the parser: every layer, and `Kind` is exhaustive, so a
minor version (D-5, D-8). Decidability changes with the flags [S39].
*Relation.* New; beside 38 (another walk through every layer) and 31.
*Room now.* `Kind`'s variant policy written down; `Term` with two
`u32` payloads reserved (D-4) so a grade index fits; `Mode::with_*`
builders for per-root bounds; counters as named fields (D-3).

### types-higher-6. Linear logic programming with terms

*What.* Lolli-style queries over first-order Horn clauses with
unification, and forward chaining over them, as Celf's `petri.clf` and
Ceptre's rules do [S5, S8]: coloured tokens for the Horn engine of step
27, uniform proofs for the rest. *Who and why.* Planning with
parameters, protocols, and the Celf and Ceptre users who want a fast
propositional core and a Rust library; `practice.md` row 7 already lists
these sources. *Cost.* It is step 38's data model plus a trail and
unification in the Horn engine; reachability with data is a different
problem from Petri-net reachability and needs its own bound and
refutations (inference). Higher-order terms and HOAS, which LLF and Celf
have [S3, S5], stay out. *Relation.* Extends 38 and 27. *Room now.*
Everything D-4 reserves (term arena, symbol table, the fragment bit)
and D-7's zone parameter.

### types-higher-7. Relevant mode

*What.* A third structural mode beside linear and affine: contraction
without weakening (BCW terms; "relevant" grades). *Who and why.* The
inhabitation side of relevance logic, where decidability of ticket
entailment reduces to inhabitation over `B, B', I, W` [S46]; a teaching
contrast to affine. *Cost.* A flag in `Mode` and the checker; the
search's prunes are built for linear and affine (counts assume no
contraction outside `?`), and provability under contraction alone is
ACKERMANN-complete already for MALL [S39], so engines would bound, not
decide. *Relation.* New; small beside 36's `cyclic` flag. *Room now.*
`Mode` `#[non_exhaustive]` with constants and builders (D-8); every mode
surface (`Engine` as data, the harness's columns) reading `Mode` and not
the two booleans.

## 3. Considered and rejected

- **An LLF/CLF framework of linlog's own.** Dependent types, HOAS and
  higher-order linear unification [S3, S4, S5] make a different tool;
  Celf and Twelf exist, and no step's data model has binders over
  terms-as-proofs. Candidate 6 takes the first-order fragment only.
- **A QTT or graded dependent core** [S12, S13, S15]. Type checking
  there is normalisation and conversion; the usage part is small and a
  checker, not a prover. Candidate 4 offers what fits: a certificate.
- **Rust borrow checking** [S22, S24]. Lifetimes and aliasing are not
  provability in linear logic; RustBelt's model is separation logic
  [S23] (not in the source).
- **Granule's grade constraints** [S16]. Semiring equations solved by
  SMT, with normalisation often faster than the solver [S15]; not proof
  search.
- **Separation logic and BI** [S47, S48]. BI has a sharing
  interpretation, not number-of-uses, and two implications side by side;
  Iris is a different logic with its own proof mode.
- **Multiparty projection and global types** [S33]. Not linear logic;
  coherence [S32] is noted under candidate 3 as a possible second
  checker only.
- **Amortised resource bounds** (ReSyn, Rast) [S44, S34]: potential
  annotations and Presburger arithmetic, no linear-logic search.
- **Move's bytecode verifier** [S21]: a dataflow analysis over a fixed
  bytecode; nothing to prove.
- **Full Forum** [S2]: higher-order quantification and unification over
  all of linear logic, the same objection as the first item.

## 4. Sources

- [S1] Hodas, Miller. "Logic programming in a fragment of intuitionistic linear logic." LICS 1991; Lolli page, lix.polytechnique.fr. https://www.lix.polytechnique.fr/~dale/lolli
- [S2] Miller. "Forum: a multiple-conclusion specification logic." TCS 165, 1996. https://www.lix.polytechnique.fr/~dale/forum
- [S3] Cervesato, Pfenning. "A Linear Logical Framework." LICS 1996. https://lics.siglog.org/1996/CervesatoPfenning-ALinearLogicalFrame.html
- [S4] Cervesato, Pfenning. "Linear higher-order pre-unification." LICS 1997. https://lics.siglog.org/1997/CervesatoPfenning-Linearhigherorderpr.html
- [S5] Schack-Nielsen, Schürmann. "Celf: a logical framework for deductive and concurrent systems." IJCAR 2008. https://www.itu.dk/~carsten/papers/ijcar08.pdf ; CMU 15-816 software page. https://www.cs.cmu.edu/~fp/courses/15816-s12/software.html
- [S6] clf/celf repository, GitHub. https://github.com/clf/celf
- [S7] López, Pfenning, Polakow, Watkins. "Monadic concurrent linear logic programming." PPDP 2005 (via Miller's survey). https://arxiv.org/pdf/2109.01483
- [S8] Martens. "Ceptre: a language for modeling generative interactive systems." AIIDE 2015. https://ojs.aaai.org/index.php/AIIDE/article/view/12784
- [S9] Cervesato, Hodas, Pfenning. "Efficient resource management for linear logic proof search." TCS 232, 2000. https://kilthub.cmu.edu/articles/journal_contribution/Efficient_Resource_Management_for_Linear_Logic_Proof_Search/6605105
- [S10] Benton. "A mixed linear and non-linear logic." UCAM-CL-TR-352, 1994. https://www.cl.cam.ac.uk/techreports/UCAM-CL-TR-352.html
- [S11] Krishnaswami, Pradic, Benton. "Integrating linear and dependent types." POPL 2015. https://www.cl.cam.ac.uk/~nk480/dlnl-paper.pdf
- [S12] Atkey. "The syntax and semantics of quantitative type theory." LICS 2018. https://strathprints.strath.ac.uk/64031
- [S13] Brady. "Idris 2: Quantitative Type Theory in practice." ECOOP 2021. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2021.9
- [S14] Orchard, Liepelt, Eades. "Quantitative program reasoning with graded modal types." ICFP 2019. https://kar.kent.ac.uk/74450/
- [S15] Moon, Eades, Orchard. "Graded modal dependent type theory." ESOP 2021. https://arxiv.org/abs/2010.13163
- [S16] granule-project/granule repository, GitHub. https://github.com/granule-project/granule
- [S17] Marshall, Vollmer, Orchard. "Linearity and uniqueness: an entente cordiale." ESOP 2022. https://link.springer.com/chapter/10.1007/978-3-030-99336-8_13
- [S18] Bernardy, Boespflug, Newton, Peyton Jones, Spiwack. "Linear Haskell." POPL 2018. https://arxiv.org/pdf/1710.09756
- [S19] GHC User's Guide, "Linear types." https://ghc.gitlab.haskell.org/ghc/doc/users_guide/exts/linear_types.html
- [S20] Tweag, "linear-base." https://tweag.io/blog/2021-02-10-linear-base/
- [S21] Blackshear et al. "Move: a language with programmable resources." Diem technical paper, 2019/2020. https://developers.diem.com/docs/technical-papers/move-paper ; Blackshear et al., "Resources: a safe language abstraction for money" (the Rust affine/Move linear contrast is in its related-work discussion), https://arxiv.org/abs/2004.05106
- [S22] Weiss, Gierczak, Patterson, Ahmed. "Oxide: the essence of Rust." arXiv 1903.00982. https://arxiv.org/abs/1903.00982
- [S23] Jung, Jourdan, Krebbers, Dreyer. "RustBelt." POPL 2018. https://www.mpi-sws.org/news/programming-languages-and-verification/2018/
- [S24] Rust blog, "Polonius update," 2023; Rust project goals 2026. https://blog.rust-lang.org/inside-rust/2023/10/06/polonius-update/
- [S25] Caires, Pfenning. "Session types as intuitionistic linear propositions." CONCUR 2010. https://www.springerprofessional.de/session-types-as-intuitionistic-linear-propositions/3352308
- [S26] Toninho, Caires, Pfenning. "Dependent session types via intuitionistic linear type theory." PPDP 2011. https://web.tecnico.ulisboa.pt/bernardo.toninho/papers/ppdp11-deps.pdf
- [S27] Toninho, Caires, Pfenning. "Higher-order processes, functions, and sessions: a monadic integration." ESOP 2013. https://www.springerprofessional.de/en/higher-order-processes-functions-and-sessions-a-monadic-integrat/4073726
- [S28] Wadler. "Propositions as sessions." JFP 24, 2014. https://doi.org/10.1017/S0956796814000185
- [S29] Lindley, Morris. "Sessions as propositions," PLACES 2014; "A semantics for propositions as sessions," ESOP 2015. https://researchportal.hw.ac.uk/en/publications/sessions-as-propositions/ ; PLACES 2014 paper: https://arxiv.org/pdf/1406.3479
- [S30] Kokke, Montesi, Peressotti. "Better late than never." POPL 2019. https://arxiv.org/pdf/1811.02209
- [S31] Dardha, Gay. "A new linear logic for deadlock-free session-typed processes." FoSSaCS 2018. https://www.dcs.gla.ac.uk/~ornela/publications/DG18.pdf
- [S32] Carbone, Montesi, Schürmann, Yoshida. "Multiparty session types as coherence proofs." CONCUR 2015. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.CONCUR.2015.412
- [S33] Honda, Yoshida, Carbone. "Multiparty asynchronous session types." POPL 2008. https://www.doc.ic.ac.uk/~yoshida/multiparty/multiparty.pdf
- [S34] Das, Pfenning. "Rast: a language for resource-aware session types." LMCS 18(1), 2022. https://lmcs.episciences.org/8954
- [S35] Chen, Balzer, Toninho. "Ferrite." ECOOP 2022. https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2022.22 ; `sesh` crate. https://docs.rs/sesh
- [S36] Wikipedia, "Type inhabitation" (Statman's PSPACE-completeness). https://en.wikipedia.org/wiki/Type_inhabitation
- [S37] Kanovich. "Horn programming in linear logic is NP-complete." LICS 1992. https://lics.siglog.org/1992/Kanovich-Hornprogramminginli.html
- [S38] Lincoln, Winkler. "Constant-only multiplicative linear logic is NP-complete." TCS 135, 1994. https://ftp.math.utah.edu/pub/tex/bib/idx/tcs1990/135/1/155_169.html
- [S39] Lazić, Schmitz. "Non-elementary complexities for branching VASS, MELL, and extensions." arXiv 1401.6785. https://arxiv.org/pdf/1401.6785
- [S40] Komori, Hirokawa. "The number of proofs for a BCK-formula." JSL 58(2), 1993. https://www.cambridge.org/core/journals/journal-of-symbolic-logic/article/number-of-proofs-for-a-bckformula/F6E90E7826DA9652BACBBB619233B78B
- [S41] Hughes, Orchard. "Resourceful program synthesis from graded linear types." LOPSTR 2020. https://www.ncbi.nlm.nih.gov/pmc/articles/PMC7880237/
- [S42] Hughes, Orchard. "Program synthesis from graded types." ESOP 2024. https://link.springer.com/chapter/10.1007/978-3-031-57262-3_4
- [S43] Mesquita, Toninho. "Functional program synthesis from linear types." INFORUM 2023 (listing). https://web.tecnico.ulisboa.pt/bernardo.toninho/publications.html
- [S44] Knoth, Wang, Polikarpova, Hoffmann. "Resource-guided program synthesis." PLDI 2019. https://arxiv.org/abs/1904.07415
- [S45] Nigam, Miller. "Algorithmic specifications in linear logic with subexponentials." PPDP 2009. https://www.academia.edu/305314/Algorithmic_Specifications_In_Linear_Logic_With_Subexponentials
- [S46] Padovani. "Ticket Entailment is decidable." MSCS 2013 (decidability of T→ is equivalent to inhabitation over B, B', I, W); Bimbó, Dunn. JSL 78(1), 2013. https://arxiv.org/abs/1106.1875
- [S47] O'Hearn, Pym. "The logic of bunched implications." BSL 5(2), 1999. https://ncatlab.org/nlab/show/David+Pym
- [S48] Jung et al. "Iris from the ground up." JFP 28, 2018. https://cs.au.dk/~birke/papers/iris-journal.pdf

Sources checked 2026-10-09: 23 checked, 4 corrected, 0 removed, 4 claims marked.
