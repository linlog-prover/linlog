---
paths:
  - "core/src/sequents/**"
  - "core/src/parse/**"
  - "core/src/fragment.rs"
  - "core/src/serialize/**"
  - "core/tests/parse.rs"
  - "core/tests/first_order.rs"
  - "core/tests/serialize.rs"
  - "core/tests/lock.rs"
---

# linlog core: sequents, parsing and serialization

Loaded, beside `core.md`, when a file of the sequents, the fragments,
the parser or the serialization is read or changed.

## Sequents are arena-allocated DAGs

`Sequent` (`core/src/sequents/mod.rs`) has six fields, all `pub(crate)`:
- `terms: Vec<Term>`: every subformula. Children are referenced by arena
  index (`TermId`, a `u32` newtype), never by pointer.
- `roots: Vec<TermId>`: the root formulas that make up the sequent, in the
  order they were written; nothing sorts them.
- `atoms: Vec<String>`: atom names. `Atom(a)`/`DualAtom(a)` index into this
  with `Atom`, a `u32` newtype.
- `antecedents: Option<u32>`: how many of the roots, the first ones, were
  written left of `⊢`; `None` where no sides were given (a JSON sequent
  without the key, `add`, a test's literal). The parser sets it for every
  text (`Some(0)` for `⊢ Γ`), `mist::read` and `ordinary::translate` from
  their own sides; `check` refuses one above the number of
  roots (`Error::Antecedents`). Only the intuitionistic reading reads it
  (`core-forest.md`); it is part of equality.
- `cuts: Vec<[TermId; 2]>`: the cut formulas (`core-proofs.md`).
- `fo: fo::FirstOrder` (`sequents/fo.rs`): the first-order tables, empty
  exactly when every atom is nullary and there is no binder, so that a
  propositional sequent pays nothing and writes level 1. `symbols` are
  `(name, arity)` in the order first used (one arity per name, and a name
  is a predicate or a function symbol, never both: `Interner::symbol`
  and `FirstOrder::check` refuse the rest); `terms` is a second arena of
  `FoTerm` (`Bound(n)`, a de Bruijn index, the innermost binder `0`;
  `App(Symbol, Args)`), topological and hash-consed; `args` and `lists`
  are the argument lists as a flat vector with starts and lengths, each
  distinct list once; `atoms` is parallel to the atom table (as long as
  it, or empty), each atom's predicate and list; `binders` is the name
  of every binder term's variable in ascending term order, for printing
  only. The atom table's key stays the canonical text (`p(f(a), b)`,
  an open atom `p(#0)`, `FirstOrder::atom_key`), so `atom_name` and
  every caller keyed by names are unchanged, and ground first-order
  atoms are propositional atoms with structured names. `check` runs
  `FirstOrder::check` and the scope pass (per term how many binders it
  needs above it; a root or cut formula needs none, else
  `Error::LooseVariable`) only when the tables are not empty. `optimize`
  moves the binders' names with their terms (the first name wins a
  merge), `renumber_atoms` and `merge_atoms` keep `fo.atoms` parallel,
  `pop_cut` drops the names of truncated terms, and `add` re-interns
  both sides' tables (`join_first_order`: a name of another arity or role
  gets `_` after it, a nullary atom of a propositional side its symbol).

**A term only references subterms with a strictly smaller index**, so the
arena is topologically sorted: one ascending pass sees every subterm before
its parents, one descending pass sees every parent before its subterms
(`fragment()` and the forest's size computation rely on the latter).
`check()` (crate-private) checks it, `Formula` printing debug-asserts it, and
deserialization runs the check. Code that builds or rewrites an arena must
preserve it.

`optimize()` runs after parsing. It deduplicates atom names, hash-conses
identical terms and drops unreachable ones; the roots keep their written
order, so the forest numbers the formulas as they were written and a
root's index is its place in the text. Nothing else
guarantees that every arena term is reachable: a deserialized sequent may
carry junk terms, and `fragment()` and `Forest` walk from the roots for that
reason. `Sequent::add` merges two sequents by offsetting atom and term indices.

**A name is an atom**: the atom table of every `Sequent` a caller can hold
has distinct names. `optimize` merges equal names (and drops unused ones),
`add` identifies the added sequent's atoms with those of the same name
(`merge_atoms`), and deserialization does the same after the integrity
check, so a JSON dictionary that repeats a name reads as the sequent with
the name once; before, such a file gave two atoms that printed alike, and
`⊢ ~A, A` was answered "unprovable". `merge_atoms` leaves a table of
distinct names untouched (unused entries and their order included), so a
JSON sequent is written back as it was read, and it never moves a term or
a root, so the occurrence ids a stored proof names are those of the file.
The parser interns the names as it reads them, so its table has distinct
names before its `optimize` already.

The public surface is read-only accessors (`terms`, `term`, `roots`,
`atom_names`, `atom_name`, `atom`, `formula`, and `occurrences`, the size
of the unfolding) plus `optimize` and `add`;
construction goes through the parser or serde. Tests inside the crate
build arenas as struct literals.

## One-sided, negation normal form

When parsing, terms on the left of `⊢` get negative polarity. Negation is
pushed down to atoms with `Term::dual()`, so there is no general negation
node, only `DualAtom`. `A ⊸ B` becomes `A^⊥ ⅋ B`. Printing therefore gives
`A |- A` as `⊢ ~A, A`. Intuitionistic sequents use the same model (plan
decision D1): an ILL sequent is a one-sided sequent of a particular shape,
read back by `Reading` (below); there is no second data model.

## Terms, kinds, fragments

- `Term` (`sequents/term.rs`) is the full classical connective set with
  the two binders `Forall(TermId)` and `Exists(TermId)` last; `Kind` is
  the same enum without payloads, for the forest's per-occurrence array and
  for tables keyed by connective. `Kind::polarity()` is `None` for literals
  because a literal's polarity is the per-atom bias (below); `Forall` is
  negative and `Exists` positive. A binder is unary (`arity` 1, its body
  the one operand, so the forest puts it at `o + 1`), `dual` swaps the
  two, and `Kind::is_binder` is the test the guards use
  (`Forest::has_binders`).
- There are no fragment-typed sequents. `Fragment` (`fragment.rs`) is a value:
  six connective-class flags (`QUANTIFIERS`, 32, set by a binder alone:
  a ground atom sets nothing, so `p(a) |- p(a)` is `MLL` and every
  engine decides it), the usual fragments as constants
  (`MLL`, `MLL_WITH_UNITS`, `ADDITIVE`, `MALL`, `MELL`, `LL`, and `MLL1`
  to `LL1` with the bit; `ADDITIVE`
  is additive-only and named `ALL`, `LL` is every propositional
  connective), `contains` as the subset order, and `Display` naming the
  smallest named fragment containing the value, a `1` after the acronym
  with the bit (`MLL1 with units`). The empty fragment (atoms
  only) prints as `MLL`. No engine's `admits` contains the bit, so the
  dispatch finds no row (`Error::NoEngine`, "no engine for MLL1 in
  classical mode yet") and a forced engine refuses by fragment; the
  checker (`CheckError::Quantifiers`), `Interactive::within`, cut
  elimination (`Error::Quantifiers`), the proof structures (`has_nets`
  is false with the bit) and the Rocq export (`Unsupported::Quantifiers`)
  refuse a forest with a binder before anything else, and
  `Disproof::check` answers `NotApplicable`; the `unreachable!` arms on
  `Kind::Forall | Kind::Exists` in the engines, the nets and the
  refuters rest on those guards. `NAMED` and `NAMES` are the named fragments and
  their names, which `FromStr` reads with or without the intuitionistic
  `I`; `has_nets` is the one test of where proof structures exist, which
  `ProofStructure` and the command ask (an engine's `admits` keeps its
  own largest fragment). `Sequent::fragment()` is the detection. `Mode`
  (same file) is what the user asks beyond the sequent: intuitionistic,
  affine, Mix, as `pub(crate)` fields behind getters (`is_intuitionistic`,
  `is_affine`, `has_mix`) and builders (`with_affine`, `with_mix`) on
  `CLASSICAL` and `INTUITIONISTIC`, so that a field added later breaks no
  caller; `check` refuses intuitionistic with Mix. `Mode::NAMES` is the
  one table of its words (`classical`, `affine`, `mix`, `affine-mix`,
  `intuitionistic`, `intuitionistic-affine`), which `name` writes and
  `FromStr` reads (`Error::UnknownName` lists them); the command's batch
  and the harness use it. `Display` is the prose the command prints.
  The dispatch's `Modes::take` destructures the whole mode, so a new
  field is a compile error there.
- **The order** (`Order`, `Mode::order`, `is_ordered`; `Mode::CYCLIC`, the
  word `cyclic`): every earlier mode is `Order::Commutative`, so its
  value, name and wire form are as before. `Mode::check` refuses an
  ordered mode with weakening or Mix (`Error::OrderedMode`), but `name`
  is total over the fields (`cyclic-mix`, `lambek-affine`, …) for the
  refused ones.
- **The Lambek calculus is the ordered intuitionistic mode**
  (`Mode::LAMBEK`, the word `lambek`; `with_empty_antecedents()`, the word
  `lambek-star`, `Display` "Lambek with empty antecedents"): `L*` is the
  intuitionistic part of cyclic MLL (Abrusci's embedding, which the
  ordered parse is), and `L` is `L*` with a hypothesis in every sequent.
  The field `empty_antecedents` means something only there:
  `with_empty_antecedents` sets it only in an ordered intuitionistic mode
  and returns every other mode unchanged, so no two values name one
  calculus; `allows_empty_antecedents()` is the one question code asks
  (true in every mode but `L`). The default is `L` (D16), the calculus
  the literature means by "the Lambek calculus". The front door
  and `Interactive` call `Mode::check` (before, they tested
  intuitionistic Mix alone, which it still answers first among the
  commutative modes). A sequent beyond unit-free MLL in an ordered mode
  is `Error::OrderedFragment`. A sequent's order needs no change of the
  arena: the written order was canonical already, and the forest's
  preorder (left operand first) is the cyclic order of the literals.

## Parsing

`core/src/parse/mod.rs` is a precedence parser written by hand: one loop
over the characters in two states (`operand`, what a formula starts with,
and `operator`, what follows an operand) with one explicit stack
(`Parser::pending`: the open parentheses, the prefix operators, and the
binary connectives with their left operands, each waiting for the operand
to its right). A term goes into the arena the moment it is complete, so
there is no syntax tree, nothing recurses, and nothing has a drop that
does: a formula nested 50 000 deep, or a chain that long, parses on a
stack of 256 KiB (`depth_costs_no_stack` in `core/tests/parse.rs`), in
time and memory linear in the text. What the code relies on:

- **The arena comes out in the order a recursive lowering gives it**:
  postorder, in the order of the text (an operand when it is read, a
  connective when its right operand is complete), the roots in the order
  of the text, the atoms numbered by first occurrence. `optimize()` at
  the end therefore gives the `Sequent` the first parser gave (a
  recursive descent that built a tree and lowered it), and with it the
  occurrence ids that every stored proof and snapshot names. A change to
  the order of the pushes is a format break.
- **The ordered parse** (`Sequent::parse_ordered(text, order, limits)`):
  in `Order::Cyclic` the dual of a product reverses its operands and the
  antecedents are reversed (`A, B |- C` is `|- ~B, ~A, C`), so that the
  cyclic calculus's reading of a two-sided sequent is the one-sided one
  in forest order; `A -o B` stays `~A par B` (Lambek's left division).
  The ordered parse alone reads the divisions `A \ B` (as `-o`, to the
  right) and `B / A` (`B par ~A`, to the left, the right operand
  negated in `reduce`), as loose as `-o`; mixed, `A \ B / C` is `A \ (B
  / C)`. The two arms sit behind an order guard in `operator`'s match,
  so the commutative reader refuses `\` and `/` as before and pays a
  compare only on those characters; `read-text` counted 12 468 102
  instructions after them.
  `FromStr` and `parse_within` stay the commutative reader. Both readers
  go through one function, `parse::read`, which makes the parser as its
  own local and runs the loop, so that the loop and all it calls have one
  caller and inline as before (`read-text` at its count before the
  order). Not so: the loop taken out of line with the parser passed in
  cost 1.5 %, and a copy of the loop per reader (inlined, or the parser
  generic over the order) 2 to 7 %, as a callee both copies call
  (`Parser::atom`, `HashMap::insert`) lost the inlining LLVM gives a
  single caller. `dualise` is generic over the order, chosen once in
  `finish`. A cyclic sequent prints one-sided as any other and
  reads back as itself under the ordered parse. The JSON form has no
  order: a JSON sequent is already lowered.
- **The record of a classical text is an addition to the one reader**:
  `read(text, limit, order, written)` fills `written` when given, and
  `parse::written` is the classical surface's entry
  (`ClassicalSequent::parse_within`). The parser checks `recording` once
  per `⊸` it reduces and `finish` once, so `parse_within` and the
  `read-text` journey pay a branch per `⊸`; the record itself is made
  after `dualise`, from `negated` and the implications, before
  `optimize` (core-forest.md, "The classical sequent as written").
- **Negation is applied at the end.** A term is pushed as it is written;
  a `~`, a `^`, being the antecedent of a `⊸` (which is pushed as `⅋`)
  and standing left of the turnstile each flip the term's flag in
  `Parser::negated`, and `finish` goes once from the last term to the
  first, dualises a flagged term (`Term::dual`) and passes the flag to
  its subterms. Before `optimize` every term has one parent, which is
  what makes the flag well defined.
- **Tokens depend on the state**: `par` is the connective where a
  connective can stand and an error where a formula starts (`|- par par
  par` is refused at the first); `bot` and `top` are constants only as
  whole identifiers; `forall` and `exists` (and `∀`, `∃`) where a
  formula starts open a binder (`Parser::binder`: the variables, then
  the dot, then one `Pending::Binder` per variable, each counted as an
  owed term like `!`, the names pushed on `Parser::scope`); `|-` is the
  turnstile only on the left side outside every
  parenthesis, and anywhere else a `|` before a `-` that starts no
  formula. An identifier starts with `_` or `XID_Start` and goes on with
  `XID_Continue` (the `unicode-ident` crate's tables, a dependency of
  every build: names enter through JSON and `ordinary::Formulas::atom`
  as well).
- **A binder binds loosest**: in `reduce` a `Pending::Binder` breaks the
  reduction when a connective follows (`next.is_some()`, the connective
  is inside the scope) and reduces at the scope's end (a `)`, a `,`, the
  turnstile, the end), popping its name from `scope` and recording it
  for the term pushed (`Parser::binders`); the negation pass carries a
  flag through it as through `!`, `dual` making the other binder of the
  same term, whose name stays. Printing brackets a binder's body when
  it is binary and a binder inside another formula (`notation.rs`,
  `bracketed`), so `∀x. (p(x) ⊗ q)` and `(∀x. p(x)) ⊗ q` read back as
  themselves.
- **A predicate's arguments** (`Parser::predicate`, after an identifier
  that a `(` follows at once): terms on an explicit stack of open
  applications, no recursion; an identifier in term position is the
  bound variable of the innermost open binder of that name
  (`FoTerm::Bound(index)`) or a symbol, interned through `fo::Interner`
  (a name's first use fixes its arity and its role; a later use at
  another arity or in the other role is a `ParseError` spanning that use
  with a static hint, `ARITY` or `ROLE`); a keyword in term or variable
  position is an error with the `KEYWORD` hint; a numeral is "expected a
  term". The atom's key is `FirstOrder::atom_key`, interned in `names`
  as a nullary name is. The text becomes first-order at its first
  predicate with arguments or binder (`enter_first_order`): every atom
  read before gets its nullary predicate then, and every nullary atom
  read after gets one as it is read, so `fo.atoms` is parallel to the
  table exactly when the text is first-order and a propositional text
  touches none of it (the `read-text` journey).
- **A name is read in NFC** (`name::normalized`, the
  `unicode-normalization` crate, a dependency of every build like
  `unicode-ident`): the parser keys its names by their NFC (`Cow`, an
  ASCII or composed name borrowed), the JSON reader composes each name
  before `merge_atoms`, and `ordinary::Formulas::atom` composes before
  its lookup; so `é` written composed and decomposed is one atom (H8,
  HD3), and a table's names are composed. A new reader of names
  composes them too.
- **One check of an atom name** (`sequents/name.rs`, `name::check`):
  an identifier that is no keyword (`par`, `top`, `bot`) and no
  quantifier's keyword (`forall`, `exists`), else `Error::AtomName` (code `atom_name`,
  malformed); `check_ordinary` refuses `true` and `false` besides, since
  an ordinary atom keeps its name in the linear image, which may hold
  minimal logic's atom `false`. The JSON sequent reader and `mist::read`
  call `check`, `ordinary::Formulas::atom` (so the ordinary text and
  TPTP readers) `check_ordinary`; the linear text parser and with it
  `lltp::read` refuse the same words as parse errors. A new reader of
  names calls it too, so that every sequent written as text reads back
  as itself (H19: a JSON atom named `top` printed as the unit).
- **The strict reading of intuitionistic text** is not this parser's:
  `IllSequent::parse_within` runs it and then `classical`, a scan for
  the classical tokens, so the parser and its journey are unchanged
  (`core-forest.md`). `ParseError::hint` (absent from the JSON when
  `None`) says what to write instead; `Display` puts it after the place.
- **An error is one `ParseError`**: the byte span of the first character
  that cannot go on a sequent, and that character, or the end of the
  input; the same place in UTF-16 code units (what an editor in
  JavaScript indexes) and as a line and a character, both counted from
  1, computed once in `ParseError::new`, which both parsers call; and
  `expected`, what could have stood there in words (`a formula`, `a
  connective`, `,`, `|-`, `)`, `the end`), which the parser's state
  decides (`operands`, `operators`). Two tokens of two characters make the exceptions the first
  parser made: a `-` that no `o` follows where a connective can stand,
  and a `|` that no `-` follows at the very start (where only the
  turnstile can stand), report the character after them.
  `error_positions` in `core/tests/parse.rs` pins both and the rest.
- A text of more terms than a forest can hold (`Forest::MOST`) is
  `Refusal::Occurrences`: every term of a text is an occurrence. A
  pending `!` or `?` counts as the term it will be (`Parser::owed`) and
  two `~` in a row cancel as they are read, so a run of prefix operators
  is refused at the bound, not held to its operand (a run of 30 million
  `!` once took 9 bytes a byte under a limit of 1 000). A waiting `(`
  or `~` makes no term but counts as a mark (`Parser::marks`), and more
  marks than the bound are refused as occurrences are: a text nested
  deeper than its bound is refused though its terms would fit (none of
  the default's fifty million is near), and a run of `(` held 9 bytes a
  byte before.
- The first parser was chumsky's Pratt parser. Before it was removed
  the two were compared on 1.1 million generated inputs (token soup,
  random sequents in every spelling, and those with one character
  removed, replaced or added; 481 000 sequents and 619 000 errors): the
  same `Sequent` or the same error span and character on every one. The
  test, `agrees_with_the_first_parser`, is in the history with the
  change that introduced this parser.

Every operator has ASCII and Unicode spellings: `* ⊗`, `| par ⅋`, `&`,
`+ ⊕`, `-o ⊸`, prefix `~ ! ?`, postfix `^`, and `|-`/`⊢`. The constants are
`0`, `1`, `bot ⊥` and `top ⊤`. Precedence, from tightest: `^` > `~ ! ?` > tensor > par > with > plus > lollipop (right-associative).
`core/tests/parse.rs` pins this behaviour through the public API.

## Serialization

**The wire level** (`core/src/wire.rs`, public, behind `serialize`):
every top-level document starts with `"version": n`, the lowest level
that holds it: 1 for everything 0.1.0 wrote, 2 for a disproof or an
unprovable outcome with the classical refutation (`wire::level_of`;
`wire::BASE` is 1, `wire::LEVEL`, the highest read, 2; the command's own
records write their body's level, `check`'s JSON level 1), nested
values carry none; a reader refuses a higher one by name
(`Error::Version`, code `unsupported_version`) through `wire::version`,
the `deserialize_with` of every proxy's `version` key, and reads a
document without one as level 1. Every reader goes through
`wire::Readable::read(deserializer, &limits)`: `wire::upgrade` (the one
reader, answering an `Error`), `wire::Within` (the same as a serde seed)
and each plain `Deserialize` (within the default `Limits`); at level 1
`upgrade` is the identity, and a level that changes a form adds one step
there. A deserializer's error carries text only, so `wire::fail` keeps
the library's own error (a refusal, an index out of bounds, the version)
in a thread-local that `upgrade` answers instead of `Error::Json`.
`wire_levels` in `core/tests/serialize.rs` pins all of it. The names
from before the release (`ids`, `var_dict`, `proof`, a mode as an object
of flags) are not read: a file in them is refused naming the key it
lacks.

`core/src/serialize/sequents.rs` uses a private serde proxy struct
`{version, terms, roots, atoms, symbols, fo, antecedents, cuts}`
(`antecedents` written
whenever the sides are known, `0` included, and absent otherwise) with
short tags (`V`, `D`, `⊗`, `⅋`, …, `∀` and `∃`, whose variables' names
are the `binders` key, one per binder in the order of the terms, so that
`Expression` stays `Copy` and three words) and `u32` indices; `Sequent::from`
is the nested form, without `version`, which every other proxy holds.
**Level 7 is the first-order sequent** (`wire::TERMS`,
`level_of_sequent`, 7 whenever the tables or the binders' names are not
empty): `symbols` and `fo` (`{"B": n}`, `{"F": [symbol, [terms]]}`) are
written only when an atom is a predicate, and then every atom is
`{"P": [symbol, [terms]]}` so that the tables read back equal; the
binders' names are the `binders` key, one per `∀` or `∃` term in the
order of the terms (`x` where one is missing), kept beside the terms
and not inside them so that `Expression` stays `Copy` and three words;
a propositional sequent's bytes are level 1's. `Atoms` reads the atoms
as a plain list of strings (`Atoms::Names`, the propositional path
exactly as before) and turns mixed (`AtomForm`, a string or `{"P": …}`,
read without buffering) at the first predicate; `Tables::of` and
`Tables::read` (shared with `serialize/ill.rs`, whose document takes the
larger of level 5 and the sequent's) build the forms and the tables: the
reader interns equal lists as the parser does, takes a plain name beside
a predicate as the nullary symbol of that name, keeps a document of
names and binders alone as names with its binders (so `fo.atoms` empty
and `fo.binders` not is a state a JSON document can hold, never a
parsed one), checks the tables (`FirstOrder::check`, then
`Sequent::check` with the scope pass), counts the terms every atom
unfolds to against `limits.occurrences` (shared terms could name a key
of exponential length) and computes every atom's key;
`core/tests/wire/level-7.txt` pins a one-sided, a ground and an
intuitionistic document. The `read-text`, `read-json` and the render
journeys of the ratchet are within 2 % of their ceilings only with the
parser's and the printers' first-order paths out of line (`#[cold]`,
`#[inline(never)]`: `binder`, `predicate`, `nullary`, `close_binder`,
`write_predicate`, `Scope::enter`), the printers' scope made at the
first binder and dropped only then (`Slot`, `close`), one map lookup per
atom (`Parser::atom` through `entry`) and `Term`'s hash as one write.
`serialize/proofs.rs` does the same for proofs: `{"version": 1,
"sequent": …, "nodes": [node, …], "mode": …, "goal": […]}` (the last
two where the proof records them), one object per node tagged `ax ⊗ ⅋ 1
⊥ & ⊕₁ ⊕₂ ⊤ ! ? copy wk mix` with the occurrence ids and premise
indices as an array (or one integer), premises before conclusions and the
root last. Deserialization rebuilds the forest within the limits, checks
bounds and order and drops unreachable nodes; whether the proof is
correct is `Proof::check`'s question, in the mode the caller names. Both are interchange formats for the CLI and the planned
web front end, so a tag or key change is a format break;
`core/tests/serialize.rs` pins the exact strings, and the behaviour lock
`core/tests/lock.rs` pins every JSON form on a corpus (sequents, every
verdict, refutation and reason of an outcome, proofs of every mode,
proof structures, sessions) in `core/tests/lock/json.txt`, rewritten
with `BLESS=1` in a commit of its own that says why.
`core/tests/wire/level-1.txt` is the lock's 33 lines as 0.1.0 wrote them,
then the disproof of each of the five refutations and a proof and a
disproof of a goal off the roots (written once by a scratch test from the
same code), and is never rewritten: `core/tests/stability.rs` reads
every line of a form that is read back through `wire::upgrade` and writes
it again byte for byte (an outcome only as its proof or disproof), so a
release that loses a level-1 document, or writes a level-1 value
otherwise, fails there; a new level adds a file `level-N.txt` beside it
(`level-2.txt`: the classical disproof and outcome, read by
`the_second_level_reads`).
Until 0.1.0 is tagged its first 33 lines must still equal the lock: a
change of a written form before the tag rewrites both. A binary format would come
from the same proxies (postcard encodes the variants by index), in the crate
that wants it.

`serialize/search.rs` gives the search's values a wire form, pinned in the
same test file:
- `Fragment` is its name (`"MALL"`, `"MLL with units"`), not its flags, so a
  human reads it. Deserializing a name gives the named fragment, which
  contains every fragment of that name: the trip is lossy towards larger,
  which as an assertion only switches off prunes, never refuses a sequent
  it came from.
- `Mode` is its name (`Mode::NAMES`: `"classical"`, `"affine-mix"`,
  …), an unknown one refused naming the known ones.
- `Outcome` serializes only (it is output): `version`, `linlog` (the
  crate's version; the command's lock and README's test read it as `…`),
  `verdict` (`proved`, `unprovable`, `unknown`), `checked` for `proved`
  (`Outcome::checked`), `reason` for `unknown` (tagged by `kind`:
  `stopped`, `recursion_limit {depth}`, `copy_bound {copies}`,
  `memory_limit {limit_bytes}`, `index_limit`), `fragment`, `mode`, `engine`,
  `statistics`, and for `proved` the proof's own `sequent`, `nodes` and
  `goal` keys (`Proof::keys`, no `version` and no `mode` of their own:
  the outcome's are the proof's), flattened, so that the whole outcome
  reads as a `Proof` (a data form ignores the other keys) and `linlog
  check` reads the output of `linlog prove --format json`; for
  `unprovable` the `Disproof`'s keys beside `mode`: `refutation` (tagged
  by `kind`: `exhausted`, `unbalanced {atom, least, most}`, `one_sided
  {atom, least, most, copied}` (`copied` the literal `"atom"` or
  `"dual"`: a kind of its own, not a key added to `unbalanced`, since a
  key never brings a new meaning to an old tag) with the
  atom's index, `equation {…}`, `state_equation {atoms: [[atom,
  weight]], clauses: [[occurrence, weight]], dropped: [occurrence]}`,
  `saturated {positive: ["atom" | "dual"], sequents: [{linear,
  unrestricted, absorbs}]}` with occurrences of the forest, wire level
  3, `wire::level_of`),
  `sequent` after `statistics`, and `goal` for a goal off the roots,
  so that an unprovable outcome reads back as a `Disproof`, whose own
  document is `{version, sequent, mode, refutation, goal}`
  (`DisproofForm`): reading counts the sequent against the limits and
  checks every atom and occurrence the refutation and the goal name,
  not the refutation, which a checker of refutations will
  (`disproof_json_format_and_round_trip`).
  A new `Reason` or `Refutation` variant needs its arms in the proxy,
  both ways (`Why`, `WhyNot`; `serialize` denies wildcard arms, so a
  variant missing there does not compile); a `Reason` reads back by its
  `kind`, and a kind the proxy does not know is refused, never read as
  another; `Statistics` derives its form
  (`serde(default)`, read back too), so a new counter is written by
  itself;
  `Outcome::net` is not serialized (the proof's keys are, and the net is
  `from_proof` of them).
- `Forest` has no serde; it is rebuilt from the sequent.

`serialize/interactive.rs` writes an `Interactive` as `{"version": 1,
"sequent": …, "mode": "intuitionistic", "inferences": [{"sequent": [0, 1, 4], "rule": "⊸L",
"principal": 1, "premises": [1, 2]}, {"sequent": [3, 4]}, …], "history":
[0]}`: the inferences in the state's own top-down order, an open goal as
its sequent alone (`rule`, `principal` and `premises` absent), rule names
as `Rule::name` (`Rule` itself serializes as its name, in
`serialize/proofs.rs`), and the history as the inferences the steps
closed. Reading it back goes through `Interactive::from_parts`, which
replays every closed inference. It is the form a web client holds between
requests, so it is pinned in `core/tests/serialize.rs`.

`serialize/ill.rs` writes an `IllSequent` as `{"version": 5, "terms":
[…], "atoms": […], "hypotheses": […], "goal": n}`, always at level 5
(`wire::ILL`): an arena of ILL terms (`V`, `1`, `0`, `⊤`, `⊗`, `⊸` with
the antecedent first, `&`, `⊕`, `!`), written from the reading in the
written order of each root, a term on one side once and a formula once
(a one-sided `~B` hypothesis-side and a goal's `B` are one `V`).
Reading checks the names, the bounds and order (through the one-sided
`Term::check_bounds` of the same shape), refuses an arena whose lowering
could pass a `u32` of terms, lowers each term once per side it is needed
on, counts the occurrences against the limits, and lowers through
`IllSequent::lowered`, which records the sides. A classical tag is an
unknown variant. `core/tests/wire/level-5.txt` holds one such document.

`serialize/nets.rs` writes a `ProofStructure` as `{"version": 1,
"sequent": …, "mix": false, "links": [[0, 2], [3, 4]]}` (the criterion's
fields flattened), the links as vertex id pairs in
the order they were made; reading validates the links as `from_links`
does and accepts a partial or incorrect structure, since whether it is a
net is `is_correct`'s question.

- **Level 6 is the cut** (`wire::CUTS`, `wire::level_of_sequent`): a
  sequent with cut formulas writes `"cuts": [a, …]`, the term of each
  `A` (its dual is in the arena, which `push_cuts` finds again on
  reading, so the arena reads back as it was), and every form that holds
  such a sequent (proof, session, outcome, disproof) takes the larger of
  its own level and 6; a proof's cut node is `{"cut": [a, l, r]}` and a
  session's cut inference names `"cut": a`. Without cut formulas no key
  is written and every byte stays as it was. `wire/level-6.txt` pins a
  proof and a session of level 6.

## Decisions

The author's answers for the release (`plan/notes/api.md` §14), which
the fixes implement and later rounds judge against. Where a bullet above
still describes code that a decision changes, the decision holds, and
the commit that lands it rewrites that bullet.

- **The written order is canonical**: `optimize` merges and drops but
  never sorts the roots; the parser, the JSON, `add`, `Display` and
  `Forest::roots()` keep them as given, and `Eq` and `Hash` are
  structural. A `Sequent` then means one thing in every mode, and the
  ordered calculi need the order.
- **The written sides are kept**: every reader sets `antecedents`
  (`Some(0)` for `|- Γ`), and the JSON writes it whenever it is `Some`,
  `0` included; `None` only where no sides were given. The intuitionistic
  reading takes the goal from it (`core-forest.md`).
- **An atom is an interned atomic formula**: the atom table holds one
  distinct key per atom, and `atom_name` is that key. `Term` and `Kind`
  gain only `Forall` and `Exists`, and their literal variants stay the
  first two, contiguous, with every new variant after the compound ones.
  Ground first-order atoms are then propositional by construction, and
  `Term::atom()` and `Kind::is_literal` stay one comparison on the
  focused engine's hot path (literal variants appended after `Quest`
  cost its journeys 6.6 %).
- **Atom names are identifiers of the text syntax**: normalized to NFC
  on every path, never a keyword (`par`, `top`, `bot`) or a reserved word
  (`forall`, `exists`). Every reader refuses them: `AtomName`, or a parse
  error that names the word as reserved. The text syntax grows by new
  tokens only, never by a new meaning of text that reads today.
- **One global wire level**: every top-level document starts with
  `"version"`, the lowest level whose reader understands it, and a reader
  refuses a higher one by name. `wire::upgrade` is the one place that
  knows older released levels: every reader goes through it, and each
  level that changes a form adds one step to it; at level 1 it is the
  identity. A client stores one number, a reader keeps one grammar, and a
  stored file outlives the release that wrote it.
- **No reader of the pre-release names** (`ids`, `var_dict`, `proof`, a
  mode as an object of flags). Nothing is kept beside its successor
  before the release, and a second grammar would be fuzz surface for
  ever. The keys that every level-1 value has are required, so an old
  file is refused naming the key it lacks.
- **A mode on the wire is its name**: an unknown word is refused naming
  the known ones, so a new mode cannot be read as a commutative one.
- **Numbers on the wire**: a written-only count that saturates is
  `u64::MAX` and means "at least"; every integer read back is below 2⁵³,
  and a larger bound is written `null`. JavaScript reads `u64::MAX` as
  2⁶⁴, which no exact count takes.
- **Ordered modes** (built: `Mode::CYCLIC`, `Mode::LAMBEK`, `parse_ordered`; written at
  wire level 2 wherever a document names the mode or a structure the
  order, `wire::level_in`): classical cyclic MLL is read by an ordered parse
  beside `parse_within` whose dual reverses products, and `FromStr` stays
  the commutative reader; the Lambek calculus is read by the same
  reversing parse (not by an order derived from the commutative lowering
  through the reading), so one order, the forest's, serves both ordered
  modes, and its two-sided reading is `Reading::of_mode`. Without the
  reversing dual, `|- ~(a * b), a, b` would be unprovable in cyclic mode.
