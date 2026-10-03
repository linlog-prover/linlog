#prooftree(
  rule(
    name: $⊸ upright(L)$,
    rule(name: $"ax"$, $A ⊢ A$),
    grid(align: center, inset: (top: 0.3em), grid.hline(stroke: (dash: "dashed")), $B ⊢ B$),
    $A, A ⊸ B ⊢ B$,
  ),
)
