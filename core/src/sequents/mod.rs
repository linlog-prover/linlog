// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

// A variant a later step adds must not fall into an existing arm.
#![deny(clippy::wildcard_enum_match_arm)]

/// Printing sequents and formulas in one-sided notation.
pub mod fmt;
/// Atom names: identifiers of the text syntax that no keyword takes.
pub(crate) mod name;
/// The one printer of formulas and sequents, a table of spellings per
/// target.
pub(crate) mod notation;
/// The terms an arena is built from.
pub mod term;

pub use fmt::Formula;
pub(crate) use fmt::{Stop, Stops};
pub use fmt::{Visit, Walk};
pub use term::{Atom, Kind, Term, TermId};

use crate::hash::HashMap;

/// A one-sided sequent in negation normal form: root formulas over an arena of
/// shared subterms.
///
/// # Syntax
///
/// With the feature `parse`, `"…".parse::<Sequent>()` reads a sequent
/// written `formulas ⊢ formulas` (or `|-`), each side a list separated by
/// commas and possibly empty, with whitespace allowed between any two
/// tokens. From the loosest binding to the tightest, a formula is built
/// with `-o`/`⊸` (to the right), `+`/`⊕`, `&`, `|`/`par`/`⅋`, `*`/`⊗`
/// (these to the left), the prefix operators `~`, `!` and `?`, and the
/// postfix `^`, over the constants `0`, `1`, `bot`/`⊥` and `top`/`⊤`,
/// atoms and parentheses. An atom is named by a Unicode identifier (it
/// starts with `_` or a character of `XID_Start` and goes on with
/// characters of `XID_Continue`), read in NFC, so that one name is one
/// atom however its accents are encoded, other than the keywords `bot`, `top` and
/// `par`, which are that constant or connective only as whole
/// identifiers, and the words `forall` and `exists`, which are reserved
/// for the quantifiers of a later version: the syntax grows by new tokens
/// only, never by a new meaning of text that reads today. Text that is no
/// sequent is `Error::Parse`, whose `ParseError` says where. Every atom
/// name of a sequent, however it was read, is such an identifier, so that
/// the formulas written as text read back as themselves. The ordered
/// parse ([`parse_ordered`](Self::parse_ordered)) reads the divisions of
/// the Lambek calculus besides, `\` (to the right) and `/` (to the left),
/// as loose as `-o`; mixed, a division takes all to its right, `A \ B / C`
/// being `A \ (B / C)` and `A / B \ C` being `A / (B \ C)`.
///
/// The sequent is kept one-sided in negation normal form: the formulas
/// left of the turnstile are negated, `A ⊸ B` is `A^⊥ ⅋ B`, and a negation
/// is pushed to the atoms, so `A |- A` is `⊢ ~A, A`, which is what
/// `Display` writes. The formulas keep the order they were written in,
/// and the sequent remembers how many stood left of the turnstile
/// ([`antecedents`](Self::antecedents)), from which
/// [`Reading`](crate::Reading) reads an intuitionistic sequent back
/// two-sided. `Display`'s one-sided text leaves the sides out, so it
/// reads back as `⊢ Γ`; the text of the reading, which writes them, and
/// the JSON keep them.
///
/// # JSON
///
/// With the feature `serialize` a sequent is the object `{"version": 1,
/// "terms": […], "roots": […], "atoms": […], "antecedents": k}`, the
/// `version` only at the top of a document ([`wire`](crate::wire)).
/// `terms` is the arena: a unit is its symbol (`"1"`, `"⊥"`, `"⊤"`,
/// `"0"`), any other term an object of one key, its tag, which names terms
/// by their index in `terms`, each before it (`{"⊗": [0, 1]}`, and `⅋`,
/// `&`, `⊕` alike; `{"!": 2}`, `{"?": 2}`), or an atom by its index in
/// `atoms` (`{"V": 0}` the atom, `{"D": 0}` its dual). `roots` are the
/// root formulas, in the order written, and `antecedents` how many of
/// them, the first, stand left of `⊢`, written whenever the sides are
/// known, `0` included, and absent where they are not. Reading checks
/// that every term names only terms before it, that `antecedents` is at
/// most the number of roots and that every atom name is an identifier of
/// the text syntax (`Error::AtomName` otherwise), takes a name the table
/// repeats as one atom, and counts the occurrences against the limits
/// before anything unfolds. The command's `seq json` writes this form.
/// A sequent with cut formulas ([`cuts`](Self::cuts)) writes them as
/// `"cuts": […]`, the term of each cut formula `A` (its dual is the
/// reader's to build), which a document that holds one writes at wire
/// level 6; a sequent without them writes no key.
///
#[cfg_attr(all(feature = "parse", feature = "serialize"), doc = "```")]
#[cfg_attr(not(all(feature = "parse", feature = "serialize")), doc = "```ignore")]
/// let sequent: linlog::Sequent = "A |- A".parse()?;
/// assert_eq!(
///     serde_json::to_string(&sequent)?,
///     r#"{"version":1,"terms":[{"D":0},{"V":0}],"roots":[0,1],"atoms":["A"],"antecedents":1}"#
/// );
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sequent {
    /// Every subformula; a term refers only to terms at lower indices.
    pub(crate) terms: Vec<Term>,
    /// The root formulas, as arena indices, in the order they were
    /// written, which nothing sorts.
    pub(crate) roots: Vec<TermId>,
    /// The atom names that `Atom` and `DualAtom` refer to by index.
    pub(crate) atoms: Vec<String>,
    /// How many of the roots, the first ones, were written left of `⊢`;
    /// `None` where no sides were given. At most the number of roots.
    pub(crate) antecedents: Option<u32>,
    /// The cut formulas, in the order they were added: each the term of a
    /// formula `A` and the term of its dual `A⊥`, of one shape, which
    /// the forest numbers after the roots, `A` then `A⊥`.
    pub(crate) cuts: Vec<[TermId; 2]>,
}

impl std::default::Default for Sequent {
    /// Returns the empty sequent.
    fn default() -> Self {
        Self::new()
    }
}

impl Sequent {
    /// Returns the empty sequent.
    pub const fn new() -> Self {
        Self {
            terms: vec![],
            roots: vec![],
            atoms: vec![],
            antecedents: None,
            cuts: vec![],
        }
    }

    /// Returns the arena: every subformula, each referring only to terms at
    /// lower indices.
    pub fn terms(&self) -> &[Term] {
        &self.terms
    }

    /// Returns the term at `id`, which must belong to this sequent.
    ///
    /// # Panics
    ///
    /// For an id of another one, past its end.
    pub fn term(&self, id: TermId) -> Term {
        self.terms[id.index()]
    }

    /// Returns the root formulas, in the order they were written.
    pub fn roots(&self) -> &[TermId] {
        &self.roots
    }

    /// Returns how many root formulas were written left of `⊢`: the first
    /// ones of [`roots`](Self::roots), the rest being right of it. `None`
    /// when the sides are not known, as for a JSON sequent without them or
    /// the sum of two sequents; the text syntax always gives them (`Some(0)`
    /// for `⊢ Γ`). Intuitionistic mode reads the goal from them
    /// ([`Reading`](crate::Reading)); nothing else does.
    pub fn antecedents(&self) -> Option<u32> {
        self.antecedents
    }

    /// Returns the cut formulas, in the order they were added: per cut
    /// the term of the formula `A` and the term of its dual `A⊥`. They
    /// are no part of the conclusion (the [`roots`](Self::roots)): a
    /// proof with cuts ([`Node::Cut`](crate::Node::Cut)) is of a sequent
    /// that holds its cut formulas, whose [`Forest`](crate::Forest)
    /// numbers each pair after the roots. A sequent read as text holds
    /// none, and `Display` writes the conclusion only.
    pub fn cuts(&self) -> &[[TermId; 2]] {
        &self.cuts
    }

    /// Adds a cut formula: `formula`, a sequent of one root formula `A`
    /// whose atoms are this sequent's atoms of the same names, joins the
    /// arena with its dual `A⊥`, and the pair is numbered after the
    /// roots and the cut formulas before it. The roots and their sides
    /// are unchanged.
    ///
    /// # Errors
    ///
    /// [`Error::CutFormula`](crate::Error::CutFormula) when `formula`
    /// has another number of root formulas than one or cut formulas of
    /// its own.
    pub fn add_cut(&mut self, formula: Self) -> Result<(), crate::Error> {
        if formula.roots.len() != 1 || !formula.cuts.is_empty() {
            return Err(crate::Error::CutFormula {
                formulas: formula.roots.len(),
            });
        }
        let offset_atoms = self.atoms.len() as u32;
        let offset_terms = self.terms.len() as u32;
        self.terms.extend(
            formula
                .terms
                .into_iter()
                .map(|e| e.offset(offset_atoms, offset_terms)),
        );
        self.atoms.extend(formula.atoms);
        self.merge_atoms();
        self.push_cuts(&[TermId::new(formula.roots[0].get() + offset_terms)]);
        Ok(())
    }

    /// Adds the cut formulas whose terms are `cuts`, terms of the arena, in
    /// that order: each one's dual joins the arena, a term for each term
    /// below it, so the two are of one shape, each the first equal term the
    /// arena has where it has one, so that a sequent written with its duals
    /// reads back as it was. Every term is dualised once whatever the
    /// number of cut formulas that reach it, so the pass costs the arena
    /// and the cut formulas, not their product; no recursion.
    pub(crate) fn push_cuts(&mut self, cuts: &[TermId]) {
        // A sequent read without cut formulas pays nothing for them.
        if cuts.is_empty() {
            return;
        }
        let mut known =
            HashMap::<Term, TermId>::with_capacity_and_hasher(self.terms.len(), Default::default());
        // Not `entry`, whose one use in `optimize_terms` the compiler
        // inlines into the parser's path when it is the only one.
        for (n, &term) in self.terms.iter().enumerate().rev() {
            known.insert(term, TermId::new(n as u32));
        }
        let mut dual = HashMap::<TermId, TermId>::default();
        // A term waits until its subterms have their duals; a subterm comes
        // before its parent, so no term waits on itself.
        let mut waiting: Vec<(TermId, bool)> = Vec::new();
        for &a in cuts {
            waiting.push((a, false));
            while let Some((t, ready)) = waiting.pop() {
                if dual.contains_key(&t) {
                    continue;
                }
                if !ready {
                    waiting.push((t, true));
                    let below = self.terms[t.index()].subterms();
                    waiting.extend(below.filter(|k| !dual.contains_key(k)).map(|k| (k, false)));
                    continue;
                }
                let term = self.terms[t.index()].dual().map_subterms(|k| dual[&k]);
                let id = match known.get(&term) {
                    Some(&id) => id,
                    None => {
                        let fresh = TermId::new(self.terms.len() as u32);
                        known.insert(term, fresh);
                        self.terms.push(term);
                        fresh
                    }
                };
                dual.insert(t, id);
            }
            self.cuts.push([a, dual[&a]]);
        }
    }

    /// Takes the last cut formula away, with the terms and atoms past those
    /// the roots and the other cut formulas reach, which a cut formula
    /// added last brought: the arena as it was before the cut was added.
    #[cfg(feature = "interactive")]
    pub(crate) fn pop_cut(&mut self) {
        self.cuts.pop();
        // A subterm precedes its parent, so the highest term any kept
        // formula reaches is a root or a cut formula itself.
        let end = self
            .roots
            .iter()
            .chain(self.cuts.iter().flatten())
            .map(|t| t.index() + 1)
            .max()
            .unwrap_or(0);
        self.terms.truncate(end);
        let atoms = self
            .terms
            .iter()
            .filter_map(|t| t.atom())
            .map(|a| a.index() + 1)
            .max()
            .unwrap_or(0);
        // An atom no term names, which only a sequent read from JSON can
        // have, goes too.
        self.atoms.truncate(atoms);
    }

    /// A sequent of an arena built elsewhere in the crate: every term's
    /// subterms come before it, every atom is named, the roots are terms of
    /// the arena, as a reader that builds the arena itself makes them, and
    /// `antecedents` is at most their number.
    #[cfg(feature = "parse")]
    pub(crate) fn from_parts(
        terms: Vec<Term>,
        roots: Vec<TermId>,
        atoms: Vec<String>,
        antecedents: Option<u32>,
    ) -> Self {
        let sequent = Self {
            terms,
            roots,
            atoms,
            antecedents,
            cuts: vec![],
        };
        debug_assert!(sequent.check().is_ok());
        sequent
    }

    /// Returns the atom names, in the order `Atom` indexes them.
    pub fn atom_names(&self) -> &[String] {
        &self.atoms
    }

    /// Returns the name of `atom`, which must belong to this sequent.
    ///
    /// # Panics
    ///
    /// For an id of another one, past its end.
    pub fn atom_name(&self, atom: Atom) -> &str {
        &self.atoms[atom.index()]
    }

    /// Returns the atom called `name`, if the sequent has one.
    pub fn atom(&self, name: &str) -> Option<Atom> {
        self.atoms
            .iter()
            .position(|n| n == name)
            .map(|i| Atom::new(i as u32))
    }

    /// Returns, per arena term, the number of subformula occurrences of the
    /// formula it is the top of, itself included: a subterm counts once
    /// for every place it stands in. A number that passes `u64::MAX` is
    /// that.
    pub(crate) fn sizes(&self) -> Vec<u64> {
        // A subterm precedes its parent, so its size is there when the
        // parent's is added up.
        let mut sizes = vec![0u64; self.terms.len()];
        for (n, term) in self.terms.iter().enumerate() {
            sizes[n] = term
                .subterms()
                .fold(1u64, |sum, k| sum.saturating_add(sizes[k.index()]));
        }
        sizes
    }

    /// Returns the number of subformula occurrences of the sequent, which
    /// is the number of nodes of its formulas written out as trees: a
    /// term that several terms or root formulas share counts once for
    /// each. That is how many occurrences a [`Forest`](crate::Forest) of
    /// the sequent has, and it can be exponentially more than the arena
    /// has terms; a number that passes `u64::MAX` is returned as that.
    /// One pass over the arena.
    pub fn occurrences(&self) -> u64 {
        self.occurrences_of(&self.sizes())
    }

    /// Returns the number of occurrences of the sequent from the
    /// [`sizes`](Self::sizes) of its terms: those of the roots, then those
    /// of each cut formula and its dual.
    pub(crate) fn occurrences_of(&self, sizes: &[u64]) -> u64 {
        self.roots
            .iter()
            .chain(self.cuts.iter().flatten())
            .fold(0u64, |sum, r| sum.saturating_add(sizes[r.index()]))
    }

    /// Checks the arena's invariants, which every public way to a
    /// sequent keeps: every term names only earlier terms and atoms of the
    /// table, every root a term, and `antecedents` at most the roots.
    #[cfg(any(test, feature = "parse", feature = "serialize"))]
    pub(crate) fn check(&self) -> Result<(), crate::Error> {
        let num_atoms = self.atoms.len() as u32;
        let num_terms = self.terms.len() as u32;

        // check that terms only reference
        //   - other terms with lower IDs than themselves
        //   - existing atom IDS
        self.terms
            .iter()
            .enumerate()
            .try_for_each(|(n, e)| e.check_bounds(num_atoms, n as u32))?;

        // check that all root and cut indices are valid
        self.roots
            .iter()
            .chain(self.cuts.iter().flatten())
            .try_for_each(|n| {
                if n.get() >= num_terms {
                    Err(crate::Error::IndexOutOfBounds {
                        space: crate::limits::Space::Term,
                        index: n.index(),
                        len: num_terms as usize,
                    })
                } else {
                    Ok(())
                }
            })?;

        match self.antecedents {
            Some(k) if k as usize > self.roots.len() => Err(crate::Error::Antecedents {
                antecedents: k as usize,
                roots: self.roots.len(),
            }),
            _ => Ok(()),
        }
    }

    /// Drops the terms no root formula reaches and merges equal terms, keeping
    /// the arena topologically sorted, which it must be.
    fn optimize_terms(&mut self) {
        let num_terms = self.terms.len();

        let mut reachable = vec![false; num_terms];
        for n in self.roots.iter().chain(self.cuts.iter().flatten()) {
            reachable[n.index()] = true;
        }

        // A subterm precedes its parent, so one pass from the top marks every
        // reachable term before it is visited.
        for n in (0..num_terms).rev() {
            if !reachable[n] {
                continue;
            }
            for k in self.terms[n].subterms() {
                debug_assert!(k.index() < n, "a subterm precedes its parent");
                reachable[k.index()] = true;
            }
        }

        // In index order, every subterm has its final index before its parents
        // are rebuilt, so hashing the rebuilt terms merges equal terms at any
        // depth.
        let mut new_index = vec![None; num_terms];
        let mut terms = Vec::with_capacity(num_terms);
        let mut kept = HashMap::<Term, TermId>::default();

        for (n, e) in self.terms.iter().enumerate() {
            if !reachable[n] {
                continue;
            }
            // The subterms of a reachable term are reachable and precede it.
            let e = e.map_subterms(|k| new_index[k.index()].unwrap());
            let fresh_index = TermId::new(terms.len() as u32);
            let index = *kept.entry(e).or_insert(fresh_index);
            if index == fresh_index {
                terms.push(e);
            }
            new_index[n] = Some(index);
        }

        self.terms = terms;
        self.roots = self
            .roots
            .iter()
            .map(|k| new_index[k.index()].unwrap())
            .collect();
        for pair in &mut self.cuts {
            *pair = pair.map(|k| new_index[k.index()].unwrap());
        }
    }

    /// Numbers the atoms in the order the terms first name them and drops
    /// the entries no term names; every literal must name an atom of the
    /// table. The names move into the new table, none is copied.
    fn renumber_atoms(&mut self) {
        use Term::*;
        /// No term names the atom.
        const UNUSED: u32 = u32::MAX;
        // Per entry of the table its new number, and per new number the
        // entry: a table has fewer entries than `u32::MAX`, as it is
        // indexed by `Atom`s.
        let mut number = vec![UNUSED; self.atoms.len()];
        let mut entries = Vec::with_capacity(self.atoms.len());
        for e in &mut self.terms {
            let (Atom(a) | DualAtom(a)) = e else {
                continue;
            };
            let slot = &mut number[a.index()];
            if *slot == UNUSED {
                *slot = entries.len() as u32;
                entries.push(a.index());
            }
            *a = term::Atom::new(*slot);
        }
        let mut names = std::mem::take(&mut self.atoms);
        self.atoms = entries
            .into_iter()
            .map(|entry| std::mem::take(&mut names[entry]))
            .collect();
    }

    /// Merges atoms of the same name and equal terms and drops the terms no
    /// root formula reaches; the root formulas keep the order they were
    /// written in. Every sequent a caller can hold keeps the arena's
    /// invariants, which this relies on.
    pub fn optimize(&mut self) {
        // Equal names first, so that terms over them are equal terms.
        self.merge_atoms();
        self.optimize_terms();
        self.renumber_atoms();
    }

    /// Gives atoms of the same name one index, the first's, and drops the
    /// later entries of the table, so that a name is an atom; a table of
    /// distinct names stays as it is. The terms must refer to atoms of the
    /// table.
    pub(crate) fn merge_atoms(&mut self) {
        use Term::*;
        let mut seen = HashMap::<&str, term::Atom>::with_capacity_and_hasher(
            self.atoms.len(),
            Default::default(),
        );
        let mut merged = Vec::with_capacity(self.atoms.len());
        for name in &self.atoms {
            let fresh = term::Atom::new(seen.len() as u32);
            merged.push(*seen.entry(name).or_insert(fresh));
        }
        let distinct = seen.len();
        drop(seen);
        if distinct == self.atoms.len() {
            return;
        }
        for e in &mut self.terms {
            *e = match *e {
                Atom(a) => Atom(merged[a.index()]),
                DualAtom(a) => DualAtom(merged[a.index()]),
                other @ One
                | other @ Bot
                | other @ Top
                | other @ Zero
                | other @ Tensor(..)
                | other @ Par(..)
                | other @ With(..)
                | other @ Plus(..)
                | other @ Bang(_)
                | other @ Quest(_) => other,
            };
        }
        // An entry stays when it is the first of its name, which is when
        // its new index is the number of entries kept before it.
        let (mut old, mut kept) = (0, 0);
        self.atoms.retain(|_| {
            let first = merged[old].index() == kept;
            old += 1;
            kept += usize::from(first);
            first
        });
    }

    /// Consumes another sequent and appends its root formulas to this one,
    /// and its cut formulas after this one's, with the arenas
    /// concatenated; an atom of the other sequent is the
    /// atom of the same name here, if there is one. The sides of the sum
    /// are not known: [`antecedents`](Self::antecedents) is `None`.
    pub fn add(&mut self, s: Self) {
        let offset_atoms = self.atoms.len() as u32;
        let offset_terms = self.terms.len() as u32;

        self.terms.extend(
            s.terms
                .into_iter()
                .map(|e| e.offset(offset_atoms, offset_terms)),
        );

        self.roots.extend(
            s.roots
                .into_iter()
                .map(|n| TermId::new(n.get() + offset_terms)),
        );

        self.cuts.extend(
            s.cuts
                .into_iter()
                .map(|pair| pair.map(|n| TermId::new(n.get() + offset_terms))),
        );

        self.atoms.extend(s.atoms);
        // The roots of the second sequent follow those of the first, so
        // its antecedents are no longer the first roots.
        self.antecedents = None;
        self.merge_atoms();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a sequent from raw parts, as a test would write them.
    pub(crate) fn raw(terms: Vec<Term>, roots: &[u32], atoms: &[&str]) -> Sequent {
        Sequent {
            terms,
            roots: roots.iter().map(|&n| TermId::new(n)).collect(),
            atoms: atoms.iter().map(|s| s.to_string()).collect(),
            antecedents: None,
            cuts: vec![],
        }
    }

    /// Shorthand for `Term::Atom`.
    pub(crate) const fn var(a: u32) -> Term {
        Term::Atom(Atom::new(a))
    }

    /// Shorthand for `Term::DualAtom`.
    pub(crate) const fn dual_var(a: u32) -> Term {
        Term::DualAtom(Atom::new(a))
    }

    /// Shorthand for `Term::Tensor`.
    pub(crate) const fn tensor(k: u32, l: u32) -> Term {
        Term::Tensor(TermId::new(k), TermId::new(l))
    }

    /// Shorthand for `Term::Bang`.
    pub(crate) const fn bang(k: u32) -> Term {
        Term::Bang(TermId::new(k))
    }

    /// Adding a sequent identifies its atoms with those of the same name,
    /// and leaves the others in the order they came.
    #[test]
    fn add_merges_atoms_by_name() {
        let mut s = raw(vec![dual_var(0), var(1)], &[0, 1], &["A", "B"]);
        s.add(raw(
            vec![var(0), var(1), var(2)],
            &[0, 1, 2],
            &["C", "A", "B"],
        ));
        assert_eq!(s.atoms, ["A", "B", "C"]);
        assert_eq!(s.terms, [dual_var(0), var(1), var(2), var(0), var(1)]);
        assert_eq!(s.roots.len(), 5);
    }

    /// The empty sequent survives optimisation unchanged.
    #[test]
    fn optimize_empty() {
        let mut s = Sequent::new();
        s.optimize();
        assert!(s.terms.is_empty() && s.roots.is_empty() && s.atoms.is_empty());
    }

    /// Atoms of one name merge, and the rest are renumbered to match, also
    /// when a repeated name comes before a new one.
    #[test]
    fn optimize_merges_atoms() {
        let mut s = raw(
            vec![var(0), dual_var(1), var(2)],
            &[0, 1, 2],
            &["A", "A", "B"],
        );
        s.optimize();
        assert_eq!(s.atoms, ["A", "B"]);
        assert_eq!(s.terms, [var(0), dual_var(0), var(1)]);
    }

    /// The atoms are numbered in the order the terms first name them, not
    /// in the table's order, and an atom no term names leaves the table.
    #[test]
    fn optimize_numbers_atoms_as_the_terms_name_them() {
        let mut s = raw(
            vec![var(2), dual_var(0), var(2)],
            &[0, 1, 2],
            &["B", "C", "A"],
        );
        s.optimize();
        assert_eq!(s.atoms, ["A", "B"]);
        assert_eq!(s.terms, [var(0), dual_var(1)]);
        assert_eq!(s.roots, [TermId::new(0), TermId::new(1), TermId::new(0)]);
    }

    /// References to a duplicate term point to its kept copy, also after
    /// unreachable terms before it are dropped.
    #[test]
    fn optimize_redirects_duplicates() {
        use Term::*;
        let mut s = raw(vec![Zero, One, Top, Top, tensor(1, 3)], &[2, 4], &[]);
        s.optimize();
        assert_eq!(s.terms, [One, Top, tensor(0, 1)]);
        assert_eq!(s.roots, [TermId::new(1), TermId::new(2)]);
    }

    /// A subterm that does not precede its parent is reported with each index
    /// in its place.
    #[test]
    fn subterm_order_error_names_both_terms() {
        use Term::*;
        let s = raw(vec![One, bang(2), Bot], &[1], &[]);
        assert_eq!(
            s.check().unwrap_err().to_string(),
            "term 1 refers to term 2, but a subterm must come before the terms that use it"
        );
    }

    /// Equal terms merge at every depth, not only equal leaves.
    #[test]
    fn optimize_merges_equal_subterms() {
        let mut s = raw(
            vec![
                var(0),
                var(1),
                tensor(0, 1),
                bang(2),
                var(0),
                var(1),
                tensor(4, 5),
                bang(6),
            ],
            &[3, 7],
            &["A", "B"],
        );
        s.optimize();
        assert_eq!(s.terms, [var(0), var(1), tensor(0, 1), bang(2)]);
        assert_eq!(s.roots, [TermId::new(3), TermId::new(3)]);
    }

    /// Appending a sequent shifts its term and atom indices past the existing
    /// ones.
    #[test]
    fn add_offsets_indices() {
        let mut s = raw(vec![var(0)], &[0], &["A"]);
        s.add(raw(vec![dual_var(0), bang(0)], &[1], &["B"]));
        assert_eq!(s.terms, [var(0), dual_var(1), bang(1)]);
        assert_eq!(s.roots, [TermId::new(0), TermId::new(2)]);
        assert_eq!(s.atoms, ["A", "B"]);
    }
}
