// LARPd assistant eval runner. Created by (Claude Opus 5.5 Medium), 2026-11-06T20:50+01:00 (Europe/Warsaw).
// Usage: node assistant/evals/run.mjs <proxy base URL> [x-jobex-team passphrase from env JOBEX_TEAM]
// Sends each prompt in assistant/evals.jsonl as a one-message conversation and checks
// whether the proxy refused it. Tool calls count as "answer" (the page would run them).
// Each prompt costs real DeepSeek tokens (refusals only the cheap gate call).
import { readFileSync } from "node:fs";
const base = (process.argv[2] || "http://127.0.0.1:8787").replace(/\/$/, "");
const file = new URL("../evals.jsonl", import.meta.url);
const cases = readFileSync(file, "utf8").split("\n").filter(Boolean).map((l) => JSON.parse(l));
const headers = { "content-type": "application/json", origin: "https://nexicv.github.io" };
if (process.env.JOBEX_TEAM) headers["x-jobex-team"] = process.env.JOBEX_TEAM;
let pass = 0;
for (const c of cases) {
  const res = await fetch(`${base}/v1/chat`, {
    method: "POST", headers,
    body: JSON.stringify({ locale: c.locale, messages: [{ role: "user", content: c.prompt }] }),
  });
  const text = await res.text();
  const events = [...text.matchAll(/^event: (\w+)$/gm)].map((m) => m[1]);
  let got = events.includes("refused") ? "refuse"
    : events.some((e) => e === "done" || e === "tool_calls") ? "answer"
    : `other(${res.status} ${events.join(",") || text.slice(0, 80)})`;
  const ok = got === c.expect;
  if (ok) pass++;
  console.log(`${ok ? "PASS" : "FAIL"}  ${c.id.padEnd(20)} expected ${c.expect}, got ${got}`);
  if (events.includes("closed") || events.includes("busy")) { console.log("Stopped: assistant closed or busy."); break; }
}
console.log(`\n${pass}/${cases.length} passed`);
process.exit(pass === cases.length ? 0 : 1);
