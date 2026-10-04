#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $! upright(w)^*$,
    rule(
      name: $⊗ upright(R)$,
      rule(name: $bold(1) upright(R)$, $⊢ bold(1)$),
      rule(name: $bold(1) upright(R)$, $⊢ bold(1)$),
      $⊢ bold(1) ⊗ bold(1)$,
    ),
    $!A, !B, !C ⊢ bold(1) ⊗ bold(1)$,
  ),
)
