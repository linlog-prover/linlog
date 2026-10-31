# Many sequents in one call

`prove` decides many sequents in one call, a batch, when it is given
`--file` more than once, a directory, a list of paths (`--files-from LIST`,
one per line, or separated by NUL with `--null` as `find -print0` writes
them, `-` for standard input), or an input format of many sequents:
`--input-format lines` (`NAME: SEQUENT` or `SEQUENT` per line, blank lines
and everything from `#` on skipped), `jsonl` (a sequent as `seq json`
writes it, or a record with a name, a mode and the sequent, per line) or
`problems` (the benchmark harness's problem files). Every sequent gets one
line, in the order of the input and as soon as it is decided, named by
its path as given, its line's name, or its file and line number:

```console
$ cat > sequents.txt
> # a comment, then four sequents
> identity: A |- A
> modus ponens: A, A -o B |- B
> A |- B
> broken: A |- (
$ linlog prove --input-format lines --file sequents.txt
identity: provable (MLL, classical, focus engine)
modus ponens: provable (MLL, classical, focus engine)
sequents.txt:4: unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
broken: error: cannot parse the sequent
   A |- (
         ^ unexpected end of input, expected a formula
$ linlog prove --input-format lines --file sequents.txt --format svg --output drawings
identity: provable (MLL, classical, focus engine)
modus ponens: provable (MLL, classical, focus engine)
sequents.txt:4: unprovable (MLL, classical, focus engine): ~A occurs 1 more time than A in the one-sided sequent, so they cannot all meet in axioms
broken: error: cannot parse the sequent
   A |- (
         ^ unexpected end of input, expected a formula
```

The exit status is the worst verdict: an error (2) before unknown (3)
before unprovable (1) before proved (0), so this one is 2; a batch that
holds no sequent is an error, and one interrupted with Ctrl-C, which
ends its input, is unknown at best. The second
command writes each proved sequent's derivation into the directory
`--output` names, in the format asked for, named after the sequent with
the format's extension (`drawings/identity.svg`; `ILL/01/X.p` becomes
`DIR/ILL/01/X.p.svg`, so the LLTP library's translations of one problem
do not overwrite each other). A file's kind
is its extension's unless `--input-format` says otherwise: `.p` is a
problem of the LLTP library, `.spec` a coverability problem in the format
of the Mist tool, `.json` a sequent in JSON, anything else the text
syntax, for a single `prove` as well. An LLTP problem does not say
whether it is intuitionistic (the library keeps those under `ILL/`), so
the flags say it:

```console
$ cat > problem.p
> % Status (intuit.) : Theorem
> fof(rule, axiom, !(a -o b)).
> fof(fact, axiom, a).
> fof(goal, conjecture, b * 1).
$ linlog prove -i -q --file problem.p
provable (IMELL, intuitionistic, horn engine)
```

A `.spec` problem is a Petri net or a vector addition system as the
coverability suites of software verification write it: counters, rules
that test and change them, an initial marking (`x >= k` for at least `k`
tokens, any number more), and the markings to cover, a line each. It is
read as a Horn program, the rules under `!` and the initial tokens left
of `⊢`, and covering is a question of affine mode, in which a `.spec`
file is decided whatever the flags say (`-i` and `--mix` keep their
meaning). Two processes in the critical section at once, for any number of
processes and one lock:

```console
$ cat > mutex.spec
> vars idle critical lock
> rules
>   idle >= 1, lock >= 1 -> idle' = idle - 1, lock' = lock - 1, critical' = critical + 1;
>   critical >= 1 -> critical' = critical - 1, idle' = idle + 1, lock' = lock + 1;
> init idle >= 1, lock = 1
> target critical >= 2
$ linlog prove -i --affine -q --file mutex.spec
unprovable (IMELL, intuitionistic affine, horn engine): the state equation of the Petri net has no solution, so no firing of its clauses yields the goal's atoms: weighting each lock by 1 and critical by 1, no clause that can fire raises the weighted count of the atoms, and the goal asks it raised
```

A directory is walked in sorted order, links followed, for the files of
the input format's extension (`.p` and `.json` by default); paths are
taken literally, relative to the current directory. On standard input
the batch is a stream: each line is answered as soon as it is decided,
so a program can ask, wait for the answer and ask again. With `--format
json` every answer is a JSON Lines record, the wire level and the name
first and then what `--format json` writes for one sequent (with
`--stats` also `seconds`, the time it took, which a single sequent's JSON
leaves out to stay reproducible); an entry
that could not be read or searched is `"verdict": "error"` with a
`message`, and the library's `error` object (its `code`, `kind` and
`details`) where the error is the library's, such as a parse error or a
bound that refused the input, and one the batch stopped before
it was decided `"verdict": "unknown"` with a `message`. The mode is the
flags' (else a settings file's `batch.mode`), a problem file's column or
a record's:

```console
$ echo '{"name": "pair", "mode": "intuitionistic", "sequent": "A, B |- A * B"}' | linlog prove --input-format jsonl --format json
{"version":1,"name":"pair","linlog":"…","verdict":"proved","checked":true,"fragment":"IMLL","mode":"intuitionistic","engine":"two-sided","statistics":{"nodes":1,"memo_hits":0,"memo_entries":1,"splits":1,"links":0,"tests":0,"copies":0,"forward_copies":0,"work":16},"sequent":{"terms":[{"D":0},{"D":1},{"V":0},{"V":1},{"⊗":[2,3]}],"roots":[0,1,4],"atoms":["A","B"],"antecedents":2},"nodes":[{"ax":[3,0]},{"ax":[4,1]},{"⊗":[2,0,1]}]}
```

By default the cores go across the sequents, one sequent per worker on
the sequential engines (as with `--deterministic`, so a sequent that
only a pool decides within its time limit stays unknown), and within one
sequent, as for a single call, when the batch has fewer sequents than
workers or is a stream from standard input; `--cores across|within`
chooses, and `--workers N` caps the sequents decided at once.
Across the cores a batch answers the most sequents in a given time, and
each answer is a function of its input; what it leaves unknown is
worth a second pass on the whole machine, which costs little because
it is a few sequents:

```sh
linlog prove -i --file problems/ > results.txt
grep ': unknown' results.txt | cut -d: -f1 > unknown.txt
linlog prove -i --files-from unknown.txt --cores within > second.txt
```

`--timeout` is each sequent's limit, reading included, and
`--batch-timeout` the whole batch's. Each search holds at most
`--memory-limit`, and as many run at once as `--batch-memory` holds (by
default half the memory the process may use, the machine's or its
control group's), so that a batch never holds more
than that for its searches; `--isolate` decides each sequent in a child
process of its own, so that one that exhausts the memory or the stack
ends only itself.
