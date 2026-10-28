// linlog © Fabian Lukas Grubmüller 2026
// Licensed under the EUPL

//! The first-order tables of a sequent: its symbols, its terms, their
//! argument lists, what each atom is as a predicate applied to arguments,
//! and the names of its bound variables. A propositional sequent has
//! empty tables and pays nothing for them.

use super::TermId;
#[cfg(any(test, feature = "parse", feature = "serialize"))]
use super::name;
#[cfg(any(test, feature = "parse", feature = "serialize"))]
use crate::Error;
use crate::hash::HashMap;
use std::fmt::{Result as FmtResult, Write};

/// A predicate or function symbol, by its index in its sequent's symbol
/// table. A symbol is a name with one arity: a constant is a function
/// symbol of arity 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(u32);

impl Symbol {
    /// Wraps a raw table index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw table index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the table.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// The index of a first-order term in its sequent's term arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FoTermId(u32);

impl FoTermId {
    /// Wraps a raw arena index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw arena index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the arena.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// An argument list, by its index in its sequent's table of argument
/// lists; the list of a constant is empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Args(u32);

impl Args {
    /// Wraps a raw table index.
    pub const fn new(index: u32) -> Self {
        Self(index)
    }

    /// Returns the raw table index.
    pub const fn get(self) -> u32 {
        self.0
    }

    /// Returns the index as a `usize`, for indexing the table.
    pub const fn index(self) -> usize {
        self.0 as usize
    }
}

/// A first-order term of a sequent: a bound variable or a function
/// symbol applied to arguments, which refers to its arguments by arena
/// index. Every term of a sequent's arena is bound or ground: the free
/// variables of a search and the eigenvariables of a proof live in their
/// own extension of the arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FoTerm {
    /// A bound variable, as the number of binders between it and the one
    /// that binds it: `0` is the innermost binder's variable.
    Bound(u32),
    /// A function symbol applied to its arguments, a constant to none.
    App(Symbol, Args),
}

/// A term is a tag and two indices at most: every arena and every table
/// keyed by terms grows with it.
const _: () = assert!(size_of::<FoTerm>() == 12);

/// The first-order tables of a sequent, empty exactly when every atom is
/// nullary and the sequent has no binder.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct FirstOrder {
    /// The symbols, each a name with its arity, in the order they were
    /// first used.
    pub(crate) symbols: Vec<(String, u32)>,
    /// The term arena; a term refers only to terms at lower indices.
    pub(crate) terms: Vec<FoTerm>,
    /// The arguments of every list, one after the other.
    pub(crate) args: Vec<FoTermId>,
    /// The argument lists, each a start and a length into `args`.
    pub(crate) lists: Vec<(u32, u32)>,
    /// Per atom of the sequent's table, its predicate symbol and its
    /// argument list; as long as the atom table, or empty where every
    /// atom is nullary and there is no binder.
    pub(crate) atoms: Vec<(Symbol, Args)>,
    /// The name of every binder's variable, by the binder's term, in
    /// ascending order of terms: for printing only.
    pub(crate) binders: Vec<(TermId, String)>,
}

impl FirstOrder {
    /// Returns the empty tables.
    pub(crate) const fn new() -> Self {
        Self {
            symbols: Vec::new(),
            terms: Vec::new(),
            args: Vec::new(),
            lists: Vec::new(),
            atoms: Vec::new(),
            binders: Vec::new(),
        }
    }

    /// Returns whether the tables are empty: every atom nullary and no
    /// binder.
    pub(crate) fn is_empty(&self) -> bool {
        self.atoms.is_empty() && self.binders.is_empty()
    }

    /// Returns the terms of the argument list `args`.
    pub(crate) fn list(&self, args: Args) -> &[FoTermId] {
        let (start, len) = self.lists[args.index()];
        &self.args[start as usize..(start + len) as usize]
    }

    /// Returns the name of the binder `term`'s variable, if `term` is a
    /// binder with a recorded name.
    pub(crate) fn binder_name(&self, term: TermId) -> Option<&str> {
        self.binders
            .binary_search_by_key(&term, |&(t, _)| t)
            .ok()
            .map(|i| self.binders[i].1.as_str())
    }

    /// Writes the term `term` with `symbol` writing a symbol's name and
    /// `bound` a bound variable, as `f(t, …)` with `, ` between the
    /// arguments, on an explicit stack: no term costs the caller's stack.
    pub(crate) fn write<W: Write + ?Sized>(
        &self,
        out: &mut W,
        term: FoTermId,
        symbol: &mut impl FnMut(&mut W, &str) -> FmtResult,
        bound: &mut impl FnMut(&mut W, u32) -> FmtResult,
    ) -> FmtResult {
        // The argument lists being written, each with the next argument.
        let mut open: Vec<(Args, u32)> = Vec::new();
        let mut next = Some(term);
        loop {
            if let Some(t) = next.take() {
                match self.terms[t.index()] {
                    FoTerm::Bound(index) => bound(out, index)?,
                    FoTerm::App(f, args) => {
                        symbol(out, &self.symbols[f.index()].0)?;
                        if self.lists[args.index()].1 > 0 {
                            out.write_char('(')?;
                            open.push((args, 0));
                        }
                    }
                }
            }
            let Some(&mut (args, ref mut i)) = open.last_mut() else {
                return Ok(());
            };
            let list = self.list(args);
            if (*i as usize) < list.len() {
                if *i > 0 {
                    out.write_str(", ")?;
                }
                next = Some(list[*i as usize]);
                *i += 1;
            } else {
                out.write_char(')')?;
                open.pop();
            }
        }
    }

    /// Writes the atom of `symbol` applied to `args` as its canonical
    /// text, the key of the atom table: `p` for a nullary atom, else
    /// `p(t, …)`, a bound variable as `#n`.
    pub(crate) fn write_atom<W: Write + ?Sized>(
        &self,
        out: &mut W,
        symbol: Symbol,
        args: Args,
    ) -> FmtResult {
        out.write_str(&self.symbols[symbol.index()].0)?;
        if self.lists[args.index()].1 == 0 {
            return Ok(());
        }
        out.write_char('(')?;
        for (i, &t) in self.list(args).iter().enumerate() {
            if i > 0 {
                out.write_str(", ")?;
            }
            self.write(
                out,
                t,
                &mut |out, name| out.write_str(name),
                &mut |out, index| write!(out, "#{index}"),
            )?;
        }
        out.write_char(')')
    }

    /// Returns the canonical text of the atom of `symbol` applied to
    /// `args` ([`write_atom`](Self::write_atom)).
    pub(crate) fn atom_key(&self, symbol: Symbol, args: Args) -> String {
        let mut key = String::new();
        self.write_atom(&mut key, symbol, args)
            .expect("writing into a string cannot fail");
        key
    }

    /// Returns, per term of the arena, how many terms it unfolds to when
    /// written out as a tree, itself included, saturating: a term that
    /// several terms share counts once for each.
    #[cfg(feature = "serialize")]
    pub(crate) fn unfolded(&self) -> Vec<u64> {
        let mut unfolded = vec![0u64; self.terms.len()];
        for (n, term) in self.terms.iter().enumerate() {
            unfolded[n] = match *term {
                FoTerm::Bound(_) => 1,
                FoTerm::App(_, args) => self
                    .list(args)
                    .iter()
                    .fold(1u64, |sum, t| sum.saturating_add(unfolded[t.index()])),
            };
        }
        unfolded
    }

    /// Returns, per term of the arena, how many binders it needs above it:
    /// one more than its largest bound index, and `0` for a ground term.
    pub(crate) fn loose(&self) -> Vec<u32> {
        let mut loose = vec![0u32; self.terms.len()];
        for (n, term) in self.terms.iter().enumerate() {
            loose[n] = match *term {
                FoTerm::Bound(index) => index.saturating_add(1),
                FoTerm::App(_, args) => self
                    .list(args)
                    .iter()
                    .map(|t| loose[t.index()])
                    .max()
                    .unwrap_or(0),
            };
        }
        loose
    }

    /// Checks the tables' invariants, which every public way to a sequent
    /// keeps: `atoms` is as long as the sequent's table of `atom_count`
    /// atoms or empty, every index names an entry, a term refers to terms
    /// before it, every application and every atom has as many arguments
    /// as its symbol's arity, no symbol is both a predicate and a function
    /// symbol, every symbol's name is an identifier of the text syntax,
    /// and the binders' terms ascend.
    #[cfg(any(test, feature = "parse", feature = "serialize"))]
    pub(crate) fn check(&self, atom_count: usize) -> Result<(), Error> {
        use crate::limits::Space;
        if !self.atoms.is_empty() && self.atoms.len() != atom_count {
            return Err(Error::Predicates {
                predicates: self.atoms.len(),
                atoms: atom_count,
            });
        }
        for (name, _) in &self.symbols {
            name::check(name)?;
        }
        let mut names = std::collections::BTreeSet::<&str>::new();
        for (name, _) in &self.symbols {
            if !names.insert(name.as_str()) {
                return Err(Error::Symbol {
                    name: name.clone(),
                    fault: SymbolFault::Repeated,
                });
            }
        }
        let symbols = self.symbols.len();
        if let Some(t) = self.args.iter().find(|t| t.index() >= self.terms.len()) {
            return Err(Error::IndexOutOfBounds {
                space: Space::Term,
                index: t.index(),
                len: self.terms.len(),
            });
        }
        for (n, &(start, len)) in self.lists.iter().enumerate() {
            let end = (start as usize).saturating_add(len as usize);
            if end > self.args.len() {
                return Err(Error::IndexOutOfBounds {
                    space: Space::Term,
                    index: n,
                    len: self.args.len(),
                });
            }
        }
        // Which role each symbol has, where a term or an atom gives it one.
        let mut role = vec![None; symbols];
        let mut arity = |symbol: Symbol, args: Args, predicate: bool| -> Result<(), Error> {
            if symbol.index() >= symbols {
                return Err(Error::IndexOutOfBounds {
                    space: Space::Symbol,
                    index: symbol.index(),
                    len: symbols,
                });
            }
            if args.index() >= self.lists.len() {
                return Err(Error::IndexOutOfBounds {
                    space: Space::Term,
                    index: args.index(),
                    len: self.lists.len(),
                });
            }
            let (name, declared) = &self.symbols[symbol.index()];
            if self.lists[args.index()].1 != *declared {
                return Err(Error::Symbol {
                    name: name.clone(),
                    fault: SymbolFault::Arity {
                        declared: *declared,
                        found: self.lists[args.index()].1,
                    },
                });
            }
            let slot = &mut role[symbol.index()];
            if slot.is_some_and(|p| p != predicate) {
                return Err(Error::Symbol {
                    name: name.clone(),
                    fault: SymbolFault::Role,
                });
            }
            *slot = Some(predicate);
            Ok(())
        };
        for (n, term) in self.terms.iter().enumerate() {
            if let FoTerm::App(f, args) = *term {
                arity(f, args, false)?;
                for t in self.list(args) {
                    if t.index() >= n {
                        return Err(Error::NotTopological {
                            space: Space::Term,
                            index: t.index(),
                            parent: n,
                        });
                    }
                }
            }
        }
        for &(p, args) in &self.atoms {
            arity(p, args, true)?;
        }
        if self.binders.windows(2).any(|w| w[0].0 >= w[1].0) {
            return Err(Error::NotTopological {
                space: Space::Term,
                index: 0,
                parent: 0,
            });
        }
        for (_, name) in &self.binders {
            name::check(name)?;
        }
        Ok(())
    }
}

/// Why a symbol of a sequent's table cannot be one.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize),
    serde(rename_all = "snake_case")
)]
pub enum SymbolFault {
    /// The name has two entries in the table.
    #[error("is listed twice")]
    Repeated,
    /// A use has another number of arguments than the symbol's arity.
    #[error("takes {declared} arguments, not {found}")]
    Arity {
        /// The arity the table declares.
        declared: u32,
        /// The number of arguments the use has.
        found: u32,
    },
    /// The name is used as a predicate and as a function symbol.
    #[error("is a predicate and a function symbol at once")]
    Role,
}

/// Builds the first-order tables of a sequent, interning every symbol,
/// term and argument list, so that equal terms are one entry.
#[derive(Debug, Default)]
pub(crate) struct Interner {
    /// The tables built.
    pub(crate) tables: FirstOrder,
    /// Every symbol by its name.
    symbols: HashMap<String, Symbol>,
    /// Whether each symbol is a predicate, once a use has said.
    predicate: Vec<Option<bool>>,
    /// Every term by its value.
    terms: HashMap<FoTerm, FoTermId>,
    /// Every argument list by its terms.
    lists: HashMap<Box<[FoTermId]>, Args>,
}

impl Interner {
    /// Returns the interner of empty tables.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Returns the symbol called `name`, in NFC, of `arity` and of the
    /// role `predicate` says, interned.
    ///
    /// # Errors
    ///
    /// [`SymbolFault::Arity`] where the name is known with another arity,
    /// [`SymbolFault::Role`] where it is known in the other role.
    pub(crate) fn symbol(
        &mut self,
        name: &str,
        arity: u32,
        predicate: bool,
    ) -> Result<Symbol, SymbolFault> {
        let symbol = match self.symbols.get(name) {
            Some(&symbol) => symbol,
            None => {
                let symbol = Symbol::new(self.tables.symbols.len() as u32);
                self.tables.symbols.push((name.to_owned(), arity));
                self.symbols.insert(name.to_owned(), symbol);
                self.predicate.push(None);
                symbol
            }
        };
        let declared = self.tables.symbols[symbol.index()].1;
        if declared != arity {
            return Err(SymbolFault::Arity {
                declared,
                found: arity,
            });
        }
        let slot = &mut self.predicate[symbol.index()];
        if slot.is_some_and(|p| p != predicate) {
            return Err(SymbolFault::Role);
        }
        *slot = Some(predicate);
        Ok(symbol)
    }

    /// Returns the term `term`, interned.
    pub(crate) fn term(&mut self, term: FoTerm) -> FoTermId {
        if let Some(&id) = self.terms.get(&term) {
            return id;
        }
        let id = FoTermId::new(self.tables.terms.len() as u32);
        self.tables.terms.push(term);
        self.terms.insert(term, id);
        id
    }

    /// Returns the argument list of `items`, interned.
    pub(crate) fn list(&mut self, items: &[FoTermId]) -> Args {
        if let Some(&args) = self.lists.get(items) {
            return args;
        }
        let args = Args::new(self.tables.lists.len() as u32);
        let start = self.tables.args.len() as u32;
        self.tables.args.extend_from_slice(items);
        self.tables.lists.push((start, items.len() as u32));
        self.lists.insert(items.into(), args);
        args
    }
}
