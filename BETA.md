# Kotisatama — julkinen beta

Tavoite: ensimmäiset ulkopuoliset käyttäjät, fallback-data whitelist-kehitykseen.

## Ennen julkaisua

1. **Cloudflare Worker** — deploy `worker/report/` (reports + fallback telemetry)
   ```bash
   export KOTISATAMA_REPORT_URL=https://<worker>/report
   export KOTISATAMA_FALLBACK_LOG_URL=https://<worker>/fallback
   ```
2. **CDN** — julkaise `output/cdn/free/` (whitelist + index.dump) R2/Bunnyhin; aseta `KOTISATAMA_CDN_BASE`
3. **Whitelist** — tarkista `config/whitelist.json` (50–100 domainia suositus)
4. **Crawler** — varmista viikoittainen `.github/workflows/kotisatama-crawl.yml` tai manuaalinen crawl

## Desktop-beta

```bash
# Windows (lyhyt polku)
$env:PATH = "C:\Program Files\LLVM\bin;" + $env:PATH
$env:CARGO_TARGET_DIR = "C:\kt\target"
./mach build --release
```

Jaa `servoshell.exe` + `config/` + Meilisearch-binääri (tai ohje asennukseen).

Pakolliset ympäristömuuttujat beta-testaajille:

| Muuttuja | Kuvaus |
|---|---|
| `KOTISATAMA_CDN_BASE` | OTA-päivitykset |
| `KOTISATAMA_REPORT_URL` | Raportit |
| `KOTISATAMA_FALLBACK_LOG_URL` | Fallback-haut (valinnainen, lokittuu myös paikallisesti) |

Paikallinen fallback-loki: `index-data/fallback-searches.jsonl` (tai Android: app files / kotisatama).

## Android-beta

Katso [support/android/README.md](support/android/README.md).

```bash
./mach build --target aarch64-linux-android --profile checked-release
./support/android/fetch-meilisearch.sh
cd support/android/apk && ./gradlew :servoapp:assembleArm64Release
```

APK: `target/aarch64-linux-android/checked-release/servoapp.apk`

## Datankeruu (vaihe 6)

### Fallback-haut

Kun paikallinen haku ei löydä osumia, selain lokittaa **vain hakusanan** (ei avomeri-dataa):

- Paikallinen: `{KOTISATAMA_DATA_DIR}/fallback-searches.jsonl`
- Pilvi: `POST /fallback` Worker-endpointiin

### Raportit

Käyttäjät voivat ehdottaa uusia sivustoja (**Ehdota kotisatamaan**) tai ilmoittaa rikkinäisistä sivuista.

### Triage (viikoittain)

```bash
# Vie Worker KV → JSONL (tai kerää testaajien paikalliset jsonl-tiedostot)
node scripts/triage-kotisatama-data.mjs \
  --reports exports/reports.jsonl \
  --fallback exports/fallback.jsonl \
  --whitelist config/whitelist.json
```

Tuloste: top fallback-haut, whitelist-ehdokkaat, crawl-tiheys-suositus.

### Whitelist-päivitys

1. Triage → `output/whitelist-candidates.json`
2. Manuaalinen tarkistus
3. Päivitä `config/whitelist.json`
4. Aja crawler → julkaise CDN → käyttäjät saavat OTA-päivityksen

## Indeksin päivitystiheys

| Fallback-volyymi (viikko) | Suositus |
|---|---|
| &lt; 50 | 2 viikon välein |
| 50–500 | Viikoittain (oletus) |
| &gt; 500 | 2× viikossa |

Muokkaa cron-lauseketta tiedostossa `.github/workflows/kotisatama-crawl.yml`.

## Beta-version merkintä

Android: `versionName` + `versionCode` tiedostossa `support/android/apk/servoapp/build.gradle.kts`.

Desktop: Git tag `v0.3.0-beta.1` + release notes (whitelist-muutokset, tunnetut rajoitteet).

## Tunnetut rajoitteet (beta)

- Meilisearch Android: Linux aarch64 -binary, testaa laitteella
- Avomeri: Startpage-fallback ilman sessiorajoja
- Ei tilejä, ei synkronointia profiilien välillä
