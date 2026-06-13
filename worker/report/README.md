# Kotisatama API Worker

Anonymous backend for Kotisatama beta: user reports and fallback search telemetry.

## Deploy

```bash
cd worker/report
npx wrangler kv namespace create REPORTS
npx wrangler kv namespace create FALLBACK
# Copy ids into wrangler.toml
npx wrangler deploy
```

## Endpoints

| Path | Body | Purpose |
|---|---|---|
| `POST /report` | `{ kind, domain, message?, context_url? }` | User reports |
| `POST /fallback` | `{ query, platform }` | Local index misses |

## Browser env vars

```bash
export KOTISATAMA_REPORT_URL=https://kotisatama-api.<account>.workers.dev/report
export KOTISATAMA_FALLBACK_LOG_URL=https://kotisatama-api.<account>.workers.dev/fallback
```

Fallback searches are also appended locally to `{KOTISATAMA_DATA_DIR}/fallback-searches.jsonl`.

## Export data for triage

```bash
npx wrangler kv key list --binding REPORTS > reports-keys.json
npx wrangler kv key list --binding FALLBACK > fallback-keys.json
# Fetch values individually, or use scripts/triage-kotisatama-data.mjs on exported JSONL
```

## Optional webhook

Set `WEBHOOK_URL` to forward reports to Google Sheets (Apps Script), Airtable, or email relay.
