// provable (IMLL, intuitionistic, two-sided engine)
#prooftree(
  rule(
    name: $⊸ upright(L)$,
    rule(name: $"ax"$, $A ⊢ A$),
    rule(name: $"ax"$, $B ⊢ B$),
    $A, A ⊸ B ⊢ B$,
  ),
)
