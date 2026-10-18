// valid (classical logic by the affine translation into affine MALL, focus engine)
#prooftree(
  rule(
    name: $→ upright(R)$,
    rule(
      name: $→ upright(L)$,
      rule(
        name: $→ upright(R)$,
        rule(
          name: $"WR"$,
          rule(name: $"ax"$, $a ⊢ a$),
          $a ⊢ b, a$,
        ),
        $⊢ a → b, a$,
      ),
      rule(name: $"ax"$, $a ⊢ a$),
      $(a → b) → a ⊢ a$,
    ),
    $⊢ ((a → b) → a) → a$,
  ),
)
