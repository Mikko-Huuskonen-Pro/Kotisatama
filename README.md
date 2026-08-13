# Kotisatama — selainmoottori

> Servo-forkki. Tuotteet (Katselin, Avomeri) ja sivuosat elävät sisarusrepoissa.

**Tämä repo on moottori**, ei tuotesovellus. Kartta kaikesta:
**[docs/EKOSYSTEEMI.md](docs/EKOSYSTEEMI.md)**.

| | |
|---|---|
| Moottori (tämä) | Servo + `components/kotisatama/` + servoshell-hookit |
| Katselin | Suljettu tuoteselain → `../Katselin` |
| Avomeri | Avoin tuoteselain → `../Avomeri` |
| Adblock | `../adblock-Katselin` (integroitu tähän) |
| Haku / consent / data | ks. ekosysteemidokumentti |

---

## What is this engine?

Kotisatama is a [Servo](https://github.com/servo/servo) fork with a thin product
layer: curated whitelist navigation, local Meilisearch client, Brave-based
adblock adapter, and harbour-themed internal pages (`servo:`).

Product shells live elsewhere:

- **Katselin** — closed-source Android browser (whitelist, search, profiles)
- **Avomeri** — open-source Android browser (open web + adblock)

Finnish is the primary project language. Fixes that belong in Servo are meant
to go upstream when ready.

---

## Project status

- Adblock: integrated via `kotisatama-content-blocking` → `adblock-Katselin`
- Windows 11: working; test users acquired
- Android: working (Katselin + Avomeri shells); engine JNI in this repo

## Mikä tämä on (归港)?

**Kotisatama** on Servo-pohjainen moottori. Tuotenimi **Katselin** tarkoittaa
suljettua selainta, joka käyttää tätä moottoria. Avoin sisartuote on **Avomeri**.

Tekniset polut, crate-nimet (`kotisatama-*`) ja `KOTISATAMA_*`-ympäristömuuttujat
säilyvät — uudelleennimeämistä ei tehdä yhdellä iskulla.

> Tämä repo on fork [servo/servo](https://github.com/servo/servo). Kotisatama-spesifiset muutokset on eriytetty omiin moduuleihinsa. Upstream-muutokset julkaistaan MPL 2.0:n mukaisesti. Koska Servo ei ole valmis, edetään sivu kerrallaan satamassa: whitelistattujen sivujen pitää toimia. Avomerellä (avoin verkko) toimivuus riippuu Servon kehityksestä. \\ MH 13.6.2026

---

## Arkkitehtuuri (moottori)

```
[Kotisatama — Servo-fork]
    ├── components/kotisatama/whitelist          ← whitelist-logiikka
    ├── components/kotisatama/search             ← Meilisearch-client
    ├── components/kotisatama/content-blocking   ← adblock-Katselin adapter
    └── ports/servoshell                         ← embedder-hook (navigointi, UI, JNI)

[Tuoteshellit — sisarusrepost]
    ├── ../Katselin/android/apk                  ← suljettu Android-APK
    └── ../Avomeri/android/apk                   ← avoin Android-APK

[Sisarusriippuvuudet]
    ├── ../adblock-Katselin                      ← path-dep content-blockingista
    ├── ../Katselin-haku                         ← Meilisearch Android (Katselin)
    ├── ../Katselin-Consent-O-Matic              ← eväste-JS (Katselin)
    └── ../Kotisataman-suljetut-osat             ← valkoiset sivut, daemonit

[CDN / crawler]
    └── Playwright → Meilisearch-dump → CDN
```

Ei omaa palvelinta. Ei VPN-infraa. Haku tapahtuu laitteelle esiladatusta indeksistä.

**Meilisearch laitteella:** indeksi on Meilisearch-dump (CDN), mutta haku vaatii Meilisearch-prosessin laitteessa (bundlattu binääri, subprocess). Meilisearch ei upoteta kirjastotasolla mobiiliappiin — prosessi käynnistetään ja kyselyt tehdään HTTP:llä paikalliseen instanssiin.

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
🧰 Varustamo| Luotettu freenet sovellukset
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

**Ekosysteemi (mitä on missäkin):** [docs/EKOSYSTEEMI.md](docs/EKOSYSTEEMI.md).

Tuote- ja kehitysfilosofia (Servo, whitelist, Kela, Telakka): [docs/FILOSOFIA.md](docs/FILOSOFIA.md).

Servo-moottorin opiskelu (suomeksi): [oppiminen/README.md](oppiminen/README.md).

Whitelist-skeema 2.1 (valkoiset + keltaiset sivut, kategoriat): [config/whitelist.schema.json](config/whitelist.schema.json).

Vanhat / toteutuneet suunnitelmat (ei nykyrakennetta): sisarusrepo
`../Kotisataman-suljetut-osat/Docs/legacy/` (ks. sen README).

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

### Rakennetaan Android (servoshell EGL + tuoteshell)

Android käyttää Servon omaa `servoshell`-embedderia (`ports/servoshell/egl/android/`), ei Tauria. **APK-Gradle elää tuoteshelleissä**, ei tässä forkissa:

- Katselin: `../Katselin` → `./scripts/build-android.sh`
- Avomeri: `../Avomeri` → ks. sen `docs/BUILD.md`

Tässä repossa rakennetaan `libservoshell.so` (+ JNI). Paikallinen junction APK:han: `./scripts/link-android-apk.ps1` / `.sh`.

```bash
export ANDROID_SDK_ROOT=~/android-sdk
export ANDROID_NDK_ROOT=$ANDROID_SDK_ROOT/ndk/28.2.13676358
./mach build --target aarch64-linux-android --profile checked-release
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

Kotisatama-spesifiset hakemistot (`components/kotisatama/`, `tauri/`, `crawler/`) eivät tule upstream-merge-konflikteja — ne ovat vain tässä forkissa. Konfliktit syntyvät **KOTISATAMA-PATCH**-kohdissa upstream-tiedostoissa, tyypillisesti `ports/servoshell/`. Katso suljetun repon `Docs/legacy/Puhdistukset.md` aiemmista siivouskierroksista.

---

## Lisenssi

Servo-koodi: [MPL 2.0](https://www.mozilla.org/en-US/MPL/2.0/)

Kotisatama-spesifiset muutokset Servo-koodiin julkaistaan MPL 2.0:n mukaisesti. Oma bisneslogiikka (whitelist-hallinta, Pro-integraatio, Pulloposti) pysyy suljetussa repossa.

---
