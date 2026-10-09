# Corpus: first-order ordinary logic

A test set for later steps of linlog, in the notation of `fo-embeddings.md` (the specification, in this directory). The question for each item: is the sequent valid in the stated logic (classical, intuitionistic or minimal)?

Each answer was given by one reader and checked blind by a second reader of another model; where they disagreed, a third decided by argument. No item was recorded as disputed, and none was dropped. 60 items: 32 valid, 28 invalid.

## drinker

### fo-ordinary-01
- Item: Classical: ⊢ ∃x (p(x) → ∀y p(y))
- Answer: valid
- Justification: Domains are non-empty. If ∀y p(y) holds, any element x makes the implication true. Otherwise some a has ¬p(a), and x := a makes the antecedent false. LK proof: CR on the goal, ∃R with a fresh parameter c, →R, ∀R with eigenvariable b, then ∃R with b on the copy, →R, ax on p(b), WL/WR (fo-embeddings.md §5(a)). ILTP v1.1.2 SYN925+1.p records classical Status: Theorem.

### fo-ordinary-02
- Item: Intuitionistic: ⊢ ∃x (p(x) → ∀y p(y))
- Answer: invalid
- Justification: Kripke countermodel with growing domains: w0 ≤ w1, D(w0)={a}, D(w1)={a,b}, p(a) forced only at w1, p(b) nowhere. At w0 the only witness is a, and p(a) → ∀y p(y) fails at w0 because w1 forces p(a) but not p(b). ILTP v1.1.2 SYN925+1.p (∃Y (p(Y) => ![X]: p(X))) records Status (intuit.): Non-Theorem.

### fo-ordinary-03
- Item: Classical: ⊢ ∃x (∃y p(y) → p(x))
- Answer: valid
- Justification: If some b has p(b), take x := b. Otherwise ∃y p(y) is false and any element works, since the domain is non-empty. In LK, contraction on the ∃R formula: the first instance uses a parameter c, ∃L on ∃y p(y) introduces eigenvariable b, and the second instance uses b. ILTP v1.1.2 SYN971+1.p records classical Status: Theorem.

### fo-ordinary-04
- Item: Intuitionistic: ⊢ ∃x (∃y p(y) → p(x))
- Answer: invalid
- Justification: Countermodel: w0 ≤ w1, D(w0)={a}, D(w1)={a,b}, p(b) forced at w1 and p(a) nowhere. At w0 the only witness is a. At w1, ∃y p(y) holds through b but p(a) does not, so ∃y p(y) → p(a) fails at w0. ILTP v1.1.2 SYN971+1.p records Status (intuit.): Non-Theorem.

## independence-of-premise

### fo-ordinary-05
- Item: Classical: q → ∃x p(x) ⊢ ∃x (q → p(x))
- Answer: valid
- Justification: If q is false, any element (the domain is non-empty) makes q → p(x) true. If q is true, the hypothesis gives some a with p(a), and x := a works.

### fo-ordinary-06
- Item: Intuitionistic: q → ∃x p(x) ⊢ ∃x (q → p(x))
- Answer: invalid
- Justification: Countermodel: w0 ≤ w1, D(w0)={a}, D(w1)={a,b}, q and p(b) forced only at w1, p(a) nowhere. The hypothesis holds at w0: q is not forced at w0, and at w1 b witnesses ∃x p(x). The conclusion fails at w0: the only witness is a, and q → p(a) fails at w1.

### fo-ordinary-07
- Item: Minimal: ∃x (q → p(x)) ⊢ q → ∃x p(x)
- Answer: valid
- Justification: →R gives q, ∃x (q → p(x)) ⊢ ∃x p(x). ∃L with eigenvariable a gives q → p(a). →L with q yields p(a), and ∃R with a closes the goal. No ⊥ rule is used.

### fo-ordinary-08
- Item: Intuitionistic: ∀x p(x) → q ⊢ ∃x (p(x) → q)
- Answer: invalid
- Justification: Countermodel: w0 ≤ w1, D(w0)={a}, D(w1)={a,b}, p(a) forced only at w1, p(b) and q nowhere. ∀x p(x) is forced at no world (p(a) fails at w0, p(b) fails at w1), so the hypothesis holds. At w0 the only witness is a, and p(a) → q fails at w1.

### fo-ordinary-09
- Item: Classical: ∀x p(x) → q ⊢ ∃x (p(x) → q)
- Answer: valid
- Justification: If ∀x p(x) holds, then q holds and any element witnesses p(x) → q (the domain is non-empty). Otherwise some a has ¬p(a), and p(a) → q is true.

## constant-domain

### fo-ordinary-10
- Item: Classical: ∀x (q ∨ p(x)) ⊢ q ∨ ∀x p(x)
- Answer: valid
- Justification: If q holds, the left disjunct holds. If q fails, each instance q ∨ p(x) gives p(x), so ∀x p(x) holds. This is the LK theorem of fo-embeddings.md §5(c).

### fo-ordinary-11
- Item: Intuitionistic: ∀x (q ∨ p(x)) ⊢ q ∨ ∀x p(x)
- Answer: invalid
- Justification: Countermodel with growing domains: w0 ≤ w1, D(w0)={a}, D(w1)={a,b}, p(a) forced at w0 and w1, q forced only at w1, p(b) nowhere. The hypothesis holds at w0: (w0,a) and (w1,a) have p(a), and (w1,b) has q. The conclusion fails at w0: q is not forced there, and ∀x p(x) fails because p(b) fails at w1. This law characterises constant domains (fo-embeddings.md §5(c)).

### fo-ordinary-12
- Item: Minimal: q ∨ ∀x p(x) ⊢ ∀x (q ∨ p(x))
- Answer: valid
- Justification: ∀R with eigenvariable a, then ∨L on the hypothesis. In the q case, ∨R1. In the ∀x p(x) case, ∀L with a and ∨R2. No ⊥ rule is used.

## double-negation-shift

### fo-ordinary-13
- Item: Intuitionistic: ∀x ¬¬p(x) ⊢ ¬¬∀x p(x)
- Answer: invalid
- Justification: Kripke countermodel: worlds n ∈ ℕ ordered by ≤, D(n)={0,…,n}, and p(k) forced at m iff k < m (upward closed). The hypothesis holds: for k ∈ D(n) and any m ≥ n, the world max(m,k+1) forces p(k), so ¬¬p(k) holds at n. ∀x p(x) is forced at no world m, since p(m) fails at m. So ¬∀x p(x) holds at 0 and ¬¬∀x p(x) fails. DNS holds on finite frames, which is why the countermodel is infinite.

### fo-ordinary-14
- Item: Classical: ∀x ¬¬p(x) ⊢ ¬¬∀x p(x)
- Answer: valid
- Justification: Classically ¬¬A ↔ A, so the sequent is ∀x p(x) ⊢ ∀x p(x), an axiom.

### fo-ordinary-15
- Item: Minimal: ¬¬∀x p(x) ⊢ ∀x ¬¬p(x)
- Answer: valid
- Justification: ∀R with eigenvariable a, then →R with ¬p(a); the goal is ⊥. →L on ¬¬∀x p(x) needs ¬∀x p(x): assume ∀x p(x), ∀L with a gives p(a), and →L on ¬p(a) gives ⊥. ⊥ is only ever a goal reached by →E, so no ex falso is used.

### fo-ordinary-16
- Item: Intuitionistic: ⊢ ¬¬∀x (p(x) ∨ ¬p(x))
- Answer: invalid
- Justification: Glivenko's theorem fails for predicate logic. The ℕ model of fo-ordinary-13 (D(n)={0,…,n}, p(k) at m iff k < m) refutes it. At any world m take x := m: p(m) fails at m because m < m is false, and ¬p(m) fails at m because m+1 forces p(m). So ∀x (p(x) ∨ ¬p(x)) is forced nowhere, ¬∀x(…) holds at 0, and ¬¬∀x(…) fails at 0.

### fo-ordinary-17
- Item: Minimal: ⊢ ∀x ¬¬(p(x) ∨ ¬p(x))
- Answer: valid
- Justification: ∀R with eigenvariable a, then assume h : ¬(p(a) ∨ ¬p(a)) and derive ⊥. Assume p(a): ∨R1 and h give ⊥, so →R gives ¬p(a). Then ∨R2 and h give ⊥. Only →I, →E and ∨I are used, with no ex falso.

## quantifier-negation

### fo-ordinary-18
- Item: Minimal: ¬∃x p(x) ⊢ ∀x ¬p(x)
- Answer: valid
- Justification: ∀R with eigenvariable a and →R with p(a). ∃R with a gives ∃x p(x), and →L on ¬∃x p(x) gives ⊥.

### fo-ordinary-19
- Item: Minimal: ∀x ¬p(x) ⊢ ¬∃x p(x)
- Answer: valid
- Justification: →R with ∃x p(x), and ∃L with eigenvariable a gives p(a). ∀L with a gives ¬p(a), and →L gives ⊥.

### fo-ordinary-20
- Item: Minimal: ∃x ¬p(x) ⊢ ¬∀x p(x)
- Answer: valid
- Justification: →R with ∀x p(x), and ∃L with eigenvariable a gives ¬p(a). ∀L with a gives p(a), and →L gives ⊥.

### fo-ordinary-21
- Item: Intuitionistic: ¬∀x p(x) ⊢ ∃x ¬p(x)
- Answer: invalid
- Justification: Countermodel with constant domain {a,b}: root w0 forcing nothing, and two incomparable leaves w1 (p(a) only) and w2 (p(b) only). ∀x p(x) is forced at no world, so ¬∀x p(x) holds at w0. ¬p(a) fails at w0 because of w1, and ¬p(b) fails at w0 because of w2, so ∃x ¬p(x) fails at w0.

### fo-ordinary-22
- Item: Classical: ¬∀x p(x) ⊢ ∃x ¬p(x)
- Answer: valid
- Justification: Classical De Morgan for quantifiers. If no x had ¬p(x), every x would have p(x), contradicting the hypothesis. LK: →L puts ∀x p(x) on the right, ∀R with eigenvariable a gives ⊢ p(a), ∃x ¬p(a)-side, ∃R with a and ¬R move p(a) left, and ax closes.

### fo-ordinary-23
- Item: Intuitionistic: ¬∀x ¬p(x) ⊢ ∃x p(x)
- Answer: invalid
- Justification: Same model as fo-ordinary-21 (constant domain {a,b}, root w0 empty, leaves w1 ⊩ p(a), w2 ⊩ p(b)). ∀x ¬p(x) is forced nowhere: at w0 and w1, ¬p(a) fails because w1 forces p(a); at w2, ¬p(b) fails. So the hypothesis holds at w0, but no p is forced at w0, so ∃x p(x) fails.

### fo-ordinary-24
- Item: Intuitionistic: ¬¬∃x p(x) ⊢ ∃x ¬¬p(x)
- Answer: invalid
- Justification: Same model as fo-ordinary-21. Every world has an extension forcing ∃x p(x) (w0 to w1, and w1 and w2 themselves), so ¬¬∃x p(x) holds at w0. ¬¬p(a) fails at w0 because w2 has no extension forcing p(a); likewise ¬¬p(b) fails because of w1.

### fo-ordinary-25
- Item: Minimal: ∃x ¬¬p(x) ⊢ ¬¬∃x p(x)
- Answer: valid
- Justification: →R with h : ¬∃x p(x); the goal is ⊥. ∃L with eigenvariable a gives ¬¬p(a). Apply it to ¬p(a), obtained by →R with p(a), ∃R with a, and →L on h. Only → and ∃ rules are used.

## quantifier-connective

### fo-ordinary-26
- Item: Minimal: ∀x (p(x) ∧ q(x)) ⊢ ∀x p(x) ∧ ∀x q(x)
- Answer: valid
- Justification: ∧R. Each side takes ∀R with eigenvariable a, ∀L with a, ∧L and ax.

### fo-ordinary-27
- Item: Classical: ∀x (p(x) ∨ q(x)) ⊢ ∀x p(x) ∨ ∀x q(x)
- Answer: invalid
- Justification: Classical countermodel: domain {a,b}, p = {a}, q = {b}. Every element is in p or in q, but p(b) and q(a) fail, so neither universal holds. The sequent is then invalid intuitionistically and minimally as well.

### fo-ordinary-28
- Item: Classical: ∃x p(x) ∧ ∃x q(x) ⊢ ∃x (p(x) ∧ q(x))
- Answer: invalid
- Justification: Classical countermodel: domain {a,b}, p = {a}, q = {b}. Both existentials hold, and no element is in both.

### fo-ordinary-29
- Item: Minimal: ∃x (p(x) ∨ q(x)) ⊢ ∃x p(x) ∨ ∃x q(x)
- Answer: valid
- Justification: ∃L with eigenvariable a, then ∨L. In the p(a) case, ∨R1 and ∃R with a. In the q(a) case, ∨R2 and ∃R with a.

### fo-ordinary-30
- Item: Minimal: ∀x (p(x) → q) ⊢ ∃x p(x) → q
- Answer: valid
- Justification: →R with ∃x p(x), and ∃L with eigenvariable a gives p(a). ∀L with a gives p(a) → q, and →L closes. ILTP v1.1.2 SYN937+1.p (the biconditional) records Status (intuit.): Theorem.

### fo-ordinary-31
- Item: Minimal: ∀x (q → p(x)) ⊢ q → ∀x p(x)
- Answer: valid
- Justification: →R with q, ∀R with eigenvariable a (a is not free in q or in the hypothesis), ∀L with a, and →L. ILTP v1.1.2 SYN936+1.p (the biconditional) records Status (intuit.): Theorem.

### fo-ordinary-32
- Item: Minimal: ∀x (p(x) → q(x)) ⊢ ∀x p(x) → ∀x q(x)
- Answer: valid
- Justification: →R, then ∀R with eigenvariable a. ∀L with a on both hypotheses gives p(a) → q(a) and p(a), and →L closes. ILTP v1.1.2 SYN394+1.p records Status (intuit.): Theorem.

### fo-ordinary-33
- Item: Classical: ∀x p(x) → ∀x q(x) ⊢ ∀x (p(x) → q(x))
- Answer: invalid
- Justification: Classical countermodel: domain {a,b}, p = {a}, q = ∅. ∀x p(x) is false, so the hypothesis is true, but p(a) → q(a) is false.

### fo-ordinary-34
- Item: Minimal: ∃x (q ∧ p(x)) ⊢ q ∧ ∃x p(x)
- Answer: valid
- Justification: ∃L with eigenvariable a, then ∧L gives q and p(a). ∧R: q by ax, and ∃x p(x) by ∃R with a.

## quantifier-order

### fo-ordinary-35
- Item: Minimal: ∃x ∀y r(x,y) ⊢ ∀y ∃x r(x,y)
- Answer: valid
- Justification: ∀R with eigenvariable b first, then ∃L with eigenvariable a (fresh), ∀L with b giving r(a,b), and ∃R with a. The opposite order of ∀R and ∃L also works, because both are eigenvariable rules with fresh names.

### fo-ordinary-36
- Item: Classical: ∀y ∃x r(x,y) ⊢ ∃x ∀y r(x,y)
- Answer: invalid
- Justification: Classical countermodel: domain {a,b}, r = identity {(a,a),(b,b)}. Each y has x := y, but neither a nor b is r-related to both elements. A prover must reject the eigenvariable violation x := y.

## eigenvariable

### fo-ordinary-37
- Item: Classical: ∃x p(x) ⊢ ∀x p(x)
- Answer: invalid
- Justification: Classical countermodel: domain {a,b}, p = {a}. In LK, ∀R's eigenvariable must differ from ∃L's, so no axiom matches.

### fo-ordinary-38
- Item: Classical: ∀x ∃y r(x,y) ⊢ ∃x r(x,x)
- Answer: invalid
- Justification: Classical countermodel: domain {a,b}, r = {(a,b),(b,a)}. Each x has a successor, but r(a,a) and r(b,b) are false.

## function-terms

### fo-ordinary-39
- Item: Minimal: ∀x r(x,f(x)) ⊢ ∀x ∃y r(x,y)
- Answer: valid
- Justification: ∀R with eigenvariable a, then ∃R with witness f(a). ∀L with a gives r(a,f(a)), and ax closes. The witness is a compound term.

### fo-ordinary-40
- Item: Minimal: ∀x (p(x) → p(f(x))), p(c) ⊢ p(f(f(c)))
- Answer: valid
- Justification: ∀L with c gives p(c) → p(f(c)), and →L yields p(f(c)). ∀L again with f(c) gives p(f(c)) → p(f(f(c))), and →L closes. This needs two instances of one universal hypothesis (contraction).

### fo-ordinary-41
- Item: Classical: ∀x (p(x) → p(f(x))), p(f(c)) ⊢ p(c)
- Answer: invalid
- Justification: Classical countermodel: domain {0,1}, c = 0, f constant 1, p = {1}. p(f(x)) = p(1) is true for every x, so the first hypothesis holds; p(f(c)) holds; p(c) = p(0) is false.

### fo-ordinary-42
- Item: Classical: ∀x (p(x) → q(x)), ∃x q(x) ⊢ ∃x p(x)
- Answer: invalid
- Justification: Classical countermodel: domain {a}, q = {a}, p = ∅. p(a) → q(a) is true and ∃x q(x) is true, but ∃x p(x) is false (affirming the consequent).

## ex-falso-minimal

### fo-ordinary-43
- Item: Minimal: ⊥ ⊢ ∀x p(x)
- Answer: invalid
- Justification: Minimal logic has no ⊥ rule: ⊥ is an atom. A minimal derivation stays a derivation, without ⊥L, when ⊥ is replaced by a fresh atom f, so a classical countermodel with f true refutes it. Here: domain {a}, f true, p(a) false. The hypothesis holds and the conclusion fails.

### fo-ordinary-44
- Item: Intuitionistic: ⊥ ⊢ ∀x p(x)
- Answer: valid
- Justification: ⊥L (ex falso) closes the sequent directly. Alternatively ∀R with eigenvariable a, then ⊥L.

### fo-ordinary-45
- Item: Minimal: ∃x p(x), ¬∃x p(x) ⊢ ∀y q(y)
- Answer: invalid
- Justification: Read ⊥ as an atom f (fo-ordinary-43). Classical countermodel: domain {a}, f true, p = {a}, q = ∅. ∃x p(x) is true, ¬∃x p(x) = ∃x p(x) → f is true, and ∀y q(y) is false. Intuitionistically the sequent is valid (→L, then ⊥L).

### fo-ordinary-46
- Item: Minimal: ∀x (p(x) ∨ ⊥) ⊢ ∀x p(x)
- Answer: invalid
- Justification: Read ⊥ as an atom f (fo-ordinary-43). Classical countermodel: domain {a}, f true, p(a) false. p(a) ∨ f is true and ∀x p(x) is false.

### fo-ordinary-47
- Item: Intuitionistic: ∀x (p(x) ∨ ⊥) ⊢ ∀x p(x)
- Answer: valid
- Justification: ∀R with eigenvariable a, ∀L with a, and ∨L. The p(a) branch closes by ax and the ⊥ branch by ⊥L.

### fo-ordinary-48
- Item: Minimal: ⊥ ⊢ ∀x ¬p(x)
- Answer: valid
- Justification: ∀R with eigenvariable a and →R with p(a) leave ⊥, p(a) ⊢ ⊥, closed by ax on ⊥ with weakening of p(a). Weak ex falso, ⊥ → ¬A, is minimally valid because ¬A is A → ⊥.

## iltp

### fo-ordinary-49
- Item: Intuitionistic: ⊢ ∀x (p ↔ big_f(x)) → (p ↔ ∀y big_f(y))   [ILTP v1.1.2 SYN052+1.p, Pelletier 22]
- Answer: valid
- Justification: ILTP v1.1.2 Problems/SYN/SYN052+1.p records Status (intuit.): Theorem and Status: Theorem. Proof: assume H : ∀x (p ↔ big_f(x)). (→) From p, ∀R with eigenvariable b, ∀L of H with b gives big_f(b). (←) From ∀y big_f(y), instantiate H and the hypothesis with a parameter c (the domain is non-empty) to get big_f(c) → p.

### fo-ordinary-50
- Item: Intuitionistic: ⊢ (r(a,b) ∧ ∀x (∃y r(x,y) → q(x,x)) ∧ ∀u ∀v (q(u,v) → ∀z r(z,v))) → ∃w (r(b,w) ∧ q(w,a))   [ILTP v1.1.2 SYN721+1.p]
- Answer: valid
- Justification: ILTP v1.1.2 Problems/SYN/SYN721+1.p records Status (intuit.): Theorem and Status: Theorem. Proof: r(a,b) gives ∃y r(a,y), so q(a,a). The third conjunct with u = v = a gives ∀z r(z,a), hence r(b,a). Take w := a: r(b,a) ∧ q(a,a).

### fo-ordinary-51
- Item: Intuitionistic: ⊢ (∀x likes(x,bruce) ∧ ∀y (∃z likes(y,z) → likes(lyle,y))) → ∃u ∀v likes(u,v)   [ILTP v1.1.2 SYN727+1.p]
- Answer: valid
- Justification: ILTP v1.1.2 Problems/SYN/SYN727+1.p records Status (intuit.): Theorem and Status: Theorem. Proof: take u := lyle and v arbitrary (∀R). likes(v,bruce) gives ∃z likes(v,z), so likes(lyle,v).

### fo-ordinary-52
- Item: Intuitionistic: ⊢ ¬∃y ∀x (a(x,y) ↔ ¬a(x,x))   [ILTP v1.1.2 SYN957+1.p, Russell's barber]
- Answer: valid
- Justification: ILTP v1.1.2 Problems/SYN/SYN957+1.p records Status (intuit.): Theorem and Status: Theorem. Proof, which is even minimal: assume ∃y ∀x (…); ∃L gives b with ∀x (a(x,b) ↔ ¬a(x,x)), and ∀L with x := b gives a(b,b) ↔ ¬a(b,b). Assuming a(b,b) yields ¬a(b,b) and ⊥, so ¬a(b,b). Then a(b,b) follows from the other direction, and ⊥.

### fo-ordinary-53
- Item: Intuitionistic: ⊢ ∀x (big_f(x,f(x)) ↔ ∃y (∀z (big_f(z,y) → big_f(z,f(x))) ∧ big_f(x,y)))   [ILTP v1.1.2 SYN082+1.p, Pelletier 60]
- Answer: valid
- Justification: ILTP v1.1.2 Problems/SYN/SYN082+1.p records Status (intuit.): Theorem and Status: Theorem. Proof: ∀R with eigenvariable a. (→) Take y := f(a); ∀z (big_f(z,f(a)) → big_f(z,f(a))) is trivial, and big_f(a,f(a)) is the assumption. (←) ∃L gives b with ∀z (big_f(z,b) → big_f(z,f(a))) and big_f(a,b); instantiate z := a.

### fo-ordinary-54
- Item: Intuitionistic: ⊢ ∀x (f(x) → (g(x) ∨ h(x))) → (∀y (f(y) → g(y)) ∨ ∃z (f(z) ∧ h(z)))   [ILTP v1.1.2 SYN407+1.p; f, g, h predicates]
- Answer: invalid
- Justification: ILTP v1.1.2 Problems/SYN/SYN407+1.p records Status (intuit.): Non-Theorem (classically Theorem). Countermodel: constant domain {a}, root w0 forcing nothing, and leaves w1 ⊩ f(a), g(a) and w2 ⊩ f(a), h(a). The hypothesis holds at w0 (f(a) appears only at w1 with g and at w2 with h). ∀y (f(y) → g(y)) fails at w0 because of w2, and ∃z (f(z) ∧ h(z)) fails at w0 because f(a) is not forced there.

### fo-ordinary-55
- Item: Intuitionistic: ⊢ (∀x (a(x) → (b(x) ∨ c(x))) ∧ ¬∀x (a(x) → b(x))) → ∃x (a(x) ∧ c(x))   [ILTP v1.1.2 SYN942+1.p]
- Answer: invalid
- Justification: ILTP v1.1.2 Problems/SYN/SYN942+1.p records Status (intuit.): Non-Theorem (classically Theorem). Countermodel: constant domain {d,e}, root w0 forcing nothing, and leaves w1 ⊩ a(d), c(d) and w2 ⊩ a(e), c(e). The first conjunct holds everywhere. ∀x (a(x) → b(x)) holds at no world (w1 fails at d, w2 at e, and w0 through w1), so its negation holds at w0. ∃x (a(x) ∧ c(x)) fails at w0, since no a is forced there.

### fo-ordinary-56
- Item: Intuitionistic: ⊢ (∀x (big_r(x) → big_p(x)) ∧ ∀x (¬big_q(x) → big_r(x))) → ∀x (big_p(x) ∨ big_q(x))   [ILTP v1.1.2 SYN355+1.p]
- Answer: invalid
- Justification: ILTP v1.1.2 Problems/SYN/SYN355+1.p records Status (intuit.): Non-Theorem (classically Theorem, by cases on big_q(x)). Countermodel: constant domain {a}, w0 ≤ w1, big_q(a) forced only at w1, and nothing else anywhere. big_r is never forced, and ¬big_q(a) is forced at no world (w1 forces big_q(a)), so both hypotheses hold. At w0, neither big_p(a) nor big_q(a) is forced.

### fo-ordinary-57
- Item: Intuitionistic: ⊢ ∀x (big_f(a,x) ∨ ∀y big_f(x,y)) → ∃x1 ∀y1 big_f(x1,y1)   [ILTP v1.1.2 SYN073+1.p, Pelletier 50]
- Answer: invalid
- Justification: ILTP v1.1.2 Problems/SYN/SYN073+1.p records Status (intuit.): Non-Theorem (classically Theorem: either ∀y big_f(a,y), or some x has ¬big_f(a,x) and then ∀y big_f(x,y)). Countermodel: w0 ≤ w1, D(w0)={a}, D(w1)={a,b}. big_f(a,a) is forced at both worlds; big_f(b,a) and big_f(b,b) at w1; big_f(a,b) nowhere. The hypothesis holds: x = a by big_f(a,a), and x = b at w1 by ∀y big_f(b,y). The conclusion fails at w0: its only witness is a, and big_f(a,b) fails at w1.

### fo-ordinary-58
- Item: Intuitionistic: ⊢ ∀u ∀v ∀w (big_p(u,v) ∨ big_p(v,w)) → ∃x ∀y big_p(x,y)   [ILTP v1.1.2 SYN369+1.p]
- Answer: invalid
- Justification: ILTP v1.1.2 Problems/SYN/SYN369+1.p records Status (intuit.): Non-Theorem (classically Theorem). Countermodel: w0 ≤ w1, D(w0)={a}, D(w1)={a,b}. big_p(a,a) is forced at both worlds; big_p(b,a) and big_p(b,b) at w1; big_p(a,b) nowhere. The hypothesis holds: at w0 big_p(a,a); at w1, u = b, or v = a (u = a gives big_p(a,a)), or u = a, v = b (then big_p(b,w) holds for every w). The conclusion fails at w0: the only witness is a, and big_p(a,b) fails at w1.

### fo-ordinary-59
- Item: Classical: ⊢ ∃x ∀y (big_f(x) ↔ big_f(y))   [ILTP v1.1.2 SYN316+1.p]
- Answer: invalid
- Justification: ILTP v1.1.2 Problems/SYN/SYN316+1.p records Status: CounterSatisfiable and Status (intuit.): Non-Theorem. Classical countermodel: domain {a,b}, big_f = {a}. x = a fails at y = b, and x = b fails at y = a.

### fo-ordinary-60
- Item: Classical: ⊢ ∀x (r(x) ∨ (s(x) ↔ (r(x) ∧ s(x)))) → ∀x (r(x) → s(x))   [ILTP v1.1.2 SYN725+1.p]
- Answer: invalid
- Justification: ILTP v1.1.2 Problems/SYN/SYN725+1.p records Status: CounterSatisfiable and Status (intuit.): Non-Theorem. Classical countermodel: domain {a}, r(a) true, s(a) false. The hypothesis holds through r(a), but r(a) → s(a) fails.

## Dropped items

None.
