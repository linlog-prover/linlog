#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $"wk"$,
    rule(name: $"ax"$, $⊢ A^⊥, A$),
    $⊢ A^⊥, B^⊥, A$,
  ),
)
