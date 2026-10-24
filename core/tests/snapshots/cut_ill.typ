#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $"cut"$,
    rule(
      name: $⊸ upright(R)$,
      rule(
        name: $⊸ upright(L)$,
        rule(name: $"ax"$, $A ⊢ A$),
        rule(name: $"ax"$, $B ⊢ B$),
        $A ⊸ B, A ⊢ B$,
      ),
      $A ⊸ B ⊢ A ⊸ B$,
    ),
    rule(
      name: $⊸ upright(L)$,
      rule(name: $"ax"$, $A ⊢ A$),
      rule(name: $"ax"$, $B ⊢ B$),
      $A, A ⊸ B ⊢ B$,
    ),
    $A, A ⊸ B ⊢ B$,
  ),
)
