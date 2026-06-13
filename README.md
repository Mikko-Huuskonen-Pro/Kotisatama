# Kotisatama

Kotisatama on Servo-pohjainen selain whitelist-pohjaisella hakumallilla. Selain ja esiladattu hakuindeksi muodostavat suljetun ympäristön — käyttäjä löytää kaiken tarvitsemansa ilman että eksyy avomerelle.

> Tämä repo on fork [servo/servo](https://github.com/servo/servo). Kotisatama-spesifiset muutokset on eriytetty omiin moduuleihinsa. Upstream-muutokset julkaistaan MPL 2.0:n mukaisesti. Koska servo ei ole valmis, edetään sivu kerrallaan kotisatamassa. Kun käyttäjä pysyy satamassa, ne sivut pitää toimia ja latautua oikein. Jos lähtee avomerelle, eli kiertää whitelistauksen, sivujen toimivuus on Servon kehityksen varassa. Tietenkin kun ratkotaan whitelistattujem sivujen ongelmia, samalla mmyöskin kotisatama paranee verrattuna Servoon. Kuitenkin siihen pisteeseen on matkaan, että päästäisiin antamaan Servolle takaisin contribuutiota. \\ MH 13.6.2026

---

## Arkkitehtuuri

```
[Kotisatama — Servo-fork]
    ├── components/kotisatama/whitelist   ← whitelist-logiikka
    ├── components/kotisatama/search      ← haku-API (Meilisearch-client)
    └── ports/servoshell                  ← embedder-hook (navigointi, UI)

[Android — servoshell EGL]
    └── support/android/apk + JNI-host    ← ei Tauri; Servon oma Android-polku

[Tauri 2.0 — erillinen hallintapaneeli]
    └── Vanhempi hallinnoi whitelistia (web/desktop), ei selainmoottori

[CDN — staattinen]
    ├── /free/whitelist.json
    └── /pro/whitelist.json  (API-avain vaaditaan)

[Crawler — CI-prosessi]
    └── Playwright → Meilisearch-dump → CDN
```

Ei omaa palvelinta. Ei VPN-infraa. Haku tapahtuu laitteelle esiladatusta indeksistä.

**Meilisearch laitteella:** indeksi on Meilisearch-dump (CDN), mutta haku vaatii Meilisearch-prosessin laitteessa (bundlattu binääri, subprocess). Meilisearch ei onnistuneena upoteta kirjastotasolla mobiiliappiin — prosessi käynnistetään ja kyselyt tehdään HTTP:llä paikalliseen instanssiin.

---

## Tuoteversiot

| Versio | Whitelist | Indeksi |
|---|---|---|
| Kotisatama | Geneerinen, perhe hallinnoi | Yleinen |
| Kotisatama Hopeakettu | Kuratoitu: Kela, terveys, uutiset, virastot | Suomenkielinen, selkeä rakenne |
| Kotisatama Lapsi | Kuratoitu: oppiminen, pelit, ikäluokitettu sisältö | Kuvat, videot, ikäluokitus |

---

## Kehitysympäristö

Seuraa ensin Servon omaa [setup-ohjetta](https://book.servo.org/hacking/setting-up-your-environment.html). Kotisatama-spesifiset ohjeet alla.

### Vaatimukset

- Rust (stable, versio Servon `rust-toolchain.toml` mukaan)
- Python 3.10+
- Android SDK + NDK (mobiili-APK)
- Node.js 20+ (crawler)
- Tauri 2.0 CLI (vain hallintapaneeli, valinnainen)

### Kloonataan

```bash
git clone https://github.com/<sinun-fork>/kotisatama.git
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

Kotisatama-spesifiset hakemistot (`components/kotisatama/`, `tauri/`, `crawler/`) ei tule upstream-merge-konflikteja — ne ovat vain tässä forkissa. Konfliktit syntyvät **KOTISATAMA-PATCH**-kohdissa upstream-tiedostoissa, tyypillisesti `ports/servoshell/` tai minimaalisissa feature-flag-hookeissa.

---

## Lisenssi

Servo-koodi: [MPL 2.0](https://www.mozilla.org/en-US/MPL/2.0/)

Kotisatama-spesifiset muutokset Servo-koodiin julkaistaan MPL 2.0:n mukaisesti. Oma bisneslogiikka (whitelist-hallinta, Pro-integraatio) pysyy suljettuna.

---


