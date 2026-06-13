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

- [ ] Fork servo/servo, hakemistorakenne kuntoon (`components/kotisatama/`)
- [ ] `kotisatama-whitelist`-crate: domain-tarkistus, JSON-lataus
- [ ] `request_navigation`-hook `ports/servoshell/`-kerroksessa (KOTISATAMA-PATCH)
- [ ] Geneerinen whitelist: `config/whitelist.json` — ensimmäiset domainit käsin
- [ ] Virhenäyttö kun domain ei ole whitelistillä:
  - Selkeä viesti: *"Tätä sivua ei löydy kotisatamassa."*
  - Nappi: *"Jatka avomerelle"* → Startpage hakusanalla (ks. Avomeri-osio)
- [ ] `./mach build --release` toimii (desktop)
- [ ] `cargo build` toimii ilman `--features kotisatama` (upstream ei rikkoudu)

**Valmis kun:** Selain aukeaa, whitelist-domain läpäisee, tuntematon domain näyttää virhenäytön.

---

## Vaihe 2 — Hakukenttä ja paikallinen indeksi

**Tavoite:** Käyttäjä hakee laitteella olevasta indeksistä, ei internetistä.

**Indeksin lähde ennen vaihe 3:** Crawler/CDN ei ole vielä valmis. Vaiheessa 2 käytetään **manuaalista testi-indeksia**:

- Kertaluontoinen crawl paikallisesti (`crawler/` tai Meilisearch käsin), tai
- Pieni testi-dump (`index-dump/`) bundlattu appiin kehitystä varten

Automaattinen päivitys tulee vasta vaiheessa 3.

- [ ] Hakukenttä servoshell-UI:ssa (desktop)
- [ ] `kotisatama-search`-crate: Meilisearch subprocess-käynnistys, HTTP-client (`http://127.0.0.1:7700`)
- [ ] Meilisearch bundlattu binäärinä appiin
- [ ] Testi-indeksi: manuaalinen dump tai kertaluontoinen crawl (ei vaadi CI/CD)
- [ ] Hakutulos klikkaamalla avaa sivun selaimessa normaalisti
- [ ] Jos ei hakuosumia: *"Ei löydy kotisatamasta — haluatko hakea avomereltä?"* → Startpage (`https://www.startpage.com/search?q=HAKUSANA`)

**Valmis kun:** Käyttäjä kirjoittaa "eläke", saa tuloksia testi-indeksistä, klikkaa, sivu aukeaa.

---

## Vaihe 3 — Crawler ja CDN-pipeline

**Tavoite:** Indeksi ja whitelist pysyvät ajan tasalla automaattisesti (korvaa manuaalinen testi-indeksi).

- [ ] Crawler (`crawler/`): Node.js + Playwright, indeksoi whitelist-sivustot
- [ ] Crawler ajaa myös JS-renderöidyt SPA-sivustot oikein
- [ ] CI-pipeline: crawler ajaa viikoittain → Meilisearch-dump → CDN
- [ ] CDN-valinta: Cloudflare R2 tai Bunny (päätös avoin)
- [ ] `/free/whitelist.json` CDN:ssä — julkinen, ei API-avainta
- [ ] OTA-päivitys: app lataa uuden indeksin ja whitelist-JSONin taustalla

**Valmis kun:** Whitelist- ja indeksipäivitys näkyy laitteella ilman manuaalista toimenpidettä.

---

## Vaihe 4 — Raportointinappi

**Tavoite:** Käyttäjä voi ilmoittaa ongelmista ja ehdottaa uusia sivustoja.

Serverless-poikkeus: raportit käyttävät **Cloudflare Worker**-endpointia (ei omaa palvelinta, sama malli kuin CDN).

- [ ] Raportointinappi osoitepalkissa ja virhenäytöllä (ei avomerellä)
- [ ] Kaksi raporttityyppiä:
  - *"Sivusto ei toimi"* — lähettää domain + vapaa tekstikenttä
  - *"Ehdota kotisatamaan"* — lähettää domain
- [ ] Raporttien vastaanotto — päätös auki:
  - Cloudflare Worker → sähköposti / Google Sheet / Airtable
- [ ] Raportti lähtee anonyymisti, ei käyttäjätunnistetta

**Valmis kun:** Nappi näkyy, raportti lähtee, tieto tulee perille.

---

## Vaihe 5 — Android

**Tavoite:** Sama kokemus mobiililla.

- [ ] Android-build Servon omaa polkua: `ports/servoshell/egl/android/` (JNI + ANativeWindow)
- [ ] `./mach build --target aarch64-linux-android --profile checked-release`
- [ ] APK testattavissa: `target/aarch64-linux-android/checked-release/servoapp.apk`
- [ ] Kaikki vaiheissa 1–4 toteutetut ominaisuudet toimivat mobiililla
- [ ] Meilisearch-binääri bundlattu Android-APK:hun

**Valmis kun:** APK toimii puhelimella, whitelist ja haku toimivat offline.

---

## Vaihe 6 — Julkaisu ja datankeruu

**Tavoite:** Oikeat käyttäjät, oikea data whitelist-kehityksen pohjaksi.

- [ ] Julkinen beta (Android + desktop)
- [ ] Fallback-haut lokataan: hakusanat joille ei löydy osumia indeksistä (ei avomeri-dataa, ei henkilötietoja)
- [ ] Raportit käydään läpi → whitelist kasvaa oikean käytön perusteella
- [ ] Indeksin päivitystiheys tarkistetaan käytön valossa

**Valmis kun:** Ensimmäiset ulkopuoliset käyttäjät käyttävät, fallback-data kertyy.

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
