#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $→ upright(R)$,
    rule(
      name: $→ upright(L)$,
      rule(
        name: $→ upright(L)$,
        rule(name: $"ax"$, $a ⊢ a$),
        rule(name: $"ax"$, $b ⊢ b$),
        $a, a → b ⊢ b$,
      ),
      rule(name: $"ax"$, $c ⊢ c$),
      $a, a → b, b → c ⊢ c$,
    ),
    $a → b, b → c ⊢ a → c$,
  ),
)
