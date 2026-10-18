// provable (MELL, classical, focus engine)
#prooftree(
  rule(
    name: $class("normal", ?) upright(c)$,
    rule(
      name: $⊗$,
      rule(
        name: $class("normal", ?) upright(d)$,
        rule(name: $"ax"$, $⊢ A^⊥, A$),
        $⊢ class("normal", ?)A^⊥, A$,
      ),
      rule(
        name: $!$,
        rule(
          name: $class("normal", ?) upright(d)$,
          rule(name: $"ax"$, $⊢ A^⊥, A$),
          $⊢ class("normal", ?)A^⊥, A$,
        ),
        $⊢ class("normal", ?)A^⊥, !A$,
      ),
      $⊢ class("normal", ?)A^⊥, class("normal", ?)A^⊥, A ⊗ !A$,
    ),
    $⊢ class("normal", ?)A^⊥, A ⊗ !A$,
  ),
)
