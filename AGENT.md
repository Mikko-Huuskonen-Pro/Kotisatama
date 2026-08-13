# AGENT.md — Kotisatama kehitysohjeet

Tämä tiedosto ohjaa koodiagentin toimintaa **moottorirepossa** (Servo-forkki).
Lue ennen kuin teet muutoksia.

**Ekosysteemi:** [docs/EKOSYSTEEMI.md](docs/EKOSYSTEEMI.md) — mitä on missäkin
(Katselin, Avomeri, adblock, haku, suljetut osat, Varustamo).

Tämä repo ≠ Katselin-tuote. Android-APK ja orkestrointi: `../Katselin` tai `../Avomeri`.

---

## Tärkein sääntö

**Älä koskaan muokkaa Servo-upstream-tiedostoja suoraan.**

Kaikki Kotisatama-spesifinen koodi elää `components/kotisatama/`-hakemistossa, `ports/servoshell/`-hookissa tai erillisissä hakemistoissa (`tauri/`, `crawler/`). Jos jokin muutos tuntuu vaativan upstream-tiedoston muokkaamista, pysähdy ja kysy ensin.

### Kommenttikieli — 注释语言

Servo on 100 % englanniksi. Kotisataman omat kommentit (`//`, `<!-- -->` ym.) kirjoitetaan **suomeksi ensisijaisesti ja mandariinikiinaksi käännöksenä** samalla rivillä tai seuraavalla. Näin sekä suomenkieliset kehittäjät että kiinan- tai englanninkieliset tekoälyagentit ymmärtävät tarkoituksen.

Muoto:

```rust
// KOTISATAMA-PATCH: suomenkielinen selitys — 中文翻译
```

```xml
<!-- KOTISATAMA-PATCH: suomenkielinen selitys — 中文翻译 -->
```

Suomi on aina ensimmäinen, kiina tulee ajatusviivan `—` jälkeen. Vanhat kommentit päivitetään samaan muotoon kun tiedostoa muutenkin muokataan.

---

## Hakemistorakenne

```
kotisatama/                      ← tämä moottorirepo
├── components/
│   ├── kotisatama/              ← KAIKKI omat Rust-muutokset tänne
│   │   ├── whitelist/           ← Whitelist-logiikka
│   │   ├── search/              ← Meilisearch-client (HTTP → paikallinen prosessi)
│   │   ├── content-blocking/    ← adblock-Katselin adapter
│   │   ├── pulloposti/          ← Pulloposti subprocess-client
│   │   ├── missa-olen/          ← Missä olen subprocess-client
│   │   ├── varustamo/           ← Sovellusrekisteri (UI)
│   │   ├── report/              ← Ilmoitukset
│   │   ├── subprocess-app/      ← Yhteinen subprocess-kehys
│   │   └── i18n/                ← FI/SV
│   └── [servo-upstream…]        ← ÄLÄ KOSKE (paitsi minimaalinen KOTISATAMA-PATCH)
├── ports/
│   └── servoshell/              ← embedder-hook: navigointi, haku, content-blocking, JNI
├── tauri/                       ← Hallintapaneeli (ei selainmoottori; ei aktiivinen)
├── crawler/                     ← Node.js, täysin erillinen
└── [servo-upstream-files…]      ← ÄLÄ KOSKE
```

Sisaruscheckout (oletuskehitys):

```
../adblock-Katselin/             ← path-dep content-blockingista
../Katselin/                     ← suljettu tuote + Android APK
../Avomeri/                      ← avoin tuote + Android APK
../Katselin-haku/                ← Meilisearch Android
../Katselin-Consent-O-Matic/     ← eväste-JS (Katselin bundlaa)
../Kotisataman-suljetut-osat/    ← valkoiset sivut, daemonit, vanha Varustamo
../Varustamo/                    ← uusi Varustamo — ei vielä integroitu
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

### Whitelist: embedder-hook (ensisijainen polku)

Servo tarjoaa navigointihook embedder-kerroksessa (`NavigationRequest::allow()` / `deny()`). **Älä toteuta whitelistia tai muita moduulien lisäystä `components/net/` tai `components/script/` -tasolla** — käytä `ports/servoshell/`.

Logiikka pysyy `kotisatama-whitelist`-cratessa. Servoshell kutsuu sitä `WebViewDelegate::request_navigation`-metodissa:

```rust
// ports/servoshell/running_app_state.rs — esimerkki (KOTISATAMA-PATCH)
impl WebViewDelegate for RunningAppState {
    fn request_navigation(&self, _webview: WebView, request: NavigationRequest) {
        if kotisatama_whitelist::is_allowed(&request.url) {
            request.allow();
        } else {
            request.deny();
        }
    }
}
```

Tämä on parempi kuin muutos `components/net/`-tasolla, koska upstream-muutokset `ports/servoshell/`-hakemistossa ovat harvemmat kuin `components/net/` tai `components/script/`.

### Integrointi Servoon feature flagilla (vain jos embedder ei riittää)

Jos hook täytyy tehdä upstream-komponentissa, käytä feature flagia. Näin upstream-merge ei riko mitään vaikka feature olisi pois päältä:

```toml
# Cargo.toml
[features]
kotisatama = ["kotisatama-whitelist", "kotisatama-search"]
```

```rust
// Upstream-tiedostossa — minimaalinen hook, ei logiikkaa
#[cfg(feature = "kotisatama")]
if !kotisatama_whitelist::is_allowed(&url) {
  // palauta upstreamin oma deny/blokki-malli — ei NavigationResult::Blocked
}
```

Logiikka pysyy omassa cratessamme. Upstream-tiedostoon koskee vain feature-flagin taakse kääritty yksi kutsu.

### Konfliktiherkkä koodi — miten tunnistetaan

Upstream-konflikteja syntyy kun muokataan tiedostoa jota Servo-tiimi myös muokkaa aktiivisesti. Vältä muutoksia näihin:

- `components/script/`
- `components/layout/`
- `components/net/`
- `Cargo.lock` (synkronoituu automaattisesti mergessä)

Suosi `ports/servoshell/` embedder-hookia whitelistille. Jos muutos on pakko tehdä upstream-komponenttiin, tee se mahdollisimman pieneksi ja merkitse kommentilla:

```rust
// KOTISATAMA-PATCH: syy tälle muutokselle
// Upstream PR: ei auki / auki osoitteessa <url>
// Revisit: kun servo/servo#XXXX mergetään
```

---

## Android — servoshell EGL, ei Tauri

Android-build käyttää Servon omaa polkua:

- `ports/servoshell/egl/android/` — JNI + `ANativeWindow` (**tässä repossa**)
- APK-paketointi: **Katselin** (`../Katselin/android/apk`) tai **Avomeri** (`../Avomeri/android/apk`)
- Paikallinen junction Katselin-APK:han: `./scripts/link-android-apk.ps1` / `.sh` → `support/android/apk` (ei gittiin)

**Tauri ei kantaa Servo-moottoria Androidilla.** Tauri 2.0 käyttää Androidilla System WebViewia (Chromium). Älä yritä wrappaa Servoa Taurin sisään selaimen kantajana.

Parimuutokset (JNI tässä + Kotlin shellissä): tee ensin moottori-PR, sitten UI-PR tuoteshellissä.

---

## Tauri — hallintapaneeli, ei selain (Ei totoutuksessa tällä hetkellä) 

Tauri-app on täysin erillinen hakemisto. Se **ei onnistuneena wrappaa Servo-moottoria** — se on vanhemman hallintapaneeli whitelistin ja asetuksien hallintaan (web/desktop).

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

### Kotisatama-cratet Tauri-backendissä

Hallintapaneeli käyttää vain omia `kotisatama/`-crateä — ei Servon sisäisiä crateä:

```toml
# tauri/src-tauri/Cargo.toml
[dependencies]
kotisatama-whitelist = { path = "../../components/kotisatama/whitelist" }
```

Tauri-backend ei importoi Servon sisäisiä crateä suoraan.

### Upstream-päivitys ei koske Tauriin

Kun synkronoidaan upstream:

```bash
git fetch upstream
git merge upstream/main
```

`tauri/`-hakemisto ei onnistuneena saa merge-konflikteja upstream:ista — se on vain tässä forkissa. Tämä on tarkoituksellinen design-päätös.

---

## Whitelist — skeema 2.1

Kuratoitu lista (`whitelist-unified.json`) on versio **2.1**. Rakenne:

```json
{
  "version": "2.1",
  "categories": [ { "id": "health", "label": "…", "icon": "health" } ],
  "types": [ { "id": "white", … }, { "id": "yellow", … } ],
  "domains": [ { "domain": "kela.fi", "label": "Kela", "category": "health", "type": "white", "tags": [] } ]
}
```

- **Valkoiset ja keltaiset sivut** ovat samassa `domains`-listassa (`type`: `white` | `yellow`).
- **`category`** viittaa `categories[].id`-kenttään (toimialaikoni UI:ssa).
- **`type`** viittaa `types[].id`-kenttään (väripiste UI:ssa).
- Tuotantodata pysyy suljetussa repossa; julkinen repo sisältää skeeman ja esimerkin.

Skeema: `config/whitelist.schema.json`. Esimerkki: `config/whitelist.example.json`.

Turvallisuus ja profiilit: `docs/TURVALLISUUS-PROFIILIT.md`.

Ekosysteemi: `docs/EKOSYSTEEMI.md`. Vanhat suunnitelmat:
`../Kotisataman-suljetut-osat/Docs/legacy/` (mm. hakutulokset, adblock-suunnitelma, Puhdistukset).

---

## Haku — Meilisearch subprocess

Meilisearch on palvelinprosessi (LMDB). Mobiilissa ja desktopilla:

1. Crawler (CI) indeksoi whitelist-sivustot → Meilisearch-dump
2. Dump CDN:ään
3. Laitteella: bundlattu Meilisearch-binääri käynnistyy subprocessina, importaa dumpin
4. `kotisatama-search` kysyy paikallista instanssia HTTP:llä (`http://127.0.0.1:7700`)

**Älä yritä upottaa Meilisearchia kirjastotasolla** — se ei onnistuneena upoteta mobiiliappiin. `components/kotisatama/search/` on HTTP-client ja prosessinhallinta, ei Meilisearch-core.

---

## Upstream-synkronointi — prosessi

```bash
# 1. Haetaan upstream
git fetch upstream

# 2. Katsotaan mitä tulee
git log HEAD..upstream/main --oneline

# 3. Mergetään
git merge upstream/main

# 4. Jos konflikteja — ne ovat KOTISATAMA-PATCH-kohdissa upstream-tiedostoissa
git diff --name-only --diff-filter=U
# Tyypillisesti: ports/servoshell/, ei components/kotisatama/

# 5. Resolvataan konflikti: ota upstream-muutos, lisää oma patch perään
# ÄLÄ hylkää upstream-muutosta kokonaan

# 6. Tarkista että feature-flag toimii
cargo build --features kotisatama
cargo build  # Pitää myös toimia ilman featurea
```

---

## Upstream-merge — Agentin vastuulla

Upstream-synkronointi (`git merge upstream/main`) tehdään **Agentilla**,
ei manuaalisesti, jotta ei tuhota kaikki puhdistuksessa muutettuja tiedostoja.

### Ennen muutoksia: tarkista suljetun repon Docs/legacy/Puhdistukset.md

`../Kotisataman-suljetut-osat/Docs/legacy/Puhdistukset.md` on rekisteri tehdyistä
Servo-siivousmuutoksista (siirretty moottorista legacyyn 2026-08).
Ennen kuin muutat tiedostoa joka sisältää Servo-viittauksen, tarkista:
onko tämä tiedosto jo käsitelty ja onko muutos tarkoituksellinen?

Älä "korjaa" Kotisatama-muutosta takaisin Servo-oletukseksi vaikka se
näyttäisi upstream:ista poikkeavalta — se on todennäköisesti tarkoituksellinen.

### Tiedostoluokat

| Luokka | Esimerkki | Upstream ylikirjoittaa? |
|---|---|---|
| Kotisatama-omat | `components/kotisatama/`, `tauri/`, `crawler/` | Ei koskaan |
| Patchattu upstream | `ports/servoshell/`, `resources/resource_protocol/newtab.html` | Voi — katso KOTISATAMA-PATCH-kommentit |
| Puhdistettu upstream | ks. suljetun repon `Docs/legacy/Puhdistukset.md` | Voi — Cursor ratkaisee konfliktin |
| Koskematon upstream | `components/script/`, `tests/wpt/` | Kyllä, tarkoituksella |

---

---

## Mitä agentin pitää tarkistaa ennen PR:ää

- [ ] `cargo build` toimii ilman `--features kotisatama` (upstream ei rikkoudu)
- [ ] `cargo build --features kotisatama` toimii
- [ ] Muutoksia ei ole `components/[upstream]/`-tiedostoissa ilman `KOTISATAMA-PATCH`-kommenttia
- [ ] Whitelist-logiikka on `components/kotisatama/whitelist/`, hook `ports/servoshell/`
- [ ] `./mach build --release` toimii (desktop)
- [ ] `tauri/`-hakemisto buildaa itsenäisesti: `cd tauri && npm run tauri build` (jos muutettu)
- [ ] Uudet tiedostot ovat `components/kotisatama/`-alla, `ports/servoshell/`-hookissa tai `tauri/`/`crawler/`-hakemistoissa

---

## Kielivalinnat

| Konteksti | Kieli |
|---|---|
| Whitelist-logiikka | Rust (`components/kotisatama/whitelist`) |
| Haku-integraatio | Rust (`components/kotisatama/search`) — HTTP-client |
| Embedder-hook | Rust (`ports/servoshell/`) |
| Tauri-hallintapaneeli | Rust + TypeScript (`tauri/`) |
| Crawler | Node.js + Playwright |
| Whitelist JSON | JSON v2.1, skeema `config/whitelist.schema.json` |

---

## Toteutusjärjestys (suositus — vastaa suljetun repon `Docs/ROADMAP-1.md`)

1. `components/kotisatama/whitelist` + `request_navigation`-hook servoshellissa
2. Hakukenttä servoshell-UI:ssa + Meilisearch subprocess + `kotisatama-search` (testi-indeksi manuaalisesti)
3. Crawler + CDN-pipeline + OTA
4. Raportointinappi (Cloudflare Worker)
5. Android: servoshell EGL (`./mach build --target aarch64-linux-android`)
6. Julkaisu ja datankeruu
7. Tauri-hallintapaneeli (valinnainen, erillinen — Pro-versioiden yhteydessä)

---

*Päivitetty: elokuu 2026*
