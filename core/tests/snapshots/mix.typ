#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $"mix"$,
    rule(name: $"ax"$, $⊢ A^⊥, A$),
    rule(name: $"ax"$, $⊢ B^⊥, B$),
    $⊢ A^⊥, B^⊥, A, B$,
  ),
)
