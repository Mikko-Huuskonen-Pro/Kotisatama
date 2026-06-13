#!/usr/bin/env node
/**
 * Aggregate Kotisatama beta data for whitelist and index decisions.
 *
 * Usage:
 *   node scripts/triage-kotisatama-data.mjs \
 *     --reports exports/reports.jsonl \
 *     --fallback exports/fallback.jsonl \
 *     --whitelist config/whitelist.json
 *
 * Input files: one JSON object per line (from Worker KV export or local jsonl).
 */

import fs from "node:fs";
import path from "node:path";

function parseArgs(argv) {
  const args = { reports: null, fallback: null, whitelist: null };
  for (let i = 2; i < argv.length; i++) {
    if (argv[i] === "--reports") args.reports = argv[++i];
    else if (argv[i] === "--fallback") args.fallback = argv[++i];
    else if (argv[i] === "--whitelist") args.whitelist = argv[++i];
  }
  return args;
}

function readJsonl(filePath) {
  if (!filePath || !fs.existsSync(filePath)) {
    return [];
  }
  return fs
    .readFileSync(filePath, "utf8")
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) => JSON.parse(line));
}

function loadWhitelistDomains(filePath) {
  if (!filePath || !fs.existsSync(filePath)) {
    return new Set();
  }
  const json = JSON.parse(fs.readFileSync(filePath, "utf8"));
  return new Set((json.domains || []).map((d) => d.toLowerCase()));
}

function countBy(items, keyFn) {
  const counts = new Map();
  for (const item of items) {
    const key = keyFn(item);
    if (!key) continue;
    counts.set(key, (counts.get(key) || 0) + 1);
  }
  return [...counts.entries()].sort((a, b) => b[1] - a[1]);
}

function normalizeDomain(raw) {
  if (!raw || typeof raw !== "string") return null;
  let domain = raw.trim().toLowerCase();
  domain = domain.replace(/^https?:\/\//, "");
  domain = domain.split("/")[0];
  domain = domain.split(":")[0];
  return domain || null;
}

const args = parseArgs(process.argv);
const reports = readJsonl(args.reports);
const fallback = readJsonl(args.fallback);
const whitelist = loadWhitelistDomains(args.whitelist);

const suggestDomains = reports
  .filter((r) => r.kind === "suggest_site")
  .map((r) => normalizeDomain(r.domain))
  .filter(Boolean);

const brokenDomains = reports
  .filter((r) => r.kind === "site_broken")
  .map((r) => normalizeDomain(r.domain))
  .filter(Boolean);

const fallbackQueries = fallback.map((f) => f.query?.trim()).filter(Boolean);

const uniqueSuggest = [...new Set(suggestDomains)];
const newCandidates = uniqueSuggest.filter((d) => !whitelist.has(d));

console.log("# Kotisatama triage summary\n");
console.log(`Reports: ${reports.length}`);
console.log(`Fallback searches: ${fallback.length}`);
console.log(`Whitelist domains: ${whitelist.size}\n`);

console.log("## Top fallback queries (index gaps)\n");
for (const [query, count] of countBy(fallback, (f) => f.query).slice(0, 25)) {
  console.log(`- ${count}× ${query}`);
}

console.log("\n## Suggested whitelist additions (from reports, not yet in whitelist)\n");
if (newCandidates.length === 0) {
  console.log("- (none)");
} else {
  for (const domain of newCandidates) {
    console.log(`- ${domain}`);
  }
}

console.log("\n## Broken site reports (review manually)\n");
for (const [domain, count] of countBy(reports.filter((r) => r.kind === "site_broken"), (r) => normalizeDomain(r.domain)).slice(0, 15)) {
  console.log(`- ${count}× ${domain}`);
}

const weeklyFallback = fallback.length;
let crawlHint = "weekly";
if (weeklyFallback > 500) crawlHint = "2× per week (high fallback volume)";
else if (weeklyFallback < 50) crawlHint = "biweekly (low fallback volume)";

console.log("\n## Index crawl cadence hint\n");
console.log(`Based on ${weeklyFallback} fallback events in export: **${crawlHint}**`);
console.log("(Adjust `.github/workflows/kotisatama-crawl.yml` cron accordingly.)\n");

if (newCandidates.length > 0) {
  const outPath = path.join(process.cwd(), "output", "whitelist-candidates.json");
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(
    outPath,
    JSON.stringify({ domains: newCandidates, generated_at: new Date().toISOString() }, null, 2) + "\n"
  );
  console.log(`Wrote ${outPath}`);
}
