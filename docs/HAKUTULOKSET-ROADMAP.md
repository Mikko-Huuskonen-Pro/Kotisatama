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

## Vaihe 1 — Hakurikastus

**Hakemisto:** `components/kotisatama/search/`

Tavoite: muuntaa Meilisearch-osuma rikastetuksi hakutulokseksi.

```rust
pub struct EnrichedSearchHit {
    pub url: String,
    pub title: String,           // crawlattu / Meilisearch
    pub label: Option<String>,   // whitelist
    pub category: Option<String>,
    pub entry_type: Option<String>,  // white | yellow
    pub tags: Vec<String>,
}
```

Tehtävät:

- [ ] `enrich_hit(hit: &SearchHit) -> EnrichedSearchHit` — host → `lookup_curated_entry`
- [ ] `search_enriched(query) -> Vec<EnrichedSearchHit>` — kääri nykyinen `search()`
- [ ] Yksikkötestit: whitelist-metatiedot liittyvät oikeaan URL:ään
- [ ] Fallback: jos whitelist-match puuttuu, näytä vain `title` + `url`

**Ei koske upstreamia.**

---

## Vaihe 2 — Sisäinen hakusivu (UI)

**Hakemistot:** `resources/resource_protocol/`, `components/kotisatama/i18n/`

Tavoite: `servo:haku?q=eläke` — klassinen hakukonenäkymä selaimessa (kuten `servo:avomeri`).

Tehtävät:

- [ ] `haku.html` + `haku.css` — hakukenttä, tuloslista, tyhjä näkymä
- [ ] SVG-ikonit kategorioille ja tyypeille (`icon`-kentän mukaan)
- [ ] `kotisatama-i18n.js` — käännökset (fi + sv)
- [ ] Tuloskortti: väripiste (`type`) + toimialaikoni (`category`) + label + domain + title
- [ ] Tyhjä tulos: ohjeet (tarkista kirjoitusasu, kokeile yleisempää, Avomeri-linkki)
- [ ] Klikkaus → `https://domain/...` (whitelist-tarkistus navigoinnissa)

**Ei koske upstreamia** (paitsi yksi `servo.rs`-case, ks. Vaihe 3).

---

## Vaihe 3 — Protokolla ja data-API

**Hakemisto:** `ports/servoshell/desktop/protocols/servo.rs` (minimaalinen patch)

Tehtävät:

- [ ] `servo:haku` → `haku.html` (kuten Avomeri)
- [ ] `servo:haku/data?q=…` → JSON: rikastetut tulokset + `categories` + `types` taulukot
- [ ] `kotisatama.rs`: `search_results_url(query) -> Url`

```rust
// servo.rs — yksi case, KOTISATAMA-PATCH
"haku" => ResourceProtocolHandler::response_for_path(..., "/haku.html"),
"haku/data" => { /* JSON response */ }
```

**Upstream-riski:** pieni, merkitty patch.

---

## Vaihe 4 — Osoitepalkin reititys

**Hakemisto:** `ports/servoshell/window.rs`, `desktop/gui.rs`

Nykyinen käyttäytyminen: haku avaa aina ensimmäisen osuman.

Uusi käyttäytyminen:

| Toiminto | Tulos |
|---|---|
| Enter + korkea varmuus (yksi selkeä osuma / alias) | Avaa paras osuma suoraan |
| Enter + epävarma haku | `servo:haku?q=…` |
| Hakupainike | `servo:haku?q=…` |
| Useita vahvoja osumia | `servo:haku?q=…` |

Tehtävät:

- [ ] `should_show_results_page(query, hits) -> bool` — `kotisatama.rs`
- [ ] Erillinen `UserInterfaceCommand::Search` hakupainikkeelle (jos erotetaan Enteristä)
- [ ] `window.rs`: reititys → `search_results_url` kun tulossivu tarvitaan
- [ ] Poista tai ohita egui-hakupaneeli (`gui.rs`) — HTML-sivu korvaa sen

**Upstream-riski:** pieni, merkitty patch `window.rs` + `gui.rs`.

---

## Vaihe 5 — Android

**Hakemistot:** `support/android/`, `ports/servoshell/egl/android/`

Tehtävät:

- [ ] Avaa `servo:haku?q=…` webviewissä (sama HTML kuin desktop)
- [ ] Poista/pienennä erillinen hakupaneeli jos se duplikoi toiminnon
- [ ] Testaa: haku → tulossivu → klikkaus → whitelist

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

Aloita **Vaihe 1** (hakurikastus `kotisatama-search`) kun whitelist 2.1 on synkattu suljetusta reposta kehitysympäristöön:

```powershell
.\scripts\sync-whitelist.ps1
$env:KOTISATAMA_WHITELIST_PATH = "index-data\cache\whitelist.json"
```
