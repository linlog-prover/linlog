# The register of later requirements

This register collects what each later step of the plan (steps 29 to 38), plan/later.md and the decisions D15 to D22 need of the library, so that step 28's audit and design can take them all into account and no later step needs a second rewrite. The requirements were extracted one source at a time and then merged: requirements that ask the same of the same item are one entry here, and that entry lists every step and every source that needs it.

How to use it. The audit judges the code against the register, entry by entry. The design answers each entry: it says where the requirement is met, or how the chosen structure leaves room for it. At the end of step 28 every entry is met or is recorded with its reason for not being met. The identifiers R1, R2 and so on are stable and follow the order of this file, so a report or a commit message may cite them.

How an entry is read. Each entry has a title, the steps that need it, the sources (file and line of the plan text that states the need), the API item it concerns, the requirement itself, how a later session knows it is met, and its state now with the evidence found when the register was made. The state is one of: met (the code already does it), partly (some of it is there), not met, or unknown (the extraction could not settle it from the code). The evidence names files and lines as they were when the register was written, so check them before relying on a line number.

## Summary

| Kind | Requirements | Met | Partly | Not met | Unknown |
|---|---|---|---|---|---|
| Wire forms | 18 | 0 | 5 | 13 | 0 |
| Bounds and stops | 25 | 0 | 9 | 15 | 1 |
| Wasm portability | 9 | 1 | 4 | 3 | 1 |
| Data model | 41 | 1 | 13 | 27 | 0 |
| Engine interface | 32 | 1 | 8 | 23 | 0 |
| Proof term and checker | 14 | 0 | 3 | 11 | 0 |
| Errors | 10 | 0 | 5 | 4 | 1 |
| Options | 13 | 1 | 5 | 7 | 0 |
| Export | 21 | 3 | 3 | 15 | 0 |
| CLI | 12 | 1 | 2 | 9 | 0 |
| Harness | 34 | 5 | 12 | 16 | 1 |
| Docs | 16 | 0 | 4 | 12 | 0 |
| Other | 9 | 0 | 2 | 6 | 1 |
| Total | 254 | 13 | 75 | 161 | 5 |

The 356 requirements extracted from the sources became 239 after merging. A completeness review then added R240 to R242 (they stand in the section of their kind, after the entries that were there), corrected the cited lines of the entries whose sources did not say what the entry needs, and moved six entries from met to partly (R33, R34, R127, R128, R163, R216): a state of met means the code already does it, not that the code to come has nothing to break. A second review then checked the register against the plan, the research notes of 2026-10-08 (plan/notes/research/) and the code: it added R243 to R254 at the end of the sections of their kinds, amended entries in place where they were incomplete or contradicted the research, and added the section Conflicts for the author, to which the entries concerned point with a line "Conflict".

## Wire forms

### R1. A JSON form for the search options and the batch options
- Steps: 32 web client; 38 first-order; every wrapper (D15, D22); batch mode (done in step 24).
- Sources: plan/32-web.md:31-33; plan/later.md:28,131-137; plan/later.md:555-558; plan/later.md:1209-1211,1257-1259; plan/README.md:685-700; plan/38-first-order.md:15-22.
- API item: search::Options, search::batch::Options, Engine and Bias (reading them by name).
- Requirement: The search options and the batch options are one plain-data value with a JSON form behind the serialize feature, as the export options already have. The form reads missing keys as the defaults and refuses unknown keys. Every setter is reachable from JSON, Engine, Bias and Fragment are written and read by name, a memory limit of none is null, and the runtime pool handle is left out. The form must exist before the first-order bounds (R38) are added, so that they are not an afterthought, and the command-line flags map onto the same value. Engine needs a way to be read from its name in the core crate, so that the command, the harness and the web client do not each keep a name table.
- Met when: A round-trip test in core/tests/serialize.rs shows that the default options written to JSON and read back are equal, that an empty object reads as the default, that a misspelt field is an error, and that the form is pinned like the other wire forms. The same holds for the batch options. Engine::from_str inverts Display.
- State now: Not met. Evidence: core/src/search/mod.rs:850-851 derives only Clone, Debug, PartialEq and Eq and has private fields. Engine has Serialize only (core/src/serialize/search.rs:265) and Bias has no serde. core/src/search/batch.rs:54 Options has no serde although Cores has (batch.rs:36). The export options and ViewOptions do have serde. plan/28-audit-and-refactor.md:76-77, 104-105 and 453-456 assign the wire form of the options values to step 28.

### R2. The outcome JSON stays stable and complete, with the refutation named
- Steps: 29 comparison; 32 web client.
- Sources: plan/29-comparison.md:41-46; plan/32-web.md:31-33,66-69.
- API item: Outcome JSON in core/src/serialize/search.rs, Verdict, Reason, Refutation, Statistics.
- Requirement: The outcome JSON carries verdict, reason, refutation, fragment, mode, engine and statistics, plus the new work-budget reason and counters (R21). An unprovable verdict carries the Refutation that names why, so a disputed answer in the comparison can be inspected and the contradiction list can cite it. An outcome from Interactive::close or close_all, whose proof is of a goal and not of the roots, must not be written as a proof file that would not check, or must say so.
- Met when: Pinned strings in core/tests/serialize.rs cover every Verdict, Reason and Refutation, and a test covers the JSON of a goal outcome. The contradiction list prints the refutation next to the verdicts.
- State now: Partly. Evidence: core/src/serialize/search.rs:196-246 writes verdict, reason, refutation, fragment, mode, engine and statistics and always flattens the proof. Outcome is output only. Interactive::close returns the goal's proof (core/src/proofs/interactive.rs:816-830), whose root does not conclude the sequent, so Proof::check rejects it. There is no work-limit reason yet.

### R3. Refutations read back, and the refutation conversion has no wildcard
- Steps: 31 Rocq library (certified refutations); 32 web client; D15 and D22.
- Sources: plan/later.md:473-477,491-497,503-510; plan/README.md:685-700,752-778; plan/notes/research/README.md (section 3, conflict 9); plan/notes/research/32-web.md (section 3, JSON wire forms); plan/notes/research/refutations.md (section 3, The refutation type).
- API item: Refutation, Reason, Outcome, Engine, Statistics and the WhyNot proxy in core/src/serialize/search.rs.
- Requirement: The conversion from Refutation to its wire form has no wildcard arm, so that a new refutation variant is a compile error in the proxy and never silently written as an exhausted search. Refutation (with every payload struct) and Reason read back (Deserialize, with the JSON version of D18), so that a refutation can be stored, sent between a worker and the client, and re-checked later by `linlog check` (R124). A stored refutation names the sequent and the mode it holds for (R71), since the outcome JSON carries neither. Outcome (and with it Statistics) stays write-only, and its rustdoc says so: it is built from a verdict plus the search's fragment, mode, engine and statistics, and reading it back would be a second parser of the same data with no consumer. Engine reads back through FromStr (R86).
- Met when: There is no `_` arm in the conversion from a Refutation to WhyNot. Refutation and Reason have round-trip and pinned-string tests for each variant in core/tests/serialize.rs. Outcome's rustdoc says that it is written only.
- State now: Not met. Evidence: core/src/serialize/search.rs, the conversion from a Refutation, ends with `_ => WhyNot::Exhausted`. The WhyNot and Why proxies (lines 75 and 115) derive only Serialize, and so do Out (line 242) and Engine (line 265). Fragment, Mode, Proof, Sequent, ProofStructure and the interactive state do read back.

### R4. A JSON form for the derivation view
- Steps: 32 web client; third wrapper (D15, D22).
- Sources: plan/README.md:662-671,685-700,752-778; plan/later.md:696-699; plan/32-web.md:66-69.
- API item: proofs::Derivation, Inference, InfId, and the answers of Interactive::goals, goal and rules.
- Requirement: A derivation (the tree of inferences with sequent ids, rule, principal, premises and times, over a forest) and the answers of an interactive session's queries (open goals with positions, the rules that apply at a position) have a JSON form, so that a client can draw its own tree or list its own buttons without scraping text. It names the sequent so that the ids mean something, as Proof does. Alternatively the documentation of Derivation and of the bindings states that the proof plus the view options is the form and that the derivation is always rebuilt.
- Met when: Derivation, Inference and InfId serialize, with a pinned test in core/tests/serialize.rs and a JSON section on the type, or the documentation records the decision that it is never sent.
- State now: Not met. Evidence: core/src/proofs/derivation.rs:542-576 Inference derives only Clone, Debug, PartialEq and Eq, and Derivation borrows its forest with no serde. Only ViewOptions and Compact (derivation.rs:354,379), Rule (serialize/proofs.rs:162) and the session state (serialize/interactive.rs:14-33) have serde.

### R5. The interactive session state stays pinned and round-trips every state
- Steps: 32 web client.
- Sources: plan/32-web.md:31,66-69; plan/later.md:694-698.
- API item: Interactive (apply, undo, close, close_all, derivation, proof) and serialize/interactive.rs.
- Requirement: The session JSON is the state a client holds between calls and may persist, so it stays pinned and round-trips every state, including states with grafted close steps and history. The refusal of a rule, the list of rules and the result of close are what the client shows next to the picture, so they have a JSON form as well (see R129 for the refusal). A state saved by an older build is refused with a clear error and never misread (see R6).
- Met when: Pinned JSON exists for the rules list and the close outcome. A round-trip test through serde at every step of a scripted session (cli/tests/cli.rs has the scripted session as the reference).
- State now: Partly. Evidence: core/src/serialize/interactive.rs:90-100 implements Serialize and Deserialize for the state, writing sequent, mode, inferences and history and reading back through from_parts, which replays every closed inference; it is pinned in core/tests/serialize.rs. Refusal (core/src/proofs/interactive.rs:39) has no serde and Outcome is serialize only.

### R6. The compatibility policy of every JSON form
- Steps: 28 audit; 30 release; 31, 32, 33, 34, 36 and 38 (every step that adds a rule, term or node); D18, D22.
- Sources: plan/28-audit-and-refactor.md:453-456 ("each JSON form with a version and documented schema"); plan/32-web.md:31; plan/later.md:28,332-334; plan/later.md:696-707 (with .claude/rules/core-sequents.md); plan/README.md:721-727,752-778.
- API item: serialize/: Sequent (terms, ids, var_dict), the Proof node tags, ProofStructure (sequent, mix, links), Outcome, the Interactive state, Mode, Fragment and the options values.
- Requirement: One stated rule for extending every JSON form without breaking a file or snapshot written earlier. New node tags, term tags, reasons, refutations and fields are additions, with serde defaults and skipped when empty, and propositional output stays byte-identical. A reader of the new version reads old files. The version of a form is either carried in the file (top-level objects a client persists) or declared never to need one. A file with a tag from a newer library fails with a message that names the tag, not a bare "invalid". A format break is a commit of its own. The same rule covers a state saved by an older build of the interactive session.
- Met when: A paragraph in the crate documentation and in core-sequents.md states the rule and the unknown-tag behaviour. The pinned strings in core/tests/serialize.rs stay the contract, a test shows that old fixtures (the proof files of 2026-09-30) still load, and a test shows that a proof file with an unknown tag is rejected with that tag in the message. Mode and ProofStructure read old JSON after a field is added.
- State now: Partly. Evidence: core/tests/serialize.rs pins exact strings and core/src/serialize/proofs.rs:10-12 says that renaming a tag breaks the interchange format. The proofs form is a closed Step enum with fixed tags (proofs.rs:11-58). No version key exists in any form (grep for version, schema and format in core/src/serialize finds none). The ModeDef proxy (serialize/search.rs:48) has no serde default, so a new flag would break reading old files. No unknown-tag policy is written.

### R7. The mode travels with a proof that is to be certified
- Steps: 31 Rocq library (the new kernel).
- Sources: plan/later.md:386-397.
- API item: Proof JSON, Mode, Outcome.
- Requirement: A certificate needs the mode (classical, affine, Mix, intuitionistic) together with the term, but a proof file does not carry it. The library offers one value or file form that holds sequent, term and mode together, either an optional mode key that reading a Proof accepts and returns, or a pair of proof and mode, so that `linlog check`, the exporter and the web client do not each guess it.
- Met when: A documented JSON form with the mode is pinned in core/tests/serialize.rs, reading a file without the mode stays valid, and the exporter takes the pair.
- State now: Partly. Evidence: the Outcome JSON writes the mode beside the flattened proof keys (core/src/serialize/search.rs), but the Proof deserializer ignores it; .claude/rules/core-sequents.md:146-149 says the mode is not in the file, and core/src/serialize/proofs.rs Prf has only sequent and proof.

### R8. The proof-structure JSON is extensible without breaking old files
- Steps: 33 MELL nets; 38 first-order; MALL nets (only if a use case appears).
- Sources: plan/33-mell-nets.md:20-21 and plan/28-audit-and-refactor.md (behaviour lock pins the ProofStructure JSON); plan/38-first-order.md:18-19,33-34; plan/later.md:512-516.
- API item: ProofStructure JSON form in core/src/serialize/nets.rs (sequent, mix, links).
- Requirement: The form is extended only additively. Boxes (and instances, if any) are added so that a structure without them serializes byte-identically to today and a new reader accepts old documents. A placement of weakening and bottom (R78), if step 33 stores one, is an optional additive key as well. Deserialization validates boxes as it validates links and builds the forest within the occurrence limit. An optional substitution for the links of first-order nets is added the same way. The form is documented as extensible (optional extra keys or a version), so that additive information for MALL nets (slices or weights on links), if they ever come, needs no break. The fields of ProofStructure stay private. The nets of cut elimination (R101) and of the planar calculus (R104) add their keys by the same rule.
- Met when: A pinned fixture of the old MLL JSON still passes, a fixture with nested boxes round-trips, a malformed box document is a deserialization error, and a first-order net with a substitution round-trips with the propositional strings unchanged.
- State now: Not met. Evidence: core/src/serialize/nets.rs:11-20 struct Net has sequent, mix and links only and does not use deny_unknown_fields, so optional keys with a serde default fit. core/src/nets/mod.rs:182-197 ProofStructure has private fields and links as a vector of pairs. Links today are pairs of occurrence ids only.

### R9. Place for new engine counters in Statistics, its JSON, the command and the harness
- Steps: 35 net engine pruning and essential nets; 37 inverse method; 38 first-order.
- Sources: plan/35-mll-engines.md:24-34; plan/37-inverse.md:17; plan/38-first-order.md:18-19.
- API item: search::Statistics (non_exhaustive), StatisticsDef in core/src/serialize/search.rs, bench run::HEADER and its tail, cli prove.rs statistics.
- Requirement: A new engine or pruning rule needs its own counters (links refused by each net rule, sequents derived, kept, subsumed and probed by the inverse engine, bindings made and undone, unifications tried and occurs-check failures for first-order). They are new fields at the end of Statistics (or a documented reuse of existing ones), with a matching line in the JSON proxy, an arm in the command's --stats text, and columns appended at the end of the harness tail. The meaning and value of the existing counters on propositional goals do not change, and the existing columns are never renamed or reordered.
- Met when: A JSON test in core/tests/serialize.rs shows the counters, `linlog prove --stats` prints them for each engine, the harness has the appended columns, and README example outputs are updated in the same commit as the code (cli/tests/readme.rs passes).
- State now: Partly. Evidence: Statistics is non_exhaustive with seven fields (nodes, memo_hits, memo_entries, splits, links, tests, copies; core/src/search/mod.rs:1461-1490). The JSON proxy StatisticsDef is a hand-listed remote struct (core/src/serialize/search.rs:144-212), so a field added to Statistics and not to the proxy is silently missing from the JSON. cli/src/prove.rs:1185-1230 has one arm per engine and falls through to the focus labels. bench/src/run.rs HEADER ends with pool_after and the rule is to add columns at the end of the tail.

### R10. The mode value gains its ordering in JSON without breaking old objects
- Steps: 36 Lambek and cyclic MLL.
- Sources: plan/36-lambek.md:17-20.
- API item: Mode JSON form (ModeDef in core/src/serialize/search.rs), the mode names of batch and problem files, CLI mode flags.
- Requirement: The serialized mode gains the ordering (see R51) without breaking existing JSON. The new key is optional on reading and defaults to commutative. Whether it is written only when set or always is decided once before the release. The textual mode names (classical, mix, affine, intuitionistic, intuitionistic-affine) of the batch file, the harness problem files and the command-line flags get names for cyclic and Lambek.
- Met when: A serde round-trip test of the old three-flag object and of the new object. The batch and problem-file parser accepts "cyclic" and "lambek". README examples are updated and cli/tests/readme.rs passes.
- State now: Not met. Evidence: core/src/serialize/search.rs:46-55 ModeDef lists exactly the three flags; the five mode words are read and written by private tables in two crates, cli/src/batch.rs:289-300 (`mode_named`) and :785-792 (`mode_name`) and bench/src/problems.rs:214-238 (`parse_mode`, `mode_name`) (their doc comments are cli/src/argument_parsing.rs:604-605 and bench/src/problems.rs:11-12), and core has none (see R241).

### R11. JSON forms for the values of ordinary logic
- Steps: 22 follow-up; 28 audit; 32 web client.
- Sources: plan/32-web.md:66-69; plan/28-audit-and-refactor.md:81-83; plan/later.md:638-657,687-708; plan/later.md:1076-1081,1089-1093.
- API item: ordinary::Sequent, ordinary::Image, ordinary::Derivation.
- Requirement: The values of ordinary logic that a front end exchanges (the ordinary sequent, the image of its translation, and the LK or LJ derivation read back) get JSON forms like the linear ones, so that the web page can import, show and export them. Otherwise the plan records that the first client leaves ordinary logic out. The Rocq import line of the ordinary certificate stays an option. The new forms are made under R6 from the start, so that step 38 adds predicates, terms, binders and a witness or eigenvariable per inference as additive keys: every form a client reads back carries a version that reads as 1 when absent, and new keys are read under serde(default) and skipped when empty. The room costs nothing now, while a later change is a format version once the web client reads the forms (plan/notes/research/impact-fo-ordinary.md, section 6 and section 12, item 4; plan/notes/research/fo-embeddings.md, section 4, JSON forms).
- Met when: Serde proxies with checked reading are pinned in core/tests/serialize.rs, or a line in plan/web/README.md lists ordinary logic as left out; a test shows that an ordinary sequent and derivation file of the propositional case stays byte-identical and still reads after step 38 (core/tests/serialize.rs).
- State now: Not met. Evidence: core/src/ordinary/mod.rs has serde only on Logic (line 68), Translation (110) and Options (205); ordinary::Sequent (mod.rs:476) and ordinary/derivation.rs have none. plan/28-audit-and-refactor.md:81-83 states that the web front end needs the forms. export::rocq::Options::prelude already carries the import (rocq.rs:82-107).

### R12. A version constant that ties the Rust writer to the Rocq library
- Steps: 31 Rocq library.
- Sources: plan/31-rocq-library.md:33.
- API item: new export::rocq::LIBRARY beside export::rocq::NANOYALLA.
- Requirement: The Rust writer's output and the Rocq library are versioned together, as the constant NANOYALLA names the kernel version today. A constant names the library release the scripts are written for. It matches the opam file and the _RocqProject, and a test or the flake check fails when they disagree. The first comment line of a certificate may carry it.
- Met when: export::rocq::LIBRARY exists, the rocq flake check asserts that it equals the version in rocq/*.opam, and the writer emits it in a comment.
- State now: Not met. Evidence: core/src/export/rocq.rs:78 has only `pub const NANOYALLA: &str = "1.1.3"`; there is no rocq/ directory.

### R13. Cut formulas in the Proof JSON
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:16-21,25-27 (the cut and the roots of its formulas; the plan says nothing of a wire form); plan/README.md:662-671 (D13); plan/32-web.md:31-33,66-69.
- API item: Proof JSON form in core/src/serialize/proofs.rs.
- Requirement: The form `{"sequent": ..., "proof": [...]}` gains the cut formulas, in order, so that the reader rebuilds the same forest ids, plus a `cut` step tag naming the cut occurrence and its two premises. A cut-free proof serializes byte-identically to today (the field is omitted when empty), so snapshots, README and older files stay valid. An older reader would silently drop a new field and misread the proof, so the policy of R6 must say what happens. The form is documented on Proof.
- Met when: A round-trip test of a proof with cuts through serde_json. A test that a cut-free proof's JSON equals the old output. The JSON section of Proof lists the new field and tag. cli/tests/readme.rs passes unchanged for the existing examples.
- State now: Not met. Evidence: core/src/serialize/proofs.rs:11-60 has the Step enum with fixed tags and :63-69 the struct Proof with sequent and proof only; there is no `cut` or `cuts`, and the struct has no deny_unknown_fields.

### R14. Cut formulas in the interactive session JSON
- Steps: 34 cut (sessions 1 and 3).
- Sources: plan/34-cut.md:17-18,25-27 (the interactive cut rule and the roots of its formulas; the plan says nothing of a wire form); plan/README.md:662-671 (D13, the state's stable JSON form); plan/32-web.md:31-33,66-69.
- API item: Interactive JSON in core/src/serialize/interactive.rs.
- Requirement: The session JSON gains the cut formulas in the order they were added, so that reading replays them into the forest before it replays the inferences whose ids name them, and it gains the history and undo entries of cuts. A session without cuts keeps its JSON byte-identical. The web client keeps this between requests.
- Met when: A state with a cut, serialized and read back, has an equal derivation. A state without cuts serializes as before. The JSON section of Interactive names the new field.
- State now: Not met. Evidence: core/src/serialize/interactive.rs:11-60 has sequent, mode, inferences and history only; from_parts (core/src/proofs/interactive.rs, about line 250) builds from a fixed forest.

### R15. A JSON form for a cut-elimination session
- Steps: 34 cut (session 2).
- Sources: plan/34-cut.md:19-21,30 (elimination one step at a time; the plan says nothing of a wire form); plan/README.md:662-671 (D13); plan/32-web.md:31-33,66-69.
- API item: new elimination step list and session value.
- Requirement: A run of elimination has a stable JSON form so that a web client can keep it between requests and a course can show one step at a time: the step number, the cut node (by NodeId) and the reduction case, and the resulting Proof in the Proof JSON form, or an Interactive-like session value that holds the current proof and can be stepped and undone. Reading it back validates every step, as Interactive does.
- Met when: A serde round-trip test of a session after k steps. The rustdoc of the type has a JSON section. The step names are a Display and FromStr pair like Rule::name, with a test in the style of names_round_trip.
- State now: Not met. Evidence: core/src/serialize/ has proofs, interactive, nets, search and sequents only; no elimination value exists.

### R16. Text syntax and JSON of a first-order sequent
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:17-18; plan/later.md:304-309,332-334.
- API item: parse::Parser, Sequent Display, serialize/sequents.rs proxy, Fragment JSON name.
- Requirement: The text syntax and the JSON of a sequent grow additively. The parser and printer read and write quantifiers, predicates with argument lists and terms, with every walk iterative (a quantifier prefix is a unary node; an argument list needs a new Visit stop or a term walk) and with a syntax that cannot change the meaning or the arena order of any existing propositional text (identifiers followed by an opening parenthesis and the quantifier keywords must not collide with par, bot and top). A propositional sequent prints and serializes byte for byte as before, because the pinned snapshots and both baselines depend on it. New optional JSON keys carry the term arena and arities, with new tags for predicates (argument term indices), binders, variables, constants and function applications. Deserialization verifies term indices, arities, binder scoping and the distinctness of symbol names, and Fragment's string form names the first-order fragments.
- Met when: core/tests/parse.rs, serialize.rs and depth.rs pass unchanged for propositional input. New tests parse and print a quantified sequent 100 000 levels deep on a 256 KiB stack, error_positions pins errors in the new syntax, and a reader rejects an out-of-scope bound variable and a wrong arity with an Error variant.
- State now: Not met. Evidence: core/src/parse/mod.rs is a two-state hand parser with one explicit stack and no argument lists or binders; Walk::operands returns at most two operands (core/src/sequents/fmt.rs:67). core/src/serialize/sequents.rs:51-58 has the proxy with terms, ids and var_dict, so there is room to add keys, and verify_integrity checks only term and atom index bounds.

### R17. Witness and eigenvariable tags in the Proof JSON
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:19-20,39.
- API item: Proof JSON (core/src/serialize/proofs.rs), Outcome JSON, linlog check.
- Requirement: The proof wire form gets node tags for the universal and the existential rule and a term table for witnesses and eigenvariables. Old files read unchanged. An outcome with a first-order proof still deserializes as a Proof, so that `linlog check` reads `prove --format json`. Axioms on unifiable literals are expressed through the substituted terms, never through an implicit unifier.
- Met when: core/tests/serialize.rs pins first-order proof strings beside the propositional ones, and prove, JSON, check round-trips on a quantified sequent.
- State now: Not met. Evidence: core/src/serialize/proofs.rs has the tags ax, ⊗, ⅋, 1, ⊥, &, ⊕₁, ⊕₂, ⊤, !, ?, copy, wk and mix only; the Outcome flattens the proof's sequent and proof keys (core/src/search/mod.rs:1195-1203).

### R247. Wire integers beyond 2^53 follow one documented rule
- Steps: 28 audit (before the release, since changing the JSON type of a number later is a version); 32 web client; D18, D22.
- Sources: plan/notes/research/32-web.md (section 3, JSON wire forms: Numbers beyond 2^53); plan/notes/research/README.md (section 2, Numbers above 2^53).
- API item: the JSON of proofs::Size (inferences, characters, height and width, which saturate at u64::MAX), and the documentation of Refutation::Equation (needed is an i128), Options::memory_limit and Statistics.
- Requirement: One rule says how an integer that a JavaScript client may receive above 2^53 appears on the wire, since JSON.parse rounds it, and the forms apply it. Size, whose counts saturate, is the one form that reaches such values; the counts of Equation (bounded by the occurrence limit), memory_limit in bytes and the counters of Statistics are documented as far below 2^53 in any run a client sees. The rule says how a saturated value is told from an exact one. The recommended rule keeps numbers as numbers and documents them as exact only below 2^53, with u64::MAX meaning saturated: it changes nothing on the wire, and no client compares a Size at that magnitude. A decimal string for Size is the alternative if a client must read exact counts.
- Met when: A serialize test with a saturated Size and a pinned string; the rule is written in the crate documentation and in core-sequents.md next to the compatibility policy (R6).
- State now: Not met. Evidence: proofs/size.rs:25-43 Size derives Serialize with u64 fields; serialize/search.rs:120-130 and :199-209 have u64 and i128 fields; no rule is written.

## Bounds and stops

### R18. One memory bound that holds for the whole process, equal for every tool
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:34-37.
- API item: search::Options::memory_limit, cli --memory-limit and --batch-memory, DEFAULT_MEMORY_LIMIT.
- Requirement: The comparison gives every tool "time and memory limits the same as linlog's". linlog's bound must be settable so that the whole process stays inside the cgroup limit BenchExec sets. The two simultaneous searches of the race must draw on one account (or the bound is halved per search), and what is not counted (the sequent, the forest, the proof check) must be stated or bounded. Otherwise linlog is killed for memory where another tool is merely slow, or holds twice its limit.
- Met when: A documented rule and a test or measurement that the peak resident memory of `linlog prove` stays at or under the --memory-limit value plus a stated constant on the comparison set, with one account shared by the race.
- State now: Partly. Evidence: the --memory-limit doc in cli/src/argument_parsing.rs says "each within the limit" for the race and counts only what grows with the search, "not the sequent itself". The default is 1<<30 (core/src/proofs/mod.rs:78). The step 28 prompt measured 2.05 GB on SYJ206+1.016.

### R19. A pool returns promptly after a stop
- Steps: 30 release (the baseline pass under the new defaults).
- Sources: plan/30-baseline-release.md:28-36 (the stop text is 34-36).
- API item: search::prove_until stop condition and the thread pool (Options jobs, pool).
- Requirement: After a stop the pool returns promptly. The 15 s spent on queued tasks after a stop on Petri nets (fixed in the step 21 review) must not return with 16 threads, so the stop is polled in every pool task and in the queued ones before they start.
- Met when: The wait_ms and time_ms columns of the lltp-default stage show stops within a small multiple of the 2 s timeout, and a focused test of prove_until with a pool and an immediate stop returns fast.
- State now: Unknown. Evidence: core/src/search/mod.rs:151-215 documents prove_until polling at every stable sequent or literal; the parallel runtime is core/src/search/parallel.rs and focus/parallel.rs (rules in .claude/rules/core-parallel.md). Whether queued tasks poll before they start was not checked.

### R20. A bounded, deterministic search for a falsifying assignment, in the library
- Steps: 31 Rocq library (certified refutations, classical reading).
- Sources: plan/31-rocq-library.md:79-86; plan/README.md:685-700 (D15); plan/notes/research/refutations.md (section 2, group A; section 3, The refutation type); plan/notes/research/README.md (section 3, conflict 8).
- API item: a refuter in the library (search::refute, or a variant Refutation::Classical { assignment } produced by the search behind a bound in search::Options), with serde and a checker (R124); export::rocq consumes the value (R153).
- Requirement: The library looks for a falsifying assignment of the sequent's atoms under the classical reading (erase the exponentials; tensor and with as and; par and plus as or; one and top true; zero and bottom false) within a stated bound, so that the command, the batch mode, the JSON and the web client share the result (D15). It runs in every mode, since the reading is sound for all of them, when the search answered Unprovable without a model, and it never changes the verdict the search gave. The evaluator is one ascending pass over the arena with no recursion over formulas and no clock. The bound is an option with a named default and a command flag (D16), for example the most atoms and the most work, plus the caller's stop closure; the search is a small DPLL over the erased formula, not a truth table over all 2^n assignments. A search that finds none within the bound leaves the refutation as it was, and the Rocq writer then refuses with an Unsupported variant that names the bound (R153). The enumeration order is fixed so the output is stable, and the search is usable on wasm (no thread, no time). The ordinary "not valid" certificate (R154) evaluates an assignment over the atoms of the ordinary formulas with its own evaluator.
- Met when: A unit test through the library entry shows that a falsified sequent such as `|- a, b` gets an assignment and a valid classical tautology gets none; the JSON of the variant is pinned and reads back (R3); the bound appears in the search options (serde round trip and command flag); a test shows that stop interrupts a search over 2^30 assignments; the checker of R124 accepts the assignment and rejects a mutated one; the exporter refuses only when the library found none.
- State now: Not met. Evidence: grep for falsif, countermodel, tautolog and fn eval under core/src, cli/src and bench/src finds nothing. Sequent::terms() is public (arena in topological order), so the pass can be written. Refutation is non_exhaustive (search/mod.rs:1257), so a variant is additive.
- Conflict: see Conflicts for the author, C2.

### R21. A deterministic work budget that replaces the clock
- Steps: 32 web client; D11, D15, D16, D22.
- Sources: plan/32-web.md:46-47; plan/later.md:696-699; plan/later.md:812-819; plan/README.md:641-643,702-710,752-778.
- API item: search::Options (new field such as a work limit), search::Reason (new variant), Statistics, the stop closure contract of prove_until.
- Requirement: The library has no clock, so a wasm caller needs a deterministic bound on the engines' own work (stable sequents, splits, literals) that is an option. It is one unit shared by all engines, and by Interactive::close, close_all and the turns of the default bias. It is identical on native and wasm and gives a Reason of its own (not Stopped) with the work spent reported in Statistics, so that a client can show progress and size the next budget. A closure that merely counts polls is wrong by the crate's own rule, because polls come millions per second on small problems and tens of milliseconds apart on large forests. The budget is an Options field so it has a wire form (R1), and the poll spacing of each engine is documented.
- Met when: The same sequent with budget N gives the same verdict, Reason and Statistics on native and wasm builds, and on one thread the same limit stops the same run at the same counters. The Reason has its variant and its line in the serialize proxy. Equal budgets give equal Statistics for focus, net, additive and Horn. The doc example in core/src/lib.rs, which counts polls, is replaced.
- State now: Not met. Evidence: core/src/search/mod.rs:46-80 Stop has only Closure, Turn and Slice (work counts only for the two searches of Bias::Auto); Reason (core/src/serialize/search.rs:72-84, search/mod.rs:1417) has Stopped, RecursionLimit, CopyBound, MemoryLimit and IndexLimit; Options (search/mod.rs:851-885) has no budget field; prove_until takes only a closure that cannot read the engine's counters (mod.rs:185); core/src/lib.rs:57-61 counts polls with `budget -= 1`.

### R22. A table of every public call that can run long, with its stop or its bound
- Steps: 32 web client; D11, D16.
- Sources: plan/32-web.md:44-47,66-69; plan/later.md:28,37-41; plan/README.md:641-643,702-710.
- API item: Forest::new and within, Proof::check and check_within, Proof::derivation_size, ProofStructure::sequentialize and is_correct, Interactive::from_parts and new, ordinary translate, read_back and Derivation::check, svg::net, the parser, the families, the lltp and mist readers.
- Requirement: No call of the bindings may run unbounded on a thread that cannot be interrupted (wasm, one thread, no clock). Each public call either takes a stop condition, or has a documented cost bound (linear with a stated constant, or a size the client can cap by input bytes or occurrences, with an error when exceeded). The calls the search does not poll are listed in one place with their bound, so that the bindings expose only bounded ones. The client's cap is a library constant or option. Calls whose measured time at the occurrence limit exceeds about 100 ms are given a stop.
- Met when: A table in the crate documentation or core-search.md (its "Not polled" list) gives each public call with its stop or bound, a documentation line sits on each public function, and a test fires the stop at the first poll of each call that has one.
- State now: Partly. Evidence: stop closures exist on prove_until and prove_goal (search/mod.rs:185,229), Interactive::close (proofs/interactive.rs:816-861), derivation_with, every export write and ordinary linear_derivation (ordinary/derivation.rs:641). Memory only: Proof::check_within (proofs/mod.rs:416). Neither: ProofStructure::is_correct and sequentialize (nets/mod.rs:406), ordinary translate (ordinary/translate.rs:289) and read_back (derivation.rs:665); svg::net takes a size limit only. core-search.md "Not polled" lists Forest::new (0.43 s on 27.8M occurrences), the check, the size pass and sequentialize.

### R23. The checker takes a stop condition
- Steps: 32 web client; 28 audit.
- Sources: plan/later.md:954-957,1167-1174.
- API item: proofs::check::check_within and Proof::check_within (new stop argument).
- Requirement: The checker takes a stop condition and polls it at a bounded cadence, so that a hostile proof file (quadratic time within flat memory) cannot hold a wasm tab or a service past the caller's deadline. A stopped check is a refusal, not an invalid proof.
- Met when: A check_until(mode, memory, stop), or a stop parameter, exists, and a test on a crafted quadratic proof returns CheckError::is_refusal() once stop fires.
- State now: Not met. Evidence: core/src/proofs/check.rs:674-707 check and check_within take the proof, the mode and the memory bound only; `stop` appears in check.rs only in a comment at line 409.

### R24. Read-back and the ordinary checker take a stop and a size bound
- Steps: 28 audit; 32 web client.
- Sources: plan/later.md:1079-1081,1089-1093.
- API item: ordinary::Image::read_back, ordinary::Derivation::check, ordinary derivations as a DAG.
- Requirement: Read-back and the ordinary checker take a stop condition and a size bound. A derivation that shares subproofs is read back as a DAG, or checked on the proof term, instead of unfolding exponentially (seven classical ILTP proofs of 8 to 32 million inferences exceeded 6 GiB).
- Met when: The signatures take a stop and a bound, and a test on a shared-subproof classical proof checks it within a small memory bound.
- State now: Not met. Evidence: core/src/ordinary/derivation.rs:311 `pub fn check(&self)` and :665 `pub fn read_back(&self, linear)` take no stop or bound and read the unfolded tree derivation.

### R25. Net conversions and the net criterion can be stopped and bounded
- Steps: 33 MELL nets; 32 web client.
- Sources: plan/33-mell-nets.md:18-21 and :14; plan/32-web.md:25-45.
- API item: ProofStructure::is_correct, sequentialize, from_proof (and new from_proof_within).
- Requirement: The box-aware calls are boundable: a stop callback, as derivation_with has, and a memory or size bound for building a net from a proof, because a net unfolds the proof's shared subproofs and copies and can be far larger than the proof. A refusal is its own error, neither correct nor incorrect, as CheckError::is_refusal is.
- Met when: Signatures such as from_proof_within(proof, mix, bound) and a sequentialize with a stop exist, and a test shows that a tiny bound or an immediate stop ends the call with the refusal error.
- State now: Not met. Evidence: core/src/nets/mod.rs:249 from_proof, :406 is_correct and core/src/nets/sequentialize.rs:27 take no stop and no bound; plan/reports/05-proof-nets.md says sequentialize is O(n^2) (n = 9599 took 122 ms).

### R26. The net drawing is stoppable and its size estimate counts boxes
- Steps: 33 MELL nets; 32 web client.
- Sources: plan/33-mell-nets.md:20-21; plan/32-web.md:25-45; plan/reports/18.
- API item: export::svg::net (stop and size bound).
- Requirement: The net drawing, which will include boxes, has a size estimate that counts boxes and an in-call stop for the web client. svg::net today takes only a byte limit, while the derivation writers take a stop closure.
- Met when: svg::net, or a sibling, takes a stop and the estimate includes box rectangles. A test shows that an immediate stop returns without a drawing and that a limit still refuses with TooLarge.
- State now: Partly. Evidence: core/src/export/svg/mod.rs:626 `pub fn net(net, style, limit) -> Result<String, TooLarge>` has a limit but no stop; svg/net.rs:138 estimate counts literals, connectives, roots and links only.

### R27. Every entry that builds a forest from input can take the occurrence limit
- Steps: 32 web client; 28 audit.
- Sources: plan/32-web.md:58-60; plan/later.md:28; plan/later.md:696-699; plan/later.md:951-953.
- API item: Interactive::new, Interactive::from_parts, Deserialize of Sequent, Proof, Interactive and ProofStructure, Forest::new and DEFAULT_LIMIT.
- Requirement: Every path that builds a forest from user input (Interactive::new, Forest::new, every deserializer of a value that carries a sequent, TryFrom<Sequent>) can take the occurrence limit the client can afford, not only prove. A few hundred bytes of JSON with shared subterms must not unfold to the 50-million default (about 1.25 GB) in a tab. serde's Deserialize takes no options, so a function or seed is needed: Interactive::new_within, a from_json_within, a DeserializeSeed or a try_from_within on the proxies. The command's --occurrence-limit applies to every JSON input through the same constructors. A documented default sized for wasm exists.
- Met when: A deserialization entry that takes the limit is used by the command's check and interact paths and by the bindings, and a test shows that a shared-subterm file beyond the limit is refused with Error::TooManyOccurrences by each reader.
- State now: Not met. Evidence: core/src/proofs/interactive.rs:214-217 Interactive::new calls Forest::new; core/src/serialize/proofs.rs:135, nets.rs:44 and interactive.rs:73 call Forest::try_from(sequent), that is Forest::DEFAULT_LIMIT of 50_000_000 (occurrences/mod.rs:191,394); the command's limit reaches only Options::occurrence_limit (cli/src/prove.rs:1016). .claude/rules/core-search.md:312-320 says the same.

### R28. A derivation and its drawing never exceed what a tab can hold
- Steps: 32 web client; D11.
- Sources: plan/32-web.md:48-54; plan/later.md:1141-1146,1174-1176,1209-1217.
- API item: ViewOptions (limit, memory, compact), Proof::derivation_size, Size::height, Derivation builder, export write functions with a stop, svg::derivation.
- Requirement: The tab never builds a tree of gigabytes. ViewOptions has a smaller web default, the size estimate (including the height) is available before a derivation is built, the builder needs no stack, the writers poll a stop, and the drawing is bounded: an SVG of a derivation allowed by the view limit must fit the tab (its peak is about 6.5 times the estimate), so the limit for drawing is related to the one for building, or the drawing refuses before it writes.
- Met when: A web ViewOptions preset exists (see R142). svg::derivation and write refuse, or the bindings check the size estimate times the SVG factor, before drawing, and a test shows that a derivation just under the limit draws within the memory budget.
- State now: Partly. Evidence: proofs/derivation.rs:354-370 ViewOptions with serde and DEFAULT_LIMIT of 64 MiB, :381 Compact, proofs/size.rs:29-100 (Size fields public, derivation_size at line 88), the builder runs on its own stack (test any_height_on_a_small_stack, derivation.rs:1910), and export/latex.rs:361 and rocq.rs:542 take a stop. svg::derivation (svg/mod.rs:570) takes an already built Derivation with no size bound; core-derivations.md reports an SVG peak of 855 MB for an estimate of 130 MiB.

### R29. The memory bound's coverage is stated and suits a browser
- Steps: 32 web client; 28 audit.
- Sources: plan/later.md:934-950.
- API item: search::Options::memory_limit and Forest (uncounted allocations), Forest::lca.
- Requirement: For a wasm tab the memory bound covers what the library allocates before and beside the search: the forest (about 25 bytes per occurrence, bounded only by the occurrence limit), the net engine's structure and scratch, and the pool's arena. The documentation says exactly which parts are uncounted, the occurrence limit's default suits a browser, and the cost of Forest::lca (a walk up the parents, so as deep as the tree) is constant or documented.
- Met when: The Options documentation lists counted and uncounted structures, there is a bound-aware default for a front end, and lca is O(1) or its cost is documented.
- State now: Partly. Evidence: core/src/search/mod.rs:925-940 the memory_limit doc says the forest and the net engine's structure are not counted; the occurrence_limit default is Forest::DEFAULT_LIMIT of 50_000_000 (occurrences/mod.rs:191); occurrences/mod.rs:361 lca walks parents.

### R30. Every engine polls the stop at a bounded interval of work
- Steps: 32 web client; 28 audit.
- Sources: plan/later.md:962-978.
- API item: the stop closure contract of every engine, including the forward search on one thread.
- Requirement: Every engine polls stop at a bounded interval of work, also in the forward search on one thread (the GPPP-1000 nets missed a 5 s stop by minutes), so that a wasm caller's deadline holds on one thread. The front-door documentation names what is not polled (forest build, proof check).
- Met when: A test per engine shows that a stop raised after N polls ends within bounded additional work, and core-search.md lists the poll sites.
- State now: Partly. Evidence: the prove_until documentation (core/src/search/mod.rs:165-175) lists the unpolled parts (forest build, proof check, single passes). The Horn engine takes the nets now, but the missed stop of the focused forward search, reachable with --engine focus or two-sided, is not stated as fixed.

### R31. A proof with cuts keeps the checker's memory bound
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:19-21.
- API item: Proof::new, Proof::check_within, CheckError, is_refusal.
- Requirement: A proof with cuts keeps the checker's memory bound, whose integers are argued from "a node has two premises at most, fewer than 2^32 nodes". Cut has two premises and takes one member from each zone, so the argument carries over, and core-proofs.md says so. The checker needs no stop closure for this, because the pass is linear.
- Met when: core-proofs.md restates the Surplus and readers argument for Cut, and a test with a Mix-style doubling through a Cut is refused with Problem::Surplus rather than wrapping.
- State now: Not met. Evidence: core/src/proofs/mod.rs check_within and core-proofs.md "No integer of the pass wraps"; no test has a cut case.

### R32. Cut elimination is bounded, stoppable and a refusal when it is too big
- Steps: 34 cut (session 2).
- Sources: plan/34-cut.md:19-21,30; plan/later.md:31-33.
- API item: new elimination options (steps, nodes, memory, stop) and result; Proof::eliminate_cuts or similar.
- Requirement: Elimination can blow up non-elementarily and the web client has no clock. Every elimination call takes an options value (a step limit, a node limit under 2^32-1 with Error::TooManyNodes, a memory limit counted as the checker and the search do) and a stop closure polled once per step. It answers with a result that says normal form, stopped, or a limit was reached, together with the proof reached so far. A limit is a refusal and never "invalid" or "normal". A shared subproof is duplicated by the contraction case as a shared node, not a copy, and each step's derivation view stays under ViewOptions. The returned proof passes Proof::check. The options value has Default, Clone, PartialEq and (with serialize) serde with deny_unknown_fields like the export options.
- Met when: A test shows that a proof with a doubling tower of contractions stops at the step limit and at the memory limit, without allocating past it (as tower(70) does for the derivation view), and that the stop closure counts polls without a clock. The exit status of the command for a stopped run is 3.
- State now: Not met. Evidence: no cut rule exists; the models are Options (setters memory_limit and recursion_limit, Reason::Stopped and MemoryLimit), ViewOptions { limit, memory } (proofs/derivation.rs:356) and proofs/size.rs; nothing rewrites a proof, and the checker takes memory but no stop.

### R33. The net engine's pruning state is a pure function of the links
- Steps: 35 net engine pruning and the cubes of the parallel runtime.
- Sources: plan/35-mll-engines.md:24-27.
- API item: search::net Engine::seed, Engine::reset, net::parallel cubes.
- Requirement: All new pruning state is a pure function of the set (and the stack order) of links, maintained only inside link and unlink, so that a cube replayed by seed on a reset engine reaches the same state, the run stays deterministic on one thread, and the cubes' per-thread counts stay a function of the input.
- Met when: cubes_agree_with_the_sequential_search and deterministic (core/src/search/net.rs:831,1013) are extended to sequents the new rules prune, and the nodes and links columns of the net-engine comparison are equal.
- State now: Partly: it holds for the existing state, and the pruning state this requirement is about is not written until step 35. Evidence: net.rs:365-385 seed and reset replay links through link and unlink; remaining[] and the copy arrays are updated only there (net.rs:493-506); the parallel search at net.rs:847-980 uses them. The new state is not built yet, so each new rule has to keep this.

### R34. Every new net-engine rule keeps its per-node work linear and inside the polled loop
- Steps: 35 net engine pruning.
- Sources: plan/35-mll-engines.md:24-27.
- API item: search::net Engine::decide and explore stop polling.
- Requirement: Every new rule keeps the per-node work linear in the forest and inside the polled loop. The stop is polled once per node and once per failed exact test. A balance scan over components or a larger candidate scan in choose() must not add an unpolled loop that can run long on a sequent of 10^5 literals, since the web client depends on a stop on every long call.
- Met when: The stops test (net.rs:682) passes on a large generated sequent with the rules on, and there is no new loop outside decide and explore.
- State now: Partly: it holds for today's rules, and the rules this requirement is about are not written until step 35. Evidence: net.rs:355-361 decide() polls per node, net.rs:316 polls per failed test, and today's rules are O(1) per candidate; the new rules are not written yet.

### R35. The closure matrix of the essential engine is charged to the memory account
- Steps: 35 essential nets for IMLL.
- Sources: plan/35-mll-engines.md:38-40.
- API item: new incremental transitive closure bit matrix with an undo log (crate-private), memory::Account.
- Requirement: The closure is quadratic in the nodes of the structure, so its bytes are charged to the search's Account (Options::memory_limit, Reason::MemoryLimit) and refused before allocation when they would not fit. The net engine's uncounted linear allocations are not the model. The forest limit of 50M occurrences alone is no bound for a quadratic structure. The undo log pairs with link and unlink in stack order, like the skeleton's, so cubes, seed and reset still work.
- Met when: Account::charge is called before the matrix is allocated, a test with a small memory_limit gives Unknown(MemoryLimit), and an undo test exists in the style of union_find_with_undo in skeleton.rs.
- State now: Not met. Evidence: the Engine::Net documentation says its structure and scratch are not counted under Options::memory_limit; core/src/search/memory.rs:1-60 holds Account; core/src/occurrences/mod.rs:191 DEFAULT_LIMIT is 50_000_000; core/src/nets/skeleton.rs has the undo log pattern.

### R36. The inverse engine's database is charged to the memory account
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:17-18; plan/notes/research/37-inverse.md (section 2, item 2; section 4, Memory).
- API item: new sequent database with a subsumption index, search::memory::Account.
- Requirement: The database of derived sequents and its index are charged to the search's Account at every growth, by the capacity allocated. The cost per stored sequent is documented, so that the bound has a stated meaning on forests of millions of occurrences (the focused engine's layout costs one bit per occurrence in a zone, and a sparse key is its open follow-up, core-focus.md); how the sequents are stored is the step's design. Over the bound the engine answers Unknown(Reason::MemoryLimit) and never aborts. A memo-style emptying is not available, because a forgotten fact is not recomputed by saturation, so the engine says what it drops, if anything.
- Met when: A test runs the engine under a small memory_limit on a growing family and gets MemoryLimit, and the peak-versus-count check the memory tests already make passes.
- State now: Not met. Evidence: core/src/search/memory.rs has Account and Charged (used by the focused memo, additive and Horn markings; .claude/rules/core-search.md "The memory bound"). OccSet (occurrences/set.rs:25) is a Box<[u64]> of the forest's width, the representation core-proofs.md says was abandoned in the checker (6 GiB on 65 000 clauses). No inverse module exists.

### R37. The inverse engine polls the stop in every saturation step
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:17-18.
- API item: search::Reason, Options::memory_limit, the stop closure of prove_goal, poll sites in the saturation loop.
- Requirement: Forward saturation with exponentials does not terminate on unprovable inputs, so the loop polls stop at every given-clause step, in any long subsumption scan or index rebuild, and in the set-up on a forest of SET_UP_POLL occurrences, without reading a clock. A run that was cut off is Unknown with an existing reason (Stopped, MemoryLimit, IndexLimit), never Unprovable. Unprovable (Ok(None)) is returned only when saturation completed with no rule cut by any bound.
- Met when: A unit test with a stop that fires after N polls on a MELL goal that never saturates gives Unknown(Stopped) within a bounded number of steps, and the rules file lists the inverse engine among "where it is polled".
- State now: Not met. Evidence: core/src/search/mod.rs:1414 Reason is non_exhaustive with Stopped, RecursionLimit, CopyBound, MemoryLimit and IndexLimit; the polling contract per engine is in .claude/rules/core-search.md ("Where it is polled"); no inverse engine exists. A fixed-width u32 index in the database would need the IndexLimit arm.

### R38. First-order search has its own bounds and reasons
- Steps: 38 first-order (D9, D17).
- Sources: plan/38-first-order.md:18-19,29-32; plan/README.md:627-631,712-720.
- API item: search::Reason, Options::copies, Verdict::Unprovable, Statistics and the JSON proxies.
- Requirement: First-order search can build terms of unbounded size and instantiate without end (undecidable with exponentials, NEXPTIME in MALL1). Besides the copy bound there are bounds on term size or depth, the number of variables and instances (per branch, with deepening), and the three-valued answer carries over: Unprovable only from an exhaustive level that never met a bound, otherwise Unknown with a new Reason that names the bound hit. Each bound has an Options setter with a default constant, a flag, a line in the JSON proxies, an arm in the command's "unknown" advice, and a harness column.
- Met when: New Reason variants (Reason is non_exhaustive) with Display and JSON tags pinned in core/tests/serialize.rs; Options setters with documentation and default constants; a test where a growing-term goal ends in Unknown(term limit) and not Unprovable; the extension points of cli.md and bench.md are followed.
- State now: Partly. Evidence: Reason (core/src/search/mod.rs:1417) and Statistics (:1462) are non_exhaustive and the proxies in core/src/serialize/search.rs:75-170 are hand-written per variant; cli.md "A new Reason" needs an arm in prove.rs `unknown`; Options::copies(Option<u32>) exists; no term or instance bound.

### R39. First-order term operations are charged, polled and keep the independence assumptions explicit
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:18-19,39; plan/later.md:313-318,286-296.
- API item: new term operations (substitution, unification, occurs check, renaming), the term arena, bindings, trail and instance table, the parallel runtime's independence of the premises of `&`.
- Requirement: Every structure the first-order case adds (term arena, bindings, trail, instance table, substituted-formula cache of the checker and the views) is charged to the memory account, and to the checker's and the view's memory bound, so that the 1 GiB default holds in a tab and the refusal is the existing MemoryLimit or Memory answer. Unification, substitution application and instance renaming poll the engine's stop at a bounded interval of work and read no clock. No term operation recurses over a term nested 100 000 deep (see R43). Memo keys up to renaming, classes of interchangeable occurrences and the parallel runtime's assumption that the premises of `&` are independent are revisited, since a goal that shares a metavariable with its sibling is not independent. The library keeps that assumption explicit and testable.
- Met when: Each new allocation site calls the account and each long loop polls Stop::fired(work) or the closure. A test with a tiny memory_limit on a term-doubling goal ends in Unknown(MemoryLimit), and a test with a counting stop bounds the polls per unit of work on a deep-term unification. A list in core-parallel.md or core-focus.md names the places that assume independence or ground keys.
- State now: Not met. Evidence: core/src/search/memory.rs defines Account, and Reason::MemoryLimit and Problem::Memory exist (proofs/check.rs, about line 500); Stop::fired(work) is the polling mechanism and set_up_stopped polls between set-up passes; focus/classes.rs, focus/memo.rs and search/parallel.rs assume propositional keys; no term type exists.

### R40. The batch can be cancelled and bounded as a whole
- Steps: batch mode (done in step 24; its open points).
- Sources: plan/later.md:559-575.
- API item: batch::run and Results (cancellation and a whole-batch limit).
- Requirement: A bound for the whole batch (a timeout for all) and an abort need the library to cancel searches in flight. Dropping Results only lets workers finish the problems they hold, so a long sequent keeps running up to its own limit. The batch needs a shared stop that run hands to the work closure, or a cancel handle on Results, which every worker's stop closure includes. The outcome is deterministic (an Unknown with Reason::Stopped) and core reads no clock.
- Met when: Results::cancel(), or a batch-level stop passed into work, exists, with a test that a running 10 s search ends within the polling latency after cancel and that Drop cancels.
- State now: Not met. Evidence: core/src/search/batch.rs:386-391 `impl Drop for Workers` "lets the workers finish the problems they hold and take no more"; the work closure gets only the problem and the Search handle (batch.rs:196-199).

### R243. The stop gives the caller the work done since the last poll
- Steps: 32 web client; 28 audit; 31 Rocq library (the export), 34 cut (elimination), 37 inverse method (saturation) and the checkers of certified refutations; D11, D15, D16.
- Sources: plan/notes/research/32-web.md (section 3, Engine interface and dispatch; open question 4); plan/notes/research/README.md (section 2, One stop and one account for every long run; section 3, conflict 7; section 4, item 4); plan/32-web.md:46-47; .claude/rules/core-search.md (an engine "must not ration its own work by counting polls").
- API item: the stop parameter of prove_until, prove_goal, Interactive::close and close_all, Derivation::of_goal and the other derivation entries, the export writers, Proof::check_within (R23), elimination (R32), the inverse engine (R37) and any refuter; Stop::fired(work) in search/mod.rs.
- Requirement: The public stop receives, at every poll, the work done since the previous poll, in the one unit of R21's budget, and the live counters a client shows (stable sequents, splits, bytes held). A caller without a clock or a timer thread (wasm) can then read performance.now() every N units of work and post its progress on the same schedule, and needs neither a count of polls (which core-search.md forbids) nor a call across the JavaScript boundary at every poll. One interface serves the searches, the checks, the exports, elimination and the refuters. The progress the caller sees is counted apart from the work that slices the two searches of the default bias, so that the counters of a decided run do not move (core-search.md:211-215 says that the focused engine's chain polls pass no work on purpose). Whether a closure that ignores the progress is still accepted is decided at step 28, since D18 allows no alias before 0.1.0. The change is made at step 28, because it touches every poll site and steps 34 and 37 add more; the name and shape of the stop type are the design's.
- Met when: Every public long call takes the one kind of stop; a test shows that on one thread the progress summed over a run equals the work counters of its final Statistics; a test with a stop that fires after N units ends the run within bounded further work for each engine (R30); the columns verdict, nodes, splits, memo_hits and memo_entries of bench/targets.sh equal those of bench/targets/after-bias.csv; the wasm32 test of step 32 reads its deadline in the caller from the progress.
- State now: Not met. Evidence: core/src/search/mod.rs:48-62 Stop::Closure(&mut dyn FnMut() -> bool) calls stop() and ignores the work, which only Stop::Turn and Stop::Slice use; every public stop is an impl FnMut() -> bool (search/mod.rs:189,234; proofs/interactive.rs:821,845,861; proofs/derivation.rs:588,633,646,683; export/latex.rs:365-394; export/typst.rs:313-547; export/rocq.rs:546,621); Statistics exists only in the final Outcome; core-search.md:211-215 says that the chain's poll passes no work.

### R250. First-order terms have an unfolded-size bound beside the occurrence limit
- Steps: 38 first-order; 32 web client; 28 audit (the place for the bound).
- Sources: plan/notes/research/impact-quantifiers.md (section 1, finding 6; section 2.3); plan/notes/research/fo-linear.md (section 3.1); plan/32-web.md:58-60; plan/38-first-order.md:15-22.
- API item: the entries R27 names (Interactive::new and the readers of Sequent, Proof, Interactive and ProofStructure), Sequent::occurrences and a sister measure for terms, the formula and term printers, proofs::size (the weights), the checker's comparison of terms, the term operations of R39.
- Requirement: Terms that are arguments of predicates and functions are shared in a hash-consed arena, so a few bytes of JSON can unfold to an exponentially large tree even when the count of formula occurrences (R27) is small. Every entry that builds a sequent, a proof or a state from input also takes a bound on the unfolded size of first-order terms (the proof's term table of R125 included), with a default sized for a tab, and refuses a larger input with a typed error (R136). A measure of the unfolded size, saturating like Sequent::occurrences, is one pass over the arena. The printers and the size estimate honour the bound or use that measure, and no term operation walks a shared term once per path: the checker compares instances and witnesses by arena id (R126) and never unfolds them, and the occurs check, substitution and renaming visit each shared node once, so that a hostile proof file cannot get past the memory bound of check_within by taking time instead.
- Met when: A test shows that a DAG of shared arguments past the bound is refused by each reader that R27 lists; a test shows that a DAG of astronomically large unfolded size is checked, substituted and occurs-checked in time linear in its arena, and is printed only after the bound is passed; the default is a named constant documented as sized for a tab.
- State now: Not met (the arena has no term arguments yet). Evidence: core/src/sequents/mod.rs:160-173 Sequent::occurrences counts formula nodes, saturating at u64::MAX; Forest::DEFAULT_LIMIT is the only bound on input (R27); the weights of proofs/size.rs count formulas; R136 and R126 name the error and the comparison by id, but no entry asks the readers, the printers and the size estimate for the bound.

## Wasm portability

### R41. A wasm32 build of the library held by a flake check
- Steps: 30 release; 32 web client; D22.
- Sources: plan/32-web.md:28-36 and CLAUDE.md; plan/later.md:28,37-41; plan/later.md:687-690; plan/later.md:960-961; plan/README.md:752-778.
- API item: the wasm32-unknown-unknown build of `linlog` without parallel, png and pdf; rust-toolchain.toml; modules/checks.nix; the workspace members.
- Requirement: The library with its default features minus the command-line-only ones builds for wasm32-unknown-unknown, and a flake check holds it, so that the audit's refactors and the later engines cannot break it unseen. The toolchain carries the wasm32 target and the checks can run wasm tests (see R198). The check covers the feature set the client ships (parse, serialize, interactive, latex, typst, svg, rocq). If the cargo-deny configuration needs a target triple for a wasm-only dependency, deny.toml lists it. The published crate (step 30) does not pin a design that the web client must undo.
- Met when: `cargo check --target wasm32-unknown-unknown -p linlog --no-default-features --features parse,serialize,interactive,latex,typst,svg,rocq` runs as a flake check, rust-toolchain.toml and modules/toolchain.nix name the target, and the feature-powerset run stays green. A new check in modules/checks.nix builds the library, the bindings and their tests for that target, so a library change that breaks wasm fails in this repository.
- State now: Not met. Evidence: rust-toolchain.toml has components only, no targets; grep for wasm in modules/, flake.nix, .github and the manifests finds only comments (core/src/hash.rs:9, core-forest.md:70, core-parallel.md:16); deny.toml:28 has an empty targets list. By convention the code is clean (see R42), but nothing was ever compiled for wasm.

### R42. No clock, no thread and no operating-system randomness outside the parallel feature
- Steps: 32 web client; 34 cut; 35 net engines; 37 inverse method; 38 first-order; D11, D22.
- Sources: plan/32-web.md:43-47; plan/34-cut.md:19-21,30; plan/35-mll-engines.md:22-41; plan/37-inverse.md:17-18 and CLAUDE.md; plan/38-first-order.md:15-22,26-29; plan/README.md:641-643,752-778; plan/30-baseline-release.md:39-40.
- API item: the whole of core without the parallel feature (std::thread, Instant, SystemTime, available_parallelism, RandomState, std::fs, env, process), the crate-private hash module.
- Requirement: With the parallel feature off, the library uses no clock, no thread, no OS randomness and no unsafe, so that it builds and runs on wasm32-unknown-unknown. Threads and timing stay behind the parallel feature or in the command and the harness. All new code (the cut rule, elimination, box nets, the pruning rules, the essential and inverse engines, the first-order term operations) obeys this: any hashing uses the crate's fixed-seed hasher, and the stop closure is the only time source. Every default feature (parse, serialize, interactive, latex, typst, svg, rocq) compiles for the target.
- Met when: A grep finds no Instant, SystemTime or thread::spawn outside cfg(test) and cfg(feature = "parallel"), or a lint forbids it; the wasm check of R41 compiles; `cargo hack check --each-feature -p linlog` passes.
- State now: Met for the existing code, held by convention only. Evidence: Instant occurs only in tests and doc examples (core/src/search/net.rs:514, occurrences/mod.rs:785 in a test, focus/tests.rs:1261); std::thread occurs under cfg(parallel) in focus/schedule.rs:22-24 and :440-470, batch.rs:201-240 and parallel.rs, and in tests; core/src/hash.rs:9 uses a fixed foldhash seed that is reproducible on wasm; no unsafe appears in core/src. Several test-only std::thread uses (core/tests/depth.rs:140,173, core/tests/parse.rs:222, src/proofs/interactive.rs:1131, src/proofs/derivation.rs:1946, src/nets/sequentialize.rs:273) must be gated when the tests run under wasm (R198). The new code that steps 33 to 38 add is not written yet.

### R43. No new walk recurses over a formula, term, proof or net
- Steps: 33 MELL nets; 34 cut; 37 inverse method; 38 first-order; D11.
- Sources: plan/33-mell-nets.md:18-20; plan/reports/20; plan/32-web.md:51-53; plan/34-cut.md:19-21,30; plan/37-inverse.md:17-18; plan/38-first-order.md:15-22; plan/later.md:313-318.
- API item: crate rule "Nothing recurses over a formula" (core.md), core/tests/depth.rs; the per-depth Danos-Regnier criterion and sequentialization through nested boxes, cut elimination and the cut check, the inverse engine's proof read-off, every walk over first-order terms.
- Requirement: The crate rule is extended to everything the later steps add. The per-depth criterion and sequentialization through nested boxes do not recurse on the box nesting depth or the net's height. Elimination keeps an explicit stack and polls stop between steps. A proof can be 2^32 nodes deep, and a sequent read from JSON can be as deep as it is long. The inverse engine's read-off of a proof is an iterative pass. Terms nested 100 000 deep (f(f(f(...)))) parse, unify, substitute, print, check and serialize on a 256 KiB stack, as formulas do today, because wasm stacks are small and a deep recursion aborts the tab.
- Met when: core/tests/depth.rs gains a case per new walk: a deeply nested box net, elimination and the cut check on a proof nested 100 000 deep, a forced inverse run on a 100 000-deep formula, and terms and binders nested 100 000 deep, each on a thread of 256 KiB or 1 MiB. The existing test a_high_derivation_needs_no_stack serves as the model for nets.
- State now: Partly. Evidence: core.md "Nothing recurses over a formula" and core/tests/depth.rs cover formulas today; nets/sequentialize.rs uses an explicit step stack and the test a_high_derivation_needs_no_stack (about lines 241-274), and the graph search is an iterative Tarjan. No box, cut, inverse or term code exists yet.

### R44. Whether the png and pdf renderers build for wasm is decided and recorded
- Steps: 32 web client.
- Sources: plan/32-web.md:30-31 (the wasm target) and 66-69 (the finished proof's exports; the plan names neither PNG nor PDF, so the need is derived).
- API item: the features png and pdf (resvg, krilla), export::png::from_svg and export::pdf::from_svg.
- Requirement: Decide and record whether png and pdf build for wasm32. The client's exports of a finished proof (SVG, PNG, PDF) need them, with the font bytes and the PDF date passed in (no clock, no system fonts). If they do not build, the client must be able to render PNG and PDF from the SVG outside the library. No feature the client enables pulls them in by accident.
- Met when: `cargo check --target wasm32-unknown-unknown -p linlog --features png,pdf` runs in a flake check, or a documentation line says which features the client ships and why.
- State now: Unknown. Evidence: core-export.md:192-206 shows that from_svg takes font data and pdf::Options::date and that the crate reads no clock, so the design is right. plan/reports/22-configurable-output.md:461-470 says the renderers are "said to build" for wasm but they were never built here.

### R45. A safe recursion limit for a given stack size, and a stop before an overflow
- Steps: 32 web client; D11, D16.
- Sources: plan/32-web.md:47-48; plan/later.md:37-41,28; plan/later.md:696-699; plan/README.md:641-643,702-710.
- API item: Options::recursion_limit, Options::stack_size, DEFAULT_RECURSION_LIMIT and the other defaults sized for 8 MiB (memory 1 GiB, 50 million occurrences), the focused engine's recursion.
- Requirement: A caller that cannot spawn a thread (a wasm module on a main stack of about 1 MiB) can run the focused engine on its own stack and gets Reason::RecursionLimit before an overflow, since an overflow is a trap that kills the module. The cost of one recursion level is a documented, computable number, with an inverse of stack_size (stack bytes available to the largest safe recursion limit), or a documented link argument. The defaults do not assume 8 MiB and the engines stop with a Reason at the lowered limit. An explicit-stack (suspendable) focused engine stays deferred and is decided by step 32's measurement, so the options must not make that change a break. Every other recursion (Interactive::close, the derivation views, the exports) stays on explicit stacks.
- Met when: A public Options::recursion_limit_for_stack(bytes) or Options::for_stack(bytes), or a documented table, exists, and a test runs prove at that limit in a thread of exactly that stack in an optimized build, and a wasm run in the first session's measurements reaches RecursionLimit and no trap. The per-level constant of stack_size is measured on wasm32 and recorded. core/tests/depth.rs stays green at 256 KiB.
- State now: Partly. Evidence: core/src/search/mod.rs:924-935 DEFAULT_RECURSION_LIMIT is 2048 and "fits the 8 MiB stack"; :1176-1187 stack_size() is limit times PER_LEVEL (2304 bytes optimized, 12288 debug) with a minimum of 8 MiB, which gives only the direction from limit to stack; Reason::RecursionLimit exists; the checker, the derivation view, sequentialize and the interactive translation are already iterative. Nothing was measured or tested on wasm.

### R46. Every integer argument is made sound for a 32-bit target
- Steps: 32 web client; D11.
- Sources: plan/32-web.md:54-60; plan/later.md:960-961; plan/README.md:641-643.
- API item: checker Bag (proofs/check.rs), Size estimate (proofs/size.rs), focused memo (search/focus/memo.rs), Horn arithmetic (search/horn/reach.rs), OccSet, Forest, usize and u64 casts, Forest::DEFAULT_LIMIT, the u64 memory account.
- Requirement: wasm32 is the first 32-bit target the code is built for. Every integer argument made for a 64-bit usize (counts, sizes, allocation estimates, u64 limits converted to usize) gets its sentence for 32 bits, with saturating or checked arithmetic where usize carries a count. The limit cases (Bag::MOST, the memo's u32 offsets and slots, the u64 to usize casts in the memo) are testable on wasm32 without allocating gigabytes, because the limits are injectable in tests.
- Met when: A wasm32 test run of the checker's and the memo's limit tests with small injected limits; a review note per file; `cargo check --target wasm32-unknown-unknown` is clean with the clippy cast lints (cast_possible_truncation) for the audited files; a recorded 32-bit review in core.md.
- State now: Partly. Evidence: proofs/check.rs:125-126,146-147 the Bag::len comment covers a 32-bit usize and MOST fits usize; search/focus/memo.rs:186-190,218,239,359 use u32 offsets and `u32::MAX as usize`; search/horn/reach.rs:718 uses checked arithmetic for 32 bits (core-horn.md:178-180); but memo.rs:405 and :251 cast a u64 or hash to usize, Statistics::memo_entries and Options::memo_limit are usize (search/mod.rs:853,1477), and grep for target_pointer_width and usize::BITS finds nothing. Nothing was compiled or tested for a 32-bit target.

### R47. The default bias takes the same scheme with and without threads
- Steps: 32 web client; 28 audit.
- Sources: plan/32-web.md:61-65; plan/later.md:801-824.
- API item: search::focus::schedule::turns and alternate, Bias::Auto, Stop::Turn and Slice, Options (new field choosing the scheme), Decide::decide.
- Requirement: Without threads the two searches of the default bias take turns from their start (up to five times the better search). This path must stay selectable and deterministic (counters a function of the input) and run under wasm in tests. An option, or a suspendable focused search, lets a caller refuse the thread the default bias starts, so that the same input gives the same Statistics and the same stop-poll sequence with and without the parallel feature. Otherwise a stop that counts work, which is all the web client has, behaves differently from the command. The engine interface must not rule out a later suspendable search (an explicit stack): Decide::decide stays the single entry so a resumable engine can replace it.
- Met when: default_bias_takes_turns (pinned counters) runs under wasm32; a test runs one Auto-bias sequent with exponentials under both schemes and asserts equal Statistics; `cargo hack check --each-feature` stays green; the Options documentation names the scheme field; the measurement on the target set's small rows is recorded; Decide's signature is documented as the place a resumable search plugs in.
- State now: Not met. Evidence: core/src/search/focus/mod.rs:245-257 takes schedule::alternate (a scoped thread, schedule.rs:470) whenever parallel is compiled in and falls back to turns only when no thread starts; turns restarts on growing budgets (schedule.rs:34, FIRST_TURN and TURN_GROWTH), so counters differ between builds; Options (search/mod.rs:851-880) has no field for it; no explicit-stack engine exists; turns was never run on wasm.

### R48. The batch can be used without Send bounds when no thread is involved
- Steps: batch mode (done in step 24); web client.
- Sources: plan/later.md:518-537,687-691.
- API item: batch::run and prove (generic bounds Send and 'static), Results.
- Requirement: The batch iterator and its closures are bounded Send, Sync and 'static, and Results boxes a dyn Iterator + Send even when the parallel feature is off and everything runs lazily on the caller's thread. A wasm wrapper whose problem source or closure holds JavaScript handles (not Send) cannot use it. The bounds are relaxed to what the threaded path needs under cfg(feature = "parallel") only, or a non-Send sequential entry is provided.
- Met when: The sequential build (no parallel) of run and prove accepts non-Send iterators and closures, checked by a compile test in a no-default-features build, and the threaded build keeps the bounds.
- State now: Not met. Evidence: core/src/search/batch.rs:179-190 run requires `IntoIter: Send + 'static`, `work: Fn + Send + Sync + 'static`, `P: Send`, `R: Send`; :230-232 `Inner::Here(Box<dyn Iterator<Item = R> + Send>)` unconditionally.

### R49. Each feature subset the client ships is a supported, tested configuration
- Steps: 32 web client.
- Sources: plan/later.md:687-708.
- API item: Cargo features of linlog (parallel, png and pdf off for wasm), Options::jobs, batch::Cores, Options::pool, Error::ThreadPool.
- Requirement: The client ships a subset of the features, so each subset is a supported and tested configuration. The behaviour with parallel off is right where the API mentions threads (Options::jobs, batch::Cores, the cfg-gated Options::pool and Error::ThreadPool), and png and pdf (resvg, krilla) are not pulled in by any feature the client enables. The documentation says which features a wasm client takes.
- Met when: A table of the client's feature set in the crate documentation; `cargo hack check --feature-powerset --depth 2` plus the wasm check; tests with jobs greater than 1 and no parallel assert sequential behaviour.
- State now: Partly. Evidence: core/Cargo.toml has parallel as a dependency on rayon and png and pdf off by default; search/mod.rs:1095-1104 documents that without the feature the search stays sequential; Options.pool is cfg(parallel) (mod.rs:884).

## Data model

### R50. Every public enum or struct that later steps add to is non_exhaustive, or is closed on purpose and listed
- Steps: 28 audit; 30 release; 33 MELL nets; 34 cut; 35 essential nets; 36 Lambek; 38 first-order; D17, D18, D22.
- Sources: plan/34-cut.md:12-13,23-25; plan/38-first-order.md:15-17,26-29; plan/later.md:66-69; plan/README.md:721-727; plan/33-mell-nets.md:18-19; plan/35-mll-engines.md:35-41; plan/36-lambek.md:17-19; plan/28-audit-and-refactor.md:445-452; plan/later.md:696-707; plan/README.md:685-700,752-778; plan/notes/research/30-release.md (section 3, Data model, Proof term and checker, Options; section 4, open question 2); plan/notes/research/README.md (section 2, options; section 3, conflict 1); plan/notes/research/impact-fo-ordinary.md (section 10, item 5; the table on Dyadic, Inference, Labels and Status); plan/notes/research/impact-quantifiers.md (section 3, item 10); plan/notes/research/35-mll-engines.md (section 3).
- API item: Error, NetError, ShapeError, Mode, ParseError, Unsupported, Fragment, batch::{Options, Plan, Problem, Answer}, ordinary::{Options, Logic, Translation, Node, Rule, Inference}, Engine, Reason; the export options structs (latex, typst, png, pdf, rocq), svg::Style, TextOptions, proofs::Labels, Inference, check::{Dyadic, CheckError}, Size, lltp::Status; Term, Kind, Node, Rule, Verdict.
- Requirement: After 0.1.0 a change to a public type is a version, so every public enum that will grow and every struct with public fields that will get a field is #[non_exhaustive] (or has private fields with accessors and builders) before the release; marking it later is itself a break. The exceptions are the types closed on purpose, for which a compile error downstream on a new variant is wanted: Term, Kind, Node, Rule and Verdict (D9: three values). They are the recommended default; the audit decides each item, writes the choice on it, and the policy line of CHANGELOG.md names the closed ones with the planned 0.y bumps (0.2.0 at step 34, 0.3.0 at step 38). Whether ordinary::Node and ordinary::Rule, which step 38 extends as it extends Term and Node, are closed like their linear twins or marked as the research marks them is settled by the audit in the same way for both pairs. Callers build options as `Options::default()` plus assignments. Where an enum is closed on purpose, matches over it in linlog-cli, linlog-bench and linlog-web may have no wildcard, and core.md lists the choice.
- Met when: A grep-able list in core.md names every public enum and every struct with public fields and says for each whether it is #[non_exhaustive] or closed on purpose; the closed ones are those the changelog names (no tool shows that: cargo-semver-checks, R215, compares only with a published release); every item of the API list above that is not closed is marked; the doc example of Mode and every literal in linlog-cli and linlog-bench still compile (`cargo check -p linlog-cli -p linlog-bench`), with wildcard-free matches only where the enum is closed.
- State now: Partly. Evidence: non_exhaustive is present on Engine, Bias, Outcome, Refutation, Reason and Statistics (search/mod.rs:665,797,1204,1257,1416,1461), Refusal (interactive.rs:38), Problem (check.rs:463), ViewError (derivation.rs:438), WriteError, RenderError (export/mod.rs:78), proofs/style.rs:300, and on lltp's Problem. It is absent on Error (errors/mod.rs:19-20), NetError (nets/mod.rs:45-46), ShapeError (occurrences/reading.rs:49-50), Node (proofs/mod.rs:117), Rule (derivation.rs:62), Term and Kind (sequents/term.rs:52,82), Verdict (search/mod.rs:1224), Mode (fragment.rs:220, pub fields used in a doc example), CheckError, Dyadic, Size, Inference (proofs and ordinary), ordinary::Node and ordinary::Rule, lltp::Status (lltp.rs:61), ParseError, Unsupported (export/rocq.rs:109), the export Options structs and svg::Style, and batch Options, Plan, Problem and Answer (batch.rs:54,122,142,153).

### R51. A place for the ordering in Mode, and Mode built without struct literals
- Steps: 36 Lambek; 38 first-order; any new calculus (D17, D19).
- Sources: plan/36-lambek.md:17-20; plan/later.md:271-276; plan/README.md:712-720,729-738.
- API item: fragment::Mode (intuitionistic, affine, mix) with its Display, Hash and Default.
- Requirement: The non-commutative mode is a value of Mode (D2), not a second type. Mode needs an ordering field or flag set (commutative, cyclic, Lambek) whose combinations with affine, mix and intuitionistic are defined (cyclic is classical, Lambek is intuitionistic, with no affine and no Mix) and rejected by one validity check with a named Error. Mode is a struct of three public bool fields built by struct literal in the command, the harness and the crate documentation, so a new field breaks every literal. Before 0.1.0 Mode is made non_exhaustive or builder-only (the constants CLASSICAL and INTUITIONISTIC and the const builders exist), and Display names the calculus. The JSON form must read old objects when a key is added (R10).
- Met when: Mode constructors for the new orderings (for example Mode::CYCLIC and Mode::LAMBEK), a validating function returning a new Error variant, a unit test over all flag combinations, and the literals in cli/src/lib.rs:235,292, bench/src/run.rs:64-68 and the core/src/lib.rs documentation replaced by builders.
- State now: Not met. Evidence: core/src/fragment.rs:221-262 has public bool fields and const constructors but no non_exhaustive; about 440 uses of Mode over core, cli and bench; the struct literals named above exist.

### R52. Fragment keeps free bits and takes the quantifier and division classes
- Steps: 36 Lambek; 38 first-order (D17).
- Sources: plan/38-first-order.md:15-22,26-29; plan/README.md:712-720; plan/later.md:271-276,310-318; plan/notes/research/impact-quantifiers.md (section 1, finding 1); plan/notes/research/impact-fo-ordinary.md (section 3).
- API item: fragment::Fragment (u8 flags), Fragment::name, name_in, NAMED, Sequent::fragment, Kind::fragment.
- Requirement: First-order input needs a fragment flag (so a first-order MLL, MALL and the exponential case are distinguishable: NP, NEXPTIME, undecidable with a copy bound). The flag is set by a predicate with arguments as well as by a binder, since a ground p(a) against ~p(b) has no quantifier, and it is computed from the terms, not from Kind::fragment alone; the named constants MLL to LL stay propositional. The two divisions of the Lambek calculus need a bit or a connective set. The detection stays in the same single pass of Sequent::fragment, containment and union stay lattice operations, and the Display and serde names are fixed. The dispatch rows (Row.fragment uses contains) take the new flags without changing the propositional names.
- Met when: The flags are added in the spare bits with names in name_in and NAMED, JSON reading of the new names, tests of detection and Display, and the existing names unchanged; a sequent with a predicate argument and no binder has the first-order flag.
- State now: Partly. Evidence: core/src/fragment.rs:12-32 Fragment is a u8 using five bits (values 1, 2, 4, 8 and 16), so three are free; serialize/search.rs:10 NAMED is a fixed list of six and deserialization gives the named fragment; the detection is a descending pass over the arena. No quantifier or division flag exists.

### R53. The sequent keeps the written order of its roots where the calculus needs it
- Steps: 36 Lambek; 28 audit.
- Sources: plan/36-lambek.md:24-27; plan/later.md:1065-1074.
- API item: Sequent::optimize and optimize_roots, Sequent.roots, Forest::new roots, Sequent canonical form and JSON.
- Requirement: The order of a sequent's formulas is kept for the non-commutative mode, while optimize sorts the roots by arena index. Either the canonical form keeps the written root order (arena indices assigned in the order of first occurrence along the roots, roots not sorted) or optimize takes the mode (or an ordered flag) and skips the sort; the commutative modes may keep sorting. The same decision settles whether an intuitionistic goal built only from top and zero is recoverable (`0, top |- top` must not print as `0, 0 |- 0`), which needs the written order or the count of right-hand roots. The JSON sequent form, the parser, Sequent::add and equality and hash of sequents agree on which order is canonical. The cost (term renumbering, memo and cache key changes, stored proofs that name occurrence ids) is reported. Alternatively the decision is that the order is not wanted and is documented as such. The two ordered modes need different canonical forms: a Lambek sequent is its written sequence, but a cyclic sequent is a sequence up to rotation (plan/36-lambek.md:24-27), so `|- A, B` and `|- B, A` are one cyclic sequent. The decision states whether equality and hash of a cyclic sequent are up to rotation (one canonical rotation, or a comparison that rotates) and whether Forest::roots() of a cyclic sequent is the written rotation or the canonical one.
- Met when: A test parses `|- A, B, C` and `|- B, A, C` (three or more formulas, since two are a rotation of each other) in the Lambek mode and gets different roots, and in the cyclic mode also different ones while `|- A, B, C` and `|- B, C, A` are equal and hash alike (or the decision says otherwise), and in the commutative modes the same sequent; Forest::roots() equals the written order in the Lambek mode; JSON round trip keeps the order; existing proof JSON and README examples are unchanged; or the decision is recorded in .claude/rules/core-forest.md and core-sequents.md.
- State now: Not met. Evidence: core/src/sequents/mod.rs:204-206 `self.roots.sort()` and :310-312 "sorts the root formulas"; core/src/occurrences/mod.rs:116 numbers roots in the order Sequent::roots lists them; occurrences/reading.rs:149-155 documents that the arena sorts roots by term and the written succedent is lost for top and zero roots; .claude/rules/core-forest.md:101-102 repeats it; no test pins a decision.
- Conflict: see Conflicts for the author, C1.

### R54. The two-sided statement is produced from Reading, and the top and zero ambiguity is settled
- Steps: 31 Rocq library (two-sided intuitionistic statement); the new kernel.
- Sources: plan/31-rocq-library.md:40; plan/later.md:383-386.
- API item: occurrences::Reading (position, goal, hypotheses, implication, formula), Problem::Shape and Succedents.
- Requirement: The two-sided lemma is stated from Reading, and the Rocq checker either recomputes the reading or takes positions as certificate data. Reading needs a public, deterministic, language-neutrally described algorithm (the bottom-up pass of possible positions, the goal choice, the symmetric implication reading), and its public accessors (position, goal, hypotheses, implication) stay. The known ambiguity must be settled or documented in the report: a succedent built from top or zero is chosen by id, not as written (`0, top |- top` prints as `0, 0 |- 0`, and `|- A, top` reads `0 |- A`), so the exported lemma can state a different two-sided sequent than the user typed. Either Reading records the written goal, or the lemma states the reading it used in a comment and the documentation says so. The exporter writes the lemma from Reading::goal and hypotheses so that it matches Reading's Display, and never re-derives the sides.
- Met when: A test that the lemma's two-sided sequent equals Reading's Display for the ILL snapshots; a test with `|- A, top` and `|- 0, top` asserts the emitted statement equals the Reading's; the choice is documented on Reading::new and in the rocq export documentation.
- State now: Partly. Evidence: core/src/occurrences/reading.rs:289-336 are public accessors and :454-460 pins the last-root rule in a test; .claude/rules/core-forest.md "The intuitionistic reading" documents the algorithm and the ambiguity, which is unresolved (plan/later.md:1067, memory note ill-reading-ambiguity). No two-sided Rocq statement exists.
- Conflict: see Conflicts for the author, C1.

### R55. The reading exposes the ordered antecedent and refuses an empty one in the Lambek mode
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:18-19.
- API item: occurrences::Reading::new, hypotheses, goal.
- Requirement: The Lambek calculus is read two-sided: the hypotheses in their written order, one goal, the two divisions as the connectives with their orientation, and no empty antecedent. Reading assigns input and output positions per occurrence and picks the goal by id. It exposes the antecedent as a sequence (not a set) and refuses a sequent with an empty antecedent in the Lambek mode. The goal position follows the written succedent.
- Met when: Reading::hypotheses() (or an equivalent) returns the hypotheses in written order in a test; a ShapeError (or an Error) is raised for the empty antecedent; Interactive::new in the Lambek mode fails for it.
- State now: Not met. Evidence: core/src/occurrences/reading.rs:184-191 Reading holds the forest, a position per occurrence and the goal; Reading::new at :197 has no mode and no order concept.

### R56. The dual of a product reverses its operands in an ordered mode
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:24-26.
- API item: Term::dual, Kind::dual and the negation normal form in parse.
- Requirement: In non-commutative logic the dual of a product reverses its operands, (A ⊗ B)^⊥ = B^⊥ ⅋ A^⊥, and the two divisions A\B = A^⊥ ⅋ B and B/A = B ⅋ A^⊥ differ in operand order. The arena's negation normal form can build the reversed dual in an ordered mode (Term::dual is a const fn that keeps child order and the parser pushes negation down with it). No new Kind is needed if the divisions are read as ⅋ with swapped operands, but the printer must print them back as \ and /, and the commutative output stays byte-identical.
- Met when: A parser and printer round-trip test for `A \ B`, `B / A` and for a cyclic negation in the ordered mode, and the commutative sequents unchanged (`cargo test --workspace`).
- State now: Not met. Evidence: core/src/sequents/term.rs:185-186 `Tensor(k, l) => Par(k, l)` keeps the operand order; core/src/parse/mod.rs:35 has only the lollipop for implication, no division.

### R57. Cut formulas get roots of their own in the forest
- Steps: 31 Rocq library (room for cut); 34 cut (session 1); D23.
- Sources: plan/31-rocq-library.md:99; plan/34-cut.md:26-28; plan/README.md:780-794; plan/later.md:31-33; plan/notes/research/34-cut.md (section 2, Dual pairs by offset); plan/notes/research/README.md (section 3, conflict 3).
- API item: Forest (roots, cut roots), Proof and Forest::new, Sequent arena.
- Requirement: A cut formula and its dual get roots of their own beside the sequent's, so that Forest tells the sequent's roots from the cut roots, or the Proof owns an extra table of formulas. Cut roots are numbered after every occurrence of the sequent and of earlier cuts, so no existing OccId changes when a cut is added, and an added cut is charged against the occurrence limit. Forest::roots() stays the sequent's roots, or every reader of it makes a deliberate choice. The Rocq datatype, the checker's formula source and the soundness statement take the formula lookup as a parameter and not as hard-wired forest numbering, so a node can index a formula outside the forest. The 31 report says where cut goes, and the Rust decision of step 28 is recorded in core-proofs.md. How a cut occurrence finds its dual is an invariant of the extended forest, stated for the dual that keeps child order: A and A⊥ laid out one after the other, so that dual(x) = root(A⊥) + (x − root(A)), or a table per cut. In an ordered mode the dual of a product reverses its operands (R56), the two trees no longer have the same shape, and the offset does not hold. Until a step asks for cut in the ordered modes, Cut is refused there with a named error (R134: a Problem arm in check(proof, mode) and a Refusal in Interactive); if a later step wants it, the dual is looked up in a table in those modes and not by offset.
- Met when: Forest has the sequent roots and the cut roots as separate accessors; a test adds a cut and checks that all earlier ids and OccSet widths are unchanged; past Forest::DEFAULT_LIMIT the addition fails with Error::TooManyOccurrences; `cargo test --workspace` stays green; the Rocq checker takes the formula lookup as a parameter; the 31 report names the place; a test shows that dual(dual(x)) = x for every occurrence under a cut pair in the commutative modes; a test shows that check(proof, mode) of a proof with a Cut node fails with the named error in the cyclic and the Lambek mode.
- State now: Not met. Evidence: core/src/occurrences/mod.rs:155-181 Forest::roots is only the sequent's roots, build() is only called with a whole Sequent (:183-240), and Forest::new(&Sequent) is the only constructor (:202); readers that assume roots equal the goal: search/mod.rs:391 is_roots, occurrences/reading.rs:221 and :301, proofs/interactive.rs:228 and :962, proofs/check.rs:700, proofs/size.rs:107, search/net.rs:149, export/svg/net.rs:157,213-215. There is no way to extend a Forest, and no cut variant (core/src/proofs/mod.rs:85-125).

### R58. A crate-level way to build the dual of a term in the arena
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:26-28.
- API item: Sequent arena and a new dual construction.
- Requirement: A cut formula and its dual are added to the arena as terms that are not roots, in negation normal form, with atoms looked up by name or added to the atom table. The forest's literal CSR (literal_start, 2*atoms+1 groups) is rebuilt or extended when a fresh atom comes in. A crate function takes an arena and a TermId (or a parsed formula) and returns the TermIds of the formula and its dual. The function takes the ordering (R51), because the cyclic dual reverses a product's operands (R56); dual(dual(A)) is A in every mode.
- Met when: That function exists, dual(dual(A)) is A in a test, and Forest::literals is consistent after a cut with a fresh atom.
- State now: Not met. Evidence: the only ways to build an arena are the parser (parse/mod.rs:181, where `negate` is a parser-private flag), Sequent::add (appends roots, core/src/sequents/mod.rs:361) and from_parts (crate-private, :110); grep for `fn dual` finds only Kind::dual (term.rs:111,176).

### R59. A public, validated builder for sequents
- Steps: 34 cut; 38 first-order; the web client and wrappers (D17, D23).
- Sources: plan/38-first-order.md:15-17,26-29; plan/README.md:712-720,780-794.
- API item: a public SequentBuilder (or Sequent::from_terms) over the arena.
- Requirement: Generators in the families, cut elimination, substitution, the web client and wrappers build sequents in code, possibly without the parse feature. A checked builder interns symbols (arity-checked), opens and closes binder scopes (once binders exist), pushes terms, atoms and roots, and verifies the integrity, so the arena invariants (descending indices, scoped bound variables, distinct symbol names) hold by construction. It produces hash-consed sequents equal to parsed ones.
- Met when: A public builder type exists with documentation examples that do not need parse, is used by at least the first-order families, and a round-trip test builder, Display, parse gives an equal sequent.
- State now: Not met. Evidence: core/src/sequents/mod.rs:92-361 has read-only accessors plus optimize, add and verify_integrity, and from_parts is pub(crate) (:110); core-sequents.md says "construction goes through the parser or serde" and that tests build arenas as struct literals; core/src/families.rs generators call parse(&format!(...)) (lines 236-296); the Sequent fields are pub(crate).

### R60. A cut rule in the derivation view
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:19-21.
- API item: proofs::Rule (new Cut), Inference, style::Labels, Rule::ALL.
- Requirement: The derivation view gets Rule::Cut. A new rule is a new entry in Rule::ALL, in both label tables (UPRIGHT and SUBSCRIPT), in name() and from_str, in the classical and intuitionistic mapping, and in the Rocq export's exhaustive match. The inference's principal is None as for Mix, since the cut formula is not in the conclusion, and the sequent of each premise shows the cut formula and its dual as the occurrence ids of the cut roots. The size estimate, derivation, compact runs and the text tree handle it.
- Met when: names_round_trip passes with Cut; the label table sizes are compile-checked against Rule::ALL.len(); a derivation of a proof with a cut renders in text, LaTeX, Typst and SVG snapshots; size.rs derives the same inference count as the built derivation.
- State now: Not met. Evidence: core/src/proofs/derivation.rs:62-125 Rule variants and :134 Rule::ALL; core/src/proofs/style.rs:47,54 tables indexed by Rule::ALL.len(); core/src/export/rocq.rs:448-469 lists the arms without a wildcard; core/src/proofs/size.rs matches on Node.

### R61. An inference keeps its sequent in order where the calculus has no exchange
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-19; plan/later.md:271-276.
- API item: proofs::derivation::{Derivation, Inference, Rule, ViewOptions}, Rule::ALL, rule names.
- Requirement: The derivation view shows no exchange. An Inference stores its sequent as occurrence ids in ascending order, which is the sorted-multiset assumption. In an ordered mode the sequent keeps the sequence (the written order, or the cyclic order), as the same Vec<OccId> with a documented meaning, and the commutative case keeps the ascending order. The one-sided and two-sided rule names include the left and right division rules (\L, \R, /L, /R), or map them to the existing ImpLeft, ImpRight and Tensor rules with an orientation. Rule::ALL (a fixed array of 34), the name parsing (UnknownRule), the style tables and every exhaustive match on Rule change with it, so the rule set is fixed once. No exchange Rule variant is needed.
- Met when: A derivation of a Lambek sequent displays the antecedent in written order with no permutation anywhere; the length of Rule::ALL and its tests and JSON names are updated in one commit; derivation tests cover both calculi.
- State now: Not met. Evidence: core/src/proofs/derivation.rs:543-565 the documentation of Inference.sequent says "as occurrence ids in ascending order", and :62 `pub const ALL: [Self; 34]`; Rule is not non_exhaustive; core-derivations.md says positions index "ascending ids with repeats".

### R62. Room for terms, binders and predicates in the formula arena
- Steps: 38 first-order (D17).
- Sources: plan/38-first-order.md:15-17; plan/later.md:303-309,332-336; plan/README.md:712-720.
- API item: sequents::Term, Kind, Atom, Sequent (arena), the symbol table.
- Requirement: The arena holds first-order terms (variables, constants, function applications) and predicates applied to terms in the same hash-consed, strictly-descending-index arena, without a second formula representation. Decide where terms live (inside Sequent::terms or a separate term arena beside the formula arena) and that Atom becomes a predicate symbol with an arity. Forall and Exists are terms of the arena with a bound-variable representation (de Bruijn index, level or named slot) chosen so that hash-consing still identifies alpha-equal formulas. Negation normal form pushes negation through them (Kind::dual swaps the two), and each has a polarity (the universal negative, the existential positive) for the focused engines. Term stays small and Copy (at most 12 bytes), Kind::arity stays 0, 1 or 2 for the connective cases, Atom stays a dictionary index, the Sequent's fields stay private so the arena can grow, optimize and Sequent::add keep merging by structure, and the parser reserves the quantifier and argument syntax. The audit settles the index types (Atom(u32), TermId(u32)) with this in mind. The term type and the symbol table are decided once for the linear Sequent and for ordinary::Formulas (R67), because the translation builds images bottom-up by id, the read-back maps by tag and the LK and LJ checker equates formulas by id, so two representations chosen apart would need a renaming layer both ways (plan/notes/research/impact-fo-ordinary.md, section 12, item 1). The choice is either one first-order term type, one binder discipline and atoms identified as (name, arity), shared by both arenas, so that the translation copies term ids unchanged and the ordinary checker keeps equating formulas by id, or a separate ordinary type with the Image holding the symbol map. The shared type is recommended. The decision is made before either arena gains terms.
- Met when: A design note (the plan/notes/api.md that step 28 owes, R230) names the arena, the Term and Kind variants and the symbol tables, with a size assertion on Term. Kind::dual and Kind::polarity are total over the new kinds (exhaustive matches), with a unit test that dual is an involution on every kind. The propositional Sequent keeps byte-identical JSON, shown by core/tests/serialize.rs unchanged. plan/notes/api.md names the decision on the term type shared with ordinary::Formulas and why.
- State now: Partly. Evidence: core/src/sequents/term.rs:52-78 Term has 12 variants (Var(Atom), DualVar(Atom), the units, four binary connectives, Bang, Quest), none binding, and Atom (line 30) is documented as a propositional variable; Sequent has pub(crate) terms, roots and atoms (sequents/mod.rs:70-80) with names only and no arity; core/src/occurrences Kind::polarity matches exactly the 12 kinds; the intent is recorded only in .claude/rules/core-ordinary.md ("Where quantifiers go"). plan/notes/api.md does not exist.

### R63. The propositional hot path is guarded by size assertions
- Steps: 38 first-order (D17).
- Sources: plan/38-first-order.md:26-29.
- API item: Fragment (a quantifier bit), Forest per-occurrence arrays, Term size.
- Requirement: The propositional case does not become meaningfully slower. Term stays small and Copy, the Forest's per-occurrence arrays do not grow for propositional sequents, and a first-order class bit in Fragment, set by arguments as well as by binders (R52), lets the dispatch and each engine pick a duplicated or generic fast path. Size assertions make a regression a compile error.
- Met when: Constant size_of::<Term>() and per-occurrence byte assertions exist, and bench/targets.sh output compared with bench/targets/after-bias.csv has identical verdict, nodes, splits, memo_hits and memo_entries columns, with pinned CPU time within a few percent (CLAUDE.md verification table).
- State now: Partly. Evidence: Fragment is a u8 with five flags used (core/src/fragment.rs:12-32), so a sixth bit fits; Node has `const _: () = assert!(size_of::<Node>() == 16)` (proofs/mod.rs) but there is no size assertion on Term or on the Forest arrays; bench/targets/after-bias.csv exists as the reference.

### R64. Instances of a quantified body live beside the forest
- Steps: 38 first-order (D17).
- Sources: plan/38-first-order.md:15-22; plan/README.md:712-720.
- API item: occurrences::Forest and a new instance table or substitution map beside it.
- Requirement: The forest keeps working for a quantified sequent (one occurrence per unfolded subformula, binder occurrence numbering in preorder with its body as the child, literal lists grouped by predicate symbol and sign). A quantifier rule instantiates its body per witness, and under ! and ? every copy needs fresh variables. OccId stays the identity of a static occurrence, and instances and substitutions live in a structure beside the forest (a substitution with a trail), or in an instance table, so that a zone member becomes an (occurrence, instance) pair. The propositional forest layout and OccSet stay byte-identical.
- Met when: Forest tests on quantified sequents (subtree ranges, literal groups by predicate); a documented decision in core-forest.md on the (occurrence, instance) zones; a prototype shows a quantifier instance as (OccId, substitution) with the forest unchanged and the propositional target-set counters identical.
- State now: Not met. Evidence: core/src/occurrences/mod.rs Forest fields (term, kind, parent, root, size, depth, literals, literal_start) assume one atom per literal and unary or binary connectives; "Two occurrences of one arena term are distinct ids that share a TermId" but there is no instance concept; core-forest.md says numbering is a pure function of the sequent, preorder, with subtree(o) a range; plan/reports/26-focused-engine.md:1239-1248 describes the (occurrence, instance) option; no substitution type exists.

### R65. The intuitionistic reading extends to quantifiers
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:15-17,21.
- API item: occurrences::Reading, IllFormula, Position, ShapeError.
- Requirement: The intuitionistic reading extends to quantifiers (input and output positions for the universal and existential, the one-succedent condition unchanged), because first-order MILL is the Lambek and LinearOne target and Mode::intuitionistic must not reject quantified sequents wholesale.
- Met when: Reading::new accepts quantified ILL sequents and prints them two-sided, ShapeError has a case for a binder in a bad position, and round-trip tests like the propositional ones pass.
- State now: Not met. Evidence: core/src/occurrences/reading.rs (595 lines) reads polarized shapes over the 12 existing kinds only.

### R66. The derivation view shows instantiated formulas and counts them in its size estimate
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:19-21.
- API item: proofs::derivation::{Inference, Rule, ViewOptions}, size::Size, style::Labels, fmt.
- Requirement: Inference.sequent, a Vec<OccId>, cannot express a substituted instance, so an inference gets an instance reference per member (or a substitution), and Rule gains the universal and existential rules with labels. The size estimate (derivation_size, ViewOptions::limit and memory) counts substituted characters, so the bounds still protect against huge views. Propositional derivations are unchanged.
- Met when: A first-order derivation prints with witnesses substituted; the size estimate matches the built size in the existing consistency tests; the Labels and style.rs tables have the new rules.
- State now: Not met. Evidence: core/src/proofs/derivation.rs:543-560 Inference { sequent: Vec<OccId>, rule, principal, premises, times }; the Rule enum has no quantifier rules; proofs/size.rs (713 lines) estimates characters from occurrences.

### R67. The ordinary-logic translation table takes quantifier rows
- Steps: 38 first-order (D17).
- Sources: plan/README.md:712-720; .claude/rules/core-ordinary.md:101.
- API item: ordinary::translate pattern table, ordinary::Sequent syntax, the read-back derivation.
- Requirement: Ordinary logic's embeddings are one pattern table. First-order ordinary logic needs rows for the universal and the existential (Girard: ∀ as ∀, ∃ as ∃!). The read-back's tags carry no more than the node. The table and the derivation checker take quantifier rows as additions that do not touch the propositional rows. The rows take the arena's term type of R62, and the translation and the read-back map symbols by identity or through the Image's map, as plan/notes/api.md decides.
- Met when: The pattern table is data with one row per connective, a quantifier row can be added without editing the propositional rows, and core-ordinary.md states it.
- State now: Partly. Evidence: .claude/rules/core-ordinary.md:101 "Where quantifiers go: Node gains Forall/Exists ... the pattern table gets their rows"; core/src/ordinary/translate.rs (505 lines) holds the table; no quantifier row exists.

### R68. The Rocq kernel is open to a second development, with a per-kernel formula printer
- Steps: 31 Rocq library (item 5); 38 first-order.
- Sources: plan/31-rocq-library.md:47.
- API item: sequents::Term and Kind (closed enums), Fragment (u8 flags), export::rocq::Kernel, the Rocq writer's formula printer.
- Requirement: The first-order extension is a second Rocq development beside the first, so the exporter's formula printer, the Kernel enum and the rule-name tables are per kernel and open to a new kernel. Kernel is #[non_exhaustive] and the formula writer is a walk over Walk and Term stops, not a hard-wired NanoYalla function. Term and Kind have no binder or term-arena room. The 31 report says where quantifiers go: a new Term and Kind pair, a terms arena in Sequent, and a Fragment flag (it uses five of eight bits). The 28 audit decides whether Term and Kind become non_exhaustive.
- Met when: Kernel is #[non_exhaustive]; rocq.rs has one printer function per kernel dispatched on Kernel; the 31 report lists the extension points; core-export.md records them.
- State now: Not met. Evidence: core/src/sequents/term.rs:50-72 Term and Kind are closed with no quantifier or term variants; core/src/fragment.rs:14 Fragment(u8) has five flags; core/src/export/rocq.rs term() (about line 240) is NanoYalla-only.

### R69. Occurrence ids, atom indices and node indices are the shared coordinates with the Rocq term
- Steps: 31 Rocq library.
- Sources: plan/31-rocq-library.md:38.
- API item: Forest numbering (preorder), the Sequent atom table (distinct names), Proof::nodes() arena order.
- Requirement: The Rocq side recomputes the forest (a preorder numbering of the written formulas) and takes atoms as indices with decidable equality, so Rust keeps three things stable and documented as public contract: the preorder numbering (roots in sequent order, left before right), the atom table with distinct names (atom indices follow the dictionary), and the arena order (premise before conclusion, root last). Any change is a format break for stored proofs and for the Rocq library.
- Met when: The documented numbering stays on Forest (occurrences/mod.rs:116), the core/tests/serialize.rs fixtures pin the JSON, and the Rocq library's forest function has a test on the same example sequent as the Forest documentation test.
- State now: Met. Evidence: core/src/occurrences/mod.rs:116 and :600 document and test the preorder; core-sequents.md "A name is an atom" states distinct names; Proof::new keeps premises before conclusions with the root last (proofs/mod.rs).

### R70. A refutation carries the data and the conditions that a Rocq lemma consumes
- Steps: 31 Rocq library (certified refutations, item 8 and the escalations; the checker of the StateEquation payload); 28 audit (the form of that payload).
- Sources: plan/31-rocq-library.md:75; plan/later.md:473-477,491-497,503-510; plan/notes/research/refutations.md (section 2, group E; section 4); plan/notes/research/README.md (section 2, Statistics and refutations grow by named fields).
- API item: search::Refutation::{Unbalanced, Equation, StateEquation, Exhausted}, focus::Rules (crate-private), Counts::absorbs_from_copies.
- Requirement: The conditions under which each refutation holds are crate-private (Rules::new: intervals off when affine or when a top lies under a quantifier ? or !; atoms under an exponential get no row; the equation only without additives, additive units, exponentials or weakening). The Rocq lemma states the same hypotheses, and the exporter must decide whether a Refutation is certifiable. So Refutation carries the conditions, or a public predicate (for example Refutation::certifiable(&Sequent, Mode) or a documented rule in core-search.md) says when it holds. The value keeps the atom's least and most, the Equation counts, and the StateEquation weights, and every variant is documented as a certificate with the lemma it instantiates. Exhausted, which carries nothing, either gains a checkable record of the invariants the search used or stays uncertified by type. The Rocq function that recomputes intervals is specified language-neutrally: literals plus or minus one, tensor and par sum, with and plus take the hull, units nothing, and the exponential rule. Refutation::StateEquation must be a certificate that a checker sharing no code with the engine can re-verify (R124). Today it holds the weights of the named atoms and a flag once for the clauses used once, and the engine drops the transitions that can never fire (live) before it certifies. The payload therefore carries every place with a weight, the clauses used once as places of their own by occurrence id, and either weights valid over every transition of the net or the closed set of places that can be marked together with the transitions dropped because an input lies outside it, so that the checker verifies the closure, the dropped transitions and the inequalities y.C <= 0 and y.(M - M0) > 0 itself. The meaning of the existing keys weights and once is settled before 0.1.0 (D18, R6); keys added later are additive. Further kinds of certificate for integer infeasibility (a modulo-k invariant) come as additive variants and ask nothing now.
- Met when: A public method or documented condition with a test over the families and modes, so that every refutation the exporter accepts is covered by the Rocq lemma; a mutation test shows that the exporter refuses an Unbalanced refutation on an affine sequent; the pinned JSON of a qcover refutation carries the new fields, and Refutation::check (R124) re-verifies every StateEquation refutation of the qcover run (bench/qcover) without consulting the engine's live set, and rejects a mutated weight or a removed dropped transition.
- State now: Partly. Evidence: StateEquation is { weights: Vec<(String, i64)>, once: bool } (search/mod.rs:1302-1308); the Horn engine's certify sees only the transitions that live kept (core-horn.md:129-139), and the weights of the class places and the place indices are not in the payload. core/src/search/mod.rs:1258-1292 holds the numbers; core/src/search/focus/mod.rs:356-397 (Rules::new) and focus/counts.rs:410 (absorbs_from_copies) are pub(crate); core-focus.md:537-550 describes the interval semantics in prose; serialize/search.rs:103-185 WhyNot covers Unbalanced, Equation, StateEquation and Exhausted.

### R71. A refutation records the assumption under which it holds, and a bound can be known sufficient
- Steps: 31 Rocq library (certified refutations); exponentials.
- Sources: plan/later.md:503-505,626-636.
- API item: Refutation variants, Reason::CopyBound, Options::copies.
- Requirement: With exponentials a refutation holds only under a bound proved sufficient for the sequent, so a refutation value records the assumption it rests on (the copy bound, or the loop-check argument), and Unprovable never means "no proof within a bound" unless the bound is known sufficient. A way to say that a copy bound is sufficient (an engine-side flag or a computed bound) exists beside Options::copies, otherwise an exhausted bounded search can only be Unknown(CopyBound).
- Met when: A Refutation variant carrying the bound or argument, documented on Refutation, and a dispatch or engine test that a search at a sufficient bound answers Unprovable while an insufficient bound stays Unknown(CopyBound).
- State now: Partly. Evidence: core-search.md:42-45 says Unprovable comes only after an exhaustive search, "which with exponentials means a deepening level that never hit the copy bound"; Reason::CopyBound(n) at search/mod.rs:1416; Options (search/mod.rs:851-885) has no sufficient-bound notion.

### R72. Countermodel values with a checker and a wire form
- Steps: 31 Rocq library (certified refutations and its escalations).
- Sources: plan/later.md:476-490.
- API item: new countermodel values (Kripke model, classical assignment, finite phase model) in ordinary:: and for MALL.
- Requirement: Countermodel certificates need a value type each with serde, a checker that evaluates the formula by passes over the arena (no recursion over a formula), and a place in the result. For ordinary logic that is a classical falsifying assignment and a finite Kripke model for LJ and minimal sequents; for MALL and affine it is a phase model. The classical decision through affine MALL yields no assignment today, so the ordinary layer needs a way to obtain one (an engine answer or a separate small procedure), and the result type can carry it next to the verdict.
- Met when: ordinary::Countermodel (and a phase-model type) with a check and JSON, a decision result that returns it, and a checker that is a Formulas-arena pass covered by core/tests/depth.rs.
- State now: Not met. Evidence: core/src/ordinary/ has Formulas, Sequent, Image and Derivation only (mod.rs:274-507, translate.rs:231, derivation.rs); no model type exists; the deciding path `prove(image.sequent(), image.mode(), ...)` has an outcome with no model slot (core-ordinary.md:95-98).
- Conflict: see Conflicts for the author, C2.

### R73. A batch problem can carry its own search overrides
- Steps: batch mode (done in step 24; its open points).
- Sources: plan/later.md:542-548.
- API item: batch::Problem (per-problem mode and copy bound).
- Requirement: The harness's problem files and the question whether a line may carry its own mode and copy bound need a per-problem override of search options, not only a mode. A Problem carries optional overrides (the copy bound at least, ideally the same setter-shaped overrides that search::Options has) applied over the batch's options inside prove and run.
- Met when: Problem gains an options override (or copies: Option<Option<u32>>), prove applies it, the command's problem-file reader fills it, and a test shows that a batch with mixed copy bounds equals single calls.
- State now: Partly. Evidence: core/src/search/batch.rs:142-149 Problem has name, sequent and mode: Option<Mode> only; cli/src/batch.rs sets copies from the command-wide arguments (line about 908).

### R74. A translator can recover the written structure of an LLTP problem
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:31-34.
- API item: linlog::lltp::read, lltp::Problem {sequent, status}, Sequent::terms, term, roots, Reading, new source-order formula access.
- Requirement: A translator per tool writes the same problem in that tool's syntax. It needs the written structure of the problem: which formulas are axioms and which the conjecture, the ILL reading (implication direction, goal), the atom names un-mangled (lltp::HYPHEN and DOT replace "-" and "." in names), and the status. The library gives this without a second LLTP parser, or the harness keeps a small one that must agree with it.
- Met when: Public read access to terms (exists) plus a documented way to recover the original names and the axioms and conjecture split, or a translator that walks Reading, and a test that a translated Petri net re-reads to the same problem.
- State now: Partly. Evidence: core/src/sequents/mod.rs has public terms(), term(), roots() and atom_names(); core/src/lltp.rs Problem has only sequent and status (roles are lost in the one-sided negation normal form); lltp::HYPHEN and DOT rewrite names (core-inputs.md); Reading gives the ILL view (core/src/occurrences).

### R75. The fragment and the mode are exposed so a driver can tell whether a tool covers a problem
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:41-45.
- API item: linlog::Fragment (has_multiplicatives .. has_exponentials, contains), Sequent::fragment, Mode, the bench CSV columns fragment and mode and the verdict refused.
- Requirement: Each driver needs the problem's fragment and mode to decide whether a tool covers it (units, additives, exponentials, intuitionistic, Mix, affine), and the row says "outside the fragment" distinctly from "unknown" and from "error". The library exposes the five connective-class flags and the mode, and the verdict vocabulary of the CSV has a value or reason for "not attempted: outside fragment".
- Met when: A driver table in the harness keyed by Fragment::contains and Mode; CSV verdict refused with a reason such as "fragment" for external tools; summary prints them in a separate column.
- State now: Partly. Evidence: core/src/fragment.rs has public has_multiplicatives, has_multiplicative_units, has_additives, has_additive_units, has_exponentials and contains; the CSV has fragment and mode columns and a refused verdict (bench/src/run.rs:641), but its reasons are linlog's own (NetFragment, NetMode, NotHorn).

### R76. Exponential boxes in the proof structure
- Steps: 33 MELL nets; 34 cut elimination on nets (D21, D23).
- Sources: plan/33-mell-nets.md:18-21; plan/later.md:189-197; plan/README.md:745-751.
- API item: new box set in nets::ProofStructure (principal ! door, auxiliary ? doors, interior, nesting).
- Requirement: A structure can carry exponential boxes: per box the principal ! occurrence, its auxiliary ? doors and its interior, with nesting. Constructors and mutators validate boxes the way link and from_links validate links (a NetError and no change on refusal). The plain MLL structure carries none at no cost. The correctness criterion applies per box depth with boxes contracted, and sequentialization goes through boxes. The fragment gate (unit-free MLL only) becomes a property of the structure, not of the constructor.
- Met when: ProofStructure has a box accessor and a validated way to add a box; ProofStructure::new accepts MELL sequents with an initially empty box list; a test builds nested boxes, and a malformed box (door not a !, overlapping interiors) is refused with a NetError variant (R132).
- State now: Not met. Evidence: grep for Bang, Quest and box in core/src/nets/*.rs finds nothing; core/src/nets/mod.rs:182 ProofStructure has fields forest, mix, partner (per occurrence), links of literal pairs, graph and skeleton; nets/mod.rs:202-207 refuses anything outside MLL with Error::NetFragment.

### R77. Net vertices that are not one per forest occurrence, and growth after construction
- Steps: 33 MELL nets; 34 cut elimination on nets.
- Sources: plan/33-mell-nets.md:18-21; plan/33-mell-nets.md:18-19 and plan/34-cut.md:19-21; plan/34-cut.md:20-21; plan/later.md:189-197.
- API item: nets::ProofStructure (per-occurrence partner, Graph::new(&forest)), the vertex arena.
- Requirement: Keep room for net vertices that are not one per forest occurrence. In the dyadic proof one ?A occurrence can be copied n times (each use proves A again), so the net of a MELL proof needs one instance of the A-subtree per copy with a map back to the forest occurrence. Contraction, weakening, dereliction and auxiliary-door ? nodes and the box boundary of a ! are not subformula occurrences. Cut elimination adds and removes vertices after construction (copying a box under a contraction adds instances), so the vertex storage is growable and not frozen to forest.len(), or a step may build a new forest and structure. The structure holds such an instance arena (or an unfolded forest) beside the plain forest case without changing MLL's cost.
- Met when: A doc'd data-model decision in core-nets.md, and a test that desequentializes a MELL proof copying one ?A twice and gets a net with two A-instances whose origin is the one forest occurrence; a test that adds an instance; a test that duplicates a box under a contraction cut and checks the result with the criterion of step 33.
- State now: Not met. Evidence: core/src/nets/mod.rs:185-205 vertex arrays are sized once from forest.len() and partner is indexed by forest OccId (line 189); graph.rs:133 Graph::new(&forest) builds a CSR over forest vertices only; the forest holds the sequent's subformulas only (core-forest.md); plan/34-cut.md:23 says the same.

### R78. The canonical net form of the ? nodes and of weakening
- Steps: 33 MELL nets.
- Sources: plan/33-mell-nets.md:29-32; plan/reports/17-assessment.md 3.4; plan/notes/research/33-mell-nets.md (sections 1, 2 and 4); plan/README.md:830,980.
- API item: the ? node representation (generalised ? with 0..n auxiliary premises, or separate dereliction, contraction and weakening nodes), Node::Quest, Copy and Weaken.
- Requirement: The net maps canonically to and from the dyadic proof term. A Quest with n Copy uses is one generalised ? node with n premises (0 is weakening, 1 is dereliction, more than one is contraction), so that proofs that differ only by rule order give equal nets. Weakening (n = 0) and the multiplicative bottom need more than a record of their box: with binary Mix only, no rule concludes the empty sequent, and a criterion without a placement is only necessary and, with units, NP-hard. The criterion at each depth must be exact for them, either by a placement of each weakening and bottom at its own depth that is chosen canonically from the term (a jump to a vertex of that depth) or by a change of the calculus (nullary Mix, see R94). Equality of nets is stated modulo the placement, and the wire form (R8) and NetError (R132) can express and refuse a wrong placement. No new proof Node variants should be needed unless nullary Mix is chosen. Jumps against a change of the calculus is the step's design, once C3 is answered.
- Met when: Two proofs that differ by where an unused ? is weakened or by copy order give the same net modulo the placement; the round trip proof, net, proof passes Proof::check; a brute-force comparison of the criterion with the sequent calculus on every structure of a few nodes with weakening or bottom, with and without Mix, finds no difference; core-nets.md records the choice and why.
- State now: Not met. Evidence: proofs/mod.rs:140-151 Node has Bang, Quest, Copy and Weaken (dyadic, copies are repeated uses of Theta); derivation.rs:62-80 Rule has Dereliction, Contraction and Weakening only as the derivation view; nothing in nets/ reads them; nets/mod.rs:60-66,408 NetError::Empty says that no rule concludes the empty sequent.
- Conflict: see Conflicts for the author, C3.

### R79. A calculus descriptor replaces the single mix flag of ProofStructure
- Steps: 33 MELL nets; 35 essential nets; 36 cyclic MLL.
- Sources: plan/later.md:189-197,199-210,271-276.
- API item: nets::ProofStructure::new(forest, mix), its Display and JSON form, Error::NetFragment.
- Requirement: The constructor and the wire form of a proof structure take a calculus descriptor instead of the single mix flag: Mix today; exponential boxes and ? nodes (step 33); polarised or essential (step 35); planar axiom linking in the cyclic order of the literals (step 36). The fragment check (unit-free MLL only) becomes a property of the descriptor. The JSON stays readable for today's three keys and the new keys are additive (R8).
- Met when: A Net or NetKind options value (Default is today's MLL without Mix; Clone, PartialEq, serde) is taken by new, from_links, from_proof and deserialization; link validation consults it (a crossing check for planar); the pinned JSON of today's nets is unchanged in core/tests/serialize.rs.
- State now: Not met. Evidence: nets/mod.rs:182-207 has fields forest, mix, partner, links, graph and skeleton, and new returns Error::NetFragment unless Fragment::MLL contains the sequent's fragment; serialize/nets.rs:12 writes sequent, mix and links; core-nets.md says "no net for affine mode".

### R80. A cut link between two dual non-literal occurrences
- Steps: 34 cut (session 3); D21, D23.
- Sources: plan/34-cut.md:20-21,30; plan/README.md:745-751.
- API item: ProofStructure (cut links), nets::graph, link(), check_link, NetError.
- Requirement: (The boxed nets of step 33 take cut links too, see R101.) An MLL net with cuts needs a cut link between two non-literal occurrences A and A⊥ (the cut roots), beside the axiom link between dual literals. The partner array, the CSR graph's axiom slot, link and check_link, and the Danos-Regnier test (a cut link is an edge between the two conclusions; the connectedness count gets the cuts) account for it. from_proof reads Cut nodes, sequentialize handles a cut conclusion, and a malformed cut link is a NetError variant. Cut links are accepted only behind an explicit constructor.
- Met when: from_proof on a proof with cuts gives a net that is_correct; sequentialize gives a proof that checks and has the same conclusion; the brute-force switching enumeration in nets::graph's tests still agrees with the criterion on structures with cuts.
- State now: Not met. Evidence: core/src/nets/mod.rs:182 ProofStructure and :340 link(x, y) return NetError::NotLiteral for non-literals (check_link validates two unlinked dual literals); core-nets.md says partner is one u32 per occurrence with a literal's axiom slot last; from_proof reads only Ax nodes (:249-260).

### R81. A net carries the polarization of its reading
- Steps: 35 essential nets for IMLL.
- Sources: plan/35-mll-engines.md:35-38; plan/later.md:199-210.
- API item: nets::ProofStructure with occurrences::Reading (polarization carried by a net).
- Requirement: A polarized structure needs the input and output position of each occurrence next to the links. ProofStructure owns its Forest while Reading borrows a Forest (a self-referential pair), so the library either recomputes Reading::new(net.forest()) on demand in O(n) or offers an owned positions-only value. The net's JSON then needs no new field, because the polarization is a function of the sequent (for unit-free IMLL the goal is unique, so the parked units ambiguity of the reading, R54, does not arise). The net engine's complete branch can ask whether a linking is an essential net of this reading (R123) without changing the classical engine's counters.
- Met when: A function from ProofStructure to its positions, with a test that every net the net engine finds in intuitionistic mode yields Ok, and a JSON round trip of such a net unchanged.
- State now: Partly. Evidence: core/src/occurrences/reading.rs:184-198 Reading<'a> borrows the forest and Reading::new is O(n); `Task` is handed the reading (core-search.md); core/src/nets/mod.rs:182 ProofStructure owns its forest; core/src/serialize/nets.rs:11-20 Net is sequent, mix and links; core-forest.md says the reading is "what the future essential nets read".

### R82. The skeleton's union-find carries per-component aggregates
- Steps: 35 net engine pruning.
- Sources: plan/35-mll-engines.md:24-25.
- API item: nets::skeleton::Skeleton, ProofStructure::same_component, new per-component aggregates.
- Requirement: The union-find of the par-free skeleton can carry per-component aggregates (per-atom counts of unlinked literals of each sign) that union and undo update in O(atoms touched) and that the net engine reads. ProofStructure exposes a component id for an occurrence, so that a per-atom balance over skeleton components can be checked after each link and unlinked in stack order.
- Met when: A skeleton unit test shows that union and undo restore the aggregates; a net test shows that the balance rule never rejects a linking that extends to a proof net (checked against the enumerator of R203).
- State now: Not met. Evidence: core/src/nets/skeleton.rs holds only parent, rank and an undo log (log: Vec<(u32, bool)>); core/src/nets/mod.rs:377 same_component is the only query.

### R83. An equality class per occurrence, valid for unoptimized sequents
- Steps: 35 net engine pruning (equal compound conclusions) and the routing feature.
- Sources: plan/35-mll-engines.md:25-27.
- API item: Forest and Sequent::optimize, Forest::term, a new structural equality class of an occurrence.
- Requirement: The sound compound-conclusion break and the routing feature need to know which subformulas are structurally equal. Forest::term gives the same TermId only for terms the parser hash-consed, so the library offers (crate-private is enough) an equality class per occurrence, computed in O(n) with the crate's fixed-seed hasher and valid for deserialized and hand-built sequents too.
- Met when: A function on Forest returns class ids, with a test that a non-optimized sequent with two equal compound conclusions gets the same class, and the result is deterministic across runs.
- State now: Not met. Evidence: .claude/rules/core-sequents.md:34 says hash-consing happens only in optimize() after parsing and deserialized sequents may carry unshared terms; core/src/occurrences/mod.rs:269 term(); core/src/hash.rs has the fixed-seed foldhash; no equality-class function exists.

### R84. A way back from a subformula class to its occurrences
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:17.
- API item: Forest::term and TermId, search::focus::Classes, a new map from a class back to an occurrence.
- Requirement: The engine works on subformulas as shared classes (equal terms, and under a reading equal positions), not on occurrence ids, because forward rules combine sequents from different branches. The library gives a class per occurrence (Classes::of) and a way to rename the finished derivation back to specific occurrence ids of the forest, since a Proof names occurrences and the checker compares the linear zone as a multiset of occurrence ids.
- Met when: Classes::new(forest, reading) is reachable from search::inverse, and a rename pass is tested by checker acceptance on goals with repeated equal subformulas (the partition-no and wide families).
- State now: Partly. Evidence: core/src/search/focus/classes.rs:21-69 pub(crate) struct Classes with new, of, same and distinct, with a documented lemma that occurrences with equal terms are interchangeable (.claude/rules/core-focus.md:577-593); Forest::term(o) at occurrences/mod.rs:269 gives the hash-consed TermId. There is no reverse lookup from a class to its occurrences, and the rename pass does not exist; the focused engine keeps Proved under the sequent's own key precisely because renaming "was not taken" (core-focus.md:185).

### R241. Mode has its names in the library, read and written by one table
- Steps: 32 web client; 36 Lambek (the cyclic and Lambek modes); D15.
- Sources: plan/32-web.md:66-69 ("the modes"); plan/36-lambek.md:19-20; plan/README.md:685-700.
- API item: fragment::Mode (a name function, a list of names and FromStr), cli/src/batch.rs `mode_named` and `mode_name`, bench/src/problems.rs `parse_mode` and `mode_name`, the web client.
- Requirement: R1 asks for Engine, Bias and Fragment to be read by name; Mode has no equivalent. The words classical, mix, affine, intuitionistic and intuitionistic-affine are a table kept twice outside core (cli/src/batch.rs:289-300 and :785-792, bench/src/problems.rs:214-238), and the two copies have drifted: the harness writes `mix-affine` for Mix with affine, a word its own reader and the command's refuse, and the command's writer calls that mode `affine`. Mode gets one table in core: `Mode::name` writing a word, `Mode::NAMES` listing the word of every valid mode, and `FromStr` as the inverse of `name`, with an error that lists the words. It covers the commutative modes (with one documented decision for Mix with affine) and, once step 36 lands, the cyclic and the Lambek mode. Display stays what it is (prose such as "classical affine with Mix", which the command prints), or changes in a commit of its own with the README examples. The command's batch reader and records, the harness's problem files and CSV, the mode flags of step 36 and the web client use the table. The JSON form of Mode is R10.
- Met when: A round trip over `Mode::NAMES` passes, an unknown word is an error that lists the words, the decision for Mix with affine is documented, and no arm `"intuitionistic-affine" =>` is left outside core (grep).
- State now: Not met. Evidence: core/src/fragment.rs:260-277 has Display only, and it writes prose; the two private tables above; the words are listed again in prose in cli/src/argument_parsing.rs:604-605 and bench/src/problems.rs:11-12.

### R244. A sequent member is one type in the public API, fixed before 0.1.0
- Steps: 28 audit (the decision); 33 MELL nets, 37 inverse method and 38 first-order (they build on it); 31 Rocq library and 32 web client (they take its wire shape); D17, D18.
- Sources: plan/notes/research/impact-quantifiers.md (section 1, finding 2; section 3, items 1 and 2); plan/notes/research/fo-linear.md (section 5.2; section 7, items (c) and (g)); plan/notes/research/README.md (section 3, conflict 6; section 4, item 2).
- API item: Inference.sequent, check::Dyadic and CheckError.premises, Interactive::goal, prove_goal and engine_for (the goal parameter), the sequent key of the interactive JSON and the command's goal line, the operands of Node::Ax and Node::Copy; the crate-private zone types Context, Classes and the memo Key, named in plan/notes/api.md.
- Requirement: Today a member of a sequent is an OccId in all of these. A member with an instance (at steps 33 and 38: the copies of a ? occurrence, the instance of a quantified body) is not an occurrence id, and a leaf of a proof term must be able to name one, because the bottom-up checker cannot recover the instance from the nodes below (two copies with different eigenvariables are one OccId in a Bag). Before 0.1.0 the design states what a member is, and every public signature and field above uses that one type, so that adding instances changes no signature and no JSON shape after the release. Whether a member stays one integer on the wire (as impact-quantifiers proposes) or becomes a pair (as fo-linear proposes, which changes the shape of the key in existing forms) is decided and written down. Node stays 16 bytes: the member is one u32, or an id into a table the Proof owns. plan/notes/api.md (R230) names the member type and the zone and frame types a first-order engine instantiates, so that the generic or duplicated fast path of D17 can be measured against it. The Rocq node of step 31 takes the member type as a parameter (R57, R118). Which type the member is, a newtype or a pair, is step 28's design; this entry records the need and the constraint.
- Met when: The public sites above compile against the one type with no change of behaviour and of no propositional JSON; plan/notes/api.md has the section; the columns of bench/targets.sh equal those of bench/targets/after-bias.csv; the node of the Rocq library takes the member type as a parameter.
- State now: Not met. Evidence: core/src/proofs/derivation.rs:546 `pub sequent: Vec<OccId>`; core/src/proofs/check.rs:349-353 `pub theta: Vec<OccId>` and `pub gamma: Vec<OccId>`; core/src/proofs/interactive.rs:460 `goal(...) -> Option<&[OccId]>`; core/src/search/mod.rs:229-234 and :314-316 `goal: &[OccId]`; core/src/serialize/interactive.rs:17 `sequent: Vec<u32>`. Notation::sequent (export/notation.rs:151) and the trait Drawn (proofs/style.rs:146) are pub(crate), and so are Context, Classes and Key (focus/context.rs:20, classes.rs:21, memo.rs:33).

### R245. A formula walk and an intuitionistic walk over Reading are public, for printers outside core
- Steps: 29 comparison (the translators); 32 web client, if its bindings print goals; 38 first-order (the shape of the walk); D17, D18.
- Sources: plan/29-comparison.md (item 2); plan/notes/research/29-comparison.md (section 3, Data model); plan/notes/research/README.md (section 2, the first item of Data model); plan/notes/research/impact-fo-ordinary.md (section 7, the row on sequents/fmt.rs; section 12, item 9).
- API item: sequents::fmt::Walk and Visit (pub(crate) today), Term::operands and Reading::operands (pub(crate)), occurrences::Reading and IllFormula, lltp::Problem.
- Requirement: A printer outside the crate (the harness's translator per tool, the goal view of the bindings) walks a formula, and an intuitionistic goal through its Reading, without recursion and without a copy of the library's walk and its positions. The library offers a public, iterative walk of a formula (stops for entering, leaving and between operands, at least for every connective, the units and the atoms) and a public two-sided walk over a Reading (hypotheses, implication and goal, with stops for the connectives of the intuitionistic reading), or an equivalent iterator. If it is public, it is shaped before the release for first-order terms: n-ary children (the stop between operands carries the operand index) and a stop for a binder, because a public walk limited to two children freezes that limit (D17, D18). Otherwise it stays crate-private and the harness keeps its own walk with an explicit stack. Printers inside core (R16, R156, R170, R242) keep using the crate-private form.
- Met when: The translators of step 29 print through the public walk and none recurses over Sequent::term (a grep in bench and cli), and a test in bench runs each translator on a formula nested 100 000 deep on a 256 KiB stack; core/tests/depth.rs covers the public walk itself at the same depth; a translated Petri net re-reads to the same problem (R74).
- State now: Not met. Evidence: core/src/sequents/fmt.rs:18 `pub(crate) enum Visit`, and :52-69 `pub(crate) struct Walk` over a child function returning (Option<T>, Option<T>); core/src/sequents/term.rs:202 `pub(crate) const fn operands`; core/src/occurrences/reading.rs:346 `pub(crate) fn operands`; bench/src has no printer yet.

### R248. Matches over the formula, node and rule kinds name their variants
- Steps: 28 audit (mechanical, no change of behaviour); 34 cut, 36 Lambek and 38 first-order (each adds variants); D17.
- Sources: plan/notes/research/impact-quantifiers.md (section 2.2, the table of wildcard arms; section 3, item 6); plan/notes/research/impact-fo-ordinary.md (section 2; section 12, item 2); plan/38-first-order.md:15-22; plan/README.md D17.
- API item: every match in core over Term, Kind, Node and Rule that has a wildcard or binding arm for the rest (sequents/term.rs atom, map_subterms and offset; sequents/mod.rs and fmt.rs; occurrences/mod.rs Kind::sign; search/focus/mod.rs asynchronous, focus_class and focus_on; proofs/derivation.rs Rule::classical and intuitionistic; proofs/interactive.rs term; nets/graph.rs; export/svg/net.rs, notation.rs, latex.rs and typst.rs; ordinary/translate.rs), and the lint that keeps it so.
- Requirement: A match over Term, Kind, Node or Rule lists the variants it means; its wildcard arm is replaced by the explicit list, so that a variant added by step 34, 36 or 38 is a compile error at each site and never takes an existing branch (a ∀ left undecomposed in the stable sequent, an ∃ released as negative, map_subterms leaving a binder's body index unmapped, a binder recorded as ⊥, a label or connective printed as 0 or ⊕). Shape readers that refuse everything but the shapes they know (horn/mod.rs, focus/schedule.rs, focus/split.rs, proofs/size.rs) may keep a default arm that refuses, with an allow and a comment saying so. The change alters no behaviour. R119 (Node), R254 (from_proof) and R3 (the Refutation proxy) are instances of the rule. The matches in cli and bench are left to R50: the one there, cli/src/interact.rs:190, is a deliberate default that stays a wildcard once Rule is non_exhaustive.
- Met when: clippy::wildcard_enum_match_arm is denied on search/focus, search/horn, export and ordinary (and on sequents and occurrences if the audit finds it tractable), with an allow and a reason on each shape reader that refuses by default; `cargo clippy --workspace --all-targets -- --deny warnings` is clean; the columns verdict, nodes, splits, memo_hits and memo_entries of bench/targets.sh equal those of bench/targets/after-bias.csv; a throwaway variant added to Kind makes the compiler name every site.
- State now: Not met. Evidence: wildcard arms over Term and Kind at core/src/sequents/term.rs:169,222,232, occurrences/mod.rs:96, search/focus/mod.rs:865,881,1708,1769, nets/graph.rs:147, export/svg/net.rs:291 and sequents/fmt.rs:162 (checked); core has 154 `_ =>` arms in all, most of them over other types; core/Cargo.toml and the workspace lints name no wildcard lint.

### R249. Ordinary sequents and their arena are built through checked constructors
- Steps: 28 audit; 38 first-order; the web client and any wrapper that builds sequents in code (D18).
- Sources: plan/notes/research/impact-fo-ordinary.md (section 12, item 7); plan/notes/research/impact-quantifiers.md (section 3, item 10); plan/38-first-order.md:15-22; plan/README.md D18.
- API item: ordinary::Sequent::new, ordinary::Formulas::add and ordinary::Formulas::atom (Node::Atom), errors::Error.
- Requirement: Before 0.1.0 the constructors of the ordinary layer refuse what would panic later, because an infallible public constructor cannot be made fallible afterwards. ordinary::Sequent::new returns a Result and refuses a hypothesis or conclusion id that is not a node of its arena. Formulas::add refuses an operand id that is not an earlier node of the arena (the arena's invariant of descending indices) and a Node::Atom whose index lies outside the atom names, with an Error that names the index and the bound (the variants of Error::OccurrenceIndexOutOfBounds are the pattern). The check does not detect an id of another arena that happens to be in range, and the documentation says so. When step 38 adds binders and arities, the same two constructors also refuse a formula with a free variable and a symbol used with two arities (named errors, R136), with no further change of signature. The parser, the translation and the read-back call the constructors as they do now, and the parsed sequents and all snapshots are unchanged. The proof side needs no entry here: R7, R13, R57, R125 and R152 hold its data, and a second Proof constructor is additive.
- Met when: A test shows that Formulas::add with an operand out of range, and Sequent::new with an id out of range, are an Err and not a later panic; the doc example of Sequent::new and the callers in ordinary/parse.rs propagate the Result; the ordinary snapshots and core/tests are otherwise unchanged.
- State now: Not met. Evidence: core/src/ordinary/mod.rs:317-331 Formulas::add checks only sharing and the size limit, and its documentation states the precondition; :334 atom; :488-494 Sequent::new returns Self and checks nothing; its callers are core/src/ordinary/parse.rs:329 and :384.

### R251. No public item calls an atom a variable
- Steps: 28 audit; 38 first-order (D17, D18).
- Sources: plan/notes/research/impact-quantifiers.md (section 2.1; section 3, item 9); plan/README.md D18; plan/38-first-order.md:15-22.
- API item: sequents::Term::{Var, DualVar}, occurrences::Kind::{Var, DualVar}, occurrences::Sign::{Var, DualVar}, Error::InvalidVariableIndex, the syntax paragraphs of Sequent, the documentation of Atom, and README.
- Requirement: Before 0.1.0 the propositional vocabulary of the public API stops using "variable" for atoms, because step 38 gives the word to term variables, metavariables and eigenvariables, and a rename after the release is a major version. The variants and the error are renamed in one commit without aliases (D18). The audit chooses the new names so that they also suit R62 (Atom becomes a predicate symbol with an arity, and a literal with arguments is a new variant). The derived order of Kind and Sign is kept, since the engines index literals by 2*atom+sign. The JSON tags V and D and the key var_dict stay on the wire, documented as atoms, so no stored file changes. The documentation (the syntax paragraph of Sequent, Atom, the parser's, README) speaks of atoms.
- Met when: grep finds no public item named Var, DualVar or Variable that denotes an atom; core/tests/serialize.rs and every pinned JSON string are unchanged; the syntax paragraph of README and the documentation of Atom no longer call atoms variables; `cargo test --workspace` passes with the renamed items.
- State now: Not met. Evidence: core/src/sequents/term.rs:26 documents Atom as "a propositional variable", :52-56 Term::Var and DualVar, :82-84 Kind::Var and DualVar; core/src/occurrences/mod.rs:45-59 Sign::Var and DualVar; core/src/errors/mod.rs:24 InvalidVariableIndex; core/src/serialize/sequents.rs:58 var_dict; DualVar occurs on 73 lines of core, cli, bench and the tests.

## Engine interface

### R85. The race of one thread and a pool lives once, in the library
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:41-43.
- API item: cli/src/prove.rs alone_first, bench/src/run.rs alone_first, a new shared race in the library.
- Requirement: The comparison runs linlog on one thread as the main column and "its default" as a second, so the harness must run exactly what the command's default runs. The race (one thread, then a pool beside it, memory shared) lives once and is called by both the command and the harness, so that a change of the default cannot make the published column differ from what a user gets.
- Met when: One function (in core behind the parallel feature, or in a shared crate) is used by cli/src/prove.rs and bench/src/run.rs; a test shows that the harness row for the default configuration and the command's verdict and counters agree; grep finds one alone_first.
- State now: Not met. Evidence: grep -n alone_first finds it defined in both cli/src/prove.rs (about line 55) and bench/src/run.rs:728; plan/28-audit-and-refactor.md names "the race ... is written twice".

### R86. Adding an engine touches one list, in the same places every time
- Steps: 35 essential nets; 37 inverse method; any new engine (D7, D19).
- Sources: plan/35-mll-engines.md:35-41; plan/37-inverse.md:17-18; plan/later.md:140-161,204-218; plan/README.md:583-589,729-738.
- API item: search::{Engine, Decide, Engine::implementation, Engine::parallel, Display, DISPATCH, Feature, Statistics}, cli EngineArg and its From arm, bench run::EngineChoice, cli statistics().
- Requirement: A new engine plugs in as one variant of Engine, one impl Decide (admits and decide), one arm in Engine::implementation, one arm in Display (its stable name, used in text and in the JSON `engine` key), a deliberate answer in Engine::parallel, and nothing else in the front door: no change to prove_goal, Answer or Task. The list of engines (names, parsing, an all-variants iterator) is one library item, so the command, the harness and the bindings cannot drift. A sequential engine says so (Engine::parallel is false, documented "runs on the calling thread whatever jobs says", as Additive and Horn do), because Engine::parallel is `!matches!(self, Additive | Horn)` and a new sequential engine would silently count as parallel and make the command add a pool beside it. The JSON engine name is documented as an open set so the web client does not break on new names. The engine-specific counters have a place that neither changes the propositional engines' counters nor the CSV's existing columns (R9).
- Met when: Engine::ALL and FromStr (the inverse of Display) exist; cli/src/argument_parsing.rs and bench/src/run.rs derive their choices from them, or a test checks that they cover Engine::ALL; a test forces the engine through prove and prove_goal and engine_for returns it; the match arms compile without a wildcard; the walkthrough of the edit points in core-search.md and cli.md stays true.
- State now: Partly. Evidence: Engine is #[non_exhaustive] (core/src/search/mod.rs:664-666) and the recipe is in .claude/rules/core-search.md:117 and cli.md "Extension points"; the sites are mod.rs:742-766 (implementation, parallel), serialize/search.rs:265-270 (collect_str), cli/src/argument_parsing.rs:1160-1194, bench/src/run.rs:159-186 and cli/src/prove.rs:1190-1240 (the net arm, then `_` for the focus labels). There is no Engine::ALL or FromStr, and EngineArg and EngineChoice repeat the same variants by hand.

### R87. The dispatch is data with the measurement beside each row
- Steps: 26, 27, 35, 37 engine rows (D19).
- Sources: plan/README.md:729-738.
- API item: search::DISPATCH, Row, the Engine rustdoc table "Which engine decides a goal".
- Requirement: The dispatch is data with the measurement beside each row, so that each later engine adds a row with its evidence and a reader can see why a row exists. At least the row's evidence is checkable (a reference to the harness run) rather than only prose in the Engine documentation.
- Met when: Each Row carries a note or a measurement id, or a test checks that the Engine documentation table and DISPATCH list the same rows in the same order.
- State now: Partly. Evidence: core/src/search/mod.rs:501 `const DISPATCH: [Row; 5]` with fragment, modes, feature and engine; the measurements are a markdown table in the rustdoc of Engine (:631-660), not in Row (:530); core-search.md:121-126 says the measurement is in the documentation and in plan/reports/26-focused-engine.md.

### R88. A new routing feature or mode dimension adds a variant, not a rewrite
- Steps: 35 net engine routing; 36 Lambek; 37 inverse method; 38 first-order (D19).
- Sources: plan/35-mll-engines.md:28-34; plan/37-inverse.md:18-20 and :26-29; plan/README.md:729-738.
- API item: search::Feature, Modes, Row, fragment::Mode, few_equal_literals, NET_MULTIPLICITY.
- Requirement: New engines need rows keyed by more than fragment and mode: a new calculus (cyclic, ordered, first-order) and new features (no two equal literals under one pure tensor or par tree, width, many hypotheses against a small goal). Modes, Feature and Mode take a new dimension by adding a variant or field, not by rewriting the table, and Row::takes stays one function. A new Feature is computable from the Forest in one linear pass without allocating more than linear memory and without a clock, because dispatch runs on every prove, on every item of a batch and in engine_for, and it honours the set-up poll on a large forest. A feature row replaces or narrows the NET_MULTIPLICITY row, rows are first-match in a fixed array, and a row the measurement does not earn leaves the goal with the focused engine (the engine then stays reachable by Options::engine only, with DISPATCH and the Engine documentation unchanged). The library works equally whether the row exists or not. The private threshold NET_MULTIPLICITY, which decides which engine runs, is an option or is recorded (R149).
- Met when: A new Feature variant plus a new Modes or Mode field compile with only the match arms in Feature::of and Modes::take; the order of the rows stays the priority; tests show that dispatch picks net or focus on constructed sequents of each case (wide-m3 against partition) and that no row's engine refuses its own goals (admits); the dispatch unit tests in search/mod.rs are extended; the Engine documentation table has the row and its measurement.
- State now: Partly. Evidence: core/src/search/mod.rs:582-608 Feature has Any, TwoFormulas, FewEqualLiterals and PetriNet, :545 Modes has Any, Linear, Classical and Intuitionistic, Row::takes at :536 is one expression, :623-641 counts multiplicity by atom and sign only, Feature::of gets only Task; fragment.rs:221 Mode is a struct of three public bools. No feature counts hypotheses against goal members.

### R89. MELL stays routed to the focused engine, and forcing the net engine still refuses it
- Steps: 33 MELL nets (D7, D8).
- Sources: plan/33-mell-nets.md:22.
- API item: search::DISPATCH, Engine::Net, Outcome::net.
- Requirement: No new engine and no new dispatch row: MELL stays routed to the focused engine, and the net of a MELL proof is made from the proof afterwards (from_proof). Outcome.net stays Some only for the net engine, and forcing Engine::Net on a sequent with exponentials still refuses with NetFragment. The front door keeps that refusal even though ProofStructure can now hold MELL.
- Met when: A test shows that prove with Engine::Net on a !-sequent is Err(Error::NetFragment) while ProofStructure::new on it succeeds.
- State now: Partly. Evidence: search/mod.rs:501 DISPATCH has five rows, none for MELL nets; mod.rs:1219 `pub net: Option<ProofStructure>`; the net engine's refusal currently comes from ProofStructure::new's fragment check (nets/mod.rs:204), which this step changes (R133).

### R90. An ordered mode is routed to the planar engine and refused by every other engine
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-20; plan/README.md D19.
- API item: search::DISPATCH, Modes, Row, Feature, Engine, Error::EngineMode.
- Requirement: The dispatch routes an ordered mode to its own engine (the planar net engine) and to no other. Modes takes a mode by its two flags, so a cyclic mode would match the Classical row and a Lambek mode the Intuitionistic row and be silently decided by the commutative focused engine, a wrong answer. The fix is a new Modes variant and row, a new Engine variant or a mode flag on Net, and explicit refusal (Error::EngineMode) when an ordered mode is forced onto an engine that has no ordered version (focus, two-sided, additive, Horn). The Modes::Any rows (the Additive row) must also not catch the ordered modes.
- Met when: Tests show that every engine other than the planar one refuses the ordered modes with a named error, that the default dispatch for a cyclic and a Lambek sequent returns the planar engine, and that the existing DISPATCH rows' behaviour on commutative modes is unchanged (the bench/targets.sh columns are equal).
- State now: Not met. Evidence: core/src/search/mod.rs:501-535 DISPATCH of five rows and :557-575 Modes::take looks at mode.affine and mode.intuitionistic only; Error::EngineMode exists (errors/mod.rs).

### R91. A query that lists the rules that would be accepted at a position
- Steps: 32 web client.
- Sources: plan/32-web.md:66-69.
- API item: Interactive::rules, Interactive::split_passes, a new applicability query.
- Requirement: For a mouse client that shows only the rules that apply, a query says which rules (and which splits of a ⊗ or Mix) would be accepted by apply for the formula clicked, without mutating or cloning the state. Today rules lists by connective and mode only.
- Met when: A method such as Interactive::applicable(goal, position) returning each Rule with Result<(), Refusal>, and a test over random states that every listed rule applies and every omitted one is refused.
- State now: Not met. Evidence: core/src/proofs/interactive.rs:497-532 rules() is by connective and mode and its documentation says whether the context lets a rule apply is what apply decides; split_passes (l.745) greys out splits only; plan/later.md "Follow-ups: interactive proving" assigns a filtered rules list to steps 22 and 32.

### R92. close_all keeps the outcomes already obtained and takes a budget per goal
- Steps: 28 audit; 22 and 32 web client.
- Sources: plan/32-web.md:66-69; plan/later.md:1096,1113-1116.
- API item: Interactive::close_all, Interactive::close, a per-goal budget.
- Requirement: Closing all goals does not lose the outcomes already obtained when one goal errors (it returns them with the error, or a per-goal Result), and takes a budget per goal (the work budget of R21, or a client's wrapping through the stop closure) so that one hard goal does not eat the rest. The outcome of each goal is reported to the client.
- Met when: close_all returns per-goal Result<Outcome, Error> or the outcomes so far with the error, and takes the per-goal budget; a test with a stopped second goal.
- State now: Not met. Evidence: core/src/proofs/interactive.rs:857-870 close_all uses `?` inside the loop (`let outcome = self.close(goal, ...)?;`), so an error drops the outcomes vector built so far; one stop closure serves all goals; plan/later.md "Follow-ups: interactive proving" and plan/28 assign it to step 28, 22 and 32.

### R93. The interactive reading is stored, and rules and drawings map to goals cheaply
- Steps: 32 web client; 22.
- Sources: plan/32-web.md:66-69; plan/later.md:1097-1099,1109-1114.
- API item: Interactive::reading, rules, apply, derivation, split_passes, the drawing-to-goal map.
- Requirement: For a client with thousands of occurrences, per-click cost is bounded. The intuitionistic reading is stored with the state (positions kept, since the forest is fixed) and not recomputed in O(n) on every operation. rules can return only the rules that apply, filtered by the goal's context, without cloning the state per rule, and an inference number of a drawing maps to the session's goal.
- Met when: The reading is held in the state; a rules that filters; Interactive::derivation_ids (exists) pinned in a test; a benchmark or test of 10 000-occurrence states.
- State now: Partly. Evidence: core-derivations.md says the reading is recomputed (Interactive::reading, O(n)) whenever intuitionistic mode needs positions; interactive.rs:438-443,512 call Reading::new(&self.forest) on each reading(), rules and apply; rules() (497-530) lists by connective and mode only; derivation_ids exists (929) and the SVG writes `<g id="i{id}-{position}">` (export/svg/mod.rs:408).

### R94. A Mix that opens an empty goal is refused or closable
- Steps: 28 audit; 32 web client.
- Sources: plan/32-web.md:66-69; plan/later.md:1096,1113-1115.
- API item: Interactive::apply with Rule::Mix.
- Requirement: Decide whether a one-sided Mix whose split sends every formula to one side is refused (a Refusal variant) or opens an empty goal that something can close. Today nothing closes it, so a session can stall. The decision is documented on apply. Whether nullary Mix exists is decided by the author before step 31 starts, because it is a rule of the calculus, the checker, the oracle and the Rocq mirror (R117 freezes them); the choice of placement for weakening at step 33 (R78) depends on the answer, and with nullary Mix the empty goal becomes closable.
- Met when: A Refusal (or a nullary close rule) with a test in the interactive tests, documented on apply.
- State now: Not met. Evidence: proofs/interactive.rs:696-700 the Mix arm builds `right = rest.difference(&going_left)` with no emptiness check; a Mix with an empty left side is refused only on reading a state (lines 398-399); plan/later.md "Follow-ups: interactive proving" lists the empty goal with no rule to close it.
- Conflict: see Conflicts for the author, C3.

### R95. An interactive cut rule that names the cut formula
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:17-18.
- API item: Interactive::apply, a new Interactive::cut.
- Requirement: The interactive rule cut, where the user names the cut formula, cannot be a rule of apply(goal, position, rule, left), which acts on a formula of the goal. It needs its own method taking a goal, the cut formula (as text under the parse feature, and as a TermId for a caller without it) and the split of the goal's formulas between the two premises. It extends the state's forest by the cut formula and its dual, opens two goals (Γ, A and Δ, A⊥) and returns them. rules(goal, position) cannot list it, so the goal-level rules are listed elsewhere. Refusals: a formula that does not parse, a bad split, and in intuitionistic mode the cut on the one-succedent goal. Undo of a cut gives back the earlier state, forest included.
- Met when: New Refusal variants for the cut (all documented in Display); a doctest on Interactive that cuts, closes both goals and gets a proof() whose Cut node passes check; a test in the intuitionistic mode.
- State now: Not met. Evidence: core/src/proofs/interactive.rs:538 apply(&mut self, goal, position, rule, left), :497 rules(goal, position), :198-210 Interactive owns one immutable Forest, :952 proof() translates through Terms (no cut), :38-100 Refusal has no cut variant.

### R96. Interactive proving in an ordered mode
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-19.
- API item: proofs::interactive::Interactive (goals, rules, apply, Refusal) and serialize/interactive.rs.
- Requirement: In the ordered modes a goal is a sequence and not a set of occurrence ids. rules() offers only the rules applicable without exchange (a ⊗ splits the goal at an interval boundary, so the split argument is a position and not a subset; a cyclic goal can be rotated by the user as an explicit step or implicitly). apply() refuses what would need exchange with a new Refusal, and Interactive::new rejects an empty Lambek antecedent. The session JSON carries the order and the new mode (R10).
- Met when: Interactive tests: the ⊗ split of `A, B |- A * B` offers exactly the planar split, `B, A |- A * B` is refused in the Lambek mode; the session JSON round-trips with the ordered mode; undo works.
- State now: Not met. Evidence: core/src/proofs/interactive.rs:198-210 Interactive holds forest, mode, inferences and history; :460 goal(&self, id) returns Option<&[OccId]>; :745 split_passes enumerates subsets; Refusal at :39 is non_exhaustive.

### R97. Interactive apply takes a value that can carry a witness
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:21; plan/later.md:319-321.
- API item: proofs::interactive::{Interactive::apply, Refusal, close_all}, serialize/interactive.rs.
- Requirement: Interactive::apply needs a witness parameter: a term, or "open" for a fresh metavariable that a later axiom's unification resolves, with undo undoing the bindings. apply takes positional arguments today, so the step's argument becomes a value (position, rule, split, optional witness) with a builder, which can grow without another signature change. New Refusal variants cover an ill-formed witness, an eigenvariable clash and an unresolved metavariable at close. The session JSON carries instance data and open metavariables, additively, so that a client can hold the session between requests, and from_parts replays them.
- Met when: The new apply signature (or a rule enum with payload) is documented; core/tests/serialize.rs pins a first-order session JSON and a round trip; tests give a witness, leave one open and close it by an axiom, and undo.
- State now: Not met. Evidence: core/src/proofs/interactive.rs:538-544 `pub fn apply(&mut self, goal: InfId, position: usize, rule: Rule, left: &[usize])` has no term argument; Refusal (interactive.rs:36-80) has no witness case; serialize/interactive.rs:14-33 writes inferences with sequent occurrence ids, rule, principal and premises, and history.

### R98. With cuts in the forest the engines still see the same goal and return cut-free proofs
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:28-29.
- API item: search::prove_goal, dispatch, Feature::FewEqualLiterals, Counts, Classes.
- Requirement: With a cut in the forest the engines see the same goal as before: every proof the search returns stays cut-free, and the dispatch and the prunes read only the goal's occurrences or the sequent's roots, never the whole forest. Interactive's close calls prove_goal on a forest that now has cut roots. The literal lists and counts that are per forest (few_equal_literals counts the literals of the whole forest, the net engine counts forest.roots().len()) are restricted, or the forest passed to an engine is the sequent's alone. The target set's counters stay identical (D17).
- Met when: A test shows that prove_goal on a goal inside a forest with a cut returns the same engine and statistics as on the forest without it. bench/targets.sh LABEL compared with bench/targets/after-bias.csv: verdict, nodes, splits, memo_hits and memo_entries equal on every row, time within a few percent. A documentation sentence on prove and Outcome says "the proof is cut-free".
- State now: Not met. Evidence: core/src/search/mod.rs:633-640 few_equal_literals reads forest.literals(atom, ..) over the whole forest; :391 is_roots compares the goal with forest.roots(); core/src/search/net.rs:149 `forest.roots().len()`; focus/classes.rs:89 iterates forest.roots(). No accessor tells a proof that it is cut-free.

### R99. The reading and the checker's succedent conditions handle a cut pair
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:19-21.
- API item: Reading::new, Derivation::two_sided, Mode.
- Requirement: Cut is defined in the intuitionistic and the affine mode and with Mix. The one-succedent condition needs the cut pair read as one hypothesis and one output (A on the output side, A⊥ as a hypothesis in the other premise), so Reading::new does not treat cut roots as further goals (it fails with SeveralGoals or NoGoal on extra roots), and the checker's conditions R1 to R3 apply to Cut. Mix with cut stays forbidden in the intuitionistic mode.
- Met when: Tests of Interactive::new, cut and proof in Mode::INTUITIONISTIC, in affine mode and with Mix; two_sided_derivation of a proof with a cut prints the premises `Γ, A ⊢ B` and `Δ ⊢ A`; Error::IntuitionisticMix is still raised.
- State now: Not met. Evidence: core/src/occurrences/reading.rs:221 `let roots = forest.roots();` and :301 take the goal and positions from the roots only; core/src/proofs/check.rs Problem::Succedents (R1 to R3) has no cut case.

### R100. Cut elimination on proof terms, step by step and to the end
- Steps: 34 cut (session 2).
- Sources: plan/34-cut.md:19-21,30.
- API item: new cut elimination on Proof (module proofs::cut or similar).
- Requirement: Elimination on terms, one step at a time and to the end, has a public API: one function that does a step (which cut and which reduction case applies, and the new Proof) and one that runs to the normal form, both over the same Forest and returning a Proof that passes check. The strategy (which cut first) is deterministic and a value. A node does not record its sequent, so a top or an absorbed context in a commuting case cannot be read off the node; the elimination takes the contexts from the checker's pass (State and Facts, now crate-private) or from the derivation view, and the term form stays valid for top. Bounds and the stop are R32.
- Met when: Public items with documentation; a test per reduction case (ax, ⊗ and ⅋, & and ⊕, 1 and ⊥, ! and ? with Copy, Quest and Weaken, ⊤, and Mix); each result passes Proof::check in the mode of the input; the end state has no Cut node.
- State now: Not met. Evidence: core/src/proofs/mod.rs: no function builds a new Proof from an old one except Proof::new; core/src/proofs/check.rs:808 `pub(crate) trait Observer` and :832 `pub(crate) struct Facts` are the only access to derived contexts; core-proofs.md says a node never records the sequent it proves and Top(o) does not say what context the ⊤ absorbs.

### R101. Cut elimination on nets is a step function over the same forest
- Steps: 34 cut (session 3).
- Sources: plan/34-cut.md:20-21,30 ("on MLL nets (and on the nets of step 33)").
- API item: new net cut-elimination step on ProofStructure and its JSON form.
- Requirement: Cut elimination on MLL nets is a step function on ProofStructure: for an axiom-cut contraction, a ⊗/⅋ cut that becomes two cuts on the subformulas. The result lives over the same forest with some occurrences gone (so a structure needs a mask of present occurrences, or a new Forest per step) and must be a correct net again. The JSON form (R8) gets the cut links and the mask and still reads cut-free nets as before. Each step reports which cut it reduced, for the drawing. The same holds for the MELL nets of step 33: a cut link may join two boxes' formulas, and the step function has the exponential reductions: dereliction, contraction (a box is duplicated, which adds vertices, R77), weakening (a box is erased) and box in box. The JSON form gets the keys for the cut links, the present-occurrence mask and the boxes (R8), additively.
- Met when: A property test over nets of generated proofs with cuts, MLL and MELL: every step result passes is_correct (the criterion at each box depth for boxed nets), the number of cut links falls or the cut formulas get smaller (for the duplicating and erasing steps a measure that decreases is named in the test, since the cut count may rise), the end has no cut, and its sequentialization checks against the same sequent. A reduction test per case: axiom, ⊗/⅋, dereliction, contraction, weakening and box in box. A serde round trip of a net with cuts and of a boxed net with cuts, and the old cut-free documents unchanged.
- State now: Not met. Evidence: core/src/serialize/nets.rs:11-20 Net is sequent, mix and links; core/src/nets/mod.rs: structures are only built from a whole forest with all occurrences present.

### R102. The Danos-Regnier test at each box depth is reachable through is_correct
- Steps: 33 MELL nets (D6a).
- Sources: plan/33-mell-nets.md:18-19; plan/README.md D6a.
- API item: ProofStructure::is_correct, is_acyclic, Graph (Yeo test), NetError.
- Requirement: The test at each box depth with boxes contracted is reachable through is_correct (witnessed) and an allocation-free verdict, with the outer test treating a box as one node. It does not use any search or proof-checker code. The variable arity of ? nodes and boxes fits the CSR graph, whose layout is parent slot first, the children, then a literal's axiom slot.
- Met when: is_correct accepts MELL structures; the brute-force switching enumeration test in nets::graph is extended to boxes and agrees on small structures (R201).
- State now: Not met. Evidence: core/src/nets/graph.rs:133 the CSR layout assumes binary ⊗/⅋ and unary parents; graph.rs:145-146 counts only Tensor and Par switched edges; nets/mod.rs:406 is_correct has no box notion.

### R103. The net engine's symmetry-break data is one general structure
- Steps: 35 net engine pruning.
- Sources: plan/35-mll-engines.md:24-27.
- API item: search::net private Engine (copy_before, copy_after, ordered, admissible).
- Requirement: The symmetry-break data (today chains of equal literal conclusions) is one general structure that also holds the leaves of a pure tensor or par tree of equal literals, with the admissibility test in the one place (admissible and ordered) and the data built once per run from the forest, so that the leaf break and the compound-conclusion break (R83) add groups and not new code paths.
- Met when: A differential test of the pruned engine against an unpruned one on generated sequents with equal leaves under one tree, plus agrees_with_the_focused_engine extended with such sequents.
- State now: Partly. Evidence: core/src/search/net.rs:197-222 copy_before and copy_after are built for conclusions only (net.rs:240-256); net.rs:455-490 admissible and ordered; .claude/rules/core-nets.md says the leaf break is a follow-up.

### R104. Planar linking: crossing links are refused in the ordered modes
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:15-17.
- API item: nets::ProofStructure, search::net (Nets), nets::NetError.
- Requirement: The net engine needs a planar linking: links may not cross in the cyclic order of the literals, which is the forest's preorder id order once the roots keep their written order (R53). ProofStructure::link, from_links, is_correct and the engine's admissible-partner test take the ordering as part of R79's descriptor and reject a crossing link with a new NetError variant naming the two crossing links, while the correctness (switching acyclicity) test is reused. The symmetry-breaking of equal literal conclusions and the skeleton merge are turned off or adapted in the ordered mode. The net JSON form gets the ordering (R8, R79) so that a stored net is checked in its own calculus.
- Met when: A crossing linking is refused in the cyclic mode and accepted in the commutative one; the planar search agrees with a brute-force enumeration of linkings on small random cyclic sequents (as the net tests enumerate switchings); the ProofStructure JSON round-trips with and without the ordering.
- State now: Not met. Evidence: core/src/nets/mod.rs:182-196 ProofStructure has forest, mix, partner, links, graph and skeleton and no ordering; :46 NetError has no crossing variant; core/src/search/net.rs:7-22 describes the symmetry break on equal literal conclusions.

### R105. The net engine's search loop is reusable with a second correctness criterion
- Steps: 35 essential nets for IMLL (D7).
- Sources: plan/35-mll-engines.md:35-41; plan/later.md:199-210.
- API item: search::net Engine (the search loop shared with the essential criterion), net::parallel cubes.
- Requirement: The essential-net engine is the net engine's linking search with the essential criterion in place of Danos-Regnier acyclicity (the dominator condition added to the complete branch). D7 forbids two implementations of one algorithm, so the net engine's loop (choose, frames, tests, stop, cubes) is reusable with a second criterion rather than copied. The cube runtime is written against the concrete Engine type, and whether the essential engine runs on a pool (Engine::parallel) is decided and documented. The classical engine's counters do not change.
- Met when: The net engine's Engine is parameterised by a criterion (trait or enum) with identical behaviour for the existing one (the nodes, links and tests columns equal to the baseline); jobs greater than 1 works or parallel() is false for the new variant.
- State now: Not met. Evidence: core/src/search/net.rs:197-224 is a concrete Engine with ProofStructure and Scratch; net.rs:847-980 the parallel module is hard-wired to it; core/src/search/mod.rs:742-744 Engine::parallel is a matches! on variant names.

### R106. One shared, iterative unifier with an undo trail
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:18-19.
- API item: new crate-private unification module (term bindings, trail with marks, occurs check, levels or Skolem terms).
- Requirement: One shared, iterative (non-recursive) unifier with an undo trail (a mark per choice point, undo by truncation), an occurs check and an eigenvariable-safety discipline (Skolem terms or variable levels), usable by the focused, two-sided and net engines and by the parallel workers. A worker starts from its spawning engine's trail and bindings never cross alternatives.
- Met when: A module with unit tests (unify, undo to a mark, occurs check, deep terms on a small stack) and a documented place in core-focus.md and core-nets.md; engines use it only on first-order goals.
- State now: Not met. Evidence: no unify or trail module exists (ls core/src/search); plan/reports/26-focused-engine.md:1228-1247 says where the trail goes (beside the branch on Engine, undone where pending arena nodes are released); the term "trail" in core-focus.md refers to the split search's own trail.

### R107. Every ground-atom prune of the focused engine gets a first-order version or is switched off for first-order goals
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:18-19; plan/notes/research/impact-quantifiers.md (section 1, finding 3; section 2.5); plan/notes/research/fo-linear.md (section 3.4); plan/notes/research/38-first-order.md (sections 3 and 4).
- API item: search::focus (Classes, Counts, memo keys, forced splits, initial rules) and the parallel cubes.
- Requirement: Every prune that assumes ground atoms is gone through and given a first-order version, or switched off on first-order goals only: dual lookup by unifiability, forced splits as choices, classes of interchangeable occurrences over ground subterms, memo and loop-check keys that include variable bindings, per-predicate counts, and the independence of `&` premises that the cubes and the parallel runtime assume (shared metavariables break it). The propositional code path keeps its counters. The premises of a tensor and of Mix share metavariables as those of `&` do (∃x.(p(x) ⊗ q(x)) gives p(X) ⊗ q(X)), so the right premise runs under the left's bindings and a failure on the right backtracks into the left for another answer. The engine's present control flow returns one proof per goal (Found::Proved) and records one per key (Entry::Proved), which loses completeness: p(a) & p(b), q(b) ⊢ ∃x.(p(x) ⊗ q(x)) is provable with x = b, but the left premise's first proof binds X = a. A first-order engine therefore enumerates the answers of a premise (success continuations or answer lists), or chooses its witnesses from a finite candidate set (finite only without function symbols), and a memo entry, success or failure, that depends on an unbound metavariable is not a fact. The mechanism is step 38's design. The propositional path keeps one answer per goal and its counters.
- Met when: An item-by-item list in core-focus.md (plan/reports/26 item 7 is the starting list); bench/targets.sh counters identical to bench/targets/after-bias.csv on the propositional target set; parallel tests with jobs greater than 1 on first-order goals; a test that the example above is proved by the focused engines, on one thread and with jobs greater than 1.
- State now: Not met. Evidence: core/src/search/focus/ (classes.rs, counts.rs, memo.rs, split.rs, parallel.rs) keys everything by occurrence id and atom; plan/reports/26-focused-engine.md:1250-1285 lists the prunes; focus/mod.rs:458-463 Found::Proved(NodeId) and focus/memo.rs:85-87 Entry::Proved(NodeId) hold one answer; search/focus/parallel.rs assumes independent `&` premises.

### R108. Proof structures have room for unification on links
- Steps: 38 first-order (LinearOne).
- Sources: plan/38-first-order.md:18-19,33-34.
- API item: search::net, nets::ProofStructure, serialize/nets.rs.
- Requirement: The net engine and ProofStructure have room for unification on links: a structure carries (or can derive) one unifier for all its links, is_correct can include the quantifier and eigenvariable dependency condition, sequentialize can emit the universal and existential nodes, and the JSON form can add an optional substitution (R8) without breaking existing files.
- Met when: ProofStructure gains an optional substitution field with a serde default; propositional links and JSON are byte-identical; tests on a quantified MLL net.
- State now: Not met. Evidence: core/src/nets/mod.rs ProofStructure::from_links, link and unlink work over OccId pairs; serialize/nets.rs writes `links: [[0,2],...]` only.

### R109. A first-order goal enters through the same Task and Decide interface, with a seam that isolates the propositional fast path
- Steps: 38 first-order (D17, D19).
- Sources: plan/38-first-order.md:18-19,26-29,35-37; plan/README.md:712-720,729-738.
- API item: search::Task, Decide, Answer, DISPATCH, Row, Engine::admits, focus memo key (OccSet), the focused engine's structure, literal matching (initial, dual_in, dual_from, mark_literals, meets).
- Requirement: A first-order goal enters through the same Task, Decide and Answer interface (the Task's forest carries terms; the verdict construction is unchanged) and gets a DISPATCH row with a quantifier fragment bit. The interface lets a first-order engine carry a substitution and a trail of bindings (undone on backtracking) and key its memo on the bindings, while the propositional engines keep their types: either generic over the sequent state or a duplicated fast path, so that the target set's counters stay identical and its pinned CPU time stays within a few percent. The engine code is structured so that a first-order variant can be added as a type parameter or a duplicated module without touching the propositional monomorphization: the atom and literal access sits behind a small internal trait or isolated functions instead of reading Forest literal lists directly all over the engine. Engines that cannot take quantifiers refuse them (R135).
- Met when: The audit (step 28) isolates literal matching behind one internal interface and bench/targets.sh counters are identical before and after that refactor; a spike in a jj workspace shows the trail in the focused engine with the counters (verdict, nodes, splits, memo_hits, memo_entries) equal to bench/targets/after-bias.csv; the dispatch test is extended and the rustdoc table of Engine updated.
- State now: Not met. Evidence: core/src/search/mod.rs:406-422 Task holds forest, goal, fragment, mode, reading and roots only; :456-470 Decide::decide takes no state beyond task, options, account and stop; the focused engine's memo is keyed by OccSet of a stable sequent (focus/memo.rs); plan/reports/26-focused-engine.md:1254-1260 names the functions that rely on the forest's literal lists, and whether they are behind one seam was not checked.

### R110. The first inverse engine declares whether it is parallel and shares the batch's memory bound
- Steps: 37 inverse method (D7, D19).
- Sources: plan/37-inverse.md:17-18; plan/README.md D19.
- API item: search::parallel::Pool, Options::jobs, Engine::parallel, search::batch::run.
- Requirement: The first inverse engine may be sequential, but then it declares so (Engine::parallel is false, documented "runs on the calling thread whatever jobs says", as Additive and Horn do), and under search::batch its database is charged to the shared per-worker share of the memory bound like any other engine, so the batch's bound and ordering guarantees hold.
- Met when: An Engine::parallel arm and an Engine rustdoc sentence; a batch test with the engine forced.
- State now: Not met. Evidence: core/src/search/mod.rs Engine::parallel (about line 740) is `!matches!(self, Additive | Horn)`; core-batch.md describes how the memory bound is shared among workers via Account; there is no inverse variant.

### R111. An incomplete engine can say "saturated" and "gave up" correctly
- Steps: 37 inverse method.
- Sources: plan/later.md:214-227.
- API item: search::Reason, Verdict::Unknown, Answer of Decide (an incomplete engine's "not found"), Decide::admits.
- Requirement: The focused inverse method is a semi-decision for MELL and ILL with large Theta and a decision for MALL. The engine interface lets an engine say "saturated, no proof" (Unprovable, with its own refutation) in the complete cases and "gave up" with its own bound (a Reason for a saturation or clause limit) in the incomplete ones. It charges its clause store and subsumption index to the search's Account and polls the stop inside saturation (R36, R37), and the front door does not turn an incomplete engine's exhaustion into a refutation.
- Met when: A Reason variant for the engine's bound (non_exhaustive, with a JSON tag and a CLI advice arm in `unknown`); Answer::refutation used only by complete cases; Decide::admits refusing the incomplete case when forced; the reference-prover differential test (R209) joined by the engine.
- State now: Partly. Evidence: Answer::refutation exists for an engine's own refutation (core-search.md "An engine's own refutation", used by Horn); Reason has Stopped, RecursionLimit, CopyBound, MemoryLimit and IndexLimit only (serialize/search.rs:76-86); the memory Account is charged by "everything which grows" (core-search.md "The memory bound").

### R112. An opt-in failure trace beside the proof
- Steps: 31 Rocq library (the failure-certificate escalation).
- Sources: plan/later.md:491-505.
- API item: search::Decide, search::Answer, a new failure-trace channel, the focused memo.
- Requirement: The failure-certificate escalation needs the focused engine to record its exhaustive failure as a DAG of synchronous choices with the memo's sharing kept. The engine interface has room for an optional recorded trace beside the proof: an opt-in Options field (off by default so that the counters and speed of decided runs do not move), the trace charged to the memory Account, and Answer carrying it to prove_goal, which turns it into a Refutation variant. The memo keeps, per failed key, the node that explains it.
- Met when: A new Answer field and an Options setter (default false); a test that the engines' counters with it off equal the pinned baseline (bench/targets after-bias.csv); a Refutation variant holding the DAG with a checker (R124).
- State now: Not met. Evidence: core/src/search/mod.rs:428-439 Answer has result, statistics, net and `refutation: Option<Refutation>` only; Decide::decide (mod.rs:478) returns Answer; the focused memo stores failures as keys without justification (core-focus.md, memo.rs).

### R113. An entry for deciding ordinary logic, with room for a second procedure
- Steps: 31 Rocq library (certified refutations); 28 audit; web client.
- Sources: plan/later.md:476-483,499-502,638-657,687-708.
- API item: new ordinary decide entry (an ordinary::decide dispatching between the embedding and a native procedure such as G4ip), Decide and Task.
- Requirement: A terminating calculus for intuitionistic logic (G4ip) is the natural countermodel finder, but Decide and Task are crate-private and shaped for a linear forest, goal, Fragment and Mode. A decision procedure on ordinary::Sequent needs its own entry and a result type that is not tied to Proof, with the same stop closure, memory Account and Error conventions. The ordinary layer today is a composition in the command (translate, prove the image with the command's defaults, read back, check, draw); wrappers and the web client need it as one library call returning a decision value (and a place for a countermodel, R72). The entry has a table like DISPATCH, and the command and the web client use it instead of hand-calling prove on the image.
- Met when: A documented ordinary decide entry point taking a stop and options exists, with a place for a second procedure to plug in.
- State now: Not met. Evidence: core-ordinary.md:95-98 "Deciding is the caller's"; there is no decide function in core/src/ordinary/mod.rs or translate.rs (the public functions are translate, the Image accessors, read_back and check); cli/src/ordinary.rs holds the decision.
- Conflict: see Conflicts for the author, C2.

### R114. A loop check for the images of the ordinary-logic translations
- Steps: ordinary logic (done as step 25; the termination left).
- Sources: plan/later.md:620-636,674-685.
- API item: search::Options::copies, Reason::CopyBound, the focused engine's memo, a new loop check.
- Requirement: For the images of the intuitionistic and minimal translations (all hypotheses under !) the search should terminate: a branch-local loop check for dyadic sequents (worth having for MELL and LL in general) or a bound computed from the input and proved enough. The library keeps room: an Options knob (default off so that the pinned counters do not move), a Refutation variant or reason for "refuted by loop check", and a memo whose entries may become branch-dependent (a failure under the loop check is not a failure of the sequent). 35 of the ILTP problems deepen past 30 and are all non-theorems that this would refute.
- Met when: A documented engine option plus a test that SYJ207-like non-theorems become Unprovable while the verdicts of every other target-set row are unchanged (bench/targets.sh against after-bias.csv).
- State now: Not met. Evidence: core/src/search/mod.rs:851-885 Options has no loop-check or sufficient-bound field; the focused memo is keyed by the stable sequent only (core-focus.md memo section); Reason::CopyBound answers instead (mod.rs:1416).

### R115. The focused search may hand a goal to the net engine at the recursion limit
- Steps: 26 (done), 27, later engine rows.
- Sources: plan/later.md:740-744,825-830.
- API item: search::DISPATCH, Engine, Decide (focused search handing over to the net engine at the recursion limit; the Horn test on the goal).
- Requirement: A focused search that reaches the recursion limit may hand the goal to the net engine (the dispatch's net rows stand in for it today). The forward bound reads the goal's members, not the forest's roots, for goals off the roots.
- Met when: A fallback path in Decide for focus, with a test on wide-m1 at 2048 literals without the net row; schedule::chains takes the goal.
- State now: Partly. Evidence: the Horn test on the goal is done (focus/schedule.rs:228 `pub(super) fn chains(forest, goal)` iterates the goal); the hand-over is not (core-search.md:161-164 says it is the follow-up and the net row FewEqualLiterals is kept).

### R116. A pool kept across calls is part of the public API
- Steps: 32 web client; 24 (Runtime).
- Sources: plan/later.md:1257-1259,1228.
- API item: search::Pool, Options::pool.
- Requirement: A runtime value kept across calls, for a caller that closes many small goals (an interactive session, a server), sits in the public API next to the plain-data options. It is absent on wasm.
- Met when: search::Pool and Options::pool(Pool) exist under parallel, documented and tested for reuse.
- State now: Met. Evidence: core/src/search/parallel.rs:47-70 `pub struct Pool` (clones share pools, nothing global); the Options field `pool` behind cfg(feature = "parallel") at search/mod.rs:877.

## Proof term and checker

### R117. The proof term and the checker are specified completely and frozen, and the first implementation stays
- Steps: 31 Rocq library (D20, D23); 33, 34, 38 add to it.
- Sources: plan/31-rocq-library.md:62; plan/later.md:23; plan/later.md:388-397; plan/README.md:740-743,780-794.
- API item: proofs::Node and its rule documentation, proofs::check (the dyadic checker as specification), Proof::check, proofs::oracle (test-only first implementation), proofs::check::Pass::rule, serialize/proofs.rs tags, proofs::Dyadic.
- Requirement: The Rocq library mirrors the proof term and proves the checker (an inductive of the nodes over occurrence ids, a checked relation with Theta, Gamma and the absorbing flag, soundness against the library's own sequent calculus), so the term and the checker's rules are stated completely and are stable through the audit: a change after step 31 starts would be a change of the Rocq development. The documentation of Node and core-proofs.md give the exact premise and conclusion of every rule in every mode (Mix, affine weakening, ?w), the rule table, the three clauses of the one-succedent condition (R1 to R3), the absorbing flag, the mode table (Mix forbidden when intuitionistic, Weaken of a non-? only when affine, Mix only when the mode has mix), the conclusion at the root, the multiset reading of the linear zone and the occurrence-id semantics, as a per-rule specification that the Rocq function can be written against. The open questions about affine Weaken placement and the dyadic shapes are closed. The Rust-only refusals (Problem::Memory, Surplus, TooManyNodes) are documented as outside the verified function (soundness needs accepts-implies-derivation only). The oracle (oracle.rs) is kept, not deleted or inlined by step 28, and agrees with the optimized checker on verdict and error, so that translating the oracle is translating the checker. The node set is frozen at the release; cut, boxes and quantifiers come as additive variants. Whether nullary Mix exists is decided by the author before step 31 starts, because it is a rule of the calculus, the checker, the oracle and the Rocq mirror; the choice of placement for weakening at step 33 (R78) depends on the answer.
- Met when: A normative "semantics of a term" section in the rustdoc of Node or Proof::check, or in core-proofs.md; a per-rule table in the rustdoc of proofs::check; a note naming which Problems are resource refusals and not rule failures; `agrees_with_the_first_implementation` still exists and passes; a grep for `mod oracle` in core/src/proofs/mod.rs succeeds; the audit report lists no open shape question.
- State now: Partly. Evidence: core/src/proofs/mod.rs:100-157 documents each node's rule in dyadic form; core/src/proofs/check.rs:3-30 module documentation gives the pass but not a per-rule table, and :1281-1300 holds the mode conditions; core-proofs.md:50-231 states the rules and describes State as characterising "exactly the set of dyadic sequents the subterm proves"; core/src/proofs/oracle.rs:1-80 is the test-only first implementation (`#[cfg(test)] mod oracle;`); Problem is non_exhaustive (check.rs:463); the Status log (plan/README.md, step 2) assigns the affine Weaken placement and the dyadic term shapes to step 7.
- Conflict: see Conflicts for the author, C3.

### R118. The Rust node variants and the Rocq constructors correspond one to one
- Steps: 31 Rocq library; 34 cut.
- Sources: plan/31-rocq-library.md:99.
- API item: proofs::Node (closed enum, 16 bytes), serialize/proofs.rs tags, the Rocq node datatype.
- Requirement: Rust's Node variants and the Rocq proof-term constructors correspond one to one, with the JSON tags as the shared names (ax ⊗ ⅋ 1 ⊥ & ⊕₁ ⊕₂ ⊤ ! ? copy wk mix). Adding a variant such as Cut in step 34 fails a test until the Rocq datatype, the checker case and the soundness case exist. The Rust side has a single list of the variants (a Node::NAMES, or a test that walks the match arms) which a test compares with the constructor list the Rocq library publishes. The writer's match on Node stays exhaustive.
- Met when: A test in core (and the flake check) compares the list of Node tags with the constructor names in rocq/; the writer in export/rocq.rs has no wildcard arm over Node.
- State now: Not met. Evidence: core/src/proofs/mod.rs:85-125 Node is a closed enum with no #[non_exhaustive] and no ALL list; Rule::ALL and names_round_trip (proofs/style.rs:330) cover only the derivation's Rule; no rocq/ directory exists.

### R119. A Cut node that fits the 16-byte Node and names a formula outside the forest
- Steps: 31 Rocq library (room for cut); 34 cut (session 1); D23.
- Sources: plan/34-cut.md:19-21,9-12; plan/31-rocq-library.md:99; plan/later.md:31-33; plan/README.md:780-794.
- API item: proofs::Node (new variant Cut), Proof::new, principal(), occurrences(), premises(), name(), map_premises(), the reachability pass.
- Requirement: Cut is one more case of Node. Node::Ax and Tensor fit 16 bytes by a static assertion. A Cut that names the cut formula, its dual and two premises is four u32 plus a tag, which does not fit, so the dual is implied by the forest (cut roots allocated in pairs, or a derivable offset) or the assertion is changed deliberately; a form such as Node::Cut(OccId, NodeId, NodeId) stays 16 bytes. A dual found by offset holds only for the dual that keeps child order; R57 states that invariant and the refusal of Cut in the ordered modes. Principal(), occurrences(), premises(), name(), map_premises(), Proof::new's bounds loop, the reachability pass and every other exhaustive match on Node get the arm, and there is no wildcard arm over Node. Proof::new rejects a Cut naming an occurrence outside the forest. The place of the cut formulas is R57. Whether cut is allowed is a rule-set parameter next to mix and affine (Mode or check options). The search stays cut-free unless asked, and engines keep returning cut-free proofs that the checker accepts with cut disallowed. A wire tag `cut` is added (R13), with a Rule::Cut label (R60), notation rows in the four exports, and the oracle updated in step (core-proofs.md).
- Met when: `const _: () = assert!(size_of::<Node>() == 16)` still holds with Cut, or the report records the change and why; a design note in core-proofs.md shows Node::Cut with its checker rule; the compiler names every site; all existing snapshots stay.
- State now: Not met. Evidence: core/src/proofs/mod.rs:96-137 the Node variants, :140 the 16-byte assertion, :143-205 principal, occurrences, premises and name, :410-425 map_premises, :322 Proof::new(forest, nodes, root); no variant for cut. grep -rniE '\bcut\b' core/src finds only the test-only reference prover's copy budget (search/reference.rs:136).

### R120. The checker has a rule for Cut in every mode, and the oracle has it too
- Steps: 34 cut (session 1).
- Sources: plan/34-cut.md:19-21,9-12.
- API item: proofs::check (examine, State, Problem), proofs/oracle.rs.
- Requirement: The checker's rule for Cut works in the dyadic pass for every mode. It joins two premise states, `⊢ Θ; Γ, A` and `⊢ Θ'; Δ, A⊥` giving `⊢ Θ∪Θ'; Γ, Δ`, with the any flag of ⊤, the Quest and Copy bookkeeping, Mix and affine weakening all taken into account. A new Problem arm says why a cut is wrong (the two formulas are not duals, or a premise lacks its cut formula). The checker's reader counts, the Surplus bound (a rule takes two members) and the Memory accounting still hold. The first implementation (oracle.rs) gets the same rule, and agrees_with_the_first_implementation covers cut proofs.
- Met when: Unit tests in check.rs next to accepts_every_rule: cut accepted and rejected in the classical, affine, Mix and intuitionistic modes, and with ⊤ absorbing on one side; Problem::Cut* has one arm in CheckError::write; agrees_with_the_first_implementation compares the verdict and the error on cut proofs; holds_no_more_than_its_bound is unchanged.
- State now: Not met. Evidence: core/src/proofs/check.rs:463-511 Problem has no cut arm, :730 examine, :808 Observer; core-proofs.md says "Surplus: a rule takes two members of a premise's zone at most"; core/src/proofs/oracle.rs is test-only and must be edited in step.

### R121. The checker handles contexts that are sequences in an ordered mode
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-19; plan/later.md:271-276.
- API item: proofs::Proof, proofs::Node, proofs::check (Multiset, Facts, Bag zones) and the proof JSON form.
- Requirement: A cut-free proof of cyclic MLL and of the Lambek calculus is a tree of the same rules without exchange, but the checker's state is a sorted multiset of occurrence ids (exchange is implicit). In an ordered mode the checker tracks the context as a sequence (cyclic: up to rotation), checks that the left premise of a ⊗ is the contiguous interval before and the right premise the one after (cyclic: contiguous arcs), that ⅋ keeps its two operands adjacent, and that an axiom links literals adjacent in the sequence, with the same CheckError type extended by an order violation, behind the same examine pass. Node need not change if the proof records no exchange; if the order cannot be recovered from the forest and Node alone, Node carries it. The proof JSON stays valid for the commutative logic, and check(proof, mode) rejects a commutative proof that is not planar in an ordered mode. The ascending-id order of Inference.sequent stays for the commutative case (R61).
- Met when: A planar proof passes check in the cyclic and the Lambek mode; a proof that needs exchange (`A, B |- B * A`; for the cyclic mode this reads under the reversed one-sided convention of R56, where the dual of a product reverses its operands, and the entry's test states the one-sided sequent it means) passes in the commutative mode and fails in the ordered ones with the named CheckError; property tests over random sequents in proofs/check.rs agree with the net engine's verdicts.
- State now: Not met. Evidence: core/src/proofs/multiset.rs:6-10 Multiset is an "ascending list with repeats"; core-proofs.md says gamma is a multiset (Bag, a table of counts); core/src/proofs/mod.rs:117-158 Node has no order fields (size asserted at 16 bytes); check(proof, mode) at proofs/check.rs:674 takes the Mode, and the mode is not in the proof.

### R122. MELL nets convert to and from proofs through boxes
- Steps: 33 MELL nets.
- Sources: plan/33-mell-nets.md:19-20,18-21.
- API item: ProofStructure::sequentialize, ProofStructure::from_proof, Graph (Yeo test).
- Requirement: sequentialize emits Bang, Quest, Copy and Weaken nodes (and keeps Mix) for a MELL net, and from_proof reads Bang, Quest, Copy and Weaken as well as Ax, so that every proof the checker accepts gives a correct net in the modes where nets exist. The result passes Proof::check in every build. The place of the ? nodes is R78.
- Met when: A round-trip test on generated MELL proofs (proof, net, proof) through the checker, plus net(proof) equal to net(sequentialize(net(proof))); the generator in search/generate.rs has an exponentials flag already.
- State now: Not met. Evidence: core/src/nets/mod.rs:249-263 from_proof reads only Node::Ax; nets/sequentialize.rs:27 emits Ax, Tensor, Par and Mix only.

### R123. A correctness criterion for essential nets, independent of any search
- Steps: 35 essential nets for IMLL.
- Sources: plan/35-mll-engines.md:35-38; plan/later.md:199-210.
- API item: ProofStructure::is_essential (new), NetError variants, ProofStructure::is_correct, Reading and Position.
- Requirement: A criterion independent of any search decides whether a linking of an intuitionistic net is an essential net: directed acyclicity over the polarized reading, plus the dominator condition. ProofStructure can be asked "is it an essential net of this reading", with NetError witnesses (a directed cycle, an occurrence with no dominator) printable by describe(), so that the engine's result, deserialized nets and tests share one definition of correctness. The net engine's complete branch can call it without changing the classical engine's counters. The verdict itself never depends on it (every sequentialization of a classical net of an IMLL sequent is intuitionistic), and IMLL by embedding stays the default until the harness says otherwise. is_essential is a method over a Reading and needs no flag of its own; the criterion value of R79 may name it as a variant when the engine of step 35 is routed by it.
- Met when: A method such as is_essential(&Reading) with witnesses like SwitchingCycle; tests that it agrees with the two-sided engine's verdict on generated IMLL sequents, with a brute-force reading of the dominator definition, and with the embedding route on ILTP and LLTP IMLL problems from step 14.
- State now: Not met. Evidence: grep for dominat, directed and closure under core/src finds nothing in nets/ or search/net.rs; core/src/nets/mod.rs:182-197 has only `mix: bool` as criterion and :406 is_correct is Danos-Regnier only. The polarisation exists (occurrences/reading.rs gives every occurrence a Position, and Task is handed the reading).

### R124. An independent checker for refutations
- Steps: 31 Rocq library (certified refutations and its escalations).
- Sources: plan/later.md:473-510.
- API item: new Refutation::check (or refutation::check(&Sequent, Mode, &Refutation)).
- Requirement: A refutation is only worth certifying if a checker that shares no code with the engines can re-verify it, as Proof::check does for proofs. The invariants Unbalanced and Equation, the Horn engine's StateEquation weights (a Farkas vector, plus the exact check, for which the checker needs the payload of R70) and any later model or failure DAG need a public check(&sequent, mode) returning a CheckError-like type that distinguishes a faulty certificate from a refused (resource-bounded) check. The fragment, mode and bounds under which the claim holds are part of the value (R70, R71).
- Met when: Refutation::check exists with tests that reject mutated certificates, and the engine's own answer is passed through it in debug assertions as proofs are.
- State now: Not met. Evidence: core/src/search/mod.rs:1257 Refutation has no check method (only needed()); prove_goal checks proofs only (.claude/rules/core-search.md:29-36); the StateEquation exact check is inside the Horn engine (core-search.md:67-71).

### R125. Proof terms carry witnesses and eigenvariables out of line
- Steps: 38 first-order; 31 Rocq library planned with quantifiers (D17, D23).
- Sources: plan/38-first-order.md:19-20; plan/later.md:318-324; plan/README.md:712-720,780-794.
- API item: proofs::Node (new Forall and Exists with a witness), Proof, serialize tags, the Rocq library's formula and rule inductives.
- Requirement: Proof terms carry witnesses: an existential rule names a term and a universal rule an eigenvariable, kept out of line in a term arena of the Proof, so that Node stays a 16-byte Copy enum (static assertion) and propositional proofs and their JSON are unchanged (R17). The nodes fit the 16 bytes (for example Exists(OccId, TermId, NodeId) is a tag plus three u32), get a JSON tag, a derivation Rule and a dispatch row in the checker. The proof an engine returns is closed under the final substitution (no open metavariables). The Rocq library of step 31 is planned with quantifiers from the start, so its formula and proof-term inductives leave the same rules in place, even if unused in the propositional development.
- Met when: Node variants added with the size assertion kept (or a deliberate new one), Proof owns a term table (empty for propositional proofs), engine proofs are checked by the checker on first-order goals, and a note in core-proofs.md names the node shapes with the old pinned strings unchanged.
- State now: Not met. Evidence: core/src/proofs/mod.rs:117-158 Node has no quantifier node (`const _: () = assert!(size_of::<Node>() == 16)` at :158) and Proof::new(forest, nodes, root) has no term table; core-ordinary.md:101 says "Node gains Forall/Exists with a bound variable"; later.md:322-324 says NanoYalla is propositional, so certificates wait for the Rocq library.

### R126. The checker verifies witnesses and the eigenvariable condition, with instances in the zones
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:19-20; plan/later.md:306-311,313-318.
- API item: proofs::check (Dyadic, Bag, State, Problem, CheckError, check_within), the oracle, Node::Copy(OccId, NodeId).
- Requirement: The checker verifies witnesses and the eigenvariable condition (the universal variable is fresh: not free in the rest of the conclusion nor in the term-table entries the premise uses) in one pass, within the memory bound, sharing no code with any engine. Under ! and ? every copy needs its own instance of a quantified formula, so the checker's zone members (today an OccId with a count) and the derivation's Inference.sequent can carry an instance (occurrence plus substitution, or a fresh instance id) without making the propositional pass slower or larger: a zone key generic over OccId, or an instance id with the propositional case staying a bare u32. New Problem variants (non_exhaustive) name the violated condition; term equality is an arena-id comparison after substitution so the pass stays linear. The pass's memory account (Pass::held) counts the free-variable sets.
- Met when: Checker unit tests for a wrong witness, a captured eigenvariable and a reused eigenvariable; a test-only independent oracle for first-order proofs like proofs/oracle.rs; the checker benchmark on the engines' largest proofs unchanged (22 ms for 566 490 inferences).
- State now: Not met. Evidence: core/src/proofs/check.rs: Dyadic zones are multisets of OccId (multiset.rs), Problem is non_exhaustive with Forbidden, Kind, NotDual, Missing, NotEmpty, Differ, NotUnderQuest, Surplus, Conclusion and Memory; oracle.rs is test-only; no variable notion; Node::Copy(OccId, NodeId) at proofs/mod.rs:147; the check time is pinned in core-search.md.

### R127. A forward derivation converts into the existing Proof terms with no new rule
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:16-18 and proof-search-specifications.md:235.
- API item: Proof and Node arena, Answer::of_arena, proofs::Node.
- Requirement: A forward derivation (a DAG of derived sequents, each premise derived before its conclusion) is convertible into the existing Proof terms with no new rule: sharing a derived sequent used twice is one node (the arena is a DAG), a subsuming use of a smaller sequent in affine mode is a Weaken node below, and an unrestricted hypothesis is a Quest and Copy pair. The root passes the existing checker unchanged, so the front door's check of every proof of the roots stays the only judge.
- Met when: Every Proved outcome of the inverse engine passes Options::check (default on) and the engines_agree* tests; no change in proofs/check.rs.
- State now: Partly: the proof term has everything the conversion needs, but the engine whose outcome the Met when tests does not exist (step 37). Evidence: core/src/proofs/mod.rs:117-155 Node has all rules including Weaken and Mix; .claude/rules/core-proofs.md says a shared subproof is stored once and Proof::new renumbers and drops unreachable nodes; Answer::of_arena (search/mod.rs:437) builds a Proof from the root, a Vec<Node> and Statistics.

### R128. The essential-net engine returns its proof by sequentializing the net it found
- Steps: 35 essential nets for IMLL.
- Sources: plan/35-mll-engines.md:35-41.
- API item: search::Outcome.net, ProofStructure::sequentialize, proofs::check, the prove_goal check.
- Requirement: The new engine returns its proof by sequentializing the net it found (Outcome.net is Some(net), a ProofStructure), so that prove_goal's checker call in every build, the --net output and the derivation view work unchanged, and the proof passes the checker in intuitionistic mode.
- Met when: The existing prove_goal check plus a test that every proof of the essential engine equals the checker-accepted sequentialization and that from_proof of it reproduces the net's links.
- State now: Partly (the mechanism is there; the engine does not exist, step 35). Evidence: core/src/search/mod.rs Answer.net is Option<ProofStructure> and prove_goal does one check (core-search.md: "Every proof of the roots has passed the checker ... in every build"); core/src/nets/sequentialize.rs:27 sequentialize; net.rs answer() sequentializes. The essential engine itself does not exist yet.

### R246. The Lambek restriction holds on every sequent of a derivation, and L* is an option
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:18-19; plan/later.md:271-276; plan/README.md D16; plan/notes/research/36-lambek.md (section 2, L against L*; section 3, Fragments and modes, and Terms, checker and derivations; section 4, first bullet).
- API item: fragment::Mode (a choice of empty antecedents, R51), proofs::check and proofs/oracle.rs (a clause R4 beside R1 to R3, a new Problem variant), nets::sequentialize, search::net, Interactive, Error (R134).
- Requirement: In the Lambek mode every sequent of a derivation, not only the input (R55), has at least one input occurrence under the Reading. The checker and its oracle enforce it as a fourth clause next to R1 to R3, with a named Problem variant. Sequentialization checks it at each stage, and the interactive session refuses a rule whose premise would have an empty antecedent. The mode's verdict is that of the calculus L: the planar engine does not answer Proved for a sequent that has only an L* derivation, and answers Unprovable only when no planar net sequentializes under the restriction. The two calculi differ on sequents with a non-empty antecedent: `x/(y/y) ⊢ x` is provable in L* only, since the premise `⊢ y/y` has an empty antecedent, so without a rule on every derived sequent the planar engine would prove a sequent that is not Lambek-provable. The report states the completeness argument (a planar net of an L*-provable sequent is L-provable exactly when some sequentialization meets the clause), or the criterion on nets that replaces it. Whether empty antecedents are allowed is a value of Mode, L the default and L* the option (D16), with its word in the table of mode names (R241), its flag in the command, and its key in the JSON under serde(default) (R10).
- Met when: A test shows that a derivation with an empty intermediate antecedent is refused by check in L and accepted in L*; a test shows that `x/(y/y) ⊢ x` is not Proved in L and is Proved in L*; the words of Mode round-trip and old mode objects still read; the oracle and the checker agree on random planar proofs; the argument is in the report.
- State now: Not met. Evidence: core/src/proofs/check.rs has Problem::Succedents for R1 to R3 only; core/src/fragment.rs:221-229 Mode has three bools; R55, R134 and R177 refuse the empty antecedent at input only; no module for the ordered modes exists.

### R254. from_proof refuses a node kind it does not read
- Steps: 34 cut (from session 1 until session 3, R80); 33 MELL nets (R122).
- Sources: plan/notes/research/35-mll-engines.md (section 3, Proof term and checker); plan/notes/research/README.md (section 2, from_proof and the net's flag); plan/34-cut.md:19-26.
- API item: ProofStructure::from_proof (core/src/nets/mod.rs).
- Requirement: from_proof matches every Node variant explicitly, with no wildcard arm, so that a variant a step adds is a compile error there. A node kind it does not read yet makes it return an error that names the kind, never a net built from the Ax nodes alone. This is Cut from the moment session 1 of step 34 adds it until R80 lands: the Ax nodes of a proof with a cut give a structure whose cut roots are extra, unlinked conclusions, which with Mix may pass is_correct as a net of a different sequent. The exponential nodes and Weaken are covered by the same explicit arms until R122, even though the fragment check of ProofStructure::new already refuses them. R80 and R122 then replace the refusing arms by arms that read the nodes.
- Met when: from_proof has no `_ =>` arm; a test builds a proof with a cut (the builder of R202) and shows that from_proof is an error with and without Mix, until R80 turns it into the test of the cut net.
- State now: Not met, and no wrong net arises today. Evidence: core/src/nets/mod.rs:249-261 filter_map with `_ => None` over Node::Ax; ProofStructure::new refuses fragments beyond unit-free MLL (mod.rs:203-206); Node (core/src/proofs/mod.rs:117-155) has no Cut yet.

## Errors

### R129. A wire form for every error and refusal a front end shows
- Steps: 32 web client; 33 MELL nets; 34 cut; D15, D22.
- Sources: plan/32-web.md:31-33,66-69; plan/33-mell-nets.md:18-21 and plan/32-web.md:12-14; plan/later.md:28,124-128; plan/later.md:696-707; plan/later.md:957-958,1141-1142,1175-1176; plan/README.md:685-700,752-778.
- API item: Error, ParseError, CheckError and Problem, ViewError, Refusal, NetError, ShapeError, WriteError, Unsupported.
- Requirement: Every error and refusal a front end shows has a machine-readable JSON form generated from one place, so that adding a variant needs one arm: a stable tag per variant, its fields (ids, positions, spans, sizes, limits, the bound that was exceeded) and the English message, so that a web client can localise (the stated reason for Refusal's structure) and highlight the failing character without parsing prose. The form distinguishes a refusal (a bound reached, no verdict; CheckError::is_refusal, a memory refusal) from a fault of the input. ParseError needs offsets in a unit a JavaScript editor can use: the byte span is documented, and a helper gives characters (or UTF-16 units), or line and column, which the command computes itself today. The composed explanations that the command builds (graft too large, stopped, unknown-reason advice) come from the library, from Display or structured fields, and are not rewritten in the client. NetError's witnesses (cycle, parts, and the new box errors) either join the shared wire form or the decision to leave it to another step is recorded. Error and its sub-enums are also non_exhaustive (R50).
- Met when: Serialize for each type, pinned in core/tests/serialize.rs; a test that every Error variant serializes (a match without wildcard); a ParseError::line_column(&str) or character span, with a test that a non-ASCII input yields a span the client can map; a test that is_refusal survives the round trip as a distinct tag; the explanation strings built by cli/src/interact.rs come from the library.
- State now: Not met. Evidence: core/src/errors/mod.rs:14-20 `#[derive(Error, Debug)] pub enum Error` has no serde and no non_exhaustive (thiserror only); errors/parse.rs:9-20 ParseError {span: Range<usize> in bytes, found, label, expected} derives Debug only; Refusal (interactive.rs:36-108), ViewError (derivation.rs:440) and NetError (nets/mod.rs:46) have no serde; grep -rln serde core/src lists no errors/ file; cli/src/interact.rs:357-408 and :499 compose the user messages; cli/src/lib.rs computes the caret line itself; plan/28:70-77 notes that the family is scattered.

### R130. No public call panics on input of the right type
- Steps: 32 web client.
- Sources: plan/32-web.md:30-36,43-47; plan/later.md:687-708.
- API item: all public entry points, the deserializers, the parser, Interactive::apply, the exports.
- Requirement: On wasm a panic is a trap that destroys the instance and any state in it. No public call panics on input of the right type, however malformed the text or JSON (shared via URL, typed, or a pasted proof file). Deserializers, the parser, Interactive::apply and the exports return errors. Internal panics (the OccSet union beyond width, expect or unreachable in Interactive) are unreachable from the API, or the audit lists the kept expect sites with their reason.
- Met when: Fuzz or property tests over parse, the Sequent, Proof, Interactive and ProofStructure deserializers and Interactive::apply assert Err and not a panic; the bindings install a panic hook that returns an error JSON; the audit's list of kept expect sites exists.
- State now: Unknown. Evidence: core-forest.md:60-70 says OccSet union "panics rather than lose the member"; interactive.rs:760 has `unreachable!`; core/tests/depth.rs covers deep formulas only and the checker has hostile-input tests; plan/28-audit-and-refactor.md:311 lists untrusted-input handling in the audit but the result is not in the code yet; batch.rs resumes worker panics (core-batch.md:28-29).

### R131. Errors for the cut rule and for elimination
- Steps: 34 cut (sessions 2 and 3).
- Sources: plan/34-cut.md:19-21,30.
- API item: Error variants, CheckError and Problem, NetError for cut.
- Requirement: Elimination and the cut rule add errors: a cut formula that does not parse, a pair that is not dual, a cut on a proof that is invalid, the elimination limits, and a net with a cut link that is wrong. Each needs a Display that says what is wrong and, where an occurrence is named, a describe(&forest) form that prints formulas. (The wire form is R129.)
- Met when: New Error, NetError and Problem variants with documentation and Display tests; CheckError::describe prints a cut problem with formulas (a test like the existing `node 2 (⊗ on A ⊗ ~B ...)`).
- State now: Not met. Evidence: core/src/errors/mod.rs:15-232 has no cut variants; core/src/nets/mod.rs:46-87 NetError; check.rs:463 Problem.

### R132. Box failures have their own NetError variants with witnesses
- Steps: 33 MELL nets.
- Sources: plan/33-mell-nets.md:18-19; plan/later.md:189-197.
- API item: nets::NetError (new variants), Error::InvalidNet.
- Requirement: Box failures need their own NetError variants with witnesses (a box whose interior is not a net, a switching cycle at a given depth, a door outside its box, an unplaced weakening), each printing ids and, via describe(&forest), formulas. NetError is not #[non_exhaustive], so the decision whether it becomes so is made before the release, and recorded.
- Met when: Variants with documentation comments and tests of the messages; NetError marked #[non_exhaustive], or the reason it is not recorded in core-nets.md.
- State now: Partly. Evidence: core/src/nets/mod.rs:45-72 NetError has eight variants and no #[non_exhaustive]; grep non_exhaustive finds none in nets/mod.rs or errors/mod.rs (it is on ViewError and the search types).

### R133. One predicate says which fragment has nets, used by the library and the command
- Steps: 33 MELL nets.
- Sources: plan/33-mell-nets.md:16; plan/README.md:568-581 (D6a: nets exist for MLL, MELL nets with boxes come later); plan/reports/17-assessment.md 3.4.
- API item: ProofStructure::new fragment check, Error::NetFragment, fragment::Fragment (MELL is MLL with units plus exponentials), cli nets_exist.
- Requirement: State which fragment has nets (unit-free MELL, or MELL with unit nodes or jumps; weakening makes the correctness of nets with ?w and ⊥ delicate) in one place used by both the library and the command, and refuse the rest with Error::NetFragment whose message names the fragment. Keep room for unit nodes if included. The refusal text and exit status are pinned by README examples.
- Met when: One shared predicate (a constant or function on Fragment) used by ProofStructure::new and nets_exist; tests for accepted and refused fragments; README and --help text updated in the same commit.
- State now: Partly. Evidence: core/src/nets/mod.rs:204 hard-codes `Fragment::MLL.contains`; cli/src/prove.rs:949-959 nets_exist duplicates it; errors/mod.rs:118-120 the message says "MLL without units only".

### R134. Named errors for the ordered calculus
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-19.
- API item: Decide::admits for Nets; Error::NetFragment, NetMode, NetGoal and new variants.
- Requirement: Refusals for the new calculus have named errors with an English message each: a fragment beyond MLL in an ordered mode (units, additives, exponentials; the Lambek calculus of the step has only the two divisions and the product), an empty antecedent in the Lambek mode, Mix or affine combined with an ordered mode, and a non-unit-free sequent. The existing errors keep their form; new variants are additive to an exhaustive enum, so the set is decided before the 0.1.0 release.
- Met when: One test per refusal asserting the variant and its message; the command maps them to exit status 2.
- State now: Partly. Evidence: core/src/search/net.rs:34-50 admits() refuses a fragment beyond MLL, affine mode and non-root goals with Error::NetFragment, NetMode and NetGoal; core/src/errors/mod.rs has EngineMode and IntuitionisticMix; no empty-antecedent variant; Error is not #[non_exhaustive].

### R135. Each engine refuses what it cannot decide with a typed error that the harness classifies as refused
- Steps: 35 essential nets; 37 inverse method; 38 first-order.
- Sources: plan/35-mll-engines.md:35-41; plan/37-inverse.md:23-26; plan/38-first-order.md:18-19,26-29; plan/later.md:310-318; plan/notes/research/impact-quantifiers.md (section 1, finding 1); plan/notes/research/impact-fo-ordinary.md (section 3).
- API item: errors::Error, nets::NetError, Decide::admits, Fragment (quantifier bit), bench run.rs refused classification.
- Requirement: The essential-net engine refuses classical and affine mode, a fragment beyond IMLL, units, and a goal other than the roots. The inverse engine states exactly which goals it admits (classical and intuitionistic through the Reading, linear and affine, with and without Mix, units and exponentials, roots or any goal prove_goal accepts) in a table in the Engine::Inverse rustdoc, and does not panic or return a wrong verdict on a mode it lacks (Mix is one of the two cases it is meant for). Every engine's Decide::admits checks the fragment of the goal, not only the mode (the focused engines) or the shape (Horn). An engine without a first-order version (additive, Horn, and the net engine until it has unification) refuses a goal with the first-order bit, whether it has a binder or only predicate arguments, with a typed error naming the engine, and never searches it as atoms, so a forced Options::engine on a first-order goal fails with that error rather than a wrong verdict. The dispatch returns a named error (Error::NoEngine or a new variant) when no row takes a goal, and never panics. Until the axiom rule compares instances, Proof::check and the oracle refuse a forest with arguments, and the commit of step 38 that adds the data model lands that refusal first. The errors are existing ones (NetFragment, NetMode, NetGoal) or new Error variants that an API user can match, the harness classifies as refused and not error, and the command describes. The bench classifier lists the refusing variants by name (bench/src/run.rs:634-641). Error and NetError are non_exhaustive before 0.1.0 (R50).
- Met when: New variants added to the bench refused list; one test per refusal; tests that force the engines across modes via the configurations helper in search/reference.rs; a test per engine that a first-order sequent is refused or decided as the dispatch table states; the rustdoc table of Engine updated; a test that a sequent with arguments and no binder is refused by every engine forced through Options::engine, and by the checker until unification exists; a dispatch test in which no row takes the goal returns the error.
- State now: Not met. Evidence: core/src/errors/mod.rs:20 `pub enum Error` has the refusals NetFragment, NetMode, EngineMode {engine, mode}, NotAdditive, NotHorn and NetGoal (a pattern for first-order) but no #[non_exhaustive]; core/src/search/net.rs:50-62 Nets::admits gives NetFragment, NetMode and NetGoal; Decide::admits(&self, &Task) -> Result<(), Error> (search/mod.rs about line 466); there is no quantifier bit or row yet. Focused::admits (focus/mod.rs:133-148) checks the mode only and Horn::admits (horn/mod.rs:38-43) the shape only; dispatch ends in .expect (search/mod.rs:495); Error::NoEngine (errors/mod.rs:195) is never constructed.

### R136. Typed errors for first-order input and use
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:15-22.
- API item: errors::Error (new variants), ParseError.
- Requirement: First-order input and use bring new failures that need typed errors with a message and a wire form: unknown or inconsistent symbol arity, a free variable where a closed formula is required, an unscoped bound variable in JSON, an exceeded term or instance bound at construction, and a first-order goal given to an engine that cannot take it (R135). ParseError reports spans in the new syntax.
- Met when: New Error variants (with Error made #[non_exhaustive]) with messages and tests; the command maps them to exit status 2.
- State now: Not met. Evidence: core/src/errors/mod.rs: Error has InvalidVariableIndex, TermIndexOutOfBounds, SubtermIndexNotDecreasing, TooManyOccurrences, SequentParsing, NotAdditive, NotHorn, EngineMode and others, and no term- or binder-related variant; the enum is not non_exhaustive.

### R137. What the command prints for an inverse run is right for that engine
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:17-18,26-29.
- API item: cli/src/prove.rs `unknown` (copy-bound text), Refutation, Verdict::Unprovable.
- Requirement: An unknown from the inverse engine must not mention a copy bound (it has none), and the unprovable line says the saturation was exhaustive. The exit status stays 0, 1, 2 or 3 by verdict. A saturation that completes returns Ok(None) and prove_goal then adds the counts' refutation or Exhausted; no new Refutation is needed unless the engine can name its own.
- Met when: A README console block or CLI test for `prove --engine inverse` on a provable, an unprovable and a non-terminating (time-limited) input; cli/tests/readme.rs runs it.
- State now: Partly. Evidence: cli/src/prove.rs:1160-1163 `deepened = fragment.has_exponentials() && matches!(engine, Focus | TwoSided)` so another engine gets no copy-bound text already; Refutation is non_exhaustive (search/mod.rs:1258) and prove_goal computes the counts' refutation for any engine that gives none (core-search.md); no inverse-specific wording exists.

### R138. What an "unknown" tells the user is exact
- Steps: 28 audit; 22.
- Sources: plan/later.md:1022,941-946; plan/README.md:626-629 (D9); plan/notes/research/30-release.md (section 3, Engine interface and dispatch).
- API item: search::Reason (a new variant for a proof found but not checked), Error::Unchecked, the exit status of `linlog prove`.
- Requirement: A memory-starved search ends early as a memory reason rather than running slow; the reason of the default bias's pair is the one a user should read (not always the backward search's); and a proof the engine found but the checker could not hold within the memory bound is not an error with exit status 2. It ends the search as Verdict::Unknown with a Reason of its own, which says that a proof was found and could not be checked, and the command exits 3. Verdict keeps the three values of D9, and a Proved is always a checked proof (search/mod.rs:118). Error::Unchecked remains for a step of the interactive session, which is not a search.
- Met when: Reason and Outcome are documented and tested for the three cases, with a test that a memory bound too tight for the check yields Unknown with that reason (not Err and not Proved) and the command's exit status 3; the verdict list of D9 is unchanged.
- State now: Partly. Evidence: search/mod.rs Reason has MemoryLimit(u64) (1417-1437); errors/mod.rs:107-111 Error::Unchecked is an Err, so the command exits 2 (cli/src/prove.rs:776; as .claude/rules/core-horn.md on tight bounds states); the pair's reason selection is in focus/schedule.rs `reason`, not pinned by a test found.

## Options

### R139. The Rocq export chooses its kernel through an option, and NanoYalla stays the default
- Steps: 31 Rocq library (items 6 and 2); 34 cut.
- Sources: plan/31-rocq-library.md:48; plan/later.md:23,40; plan/later.md:380-386,412-414; plan/README.md:740-743.
- API item: export::rocq::Options (new field kernel; new enum Kernel with Kernel::for_mode(Mode)), export::rocq::Unsupported, NANOYALLA.
- Requirement: A second kernel sits behind rocq::Options (D15) as a serde-visible field, with a Kernel enum that is #[non_exhaustive] and has an Auto value. Auto picks the kernel from the mode when the user did not choose one (NanoYalla for classical without Mix or affine weakening, linlog's library otherwise). Options::default() and Auto for plain classical mode keep today's NanoYalla output byte for byte, and a forced NanoYalla still refuses Mix, affine weakening, open goals and compact derivations. Unsupported becomes a property of the kernel and is made #[non_exhaustive] before the first release, because a new kernel and a new refusal reason may arrive. Options keeps serde(default, deny_unknown_fields), so `--style rocq.kernel=...` and a JSON options value work for every front end (D15); the command reaches it only through the style key, since a new option is a field and never a flag (cli.md).
- Met when: A unit test that Options::default() output equals the existing snapshots ll.v, mll.v, mell.v, labels.v, mall.v and ill.v; a test that Kernel::for_mode returns the library kernel for Mix, affine and intuitionistic; a serde round-trip test for Options with kernel; cli/tests/readme.rs still passes unchanged; the certificates flake check runs for both kernels.
- State now: Not met. Evidence: core/src/export/rocq.rs:84-93 Options has form, lemma and prelude only; :107-127 Unsupported {Open, Mix, AffineWeakening, Compact} with messages naming NanoYalla and no #[non_exhaustive] (the non_exhaustive at core/src/export/mod.rs:78 is RenderError); :78 the NANOYALLA constant; rocq.rs:4-12 binds the export to NanoYalla's macroll.

### R140. The standalone prelude defaults per kernel
- Steps: 31 Rocq library (items 6 and 2).
- Sources: plan/31-rocq-library.md:48-49.
- API item: export::rocq::Options::prelude (a String whose default is NanoYalla's import line).
- Requirement: A user who switches the kernel but not the prelude would otherwise get NanoYalla's `From NanoYalla Require Import macroll.` in front of a library certificate. So prelude becomes an Option<String> (None means the chosen kernel's own import lines), or the kernel carries its own default. The command's --prelude still sets it explicitly.
- Met when: A test that Kernel::Linlog with the default prelude writes `From Linlog Require ...` (or whatever the library names it) and that an explicit prelude overrides it; the README --prelude text and the --help documentation comment in cli/src/argument_parsing.rs:917 are updated.
- State now: Not met. Evidence: core/src/export/rocq.rs:95-103 Default sets prelude to the constant "From NanoYalla Require Import macroll."; cli/src/style.rs:84 assigns styles.rocq.prelude directly.

### R141. The aggregate of every output's options belongs in the library
- Steps: 32 web client; a third wrapper (D15).
- Sources: plan/32-web.md:31-33; plan/reports/22-configurable-output.md:461; plan/later.md:131-137; plan/README.md:685-700.
- API item: new linlog::export::Styles (the aggregate now in cli/src/style.rs), cli/src/style.rs Styles (text, latex, typst, svg, png, pdf, rocq).
- Requirement: The value that holds one options value per output format (text, LaTeX, Typst, SVG, PNG, PDF, Rocq), with its JSON form (serde, Default, PartialEq, deny_unknown_fields), its dotted-key set and its `--style KEY=VALUE` semantics, lives in core, so that the web client, a notebook, an editor plugin and the command read and write the same JSON ("the same JSON" is the stated design) with the same settings surface as `--style-file`, instead of the web crate re-deriving the aggregate. The aggregate is #[non_exhaustive] from the start (R50), because a later output's options are additive only if the type is.
- Met when: linlog::export::Styles exists (behind serialize); the command's Styles::read is reduced to flag parsing and its --style-file reads exactly it; a core test pins the JSON of the defaults.
- State now: Not met. Evidence: cli/src/style.rs:20-40 defines `pub struct Styles { text, latex, typst, svg, png, pdf, rocq }` in the command crate; the member types each have serde(default, deny_unknown_fields) in core (proofs/fmt.rs:19-20 TextOptions, export/latex.rs:213-215, typst.rs:128,144, svg/mod.rs:59-60, png.rs:20, pdf.rs:34,94, rocq.rs:82-83; core-export.md:36-41).

### R142. Named presets of the options types for a different environment
- Steps: 32 web client (D15).
- Sources: plan/32-web.md:50-52,58-60; plan/README.md:685-700.
- API item: search::Options, ViewOptions and the export options (named presets); DEFAULT_MEMORY_LIMIT, DEFAULT_OCCURRENCE_LIMIT, DEFAULT_MEMO_LIMIT, DEFAULT_RECURSION_LIMIT, ViewOptions::DEFAULT_LIMIT, check_within memory.
- Requirement: Presets are constants or constructors of the options types, never code paths. Besides the SVG Style::dark() and monospace(), there are presets a front end can pick for a different environment: defaults that fit a browser tab (memory and occurrence bounds, memo size, recursion limit for a 1 MiB stack, derivation limit, work budget) as one named preset or one bounds value (possibly a Limits value with serde) that a front end sets in one call, not four setters on three types. The library's gibibyte stays the native default. A wrapper names a preset instead of copying numbers.
- Met when: A constructor such as Options::browser() plus a ViewOptions equivalent exists, is documented with the reason for each number and the tab budget they sum to, a test checks that a preset is a value of the plain type, and the bindings' defaults use it.
- State now: Partly. Evidence: core/src/export/svg/mod.rs:159 dark and :173 monospace exist; search/mod.rs:924-956 constants exist and each is settable (memory_limit, occurrence_limit, memo_limit, recursion_limit); the memory default is 1 GiB, the memo 2^20 and ViewOptions::DEFAULT_LIMIT 64 MiB (proofs/derivation.rs:395); plan/28:56-58 notes that the bounds of a search, a check and a view are three values set one by one. There is no preset on search::Options or proofs::ViewOptions.

### R143. Each pruning rule of the net engine can be switched off from the public API
- Steps: 35 net engine pruning and routing measurement.
- Sources: plan/35-mll-engines.md:24-34.
- API item: search::Options (a new switch for the net engine's pruning rules).
- Requirement: The harness is a separate crate and can only use the public API, so ablating each pruning rule (to show that a rule earns its place, and to run the differential review against the unpruned engine) needs a documented Options setter, or a bench-only feature, that switches each rule off. The engine's documentation says which options it reads, and the choice needs no wire form beyond the command flag and the CSV column if it is public.
- Met when: An Options setter (or crate feature), a bench column, and a test that each rule off still agrees on the verdict.
- State now: Not met. Evidence: core/src/search/mod.rs Options fields (about lines 870-920) have test_period for the net engine and nothing for pruning; Options has no serde impl.

### R144. The drawing of boxes and of essential nets is configured through svg::Style fields
- Steps: 33 MELL nets; 35 essential nets (D15, D16).
- Sources: plan/33-mell-nets.md:20-21; plan/README.md D15, D16; plan/later.md:189-197,199-210.
- API item: svg::Style (new box fields, polarisation and dominator-arrow fields), PNG and PDF options.
- Requirement: Every choice in the box drawing (padding, stroke, dash, fill, colour, nesting inset, whether doors are marked), and in the essential-net drawing (polarisation and dominator arrows), is a field of Style with a default, with serde behind serialize, never a constant, so that the command flags and the web settings JSON can set it. Style is deny_unknown_fields with default, so the new fields are added with defaults that leave today's net snapshots byte-identical, with the same limit refusal (TooLarge) and the stop-free bound as today. The command reaches them through `--style svg.<field>`. A new Style field is additive for Rust callers only because Style is #[non_exhaustive] (R50), which it is not today.
- Met when: New Style fields with documentation comments; a StyleArgs flag per field in cli/src/style.rs; a JSON Style test that old settings still deserialize; a snapshot test per new drawing.
- State now: Not met. Evidence: core/src/export/svg/mod.rs:61-157 Style has link and node fields but none for boxes or polarisation; the serde attribute `deny_unknown_fields, default` is at mod.rs:59-60; the existing net drawing knows no boxes (export has `net`, core-export.md "The entry points").

### R145. The modes in which MELL nets exist are stated and enforced
- Steps: 33 MELL nets (D6a).
- Sources: plan/33-mell-nets.md:15-19; plan/README.md D6a; plan/reports/09.
- API item: the criterion value of R79 (today ProofStructure::mix), Error::NetMode, the intuitionistic one-sided net.
- Requirement: State the modes in which MELL nets exist. Mix combines with boxes (the criterion at each depth, with or without the connectedness count), intuitionistic mode uses the one-sided sequent as for IMLL, and affine mode stays refused with Error::NetMode (weakening of any formula has no net node). Boxes add no criterion choice of their own: the structure is checked against the one criterion value of R79, and for MELL its only choice is Mix on or off, applied at each depth.
- Met when: Tests for a boxed net with and without Mix and with intuitionistic input, and NetMode for affine, documented in core-nets.md.
- State now: Partly. Evidence: nets/mod.rs:185-195 `mix: bool`; errors/mod.rs:121-123 NetMode; cli/src/prove.rs:949-952 refuses affine; the interaction of boxes with Mix is not specified anywhere.

### R146. The planar engine reads the same options and returns the same outcome
- Steps: 36 Lambek (D19).
- Sources: plan/36-lambek.md:17-19.
- API item: search::Options and Outcome (engine, fragment, bounds, Statistics).
- Requirement: The planar engine reads the same Options (memo_limit, recursion_limit, stop condition, jobs, test_period, memory limit) and returns the same Outcome, Verdict, Reason and Statistics, with no new required option. Any ordering-specific counter (crossings pruned) is an addition to Statistics and its JSON form (R9). The stop condition is polled in the planar search, which keeps no clock. The result is a Proved with a Proof and the net (Outcome.net) in the ordered form, or an exhaustive Unprovable, because the logic is NP-complete and the search terminates.
- Met when: A test with an interrupting stop closure on a hard cyclic family returns Reason::Stopped; the Outcome JSON for an ordered proof is read back by check; `cargo hack check --each-feature -p linlog` passes.
- State now: Partly. Evidence: core/src/search/net.rs:34-60 Nets implements Decide with `stop: &mut dyn FnMut() -> bool` and the Options; Statistics at core/src/search/mod.rs:1462 has the exact-test counter (tests); there are no ordered-mode counters.

### R147. Every Options setter and the engine variant say which options the inverse engine reads
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:17-18,27-29.
- API item: Options (copies, forward_copies, memo_limit, recursion_limit, bias, jobs, pool) and the per-variant "options it reads" documentation.
- Requirement: The documentation of every Options setter and of the Inverse variant says which options the inverse engine reads and which it ignores (it has no copy bound, no recursion and no atom bias; it reads memory_limit, check, occurrence_limit and probably memo_limit as its database cap). Any new knob (subsumption on or off, database size) follows the existing pattern: private field, setter, DEFAULT_ constant, command flag, harness flag and CSV column, not an ad-hoc field.
- Met when: Engine::Inverse rustdoc states the options it reads; each new setter has the constant and the flag; Options PartialEq and Debug cover the field.
- State now: Partly. Evidence: core/src/search/mod.rs:915-945 Options has private fields with setters; each Engine variant documents the options it reads (l.666-730) and an unread option is "documented there, never refused" (core-search.md); the harness side of a new knob is listed in .claude/rules/bench.md "A configuration axis" (RunArgs flag, OneArgs field, tail column, `finished` key, summary label).

### R148. The default time limit is a named constant of the options value
- Steps: 32 web client (D16).
- Sources: plan/README.md:702-710.
- API item: new Options::DEFAULT_TIME_LIMIT (a Duration as data, no clock).
- Requirement: The default time limit of a call (2 s in the command) is a default of D16 but lives in the command, so the web bindings and the harness cannot share it. The library holds it as a named constant of the options value (a Duration is data, not a clock), with the front ends applying it through their own stop.
- Met when: A constant on the options value, used by cli/src/argument_parsing.rs and the bindings, with its reason in the documentation.
- State now: Not met. Evidence: cli/src/argument_parsing.rs:389 `pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2)`; there is no time constant in core/src/search/mod.rs.

### R149. Hidden constants that a user might want to change are options or named follow-ups
- Steps: 28 audit; every wrapper (D15, D16).
- Sources: plan/README.md:685-710.
- API item: search::NET_MULTIPLICITY, focus::schedule::POLL, Options::stack_size PER_LEVEL, Options::test_period with the private SMALL and PERIOD of search/net.rs.
- Requirement: A hidden constant a user might want to change is an option, or is named as a follow-up. The dispatch threshold of the net engine (no literal more than twice) is a private constant that decides which engine runs, and a front end or the harness cannot move it. The audit decides each such constant and records the rest. The same holds for the options already public: every setter that sets a value a user may want to change has a flag and a documentation line (D16). Options::test_period, the number of links the net engine places between two exact acyclicity tests, has no flag in the command, and its default (every link up to 200 occurrences, every fourth above) is two private constants, not named defaults of the options value. The pool handle (Options::pool) is built from --jobs and --pool-after and needs no flag of its own.
- Met when: Each constant is either an Options field with a DEFAULT_ constant, a command flag and a documentation line (D16), or is listed in plan/later.md with the reason it stays fixed; `linlog prove` has a `--test-period` flag (with --help text) mapped onto Options::test_period, and its default values are named constants of the options value; the audit's walkthrough in cli.md lists every Options setter against its flag.
- State now: Partly. Evidence: core/src/search/mod.rs:625 `const NET_MULTIPLICITY: usize = 2` is private and used by few_equal_literals in DISPATCH; core/src/search/focus/schedule.rs:280 POLL (parallel only); Options::DEFAULT_* constants exist for memo, recursion, copies, forward copies, check, memory and occurrence (search/mod.rs:924-956). Options::test_period (core/src/search/mod.rs:1082) has no flag under cli/src, though bench/src/main.rs:174 has one; SMALL and PERIOD are private (core/src/search/net.rs:34,38); every other setter has a flag (cli/src/prove.rs:1006-1016).

### R150. The batch structs can grow fields without breaking callers
- Steps: batch mode (done in step 24); the release.
- Sources: plan/later.md:539-558.
- API item: batch::Options, batch::Plan, batch::Problem, batch::Answer.
- Requirement: The batch structs have public fields and no #[non_exhaustive], so every later field (a per-problem copy bound, a second pass over unknowns, a stream flag, isolation) breaks struct-literal callers, unlike search::Options, which has private fields and setters. Before the first release they are made non_exhaustive or builder-shaped, as search::Options is.
- Met when: #[non_exhaustive] plus a constructor and setters on batch::Options, Plan, Problem and Answer, with the doc test using them.
- State now: Not met. Evidence: core/src/search/batch.rs:54-67 Options (all pub fields), :122 Plan, :142 Problem, :153 Answer: none is #[non_exhaustive]; the doc example builds Problem with a struct literal (batch.rs:18-22).

### R151. linlog on one thread is a first-class, reproducible configuration
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:41-43.
- API item: Options::jobs, cli --jobs and --deterministic, harness --jobs 1.
- Requirement: linlog on one thread is a first-class, reproducible configuration (the other tools are sequential), with the sequential engines and counters a function of the input, expressible as one flag in the command and the harness and recorded in the CSV row.
- Met when: `--deterministic` in the command and `--jobs 1` in the harness give equal verdicts and counters, and the jobs column records it.
- State now: Met. Evidence: cli/src/argument_parsing.rs --deterministic ("same as --jobs 1"); bench/src/main.rs RunArgs.jobs defaults to '1'; the CSV has a jobs column in HEADER.

## Export

### R152. A Rocq writer that takes a Proof and a Mode and keeps the certificate linear
- Steps: 31 Rocq library (items 3 and 7); D20.
- Sources: plan/31-rocq-library.md:38; plan/later.md:386-397; plan/README.md:740-743.
- API item: new export::rocq::proof(&Proof, Mode, &Options, out, stop) -> Result<(), WriteError>.
- Requirement: The term kernel needs a writer that takes a Proof and a Mode directly, not a Derivation. A Derivation unfolds the proof DAG into a tree and is built under ViewOptions bounds, so a certificate would be exponential or refused (the user's tree also differs from the term in where ?c and ?w sit, and a compact view has no certificate). The writer emits the sequent, the mode, the forest (the table from occurrence id to negation-normal-form formula, with the parent and child structure) and the nodes of Proof::nodes() in arena order, premises before conclusions, as an indexed list that the checker reads in one pass. Sharing is kept, so the output is linear in the proof, never unfolding shared nodes, and a proof with an exponential unfolding still has a linear certificate. It follows the existing one-signature convention (any fmt::Write, a stop closure asked after each chunk or per node, WriteError). It builds its text per node and holds no more than one chunk plus tables linear in the forest. It reads no clock and uses no thread. The mode is an argument because Proof does not store it (or see R7). It is selected by the kernel option (R139), and the exporter's options keep one value (D15).
- Met when: A function with that signature in core/src/export/rocq.rs; a test that a proof with 2^20 shared nodes writes in linear space and that the text length is O(nodes + forest); a test that stop yields WriteError::Stopped; a snapshot test in core/tests/export.rs; a flake check compiling the output against the new library, with the NanoYalla snapshots unchanged; core-export.md documents it.
- State now: Not met. Evidence: core/src/export/rocq.rs:530-554 `derivation(&Derivation, &Options)` and `write(&Derivation, ...)` are the only entries and they refuse Mix, AffineWeakening, Open and Compact (:545-554); cli/src/prove.rs:253 forces Compact::Never for Rocq and :687 builds the derivation first; Proof::mode does not exist (proofs/mod.rs:299); Proof::nodes(), node() and forest() are public (proofs/mod.rs:372-399), so the data is reachable.

### R153. A writer for certified refutations
- Steps: 31 Rocq library (item 8 and the escalations).
- Sources: plan/31-rocq-library.md:73; plan/later.md:473-475.
- API item: new export::rocq::refutation(&Sequent, Mode, &Refutation, &Options, out, stop), export::rocq::Unsupported.
- Requirement: An entry point writes a refutation certificate. The lemma states that the library's calculus (an inductive) does not derive the sequent, never a negation over Prop for the linear logics. The certificate is the few numbers Refutation already holds (the atom with least and most, or the Equation counts with mix), and Rocq recomputes the intervals or counts by computation. It refuses with a new Unsupported variant for Exhausted, StateEquation and any mode or fragment where the search does not rely on the invariant (R70). Unsupported is #[non_exhaustive] so later refutations add variants without a break. A classical falsifying assignment that the writer certifies comes from the library's search (R20). The entry takes the sequent and the mode, since the Unprovable verdict and its JSON do not carry them. It uses the same one signature (out, stop, WriteError) and options value as the other writers.
- Met when: Snapshots of a refutation per kind in core/tests/snapshots, compiled by the flake rocq check; a test that Exhausted yields the new Unsupported variant; Print Assumptions of the library's refutation lemmas shows none.
- State now: Not met. Evidence: export/rocq.rs has only derivation, write and ordinary; Unsupported at rocq.rs:108 has no #[non_exhaustive]; serialize/search.rs:195-217 the unprovable outcome JSON has no sequent; grep -n efutation core/src/export/rocq.rs is empty.
- Conflict: see Conflicts for the author, C2.

### R154. A certificate that an ordinary sequent is not valid, for classical logic only
- Steps: 31 Rocq library (item 8, the classical "not valid" of ordinary logic).
- Sources: plan/31-rocq-library.md:87; plan/later.md:473-475.
- API item: export::rocq::ordinary, ordinary::Sequent, Image::ordinary(), an evaluator for ordinary::Formulas.
- Requirement: A writer for `~ (forall a b : Prop, F)` takes the ordinary sequent (Image::ordinary() or ordinary::Sequent) and an assignment, with an evaluator for ordinary::Formulas over the assignment (the assignment search of R20 takes a bound option and a stop). It applies to classical logic only. Intuitionistic and minimal "not valid" yield an explicit no-certificate error, since no Kripke countermodel exists here (R72). The positive certificate writers keep working unchanged.
- Met when: A snapshot compiled by the flake rocq check for a classical non-theorem; a test that `--logic intuitionistic` not-valid reports no certificate; the assignment evaluator is tested against the ordinary checker on the existing LK corpus.
- State now: Not met. Evidence: core/src/export/rocq.rs:617 `ordinary` takes only a Derivation; core/src/ordinary/rocq.rs writes positive terms only; ordinary::Formulas::node and atom_names (ordinary/mod.rs:290-310) are public, so the evaluator is writable.
- Conflict: see Conflicts for the author, C2.

### R155. Atom-name escaping and the reserved names are per kernel
- Steps: 31 Rocq library (items 2 and 6).
- Sources: plan/31-rocq-library.md:48-49; plan/later.md:455-457 (identifier escaping writes non-ASCII as code points where Rocq accepts many Unicode letters).
- API item: export::rocq::identifiers (pub(crate)) and RESERVED.
- Requirement: Atom-name escaping is shared by the NanoYalla script, the ordinary certificate and the library kernel. RESERVED today is NanoYalla's keyword and kernel-name list, so the library kernel needs its own reserved names (its constructors, checker names and the lemma name); the reserved set becomes per kernel and identifiers takes it as an argument, with the lemma name still excluded. Existing outputs (the test names_become_identifiers and the snapshots) do not change. The O(n^2) `names.contains` in identifiers becomes a set, as a risk for sequents with very many atoms.
- Met when: names_become_identifiers still passes; a new test with a clash against a library constructor name; atom escaping covers the new kernel in the snapshots.
- State now: Partly. Evidence: core/src/export/rocq.rs:123-215 RESERVED is a single NanoYalla-only constant; :228-236 identifiers(atoms, lemma) with names.contains at :232; it is pub(crate) and also used by ordinary/rocq.rs.

### R156. The statement printer for the new kernel is one walk over Walk stops
- Steps: 31 Rocq library (the new kernel).
- Sources: plan/later.md:379-386,419-421.
- API item: a new sequents::fmt::Walk-based Rocq statement printer.
- Requirement: The lemma a user reads states the sequent over the plain inductive (negation normal form formulas over atoms with decidable equality), which needs a formula printer for the new constructors, with atoms as indices or escaped names. By the crate rule it is one match over sequents::fmt::Walk, with no recursion, so that a formula nested 100 000 deep prints on a 256 KiB stack. Atoms may be emitted as numbers with the name in a comment, which avoids the Unicode-identifier limits of identifiers.
- Met when: The printer is a Walk loop and core/tests/depth.rs gains a case for it; a snapshot pins the statement for classical, Mix, affine and two-sided inputs.
- State now: Not met. Evidence: core/src/export/rocq.rs imports Visit and Walk (line 8) for the NanoYalla printer only; .claude/rules/core.md:111-125 states the no-recursion rule and core/tests/depth.rs runs every public walk.

### R157. An unfinished proof can be exported with its open goals as hypotheses
- Steps: 31 Rocq library (optional, cheap now).
- Sources: plan/later.md:370-377.
- API item: rocq::write with open goals (Rule::Open).
- Requirement: Optional but cheap now: an unfinished proof exported with its open goals as hypotheses of the lemma (`Goal H1 -> H2 -> conclusion`), as Click & coLLecT writes it. If wanted, the NanoYalla writer accepts a derivation containing Rule::Open and names each open goal's sequent as a hypothesis; otherwise Unsupported::Open stays and is documented as final.
- Met when: A snapshot of an Interactive::derivation() export with open goals compiling in the rocq flake check, or a documentation line stating that the refusal is intentional.
- State now: Not met. Evidence: core/src/export/rocq.rs:548 `Rule::Open => return Err(Unsupported::Open.into())`; plan/reports/09-interactive.md:292-300 notes that the Rocq certificate should refuse or admit.

### R158. A further proof-assistant target is one more exporter of the same shape
- Steps: after 31 (a Lean target).
- Sources: plan/later.md:23,40.
- API item: the export module shape write(&derivation, &options, out, stop), cli Format, the style key.
- Requirement: A Lean target is one more exporter of the same shape and needs nothing else from the derivation view. It takes a Derivation, a plain-data options value with serde, any fmt::Write and a stop, answers WriteError, and the command adds a Format variant and a Styles key. The derivation view carries no NanoYalla-specific data.
- Met when: The documented one-signature rule in core-export.md and the extension point of cli.md ("An output format (lean, say)"), with the generic Drawn trait for layout-based exporters.
- State now: Met. Evidence: .claude/rules/core-export.md "One signature writes a derivation: latex::write, typst::write, svg::write, rocq::write ... answer WriteError"; .claude/rules/cli.md "Extension points: An output format".

### R159. The NanoYalla certificates stay byte-identical while the library is written
- Steps: 31 Rocq library (D20).
- Sources: plan/README.md:740-743; plan/31-rocq-library.md (goal: the NanoYalla export stays exactly as it is).
- API item: export::rocq (derivation, write, Options, NANOYALLA) and the flake rocq check.
- Requirement: While the library is written, the NanoYalla certificates stay byte-identical and are checked by the flake rocq check, so the new library is added beside it and not instead.
- Met when: The snapshots of the certificate in core/tests/export.rs and `nix build .#checks.x86_64-linux.rocq` are unchanged after the step.
- State now: Met. Evidence: core/src/export/rocq.rs:70 NANOYALLA = "1.1.3", Options at :84; core/tests/snapshots hold the exports; CLAUDE.md lists the rocq flake check (NanoYalla checks the certificates); modules/rocq.nix exists.

### R160. The Rocq export either certifies a Cut or refuses it with a named reason
- Steps: 34 cut.
- Sources: plan/34-cut.md:9-12.
- API item: export::rocq (the derivation certificate) and the Rocq library.
- Requirement: The Rocq library of step 31 gains cut as a case of its datatype, checker and soundness proof. For the exporter, rocq::derivation either emits the kernel's cut rule for a Cut inference or refuses it with a new Unsupported variant, and the proof JSON and term order of the library's datatype is the same as the library's, so the cut case is one added constructor. The cut-free and the eliminated versions of one proof both certify.
- Met when: A test that rocq::derivation on a derivation with a cut returns the script or Err(Unsupported::Cut); the rocq flake check compiles a certificate with a cut if the kernel has a cut rule (or the report says it has none).
- State now: Not met. Evidence: core/src/export/rocq.rs:548-555 refuses Open, Mix and AffineWeakening with Unsupported, and :448-469 lists all other rules explicitly; whether NanoYalla's macroll has a cut lemma is unknown (the kernel source is the flake input nanoyalla, not in the tree); rocq/ does not exist yet.

### R161. The exporter refuses a first-order proof by name until the first-order development exists
- Steps: 38 first-order; 31 Rocq library.
- Sources: plan/38-first-order.md:21-22; plan/later.md:318-324.
- API item: export::rocq (NanoYalla certificates), the future linlog Rocq library, the Proof wire form.
- Requirement: The NanoYalla exporter is propositional, so on a first-order proof it refuses with a named WriteError or Error (for example Unsupported::FirstOrder) rather than emitting a wrong certificate. The library's own proof term for first-order logic (witnesses, eigenvariables, a term table, closed under substitution, a checker expressible as a total function) is fixed so that the Rocq development can mirror it and a certificate is the sequent, the term and `check ... = true`.
- Met when: An exporter test that a quantified proof is refused with a clear message, and the first-order proof form is documented once (the rustdoc on Proof) as the contract the Rocq checker implements.
- State now: Not met. Evidence: core/src/export/rocq.rs targets NanoYalla (plan/later.md: "NanoYalla is propositional, so certificates wait for the Rocq library of linlog's own"); no quantifier handling in the exporter.

### R162. The Rocq library has its opam file, a draft at the release and final with the library
- Steps: 30 release (the draft); 31 Rocq library (the file in its final form).
- Sources: plan/30-baseline-release.md:39-40; plan/31-rocq-library.md:33-37; plan/README.md:740-743,767-772 (D20, D22: the archive's description says how the code was written; rocq-linlog is published once the library has a release); plan/notes/distribution.md (The Rocq library; Policies on AI assistance); plan/notes/research/30-release.md (section 4, Inconsistencies and open question 8); plan/notes/research/31-rocq.md (section 1, Rocq 9's ecosystem; section 2, The build).
- API item: the opam file rocq-linlog under rocq/ (new).
- Requirement: The file names the package rocq-linlog, the tag `logpath:Linlog`, the github.com/linlog-prover/linlog addresses (homepage, dev-repo, bug-reports) and bounds on both sides for rocq-core and rocq-stdlib only. It does not depend on NanoYalla or Yalla, which only the optional bridges import (R240), whatever prelude the NanoYalla export emits (R159 keeps that export unchanged). Its version equals export::rocq::LIBRARY (R12), which is how the library's own export stays in step with it. The archive's description (the file's synopsis or description, and the pull request to the archive) says how the code was written, with the AI-assistance statement that R218, R222 and the README carry. Step 30 prepares the file as a draft that `opam lint` accepts, since the library and the release archive with its sha512 checksum do not exist yet; step 31 completes it, and its publication to the archive waits for a release of the library (D22) and is the author's act.
- Met when: At step 30 `opam lint` passes on the draft, and the draft carries the addresses and the AI-assistance statement in the words of the README's; at step 31 the dependencies are exactly rocq-core and rocq-stdlib, the version equals LIBRARY (the check of R12), and the file passes `opam lint --check-upstream` once an archive exists; the rocq flake check still checks the NanoYalla certificates with the pinned NanoYalla.
- State now: Not met. Evidence: ls rocq: no such directory; core/src/export/rocq.rs:102 the prelude is `From NanoYalla Require Import macroll.`; modules/rocq.nix checks certificates with NanoYalla.

### R163. Click targets in the SVG map back to the state, and the calls stay pure
- Steps: 32 web client.
- Sources: plan/32-web.md:66-69; plan/reports/11-svg.md:239,254-278; plan/later.md:700-708.
- API item: export::svg::Style::ids, svg::derivation, svg::two_sided, svg::net, svg::sequent, Interactive::derivation_ids.
- Requirement: Click targets map back to the state: per-inference ids `i<n>`, per-formula ids `i<n>-<p>` (p is the position Interactive uses), literal and node ids `o<n>`, links `l<m>-<n>`, and Interactive::derivation_ids from a drawing id to the state id, for one- and two-sided (intuitionistic) drawings and for open goals. The SVG calls stay pure, clock-free and wasm-safe, with Euler Math supplied by the page (family name Euler Math, layout through textLength). Any new drawing (a new rule's glyph, a ProofStructure with weights, an ordinary derivation) keeps that id scheme, and the meaning of the ids is documented on svg::Style::ids as part of the public contract. The documentation of svg::Style::ids also records the accessibility contract of an interactive drawing: the root stays role=img with its title and, behind Style::description, its desc, and the click targets `i<n>-<p>` are not the accessible interface, which the client provides as controls outside the picture (a button per goal formula). If the author prefers the picture itself to be operable, Style gains a field that selects the root role (graphics-document with focusable targets) whose default reproduces today's output. Either way the choice is made before step 32 builds the client and does not alter the id grammar; a new Style field needs Style to be non_exhaustive or built only through its defaults (R50) (plan/notes/research/32-web.md, section 2).
- Met when: A test in core/tests/export.rs (or serialize) that every `i<n>-<p>` of an Interactive drawing, mapped through derivation_ids, is accepted by rules() and apply() at the same position in every mode; a test per drawing function asserting the ids and the absence of any host dependency; the id grammar in the rustdoc of svg; svg compiled for wasm in the wasm check (R41); the accessibility sentence is on Style::ids, or the field exists with a snapshot per role and the default snapshots unchanged.
- State now: Partly: the ids and the map exist, but the test of the Met when (every drawn position accepted by rules() and apply(), in every mode) was not re-verified for the two-sided drawing, and the wasm check of R41 does not exist. Evidence: export/svg/mod.rs:64-68 the `ids` field (id i<n>-<p> per formula) and :408; proofs/interactive.rs:929 derivation_ids; core-export.md:28-30; the OpenGoal style exists; plan/reports/11-svg.md:254-278 lists the calls and ids; svg is a default feature with no OS dependency. Position agreement in two-sided mode was not re-verified.

### R164. A net for a proved sequent is drawn as the command draws it, and a student-built net works
- Steps: 32 web client.
- Sources: plan/32-web.md:66-69,21-22.
- API item: ProofStructure (new, from_links, link, unlink, is_acyclic, is_correct, from_proof), svg::net, serialize/nets.rs.
- Requirement: For MLL the net is drawn as the command draws it, and a student-built net works: a net from a proved sequent (from_proof of the outcome's proof), partial structures drawn, a JSON round trip, link and unlink with cheap feedback (is_acyclic with a Scratch), the NetError as structured data with the occurrence ids to highlight (R129), and a size limit on drawing.
- Met when: The bindings call from_proof then svg::net(limit); tests in core/tests/export.rs and serialize.rs (existing); NetError gets its wire form.
- State now: Met for the library calls. Evidence: nets/mod.rs:232-406 from_links, from_proof (l.249), link and unlink (340, 365), is_acyclic(scratch) (394), is_correct (406); export/svg/mod.rs:626 net(&structure, &style, limit) returns TooLarge; serialize/nets.rs {sequent, mix, links}; NetError (l.46-77) carries ids but has no wire form.

### R165. The box drawing needs a layout order that may permute roots
- Steps: 33 MELL nets (D12).
- Sources: plan/33-mell-nets.md:20-21; plan/README.md D12.
- API item: export::svg::net and svg::Style (the layout order of literals and roots).
- Requirement: A box is drawn as a rectangle, but the interior of a box is in general not a contiguous id range: its links join literals of the principal formula and of several ? doors, which sit in different roots. The drawing therefore needs a layout order that may permute roots and trees (an input the drawer computes or is given), instead of literals strictly in occurrence order, and arcs for nested boxes must not cross rectangle borders wrongly.
- Met when: svg::net draws a structure whose box interiors are non-contiguous in id order with each rectangle enclosing exactly its interior; snapshot tests in core/tests.
- State now: Not met. Evidence: export/svg/net.rs:1-16 the documentation says literals sit side by side along the top in occurrence order; there is no box notion.

### R166. A drawing of an essential net
- Steps: 35 essential nets for IMLL (D12, D15).
- Sources: plan/35-mll-engines.md:40-41.
- API item: export::svg::net, net::estimate, Style, the svg.* style keys.
- Requirement: A drawing of an essential net (polarized, with the directions of its edges and the input and output split) exists whatever the measurement says about the engine: a library function taking a ProofStructure (the polarization from its Reading), honouring the same size estimate and limit (TooLarge) as svg::net, with its own style keys in the svg. option group (R144) so that the command and web options value covers it, and no font set in LaTeX or Typst output (Euler is for SVG and web only).
- Met when: A new svg entry point plus tests that the SVG of an engine-found net and of a from_proof net agree; the estimate is at least the drawn size, as tested for svg::net.
- State now: Not met. Evidence: core/src/export/svg/mod.rs:626 `pub fn net(net: &ProofStructure, style, limit)` draws the classical net only; core/src/export/svg/net.rs (398 lines) has no direction or polarity; Style at svg/mod.rs:61.

### R167. A net with cut links, and each step of elimination, can be drawn
- Steps: 34 cut (session 3).
- Sources: plan/34-cut.md:20-21.
- API item: export::svg::net and svg::Style.
- Requirement: Each step of the elimination is drawn: a net with cut links (two conclusion edges joined by a cut link, the cut formula trees between the sequent's conclusions), the redex highlighted, and a sequence of such pictures. The drawing reads the structure's conclusions as forest.roots() at the bottom layer and links as arcs over literals only, so both change. New knobs are fields of svg::Style (plain data, serde, deny_unknown_fields) defaulting to today's output, so the snapshots stay as they are. svg::net keeps the TooLarge limit argument.
- Met when: An SVG snapshot test of a net with a cut and of a step sequence; the existing net snapshots in core/tests/snapshots are byte-identical.
- State now: Not met. Evidence: core/src/export/svg/net.rs:1-12 (literals along the top, conclusions on the bottom, links as half-ellipses over two literals), :213-215 and :324 `for &r in forest.roots()`; svg/mod.rs:626 `pub fn net(net, style, limit)`.

### R168. The box groups have stable element ids and are read in the description
- Steps: 33 MELL nets.
- Sources: plan/33-mell-nets.md:20-21; plan/32-web.md:22; plan/reports/11-svg.md (Style::ids, desc).
- API item: svg::Style::ids and the description, element ids for boxes.
- Requirement: A web front end that highlights or selects parts of a net needs stable element ids for the new box groups (the box and its door), like ids gives formulas, and the `<desc>` text for screen readers reads boxes too. The ids are a pure function of the structure.
- Met when: Style flags select ids for boxes; the `<desc>` (the net's text form) lists boxes; snapshot tests.
- State now: Partly. Evidence: export/svg/mod.rs:66-76 `ids` (per formula of a conclusion) and `description` (the net's text form) exist; there is no box element or id.

### R169. The exports of an ordered derivation show no exchange and draw planar arcs
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-19.
- API item: export::{latex, typst, svg, rocq} and export::notation.
- Requirement: The outputs of an ordered derivation show no exchange and draw the sequent in its order. LaTeX and Typst print the Lambek rules and the two divisions (\\ and /) with notation entries and never set a font. The SVG draws the net with the planar linking, whose arcs on one side of the sequent do not cross, so the layout uses the literal order. The Rocq/NanoYalla export refuses the ordered modes with Unsupported (the kernel is commutative: it exports ex_perm_r), unless a non-commutative kernel is chosen. The options values stay one per output feature (D15) and gain no required field.
- Met when: Export tests for a Lambek derivation in LaTeX and Typst (compiled by the export flake check), an SVG test that the arcs are planar, and an Unsupported test for rocq in cyclic mode.
- State now: Not met. Evidence: core/src/export/rocq.rs:12-24 exports through ex_perm_r and refuses what has no kernel rule (Unsupported); core/src/export/notation.rs exists with per-rule notations; no ordered-mode handling is found by grep for cyclic, planar and Lambek under core/src.

### R170. The four outputs print quantifiers, predicates and terms through the shared notation
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:21,17; plan/later.md:319-321.
- API item: export::{latex, typst, svg, png, pdf, rocq}, export::notation::Notation and its Notation::term and ill, the export options values, RESERVED.
- Requirement: The outputs follow the data model: notation rows for the universal, the existential, predicate application and terms in each target, through the shared Notation (one match over walk stops, never recursion), with the quantifier symbols and term syntax as fields of each target's serde-able options value, never setting a font in LaTeX or Typst. Binder and variable names cannot clash with the target's keywords (as RESERVED does for Rocq). The SVG layout measures the new text. Forest::formula(o) and Reading::formula print an occurrence below a binder with the names of the binders above it (taken from its parents or from the member's instance), and the documentation of each function says so (plan/notes/research/impact-quantifiers.md, section 2.2 and section 3, item 5).
- Met when: Notation methods and snapshot tests per target on a quantified derivation, including a first-order sequent in each format; the export options keep serde and defaults; the flake export check compiles the output.
- State now: Not met. Evidence: core/src/export/notation.rs has Notation::term and ill over Visit stops and the shared symbol and label tables; each target has one options value with serde (export/mod.rs:12,62); core/tests/export.rs and the snapshots cover propositional output only; export/rocq.rs:129-160 has RESERVED and Unsupported; no quantifier rows exist.

### R171. The Typst export of a deep proof compiles without a package layout limit
- Steps: 32 web client; 22 follow-up.
- Sources: plan/later.md:1128-1140.
- API item: export::typst (tree layout without curryst).
- Requirement: The Typst export of a proof deeper than about eleven inferences compiles: write the tree with linlog's own layout (explicit widths in a grid or stack, as the SVG layout does) instead of curryst's nested show rules. This also removes the package import.
- Met when: An export test that compiles a 30-inference proof with typst (core/tests/export.rs) and a snapshot.
- State now: Not met. Evidence: core/src/export/typst.rs:5,14,19,73 still emit curryst (the CURRYST version constant) and document the show-rule depth limit.

### R242. A writer for first-order certificates
- Steps: 38 first-order; 31 Rocq library.
- Sources: plan/38-first-order.md:20-22 ("certificates in the Rocq library's first-order development"); plan/31-rocq-library.md:45-47; plan/later.md:318-324.
- API item: export::rocq::proof (R152), sequents::fmt::Walk, the library's first-order development (R68, R125, R161).
- Requirement: After step 38 the term writer of R152 emits a first-order certificate: the term table, the witness and eigenvariable nodes of R125 and the quantified statement, over the first-order development beside the first (R68). A certificate is still the sequent, the term and `check … = true` by computation. The statement printer of R156 handles binders and terms, as one loop over Walk stops with the term stops, with no recursion over a term (R43), and a bound variable's name cannot clash with the kernel's reserved names (R155). R161's refusal stays for the NanoYalla kernel, and for a first-order proof while the first-order development does not exist.
- Met when: A flake check compiles a quantified certificate (one with an existential witness, one with a universal eigenvariable, one under `!`) against the library, with `Print Assumptions` empty; core/tests/depth.rs has a case for a term nested 100 000 deep in the printed statement; the propositional snapshots are unchanged.
- State now: Not met. Evidence: core/src/export/rocq.rs targets NanoYalla only and handles no quantifier (R161); no first-order development exists; plan/38-first-order.md:20-22 is the only source that names the certificates.

## CLI

### R172. One `linlog prove` call on one LLTP file is enough for BenchExec
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:38-39.
- API item: linlog prove: --file with --input-format lltp, exit statuses 0, 1, 2 and 3, --quiet, --format json, the mode flags.
- Requirement: If BenchExec measures linlog, one `linlog prove` call on one LLTP file is enough: the mode (ILL problems are intuitionistic by their directory, not by the file), the one-thread or default configuration, the limits and a machine-readable verdict (exit status plus a verdict line or JSON) are all flags of the single call, so that a tool-info module can read it without parsing prose.
- Met when: A documented command line per column; a test or README example that `linlog prove --file X.p --input-format lltp --intuitionistic --deterministic --timeout 60s --memory-limit 4GiB --quiet` exits with 0, 1 or 3 and prints the verdict line the module reads.
- State now: Partly. Evidence: cli/src/batch.rs Entries and InputFormat::Lltp read .p files; exit statuses at argument_parsing.rs (Prove documentation: 0 provable, 1 unprovable, 3 stopped, 2 error); --quiet at argument_parsing.rs:787; the mode of an LLTP file from its directory is only in bench/src/problems.rs, so whether the command infers it is unknown.

### R173. `prove --format rocq` on an unprovable sequent writes a refutation certificate when one exists
- Steps: 31 Rocq library (items 8 and 6, CLI side).
- Sources: plan/31-rocq-library.md:48-49; plan/later.md:123-128 (`interact` certifies a finished session through `Interactive::proof()`).
- API item: cli/src/prove.rs (the match on verdict, format and quiet), cli/src/batch.rs, cli/src/ordinary.rs, check --format rocq and interact's proof --rocq.
- Requirement: `prove --format rocq` on an unprovable sequent writes a refutation certificate when one exists, and says on the verdict line, as a Rocq comment, when none does. It does so for plain and ordinary logic, in single and batch runs. The exit status stays 1 for unprovable. The kernel is chosen via `--style rocq.kernel` and Auto, with no flag of its own (cli.md). The term writer is called from the Proved arm with the mode, with no ViewOptions and no derivation-limit involvement. `check --format rocq` and interact's `proof --rocq` (which uses Interactive::proof) go through the same writer. README's examples and --help show the new outputs, and cli/tests/readme.rs runs them.
- Met when: New console blocks in README run by cli/tests/readme.rs; the flake rocq check compiles the command's outputs for each mode; cli/src/prove.rs has an arm for (Unprovable, Rocq).
- State now: Not met. Evidence: cli/src/prove.rs:1074-1111 `_ => Shown::Nothing` covers Unprovable and Unknown; prove.rs:253 `Format::Rocq => Compact::Never` and :687 `rocq::write(&d, ...)` build a Derivation; cli/src/batch.rs:399 maps Format::Rocq to the `v` extension.

### R174. `--net` accepts MELL proofs and the text form lists boxes
- Steps: 33 MELL nets.
- Sources: plan/33-mell-nets.md:20-21.
- API item: cli prove --net, check --net, nets_exist, the text form (Display for ProofStructure), README.
- Requirement: The command's net output extends to MELL: --net accepts MELL proofs, the text form lists boxes (and their doors) after the links, nets_exist and --help say which fragment is covered, and README's examples (run by cli/tests/readme.rs) change in the same commit. The net engine stays forced-only for MLL: `--engine net` on MELL still gives NetFragment (R89).
- Met when: `cargo test --workspace` (readme.rs) passes with a new MELL net example; `linlog prove --net` on a MELL sequent exits 0 with boxes in text and in svg.
- State now: Not met. Evidence: cli/src/argument_parsing.rs:777-783 --net is documented as MLL without units; cli/src/prove.rs:949-959 nets_exist refuses non-MLL; nets/mod.rs:428 Display lists links only.

### R175. The command can write the essential net as text, svg, png or pdf
- Steps: 35 essential nets for IMLL.
- Sources: plan/35-mll-engines.md:40-41.
- API item: cli OutputArgs::net, prove.rs net_into and nets_exist, the text Display of ProofStructure.
- Requirement: The command can write the essential net as text, svg, png or pdf for an intuitionistic IMLL sequent, taking it from the engine when that ran and from the proof otherwise (net_into already does the second). --net is a bool today, so the choice between the classical-style and the essential drawing needs a flag shape decided now (before 0.1.0 replace outright, no aliases), documented in --help and README.
- Met when: The flag or default chosen; an arm in net_into; the nets_exist message; a README console example run by cli/tests/readme.rs.
- State now: Not met. Evidence: cli/src/argument_parsing.rs:777-783 `pub net: bool`; cli/src/prove.rs:908-945 net_into (falls back to ProofStructure::from_proof), 949-960 nets_exist; .claude/rules/cli.md:475 says a format that draws nets gets an arm in net_into.

### R176. The command can ask for the ordered modes
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-20.
- API item: cli: prove, check, interact, seq print|json|fragment flags; exit statuses.
- Requirement: The command has a way to ask for the ordered modes (flags or a --calculus or --mode value) for prove, check, interact and seq, with the mode named in the output, the README console blocks and the batch file's mode column. The exit statuses stay 0, 1, 2 and 3 (an ordered-mode refusal is 2). The --help text comes from the documentation comments of argument_parsing.rs, and README examples are run by cli/tests/readme.rs, so each new flag needs a README block in the same commit.
- Met when: `cargo test --workspace` passes with new README blocks for cyclic MLL and a Lambek sequent; `linlog prove --help` lists the new mode.
- State now: Not met. Evidence: cli/src/argument_parsing.rs:328 conflicts_with_all lists intuitionistic, affine and mix only; :486-540 the flags --intuitionistic, --affine and --mix; no ordering flag.

### R177. The parser reads the two divisions and an order-keeping two-sided Lambek sequent
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-20.
- API item: parse::Parser (grammar) and Sequent Display / formula printing.
- Requirement: The text syntax needs the two divisions (left and right) and an order-keeping reading and printing of a two-sided Lambek sequent `A, B |- C` (antecedent order kept, the left of the turnstile not negated away in the printed form), plus a way to tell the parser the mode so that it keeps the root order and rejects an empty antecedent for Lambek. Parsing does not depend on the mode for the commutative syntax.
- Met when: Parser tests for the new operators and for an empty Lambek antecedent (a named error); a seq print round trip in the command; a README example block for a Lambek sequent.
- State now: Not met. Evidence: core/src/parse/mod.rs:32-48 Binary has the lollipop and the connectives, grammar line 12 "A ⊸ B is A^⊥ ⅋ B"; no division operator.

### R178. `--engine inverse` and `--engines inverse` are accepted
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:26-29.
- API item: cli EngineArg::Inverse and its From arm, bench run::EngineChoice::Inverse, the README --engine list, the bench --engines help text.
- Requirement: `--engine inverse` is accepted by the command and `--engines inverse` by the harness, with a documentation comment (the --help line) in each, and the README's list of engine names and the `linlog prove --help` text name it, so that an option-only engine is reachable from every front end.
- Met when: `linlog prove --engine inverse` and `linlog-bench run --engines inverse` run; cli/tests/readme.rs passes with the README engine list updated.
- State now: Not met. Evidence: cli/src/argument_parsing.rs:1161-1196 EngineArg has Auto, Focus, Net, TwoSided, Additive and Horn; bench/src/run.rs:157-186 EngineChoice the same; bench/src/main.rs (about line 102) hard-codes the list in the --engines documentation; README.md:59 lists `focus|net|two-sided|additive|horn`; cli.md "Extension points" describes the steps.

### R179. The command gets the cut: interact, check and an elimination command
- Steps: 34 cut (sessions 1 to 3).
- Sources: plan/34-cut.md:16-21 (the plan names no command; the commands are derived from the goal).
- API item: cli interact, check, a new elimination command.
- Requirement: `linlog interact` gets a `cut G FORMULA [P...]` command (and help and the --help text). `linlog check` accepts proofs with cuts (it reads the Proof JSON, so it follows from the wire form, R13). A command or flag runs the elimination (--steps, memory and time limits, a format for each step: text, LaTeX, Typst, SVG, JSON) with the exit status 0 for a normal form, 1 invalid, 2 error, 3 stopped or a limit. README gets examples and cli/tests/readme.rs runs them.
- Met when: `cargo test --workspace` runs README's new console blocks; `linlog <command> --help` shows the documentation comments of cli/src/argument_parsing.rs; a cli/tests/cli.rs test for the exit status 3 on a step limit.
- State now: Not met. Evidence: cli/src/interact.rs:23-45 HELP lists goals, rules, apply, undo, close, show, proof, save, load, help and quit (no cut); cli/src/argument_parsing.rs:40-80 Command has Prove, Check, Interact and Seq; no elimination command.

### R180. Every command accepts first-order input and prints first-order output
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:15-22,26-29.
- API item: cli commands prove, check, interact, seq print|json|fragment; README console blocks; cli/tests/readme.rs.
- Requirement: Every command accepts first-order input and prints first-order output without changing any existing README example's output or exit status (0, 1, 2 and 3). `seq fragment` names the new fragments. `interact` takes a witness (or "open", R97). New flags for the first-order bounds (R38) come from the documentation comments of cli/src/argument_parsing.rs. Each addition gets a README example that cli/tests/readme.rs runs.
- Met when: `cargo test --workspace` runs README's examples including new first-order ones; `linlog <cmd> --help` shows the new flags.
- State now: Not met. Evidence: CLAUDE.md says README's usage section shows every command and cli/tests/readme.rs runs them; cli/src/argument_parsing.rs holds the flags; there is no first-order example.

### R181. `interact --logic` shows the goals of an ordinary-logic session read back
- Steps: 22 follow-up; 28.
- Sources: plan/later.md:1082-1083.
- API item: cli interact --logic, an ordinary session over Image.
- Requirement: An interactive session over an ordinary-logic image has its goals shown read back as ordinary sequents: the library needs a mapping from the session's linear goals to ordinary sequents (Image-side display), and the command an `interact --logic`.
- Met when: `linlog interact --logic ...` is accepted, with a README example run by cli/tests/readme.rs.
- State now: Not met. Evidence: cli/src/argument_parsing.rs InteractArgs (lines 316-380) has no logic or translation argument; prove has them (ordinary.rs).

### R182. The command reads LLTP and .spec files through the library readers
- Steps: 22; 28.
- Sources: plan/later.md:1334,1354.
- API item: cli reading LLTP problems and coverability .spec files.
- Requirement: The command reads LLTP problems and coverability .spec files through the library readers, with the format chosen by file extension.
- Met when: `linlog prove --file X.p` and X.spec work and are in README.
- State now: Met. Evidence: cli/src/argument_parsing.rs:554-620 the input-format enum has Lltp, Mist (.spec) and Tptp, chosen by extension.

### R183. The search options can be read from a file by the command
- Steps: 32 web client (D15).
- Sources: plan/README.md:685-700.
- API item: cli --style-file and --style, and the search options (no file).
- Requirement: Every options value is reachable from the command as flags and as a file. Outputs have both, but the search and batch options have flags only, so the same JSON that the web client keeps in its settings cannot be handed to the command. The command gets an --options-file (or a section of the style or config file) that reads the search Options JSON (R1), with flags overriding it.
- Met when: An options file is read, flags override it, and the README shows an example run by cli/tests/readme.rs.
- State now: Partly. Evidence: cli/src/style.rs:1-6 reads `--style-file` JSON and `--style KEY=VALUE`; cli/src/argument_parsing.rs has flags for fragment, engine, bias, copies, forward-copies, timeout, memo-limit, memory-limit, recursion-limit, jobs, no-check and occurrence-limit (lines 102-217, 572), but no file for them.

## Harness

### R184. linlog is timed the same way as the other tools, and the time used is stated
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:32-37,38-44.
- API item: cli --timeout (counted from process start), bench time_ms (the search alone), check_ms, a new CSV column load_ms.
- Requirement: Other tools are timed by BenchExec over the whole process (reading and translation included), while the harness times linlog's search alone, after the load and without the proof check. The comparison needs linlog measured the same way as the others: the whole process under BenchExec running the command, or the harness reporting load_ms and check_ms separately so that the sum is reproducible. The translation time of each tool is stated.
- Met when: Either BenchExec runs `linlog prove --file P.p ...` directly and the table uses its wall and CPU time, or a load_ms column is appended to the bench HEADER and filled by the child; a documentation line says which time the table uses.
- State now: Partly. Evidence: bench/src/run.rs HEADER (line 32) has time_ms, cpu_ms, wait_ms and check_ms but no load or total time; the "loaded" line (run.rs:36) is only used to start the limit; the command's --timeout counts from the command's start (argument_parsing.rs documentation).

### R185. Tools other than linlog's engines are rows of the same CSV
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:31-37,44-46.
- API item: bench::run::HEADER, run::one and the child protocol, a new tool column and a driver interface.
- Requirement: The harness runs tools other than linlog's engines (each with a command line, a verdict reader and its own limits) and writes rows that summary, the cactus script and the cross-tool contradiction list read. The CSV needs a tool column (and a tool version), a raw-status field, and columns that fit a tool with no node or memo counters. Columns are only ever appended (bench rules), and summary's configuration key includes the tool.
- Met when: A tool (and tool_version, translate_ms, peak memory) column appended to HEADER; a Driver trait or enum in bench/src/run.rs with linlog as one implementation; the summary.rs configuration key includes the tool; a test that a mixed CSV summarises per tool.
- State now: Not met. Evidence: bench/src/run.rs HEADER (lines 32-36) has engine_requested and engine only for linlog's own engines; grep for tool in bench/src finds nothing; bench.md says to add columns at the end of the tail and never rename one.

### R186. Wall time, CPU time and peak memory per run for every tool
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:38-39,48-50.
- API item: bench CSV columns time_ms and cpu_ms; a new peak memory column.
- Requirement: Ranking is by problems solved within a limit, but the published rows carry wall time, CPU time and peak memory per run for every tool, including linlog, so that memory failures and the memory comparison are visible.
- Met when: A peak_kb (or peak_bytes) column appended to HEADER, read from cgroup or getrusage by the parent, filled for every tool.
- State now: Not met. Evidence: grep -ni 'peak\|max_rss' bench/src finds nothing; only memory_limit (the bound) is a column; cpu_ms and wait_ms exist.

### R187. The report lists the problems on which tools disagree
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:44-45.
- API item: bench CSV expected and verdict columns, summary --against, compare.rs.
- Requirement: The report lists the problems on which tools disagree (and where linlog disagrees with a header's Status). summary compares verdicts across tools, treating timeouts and "unknown" as no answer, and the disagreement list names the problem and each tool's verdict. linlog's proofs are checked; a proved and unprovable pair of tools is a finding.
- Met when: A `summary --against` mode (or a new subcommand) keyed on problem and mode across tool values, with a test on two small CSVs with a planted disagreement.
- State now: Partly. Evidence: bench/src/compare.rs against(reference, files) compares files by problem "whatever the files' names" but is written for linlog configurations of one baseline and has no tool key; the expected column is present in HEADER.

### R188. The problem set can be split deterministically into shards for CI jobs
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:51-55.
- API item: linlog-bench run --only, a new --shard K/N or a list subcommand.
- Requirement: The workflow_dispatch job splits the problems into N jobs so that every tool runs every problem of a job on that job's machine. The harness has a deterministic split of the problem list (a stable order, the same split in every job and for every tool) and rows that concatenate into one file.
- Met when: `linlog-bench run --shard K/N` (or `linlog-bench list` feeding --only), and a test that the N shards partition the list exactly and in a stable order.
- State now: Not met. Evidence: bench/src/main.rs RunArgs has --only (substring match, line 93), --reverse and --resume but no shard; there is no list subcommand (Command: Run, Summary, Families, One).

### R189. The comparison runs as a CI job with nothing but Nix installed
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:51-55.
- API item: a new .github/workflows job (workflow_dispatch), modules/bench.nix packages per tool, nix build .#lltp, .#iltp, .#qcover.
- Requirement: The job installs only Nix, builds each tool at a pinned version and the libraries from the flake, runs the one script, and uploads rows, tables and figures. The library and the command therefore build and run on a hosted runner (4 vCPU, 16 GB) with the baseline script's limits, and the script does not need the author's machine (taskset pinning to performance cores, systemd units, prlimit, jj).
- Met when: A script (separate from bench/baseline.sh, or a mode of it) with no machine-specific pinning; an actionlint-clean workflow with SHA-pinned actions and permissions: contents: read; a flake package per tool.
- State now: Partly. Evidence: modules/bench.nix has lltp, iltp and qcover only (no other tool); .github/workflows/ci.yml has workflow_dispatch at line 15 but no comparison job; bench/baseline.sh pins cores and uses systemd (bench.md "baseline.sh").

### R190. The hardware, the versions and the commands are recorded by the run itself
- Steps: 29 comparison; 32 web client.
- Sources: plan/29-comparison.md:47-50,56-58; plan/notes/research/29-comparison.md (section 3).
- API item: linlog --version and linlog-bench --version, bench/baseline.sh starts.txt, the RESULTS.md header, a new environment record.
- Requirement: The published page needs the hardware, every tool's version, linlog's commit, the commands and the limits, recorded by the run itself and not typed by hand, and the configuration each tool ran in, for the message to its authors. The harness writes one machine-readable record (tool versions, CPU model, cores, memory, kernel, command lines) next to the CSV, and linlog's version names its commit. The JSON a verdict is written as, and the command's text output on request, name the library version and the Options value (R1) the verdict ran under, so that a row of the comparison and a stored outcome can be traced without a second record; the form is optional keys, additive under R6, and it needs the wire form of the options (R1) first.
- Met when: A file such as env.json, or a header block written by `run`, beside the CSV; `linlog --version` prints the commit; the README section cites it; `linlog prove --format json` shows the version and the options, a pinned outcome string in core/tests/serialize.rs carries them, and the options read back as equal (R1).
- State now: Partly. Evidence: bench/baseline.sh:332-335 appends the commit, load and force flag to starts.txt and RESULTS.md takes lscpu's model name (line 541); cli/src/argument_parsing.rs uses clap `#[command(version)]` only (the crate version, no commit); CSV rows have no tool-version field.

### R191. Cactus and scatter plots can be drawn from the CSV alone
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:41-46.
- API item: bench CSV (time_ms, verdict, tool, problem, mode) and linlog-bench summary output.
- Requirement: A script draws cactus and pairwise scatter plots from the CSV alone, so each row carries, per tool and problem, a decided or undecided flag, one comparable time and the problem identity (source, family, problem, mode), and the CSV remains comma-split without quoting. A machine-readable summary (sorted solved times per tool) is an optional convenience; the table form of summary is Markdown.
- Met when: The plot script reads only HEADER columns by name; a test or example file; the time column semantics are documented (see R184).
- State now: Partly. Evidence: bench/src/run.rs HEADER has source, family, problem, mode, verdict and time_ms; bench/src/summary.rs outputs Markdown only; no tool column; "A problem counts once per configuration" in bench.md.

### R192. The checking of linlog's answers is stated and separable from the speed column
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:44-46 (with plan/notes/comparison.md "say whose answers are checked").
- API item: Options::check, cli --no-check, the bench columns check_ms and checked, proofs::check.
- Requirement: The table says that linlog's proofs are checked by a checker sharing no code with the search, and what the check costs, while the speed column excludes or includes it by a stated choice. The library keeps check on by default, the command has a switch, and the harness records the check's verdict and time per run. An external tool's output has no such check (unprovable answers are unchecked for everyone).
- Met when: The columns checked and check_ms are present; a documentation sentence in TOOLS.md; both the command's --no-check and Options::check(false) are usable for a run without the check.
- State now: Met. Evidence: core/src/search/mod.rs Options::check (about line 1003, DEFAULT_CHECK true); cli --no-check; bench HEADER has checked and check_ms; bench.md "checks it outside the timed part".

### R193. The practice problems of step 27 are a stable, fixed-order part of the shared set
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:44-46.
- API item: linlog::mist::read, bench --spec, nix build .#qcover, problems.rs Reference.
- Requirement: The set includes what step 27 added from practice (the qcover coverability suite) beside LLTP and the families. Other tools need translators from .spec as well, and the harness lists it as a source with a fixed order and expected results, so that shards and tools see the same problems. A coverability question is affine, run intuitionistic affine.
- Met when: The qcover problems enter the shared problem list with their mode; the .spec translator, or an explicit "not covered" row per tool.
- State now: Partly. Evidence: bench/src/main.rs RunArgs.spec (lines 82-86) and core/src/mist.rs exist; bench.md says the expected result is stated in 12 of 176 files; there is no translator for other tools.

### R194. The CSV stays comparable between baselines
- Steps: 30 release (the third baseline).
- Sources: plan/30-baseline-release.md:25-35; .claude/rules/bench.md.
- API item: bench CSV HEADER (bench/src/run.rs:33-36).
- Requirement: The third baseline stays comparable with the second: the CSV columns and the meaning of the existing ones (verdict, reason, nodes, splits, memo_*, pool_after, cpu_ms, wait_ms) remain, and any new column goes at the end, with older files still read by summary and compare.
- Met when: `linlog-bench summary --before bench/results/2026-10-02 bench/results/DAY/*.csv` reads both generations; the harness check of nix flake check passes.
- State now: Met. Evidence: bench/src/run.rs:33 HEADER has 36 columns with pool_after last; a note there says older files with a portfolio column are still read; bench/src/compare.rs implements before().

### R195. The comparison command sets three baselines side by side
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:28.
- API item: linlog-bench summary --before, compare::before.
- Requirement: The comparison command sets three baselines side by side (first, second, third), not only one before directory against the files given, since COMPARISON.md gains a third column.
- Met when: A summary invocation or option taking more than one earlier directory prints the three-way tables; COMPARISON.md documents the command.
- State now: Not met. Evidence: bench/src/main.rs:47-48 has a single `before: Option<PathBuf>` (conflicts_with against); bench/COMPARISON.md documents only the two-way command with --before.

### R196. The pass under the new defaults can be run by the baseline script and reproduced by flags
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:29-35.
- API item: baseline.sh stage lltp-default; CLI --jobs all, --pool-after, --timeout; DEFAULT_POOL_AFTER, DEFAULT_TIMEOUT.
- Requirement: The pass under the new defaults (every core, pool after 0.1 s, timeout 2 s) is runnable by the baseline script and reproducible by flags, and it still agrees with what the command defaults are, so that the harness pass measures what a user gets. The CSV records the stop reasons so that its latest stops (queued pool tasks after a stop) can be read.
- Met when: bench/baseline.sh runs lltp-default with --jobs all --pool-after 0.1 --timeout 2, and a test or comment ties these numbers to the command constants; the reason column holds the stop cause.
- State now: Met. Evidence: bench/baseline.sh:497 `run - lltp-default --lltp "$lltp/ILL" --copies none --jobs all --pool-after 0.1 --timeout 2`; cli/src/argument_parsing.rs:389 DEFAULT_TIMEOUT is 2 s and :395 DEFAULT_POOL_AFTER is 100 ms; bench HEADER has reason and wait_ms. The numbers in the script are literals, not derived from the constants.

### R197. The baseline's results name the exact commit
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:25-27; bench/baseline.sh:224-234.
- API item: the baseline commit stamp in RESULTS.md.
- Requirement: The third baseline's RESULTS.md header names the exact commit measured, and the script records a working copy with uncommitted changes. Derived, not stated in any source: step 30 lists the baseline and the release as separate tasks (plan/30-baseline-release.md:25-27 and 39-45) and says nothing of their commit order, but it would be best for the release and the numbers to cite one tree, so the baseline is taken at, or its commit is an ancestor with the same core, cli and Cargo files as, the commit the 0.1.0 manifests are at.
- Met when: The RESULTS.md header says commit X with no "uncommitted changes". If the derived wish is kept, X contains version 0.1.0 in the manifests and CHANGELOG, or the report says how X differs from the release commit.
- State now: Met for the mechanism. Evidence: bench/baseline.sh:224-234 commit() prints the jj commit_id and appends "with uncommitted changes to ..."; the manifests already read version 0.1.0 (core/Cargo.toml:7, cli/Cargo.toml). The commit order (the baseline after the release metadata) is the step's own work.

### R198. The library's tests can run under wasm
- Steps: 32 web client.
- Sources: plan/32-web.md:30-31,44-47.
- API item: tests in core/src (unit) and core/tests/depth.rs and parse.rs.
- Requirement: The library's tests are runnable under wasm (the bindings' tests "run under wasm"). Tests that spawn threads or read Instant are gated (cfg not wasm) or split, and a runner (wasm-bindgen-test, wasmtime or node) is chosen so that the checker's, the memo's and the recursion-limit tests run there.
- Met when: A flake check runs the core test suite for wasm32 and passes; the gated tests are listed.
- State now: Not met. Evidence: std::thread in tests: core/tests/depth.rs:140,173; core/tests/parse.rs:222; src/proofs/interactive.rs:1131; src/proofs/derivation.rs:1946; src/nets/sequentialize.rs:273; Instant in src/search/net.rs:762-765 and src/occurrences/mod.rs:785; no wasm runner anywhere.

### R199. The Rocq checker is cross-checked against the Rust checker on mutants and unchecked terms
- Steps: 31 Rocq library (item 7).
- Sources: plan/31-rocq-library.md:51.
- API item: the export::rocq::proof writer, Proof::new and serialize/proofs (unchecked terms), agrees_with_the_first_implementation.
- Requirement: The term writer writes any well-formed Proof without checking it, including mutants and terms the Rust checker rejects, and does not go through Proof::derivation(), which checks first. Only then can the Rocq checker be run against the Rust checker's verdict on rejected terms. Where a certificate is wanted for a search result, the caller has already checked it (prove_goal does). The cross-check corpus is the proofs, mutants and random terms that agrees_with_the_first_implementation already generates, written as Rocq terms with the expected accept or reject. Rocq must agree on accept or reject; the error kind may differ (Problem::Surplus is a lemma or nothing over nat, and Problem::Memory is no verdict).
- Met when: A test or script writes N valid and N mutant proofs per mode into one .v file with `Example t_i : check ... = true/false := eq_refl.`, the flake rocq check compiles it, and the Rust side records its verdict per term in the same fixture.
- State now: Not met. Evidence: proofs/check.rs and proofs/oracle.rs generate and compare mutants only inside Rust (core-proofs.md, `agrees_with_the_first_implementation`); no export of unchecked terms exists; grep for rocq in core/tests/export.rs shows only derivation-based snapshots.

### R200. The flake builds the Rocq library and compiles the certificates of both kernels
- Steps: 31 Rocq library (items 1, 4 and 2).
- Sources: plan/31-rocq-library.md:28; plan/later.md:407-417.
- API item: modules/rocq.nix (checks.rocq), core/tests/export.rs snapshots, core/tests/snapshots/*.v, the new repository layout.
- Requirement: The flake builds the library the way nixpkgs builds a Rocq library (a package) and as a check that compiles every certificate of both kernels and prints the assumptions of the main theorem. Snapshots exist per mode for the library kernel (classical, Mix, affine, two-sided intuitionistic, affine ILL, and a refutation of each kind), kept apart from NanoYalla's by name or directory so that the check compiles each against the right load path. The library lives in this repository under linlog's licence, builds with Rocq's standard library only (nixpkgs' Rocq, From Stdlib, a _RocqProject or dune theory, an opam file), has no Admitted and no axiom, and the check fails on any Rocq output and checks that Print Assumptions of the main theorems is empty (a no-Admitted and no-Axiom grep, or the assumption check).
- Met when: `nix build .#checks.x86_64-linux.rocq` compiles both sets; `nix flake check` runs the assumption check; the snapshot test in core/tests/export.rs blesses the new .v files.
- State now: Not met. Evidence: modules/rocq.nix:18-20 snapshots filter hasExt "v" over all of core/tests/snapshots and compile with `-R nanoyalla NanoYalla`, and :26-54 builds checks.rocq for NanoYalla only; there are no library snapshots and no package; core/tests/export.rs:82-87 and :313-331 pin only derivation-based NanoYalla output and the Mix and affine refusals.

### R201. An independent test oracle for the box criterion
- Steps: 33 MELL nets.
- Sources: plan/33-mell-nets.md:26-28; plan/reports/17-assessment.md 3.4.
- API item: nets::graph tests (enumerate, agrees_with_enumeration), search/generate.rs proof generators.
- Requirement: A test oracle for the box criterion independent of the implementation: brute-force enumeration of switchings at every box depth (boxes contracted) on small structures, plus generated MELL proofs for the round trip. The existing generator already has an exponentials flag. For structures with weakening or bottom the oracle is the sequent calculus, by brute force on every structure of a few nodes, with and without Mix, besides the enumeration of switchings (R78).
- Met when: An agrees_with_enumeration-style test over random linkings and random box placements, and a round-trip test over generate proofs with exponentials, both in `cargo test -p linlog`.
- State now: Partly. Evidence: core/src/nets/graph.rs:430 `fn enumerate` and :566 `agrees_with_enumeration` (up to 16 pars, MLL only); core/src/search/generate.rs:53,205 `exponentials: bool` options exist for the proof generators.

### R202. Proofs with cuts can be built for tests and for the interactive rule
- Steps: 34 cut (sessions 1 to 3).
- Sources: plan/34-cut.md:31-32.
- API item: a constructor of proofs with cuts (Proof composition), the test generators in search/generate.rs.
- Requirement: Termination and preservation of the conclusion are tested "on generated proofs with cuts", and every result goes through the checker. The test generators (search/generate.rs, test-only, cut-free by construction) need a way to build a proof with cuts: a function that composes two proofs on a cut formula, which is also how the interactive proof builds its Cut nodes, and a generator that puts cuts into generated proofs (also in the intuitionistic generator and with exponentials and Mix). Whether it is public (a compose or cut constructor on Proof) or test-only is a decision.
- Met when: A generator in generate.rs plus tests that run elimination to the end on at least a few thousand generated proofs per mode, with bounded step counts, checking Proof::check on each intermediate and the final proof, and that the final one has no Cut node and the sequent is unchanged.
- State now: Not met. Evidence: core/src/search/generate.rs:1-10 (module documentation: "a cut-free proof is built bottom-up"); it is `#[cfg(test)] pub(crate) mod generate` (search/mod.rs:19-20), so the harness cannot use it; grep -i cut core/src finds only documentation sentences and reference.rs's budget "cut".

### R203. A test-only enumerator of all axiom linkings
- Steps: 35 net engine pruning (differential review).
- Sources: plan/35-mll-engines.md:25-27.
- API item: a new test-only enumerator of all axiom linkings of an MLL forest.
- Requirement: A brute-force enumeration of every linking (and the orbits under the symmetries a break claims to quotient) exists in the tests, so that each pruning rule is checked as "every orbit of proof nets keeps a representative", not only by verdict against the focused engine.
- Met when: A `#[cfg(test)]` function in core/src/search/net.rs or nets/ lists all complete linkings accepted by ProofStructure::is_correct, used by a test per pruning rule.
- State now: Not met. Evidence: grep for brute finds only the switching enumeration core/src/nets/graph.rs:426,562; the net.rs tests compare with the focused engine only (net.rs:697-800).

### R204. The harness records the routing feature per problem, computed through the library
- Steps: 35 routing feature.
- Sources: plan/35-mll-engines.md:31-34.
- API item: bench run::HEADER column multiplicity, families, summary and compare.
- Requirement: The harness records the new routing feature per problem (a CSV column appended at the end of the tail, computed through the library's own function so that the two cannot drift) and has families in which the feature varies independently of literal multiplicity, so that summary can tabulate focus against net by feature value. The existing --engines focus,net runs are the vehicle.
- Met when: A new column in HEADER and the tail, summary grouping by it, the baseline.sh engines stage unchanged otherwise, and the bench flake check still passes.
- State now: Partly. Evidence: bench/src/run.rs:32-39 HEADER has occurrences and multiplicity only; run.rs:825 multiplicity(); the families wide-m1..m4, partition-* and 3-partition-mll-* in core/src/families.rs:160-264 already separate the two cases; bench/baseline.sh:433-438 runs the engines stage.

### R205. Unit-free intuitionistic families exist for the comparison of the essential engine
- Steps: 35 essential nets for IMLL.
- Sources: plan/35-mll-engines.md:38-40.
- API item: core::families (generators), bench problems.rs and run.rs, the baseline.sh engines stage.
- Requirement: The comparison "with the embedding on the baseline's IMLL rows" needs IMLL rows the harness can run under the new engine and under net and two-sided: generated unit-free intuitionistic families (wide, curried, chain, repeated literals) in core::families, and the unit-free LLTP ILL subset, with --engines accepting the new name and the refused classification keeping non-IMLL rows out of the tables.
- Met when: Families added with expected verdicts derived from the combinatorial problem; an IMLL engines stage in baseline.sh; summary --against tabulates essential against net.
- State now: Partly. Evidence: core/src/families.rs: every generated family uses Mode::CLASSICAL (lines 110-264, 606), so no generated IMLL family exists; the Engine documentation (search/mod.rs:657) quotes intuitionistic wide and curried measurements from outside families.rs; bench/baseline.sh:433-438 the engines stage; the LLTP ILL problems come from bench/lltp.

### R206. A counter-exact target set for the net engine exists before the first step that changes ProofStructure
- Steps: 28 audit or 30 release (created, with the baseline of step 30); 33, 34, 35 and 36 (every step that changes ProofStructure or the net engine); D17.
- Sources: plan/35-mll-engines.md:5-7; plan/README.md:712-719; plan/notes/research/33-mell-nets.md (section 3, Nets; section 4); plan/notes/research/impact-boxes.md (the net engine's hot path; stage 2, item 9); plan/notes/research/36-lambek.md (section 3, Nets and the net engine); plan/notes/research/35-mll-engines.md (section 4, API stability).
- API item: a target list for the net engine beside bench/targets.sh and bench/TARGETS.md, bench/targets/*.csv, the search::Statistics columns links and tests.
- Requirement: The net engine has a target set of its own (the focused engine's does not cover it at all), run on one thread on pinned cores, with a committed CSV of verdict, nodes, links, tests and cpu_ms per row, taken before step 33 changes ProofStructure. Every later step that generalises the structure (boxes, cut links, the calculus descriptor, planarity, the essential criterion) compares against it: verdict, links and tests equal for every decided row and cpu_ms within a few percent, as D17 asks of the focused engine. A change of the net engine's own search (step 35) compares verdicts and time against the baseline of step 30, and the counters against this set before and after.
- Met when: The target list and its CSV exist and are named in CLAUDE.md's verification table for changes under nets/ and search/net.rs; linlog-bench summary sets two runs side by side.
- State now: Not met. Evidence: bench/targets.sh and bench/TARGETS.md are the focused engine's set only; the baseline's net rows (the engines and period stages, bench/baseline.sh:433-442) are not counter-exact.

### R207. Ordered-mode problems with known verdicts exist, and the harness accepts the mode words
- Steps: 36 Lambek.
- Sources: plan/36-lambek.md:17-19.
- API item: core/src/families.rs generated families, bench problem files (bench/src/problems.rs), the harness mode column.
- Requirement: The test and benchmark inputs need ordered-mode problems with known verdicts: a Lambek and cyclic family (type-raising, composition, the standard categorial-grammar sentences, and sequents provable commutatively but not cyclically), and the harness's mode column and problem-file mode words accept the new modes, so that `bench run --all-families` checks verdicts (a MISMATCH is a bug). The generator and the expected verdicts are the library's to offer; the benchmarking itself is the step's.
- Met when: A new family in families.rs with a verdict per instance, listed by `linlog-bench run --all-families --timeout 5` with no MISMATCH; the problem file format accepts "cyclic" and "lambek".
- State now: Not met. Evidence: bench/src/problems.rs:11 the mode words are classical, mix, affine, intuitionistic and so on; core/src/families.rs is #[non_exhaustive] at :35 and :53 (room to add); no ordered family found by grep.

### R208. The inverse engine can be run over exactly the rows a baseline left undecided
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:23-26,27-28.
- API item: bench run --only, problems sources, summary, run::HEADER columns, bench/results/DAY/*.csv.
- Requirement: The harness runs the inverse engine over exactly the rows a baseline left undecided (timeout or unknown), and over the families, and sets its verdicts and times beside the baseline's, with the engine in the engine column. That needs a way to select problems by a previous CSV's status (or a generated problem list) rather than by name substring, and new inverse counters as columns appended at the end of the tail only.
- Met when: A command or flag such as `--only-undecided FILE`, or a summary output listing undecided rows as a --problems file; `linlog-bench run --engines inverse --family ... --timeout 5` rows readable by summary and --against; mismatches against expected show as MISMATCH.
- State now: Not met. Evidence: bench/src/main.rs:91-94 `--only` takes substrings of FAMILY/NAME; bench/src/main.rs:35-58 Summary has --before and --against comparison but no extraction of undecided rows; run::HEADER (bench/src/run.rs:32) has 36 columns and bench.md says to add columns at the end of the tail.

### R209. A new engine joins the differential setup against the reference prover from the start
- Steps: 35 net pruning and essential nets; 36 planar engine; 37 inverse method; 38 first-order (every change of a search is reviewed by a panel).
- Sources: plan/37-inverse.md:17-18,30-32; plan/later.md:714-715 ("the engines of steps 27, 35 and 37 join its test"); plan/35-mll-engines.md:43-45; plan/36-lambek.md:30-32; plan/38-first-order.md:35-37.
- API item: search::reference test prover, configurations(), Tally, the engines_agree_* tests, the bench targets.sh comparison columns.
- Requirement: The new engine is in the test-only differential setup from the start: it joins `configurations` in search/reference.rs, and the tests that compare every engine to the reference prover (classical, intuitionistic, affine, Mix, Horn programs, contraction-needing sequents, families at decidable sizes) cover it, with "never a proof against a refutation" as the assertion. Its counters on one thread are a function of the input (no iteration-order or address dependence), so runs can be compared like bench/targets/after-bias.csv. The same holds for every engine a later step adds or changes: the pruned net engine and the essential-net engine of step 35 join `configurations` (the pruned engine as the same variant with each rule switched off, R143, beside the unpruned run), or the entry names their brute-force oracle (R203 for the linkings, R123's brute-force reading of the dominator condition for essential nets); the planar engine of step 36 joins it in the ordered modes, or is checked against the enumeration of planar linkings of R104, since the reference prover is commutative; first-order (step 38) has no reference prover, so either a bounded-instantiation reference decision for first-order is added to search/reference.rs, or the entry states that first-order verdicts rest on the families' constructions only (R211).
- Met when: `configurations` has an inverse entry, and an entry for each of the other engines above or the named oracle; the existing agreement tests pass with it; the fixed-seed hasher (core/src/hash.rs) is used for every table of the engine; for first-order, the reference decision exists or the statement is in core-inputs.md.
- State now: Not met. Evidence: core/src/search/reference.rs:675-693 `configurations` lists dispatch, net, additive, horn, focus and two-sided under each bias, and dispatch on two threads; .claude/rules/core-search.md ("A new engine joins `configurations` in its tests"); core/src/hash.rs is foldhash::fast::FixedState, so determinism is available.

### R210. Families exist on which the inverse engine is meant to win
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:23-25,30-31.
- API item: core/src/families.rs generated families, lltp::read, bench/problems, mist::read.
- Requirement: The inputs on which the engine is to win are generatable and loadable on demand: a family with many hypotheses and a small goal (theory-heavy), and Mix families (mix at ten and eleven pairs), each with a verdict proved by construction, so that the first session's measurement and the dispatch feature's threshold can be set on them, and `verdicts_as_constructed` covers them.
- Met when: `linlog-bench families` lists them; `bench run --all-families --timeout 5` shows no MISMATCH for the inverse column.
- State now: Partly. Evidence: plan/later.md:221-224 names Mix (mix at ten pairs 146 s) and the Petri nets beyond the forward search as what is left; core/src/families.rs holds the generated families and bench.md "A family" says the entry needs a verdict by construction; whether a family of many hypotheses and a small goal exists was not checked (the `mix` family exists per later.md).

### R211. First-order families with known verdicts
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:33-34.
- API item: families::{Family, Instance, FAMILIES}, bench --all-families.
- Requirement: The families can produce first-order instances: Instance already holds a Sequent, Mode, provable and copies, so first-order sequents fit once the data model exists. The generators need the programmatic builder (R59), known verdicts proved by construction from the combinatorial problem (the rule of bench.md: never from an engine; no independent first-order engine exists in the repository, see R209), a per-family size ladder, and the harness's MISMATCH check runs on them. Family and Instance are #[non_exhaustive] so fields can be added (for example an expected bound).
- Met when: New FAMILIES rows with known verdicts; `cargo run --release -p linlog-bench -- run --all-families --timeout 5` shows no MISMATCH; core-inputs.md is updated.
- State now: Partly. Evidence: core/src/families.rs:35-66 Instance and Family are #[non_exhaustive], with generate: fn(u32, u32) -> Instance; all existing generators use `parse(&format!(...))`; no first-order row.

### R212. The harness can hand LinearOne the same first-order problems
- Steps: 38 first-order.
- Sources: plan/38-first-order.md:33-34.
- API item: bench/src/compare.rs, a new first-order sequent printer in LinearOne's input syntax, README and COMPARISON.
- Requirement: To compare with LinearOne the harness hands it the same problems: the library (or the harness) has a writer of a first-order MILL sequent in LinearOne's input syntax, and a way to record its verdict and time next to linlog's in the CSV (new columns appended, existing ones unchanged). Probably a harness-side writer over the public term accessors, so the public Sequent API exposes terms, binders and symbols read-only.
- Met when: Public read accessors for terms and binders; a bench compare mode with a LinearOne driver; a documented run in bench/COMPARISON.md.
- State now: Unknown. Evidence: bench/src/compare.rs exists and COMPARISON.md compares baselines of linlog itself; it was not checked which external provers compare.rs drives, or whether the Sequent accessors (terms(), term(), roots(), atom_names()) suffice for a writer once terms exist.

### R213. The summary does not count a late verdict as solved, and prints the engine counters
- Steps: 28 audit; the engine-comparison step.
- Sources: plan/later.md:1315-1355.
- API item: bench summary (late verdict, counters table), reruns.txt.
- Requirement: `linlog-bench summary` does not count a verdict that arrived after the time limit as solved, and prints a table of the counters (nodes, splits, memo_hits, memo_entries) per configuration, since the engine-comparison step and the audit's oracle rest on them.
- Met when: A summary section with counters; a test or smoke run showing a late row counted apart.
- State now: Not met. Evidence: bench/src/summary.rs:45-47 solved() is only `verdict in proved|unprovable` (no timeout_s comparison); grep for nodes and memo_hits in bench/src/summary.rs and compare.rs finds none; the harness already writes the verdict before the check (run.rs:8-12, 708) and has --load-limit.

### R214. A new problem set enters as a problem file or a source without changing the CSV columns
- Steps: 28 audit; bench.
- Sources: plan/later.md:1378-1381.
- API item: bench/src/problems.rs sources (problem files under bench/problems/).
- Requirement: New problem sets from practice (coverability suites, planning domains, ILL synthesis, llprover's examples) enter as a problem file under bench/problems/ (`name; mode; expected; copies; sequent`) or a new source in problems.rs, without changing the CSV columns, which are the harness's interface.
- Met when: A problem-file source documented in bench.md and exercised by `linlog-bench run --problems DIR` on a small file; the CSV tail fields unchanged.
- State now: Met. Evidence: bench/src/problems.rs reads problem files, LLTP (--lltp), .spec (--spec) and families; .claude/rules/bench.md:14-27 describes the four sources; bench/problems/slow-tests.txt exists.

### R215. A mechanical check that a public API change is a version change
- Steps: 30 release (D18).
- Sources: plan/README.md:721-727.
- API item: cargo-semver-checks (or cargo public-api) in the devshell and the flake check.
- Requirement: From 0.1.0 on the crates follow semantic versioning, which needs a mechanical check that a change to the public API of linlog is a version change; the check compares against the last published release.
- Met when: A flake check (or CI job) runs the tool against the released crate and passes on main.
- State now: Not met. Evidence: grep for semver in modules/, .github/, deny.toml and CLAUDE.md finds nothing; modules/checks.nix lists build, clippy, test, doc, deny, features, export, rocq and bench only.

### R216. The target set is the regression oracle for "the propositional case does not change"
- Steps: 28 audit; 33, 35, 36, 37, 38 (every step that changes a search or a data type); D17.
- Sources: plan/later.md:78-86,332-334; plan/README.md:712-720.
- API item: bench/targets.sh, bench/targets/after-bias.csv, linlog-bench summary and compare, core/tests/serialize.rs snapshots.
- Requirement: On one thread the engines' counters on the target set (verdict, nodes, splits, memo_hits, memo_entries) are a function of the input, the JSON snapshots are pinned, and the two baselines stay comparable. Each later step compares against the same file. The harness keeps its CSV columns as an interface (new columns only at the end) and the target script runs unchanged on a new engine or data model. The no-slowdown promise is mechanical: the target set runs on pinned cores and compares the counters exactly and cpu_ms (or instruction counts) within a tolerance against a committed baseline, with the callgrind recipe of bench.md for differences of a few percent.
- Met when: `bench/targets.sh LABEL` and the comparison named in CLAUDE.md's verification table with `linlog-bench summary` side by side against after-bias.csv, the counters equal for every decided row; bench/RESULTS.md and COMPARISON.md unchanged in shape; a rule in bench.md that new columns are appended.
- State now: Partly: the target set, its committed counters and the verification table exist, but the requirement says the promise is mechanical, and nothing compares the counters automatically (grep for nodes and memo_hits finds none in bench/src/summary.rs or compare.rs; `ratchet` counts the instructions of the journeys, not the target set's counters; R213 asks summary to print them). Evidence: bench/targets/after-bias.csv and the other targets/*.csv exist; the CLAUDE.md verification table; .claude/rules/bench.md "A configuration axis (a new Options knob)" lists the appended-column discipline and :494-509 describes the callgrind recipe. bench/src/compare.rs compares two sets of rows (times, verdicts) but not the counters automatically.

### R240. The flake pins Yalla's standalone intuitionistic kernel and checks the bridge modules
- Steps: 31 Rocq library (the third stage).
- Sources: plan/31-rocq-library.md:40-44; plan/later.md:398-406,415-417.
- API item: modules/rocq.nix, the inputs of flake.nix, the bridge modules under rocq/.
- Requirement: The library's bridge modules are optional and not needed to check a certificate. They import the pinned NanoYalla and Yalla's standalone `nanoill.v` and prove that the library's classical calculus derives the same sequents as NanoYalla's `ll`, and its ILL the same as `nanoill.v`. The flake fetches both at pinned commits (flake.nix has only the `nanoyalla` input, the Click & coLLecT directory, today; the second input must name the commit that holds `nanoill.v`) and never copies them into the tree, since they are LGPL and the bridges only import them when they are checked (later.md:415-417). For Mix the sources differ: plan/31-rocq-library.md:42-44 says Yalla proves the reduction to `?(⊥⊗⊥), Γ` without cut (`mix2_to_ll` in `ll_fragments.v`) and that it can be followed, while later.md:402-406 calls it sketched on paper and not machine-checked. The step decides, and the report records it: the reduction is a theorem in the bridges, or it is paper-only. For affine there is no reduction and the weakening rule is the definition. The opam package does not depend on the bridges, and the package and the certificate check of R200 build without them.
- Met when: A flake check compiles the bridge modules against the pinned inputs and `Print Assumptions` of each bridge theorem is empty; the check of R200 and the opam file of R162 need neither the second input nor the bridges; the Mix theorem exists or the report says paper-only.
- State now: Not met. Evidence: flake.nix:39-45 pins `nanoyalla` only; modules/rocq.nix builds NanoYalla alone; there is no rocq/ directory.

## Docs

### R217. linlog's own claims in the feature matrix are stated once from the code
- Steps: 29 comparison.
- Sources: plan/29-comparison.md:24-30.
- API item: README "What exists and what is planned", cli --help, the Cargo features, plan/notes/comparison.md rows.
- Requirement: The matrix rows for linlog (fragments, modes, automatic against interactive, proofs, proof nets, checker, certificates, exports, limits, threads, input formats, licence, activity) are stated once from the code and verified, with a source and a date per cell. The facts the library owns (the fragment and mode list, the input formats lltp, tptp, spec, json, jsonl, lines and problems, the export formats, the limits of time, memory and copies, feature flags such as parallel) are listed in the rustdoc or README, so that the matrix cites them rather than re-deriving them.
- Met when: A single documentation table of features and flags in README or rustdoc, checked by cli/tests/readme.rs where it prints output; bench/TOOLS.md cites it.
- State now: Partly. Evidence: core/src/lib.rs crate documentation lists features and exports; cli/src/argument_parsing.rs InputFormat and --help list the formats; no single place lists the fragments and modes supported per engine except .claude/rules and README (not read in full).

### R218. The manifests carry the metadata crates.io and the release need
- Steps: 30 release (D22).
- Sources: plan/30-baseline-release.md:39-40; plan/later.md:31-35; plan/README.md:752-778; plan/notes/distribution.md (crates.io section).
- API item: core/Cargo.toml and cli/Cargo.toml package metadata, crate documentation, README, licence headers.
- Requirement: Both published crates (linlog 0.1.0 and linlog-cli) carry the metadata crates.io needs: repository and homepage or documentation (github.com/linlog-prover/linlog, final because they cannot be changed after publication), readme, keywords, categories, rust-version (the measured value of R252) and the AI-assistance statement, and a licence file inside each crate directory. The `license` of linlog-cli is an SPDX expression that covers the Euler Math font it embeds (expected `EUPL-1.2 AND OFL-1.1`), with cli/fonts/OFL.txt inside the package. The set of cargo features of linlog is fixed before the tag and documented as kept, since removing a feature is a semver break and adding one is not; the audit settles and records whether `parse` should be split from lltp, mist, families and ordinary::read_tptp (plan/notes/research/30-release.md, section 1; section 2, item 8; section 3, Features). The feature list in the crate documentation matches `[features]`, and no behaviour (clock, threads, files) hides behind default features that wasm and library users would not expect. bench stays unpublished (publish = false). Items at the API's edge decided in step 28 (non_exhaustive, the public surface) are what semver then protects.
- Met when: `cargo publish --workspace --dry-run` succeeds; `cargo package --list` for linlog and linlog-cli shows LICENSE and the readme; a grep of the manifests shows the fields; the documentation feature paragraph in lib.rs is verified against `[features]`; the license of cli/Cargo.toml contains OFL-1.1 and `cargo package --list -p linlog-cli` shows cli/fonts/OFL.txt; the feature paragraph of core/src/lib.rs lists exactly the ten features of [features] and says that none will be removed within 0.1.
- State now: Not met. Evidence: core/Cargo.toml:1-9 and cli/Cargo.toml set name, description, version, edition and license only (no repository, readme, keywords, categories or rust-version); `ls core/LICENSE cli/LICENSE` finds nothing (LICENSE exists only at the root); core/src/lib.rs:80-91 documents the features; bench/Cargo.toml:10 has `publish = false`; cargo publish is denied in .claude/settings.json by design (claude-infra.md).

### R219. linlog-cli has a description of its own
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40; plan/notes/distribution.md (crates.io section).
- API item: the linlog-cli package description.
- Requirement: linlog-cli gets a description of its own (the command-line prover) instead of the library's sentence, because it is the text crates.io shows.
- Met when: cli/Cargo.toml's description differs from core/Cargo.toml's and names the command.
- State now: Not met. Evidence: core/Cargo.toml:6 and cli/Cargo.toml (about line 6) both read "A linear logic suite for all your needs."

### R220. docs.rs builds the library with every feature and shows which feature each item needs
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40; plan/notes/distribution.md (crates.io section); plan/notes/research/30-release.md (section 1, docs.rs; section 2, item 5); .claude/rules/core.md:100-104.
- API item: [package.metadata.docs.rs] of linlog; the "Needs the cargo feature" lines of the public items.
- Requirement: docs.rs builds the library with all features (parallel, png and pdf included, which must build without network) on one target (x86_64-unknown-linux-gnu: nothing in the API depends on the target, and without `targets` docs.rs builds five). docs.rs passes `--cfg docsrs` itself. Every feature-gated public item says which feature it needs, by the hand-written lines core.md requires; an automatic badge is added only by a mechanism that builds on the current nightly of docs.rs. In 2025 `doc_auto_cfg` was folded into `doc_cfg` (rust-lang/rust#138907), and crates that had enabled the old feature under cfg(docsrs) stopped building; the crates that kept badges moved to a crate-specific cfg passed through `rustdoc-args`. The doc examples gated by cfg_attr on parse and serialize run or are ignored correctly in that build.
- Met when: core/Cargo.toml has [package.metadata.docs.rs] with all-features and the target list; a build with a nightly rustdoc, offline, with all features and RUSTDOCFLAGS="--cfg docsrs" is free of warnings (done once before the tag and reported); the flake doc check builds with --all-features; every "Needs the cargo feature" line is checked against [features].
- State now: Not met. Evidence: grep for docs.rs, docsrs and doc(cfg in the Cargo.toml files and core/src finds nothing; the flake's doc check (modules/workspace.nix) builds rustdoc for the default features, so docs.rs's view of parallel, png and pdf is unchecked; the hand-written lines exist (core/src/errors/parse.rs:8, core/src/search/parallel.rs:53, core/src/lltp.rs:31).

### R221. A changelog states the public surface of the first release
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40,54-55.
- API item: new CHANGELOG.md.
- Requirement: A changelog at the repository root for 0.1.0 states the public surface of the first release (features, command-line commands and statuses, JSON wire forms), so that later steps append to it.
- Met when: CHANGELOG.md exists, has a 0.1.0 section, and is listed in the crate package.
- State now: Not met. Evidence: ls CHANGELOG* finds nothing at the repository root.

### R222. A CITATION.cff exists on the default branch
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40; plan/notes/distribution.md (crates.io: CITATION.cff).
- API item: new CITATION.cff.
- Requirement: CITATION.cff on the default branch has version 0.1.0, EUPL-1.2, the repository address github.com/linlog-prover/linlog, and the AI-use statement that the notes say README and Zenodo carry.
- Met when: CITATION.cff exists and validates (cffconv or GitHub's Cite panel).
- State now: Not met. Evidence: ls CITATION* finds nothing.

### R223. Every address written is the final one
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-46; plan/notes/distribution.md.
- API item: every address written (manifests, CITATION.cff, opam, README, CHANGELOG, --help text).
- Requirement: All addresses are github.com/linlog-prover/linlog and linlog-prover.github.io/linlog, with none left at flgrubm, because the manifest repository field is immutable once published.
- Met when: grep -r flgrubm over tracked files finds no address (the author's name in headers and the commit identity excepted).
- State now: Partly. Evidence: README.md:8 and :13 and CLAUDE.md:126 already use linlog-prover; the manifests have no address yet.

### R224. README is the face of the release: an install section, a first proof, absolute links
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:54-55.
- API item: README.md.
- Requirement: README gains an installation section (`cargo install linlog-cli` giving the binary linlog, nix run and build, the Rocq library) and a first proof near the top, and its links and images are absolute, because crates.io and docs.rs render the readme out of the repository and relative paths break. Any new console example stays a tested block (cli/tests/readme.rs).
- Met when: README has Install and First proof sections; no relative image link; `cargo test --workspace` passes cli/tests/readme.rs.
- State now: Partly. Evidence: README.md:729-730 use the relative images core/tests/snapshots/ill.svg and net.svg; grep -in 'install' README.md finds only the NanoYalla installation at :760; README.md:13 shows nix run but no cargo install; the readme test is cli/tests/readme.rs.

### R225. The rules machinery covers the Rocq library directory
- Steps: 31 Rocq library (items 1 and 2).
- Sources: plan/31-rocq-library.md:28.
- API item: the new rocq/ directory, .claude/rules/rocq.md, the CLAUDE.md rules table and conventions, .claude/rules/flake.md.
- Requirement: A rules file for `rocq/**` holds the library's invariants and where a new rule or calculus plugs in. The CLAUDE.md table and Conventions name it. The licence-header convention is extended to Rocq files (CLAUDE.md lists `//` and `#` only), as are the documentation-comment rule (rocqdoc) and the formatter question. The opam file carries github.com/linlog-prover/linlog addresses. The flake.md and claude-infra.md rules mention the new modules. Version control and the "Never run git" rules are unchanged.
- Met when: .claude/rules/rocq.md exists with paths `rocq/**`; CLAUDE.md's table has the row and the header rule says what a .v file starts with; the claude-hooks flake check still passes.
- State now: Not met. Evidence: ls rocq fails; .claude/rules/ lists bench, ci, claude-infra, cli, core*, flake only; CLAUDE.md Conventions covers headers for `//` and `#` only.

### R226. The web crate and its wasm check have their rules file and entries
- Steps: 32 web client.
- Sources: plan/32-web.md:66-69,83-87.
- API item: .claude/rules/*.md, CLAUDE.md (workspace crates, commands table), README usage.
- Requirement: The new crate and the wasm check need their rules file and entries (a web.md rules file with paths for linlog-web, the verification-table row for touching wasm-relevant code, the flake rules for the wasm check), and the rustdoc says which features and bounds the client ships, so the step needs no rewrite of the documentation structure.
- Met when: .claude/rules/web.md and the CLAUDE.md table are amended in the same change as the crate; claude-infra.md lists it.
- State now: Partly. Evidence: .claude/rules/claude-infra.md:177 and CLAUDE.md already mention "the planned linlog-web crate" as a trigger to amend .claude/; no web.md exists in .claude/rules/.

### R227. Every statement that nets exist only for unit-free MLL is updated together
- Steps: 33 MELL nets (D6a).
- Sources: plan/33-mell-nets.md:10-12; CLAUDE.md conventions; plan/README.md D6a.
- API item: .claude/rules/core-nets.md, README, CLAUDE.md, the rustdoc of nets.
- Requirement: Every statement that nets exist only for unit-free MLL is updated together: the module documentation of nets, ProofStructure and the NetFragment and NetMode documentation, core-nets.md (including the choice of ? nodes and why weakening is checkable), README "What exists", the --net help, and plan D6a, in the same changes as the code.
- Met when: grep for "MLL" near "proof net" in core/src/nets, errors, cli and README finds no stale claim; core-nets.md has the new invariants as bullets.
- State now: Not met. Evidence: core/src/nets/mod.rs:4 "Proof nets for unit-free MLL"; .claude/rules/core-nets.md:15 "proof-net model for unit-free MLL"; cli/src/argument_parsing.rs:777.

### R228. The cut is documented in the rustdoc and the rules files
- Steps: 34 cut (sessions 1 to 3).
- Sources: plan/34-cut.md:3-5.
- API item: rustdoc and .claude/rules.
- Requirement: Every public item of the cut gets a documentation comment (the workspace lints enforce it), the JSON forms are described on their types (Proof, Interactive, ProofStructure, the elimination value), and the rules files core-forest.md, core-proofs.md, core-derivations.md, core-nets.md and core-export.md record what a later session cannot see in the code: cut roots are appended, the Node size, the checker's cut rule, the contexts of ⊤ in elimination. `RUSTFLAGS="-W unreachable_pub -W unnameable_types" cargo check -p linlog --all-features` stays clean.
- Met when: `cargo clippy --workspace --all-targets -- --deny warnings` passes (missing_docs); the diff of .claude/rules/*.md is in the same jj commits as the code.
- State now: Not met. Evidence: grep -i cut .claude/rules/core-*.md finds nothing about a cut rule.

### R229. The engine documentation carries each new or changed row with its measurement
- Steps: 35 routing feature and essential nets.
- Sources: plan/35-mll-engines.md:28-41.
- API item: the Engine rustdoc table "Which engine decides a goal", the README D8 table, .claude/rules/core-search.md and core-nets.md.
- Requirement: Each new or changed row carries its measurement in the Engine documentation table (the rustdoc is the manual). The README usage and dispatch text change in the same commit as any printed output (cli/tests/readme.rs runs the examples), and the "Where it loses" paragraph of core-nets.md and the "Rows now" list of core-search.md are rewritten.
- Met when: Engine documentation table rows; `cargo test --workspace` passes readme.rs; the rules files are amended.
- State now: Not met. Evidence: core/src/search/mod.rs:654-660 the table; .claude/rules/core-search.md:125-135 "Rows now"; .claude/rules/core-nets.md "Where it loses" says dispatch routes multiplicity above 2 to focus.

### R230. plan/notes/api.md records the room each later step needs
- Steps: 28 audit (stage 2); 34 cut; 36 Lambek; 38 first-order.
- Sources: plan/34-cut.md:29-31; plan/36-lambek.md:3-5; plan/38-first-order.md:9-13,39-40; plan/28-audit-and-refactor.md:398-415.
- API item: plan/notes/api.md, plan/first-order/README.md, the .claude/rules/core-*.md files, the rustdoc.
- Requirement: Steps 34, 36 and 38 start from plan/notes/api.md, which step 28 owes and which does not exist. It records the arena, symbol-table, proof-term and wire-form decisions listed in this register: where terms and binders go (a design for the quantifier extension, with the cut roots laid out so that they have room for a binder table), the register of room-for-change for the ordered mode (the canonical sequent form, the Mode value and its JSON, the Rule set, the dispatch rows), the member type of a sequent (R244) and the zone and frame types of the focused engine (Context, Classes, the memo Key), and the decisions of D1, D5, D6 and D17. The rules files that state "optimize sorts the roots", "the sequent is ids in ascending order" and "a Mode has three flags" are updated when the ordered mode lands, and the rules files record the first-order invariants one point per bullet (core-sequents, core-forest, core-proofs, core-focus, core-nets, core-search, core-export), with every new public item's rustdoc and JSON form documented on its type. The report of step 34 has a section "Where quantifiers go" naming the types.
- Met when: plan/notes/api.md exists and names each decision; plan/reports/28-audit-and-refactor.md checks it off; the rules files no longer state the sorted-root invariant unconditionally; the rules files are updated in the same commits as the code.
- State now: Not met (the part about the cut's quantifier room is unknown until step 34). Evidence: ls plan/notes shows comparison.md, distribution.md, export-targets.md and lltp-headers(.md) only; plan/reports/28-audit-and-refactor.md:46 lists "2.2 ... plan/notes/api.md" as open; .claude/rules/core-sequents.md states "optimize() ... sorts roots"; Term (core/src/sequents/term.rs:52-78) has no quantifier or argument and Atom is a bare u32; plan/reports/31 does not exist yet.
- Conflict: see Conflicts for the author, C1.

### R231. The inverse engine is recorded in the documentation in the same commit
- Steps: 37 inverse method.
- Sources: plan/37-inverse.md:17-20,26-29.
- API item: the Engine rustdoc table, lib.rs crate documentation, the README engine paragraph, .claude/rules/core-search.md, and a new core-inverse.md.
- Requirement: The documentation that is the library's manual records the engine in the same commit: the Engine::Inverse variant documentation (what it saturates, its subsumption, termination on MALL and non-termination with exponentials, the options it reads), the dispatch table row or the sentence that it is option-only and why, the README's engines text, and a `.claude/rules/core-inverse.md` (with its row in core.md's table and `paths`) saying what a later session cannot see in the code.
- Met when: RUSTFLAGS unreachable_pub and unnameable_types are clean; rustdoc builds; core.md lists the new rules file; README console blocks still pass cli/tests/readme.rs.
- State now: Not met. Evidence: .claude/rules/ has core-search.md, core-focus.md, core-horn.md, core-nets.md and others but no inverse file; core.md's module table (lines 12-26) lists thirteen module files.

### R253. The status of linlog_cli under semver is decided and written
- Steps: 30 release (D18, D22); 28 audit (the surface).
- Sources: plan/notes/research/30-release.md (section 3, linlog_cli; section 4, open question 3); .claude/rules/cli.md:20.
- API item: linlog_cli (cli/src/lib.rs and its public modules), cli.md, R215.
- Requirement: linlog-cli publishes its library `linlog_cli`, so cargo's semver rules apply to every public item of it. Before the tag the crate says which it is: an internal library that carries no promise (a sentence in the crate documentation, a public surface cut to what main.rs and the tests use, and the check of R215 naming only linlog), or an API reviewed under R50. The internal library is recommended.
- Met when: The crate documentation of cli/src/lib.rs states the choice, cli.md records it, and the check of R215 lists the crates it covers.
- State now: Not met. Evidence: cli/Cargo.toml [lib] name = "linlog_cli"; cli/src/lib.rs declares argument_parsing, batch, interact, io, ordinary, prove and style as pub mod and says it is a library "so that this documentation exists"; neither it nor cli.md says anything about semver.

## Other

### R232. The packaged crates build and reference nothing outside their own directory
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40 (a dry run of publishing).
- API item: packaged crate contents: tests and include_str outside the crate directory.
- Requirement: The packaged archives build and do not reference files outside their own directory in the code that cargo publish verifies. Tests that read ../../README.md, ../cli/fonts or the repository's snapshots are either excluded from the package with exclude or travel with it. The order is linlog, then linlog-cli (a path plus version dependency); linlog-bench stays unpublished.
- Met when: `cargo publish --workspace --dry-run` passes; `cargo package --list` shows no dangling test; bench/Cargo.toml keeps publish = false.
- State now: Partly. Evidence: cli/tests/readme.rs:31 include_str!("../../README.md"); core/tests/export.rs:542 joins CARGO_MANIFEST_DIR/../cli/fonts/Euler-Math.otf; cli/src/prove.rs:786 include_bytes!("../fonts/Euler-Math.otf") is inside the crate (fine); bench/Cargo.toml has publish = false; cli depends on linlog with version = "0.1.0" and path; no exclude or include in any manifest.

### R233. A release workflow that publishes with Trusted Publishing
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40; .claude/rules/ci.md.
- API item: new .github/workflows/release.yml.
- Requirement: A release workflow triggered by a version tag builds with Nix only, publishes with Trusted Publishing (id-token write, a short-lived token), attaches the archives, and follows the CI rules: actions pinned to full commit SHAs, least-privilege permissions, actionlint-clean. The first publication needs a token, so the workflow cannot be tried before the author's act.
- Met when: `nix flake check` (actionlint) passes with the file present; every `uses:` carries a SHA and a version comment.
- State now: Not met. Evidence: ls .github/workflows shows only ci.yml and docs.yml.

### R234. The crate-wide dead-code allowance is gone or narrowed before 0.1.0
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40; CLAUDE.md (dead_code allowance).
- API item: core/src/lib.rs `#![allow(dead_code)]` and `#![allow(unused_variables)]`.
- Requirement: The crate-wide dead-code allowance, which CLAUDE.md says the audit before the release sorts out, is gone or narrowed before 0.1.0 is cut, so that published code carries no unexplained dead items.
- Met when: `cargo clippy --workspace --all-targets --deny warnings` passes with the two allows removed or reduced to named items with a comment.
- State now: Not met. Evidence: core/src/lib.rs:93-94 still carry `#![allow(dead_code)]` and `#![allow(unused_variables)]`.

### R235. The version 0.1.0 is single-sourced
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40; plan/notes/distribution.md.
- API item: Cargo.toml [workspace.package] version and the per-crate versions.
- Requirement: Version 0.1.0 is single-sourced, so that the tag, the manifests, the path-dependency version requirements, CITATION.cff and the flake's pname and version agree.
- Met when: All crates inherit `version.workspace = true`, or a check compares them; `cargo metadata` prints 0.1.0 for all three; the flake build reports 0.1.0.
- State now: Partly. Evidence: Cargo.toml [workspace.package] has version = "0.1.0", but core/Cargo.toml:7 and cli/Cargo.toml set version = "0.1.0" literally, and `linlog = { version = "0.1.0", path = ... }` repeats it in cli and bench; modules/bench.nix:31 takes pname and version from bench/Cargo.toml.

### R236. The dry run of publishing is a flake check
- Steps: 30 release.
- Sources: plan/30-baseline-release.md:39-40 (a dry run of publishing, flake checks).
- API item: a new flake check for cargo package or cargo publish --dry-run.
- Requirement: The dry run is a flake check or a documented command, so that every later metadata or file change keeps the packages publishable, and the tag does not meet the first failure.
- Met when: modules/checks.nix has a package check that runs `cargo package --workspace` (or the publish dry run) offline, or .claude/rules/flake.md documents the command.
- State now: Not met. Evidence: grep for package and publish in modules/*.nix finds no such check; modules/checks.nix:34-76 has build, doc, features and the others.

### R237. The ordered sequent and the mode are the input type of a later translation to first-order MILL
- Steps: 36 Lambek; 38 first-order.
- Sources: plan/36-lambek.md:27-29.
- API item: the embedding of Lambek into first-order MILL (Moot and Piazza).
- Requirement: The step builds the planar search only, but the library keeps room for the other route: an ordered sequent kept in written order (R53) and a mode value (R51) that a later translation layer (like core/src/ordinary's translations of ordinary logic) can accept as input, so that the planar and the embedded decision can be compared on the same inputs once step 38 exists. No code is required now beyond the order-preserving sequent and the mode.
- Met when: The ordered Sequent and Mode exist and are the translation's input type; documented in .claude/rules/core-ordinary.md or the report.
- State now: Unknown. Evidence: core/src/ordinary/translate.rs exists as the model of a translation layer; there are no first-order terms in Term (core/src/sequents/term.rs:52-77).

### R238. The checker's tables can face untrusted input with a seeded hasher
- Steps: 28 audit.
- Sources: plan/later.md:957-960.
- API item: hash::BuildHasher (foldhash FixedState) in the proofs::check tables.
- Requirement: The checker's tables face untrusted input with a fixed hash seed. A second, seeded hasher for the checker (not for the reproducible search tables) removes hash-flooding as a way to make a proof file expensive, if that stays in scope.
- Met when: check.rs uses a different hasher type than the engines, with the choice recorded in core-proofs.md; or the decision to defer is recorded.
- State now: Not met. Evidence: core/src/hash.rs:3-12 is a fixed-seed FixedState for all tables; proofs/check.rs:37 `use crate::hash::{HashMap, HashSet}`.

### R239. linlog-web is a workspace member and its wasm build is a flake package
- Steps: 32 web client (D22).
- Sources: plan/32-web.md:28-36; plan/README.md:752-778.
- API item: the new crate linlog-web, the Cargo.toml workspace members, a flake output for the wasm bundle.
- Requirement: There is room for a workspace member linlog-web (JSON in, JSON or SVG out, built for wasm32-unknown-unknown without parallel). The client repository takes this one as a pinned flake input, so the wasm build is a flake package (and check) the client can consume, versioned with the library, built and tested in the flake check of R41.
- Met when: `nix build .#linlog-web` yields the wasm package, the client's flake input uses it, and Cargo.toml `members` lists the crate.
- State now: Not met. Evidence: Cargo.toml:6 `members = ["bench", "cli", "core"]`; modules/ has bench, checks, claude-hooks, devshell, export, rocq, systems, toolchain, treefmt, workspace only.

### R252. The minimum toolchain is measured, held by a check and stated
- Steps: 30 release (D22).
- Sources: plan/notes/research/30-release.md (section 1, MSRV; section 2, item 4; section 4, Risks and The prompt should add).
- API item: rust-version in [workspace.package], a flake check, README, CHANGELOG.
- Requirement: `rust-version` in [workspace.package], inherited by linlog and linlog-cli, is the lowest toolchain on which the library with all features and the command build with the committed Cargo.lock, measured with cargo-msrv or `cargo hack --version-range` and not guessed: edition 2024 gives 1.85, the let chains the code uses give 1.88, and the dependencies may need more. A flake check builds with that toolchain, so that a Cargo update or a new language feature cannot raise the minimum unnoticed. README states the policy (for example the latest stable minus N, raised only in a 0.y bump), and the changelog names each raise.
- Met when: rust-version is set and the report records the measurement; a flake check fails when the code needs a newer toolchain than rust-version; README has the policy.
- State now: Not met. Evidence: neither manifest nor [workspace.package] has rust-version; rust-toolchain.toml says channel = "stable"; let chains are used (core/src/search/focus/mod.rs:285, core/src/search/horn/reach.rs:340); grep for rust-version and msrv in modules/ finds nothing.

## Conflicts for the author

The second review found three questions that the register cannot settle, because the plan's own text, the research notes and the decisions point different ways. Each subsection names the entries it concerns, the options and the recommended answer; the entries concerned carry a line that points here. Until the author answers, the entries keep their present text.

### C1. When the written order of the roots becomes the canonical form of Sequent
- Entries: R53 (the sequent keeps the written order of its roots), R54 (the top and zero ambiguity), R230 (plan/notes/api.md and the rules files).
- The question: R53 and R230 leave it open whether the change happens at step 28 or at step 36. plan/36-lambek.md:24-27 fixes only that the order is kept for the ordered modes, not when. plan/later.md:78-79 ("the pinned snapshots do not change") and D18 apply to the audit, D5 says that the order of the roots is the order of the term ids, and plan/notes/research/README.md (section 3, conflict 2) and plan/notes/research/36-lambek.md (sections 3 and 4) recommend doing it at step 28 in one commit. The cost is smaller than it first seems: the parser lowers in text order, so the roots are already ascending unless a later root is shared with, or is a subterm of, an earlier one (as in `a ⅋ b, a`), and a file that carries its own sequent keeps its ids once the reader stops sorting. R6, R69 and R216 need no change beyond an explicit exemption for the sequents whose root order changes.
- Option A: At step 28, before 0.1.0, one commit stops sorting, so that the roots stay as written in every mode. It regenerates the snapshots and README blocks whose roots were not already ascending, and the audit report states the exemption from plan/later.md:78-79 for exactly those. Later steps (the written succedent of step 31 in R54, step 36) then add no format break.
- Option B: At step 36, after the release. Step 28 only leaves the room (R230), and the change is a versioned break (D18, R6) that reports its cost; stored files are unaffected because they carry their sequent.
- Option C: The order depends on the mode. The parser or Sequent::optimize takes the mode and sorts only in the commutative modes. Commutative output never changes, but a Sequent then means different things depending on how it was built, and a mismatch of modes gives a wrong verdict silently, since today the parser calls optimize with no mode (core/src/parse/mod.rs:412).
- In every option R53's alternative "the order is not wanted and is documented as such" is dropped, because plan/36-lambek.md:24-27 says the order has to be kept.
- Recommended: Option A. The type should hold what the user wrote and the mode should interpret it; the sort is only a convenience of the arena. The affected inputs are few, doing it before 0.1.0 costs one reviewed commit instead of a version, and it removes the top and zero ambiguity of R54 at its source. Option C is the weakest, because Sequent, Forest and Reading would need the mode before they are safe to use.
- If A is chosen, R53 becomes "The sequent keeps the written order of its roots" (steps 28 audit and 36 Lambek; sources as now plus plan/notes/research/36-lambek.md, sections 3 and 4, and plan/notes/research/README.md, section 3, conflict 2), with this requirement: the written order of the roots is the canonical form of Sequent in every mode, so that parse, the JSON form, Sequent::add, Display and Forest::roots() keep the roots as given, and optimize hash-conses and drops unreachable terms but does not sort. Arena indices are assigned in the order of first occurrence along the roots. The change lands in one commit before 0.1.0, which regenerates the pinned snapshots and README blocks whose roots were not already ascending and names them in the audit report, as the one exemption from plan/later.md:78-79 and D18. The counters of the target set (R216) are compared before and after, and every row whose roots changed order is listed. The decision also states whether equality and hash of a cyclic sequent are up to rotation and whether Forest::roots() of a cyclic sequent is the written rotation or the canonical one, and it settles the top and zero succedent of R54 (`0, top |- top` keeps its written goal) or documents that it does not. The cost (renumbered ids, memo and cache keys, the places that compare sequents for equality) is reported. It is met when tests parse `|- A, B, C` and `|- B, A, C` and get different roots in every mode, `|- A, B, C` and `|- B, C, A` are equal in the cyclic mode (or the decision says otherwise), Forest::roots() equals the written order, the JSON round trip keeps the order and a file written before the change still loads with its ids, the regenerated snapshots are pinned again, after-bias.csv is equal on every row whose roots did not change, and core-sequents.md and core-forest.md no longer say that optimize sorts the roots (R230).
- If B is chosen, R53 keeps its steps 28 and 36 but says that step 28 only records the room (R230) and that the shift is reported at step 36 as a version.

### C2. Where the search for a falsifying assignment lives, and whether a refuter may decide an unknown
- Entries: R20 (the search for a falsifying assignment), R72 (countermodel values), R113 (an entry for deciding ordinary logic), R153 (the refutation writer), R154 (the ordinary "not valid" certificate).
- The question: plan/31-rocq-library.md:66-92 (item 8) puts the search in the exporter, with its bound in export::rocq::Options, run when the search has answered Unprovable; it never turns Unknown into Unprovable. The research (plan/notes/research/refutations.md, section 3, Engine interface and dispatch, and Options; plan/notes/research/README.md, section 3, conflict 8) wants it in the library behind search::Options, so that the command, the web client and the JSON outcome show the same reason (D15), and wants a refuter to be able to turn Unknown into Unprovable: the 35 ILTP non-theorems that end in Reason::CopyBound and 19 LLTP files would be decided. The promotion is logically safe, since a classical falsifying assignment refutes the sequent in every mode. prove_goal is the one place where an answer becomes a Verdict (search/mod.rs:258-267), and core-search.md:42-45 says that Unprovable comes only after an exhaustive search. R20, which this review took in the library form, R72, R113 and R124 each cover a piece, but none says where a refuter runs, under which stop and memory account, and that it never overturns a verdict. The batch needs no change in core, since search::batch::Answer already carries the whole Outcome (batch.rs:153-158).
- Option A: Keep the search in the exporter as step 31 words it. The command and the web client get no assignment, and no unknown is decided.
- Option B: A library function run by prove_goal after an Unprovable only, which attaches a checked Refutation::Classical; the exporter then consumes the Refutation.
- Option C: Option B, plus a run after an Unknown, behind an option that is off by default.
- Recommended: Option C, with the run after an Unknown off by default. D15 asks for one options value shared by the command, the web client and the wrappers, which option A cannot give; Refutation::check (R124) lives in the library, so the certificate should too; and with the option off the verdicts of the baselines stay as they are (D16), which the comparison needs, since steps 29 and 37 count a bounded failure as unknown. A refuter never changes a verdict the search gave. A table or trait of refuters waits until a second refuter exists.
- If C is chosen, a new entry is added: a refuter runs after the search, under its stop and memory account, and never overturns a verdict (steps 31, the later refuters of R72 and R112, and 28 for the place in prove_goal; D15, D16, D19). The front door has one place where a refuter runs after the search: after an Unprovable, to replace or complete its Refutation by a checkable one, and, behind an option that is off by default, after an Unknown, to decide it. The refuter runs under the caller's stop and the search's memory account, with its bounds (atoms, work) as Options fields with named defaults and flags, no clock and no thread, so that it works on wasm. A refuter never changes a Proved verdict or an Unprovable one the search gave, and turns Unknown into Unprovable only with a Refutation that Refutation::check accepts (a debug assertion in every build, as for proofs). The Outcome JSON, the command and the batch show the refutation, and the exporter consumes it and no longer searches. It is met when a search function in the library has a fixed enumeration order and its bounds in the Options JSON (R1) and as flags; a test shows that an Unknown(CopyBound) non-theorem with a falsifying assignment becomes Unprovable only with the option set and checks, that a theorem's verdict never changes, and that the stop interrupts a search over 2^30 assignments; a Classical refutation is pinned in core/tests/serialize.rs; and with the option off the counters and verdicts of the target set equal after-bias.csv. R20 then says that export::rocq::Options carries no bound of its own, and R72 and R113 say that a second procedure or countermodel plugs into the same place.

### C3. Whether nullary Mix exists
- Entries: R78 (the canonical net form of weakening), R94 (a Mix that opens an empty goal), R117 (the proof term and the checker are frozen).
- The question: With binary Mix only, no rule concludes the empty sequent (NetError::Empty, nets/mod.rs:60-66), so a criterion for weakening and bottom needs a placement: a criterion without one is only necessary and, with units, NP-hard (plan/notes/research/33-mell-nets.md, sections 1, 2 and 4). plan/README.md:830 and :980 left nullary Mix open "until something needs them". Nullary Mix is a rule of the calculus, the checker, the oracle and the Rocq mirror, which R117 freezes at step 31, so the author must answer before step 31 starts, and the choice of placement at step 33 depends on the answer.
- Option A: No nullary Mix; weakening and bottom are placed by a jump, chosen canonically from the term. The calculus and the Rocq mirror stay unchanged, and the net carries a control structure.
- Option B: Nullary Mix in the Mix mode. The criterion becomes acyclicity alone and weakening is simpler, but Node, the checker, oracle.rs and the Rocq development gain a rule, and the empty goal of R94 becomes closable.
- Recommended: Option A for 0.1.0 and step 33, since the release and the Rocq library are measured against the calculus as it is, and option B can come afterwards as an additive Node variant. The question is reopened only if the brute-force comparison of R78 shows that jumps cannot be made exact.

## Not merged

The following pairs or groups ask something close to each other but were kept apart on purpose, because they ask different things of different items. In the entries that share sources the overlap is named in the text.

- R42 (no clock or thread) and R43 (no recursion over new structures) share the sources for steps 34, 37 and 38, because those sources state both rules in one sentence. The two rules are checked by different tests (a grep or lint for R42, depth.rs for R43), so they stay separate.
- R45 (a safe recursion limit for a given stack) and R43 are kept apart: the first concerns the focused engine's own recursion depth and its limit, the second the walks that must not recurse at all.
- R50 (non-exhaustive enums) and the error entries R129, R131, R132, R134, R136 are kept apart: R50 holds the one decision on what is non-exhaustive, while the error entries hold the variants, messages and wire forms that each step adds. R129 and R135 both mention the same enum and point back to R50.
- R57 (where cut formulas live in the forest), R58 (building the dual of a term), R119 (the Cut node), R120 (the checker's cut rule) and R160 (the Rocq cut) all come from the cut step and from step 31's room for cut. They concern different items (Forest, the arena, Node, the checker, the exporter) and stay separate.
- R58 (a crate-private dual construction) and R59 (a public validated builder) are kept apart: the first is an internal function for cut formulas, the second a public API for any caller. One builder may satisfy both, and the design may merge them.
- R76 (boxes), R77 (vertices beyond the forest), R78 (the ? node form) and R79 (the calculus descriptor) all describe growth of ProofStructure for step 33 and later. They are kept apart because each is a separate design decision with its own test.
- R8 (the net JSON form) and R79 (the descriptor taken by the constructor and the wire form) overlap in the wire part. R8 keeps the compatibility rule, R79 the constructor.
- R54 (Reading and the top and zero ambiguity), R53 (the sequent keeps the written root order) and R55 (the ordered antecedent) concern the same area. R53 and R54 share the ambiguity: the decision of R53 settles R54 if the written order is kept. They stay separate because different steps need them for different reasons.
- R1 (the options JSON) and R142 (presets) and R183 (the command reads an options file) are kept apart: the wire form, the named presets and the command's file flag are three deliverables.
- R21 (the work budget) and R30 (every engine polls at a bounded interval) are kept apart: the budget is a new option, the polling is the existing contract.
- R22 (the table of unpolled calls) and R23, R24, R25, R26 (the checker, the ordinary read-back, the net calls and the net drawing) are kept apart: R22 is the one list and the others are the specific calls that need a stop or a bound.
- R152 (the Rocq writer for a Proof) and R199 (the cross-check corpus of unchecked terms) both need a writer that does not check. They stay separate because the second is a test deliverable with its own fixture.
- R153 (the refutation writer) and R154 (the ordinary "not valid" writer) share R20 (the assignment search); they are separate items, one for the linear calculus and one for ordinary logic.
- R85 (the shared race) and R184 (time counted alike) both come from the comparison step. The first is the code that must exist once; the second is how time is measured.
- R194 (the CSV stays comparable) and R216 (the target set as the regression oracle) both protect old results but compare different files: the baseline CSV and the target-set CSV.
- R216 and R206 (a target set for the net engine) are kept apart: R216 is the existing oracle for the focused engine, R206 asks for a new one.
- R233 (the release workflow), R236 (the package dry-run check) and R232 (the packaged contents) concern the same release but different files (a workflow, a flake check, the manifests and tests).
- R79 (the calculus descriptor), R104 (the ordering), R123 (the essential criterion) and R145 (the modes of MELL nets) all concern the criterion a ProofStructure is checked against. R79 is the one place the descriptor is defined; R104, R123 and R145 are its variants or users and say so in their own text. They stay separate because each has its own test and step.
- R56 (the ordered dual reverses a product's operands) stands against R57, R58 and R119 (the cut's dual found by offset in the forest): the offset needs trees of the same shape, which the ordered dual breaks. They are kept apart because they concern different items (the arena's negation normal form, the forest's layout, the Cut node); the three cut entries state the invariant for the dual that keeps child order, and cut in an ordered mode is refused until a step needs it.

