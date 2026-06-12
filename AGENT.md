# AGENT.md — Kotisatama kehitysohjeet

Tämä tiedosto ohjaa koodiagentin toimintaa Kotisatama-repossa. Lue ennen kuin teet muutoksia.

---

## Tärkein sääntö

**Älä koskaan muokkaa Servo-upstream-tiedostoja suoraan.**

Kaikki Kotisatama-spesifinen koodi elää `components/kotisatama/`-hakemistossa tai erillisissä Tauri-hakemistoissa. Jos jokin muutos tuntuu vaativan upstream-tiedoston muokkaamista, pysähdy ja kysy ensin.

---

## Hakemistorakenne 

```
kotisatama/
├── components/
│   ├── kotisatama/          ← KAIKKI omat Rust-muutokset tänne
│   │   ├── whitelist/       ← Whitelist-logiikka
│   │   ├── search/          ← Meilisearch-integraatio
│   │   └── lib.rs
│   └── [servo-upstream]/    ← ÄLÄ KOSKE
├── tauri/                   ← Tauri-app, täysin erillinen
│   ├── src/                 ← TypeScript/JS UI
│   └── src-tauri/           ← Tauri Rust-backend
├── crawler/                 ← Node.js, täysin erillinen
└── [servo-upstream-files]/  ← ÄLÄ KOSKE
```

---

## Rust — miten lisätään toiminnallisuus

### Tee oma crate, älä muokkaa olemassa olevia

Uusi toiminnallisuus lisätään aina uutena cratenä `components/kotisatama/`-alle:

```bash
# Oikein
components/kotisatama/whitelist/
components/kotisatama/search/

# Väärin
components/script/kotisatama_whitelist.rs  ← upstream-hakemisto
```

### Integrointi Servoon feature flagilla

Jos Servo-koodi täytyy kutsua omaa crates, käytä feature flagia. Näin upstream-merge ei riko mitään vaikka feature olisi pois päältä:

```toml
# Cargo.toml
[features]
kotisatama = ["kotisatama-whitelist", "kotisatama-search"]
```

```rust
// Servo-upstream-tiedostossa — minimaalinen hook, ei logiikkaa
#[cfg(feature = "kotisatama")]
use kotisatama_whitelist::is_allowed;

#[cfg(feature = "kotisatama")]
if !is_allowed(&url) {
    return NavigationResult::Blocked;
}
```

Logiikka pysyy omassa cratessamme. Upstream-tiedostoon koskee vain feature-flagin taakse kääritty yksi kutsu.

### Konfliktiherkkä koodi — miten tunnistetaan

Upstream-konflikteja syntyy kun muokataan tiedostoa jota Servo-tiimi myös muokkaa aktiivisesti. Vältä muutoksia näihin:

- `components/script/`
- `components/layout/`
- `components/net/`
- `Cargo.lock` (synkronoituu automaattisesti mergessä)

Jos muutos on pakko tehdä näihin, tee se mahdollisimman pieneksi ja merkitse kommentilla:

```rust
// KOTISATAMA-PATCH: syy tälle muutokselle
// Upstream PR: ei auki / auki osoitteessa <url>
// Revisit: kun servo/servo#XXXX mergetään
```

---

## Tauri — miten pidetään erillään

Tauri-app on täysin erillinen hakemisto. Se ei periydy Servosta — se käyttää Servoa komponenttina.

### Rakenne

```
tauri/
├── src/                  ← UI (TypeScript, React/HTML)
├── src-tauri/
│   ├── Cargo.toml        ← OMA Cargo.toml, ei Servon
│   ├── tauri.conf.json
│   └── src/
│       └── main.rs       ← Tauri entry point
└── package.json
```

### Servo Tauri-backendissä

Servo integroidaan Tauri-backendiin cratenä, ei forkkina:

```toml
# tauri/src-tauri/Cargo.toml
[dependencies]
kotisatama-whitelist = { path = "../../components/kotisatama/whitelist" }
kotisatama-search = { path = "../../components/kotisatama/search" }
```

Tauri-backend ei importoi Servon sisäisiä crateä suoraan — vain omia `kotisatama/`-crateä.

### Upstream-päivitys ei koske Tauriin

Kun synkronoidaan upstream:

```bash
git fetch upstream
git merge upstream/main
```

`tauri/`-hakemisto ei ole osa Servo-repoa — se ei koskaan saa merge-konflikteja upstream:ista. Tämä on tarkoituksellinen design-päätös.

---

## Upstream-synkronointi — prosessi

```bash
# 1. Haetaan upstream
git fetch upstream

# 2. Katsotaan mitä tulee
git log HEAD..upstream/main --oneline

# 3. Mergetään
git merge upstream/main

# 4. Jos konflikteja — ne ovat lähes aina KOTISATAMA-PATCH-kohdissa
git diff --name-only --diff-filter=U

# 5. Resolvataan konflikti: ota upstream-muutos, lisää oma patch perään
# ÄLÄ hylkää upstream-muutosta kokonaan

# 6. Tarkista että feature-flag toimii
cargo build --features kotisatama
cargo build  # Pitää myös toimia ilman featurea
```

---

## Mitä agentin pitää tarkistaa ennen PR:ää

- [ ] `cargo build` toimii ilman `--features kotisatama` (upstream ei rikkoudu)
- [ ] `cargo build --features kotisatama` toimii
- [ ] Muutoksia ei ole `components/[upstream]/`-tiedostoissa ilman `KOTISATAMA-PATCH`-kommenttia
- [ ] `tauri/`-hakemisto buildaa itsenäisesti: `cd tauri && cargo build`
- [ ] Uudet tiedostot ovat `components/kotisatama/`-alla tai `tauri/`-alla

---

## Kielivalinnat

| Konteksti | Kieli |
|---|---|
| Whitelist-logiikka | Rust (`components/kotisatama/whitelist`) |
| Hakuindeksi-integraatio | Rust (`components/kotisatama/search`) |
| Tauri-backend | Rust (`tauri/src-tauri`) |
| Tauri-UI | TypeScript |
| Crawler | Node.js + Playwright |
| Whitelist JSON | JSON, skeema `config/whitelist.schema.json` |

---

*Päivitetty: kesäkuu 2026*
