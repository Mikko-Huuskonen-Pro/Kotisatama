# Kotisatama-cratet

Kaikki Kotisatama-spesifinen Rust-koodi on `components/kotisatama/`-hakemistossa. Jokainen crate on itsenäinen kirjasto, jota embedder (`ports/servoshell/kotisatama.rs`) tai Tauri-hallintapaneeli kutsuu.

> Uutta toiminnallisuutta lisätään **aina uutena cratenä** — ei muokkaamalla `components/script/` tai muita upstream-hakemistoja. Katso [AGENT.md](../../AGENT.md).

## Yleiskuva

| Crate | Cargo-nimi | Tehtävä |
|-------|------------|---------|
| `whitelist/` | `kotisatama-whitelist` | Whitelist JSON 2.1, navigointitarkistus, käyttäjän overlay |
| `search/` | `kotisatama-search` | Meilisearch-subprocess, CDN-synkronointi, hakutulosten rikastus |
| `report/` | `kotisatama-report` | Lokikirja-ilmoitukset (GitHub Issues / worker) |
| `varustamo/` | `kotisatama-varustamo` | Luotettujen sovellusten rekisteri |
| `pulloposti/` | `kotisatama-pulloposti` | Pulloposti-daemonin HTTP-client |
| `missa-olen/` | `kotisatama-missa-olen` | Missä olen -daemonin HTTP-client |
| `subprocess-app/` | `kotisatama-subprocess-app` | Yhteinen subprocess-kehytys |
| `i18n/` | `kotisatama-i18n` | Suomi/ruotsi UI-tekstit |

## `kotisatama-whitelist`

**Polku:** `components/kotisatama/whitelist/`

Whitelist määrittää, mihin verkko-osoitteisiin navigointi on sallittu. Se ei takaa sivun toimivuutta — vain että URL ei estetä embedderissä.

### Moduulit

| Moduuli | Tehtävä |
|---------|---------|
| `document.rs` | JSON-skeema 2.1 (`categories`, `types`, `domains`) |
| `domain.rs` | Domain-normalisointi, alidomain-tarkistus (`host_matches_domain`) |
| `state.rs` | Ajonaikainen tila: kuratoitu lista ∪ käyttäjän overlay |
| `user.rs` | Käyttäjän omat domainit (tallennetaan levylle) |
| `resolve.rs` | Whitelist-tiedoston lataus prioriteettijärjestyksessä |
| `product_profile.rs` | Tuoteprofiilit (normaali, lapsi, seniori, hopeakettu) |

### Keskeiset julkiset funktiot

| Funktio | Kuvaus |
|---------|--------|
| `init_with_fallback()` | Lataa whitelist ensimmäisestä löytyvästä polusta |
| `is_navigation_allowed(url)` | Onko navigointi sallittu (sisäiset `servo:` / `about:` aina) |
| `blocked_page_url(url)` | Rakentaa `servo:blocked?u=...` |
| `avomeri_gateway_url(query)` | Rakentaa `servo:avomeri?q=...` |
| `add_user_domain()` / `remove_user_domain()` | Käyttäjän overlay-muutokset |

### Whitelist-latauksen prioriteetti

1. CDN-välimuisti (`kotisatama-search::cached_whitelist_path()`)
2. `KOTISATAMA_WHITELIST_PATH`
3. `config/whitelist.json`
4. Pakatun binäärin viereinen `config/whitelist.json` tai `whitelist.json`

### Ympäristömuuttujat

| Muuttuja | Vaikutus |
|----------|----------|
| `KOTISATAMA_WHITELIST_PATH` | Paikallinen whitelist JSON |
| `KOTISATAMA_WHITELIST_PROFILE` | `free` / `pro` — kuratoidun listan suodatus |
| `KOTISATAMA_PRODUCT_PROFILE` | `normaali`, `lapsi`, `seniori`, `hopeakettu` |
| `KOTISATAMA_SENIORI_AVOMERI` | Senioriprofiilin Avomeri-oikeus |
| `KOTISATAMA_DATA_DIR` | Käyttäjän overlay-tiedostojen hakemisto |

---

## `kotisatama-search`

**Polku:** `components/kotisatama/search/`

Paikallinen haku Meilisearch-instanssia vasten. Crate **ei sisällä** Meilisearch-corea — vain HTTP-clientin ja prosessinhallinnan.

### Moduulit

| Moduuli | Tehtävä |
|---------|---------|
| `lib.rs` | `SearchClient`, subprocess-käynnistys, indeksin import |
| `cdn.rs` | CDN:stä whitelist + index.dump -lataus |
| `cdn_integrity.rs` | Manifestin allekirjoituksen tarkistus |
| `enrich.rs` | Hakutulosten rikastus whitelist-metadatalla (kategoria, tyyppi) |

### Keskeiset tyypit

| Tyyppi | Kuvaus |
|--------|--------|
| `SearchClient` | HTTP-client; voi omistaa Meilisearch-subprocessin |
| `SearchHit` | Yksi hakutulos (`id`, `url`, `title`) |
| `SearchOutcome` | `Hits` / `NoResults` / `Error` |
| `EnrichedSearchHit` | Hakutulos + whitelist-kategoria ja -tyyppi |

### Ympäristömuuttujat

| Muuttuja | Vaikutus |
|----------|----------|
| `KOTISATAMA_MEILISEARCH_BIN` | Meilisearch-binäärin polku |
| `KOTISATAMA_MEILISEARCH_URL` | Oletus `http://127.0.0.1:7700` |
| `KOTISATAMA_MEILISEARCH_DB` | LMDB-tietokannan polku |
| `KOTISATAMA_INDEX_DUMP` | Dump-tiedoston polku importtiin |
| `KOTISATAMA_CDN_BASE` | CDN-perus-URL synkronointiin |
| `KOTISATAMA_CDN_PUBLIC_KEY` | Manifestin allekirjoituksen avain |
| `KOTISATAMA_CDN_SKIP_INTEGRITY` | Ohita allekirjoitus (vain kehitys) |
| `KOTISATAMA_DATA_DIR` | Välimuistin hakemisto |
| `KOTISATAMA_SEARCH_DOCUMENTS` | Testidokumenttien JSON (kehitys) |

---

## `kotisatama-report`

**Polku:** `components/kotisatama/report/`

Lokikirja-ilmoitukset: rikkinäinen whitelist-sivu tai ehdotus uudesta sivusta.

### Lähetyskanavat (prioriteetti)

1. `KOTISATAMA_REPORT_URL` — Cloudflare Worker (tuotanto)
2. `KOTISATAMA_GITHUB_TOKEN` + `KOTISATAMA_GITHUB_REPO` — GitHub Issues (kehitys)
3. Paikallinen `reports.jsonl` — jos mitään endpointia ei ole asetettu

### Muut funktiot

| Funktio | Kuvaus |
|---------|--------|
| `note_blocked_url()` | Muistaa estetyn URL:n ilmoituslomakkeen esitäyttöön |
| `log_fallback_search()` | Kirjaa hakusanat, joille ei löytynyt tuloksia |

---

## `kotisatama-varustamo`

**Polku:** `components/kotisatama/varustamo/`

Luotettujen sovellusten rekisteri (`varustamo/registry.json`). Renderöi Varustamo-sivun (`servo:varustamo`) ja ohjaa sovelluksiin.

| Funktio | Kuvaus |
|---------|--------|
| `load_registry()` | Lukee rekisterin levyltä |
| `load_registry_json()` | JSON API `servo:varustamo/registry` |
| `gateway_url()` | `servo:varustamo` |
| `app_gateway_url(id)` | Yksittäisen sovelluksen sisäinen URL |

**Ympäristö:** `KOTISATAMA_VARUSTAMO_REGISTRY` — rekisteritiedoston polku.

---

## `kotisatama-pulloposti` ja `kotisatama-missa-olen`

Nämä kaksi crateä seuraavat samaa mallia: HTTP-client paikalliselle daemonille, joka bundlataan suljetusta reposta.

| | Pulloposti | Missä olen |
|---|-----------|------------|
| Oletusportti | 7701 | 7702 |
| Health | `/health` | `/healthz` |
| Binääri | `KOTISATAMA_PULLOPOSTI_BIN` | `KOTISATAMA_MISSA_OLEN_BIN` |
| URL override | `KOTISATAMA_PULLOPOSTI_URL` | `KOTISATAMA_MISSA_OLEN_URL` |
| Sisäinen sivu | `servo:pulloposti` | `servo:missa-olen` |

Julkisessa repossa on vain prosessinhallinta ja HTTP-rajapinta. Salaus, BLE ja geokoodauslogiikka ovat suljetussa repossa.

---

## `kotisatama-subprocess-app`

**Polku:** `components/kotisatama/subprocess-app/`

Yhteinen apukirjasto subprocess-sovelluksille:

- `ManagedSubprocess` — käynnistää prosessin, tappaa `Drop`:issa
- `wait_for_health()` — odottaa `/health`-vastauksen
- `find_binary()` — etsii binäärin ympäristömuuttujasta tai PATH:sta

---

## `kotisatama-i18n`

**Polku:** `components/kotisatama/i18n/`

Suomenkieliset ja ruotsinkieliset UI-tekstit desktop-työkalupalkille ja sisäisille sivuille.

Kielivalinta (prioriteetti):

1. `KOTISATAMA_LOCALE` — pakotettu (`fi` / `sv`)
2. Tallennettu valinta (`~/.config/Kotisatama/locale`)
3. `LANG`-ympäristömuuttuja
4. Suomi oletuksena

Sisäinen sivu: `servo:locale?set=fi|sv|auto`

---

## Miten cratet kytkeytyvät embedderiin

```
ports/servoshell/kotisatama.rs
    ├── kotisatama_whitelist  → init, is_navigation_allowed, blocked_page_url
    ├── kotisatama_search     → SearchClient, sync_from_cdn, enrich_outcome
    ├── kotisatama_report     → submit, note_blocked_url, log_fallback_search
    ├── kotisatama_varustamo  → load_registry, gateway_url
    ├── kotisatama_pulloposti → PullopostiClient (lazy start)
    └── kotisatama_missa_olen → MissaOlenClient (lazy start)
```

Tauri-hallintapaneeli (`tauri/src-tauri/`) käyttää vain `kotisatama-whitelist`-cratetta — ei Servon sisäisiä cratejä.

## Seuraavaksi

- [navigointi.md](navigointi.md) — miten whitelist ja haku käytännössä toimivat
- [sisaiset-sivut.md](sisaiset-sivut.md) — `servo:`-sivut
- [arkkitehtuuri.md](arkkitehtuuri.md) — kokonaiskuva
