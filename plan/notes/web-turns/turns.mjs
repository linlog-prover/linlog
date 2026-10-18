// Item 3 under wasm: for each row, the default bias (its two searches
// taking turns, as without threads) against each explicit bias alone, the
// engine forced to the one the two searches belong to. Times by
// performance.now() around each call; work from the outcome's statistics.
// usage: node turns.mjs PKG_DIR ROWS.jsonl WORK_BOUND OUT.jsonl
import { readFileSync, writeFileSync, appendFileSync } from "node:fs";
import { performance } from "node:perf_hooks";

const [pkg, rowsPath, bound, outPath] = process.argv.slice(2);
const bindings = await import(`${pkg}/linlog_web.js`);
bindings.initSync({ module: readFileSync(`${pkg}/linlog_web_bg.wasm`) });
const W = Number(bound);

// Within the default bias the forward search of a Horn program runs within
// `forward_copies` (30), elsewhere within `copies`: the explicit forward
// search is given the same bound.
function run(row, bias, work) {
  const copies = bias === "factors" && row.oracle_engine === "horn" ? 30 : row.copies;
  const settings = JSON.stringify({
    search: { engine: row.engine, bias, copies, classical_work: 0 },
    limits: { work, memory_bytes: 1 << 30, occurrences: 50000000, recursion_depth: 384 },
  });
  const start = performance.now();
  let outcome;
  try {
    outcome = JSON.parse(bindings.prove(JSON.stringify(row.sequent), row.mode, settings)).outcome;
  } catch (e) {
    return { ms: performance.now() - start, verdict: "error", error: String(e).slice(0, 200) };
  }
  const ms = performance.now() - start;
  return {
    ms,
    verdict: outcome.verdict,
    reason: outcome.reason?.kind ?? null,
    work: outcome.statistics.work,
  };
}

writeFileSync(outPath, "");
for (const line of readFileSync(rowsPath, "utf8").split("\n")) {
  if (!line.trim()) continue;
  const row = JSON.parse(line);
  // Twice each, the second kept: the first compiles what it runs.
  const result = { name: row.name, mode: row.mode, oracle: row.oracle, horn: row.oracle_engine === "horn" };
  for (const [key, bias, work] of [
    ["rarer", "rarer", W],
    ["factors", "factors", W],
    ["auto", "auto", 8 * W],
  ]) {
    run(row, bias, work);
    result[key] = run(row, bias, work);
  }
  appendFileSync(outPath, `${JSON.stringify(result)}\n`);
}
