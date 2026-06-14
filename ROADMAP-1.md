# ROADMAP-1.md — Kotisatama (ilmainen perusversio)
*Kesäkuu 2026*

Tämä roadmap kattaa vain ilmaisen perusversion. Maksulliset versiot (Hopeakettu, Lapsi) suunnitellaan erikseen kun perusversio on valmis ja käyttödataa on kertynyt.

Tekninen linja vastaa `README.md`, `AGENT.md` ja `kotisatama-konsepti-1.md`: Servo + servoshell, whitelist embedder-hookissa, Meilisearch subprocess, Android servoshell EGL.

---

## Nimistö ja tiedostopolut

| Konteksti | Polku / nimi |
|---|---|
| Paikallinen kehitys | `config/whitelist.json` (kopio `config/whitelist.example.json`) |
| Env-muuttuja | `KOTISATAMA_WHITELIST_PATH=config/whitelist.json` |
| CDN (ilmainen) | `/free/whitelist.json` |
| CDN (Pro, myöhemmin) | `/pro/whitelist.json` |
| Skeema | `config/whitelist.schema.json` |

Älä käytä eri paikallisia nimiä (`whitelist-free.json` jne.) — sama `whitelist.json` kehityksessä, CDN julkaisee `/free/`-polun.

---

## Avomeri — mitä kuuluu ja mitä ei

**Kuuluu ilmaiseen perusversioon (vaiheet 1–2):**

- Kun navigointi blokataan tai haku ei löydä osumia, käyttäjä näkee nappi *"Jatka avomerelle"* / *"Hae avomereltä"*
- Nappi avaa **Startpage**-hakua (`https://www.startpage.com/search?q=...`) — käyttäjä menee avomerelle vain tietoisesti
- Ei pinkoodia, ei porttia, ei erillistä avomeri-näkymää

**Ei kuulu ilmaiseen perusversioon (myöhemmin):**

- Pinkoodi / avomeri-portti (lapsi- ja Hopeakettu-logiikka)
- Rajattu avomeri-sessio, aikarajat, ostajan hallinta
- Tauri-hallintapaneeli avomeri-asetuksille

---

## Vaihe 1 — Whitelist-selain (desktop MVP)

**Tavoite:** Toimiva selain joka päästää läpi vain whitelistan domainit.

- [x] Fork servo/servo, hakemistorakenne kuntoon (`components/kotisatama/`)
- [x] `kotisatama-whitelist`-crate: domain-tarkistus, JSON-lataus
- [x] `request_navigation`-hook `ports/servoshell/`-kerroksessa (KOTISATAMA-PATCH)
- [x] Geneerinen whitelist: `config/whitelist.json` — ensimmäiset domainit käsin
- [x] Virhenäyttö kun domain ei ole whitelistillä:
  - Selkeä viesti: *"Tätä sivua ei löydy kotisatamassa."*
  - Nappi: *"Jatka avomerelle"* → Startpage hakusanalla (ks. Avomeri-osio)
- [ ] `./mach build --release` toimii (desktop) — vaatii MSVC Windowsilla
- [ ] `cargo build` toimii ilman `--features kotisatama` (upstream ei rikkoudu)

**Valmis kun:** Selain aukeaa, whitelist-domain läpäisee, tuntematon domain näyttää virhenäytön.

---

## Vaihe 2 — Hakukenttä ja paikallinen indeksi

**Tavoite:** Käyttäjä hakee laitteella olevasta indeksistä, ei internetistä.

**Indeksin lähde ennen vaihe 3:** Crawler/CDN ei ole vielä valmis. Vaiheessa 2 käytetään **manuaalista testi-indeksia**:

- Kertaluontoinen crawl paikallisesti (`crawler/` tai Meilisearch käsin), tai
- Pieni testi-dump (`index-dump/`) bundlattu appiin kehitystä varten

Automaattinen päivitys tulee vasta vaiheessa 3.

- [x] Hakukenttä servoshell-UI:ssa (desktop)
- [x] `kotisatama-search`-crate: Meilisearch subprocess-käynnistys, HTTP-client (`http://127.0.0.1:7700`)
- [ ] Meilisearch bundlattu binäärinä appiin (dev: asenna PATH / `KOTISATAMA_MEILISEARCH_BIN`)
- [x] Testi-indeksi: `config/search-index/documents.json` (seed automaattisesti)
- [x] Hakutulos klikkaamalla avaa sivun selaimessa normaalisti
- [x] Jos ei hakuosumia: *"Ei löydy kotisatamasta — haluatko hakea avomereltä?"* → Startpage

**Valmis kun:** Käyttäjä kirjoittaa "eläke", saa tuloksia testi-indeksistä, klikkaa, sivu aukeaa.

---

## Vaihe 3 — Crawler ja CDN-pipeline

**Tavoite:** Indeksi ja whitelist pysyvät ajan tasalla automaattisesti (korvaa manuaalinen testi-indeksi).

- [x] Crawler (`crawler/`): Node.js + Playwright, indeksoi whitelist-sivustot
- [x] Crawler ajaa myös JS-renderöidyt SPA-sivustot oikein
- [x] CI-pipeline: crawler ajaa viikoittain → Meilisearch-dump → CDN-artifakti
- [ ] CDN-valinta: Cloudflare R2 tai Bunny (päätös avoin — artifact valmis julkaisuun)
- [x] `/free/whitelist.json` CDN-paketissa — julkinen rakenne
- [x] OTA-päivitys: `KOTISATAMA_CDN_BASE` lataa whitelist + indeksidumpin käynnistyksessä

**Valmis kun:** Whitelist- ja indeksipäivitys näkyy laitteella ilman manuaalista toimenpidettä.

---

## Vaihe 4 — Raportointinappi

**Tavoite:** Käyttäjä voi ilmoittaa ongelmista ja ehdottaa uusia sivustoja.

Serverless-poikkeus: raportit käyttävät **Cloudflare Worker**-endpointia (ei omaa palvelinta, sama malli kuin CDN).

- [x] Raportointinappi osoitepalkissa ja virhenäytöllä (ei avomerellä)
- [x] Kaksi raporttityyppiä:
  - *"Sivusto ei toimi"* — lähettää domain + vapaa tekstikenttä
  - *"Ehdota kotisatamaan"* — lähettää domain
- [x] Raporttien vastaanotto — Cloudflare Worker + KV (`worker/report/`), valinnainen `WEBHOOK_URL`
- [x] Raportti lähtee anonyymisti, ei käyttäjätunnistetta

**Valmis kun:** Nappi näkyy, raportti lähtee, tieto tulee perille.

---

## Vaihe 5 — Android

**Tavoite:** Sama kokemus mobiililla.

- [x] Android-build Servon omaa polkua: `ports/servoshell/egl/android/` (JNI + ANativeWindow)
- [x] `./mach build --target aarch64-linux-android --profile checked-release` (dokumentoitu `support/android/README.md`)
- [x] APK testattavissa: `target/aarch64-linux-android/checked-release/servoapp.apk`
- [x] Kaikki vaiheissa 1–4 toteutetut ominaisuudet toimivat mobiililla (whitelist Rust, haku/raportti Java UI + JNI)
- [x] Meilisearch-binääri bundlattu Android-APK:hun (`fetch-meilisearch.sh`, assets → `KotisatamaAssets`)

**Valmis kun:** APK toimii puhelimella, whitelist ja haku toimivat offline.

---

## Vaihe 6 — Julkaisu ja datankeruu

**Tavoite:** Oikeat käyttäjät, oikea data whitelist-kehityksen pohjaksi.

- [x] Julkinen beta (Android + desktop) — `BETA.md`, build-ohjeet
- [x] Fallback-haut lokataan: hakusanat joille ei löydy osumia indeksistä (paikallinen JSONL + Worker `/fallback`)
- [x] Raportit käydään läpi → whitelist kasvaa oikean käytön perusteella (`scripts/triage-kotisatama-data.mjs`)
- [x] Indeksin päivitystiheys tarkistetaan käytön valossa (triage-skripti + crawl workflow -ohje)

**Valmis kun:** Ensimmäiset ulkopuoliset käyttäjät käyttävät, fallback-data kertyy.

---

## Vaihe 7 - teemat

assets/themes löytyy eri tiloihin teemakuvat tekstin taakse. Upotetaan kuvatekstin taakse, eli Satama on etusivu, whitelistatut hakutulokset. Avomeri avautuu kun painaa avomeri painiketta. Myrsky on offline-tilan teema, ei pääse vesille=ei pääse nettiin.

Kuvat ovat staattisia bundlattuja assetteja — ei CDN-hakua, toimii offline-tilassa


---


## Avoimet päätökset

- [ ] CDN-valinta: Cloudflare R2 vai Bunny
- [ ] Raportit: Cloudflare Worker → minne (sähköposti / Sheet / Airtable)
- [ ] Whitelist-domainien ensimmäinen lista: mitkä 50–100 sivustoa mukaan heti?
- [ ] Indeksointisyvyys crawlerissa: kuinka monta tasoa per domain?

---

## Ei kuulu tähän vaiheeseen

Nämä siirretään myöhempään — ei ennen kuin perusversio on käytössä ja dataa on:

- Hopeakettu-profiili
- Lapsi-profiili
- Pinkoodi ja avomeri-portti (Startpage-fallback on jo perusversiossa — ks. Avomeri-osio)
- Tauri-hallintapaneeli
- Hakumainonta
- Pro-tili ja maksujärjestelmä (`/pro/whitelist.json`, API-avain)

---

*Kotisatama on osa Ilio-toiminimeä (Y-tunnus 2010). Tekninen pohja: Servo + servoshell (MPL 2.0), Meilisearch subprocess (MIT), CDN serverless.*
