# ROADMAP.md — Kotisatama (ilmainen perusversio)
*Kesäkuu 2026*

Tämä roadmap kattaa vain ilmaisen perusversion. Maksulliset versiot (Hopeakettu, Lapsi) suunnitellaan erikseen kun perusversio on valmis ja käyttödataa on kertynyt.

---

## Vaihe 1 — Whitelist-selain (desktop MVP)

**Tavoite:** Toimiva selain joka päästää läpi vain whitelistan domainit.

- [ ] Fork servo/servo, hakemistorakenne kuntoon (`components/kotisatama/`)
- [ ] `kotisatama-whitelist`-crate: domain-tarkistus, JSON-lataus
- [ ] `request_navigation`-hook `ports/servoshell/`-kerroksessa
- [ ] Geneerinen whitelist (`config/whitelist-free.json`) — ensimmäiset domainit käsin
- [ ] Virhenäyttö kun domain ei ole whitelistillä:
  - Selkeä viesti: *"Tätä sivua ei löydy kotisatamassa."*
  - Nappi: *"Jatka avomerelle"* → avaa Startpage hakusanalla
- [ ] `./mach build --release` toimii (desktop)

**Valmis kun:** Selain aukeaa, whitelist-domain läpäisee, tuntematon domain näyttää virhenäytön.

---

## Vaihe 2 — Hakukenttä ja paikallinen indeksi

**Tavoite:** Käyttäjä hakee laitteella olevasta indeksistä, ei internetistä.

- [ ] Hakukenttä servoshell-UI:ssa (desktop)
- [ ] `kotisatama-search`-crate: Meilisearch subprocess-käynnistys, HTTP-client (`http://127.0.0.1:7700`)
- [ ] Meilisearch bundlattu binäärinä appiin
- [ ] Hakutulos klikkaamalla avaa sivun selaimessa normaalisti
- [ ] Jos ei hakuosumia: *"Ei löydy kotisatamasta — haluatko hakea avomereltä?"* → Startpage samalla hakusanalla (`https://www.startpage.com/search?q=HAKUSANA`)

**Valmis kun:** Käyttäjä kirjoittaa "eläke", saa tuloksia, klikkaa, sivu aukeaa.

---

## Vaihe 3 — Crawler ja CDN-pipeline

**Tavoite:** Indeksi pysyy ajan tasalla automaattisesti.

- [ ] Crawler (`crawler/`): Node.js + Playwright, indeksoi whitelist-sivustot
- [ ] Crawler ajaa myös JS-renderöidyt SPA-sivustot oikein
- [ ] CI-pipeline: crawler ajaa viikoittain → Meilisearch-dump → CDN
- [ ] CDN-valinta: Cloudflare R2 tai Bunny (päätös avoin)
- [ ] `/free/whitelist.json` CDN:ssä — julkinen, ei API-avainta
- [ ] OTA-päivitys: app lataa uuden indeksin ja whitelist-JSONin taustalla

**Valmis kun:** Whitelist-päivitys näkyy laitteella ilman manuaalista toimenpidettä.

---

## Vaihe 4 — Raportointinappi

**Tavoite:** Käyttäjä voi ilmoittaa ongelmista ja ehdottaa uusia sivustoja.

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
- Pinkoodi / avomeri-porttilogiikka
- Tauri-hallintapaneeli
- Hakumainonta
- Pro-tili ja maksujärjestelmä

---

*Kotisatama on osa Ilio-toiminimeä (Y-tunnus 2010). Tekninen pohja: Servo (MPL 2.0) + Meilisearch (MIT).*
