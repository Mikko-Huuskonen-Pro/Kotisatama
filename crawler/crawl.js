#!/usr/bin/env node
/**
 * Kotisatama crawler — indeksoi whitelist-domainit Playwrightilla ja tuottaa Meilisearch-dumpin.
 *
 * Usage:
 *   node crawl.js --whitelist ../config/whitelist.json --output ./output
 *   node crawl.js --whitelist ../config/whitelist.json --output ./output --max-depth 2 --max-pages 40
 */

import { cpSync, mkdirSync, readFileSync, readdirSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { MeiliSearch } from "meilisearch";
import { chromium } from "playwright";

const __dirname = dirname(fileURLToPath(import.meta.url));

const DEFAULTS = {
  whitelist: "../config/whitelist.json",
  output: "./output",
  meilisearch: "http://127.0.0.1:7700",
  "dump-dir": "./dumps",
  "max-depth": 2,
  "max-pages": 40,
  "request-delay-ms": 500,
};

const SKIP_EXTENSIONS = new Set([
  ".pdf", ".zip", ".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg",
  ".mp4", ".mp3", ".avi", ".mov", ".woff", ".woff2", ".ttf", ".exe",
]);

function parseArgs(argv) {
  const opts = { ...DEFAULTS };
  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i];
    if (arg === "--help" || arg === "-h") {
      opts.help = true;
      continue;
    }
    if (!arg.startsWith("--")) continue;
    const key = arg.slice(2);
    const next = argv[i + 1];
    if (next && !next.startsWith("--")) {
      opts[key] = next;
      i++;
    } else {
      opts[key] = true;
    }
  }
  return opts;
}

function printHelp() {
  console.log(`Kotisatama crawler

Options:
  --whitelist <path>       Whitelist JSON (default: ${DEFAULTS.whitelist})
  --output <dir>           Output directory for CDN bundle (default: ${DEFAULTS.output})
  --meilisearch <url>      Meilisearch URL (default: ${DEFAULTS.meilisearch})
  --dump-dir <dir>         Meilisearch dump directory (default: ${DEFAULTS["dump-dir"]})
  --max-depth <n>          Link depth per domain (default: ${DEFAULTS["max-depth"]})
  --max-pages <n>          Max pages per domain (default: ${DEFAULTS["max-pages"]})
  --request-delay-ms <n>   Delay between page loads (default: ${DEFAULTS["request-delay-ms"]})
`);
}

function loadWhitelist(path) {
  const raw = readFileSync(resolve(path), "utf8");
  const data = JSON.parse(raw);
  if (!Array.isArray(data.domains)) {
    throw new Error("whitelist JSON must contain a domains array");
  }
  return data.domains.map((d) => d.trim().toLowerCase()).filter(Boolean);
}

function registrableDomain(hostname) {
  return hostname.replace(/^www\./, "").toLowerCase();
}

function isAllowedUrl(url, allowedDomains) {
  try {
    const parsed = new URL(url);
    if (!["http:", "https:"].includes(parsed.protocol)) return false;
    const host = registrableDomain(parsed.hostname);
    return allowedDomains.some(
      (d) => host === d || host.endsWith(`.${d}`),
    );
  } catch {
    return false;
  }
}

function shouldSkipUrl(url) {
  try {
    const { pathname } = new URL(url);
    const dot = pathname.lastIndexOf(".");
    if (dot === -1) return false;
    return SKIP_EXTENSIONS.has(pathname.slice(dot).toLowerCase());
  } catch {
    return true;
  }
}

function normalizeUrl(url) {
  const parsed = new URL(url);
  parsed.hash = "";
  if (parsed.pathname.endsWith("/") && parsed.pathname.length > 1) {
    parsed.pathname = parsed.pathname.replace(/\/+$/, "");
  }
  return parsed.href;
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function crawlDomain(page, domain, allowedDomains, opts) {
  const maxDepth = Number(opts["max-depth"]);
  const maxPages = Number(opts["max-pages"]);
  const delayMs = Number(opts["request-delay-ms"]);

  const seeds = [`https://${domain}/`, `https://www.${domain}/`];
  const queue = seeds.map((url) => ({ url: normalizeUrl(url), depth: 0 }));
  const seen = new Set();
  const documents = [];
  let docId = 1;

  while (queue.length > 0 && documents.length < maxPages) {
    const { url, depth } = queue.shift();
    if (seen.has(url)) continue;
    seen.add(url);

    if (!isAllowedUrl(url, allowedDomains) || shouldSkipUrl(url)) continue;

    try {
      const response = await page.goto(url, {
        waitUntil: "domcontentloaded",
        timeout: 30_000,
      });
      if (!response || response.status() >= 400) continue;

      await page.waitForTimeout(300);
      const title = (await page.title()) || url;
      documents.push({ id: docId++, url, title: title.trim().slice(0, 500) });

      if (depth < maxDepth) {
        const hrefs = await page.$$eval("a[href]", (anchors) =>
          anchors.map((a) => a.href).filter(Boolean),
        );
        for (const href of hrefs) {
          const normalized = normalizeUrl(href);
          if (!seen.has(normalized) && isAllowedUrl(normalized, allowedDomains)) {
            queue.push({ url: normalized, depth: depth + 1 });
          }
        }
      }

      if (delayMs > 0) await sleep(delayMs);
    } catch (error) {
      console.warn(`Skip ${url}: ${error.message}`);
    }
  }

  return documents;
}

async function ensureIndex(client) {
  try {
    await client.getIndex("documents");
  } catch {
    await client.createIndex("documents", { primaryKey: "id" });
  }
}

async function exportDump(client, outputDir, dumpDir) {
  const { taskUid } = await client.createDump();
  await client.waitForTask(taskUid);

  const resolvedDumpDir = resolve(dumpDir);
  const dumps = readdirSync(resolvedDumpDir)
    .filter((f) => f.endsWith(".dump"))
    .map((f) => ({
      name: f,
      mtime: statSync(join(resolvedDumpDir, f)).mtimeMs,
    }))
    .sort((a, b) => b.mtime - a.mtime);

  if (dumps.length === 0) {
    throw new Error(`No .dump file found in ${resolvedDumpDir}`);
  }

  const freeDir = join(outputDir, "free");
  mkdirSync(freeDir, { recursive: true });
  const dest = join(freeDir, "index.dump");
  cpSync(join(resolvedDumpDir, dumps[0].name), dest);
  return dest;
}

async function main() {
  const opts = parseArgs(process.argv);
  if (opts.help) {
    printHelp();
    return;
  }

  const whitelistPath = resolve(opts.whitelist);
  const outputDir = resolve(opts.output);
  const meilisearchUrl = opts.meilisearch;

  const domains = loadWhitelist(whitelistPath);
  console.log(`Crawling ${domains.length} whitelist domains…`);

  const client = new MeiliSearch({ host: meilisearchUrl });
  await ensureIndex(client);
  const index = client.index("documents");

  try {
    await index.deleteAllDocuments();
  } catch {
    // empty index
  }

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    userAgent: "KotisatamaCrawler/0.1 (+https://ilio.fi/kotisatama)",
  });
  const page = await context.newPage();

  const allDocuments = [];
  let nextId = 1;

  for (const domain of domains) {
    console.log(`→ ${domain}`);
    const docs = await crawlDomain(page, domain, domains, opts);
    for (const doc of docs) {
      allDocuments.push({ ...doc, id: nextId++ });
    }
    console.log(`  ${docs.length} pages`);
  }

  await browser.close();

  if (allDocuments.length === 0) {
    throw new Error("No documents crawled — check whitelist and network");
  }

  console.log(`Indexing ${allDocuments.length} documents…`);
  await index.addDocuments(allDocuments);
  await sleep(1000);

  mkdirSync(outputDir, { recursive: true });
  const freeDir = join(outputDir, "free");
  mkdirSync(freeDir, { recursive: true });
  cpSync(whitelistPath, join(freeDir, "whitelist.json"));

  const dumpPath = await exportDump(client, outputDir, opts["dump-dir"]);
  console.log(`Done. CDN bundle:`);
  console.log(`  ${join(freeDir, "whitelist.json")}`);
  console.log(`  ${dumpPath}`);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
