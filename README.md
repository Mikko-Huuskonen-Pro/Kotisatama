# Kotisatama

Kotisatama on Servo-pohjainen selain whitelist-pohjaisella hakumallilla. Selain ja esiladattu hakuindeksi muodostavat suljetun ympäristön — käyttäjä löytää kaiken tarvitsemansa ilman että eksyy avomerelle.

> Tämä repo on fork [servo/servo](https://github.com/servo/servo). Kotisatama-spesifiset muutokset on eriytetty omiin moduuleihinsa. Upstream-muutokset julkaistaan MPL 2.0:n mukaisesti.

---

## Arkkitehtuuri

```
[Kotisatama — Servo-fork]
    ├── Whitelist-moduuli (android-components)
    └── Meilisearch-integraatio (lokaali indeksi)

[Tauri 2.0]
    └── Wrappaa Servo-UI:n Android-applikaatioksi

[CDN — staattinen]
    ├── /free/whitelist.json
    └── /pro/whitelist.json  (API-avain vaaditaan)

[Crawler — CI-prosessi]
    └── Playwright → Meilisearch-dump → CDN
```

Ei omaa palvelinta. Ei VPN-infraa. Haku tapahtuu laitteelle esiladatusta Meilisearch-indeksistä.

---

## Tuoteversiot

| Versio | Whitelist | Indeksi |
|---|---|---|
| Kotisatama | Geneerinen, perhe hallinnoi | Yleinen |
| Kotisatama Hopeakettu | Kuratoidtu: Kela, terveys, uutiset, virastot | Suomenkielinen, selkeä rakenne |
| Kotisatama Lapsi | Kuratoidtu: oppiminen, pelit, ikäluokitettu sisältö | Kuvat, videot, ikäluokitus |

---

## Kehitysympäristö

Seuraa ensin Servon omaa [setup-ohjetta](https://book.servo.org/hacking/setting-up-your-environment.html). Kotisatama-spesifiset ohjeet alla.

### Vaatimukset

- Rust (stable, versio Servon `rust-toolchain.toml` mukaan)
- Python 3.10+
- Android SDK (jos rakennat mobiilia)
- Tauri 2.0 CLI
- Node.js 20+ (crawler ja Tauri-build)

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

### Rakennetaan Android (Tauri)

```bash
cd tauri
npm install
npm run tauri android build
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

Meilisearch pyörii laitteella. Indeksi ladataan CDN:stä asennuksen yhteydessä ja päivittyy OTA-päivitysten mukana.

### Crawlerin ajaminen paikallisesti

```bash
cd crawler
npm install
node crawl.js --whitelist ../config/whitelist.json --output ./index-dump
```

Crawler käyttää Playwrightia — indeksoi myös JS-renderöidyt SPA-sivustot.

---

## Upstream-synkronointi

Servo kehittyy aktiivisesti. Synkronointi upstream:ista:

```bash
git fetch upstream
git checkout main
git merge upstream/main
```

Kotisatama-muutokset on tarkoituksella eriytetty omiin moduuleihinsa konfliktien minimoimiseksi. Jos tulee konflikti, se on lähes aina `components/kotisatama/`-hakemistossa.

---

## Lisenssi

Servo-koodi: [MPL 2.0](https://www.mozilla.org/en-US/MPL/2.0/)

Kotisatama-spesifiset muutokset Servo-koodiin julkaistaan MPL 2.0:n mukaisesti. Oma bisneslogiikka (whitelist-hallinta, Pro-integraatio) pysyy suljettuna.

---


