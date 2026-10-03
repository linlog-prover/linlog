#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $\& upright(R)$,
    rule(
      name: $\& upright(R)$,
      rule(
        name: $\& upright(R)$,
        rule(
          name: $⊸ upright(R)$,
          rule(
            name: $⊸ upright(R)$,
            rule(
              name: $⊸ upright(L)$,
              rule(name: $"ax"$, $A ⊢ A$),
              rule(name: $"ax"$, $B ⊢ B$),
              $A, A ⊸ B ⊢ B$,
            ),
            $A ⊢ (A ⊸ B) ⊸ B$,
          ),
          $⊢ A ⊸ ((A ⊸ B) ⊸ B)$,
        ),
        rule(
          name: $⊸ upright(R)$,
          rule(
            name: $⊗ upright(L)$,
            rule(
              name: $⊗ upright(R)$,
              rule(name: $"ax"$, $B ⊢ B$),
              rule(name: $"ax"$, $A ⊢ A$),
              $A, B ⊢ B ⊗ A$,
            ),
            $A ⊗ B ⊢ B ⊗ A$,
          ),
          $⊢ (A ⊗ B) ⊸ (B ⊗ A)$,
        ),
        $⊢ (A ⊸ ((A ⊸ B) ⊸ B)) class("binary", \&) ((A ⊗ B) ⊸ (B ⊗ A))$,
      ),
      rule(
        name: $\& upright(R)$,
        rule(
          name: $⊸ upright(R)$,
          rule(
            name: $\& upright(L)_1$,
            rule(name: $"ax"$, $A ⊢ A$),
            $A class("binary", \&) B ⊢ A$,
          ),
          $⊢ (A class("binary", \&) B) ⊸ A$,
        ),
        rule(
          name: $⊸ upright(R)$,
          rule(
            name: $\& upright(L)_2$,
            rule(name: $"ax"$, $B ⊢ B$),
            $A class("binary", \&) B ⊢ B$,
          ),
          $⊢ (A class("binary", \&) B) ⊸ B$,
        ),
        $⊢ ((A class("binary", \&) B) ⊸ A) class("binary", \&) ((A class("binary", \&) B) ⊸ B)$,
      ),
      $⊢ ((A ⊸ ((A ⊸ B) ⊸ B)) class("binary", \&) ((A ⊗ B) ⊸ (B ⊗ A))) class("binary", \&) (((A class("binary", \&) B) ⊸ A) class("binary", \&) ((A class("binary", \&) B) ⊸ B))$,
    ),
    rule(
      name: $\& upright(R)$,
      rule(
        name: $\& upright(R)$,
        rule(
          name: $⊸ upright(R)$,
          rule(
            name: $⊕ upright(L)$,
            rule(
              name: $⊕ upright(R)_2$,
              rule(name: $"ax"$, $A ⊢ A$),
              $A ⊢ B ⊕ A$,
            ),
            rule(
              name: $⊕ upright(R)_1$,
              rule(name: $"ax"$, $B ⊢ B$),
              $B ⊢ B ⊕ A$,
            ),
            $A ⊕ B ⊢ B ⊕ A$,
          ),
          $⊢ (A ⊕ B) ⊸ (B ⊕ A)$,
        ),
        rule(
          name: $⊸ upright(R)$,
          rule(
            name: $bold(1) upright(L)$,
            rule(name: $bold(1) upright(R)$, $⊢ bold(1)$),
            $bold(1) ⊢ bold(1)$,
          ),
          $⊢ bold(1) ⊸ bold(1)$,
        ),
        $⊢ ((A ⊕ B) ⊸ (B ⊕ A)) class("binary", \&) (bold(1) ⊸ bold(1))$,
      ),
      rule(
        name: $\& upright(R)$,
        rule(
          name: $\& upright(R)$,
          rule(
            name: $⊸ upright(R)$,
            rule(name: $0 upright(L)$, $0 ⊢ A$),
            $⊢ 0 ⊸ A$,
          ),
          rule(name: $⊤ upright(R)$, $⊢ ⊤$),
          $⊢ (0 ⊸ A) class("binary", \&) ⊤$,
        ),
        rule(
          name: $\& upright(R)$,
          rule(
            name: $⊸ upright(R)$,
            rule(
              name: $! upright(c)$,
              rule(
                name: $⊗ upright(R)$,
                rule(
                  name: $! upright(L)$,
                  rule(name: $"ax"$, $A ⊢ A$),
                  $!A ⊢ A$,
                ),
                rule(
                  name: $! upright(L)$,
                  rule(name: $"ax"$, $A ⊢ A$),
                  $!A ⊢ A$,
                ),
                $!A, !A ⊢ A ⊗ A$,
              ),
              $!A ⊢ A ⊗ A$,
            ),
            $⊢ !A ⊸ (A ⊗ A)$,
          ),
          rule(
            name: $\& upright(R)$,
            rule(
              name: $⊸ upright(R)$,
              rule(
                name: $! upright(R)$,
                rule(
                  name: $! upright(L)$,
                  rule(name: $"ax"$, $A ⊢ A$),
                  $!A ⊢ A$,
                ),
                $!A ⊢ !A$,
              ),
              $⊢ !A ⊸ !A$,
            ),
            rule(
              name: $⊸ upright(R)$,
              rule(
                name: $! upright(w)$,
                rule(name: $bold(1) upright(R)$, $⊢ bold(1)$),
                $!A ⊢ bold(1)$,
              ),
              $⊢ !A ⊸ bold(1)$,
            ),
            $⊢ (!A ⊸ !A) class("binary", \&) (!A ⊸ bold(1))$,
          ),
          $⊢ (!A ⊸ (A ⊗ A)) class("binary", \&) ((!A ⊸ !A) class("binary", \&) (!A ⊸ bold(1)))$,
        ),
        $⊢ ((0 ⊸ A) class("binary", \&) ⊤) class("binary", \&) ((!A ⊸ (A ⊗ A)) class("binary", \&) ((!A ⊸ !A) class("binary", \&) (!A ⊸ bold(1))))$,
      ),
      $⊢ (((A ⊕ B) ⊸ (B ⊕ A)) class("binary", \&) (bold(1) ⊸ bold(1))) class("binary", \&) (((0 ⊸ A) class("binary", \&) ⊤) class("binary", \&) ((!A ⊸ (A ⊗ A)) class("binary", \&) ((!A ⊸ !A) class("binary", \&) (!A ⊸ bold(1)))))$,
    ),
    $⊢ (((A ⊸ ((A ⊸ B) ⊸ B)) class("binary", \&) ((A ⊗ B) ⊸ (B ⊗ A))) class("binary", \&) (((A class("binary", \&) B) ⊸ A) class("binary", \&) ((A class("binary", \&) B) ⊸ B))) class("binary", \&) ((((A ⊕ B) ⊸ (B ⊕ A)) class("binary", \&) (bold(1) ⊸ bold(1))) class("binary", \&) (((0 ⊸ A) class("binary", \&) ⊤) class("binary", \&) ((!A ⊸ (A ⊗ A)) class("binary", \&) ((!A ⊸ !A) class("binary", \&) (!A ⊸ bold(1))))))$,
  ),
)
