# Corpus: first-order linear logic

A test set for later steps of linlog, in the notation of `plan/notes/research/fo-linear.md` (the specification). The question for each item: is the sequent provable?

Each answer was given by one reader and checked blind by a second reader of another model; where the two disagreed, a third decided by argument. No item was disputed, so no item carries a dispute note. 80 items: 42 provable, 38 unprovable, none dropped.

## quantifier basics

### fo-linear-01
- Item: `CLL (default classical reading): forall x. p(x) |- p(c)`
- Answer: provable
- Justification: One-sided |- exists x. ~p(x), p(c). The exists rule with witness c gives |- ~p(c), p(c), which is an axiom (E1 of fo-linear.md).

### fo-linear-02
- Item: `CLL: p(c) |- exists x. p(x)`
- Answer: provable
- Justification: One-sided |- ~p(c), exists x. p(x). The exists rule with witness c gives the axiom |- ~p(c), p(c).

### fo-linear-03
- Item: `CLL: forall x. p(x) |- exists x. p(x)`
- Answer: provable
- Justification: One-sided |- exists x. ~p(x), exists x. p(x). Apply exists to both with the same arbitrary term t (rule 2.1 allows any term), which gives the axiom |- ~p(t), p(t). The signature has no constant, so the search's two metavariables unify with each other and stay unbound. This item tests open question 3 (open metavariables in a found proof). Standard first-order sequent calculus proves the sequent because the domain is nonempty.

### fo-linear-04
- Item: `CLL: exists x. p(x) |- forall x. p(x)`
- Answer: unprovable
- Justification: Classical countermodel. Erasing the modalities and reading tensor/with as 'and', par/plus as 'or', -o as '->' and the units as truth values turns every CLL or ILL proof (affine and Mix too) into a classical one. Take domain {0,1} with p = {0}: the antecedent is true and the succedent is false. In search terms, the forall's eigenvariable is fresh and cannot equal the exists's eigenvariable.

### fo-linear-05
- Item: `ILL (-i): forall x. p(x) |- forall y. p(y)`
- Answer: provable
- Justification: forallR with eigenvariable a, then forallL with a: p(a) |- p(a). This is E2 of fo-linear.md. The forallL step must lie above the forallR step: opening it first would ask X(level 0) := a(level 1), which the level check refuses.

### fo-linear-06
- Item: `ILL (-i): exists x. p(x), forall y. (p(y) -o q) |- q`
- Answer: provable
- Justification: existsL with eigenvariable a: p(a), forall y.(p(y) -o q) |- q. forallL with a: p(a), p(a) -o q |- q. -oL: p(a) |- p(a) and q |- q. Each hypothesis is used exactly once.

### fo-linear-07
- Item: `ILL (-i): exists x. p(x), forall y. (p(y) -o q(y)) |- q(c)`
- Answer: unprovable
- Justification: Classical countermodel (ILL provability implies classical validity): domain {c,d} with p = q = {d}. exists x p(x) is true and forall y (p(y) -> q(y)) is true, but q(c) is false.

## quantifier alternation

### fo-linear-08
- Item: `CLL: exists x. forall y. q(x, y) |- forall y. exists x. q(x, y)`
- Answer: provable
- Justification: Two-sided: forallR with eigenvariable b, existsL with eigenvariable a, forallL with b, existsR with a, ending in q(a,b) |- q(a,b). Neither eigenvariable occurs in its rule's conclusion. In search, the exists metavariable is created after a and b, so it may take a.

### fo-linear-09
- Item: `CLL: forall y. exists x. q(x, y) |- exists x. forall y. q(x, y)`
- Answer: unprovable
- Justification: Classical countermodel: domain {0,1} with q(x,y) iff x = y. Every y has an x equal to it, but no x equals both 0 and 1. In search, the metavariable X is older than the forall's eigenvariable b, and the hypothesis's eigenvariable c (introduced for y := b) depends on b, so the identity would need X := c, which the level check refuses.

### fo-linear-10
- Item: `ILL (-i): forall x. exists y. r(x, y), forall x. forall y. (r(x, y) -o s(x)) |- forall x. s(x)`
- Answer: provable
- Justification: forallR with eigenvariable a. forallL on the first hypothesis with a gives exists y. r(a,y); existsL with fresh b gives r(a,b). forallL on the second hypothesis with a, b gives r(a,b) -o s(a). -oL: r(a,b) |- r(a,b) and s(a) |- s(a). Every hypothesis is used once.

### fo-linear-11
- Item: `ILL (-i): forall x. exists y. r(x, y), forall x. forall y. (r(x, y) -o s(y)) |- forall y. s(y)`
- Answer: unprovable
- Justification: Classical countermodel: domain {0,1}, r = {(0,0),(1,0)}, s = {0}. forall x exists y r(x,y) holds with y = 0. r(x,y) -> s(y) holds because every r-pair has y = 0 and s(0) is true. s(1) is false.

## drinker

### fo-linear-12
- Item: `CLL: |- exists x. forall y. (~p(x) par p(y))`
- Answer: unprovable
- Justification: The sequent has one formula and no tensor, with, unit or exponential, so every rule is unary. The proof is a single branch ending in one axiom |- L, ~L, and only one rule applies at each step. exists with some t gives |- forall y.(~p(t) par p(y)). forall with a fresh a (a not in t) gives |- ~p(t) par p(a). par gives |- ~p(t), p(a), and the axiom needs t = a, which is impossible. Classically the formula is valid, but only through contraction.

### fo-linear-13
- Item: `CLL: |- ?exists x. forall y. (~p(x) par p(y))`
- Answer: unprovable
- Justification: The only rules available are ?-weakening, ?-contraction, dereliction, exists, forall, par and the axiom. All are unary, so the proof is one branch ending in one axiom |- L, ~L. A derelicted copy is not a ?-formula, so it cannot be weakened: it must be decomposed into its two literals, which then reach the leaf. Hence exactly one copy is derelicted, and the leaf is |- ~p(t), p(a) with a fresh for a conclusion that contains t. So t is not a, and no axiom closes the leaf. Contraction alone does not prove the drinker; weakening of the literals is also needed (see items 15 and 16).

### fo-linear-14
- Item: `CLL --affine: |- exists x. forall y. (~p(x) par p(y))`
- Answer: unprovable
- Justification: Affine mode adds weakening but no contraction, and there is no ?. The single formula is instantiated once, which gives the literals ~p(t) and p(a) with a not in t. Weakening one of them leaves |- L alone, and weakening both leaves the empty sequent; neither is provable, since both are classically false. The axiom would need t = a, which is impossible.

### fo-linear-15
- Item: `CLL --affine: |- ?exists x. forall y. (~p(x) par p(y))`
- Answer: provable
- Justification: Contract the ?-formula into two copies. Derelict copy 1, apply exists with an arbitrary term t0, then forall with fresh a and par: |- ~p(t0), p(a), ?D. Derelict copy 2, apply exists with a (allowed, since a was introduced below), then forall with fresh b and par: |- ~p(t0), p(a), ~p(a), p(b). Affine weakening removes ~p(t0) and p(b), leaving the axiom p(a), ~p(a). t0 stays an open metavariable, because the sequent has no constant.

### fo-linear-16
- Item: `CLL: |- ?exists x. forall y. (?~p(x) par ?p(y))`
- Answer: provable
- Justification: Contract and derelict. Copy 1: exists with t0 (arbitrary), forall with fresh a, par: |- ?~p(t0), ?p(a), ?D. Derelict ?D, then exists with a, forall with fresh b, par: |- ?~p(t0), ?p(a), ?~p(a), ?p(b). Weaken ?~p(t0) and ?p(b), which are ?-formulas, so no affine mode is needed. Derelict the remaining two: |- p(a), ~p(a) is an axiom. Each eigenvariable is fresh at its step.

## quantifiers against connectives

### fo-linear-17
- Item: `CLL: forall x. (p(x) & q(x)) |- (forall x. p(x)) & (forall x. q(x))`
- Answer: provable
- Justification: Two-sided: &R splits into two branches. In each, forallR with fresh a, forallL with a, and &L1 (respectively &L2) close it: p(a) & q(a) |- p(a) and p(a) & q(a) |- q(a).

### fo-linear-18
- Item: `CLL: exists x. (p(x) + q(x)) |- (exists x. p(x)) + (exists x. q(x))`
- Answer: provable
- Justification: existsL with a, then +L into two branches. Branch p(a): +R1, existsR with a. Branch q(a): +R2, existsR with a.

### fo-linear-19
- Item: `CLL: (forall x. p(x)) * (forall x. q(x)) |- forall x. (p(x) * q(x))`
- Answer: provable
- Justification: forallR with a, then *L: forall x.p(x), forall x.q(x) |- p(a) * q(a). *R splits {forall x.p(x)} |- p(a) and {forall x.q(x)} |- q(a), and forallL with a closes both.

### fo-linear-20
- Item: `CLL: forall x. (p(x) * q(x)) |- (forall x. p(x)) * (forall x. q(x))`
- Answer: unprovable
- Justification: The sequent is classically valid, so the argument is linear. One-sided: |- exists x.(~p(x) par ~q(x)), (forall x.p(x)) * (forall x.q(x)). The tensor must split its context. If it splits before the exists is opened, the branch without the exists is |- forall x.p(x) or |- forall x.q(x) alone, which is unprovable. Otherwise exists with t and par come first, and the split sends ~p(t) to the forall x.p(x) branch (any other split leaves a branch with no matching literal). That branch opens the forall with a fresh a, and the axiom ~p(t), p(a) needs t = a, which the eigenvariable condition forbids. This is Lincoln and Shankar's forall/exists impermutability.

### fo-linear-21
- Item: `CLL: exists x. (p(x) * q(x)) |- (exists x. p(x)) * (exists x. q(x))`
- Answer: provable
- Justification: existsL with a, *L: p(a), q(a) |- (exists x.p(x)) * (exists x.q(x)). *R splits {p(a)} | {q(a)}, and existsR with a closes each branch.

### fo-linear-22
- Item: `CLL: (exists x. p(x)) * (exists x. q(x)) |- exists x. (p(x) * q(x))`
- Answer: unprovable
- Justification: Classical countermodel: domain {0,1}, p = {0}, q = {1}. exists x p and exists x q are true, but no x satisfies both.

### fo-linear-23
- Item: `CLL: (forall x. p(x)) + (forall x. q(x)) |- forall x. (p(x) + q(x))`
- Answer: provable
- Justification: forallR with a, then +L. Branch forall x.p(x): forallL with a and +R1. Branch forall x.q(x): forallL with a and +R2. forallR may equally come after +L in each branch.

### fo-linear-24
- Item: `CLL: forall x. (p(x) + q(x)) |- (forall x. p(x)) + (forall x. q(x))`
- Answer: unprovable
- Justification: Classical countermodel: domain {0,1}, p = {0}, q = {1}. Every x is in p or q, but neither p nor q holds everywhere.

### fo-linear-25
- Item: `CLL: exists x. (p(x) & q(x)) |- (exists x. p(x)) & (exists x. q(x))`
- Answer: provable
- Justification: existsL with a, then &R copies the context. Left branch: &L1, existsR with a. Right branch: &L2, existsR with a.

### fo-linear-26
- Item: `CLL: (exists x. p(x)) & (exists x. q(x)) |- exists x. (p(x) & q(x))`
- Answer: unprovable
- Justification: Classical countermodel: domain {0,1}, p = {0}, q = {1}. Both existentials hold, but no single x satisfies p and q.

### fo-linear-27
- Item: `CLL: forall x. (p(x) par q) |- (forall x. p(x)) par q`
- Answer: provable
- Justification: One-sided: |- exists x.(~p(x) * ~q), (forall x.p(x)) par q. Apply par, forall with fresh a, and exists with a: |- ~p(a) * ~q, p(a), q. The tensor splits {p(a)} | {q}, giving two axioms. This is the classical quantifier shift, which needs par on the right (compare item 28).

### fo-linear-28
- Item: `ILL (-i): forall x. (p(x) + q) |- (forall x. p(x)) + q`
- Answer: unprovable
- Justification: The rules that apply at the root are +R1, +R2 and forallL. +R2: after forallL t and +L, the branch p(t) |- q fails. +R1: with forallR a first, forallL t, +L, the branch q |- p(a) fails; with forallL t first, +L, the branch q |- forall x.p(x) fails. forallL t at the root, then +L: the branch p(t) |- (forall x.p(x)) + q fails, by +R1 (a fresh, a not equal to t) and by +R2. Intuitionistically this is the constant-domain schema, which also fails.

### fo-linear-29
- Item: `ILL (-i): forall x. (q -o p(x)) |- q -o forall x. p(x)`
- Answer: provable
- Justification: -oR: forall x.(q -o p(x)), q |- forall x.p(x). forallR with a, forallL with a: q -o p(a), q |- p(a). -oL: q |- q and p(a) |- p(a).

### fo-linear-30
- Item: `ILL (-i): exists x. (p(x) -o q) |- (forall x. p(x)) -o q`
- Answer: provable
- Justification: -oR: exists x.(p(x) -o q), forall x.p(x) |- q. existsL with a: p(a) -o q, forall x.p(x) |- q. -oL: forall x.p(x) |- p(a), closed by forallL with a, and q |- q.

### fo-linear-31
- Item: `ILL (-i): (forall x. p(x)) -o q |- exists x. (p(x) -o q)`
- Answer: unprovable
- Justification: -oL at the root leaves |- forall x.p(x) with an empty context, which is unprovable. existsR with t, then -oR: (forall x.p(x)) -o q, p(t) |- q. The only rule left is -oL, whose first premise is either |- forall x.p(x) (unprovable) or p(t) |- forall x.p(x), where forallR's fresh a differs from t, so p(t) |- p(a) fails. The one-sided CLL form fails in the same way, for want of contraction.

### fo-linear-32
- Item: `CLL: (forall x. p(x)) + (forall x. q(x)) |- forall x. p(x)`
- Answer: unprovable
- Justification: Classical countermodel: domain {0}, p empty, q = {0}. The antecedent is true through its right disjunct, and forall x p(x) is false.

## units

### fo-linear-33
- Item: `ILL (-i): exists x. 0 |- p(c)`
- Answer: provable
- Justification: existsL with fresh a gives 0 |- p(c), which 0L closes.

### fo-linear-34
- Item: `CLL: |- forall x. bot, p(c), ~p(c)`
- Answer: provable
- Justification: forall with fresh a gives |- bot, p(c), ~p(c). The bot rule gives |- p(c), ~p(c), an axiom.

### fo-linear-35
- Item: `CLL: |- forall x. bot`
- Answer: unprovable
- Justification: Classical reading: forall x false is false on any nonempty domain, so no mode proves it. In search, forall a then bot leaves the empty sequent, which is unprovable without Mix0.

### fo-linear-36
- Item: `ILL (-i): exists x. q(x), forall x. p(x) |- exists y. (p(y) * top)`
- Answer: provable
- Justification: existsL with a: q(a), forall x.p(x) |- exists y.(p(y) * top). existsR with a, then *R splits {forall x.p(x)} |- p(a), closed by forallL a, and {q(a)} |- top, closed by topR. The eigenvariable a ends up inside what top absorbs, which tests the checker's 'any' flag together with the eigenvariable rule of section 4.2.

## resources

### fo-linear-37
- Item: `ILL (-i): forall x. p(x) |- p(a) * p(b)`
- Answer: unprovable
- Justification: The sequent is classically valid, so the argument is a count. One-sided: |- exists x.~p(x), p(a) * p(b). This is MLL1 with no exponentials, so the exists is opened once and every literal sits in exactly one axiom that pairs p with ~p. There is one negative p-literal, ~p(t), and two positive ones, so no such pairing exists.

### fo-linear-38
- Item: `ILL (-i): forall x. (p(x) -o q(x)), p(a), p(b) |- q(a) * q(b)`
- Answer: unprovable
- Justification: The unbanged clause is used at most once. One-sided: |- exists x.(p(x) * ~q(x)), ~p(a), ~p(b), q(a) * q(b). This is MLL1: the exists is opened once and every literal is in exactly one axiom. There is one positive p-literal, p(t), and two negative ones, ~p(a) and ~p(b), so the literals cannot be paired.

### fo-linear-39
- Item: `ILL (-i): forall x. (p(x) & q(x)) |- p(a) * q(a)`
- Answer: unprovable
- Justification: *R splits the antecedents, and some branch gets none: the single hypothesis is not banged, and its instance p(t) & q(t) can be decomposed by &L to only one atom. That branch is |- p(a) or |- q(a) with an empty context, which is unprovable. The rule order does not matter, since forallL and &L before *R still leave a single atom.

### fo-linear-40
- Item: `ILL (-i): forall x. (p(x) & q(x)) |- p(a) & q(b)`
- Answer: provable
- Justification: &R copies the context. Left: forallL a, &L1, giving p(a) |- p(a). Right: forallL b, &L2, giving q(b) |- q(b). The two branches use different witnesses.

### fo-linear-41
- Item: `ILL (-i): forall x. (p(x) -o q(x)), p(a) + p(b) |- q(a) + q(b)`
- Answer: provable
- Justification: +L into two branches. Branch p(a): forallL a, -oL with p(a) |- p(a) and q(a) |- q(a) + q(b) closed by +R1. Branch p(b): forallL b, -oL, +R2. Each branch instantiates the clause once, with its own witness.

### fo-linear-42
- Item: `ILL (-i): forall x. forall y. (r(x, y) -o r(y, x)), r(a, b) |- r(b, a)`
- Answer: provable
- Justification: forallL with a, then b: r(a,b) -o r(b,a). -oL: r(a,b) |- r(a,b) and r(b,a) |- r(b,a).

### fo-linear-43
- Item: `ILL (-i): forall x. forall y. (r(x, y) -o r(y, x)), r(a, b) |- r(a, b)`
- Answer: unprovable
- Justification: The clause must be consumed. One-sided: |- exists x y.(r(x,y) * ~r(y,x)), ~r(a,b), r(a,b), with the exists opened once at (t1,t2). The tensor sends r(t1,t2) and ~r(t2,t1) to different branches. The branch with r(t1,t2) has only ~r(a,b) to match, so t1 = a and t2 = b. The other branch is then |- ~r(b,a), r(a,b), and a, b are distinct constants. If the context goes otherwise, a branch keeps a lone literal. In affine mode the sequent is provable by weakening the clause.

## exponentials

### fo-linear-44
- Item: `ILL (-i): !forall x. p(x) |- p(a) * p(b)`
- Answer: provable
- Justification: Contract !forall x.p(x), then *R gives one copy to each side. Derelict and forallL with a, respectively b.

### fo-linear-45
- Item: `ILL (-i): !forall x. p(x) |- forall x. !p(x)`
- Answer: provable
- Justification: forallR with a: !forall x.p(x) |- !p(a). !R applies because the context is all banged: !forall x.p(x) |- p(a). Dereliction, then forallL with a.

### fo-linear-46
- Item: `ILL (-i): forall x. !p(x) |- !forall x. p(x)`
- Answer: unprovable
- Justification: !R needs a banged context, and forall x.!p(x) is not banged, so forallL with some t comes first: !p(t) |- !forall x.p(x). Dereliction or weakening of !p(t) leaves an unprovable sequent: p(t) |- !forall x.p(x) blocks !R for good, and |- !forall x.p(x) ends in |- p(a). !R gives !p(t) |- forall x.p(x), and forallR's fresh a (not in t) leaves !p(t) |- p(a), which no structural rule closes. Contraction only adds copies of !p(t). This is the BRS24 counterexample.

### fo-linear-47
- Item: `CLL: |- ?exists x. ~p(x), p(a) * p(b)`
- Answer: provable
- Justification: Contract the ?-formula, and let the tensor send one copy to each side. Derelict, then exists with a (respectively b), giving two axioms.

### fo-linear-48
- Item: `CLL: |- ?exists x. ~p(x), !forall y. p(y)`
- Answer: provable
- Justification: Promotion applies because the other formula is a ?-formula: |- ?exists x.~p(x), forall y.p(y). forall with fresh a, derelict, and exists with a give |- ~p(a), p(a).

### fo-linear-49
- Item: `CLL: |- exists x. ?~p(x), !forall y. p(y)`
- Answer: unprovable
- Justification: Promotion is blocked while exists x.?~p(x), which is not a ?-formula, is present, so exists with some t comes first: |- ?~p(t), !forall y.p(y). Derelicting ?~p(t) before promotion blocks promotion forever. After promotion, forall's fresh a (not in t) leaves |- ?~p(t), p(a), and every use of ?~p(t) gives ~p(t), never ~p(a). This is the one-sided form of item 46.

### fo-linear-50
- Item: `CLL: !forall y. exists x. q(x, y) |- ?exists x. forall y. q(x, y)`
- Answer: unprovable
- Justification: Classical countermodel; the modalities are erased. Domain {0,1} with q(x,y) iff x = y: forall y exists x q holds, and exists x forall y q fails. Exponentials cannot rescue a classically invalid sequent.

## eigenvariable and Skolemization

### fo-linear-51
- Item: `CLL: |- exists x. (~r par ~q(x)), (forall y. q(y)) * r`
- Answer: unprovable
- Justification: LS94 section 2 (example U of fo-linear.md). A tensor before the exists sends the whole exists to one side, leaving |- forall y.q(y) or |- r alone. Otherwise exists with t and par come first, and the only split with no lone literal is {~q(t), forall y.q(y)} | {~r, r}. That branch needs t equal to the forall's fresh eigenvariable, which is impossible.

### fo-linear-52
- Item: `CLL: |- exists x. (~r par ~q(x)), q(c) * r`
- Answer: provable
- Justification: This is the statically Skolemized form of item 51. exists with c, par, then the tensor splits {~q(c)} | {~r} against q(c) and r, giving two axioms. Static Skolemization is unsound for LL, which is why this answer differs from item 51.

### fo-linear-53
- Item: `ILL (-i): forall x. (r * s(x)) |- r * forall u. s(u)`
- Answer: unprovable
- Justification: BRS24's example. The goal is a tensor, so the right rules start with *R. *R at the root sends the hypothesis to one side: the other side is |- r or |- forall u.s(u) with an empty context, which fails. forallL t and *L first give r, s(t) |- r * forall u.s(u), and *R must give r to r and s(t) to forall u.s(u), where forallR's fresh a differs from t, so s(t) |- s(a) fails. The sequent is classically valid.

### fo-linear-54
- Item: `CLL: |- forall y. (p(y) * q), exists x. ~p(x), ~q`
- Answer: provable
- Justification: forall with fresh a gives |- p(a) * q, exists x.~p(x), ~q. The tensor splits {exists x.~p(x)} | {~q}: |- p(a), exists x.~p(x) closes by exists with a, and |- q, ~q is an axiom. Opening the exists first fails, so the forall step must lie below the exists step (the forall/exists impermutability).

### fo-linear-55
- Item: `CLL: |- ~p(c) & ~p(d), exists x. p(x)`
- Answer: provable
- Justification: The & rule first, then exists with c in the left branch and with d in the right. One occurrence gets two witnesses, which is the exists/& impermutability (E5 of fo-linear.md). Opening the exists below the & would fix a single witness and fail.

## occurs check

### fo-linear-56
- Item: `CLL: forall x. p(x, f(x)) |- exists y. p(y, y)`
- Answer: unprovable
- Justification: Classical countermodel: domain N, f(n) = n+1, p(x,y) iff y = x+1. forall x p(x,f(x)) holds, and no y has y = y+1. In search, unifying p(X, f(X)) with p(Y, Y) gives Y = X and X = f(X), which the occurs check rejects. Without the occurs check, the search would wrongly prove the sequent.

### fo-linear-57
- Item: `CLL: forall x. p(x, f(x)) |- exists y. p(f(c), y)`
- Answer: provable
- Justification: existsR with f(f(c)) and forallL with f(c) give p(f(c), f(f(c))) |- p(f(c), f(f(c))). Unification binds X := f(c) and Y := f(X).

### fo-linear-58
- Item: `CLL: forall x. forall y. p(x, f(y), y) |- exists z. p(z, z, z)`
- Answer: unprovable
- Justification: Classical countermodel: domain N, f(n) = n+1, p(x,u,v) iff u = v+1. The hypothesis holds, and p(z,z,z) would need z = z+1. Unification gives X = Z, f(Y) = Z and Y = Z, hence Z = f(Z), which fails the occurs check.

### fo-linear-59
- Item: `ILL (-i): !forall x. eq(x, x) |- exists y. eq(y, f(y))`
- Answer: unprovable
- Justification: Classical countermodel: domain N, eq is equality, f(n) = n+1. Reflexivity holds, and no y equals y+1. This is the standard Prolog occurs-check example: without the check, Y = f(Y) unifies and the search proves the sequent wrongly.

## function symbols

### fo-linear-60
- Item: `ILL (-i): p(f(c), g(c)), forall x. forall y. (p(f(x), y) -o q(x, y)) |- q(c, g(c))`
- Answer: provable
- Justification: forallL with x := c and y := g(c): p(f(c), g(c)) -o q(c, g(c)). -oL closes with p(f(c),g(c)) |- p(f(c),g(c)) and q(c,g(c)) |- q(c,g(c)). Every hypothesis is used once.

### fo-linear-61
- Item: `ILL (-i): p(f(c), g(c)), forall x. (p(f(x), g(x)) -o q(x)) |- q(d)`
- Answer: unprovable
- Justification: Classical countermodel: domain N, c = 0, d = 1, f(n) = 2n+2, g(n) = 2n+3, p(u,v) iff u = 2 and v = 3, q = {0}. p(f(0), g(0)) = p(2,3) is true. p(f(n), g(n)) holds only at n = 0, where q(0) is true, so the clause holds. q(1) is false.

## first-order Horn

### fo-linear-62
- Item: `ILL (-i): !forall x. (nat(x) -o nat(s(x))), nat(z) |- nat(s(s(s(z))))`
- Answer: provable
- Justification: Use three copies of the clause, with x := s(s(z)), s(z) and z. Each -oL takes its premise nat(...) from the previous step, and the first takes nat(z). The left-over !clause is weakened.

### fo-linear-63
- Item: `ILL (-i): forall x. (nat(x) -o nat(s(x))), nat(z) |- nat(s(s(z)))`
- Answer: unprovable
- Justification: The clause is unbanged, so forallL t then -oL is the only possible use. Its second premise, Delta2, nat(s(t)) |- nat(s(s(z))) with Delta2 a subset of {nat(z)}, needs Delta2 empty and t = s(z). The first premise is then nat(z) |- nat(s(z)), which fails. The axiom nat(z) |- nat(s(s(z))) also fails. Two applications would need contraction.

### fo-linear-64
- Item: `CLL: forall x. (nat(x) -o nat(s(x))), nat(z) |- nat(z)`
- Answer: unprovable
- Justification: One-sided: |- exists x.(nat(x) * ~nat(s(x))), ~nat(z), nat(z). The exists is opened once, at t. The tensor separates nat(t) from ~nat(s(t)), so ~nat(s(t)) needs a positive partner other than nat(t). The only one is nat(z), and z differs from s(t). The clause cannot be left unused without weakening. A link of nat(t) with ~nat(s(t)) would also fail the occurs check (t = s(t)).

### fo-linear-65
- Item: `CLL --affine: forall x. (nat(x) -o nat(s(x))), nat(z) |- nat(z)`
- Answer: provable
- Justification: Weaken the clause, and the axiom ~nat(z), nat(z) closes. Affine mode changes the answer of item 64.

### fo-linear-66
- Item: `ILL (-i): !forall x. forall y. (e(x, y) -o path(x, y)), !forall x. forall y. forall w. ((e(x, y) * path(y, w)) -o path(x, w)), e(a, b), e(b, c) |- path(a, c)`
- Answer: provable
- Justification: Derelict clause 2 with x,y,w := a,b,c. -oL: the first premise is e(a,b), e(b,c), !C1 |- e(a,b) * path(b,c). *R gives e(a,b) |- e(a,b) and e(b,c), !C1 |- path(b,c), which closes by derelicting C1 with b,c and -oL. The second premise is path(a,c) |- path(a,c). The banged clauses are weakened, and both edges are consumed exactly once.

### fo-linear-67
- Item: `ILL (-i): !forall x. forall y. (e(x, y) -o path(x, y)), !forall x. forall y. forall w. ((e(x, y) * path(y, w)) -o path(x, w)), e(a, b), e(b, c) |- path(a, c) * e(a, b) * 0 ... (see note) -- the sequent is: same program and facts |- path(a, b)`
- Answer: unprovable
- Justification: The sequent is the program and facts of item 66 with goal path(a, b). Linear Horn sequents (banged clauses whose body and head are tensors of atoms, atomic facts, atomic goal) are provable exactly when the facts rewrite into exactly the goal multiset by ground clause instances (HM94; the Petri-net reading behind linlog's Horn engine). Let phi(c) = 1 and phi(t) = 0 for every other term, and weigh e(x,y) and path(x,y) by phi(y) - phi(x). Every instance of both clauses preserves the total weight: (phi(y)-phi(x)) + (phi(w)-phi(y)) = phi(w)-phi(x). The facts weigh 0 + 1 = 1 and the goal weighs 0. The edge e(b,c) cannot be consumed.
- Note: the item text is copied exactly as received and contains stray text ("path(a, c) * e(a, b) * 0 ... (see note) -- the sequent is: same program and facts"). The intended sequent is the program and facts of item 66 with goal path(a, b), as the justification says. Not a dispute over the answer.

### fo-linear-68
- Item: `ILL -i --affine: the program and facts of item 66 (!forall x. forall y. (e(x, y) -o path(x, y)), !forall x. forall y. forall w. ((e(x, y) * path(y, w)) -o path(x, w)), e(a, b), e(b, c)) |- path(a, b)`
- Answer: provable
- Justification: Derelict clause 1 with a,b, then -oL with e(a,b) |- e(a,b) and path(a,b) |- path(a,b). The left-over fact e(b,c) is weakened, which affine mode allows. Affine mode changes the answer of item 67.

### fo-linear-69
- Item: `ILL (-i): !forall x. add(z, x, x), !forall x. forall y. forall w. (add(x, y, w) -o add(s(x), y, s(w))) |- add(s(s(z)), s(z), s(s(s(z))))`
- Answer: provable
- Justification: Clause 2 with x,y,w := s(z), s(z), s(s(z)) reduces the goal to add(s(z), s(z), s(s(z))). Clause 2 with z, s(z), s(z) reduces that to add(z, s(z), s(z)). Clause 1 with x := s(z) closes it. The left-over banged clauses are weakened.

### fo-linear-70
- Item: `ILL (-i): !forall x. add(z, x, x), !forall x. forall y. forall w. (add(x, y, w) -o add(s(x), y, s(w))) |- add(s(s(z)), s(z), s(s(z)))`
- Answer: unprovable
- Justification: Classical countermodel: N with z = 0, s = successor and add(x,y,w) iff x+y = w. Both clauses hold (0+x = x, and x+y = w implies (x+1)+y = w+1), and 2+1 = 2 is false.

### fo-linear-71
- Item: `ILL (-i): !forall x. add(z, x, x), !forall x. forall y. forall w. (add(x, y, w) -o add(s(x), y, s(w))) |- exists x. add(x, s(z), z)`
- Answer: unprovable
- Justification: Classical countermodel: the standard model of item 70, where no natural x has x+1 = 0. In search, every clause head unifies with add(X, s(z), z) only through z = s(W), which is a clash, or through clause 1 with s(z) = z, which is also a clash.

### fo-linear-72
- Item: `ILL (-i): !forall x. (p(x) -o p(f(x))), p(c) |- exists x. p(f(f(x)))`
- Answer: provable
- Justification: existsR with c gives p(f(f(c))). Derelict the clause with x := f(c), then -oL: the goal needs p(f(c)). Derelict again with x := c: p(c) |- p(c). The unused !clause is weakened. In search, X is bound to c only after two copies.

### fo-linear-73
- Item: `ILL (-i): !forall x. (p(f(x)) -o p(x)), p(c) |- p(f(c))`
- Answer: unprovable
- Justification: Classical countermodel: domain N, c = 0, f(n) = n+1, p = {0}. p(n+1) is always false, so the clause holds, and p(0) is true while p(1) is false. The clause only rewrites downwards.

### fo-linear-80
- Item: `ILL (-i): !forall x. add(z, x, x), !forall x. forall y. forall w. (add(x, y, w) -o add(s(x), y, s(w))) |- exists w. add(s(z), s(z), w)`
- Answer: provable
- Justification: existsR with s(s(z)). Clause 2 with x,y,w := z, s(z), s(z) reduces the goal to add(z, s(z), s(z)), which clause 1 with x := s(z) closes. In search, W is bound to s(W') and then W' := s(z), giving the computed answer 1+1 = 2.

## modes

### fo-linear-74
- Item: `CLL: |- p(a) par q(b), exists x. ~p(x), exists y. ~q(y)`
- Answer: unprovable
- Justification: The sequent has no tensor, with, unit or exponential, so every rule is unary. The proof is one branch ending in one axiom |- L, ~L. par and exists keep every subformula, so all four literals p(a), q(b), ~p(t1), ~q(t2) reach the leaf, and an axiom holds only two. The sequent is classically valid.

### fo-linear-75
- Item: `CLL --mix: |- p(a) par q(b), exists x. ~p(x), exists y. ~q(y)`
- Answer: provable
- Justification: par gives |- p(a), q(b), exists x.~p(x), exists y.~q(y). Mix splits it into |- p(a), exists x.~p(x), closed by exists with a, and |- q(b), exists y.~q(y), closed by exists with b. Mix changes the answer of item 74.

### fo-linear-76
- Item: `CLL --affine: forall x. p(x) |- p(a) * p(b)`
- Answer: unprovable
- Justification: One-sided: |- exists x.~p(x), p(a) * p(b). Affine mode adds weakening, not contraction, so the exists (or its single instance ~p(t)) goes to one side of the tensor. The other branch is |- p(a) or |- p(b) alone, which is classically false (domain {0}, p empty) and so unprovable in every mode. Weakening does not supply a second copy.

### fo-linear-77
- Item: `CLL: forall x. (p(x) * q(x)) |- forall x. p(x)`
- Answer: unprovable
- Justification: One-sided: |- exists x.(~p(x) par ~q(x)), forall x.p(x). There are no exponentials, so the exists is opened once, and q occurs in only one literal, the negative ~q(t). With no weakening, every literal needs an axiom partner, and ~q(t) has none.

### fo-linear-78
- Item: `CLL --affine: forall x. (p(x) * q(x)) |- forall x. p(x)`
- Answer: provable
- Justification: forall with fresh a, exists with a, par: |- ~p(a), ~q(a), p(a). Weaken ~q(a), and the axiom closes. Affine mode changes the answer of item 77.

### fo-linear-79
- Item: `CLL --affine: forall x. (p(x) * q(x)) |- (forall x. p(x)) * (forall x. q(x))`
- Answer: unprovable
- Justification: If the tensor splits before the exists is opened, one branch is |- forall x.p(x) or |- forall x.q(x) alone, which is classically false (p and q empty). If the exists is opened first with t (par optional), the branch with forall x.p(x) holds at most ~p(t), ~q(t), or their par. In the model with domain {0,1}, every function symbol mapped to 0, p = {0} and q = {0,1}, that branch reads not p(0) or not q(0) or forall x p(x), which is false, so it is unprovable even with weakening. Affine mode does not change the answer of item 20.

## Dropped items

None.
