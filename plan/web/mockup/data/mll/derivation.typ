// provable (MLL, classical, focus engine)
#prooftree(
  rule(
    name: $⅋$,
    rule(
      name: $⊗$,
      rule(name: $"ax"$, $⊢ B^⊥, B$),
      rule(name: $"ax"$, $⊢ A^⊥, A$),
      $⊢ A^⊥, B^⊥, B ⊗ A$,
    ),
    $⊢ A^⊥ ⅋ B^⊥, B ⊗ A$,
  ),
)
