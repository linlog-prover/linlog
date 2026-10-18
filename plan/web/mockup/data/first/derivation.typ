// provable (MLL, classical, focus engine)
#prooftree(
  rule(
    name: $⊗$,
    rule(name: $"ax"$, $⊢ A^⊥, A$),
    rule(name: $"ax"$, $⊢ B^⊥, B$),
    $⊢ A^⊥, A ⊗ B^⊥, B$,
  ),
)
