# Kotisatama crawler

Indeksoi whitelist-domainit Playwrightilla (JS/SPA-tuki) ja tuottaa CDN-valmiin paketin.

## Vaatimukset

- Node.js 20+
- Meilisearch käynnissä paikallisesti (`http://127.0.0.1:7700`)

```bash
# Asenna Meilisearch: https://www.meilisearch.com/docs/learn/getting_started/installation
meilisearch --http-addr 127.0.0.1:7700 --env development
```

## Ajaminen

```bash
cd crawler
npm install
npm run crawl -- --whitelist ../config/whitelist.json --output ../output/cdn
```

## Tuloste (CDN-paketti)

```
output/cdn/free/
  manifest.json     ← SHA-256-tiivisteet (automaattinen)
  whitelist.json    ← kopio syötteestä
  index.dump        ← Meilisearch-dump
```

Julkaise nämä CDN-polkuun `/free/manifest.json`, `/free/whitelist.json` ja `/free/index.dump`.

### CDN-allekirjoitus (CI)

Aseta CI-salaisuus `KOTISATAMA_CDN_SIGNING_KEY_HEX` (32 tavua, 64 hex-merkkiä). Crawler allekirjoittaa `manifest.json`:n automaattisesti.

Julkinen avain asiakkaalle: `config/cdn-signing-public.hex` (kehitys) tai `KOTISATAMA_CDN_PUBLIC_KEY` (tuotanto-build).

## Parametrit

| Parametri | Oletus | Kuvaus |
|---|---|---|
| `--max-depth` | 2 | Linkkisyvyys per domain |
| `--max-pages` | 40 | Max sivumäärä per domain |
| `--request-delay-ms` | 500 | Viive sivulatausten välillä |
