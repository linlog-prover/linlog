#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $backslash upright(L)$,
    rule(
      name: $slash upright(L)$,
      rule(name: $"ax"$, $b ⊢ b$),
      rule(name: $"ax"$, $c ⊢ c$),
      $c class("binary", slash) b, b ⊢ c$,
    ),
    rule(name: $"ax"$, $a ⊢ a$),
    $c class("binary", slash) b, a, a class("binary", backslash) b ⊢ c$,
  ),
)
