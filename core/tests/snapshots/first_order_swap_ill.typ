#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $exists upright(L)$,
    rule(
      name: $forall upright(R)$,
      rule(
        name: $forall upright(L)$,
        rule(
          name: $exists upright(R)$,
          rule(name: $"ax"$, $p(a, b) ⊢ p(a, b)$),
          $p(a, b) ⊢ exists x. p(x, b)$,
        ),
        $forall y. p(a, y) ⊢ exists x. p(x, b)$,
      ),
      $forall y. p(a, y) ⊢ forall y. (exists x. p(x, y))$,
    ),
    $exists x. (forall y. p(x, y)) ⊢ forall y. (exists x. p(x, y))$,
  ),
)
