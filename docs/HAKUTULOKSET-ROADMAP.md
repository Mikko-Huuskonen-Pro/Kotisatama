# Hakutulossivu — toteutusroadmap

*Päivitetty: kesäkuu 2026 — whitelist 2.1 + graafinen hakunäkymä*

Tämä dokumentti kuvaa hakutulossivun (`servo:haku`) toteutusjärjestyksen. Tavoite on rakentaa klassinen hakukonenäkymä nykyisen Meilisearch-moottorin päälle **ilman Servo-upstream-konflikteja**.

Liittyvät dokumentit:

- [`Hakutulokset.md`](Hakutulokset.md) — tuotespesifikaatio
- [`../config/whitelist.schema.json`](../config/whitelist.schema.json) — whitelist 2.1 -skeema
- [`../AGENT.md`](../AGENT.md) — fork-säännöt ja upstream-strategia

---

## Edellytykset (valmiina tai tehty tässä PR:ssä)

| Kohde | Tila |
|---|---|
| Whitelist 2.1 -skeema (`categories`, `types`, `domain.category`, `type: yellow`) | Valmis julkisessa repossa |
| `kotisatama-whitelist` parseri v2.1 | Valmis |
| Meilisearch subprocess + `kotisatama-search` | Toimii |
| Whitelist-navigointi (`load_url_or_blocked`) | Toimii |
| Kuratoitu data (`whitelist-unified.json`) | Suljetussa repossa |

---

## Arkkitehtuuripäätös

```text
Osoitepalkki
    ↓
Meilisearch (url + title)
    ↓
servo:haku?q=…  (HTML-sivu resource_protocol/)
    ↓
Rikastus: url-host → whitelist.domains[] → category + type
    ↓
Lookup: categories[] + types[] → ikoni + väripiste
    ↓
Klikkaus → load_url_or_blocked (whitelist päättää)
```

**Upstream-kosketukset** rajoittuvat ohuihin `KOTISATAMA-PATCH`-kohtiin `ports/servoshell/`-hakemistossa. Kaikki logiikka ja UI elävät fork-omistuksessa.

---

## Vaihe 0 — Whitelist 2.1 ✅

**Hakemistot:** `config/`, `components/kotisatama/whitelist/`

- [x] `whitelist.schema.json` versio 2.1
- [x] `whitelist.example.json` kategorioilla, tyypeillä ja esimerkkidomaineilla
- [x] Rust: `CategoryMeta`, `TypeMeta`, `WhitelistEntry.category`
- [x] Rust: `lookup_entry_for_host`, `category_meta`, `type_meta`
- [x] Rust: `lookup_curated_entry` runtime-API hakusivulle

**Ei koske upstreamia.**

---

## Vaihe 1 — Hakurikastus ✅

**Hakemisto:** `components/kotisatama/search/`

- [x] `enrich_hit(hit: &SearchHit) -> EnrichedSearchHit`
- [x] `enrich_outcome()` / `search_results_json()` servoshellissa
- [x] Yksikkötestit
- [x] Fallback ilman whitelist-matchia

---

## Vaihe 2 — Sisäinen hakusivu (UI) ✅

**Hakemistot:** `resources/resource_protocol/`, `components/kotisatama/i18n/`

- [x] `haku.html` + `haku.css`
- [x] `haku-icons.js` — SVG-ikonit kategorioille ja tyypeille
- [x] `kotisatama-i18n.js` — käännökset (fi + sv)
- [x] Tuloskortti: väripiste + toimialaikoni + label + domain + title
- [x] Tyhjä tulos ja virhenäkymä

---

## Vaihe 3 — Protokolla ja data-API ✅

**Hakemisto:** `ports/servoshell/desktop/protocols/servo.rs`

- [x] `servo:haku` → `haku.html`
- [x] `servo:haku/data?q=…` → JSON
- [x] `kotisatama.rs`: `search_results_url()`, `search_results_json()`

---

## Vaihe 4 — Osoitepalkin reititys ✅

**Hakemisto:** `ports/servoshell/window.rs`, `desktop/gui.rs`

- [x] `open_search_or_results()` + `should_open_best_hit_directly()`
- [x] `UserInterfaceCommand::Search` hakupainikkeelle
- [x] Enter: yksi osuma → suoraan, muuten `servo:haku`
- [x] egui-hakupaneeli poistettu

---

## Vaihe 5 — Android ✅

- [x] `KotisatamaUi` avaa `servo:haku?q=…` webviewissä
- [x] Yksi osuma Enterillä → suoraan (kuten desktop)

---

## Vaihe 6 — Crawler ja indeksi (myöhemmin)

**Hakemisto:** `crawler/`

Valinnainen optimointi — ei pakollinen v1:ssä:

- [ ] Indeksoi `domain`, `label`, `category`, `type` mukaan Meilisearch-dumpiin
- [ ] Pidä rikastus silti whitelist-lookupina (indeksi voi olla vanhentunut)

Konseptin mukaan visuaaliset valinnat tulevat whitelististä, ei indeksistä.

---

## Vaihe 7 — Julkinen hakemisto (pitkä aikaväli)

**Hakemisto:** Katselin.fi-repo (erillinen)

- [ ] `katselin.fi/haku?q=…` staattinen tai palvelinhaku
- [ ] Sama visuaalinen malli kuin selaimen sisäisellä hakusivulla

---

## Upstream-turvallisuus — yhteenveto

| Hakemisto | Merge-konflikti upstreamiin? |
|---|---|
| `components/kotisatama/` | Ei koskaan |
| `config/` | Ei koskaan |
| `resources/resource_protocol/` | Ei koskaan |
| `docs/` | Ei koskaan |
| `ports/servoshell/` | Harvoin — vain `KOTISATAMA-PATCH` |

Ennen jokaista PR:ää:

```bash
cargo build
cargo build --features kotisatama
cargo test -p kotisatama-whitelist -p kotisatama-search
```

---

## Ensimmäisen version rajaus (muistilista)

Sisällytä:

- hakukenttä, tuloslista, label, domain, type, category, tags
- tyhjän haun näkymä
- klikkaus domainiin (whitelist-tarkistus)

Älä vielä toteuta:

- logoja / favicon-hakua
- käyttäjän sijaintia
- ulkoista serveriä
- katselin.fi-julkista hakua
- ehdota sivua -lomaketta

---

## Seuraava askel

Vaiheet 1–5 on toteutettu. Seuraavaksi: **Vaihe 6** (crawler-indeksin laajennus, valinnainen) tai manuaalinen testaus paikallisella whitelistillä ja Meilisearchilla.

```powershell
.\scripts\sync-whitelist.ps1
$env:KOTISATAMA_WHITELIST_PATH = "index-data\cache\whitelist.json"
```
