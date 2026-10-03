#import "@preview/curryst:0.6.0": prooftree, rule
#set page(width: auto, height: auto, margin: 5pt)

#prooftree(
  rule(
    name: $\&$,
    rule(
      name: $\&$,
      rule(
        name: $\&$,
        rule(
          name: $⅋$,
          rule(
            name: $⅋$,
            rule(
              name: $⊗$,
              rule(name: $"ax"$, $⊢ A^⊥, A$),
              rule(name: $"ax"$, $⊢ B^⊥, B$),
              $⊢ A^⊥, B^⊥, A ⊗ B$,
            ),
            $⊢ A^⊥ ⅋ B^⊥, A ⊗ B$,
          ),
          $⊢ (A^⊥ ⅋ B^⊥) ⅋ (A ⊗ B)$,
        ),
        rule(name: $bold(1)$, $⊢ bold(1)$),
        $⊢ ((A^⊥ ⅋ B^⊥) ⅋ (A ⊗ B)) class("binary", \&) bold(1)$,
      ),
      rule(
        name: $\&$,
        rule(
          name: $⅋$,
          rule(
            name: $⊥$,
            rule(name: $bold(1)$, $⊢ bold(1)$),
            $⊢ ⊥, bold(1)$,
          ),
          $⊢ ⊥ ⅋ bold(1)$,
        ),
        rule(name: $⊤$, $⊢ ⊤$),
        $⊢ (⊥ ⅋ bold(1)) class("binary", \&) ⊤$,
      ),
      $⊢ (((A^⊥ ⅋ B^⊥) ⅋ (A ⊗ B)) class("binary", \&) bold(1)) class("binary", \&) ((⊥ ⅋ bold(1)) class("binary", \&) ⊤)$,
    ),
    rule(
      name: $\&$,
      rule(
        name: $\&$,
        rule(
          name: $⊕_1$,
          rule(name: $bold(1)$, $⊢ bold(1)$),
          $⊢ bold(1) ⊕ 0$,
        ),
        rule(
          name: $⊕_2$,
          rule(name: $bold(1)$, $⊢ bold(1)$),
          $⊢ 0 ⊕ bold(1)$,
        ),
        $⊢ (bold(1) ⊕ 0) class("binary", \&) (0 ⊕ bold(1))$,
      ),
      rule(
        name: $\&$,
        rule(
          name: $\&$,
          rule(
            name: $!$,
            rule(name: $bold(1)$, $⊢ bold(1)$),
            $⊢ !bold(1)$,
          ),
          rule(
            name: $class("normal", ?) upright(d)$,
            rule(name: $bold(1)$, $⊢ bold(1)$),
            $⊢ class("normal", ?)bold(1)$,
          ),
          $⊢ !bold(1) class("binary", \&) class("normal", ?)bold(1)$,
        ),
        rule(
          name: $\&$,
          rule(
            name: $⅋$,
            rule(
              name: $class("normal", ?) upright(w)$,
              rule(name: $bold(1)$, $⊢ bold(1)$),
              $⊢ class("normal", ?)A, bold(1)$,
            ),
            $⊢ class("normal", ?)A ⅋ bold(1)$,
          ),
          rule(
            name: $⅋$,
            rule(
              name: $class("normal", ?) upright(c)$,
              rule(
                name: $⊗$,
                rule(
                  name: $class("normal", ?) upright(d)$,
                  rule(name: $"ax"$, $⊢ A^⊥, A$),
                  $⊢ class("normal", ?)A^⊥, A$,
                ),
                rule(
                  name: $class("normal", ?) upright(d)$,
                  rule(name: $"ax"$, $⊢ A^⊥, A$),
                  $⊢ class("normal", ?)A^⊥, A$,
                ),
                $⊢ class("normal", ?)A^⊥, class("normal", ?)A^⊥, A ⊗ A$,
              ),
              $⊢ class("normal", ?)A^⊥, A ⊗ A$,
            ),
            $⊢ class("normal", ?)A^⊥ ⅋ (A ⊗ A)$,
          ),
          $⊢ (class("normal", ?)A ⅋ bold(1)) class("binary", \&) (class("normal", ?)A^⊥ ⅋ (A ⊗ A))$,
        ),
        $⊢ (!bold(1) class("binary", \&) class("normal", ?)bold(1)) class("binary", \&) ((class("normal", ?)A ⅋ bold(1)) class("binary", \&) (class("normal", ?)A^⊥ ⅋ (A ⊗ A)))$,
      ),
      $⊢ ((bold(1) ⊕ 0) class("binary", \&) (0 ⊕ bold(1))) class("binary", \&) ((!bold(1) class("binary", \&) class("normal", ?)bold(1)) class("binary", \&) ((class("normal", ?)A ⅋ bold(1)) class("binary", \&) (class("normal", ?)A^⊥ ⅋ (A ⊗ A))))$,
    ),
    $⊢ ((((A^⊥ ⅋ B^⊥) ⅋ (A ⊗ B)) class("binary", \&) bold(1)) class("binary", \&) ((⊥ ⅋ bold(1)) class("binary", \&) ⊤)) class("binary", \&) (((bold(1) ⊕ 0) class("binary", \&) (0 ⊕ bold(1))) class("binary", \&) ((!bold(1) class("binary", \&) class("normal", ?)bold(1)) class("binary", \&) ((class("normal", ?)A ⅋ bold(1)) class("binary", \&) (class("normal", ?)A^⊥ ⅋ (A ⊗ A)))))$,
  ),
)
