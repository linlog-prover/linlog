---
paths:
  - "core/src/sequents/**"
  - "core/src/parse/**"
  - "core/src/fragment.rs"
  - "core/src/serialize/**"
  - "core/tests/parse.rs"
  - "core/tests/serialize.rs"
  - "core/tests/lock.rs"
---

# linlog core: sequents, parsing and serialization

Loaded, beside `core.md`, when a file of the sequents, the fragments,
the parser or the serialization is read.

## Sequents are arena-allocated DAGs

`Sequent` (`core/src/sequents/mod.rs`) has three fields, all `pub(crate)`:
- `terms: Vec<Term>`: every subformula. Children are referenced by arena
  index (`TermId`, a `u32` newtype), never by pointer.
- `roots: Vec<TermId>`: the root formulas that make up the sequent, in the
  order the sequent lists them.
- `atoms: Vec<String>`: atom names. `Var(a)`/`DualVar(a)` index into this
  with `Atom`, a `u32` newtype.

**A term only references subterms with a strictly smaller index**, so the
arena is topologically sorted: one ascending pass sees every subterm before
its parents, one descending pass sees every parent before its subterms
(`fragment()` and the forest's size computation rely on the latter).
`verify_integrity()` checks it, `Formula` printing debug-asserts it, and
deserialization runs the check. Code that builds or rewrites an arena must
preserve it.

`optimize()` runs after parsing. It deduplicates atom names, hash-conses
identical terms, drops unreachable ones and sorts `roots`. Nothing else
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
of the unfolding) plus `optimize`, `add` and `verify_integrity`;
construction goes through the parser or serde. Tests inside the crate
build arenas as struct literals.

## One-sided, negation normal form

When parsing, terms on the left of `⊢` get negative polarity. Negation is
pushed down to atoms with `Term::dual()`, so there is no general negation
node, only `DualVar`. `A ⊸ B` becomes `A^⊥ ⅋ B`. Printing therefore gives
`A |- A` as `⊢ ~A, A`. Intuitionistic sequents use the same model (plan
decision D1): an ILL sequent is a one-sided sequent of a particular shape,
read back by `Reading` (below); there is no second data model.

## Terms, kinds, fragments

- `Term` (`sequents/term.rs`) is the full classical connective set; `Kind` is
  the same enum without payloads, for the forest's per-occurrence array and
  for tables keyed by connective. `Kind::polarity()` is `None` for literals
  because a literal's polarity is the per-atom bias (below).
- There are no fragment-typed sequents. `Fragment` (`fragment.rs`) is a value:
  five connective-class flags, the usual fragments as constants
  (`MLL`, `MLL_WITH_UNITS`, `ALL`, `MALL`, `MELL`, `LL`; `ALL` is
  additive-only, `LL` is everything), `contains` as the subset order, and
  `Display` naming the smallest named fragment containing the value. The
  empty fragment (atoms only) prints as `MLL`. `Sequent::fragment()` is the
  detection. `Mode` (same file) is what the user asks beyond the sequent:
  intuitionistic, affine, Mix; three bools with builder methods.

## Parsing

`core/src/parse/mod.rs` is a precedence parser written by hand: one loop
over the characters in two states (`operand`, what a formula starts with,
and `operator`, what follows an operand) with one explicit stack
(`Parser::pending`: the open parentheses, the prefix operators, and the
binary connectives with their left operands, each waiting for the operand
to its right). A term goes into the arena the moment it is complete, so
there is no syntax tree, nothing recurses, and nothing has a drop that
does: a formula nested 100 000 deep, or a chain that long, parses on a
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
- **Negation is applied at the end.** A term is pushed as it is written;
  a `~`, a `^`, being the antecedent of a `⊸` (which is pushed as `⅋`)
  and standing left of the turnstile each flip the term's flag in
  `Parser::negated`, and `finish` goes once from the last term to the
  first, dualises a flagged term (`Term::dual`) and passes the flag to
  its subterms. Before `optimize` every term has one parent, which is
  what makes the flag well defined.
- **Tokens depend on the state**: `par` is the connective only where a
  connective can stand and a variable where a formula starts (`|- par par
  par` is `par ⅋ par`); `bot` and `top` are constants only as whole
  identifiers; `|-` is the turnstile only on the left side outside every
  parenthesis, and anywhere else a `|` before a `-` that starts no
  formula. An identifier starts with `_` or `XID_Start` and goes on with
  `XID_Continue` (the `unicode-ident` crate's tables).
- **An error is one `ParseError`**: the byte span of the first character
  that cannot go on a sequent, and that character, or the end of the
  input. Two tokens of two characters make the exceptions the first
  parser made: a `-` that no `o` follows where a connective can stand,
  and a `|` that no `-` follows at the very start (where only the
  turnstile can stand), report the character after them.
  `error_positions` in `core/tests/parse.rs` pins both and the rest.
- A text of more terms than a forest can hold (`Forest::MOST`) is
  `Error::TooManyOccurrences`: every term of a text is an occurrence.
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

`core/src/serialize/sequents.rs` uses a private serde proxy struct
`{terms, ids, var_dict}` with short tags (`V`, `D`, `⊗`, `⅋`, …) and `u32`
indices. `serialize/proofs.rs` does the same for proofs: `{"sequent": …,
"proof": [node, …]}`, one object per node tagged `ax ⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ?
copy wk mix` with the occurrence ids and premise indices as an array (or
one integer), premises before conclusions and the root last. Deserialization
rebuilds the forest, checks bounds and order and drops unreachable nodes;
whether the proof is correct is `Proof::check`'s question, since the mode
is not in the file. Both are interchange formats for the CLI and the planned
web front end, so a tag or key change is a format break;
`core/tests/serialize.rs` pins the exact strings, and the behaviour lock
`core/tests/lock.rs` pins every JSON form on a corpus (sequents, every
verdict, refutation and reason of an outcome, proofs of every mode,
proof structures, sessions) in `core/tests/lock/json.txt`, rewritten
with `BLESS=1` in a commit of its own that says why. A binary format would come
from the same proxies (postcard encodes the variants by index), in the crate
that wants it.

`serialize/search.rs` gives the search's values a wire form, pinned in the
same test file:
- `Fragment` is its name (`"MALL"`, `"MLL with units"`), not its flags, so a
  human reads it. Deserializing a name gives the named fragment, which
  contains every fragment of that name: the trip is lossy towards larger,
  which as an assertion only switches off prunes, never refuses a sequent
  it came from.
- `Mode` is `{"intuitionistic": …, "affine": …, "mix": …}`.
- `Outcome` serializes only (it is output): `verdict` (`proved`,
  `unprovable`, `unknown`), `reason` for `unknown` (a snake_case tag,
  `{"copy_bound": n}`, `{"memory_limit": bytes}`, `"index_limit"`),
  `fragment`, `mode`, `engine`, `statistics`,
  and for `proved` the proof's own `sequent` and `proof` keys, flattened, so
  that the whole outcome deserializes as a `Proof` (serde ignores the other
  keys) and `linlog check` reads the output of `linlog prove --format json`.
  A new `Reason` variant or `Statistics` field needs its line in the proxy;
  `Outcome::net` is not serialized (the proof's keys are, and the net is
  `from_proof` of them).
- `Forest` has no serde; it is rebuilt from the sequent.

`serialize/interactive.rs` writes an `Interactive` as `{"sequent": …,
"mode": …, "inferences": [{"sequent": [0, 1, 4], "rule": "⊸L",
"principal": 1, "premises": [1, 2]}, {"sequent": [3, 4]}, …], "history":
[0]}`: the inferences in the state's own top-down order, an open goal as
its sequent alone (`rule`, `principal` and `premises` absent), rule names
as `Rule::name` (`Rule` itself serializes as its name, in
`serialize/proofs.rs`), and the history as the inferences the steps
closed. Reading it back goes through `Interactive::from_parts`, which
replays every closed inference. It is the form a web client holds between
requests, so it is pinned in `core/tests/serialize.rs`.

`serialize/nets.rs` writes a `ProofStructure` as `{"sequent": …, "mix":
false, "links": [[0, 2], [3, 4]]}`, the links as occurrence id pairs in
the order they were made; reading validates the links as `from_links`
does and accepts a partial or incorrect structure, since whether it is a
net is `is_correct`'s question.
