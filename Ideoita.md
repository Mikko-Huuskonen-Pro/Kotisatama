### Ideoita ja ajatuksia tulevaisuudesta. Ehkä toteutukseen, ei lähdetä rakentamaan näiden mukaan, vielä. 

# Whitelist-pipeline — suunnitelma
*Kotisatama— kesäkuu 2026*

---

## Reporakenne

```
GitHub (Ilio-organisaatio)
│
├── kotisatama/                  ← JULKINEN (Servo-fork, MPL pakottaa)
│   └── config/
│       ├── whitelist.schema.json    ← julkinen, dokumentaatio
│       └── whitelist-example.json  ← esimerkki ilman oikeita domaineja
│
├── kotisatama-whitelistat/      ← YKSITYINEN (bisnesdata)
│   ├── whitelist-hopeakettu.json
│   ├── whitelist-lapsi.json
│   └── whitelist-hyvinvointialueet.json
│
└── kotisatama-infra/            ← YKSITYINEN (CDN-skriptit, API-avaimet)
    └── deploy/
```

`kotisatama-whitelistat` ei koskaan forkkaudu julkiseen repoon. Kun niissä alkaa olla tuhansia ja tuhansia sivuja, niille alkaa muodostumaan arvoa. 
MPL-velvoite koskee Servo-koodia — ei JSON-dataa erillisessä repossa.

---

## Pipeline: whitelist → CDN

### Triggerit

| Tapahtuma | Mitä käynnistyy |
|---|---|
| Push `kotisatama-whitelistat/main` | Validointi + CDN-deploy |
| Push `kotisatama/main` | Selain-build (ei whitelist-deployta) |
| Cron: maanantai 06:00 | Crawler + Meilisearch-dump + CDN-deploy |

### Vaiheet

```
1. VALIDOINTI (whitelist-repo push)
   └── Tarkista JSON-skeema (whitelist.schema.json)
   └── Tarkista domainien muoto (regex: ei protokollaa, ei trailaavia slasheja)
   └── Jos virhe → PR blokataan, Slack-ilmoitus

2. CDN-DEPLOY (whitelist-repo push → validointi ok)
   └── Hae whitelist-tiedostot yksityisestä reposta
   └── Yhdistä hopeakettu + hyvinvointialueet → /pro/whitelist.json
   └── Rakenna /free/whitelist.json (geneerinen)
   └── Push CDN:ään (Cloudflare R2 tai Bunny)
   └── Invalidoi CDN-cache

3. CRAWLER + INDEKSI (cron / manuaalinen trigger)
   └── Hae tuore whitelist CDN:stä (tai suoraan reposta)
   └── Aja Playwright-crawler per versio
   └── Generoi Meilisearch-dump
   └── Push dump CDN:ään (/dumps/hopeakettu-YYYY-MM-DD.bin jne.)
   └── Päivitä /dumps/latest-hopeakettu.bin symlink/redirect
```

---

## GitHub Actions — konkreettiset tiedostot

### `kotisatama-whitelistat/.github/workflows/deploy.yml`

```yaml
name: Whitelist deploy

on:
  push:
    branches: [main]

jobs:
  validate-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Skeemavalidointi
      - name: Validoi JSON-skeema
        run: |
          npm install -g ajv-cli
          ajv validate \
            -s schema/whitelist.schema.json \
            -d "*.json" \
            --all-errors

      # Hae deploy-skriptit infra-reposta
      - uses: actions/checkout@v4
        with:
          repository: ilio/kotisatama-infra
          token: ${{ secrets.INFRA_REPO_TOKEN }}
          path: infra

      # Yhdistä ja deploy
      - name: Yhdistä whitelistat
        run: node infra/deploy/merge-whitelists.js

      - name: Deploy CDN:ään
        env:
          CDN_API_KEY: ${{ secrets.CDN_API_KEY }}
        run: node infra/deploy/push-cdn.js
```

### `kotisatama/.github/workflows/crawler.yml`

```yaml
name: Crawler + indeksi

on:
  schedule:
    - cron: '0 6 * * 1'   # Maanantai 06:00 UTC
  workflow_dispatch:        # Manuaalinen trigger

jobs:
  crawl:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Hae tuore whitelist CDN:stä
      - name: Hae whitelist
        env:
          CDN_PRO_KEY: ${{ secrets.CDN_PRO_KEY }}
        run: |
          curl -H "Authorization: Bearer $CDN_PRO_KEY" \
            https://cdn.kotisatama.fi/pro/whitelist.json \
            -o config/whitelist.json

      # Aja crawler
      - name: Asenna Playwright
        run: |
          cd crawler
          npm ci
          npx playwright install chromium

      - name: Aja crawler
        run: |
          cd crawler
          node crawl.js \
            --whitelist ../config/whitelist.json \
            --output ./dump

      # Push dump CDN:ään
      - name: Push indeksi CDN:ään
        env:
          CDN_API_KEY: ${{ secrets.CDN_API_KEY }}
        run: node infra/deploy/push-dump.js ./crawler/dump
```

---

## Secrets — mitä tarvitaan

| Secret | Missä repossa | Käyttö |
|---|---|---|
| `INFRA_REPO_TOKEN` | whitelist-repo | Lukee infra-repon deploy-skriptit |
| `CDN_API_KEY` | whitelist-repo, kotisatama | Kirjoittaa CDN:ään |
| `CDN_PRO_KEY` | kotisatama (crawler) | Lukee Pro-whitelist CDN:stä |

Kaikki secrets GitHub-organisaatiotasolla → saatavilla vain halutuissa repoissa.

---

## CDN-rakenne (Cloudflare R2 tai Bunny)

```
cdn.kotisatama.fi/
├── free/
│   └── whitelist.json               ← julkinen, ei autentikaatiota
├── pro/
│   └── whitelist.json               ← API-avain vaaditaan (sovelluksessa)
└── dumps/
    ├── hopeakettu-2026-06-16.bin    ← versionoitu
    ├── lapsi-2026-06-16.bin
    ├── latest-hopeakettu.bin        ← redirect / symlink tuoreimpaan
    └── latest-lapsi.bin
```

Pro-polku suojataan CDN:n omalla token-autentikaatiolla — ei omaa palvelinta.

---

## Paikallinen kehitys

Kehittäjä (eli sinä) käyttää omaa kopiota whitelististä:

```bash
# Kloonaa whitelist-repo rinnalle
git clone git@github.com:ilio/kotisatama-whitelistat.git ../kotisatama-whitelistat

# Kopioi haluamasi profiili
cp ../kotisatama-whitelistat/whitelist-hopeakettu.json config/whitelist.json
export KOTISATAMA_WHITELIST_PATH=config/whitelist.json

# Build toimii normaalisti
./mach build --release
```

README:ssä `kotisatama`-repossa riittää mainita:
> *Whitelist-tiedostot haetaan sisäisestä reposta. Kehitystä varten käytä `whitelist-example.json` tai pyydä pääsy `kotisatama-whitelistat`-repoon.*

---

## Avoimet päätökset

- [ ] CDN-valinta: Cloudflare R2 (halvin siirto, Workers-integraatio) vai Bunny (yksinkertaisempi API)?
- [ ] Pro-autentikaatio CDN-tasolla: Cloudflare Token vs. Bunny Token — kumpi helpompi mobiilisovelluksessa?
- [ ] Crawler-ajoväli: viikko riittää vai tarvitaanko tiheämpi (esim. hyvinvointialueiden sivut muuttuvat)?
- [ ] Dump-versioiden säilytysaika CDN:ssä (ehdotus: 4 viikkoa, sitten poisto)
- [ ] GitHub-organisaatio: onko `ilio`-org jo olemassa vai luodaanko nyt?
