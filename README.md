# Katselin

> Finnish-first browser built on Servo.

## What is Katselin?

Katselin is a Servo-based browser focused on a curated browsing experience.

Instead of relying on a traditional search engine, Katselin uses a locally stored
search index and a whitelist of trusted websites. The goal is to make everyday
browsing simple, predictable and safe while remaining fully open source.

The browser is developed in Finnish first. Community translations are welcome,
but Finnish is considered the primary language of the project.

---

## Why another Servo fork?

Katselin experiments with a different browser philosophy rather than a different
rendering engine.

Servo remains responsible for rendering the web.

Katselin builds user experience on top of Servo:
- curated search
- locally indexed content
- harbour-based navigation model
- Brave's ad-block
- simple UI designed for all ages

Whenever possible, fixes are intended to be contributed upstream to Servo.

---

## Project status

- Adblock integration is on process
- First win 11 test user aqruired
- Android version is broken mess, try to find time to fix it to the working state. Win11 version is working

## Mikä on katselin? 

Katselin on Servo-pohjainen selain whitelist-pohjaisella hakumallilla. Selain ja
esiladattu hakuindeksi muodostavat suljetun ympäristön: käyttäjä löytää kaiken
tarvitsemansa ilman että eksyy avomerelle.

Kotisatama on projektin aiempi nimi ja säilyy toistaiseksi teknisissä poluissa,
crate-nimissä, moduuleissa ja meriteemaisessa käyttöliittymäkielessä. Nimeä
vaihdetaan asteittain Katseliniksi, jotta toimivaa koodia ja build-skriptejä ei
rikota yhdellä isolla uudelleennimeämisellä.

> Tämä repo on fork [servo/servo](https://github.com/servo/servo). Kotisatama-spesifiset muutokset on eriytetty omiin moduuleihinsa. Upstream-muutokset julkaistaan MPL 2.0:n mukaisesti. Koska servo ei ole valmis, edetään sivu kerrallaan kotisatamassa. Kun käyttäjä pysyy satamassa, ne sivut pitää toimia ja latautua oikein. Jos lähtee avomerelle, eli kiertää whitelistauksen, sivujen toimivuus on Servon kehityksen varassa. Tietenkin kun ratkotaan whitelistattujen sivujen ongelmia, samalla myöskin kotisatama paranee verrattuna Servoon. Kuitenkin siihen pisteeseen on matkaan, että päästäisiin antamaan Servolle takaisin contribuutiota. \\ MH 13.6.2026

---

## Arkkitehtuuri

```
[Kotisatama — Servo-fork]
    ├── components/kotisatama/whitelist   ← whitelist-logiikka
    ├── components/kotisatama/search      ← haku-API (Meilisearch-client)
    └── ports/servoshell                  ← embedder-hook (navigointi, UI)

[Android — servoshell EGL]
    └── support/android/apk + JNI-host    ← ei Tauri; Servon oma Android-polku

[CDN — staattinen]
    ├── /free/whitelist.json
    └── /pro/whitelist.json  (API-avain vaaditaan)

[Crawler — CI-prosessi]
    └── Playwright → Meilisearch-dump → CDN
```

Ei omaa palvelinta. Ei VPN-infraa. Haku tapahtuu laitteelle esiladatusta indeksistä.

**Meilisearch laitteella:** indeksi on Meilisearch-dump (CDN), mutta haku vaatii Meilisearch-prosessin laitteessa (bundlattu binääri, subprocess). Meilisearch ei onnistuneena upoteta kirjastotasolla mobiiliappiin — prosessi käynnistetään ja kyselyt tehdään HTTP:llä paikalliseen instanssiin.

---

## Katselimen nimimaailma

Katselin rakentuu yhtenäisen meriteeman ympärille. Käyttöliittymä käyttää samoja
käsitteitä kaikkialla, jolloin kokonaisuus tuntuu omalta tuotteelta eikä
kokoelmalta irrallisia ominaisuuksia. Kotisatama jää tässä sanastossa
turvallisen etusivun ja suljetun käyttötilan nimeksi.

Käsite| Merkitys
🏠 Kotisatama| Sovelluksen turvallinen etusivu
⚓ Satama| Luotettujen (whitelist) verkkosivujen alue
🌐 Avomeri| Avoin internet
🌊 Myrsky| Ei verkkoyhteyttä tai palvelu ei vastaa
🧰 Varustamo| Luotettu sovellusvarasto
📦 Ruuma| Asennetut sovellukset ja paikallinen sisältö
🛟 Majakka| Ohjeet, opastus ja tuki
📜 Lokikirja| Tapahtumat, ilmoitukset ja diagnostiikka

Filosofia

Katselin ei pyri jäljittelemään perinteistä käyttöjärjestelmää.

Sen sijaan koko käyttöliittymä rakentuu yhden selkeän metaforan ympärille:

Käyttäjä on omassa Kotisatamassaan.

Sieltä voi:

- käyttää turvallista Satamaa
- varustaa ympäristöään Varustamossa
- tarkastella Lokikirjaa
- hakea apua Majakasta

Yhtenäinen sanasto tekee tuotteesta helposti tunnistettavan ja tukee Katselimen
tavoitetta tarjota turvallinen, helposti ymmärrettävä ja luotettava
käyttökokemus kaikenikäisille käyttäjille.

Tuote- ja kehitysfilosofia (Servo, whitelist, Kela, Telakka): [docs/FILOSOFIA.md](docs/FILOSOFIA.md).

Servo-moottorin opiskelu (suomeksi): [oppiminen/README.md](oppiminen/README.md).

Nykytilakirjaus whitelistin onnistuneesta tilasta ja Avomeren avoimesta arkkitehtuuripäätöksestä: [docs/NYKYTILA-2026-06-24.md](docs/NYKYTILA-2026-06-24.md).

Hakutulossivu (tuotespesifikaatio): [docs/Hakutulokset.md](docs/Hakutulokset.md). Toteutusroadmap: [docs/HAKUTULOKSET-ROADMAP.md](docs/HAKUTULOKSET-ROADMAP.md).

Whitelist-skeema 2.1 (valkoiset + keltaiset sivut, kategoriat): [config/whitelist.schema.json](config/whitelist.schema.json).

---

## Kehitysympäristö

Seuraa ensin Servon omaa [setup-ohjetta](https://book.servo.org/hacking/setting-up-your-environment.html). Kotisatama-spesifiset ohjeet alla.

### Vaatimukset

- Rust (stable, versio Servon `rust-toolchain.toml` mukaan)
- Python 3.10+
- Android SDK + NDK (mobiili-APK, compile SDK 37)
- Node.js 20+ (crawler)
- Tauri 2.0 CLI (vain hallintapaneeli, valinnainen)

### Kloonataan

```bash
git clone https://github.com/Mikko-Huuskonen-Pro/Kotisatama.git
cd kotisatama

# Lisätään Servo upstream
git remote add upstream https://github.com/servo/servo
git fetch upstream
```

### Rakennetaan desktop (kehitys)

```bash
./mach build --release
./mach run
```

### Rakennetaan Android (servoshell EGL)

Android käyttää Servon omaa `servoshell`-embedderia (`ports/servoshell/egl/android/`), ei Tauria. Tauri käyttää Androidilla System WebViewia (Chromium) — se ei kantaa Servo-moottoria.

Aseta Android-ympäristö (upstream NDK-versio):

```bash
export ANDROID_SDK_ROOT=~/android-sdk
export ANDROID_NDK_ROOT=$ANDROID_SDK_ROOT/ndk/28.2.13676358
```

```bash
# Esimerkki: arm64-APK
./mach build --target aarch64-linux-android --profile checked-release
# APK: target/aarch64-linux-android/checked-release/servoapp.apk
```

### Hallintapaneeli (Tauri, valinnainen)

Erillinen app vanhemman whitelist-hallintaan — ei osa selainmoottoria:

```bash
cd tauri
npm install
npm run tauri build
```

### Whitelist paikallisesti

Whitelist-JSON haetaan oletuksena CDN:stä. Paikallista kehitystä varten:

```bash
cp config/whitelist.example.json config/whitelist.json
# Muokkaa whitelist.json haluamaksesi
export KOTISATAMA_WHITELIST_PATH=config/whitelist.json
```

---

## Hakuindeksi

Indeksi ladataan CDN:stä asennuksen yhteydessä ja päivittyy OTA-päivitysten mukana. Laitteella Meilisearch-prosessi importaa dumpin käynnistyksessä ja palvelee hakuja paikallisesti (offline, kun indeksi on ladattu).

### Crawlerin ajaminen paikallisesti

```bash
# Terminaali 1: Meilisearch
meilisearch --http-addr 127.0.0.1:7700 --env development --dump-dir ./dumps

# Terminaali 2: Crawler
cd crawler
npm install
npm run crawl -- --whitelist ../config/whitelist.json --output ../output/cdn --dump-dir ../dumps
```

Crawler käyttää Playwrightia — indeksoi myös JS-renderöidyt SPA-sivustot.

### OTA-päivitys CDN:stä

Aseta CDN-URL käynnistyksessä:

```bash
export KOTISATAMA_CDN_BASE=https://cdn.example.com
./mach run
```

Selain lataa `/free/whitelist.json` ja `/free/index.dump` käynnistyksessä.

---

## Upstream-synkronointi

Servo kehittyy aktiivisesti. Synkronointi upstream:ista:

```bash
git fetch upstream
git checkout main
git merge upstream/main
```

Kotisatama-spesifiset hakemistot (`components/kotisatama/`, `tauri/`, `crawler/`) eivät tule upstream-merge-konflikteja — ne ovat vain tässä forkissa. Konfliktit syntyvät **KOTISATAMA-PATCH**-kohdissa upstream-tiedostoissa, tyypillisesti `ports/servoshell/`. Katso suljetun repon `Docs/Puhdistukset.md` aiemmista siivouskierroksista.

---

## Lisenssi

Servo-koodi: [MPL 2.0](https://www.mozilla.org/en-US/MPL/2.0/)

Kotisatama-spesifiset muutokset Servo-koodiin julkaistaan MPL 2.0:n mukaisesti. Oma bisneslogiikka (whitelist-hallinta, Pro-integraatio, Pulloposti) pysyy suljetussa repossa.

---
