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
          rule(name: $"ax"$, $r(b, a) ⊢ r(b, a)$),
          $r(b, a) ⊢ exists y. r(b, y)$,
        ),
        $forall x. r(x, a) ⊢ exists y. r(b, y)$,
      ),
      $forall x. r(x, a) ⊢ forall x. (exists y. r(x, y))$,
    ),
    $exists y. (forall x. r(x, y)) ⊢ forall x. (exists y. r(x, y))$,
  ),
)
