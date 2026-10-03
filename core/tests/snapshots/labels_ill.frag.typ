#prooftree(
  rule(
    name: $\&_(upright(R))$,
    rule(
      name: $\&_(upright(R))$,
      rule(
        name: $\&_(upright(R))$,
        rule(
          name: $⊸_(upright(R))$,
          rule(
            name: $⊸_(upright(R))$,
            rule(
              name: $⊸_(upright(L))$,
              rule(name: $"ax"$, $A ⊢ A$),
              rule(name: $"ax"$, $B ⊢ B$),
              $A, A ⊸ B ⊢ B$,
            ),
            $A ⊢ (A ⊸ B) ⊸ B$,
          ),
          $⊢ A ⊸ ((A ⊸ B) ⊸ B)$,
        ),
        rule(
          name: $⊸_(upright(R))$,
          rule(
            name: $⊗_(upright(L))$,
            rule(
              name: $⊗_(upright(R))$,
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
        name: $\&_(upright(R))$,
        rule(
          name: $⊸_(upright(R))$,
          rule(
            name: $\&_("L1")$,
            rule(name: $"ax"$, $A ⊢ A$),
            $A class("binary", \&) B ⊢ A$,
          ),
          $⊢ (A class("binary", \&) B) ⊸ A$,
        ),
        rule(
          name: $⊸_(upright(R))$,
          rule(
            name: $\&_("L2")$,
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
      name: $\&_(upright(R))$,
      rule(
        name: $\&_(upright(R))$,
        rule(
          name: $⊸_(upright(R))$,
          rule(
            name: $⊕_(upright(L))$,
            rule(
              name: $⊕_("R2")$,
              rule(name: $"ax"$, $A ⊢ A$),
              $A ⊢ B ⊕ A$,
            ),
            rule(
              name: $⊕_("R1")$,
              rule(name: $"ax"$, $B ⊢ B$),
              $B ⊢ B ⊕ A$,
            ),
            $A ⊕ B ⊢ B ⊕ A$,
          ),
          $⊢ (A ⊕ B) ⊸ (B ⊕ A)$,
        ),
        rule(
          name: $⊸_(upright(R))$,
          rule(
            name: $bold(1)_(upright(L))$,
            rule(name: $bold(1)_(upright(R))$, $⊢ bold(1)$),
            $bold(1) ⊢ bold(1)$,
          ),
          $⊢ bold(1) ⊸ bold(1)$,
        ),
        $⊢ ((A ⊕ B) ⊸ (B ⊕ A)) class("binary", \&) (bold(1) ⊸ bold(1))$,
      ),
      rule(
        name: $\&_(upright(R))$,
        rule(
          name: $\&_(upright(R))$,
          rule(
            name: $⊸_(upright(R))$,
            rule(name: $0_(upright(L))$, $0 ⊢ A$),
            $⊢ 0 ⊸ A$,
          ),
          rule(name: $⊤_(upright(R))$, $⊢ ⊤$),
          $⊢ (0 ⊸ A) class("binary", \&) ⊤$,
        ),
        rule(
          name: $\&_(upright(R))$,
          rule(
            name: $⊸_(upright(R))$,
            rule(
              name: $!_(upright(c))$,
              rule(
                name: $⊗_(upright(R))$,
                rule(
                  name: $!_(upright(L))$,
                  rule(name: $"ax"$, $A ⊢ A$),
                  $!A ⊢ A$,
                ),
                rule(
                  name: $!_(upright(L))$,
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
            name: $\&_(upright(R))$,
            rule(
              name: $⊸_(upright(R))$,
              rule(
                name: $!_(upright(R))$,
                rule(
                  name: $!_(upright(L))$,
                  rule(name: $"ax"$, $A ⊢ A$),
                  $!A ⊢ A$,
                ),
                $!A ⊢ !A$,
              ),
              $⊢ !A ⊸ !A$,
            ),
            rule(
              name: $⊸_(upright(R))$,
              rule(
                name: $!_(upright(w))$,
                rule(name: $bold(1)_(upright(R))$, $⊢ bold(1)$),
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
