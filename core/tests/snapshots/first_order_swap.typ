#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $forall$,
    rule(
      name: $forall$,
      rule(
        name: $exists$,
        rule(
          name: $exists$,
          rule(name: $"ax"$, $⊢ p(a, b)^⊥, p(a, b)$),
          $⊢ exists x. p(x, b), p(a, b)^⊥$,
        ),
        $⊢ exists y. p(a, y)^⊥, exists x. p(x, b)$,
      ),
      $⊢ forall y. (exists x. p(x, y)), exists y. p(a, y)^⊥$,
    ),
    $⊢ forall x. (exists y. p(x, y)^⊥), forall y. (exists x. p(x, y))$,
  ),
)
