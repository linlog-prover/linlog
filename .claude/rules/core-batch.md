---
paths:
  - "core/src/search/batch.rs"
---

# linlog core: the batch

Loaded, beside `core.md`, when `search/batch.rs` is read: many sequents
in one call, the library's side of `linlog prove`'s batch.

- **Problems in, results out, as iterators** (`run`, and `prove` over
  `Problem`s for a wrapper that needs no stop of its own). `run` takes a
  closure that does all of one problem's work (the command reads, searches
  under its time limit, draws and formats in it), so that the workers do
  the expensive part and the iterator only reorders. Results come in the
  order of the problems, each as soon as it and those before it are done;
  a worker takes a problem only while it is fewer than `AHEAD` per worker
  past the first result not yet given out, so held results stay few.
- **The iterator never waits on the queue's lock** to give a result out:
  `given` is an atomic beside it. A worker that waits on a stream for the
  next problem holds the lock, and a client that sends the next question
  only after the answer would otherwise deadlock with it. A worker that
  waits for its turn looks again every `WAIT`, so a lost wake-up costs
  10 ms, not a hang.
- **Threads only behind `parallel`** (the crate's rule): without the
  feature, or with one worker, the batch runs lazily on the caller's
  thread, one problem per `next`; the workers' stacks are the plan's
  `Limits::stack_bytes`. **A panic in the work is resumed at its
  problem's place** (`a_panic_ends_the_batch_at_its_place`): the worker
  catches it and sends it with its place, then cancels the batch and ends
  the queue (`try_lock`: a worker that holds the queue waits on a
  stream, and ends at its next send), so the other workers take no more
  problems and the iterator, having given out the results before it,
  resumes the panic. A panic that ended a worker had it drop no result,
  so a later place's results filled the window of `AHEAD` per worker,
  the others waited for their turn for good and the iterator for the
  panicked place. A worker's own panic (outside the work) is resumed
  once every worker ended, as before. **A worker that cannot start is no panic**
  (`Workers::start`): the batch runs on the workers that started, and
  when none did it gives the problems and the work back to `run`, which
  runs them on the caller's thread; a `recursion_depth` read from a
  settings file can ask for terabytes of stack
  (`workers_that_cannot_start_leave_the_batch_here`).
- **How the memory is shared** (`Options::plan`, crate-private: `run`
  resolves `Cores::Auto` before it plans): across the sequents,
  each search keeps the bound of the search options (so a batch's
  verdicts are those of single calls with the same bound) and as many
  workers run as the batch's bound holds such searches, at least one,
  whose bound is then the batch's; within, one worker at the batch's
  bound, which the command's race of one thread against a pool shares
  between its two searches (`search::race`, one account; each held half
  of it until the race's searches shared one, which halved the bound
  twice). What the bound does not count is what the
  search's bound does not count (the forest, the derivation's own bound,
  a render's bound, thread stacks): a worker can hold the forest of its
  sequent beside its search.
- `Cores::Auto` reads ahead as many problems as there are workers
  (clamped to `MAX_JOBS`, as the plan clamps them) and goes within when
  the batch is shorter; it blocks on a stream, so a
  front end maps a stream to `Within` or `Across` itself (the command
  takes `Within`).
- **`Options` is plain data** (`#[non_exhaustive]`, `with_*`, serde with
  defaults and unknown keys refused); the search's options and the
  limits are the batch's arguments beside it, and `Settings` holds all
  three as one JSON value. `workers` is clamped to `search::Options::
  MAX_JOBS` where the plan reads it, as `jobs` is.
- **`Results` cancels**: `run` hands every work call the batch's
  `Cancel`, `Results::cancel` and `canceller` raise it, and dropping the
  results does too; `prove`'s stop asks it, so the searches in flight
  end as `Unknown(Reason::Stopped)` and the problems not begun are
  answered at once (`cancelled_results_stop_their_searches`). The
  command keeps its own interruption flag and ignores the cancel.
  `Problem::new` and `Plan::alone` build the two non-exhaustive values
  a caller makes.
