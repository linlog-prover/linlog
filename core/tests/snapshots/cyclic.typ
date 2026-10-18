#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $⅋$,
    rule(
      name: $⊗$,
      rule(name: $"ax"$, $⊢ c^⊥, c$),
      rule(
        name: $⊗$,
        rule(name: $"ax"$, $⊢ a^⊥, a$),
        rule(name: $"ax"$, $⊢ b^⊥, b$),
        $⊢ b^⊥, a^⊥, a ⊗ b$,
      ),
      $⊢ b^⊥, a^⊥, c^⊥, c ⊗ (a ⊗ b)$,
    ),
    $⊢ b^⊥ ⅋ a^⊥, c^⊥, c ⊗ (a ⊗ b)$,
  ),
)
