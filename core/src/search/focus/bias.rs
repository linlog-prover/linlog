// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The atom bias: which literal of every atom the engine takes for
//! positive, under each rule of [`Bias`]. A function of the sequent alone,
//! so a run stays deterministic.

use crate::occurrences::{Forest, Sign};
use crate::search::Bias;
use crate::sequents::{Atom, Kind};
use std::cmp::Ordering;

/// Returns, per atom, the sign of its positive literal under the rule
/// given. Under [`Bias::Auto`] that is the factor rule without an
/// exponential in the sequent and the rarer literal with one.
pub(super) fn signs(forest: &Forest, rule: Bias) -> Box<[Sign]> {
    let num_atoms = forest.sequent().atom_names().len();
    // A `⊗` with a positive literal for a factor has its split forced, so
    // the literal that is more often such a factor is the positive one; an
    // occurrence counts half for every additive choice above it, of which
    // a proof takes one side.
    let by_factors = match rule {
        Bias::Auto => !forest
            .ids()
            .any(|o| matches!(forest.kind(o), Kind::Bang | Kind::Quest)),
        Bias::Rarer => false,
        Bias::Factors => true,
    };
    let mut factors = vec![0u64; forest.lists()];
    if by_factors {
        /// The weight of an occurrence under no additive choice.
        const WHOLE: u64 = 1 << 31;
        let mut choices = vec![0u8; forest.len()];
        for o in forest.ids() {
            let Some(p) = forest.parent(o) else {
                continue;
            };
            let additive = matches!(forest.kind(p), Kind::With | Kind::Plus);
            choices[o.index()] = choices[p.index()].saturating_add(u8::from(additive));
            if let (Some(atom), Some(s)) = (forest.atom(o), forest.sign(o))
                && forest.kind(p) == Kind::Tensor
            {
                factors[Forest::list(atom, s)] += WHOLE
                    .checked_shr(u32::from(choices[o.index()]))
                    .unwrap_or(0);
            }
        }
    }
    (0..num_atoms)
        .map(|a| {
            let atom = Atom::new(a as u32);
            let (vars, duals) = (
                forest.literals(atom, Sign::Atom).len(),
                forest.literals(atom, Sign::Dual).len(),
            );
            match factors[Forest::list(atom, Sign::Atom)]
                .cmp(&factors[Forest::list(atom, Sign::Dual)])
            {
                Ordering::Greater => Sign::Atom,
                Ordering::Less => Sign::Dual,
                Ordering::Equal if duals < vars => Sign::Dual,
                Ordering::Equal => Sign::Atom,
            }
        })
        .collect()
}

#[cfg(all(test, feature = "parse"))]
mod tests {
    use super::*;

    /// The positive literal of `atom` in `input` under `rule`.
    fn sign(input: &str, atom: &str, rule: Bias) -> Sign {
        let forest = Forest::new(&input.parse().unwrap()).unwrap();
        signs(&forest, rule)[forest.sequent().atom(atom).unwrap().index()]
    }

    /// Where no literal is a factor of a `⊗`, the literal with fewer
    /// occurrences is positive and a tie makes the atom positive.
    #[test]
    fn rarer_literal() {
        // ⊢ ~A, ~A, A, B, ~C ⅋ C, ~C ⅋ C
        let input = "A, A |- A, B, C -o C, C -o C";
        assert_eq!(
            sign(input, "A", Bias::Auto),
            Sign::Atom,
            "one A against two ~A"
        );
        assert_eq!(
            sign(input, "B", Bias::Auto),
            Sign::Dual,
            "no ~B at all, so ~B is the rarer one"
        );
        assert_eq!(sign(input, "C", Bias::Auto), Sign::Atom, "two of each");
        assert_eq!(sign("|- A, A, ~A", "A", Bias::Auto), Sign::Dual);
    }

    /// The literal that is more often a factor of a `⊗` is positive, an
    /// occurrence under an additive choice counting half; with an
    /// exponential in the sequent the rarer literal is.
    #[test]
    fn tensor_factors() {
        // `A` is a factor once and `~A` never, though `A` is not the rarer.
        let horn = "|- ~A, A * ~B, A * ~B, B";
        assert_eq!(sign(horn, "A", Bias::Auto), Sign::Atom);
        assert_eq!(sign(horn, "B", Bias::Auto), Sign::Dual);
        // `~D` is a factor under each side of a choice, `D` once outside
        // one: a tie, which the rarer literal wins.
        let choice = "|- (A * ~D) + (B * ~D), D * C, ~A, ~B, ~C";
        assert_eq!(sign(choice, "D", Bias::Auto), Sign::Atom);
        let exponential = "|- ?~A, A * ~B, A * ~B, B";
        assert_eq!(sign(exponential, "A", Bias::Auto), Sign::Dual);
        // The rules by name, whatever the sequent.
        assert_eq!(sign(exponential, "A", Bias::Factors), Sign::Atom);
        assert_eq!(sign(horn, "A", Bias::Rarer), Sign::Dual);
    }
}
