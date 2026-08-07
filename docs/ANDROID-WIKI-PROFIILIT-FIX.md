# Android-wiki + profiilit + Enter — korjaussuunnitelma

Päivämäärä: 2026-08-07  
Tila: Toteutettu (kierros 1 + kierros 2, emulaattori PASS 2026-08-07)

## Ongelman yhteenveto

Emulaattoritestaus paljasti, että desktop-muutokset eivät olleet päätyneet Android-versioon, ja toinen kierros korjasi jäljelle jääneet bugit.

### Kierros 1 (toteutettu)

1. **Enter-näppäin** — pehmonäppäimistön Enter lisäsi rivinvaihdon → korjattu.
2. **Offline-Wikipedia + Meilisearch** — dump/binääri APK:han.
3. **Profiilivalikko Androidissa** — Settings → Käyttäjäprofiili → `servo:config`, first-run.

### Kierros 2 (toteutettu, emulaattori PASS)

| # | Ongelma | Korjaus |
|---|---------|---------|
| 1–2 | Profiililukko + Normi-paluu | `servo:profile/verify` + unlock-overlay; emoji lähetetään myös Normi-paluuun |
| 3 | Consent-O-Matic avoinna | Natiiviasetukset eivät lukitu (vain `servo:config`) |
| 4 | Wiki-kortit | SHA-256 dump-import + puhdas asennus; dumpissa wiki-indeksit |
| 5 | Lisenssit | Siisti `license.html` + kuratoitu `licenses.json` |
| 6 | Emoji-ikonit | SVG-picker (`config-emoji-icons.js`) |
| 7 | Servo-prefs vuoto | Prefs-taulukko poistettu `config.html`:stä |
| — | YouTube Lapsi-profiililla | Poistettu `lapsi`-tagi + whitelist **hot-reload** profiilinvaihdossa |
| — | Väärät hakunimet (Yle) | `documents`-indeksi tyhjennetään ja ladataan uudelleen käynnistyksessä |

---

## Juurisyyt (kierros 2)

### Normi-paluu ei toiminut

Backend `set_profile()` vaati emoji-parametrin, mutta `config.html` piilotti kentän Normi-valinnalla eikä lähettänyt salasanaa.

### YouTube Lapsi-profiililla

Whitelist ladattiin kerran käynnistyksessä (`OnceLock`). Profiilinvaihto ei päivittänyt `EFFECTIVE`-listaa → vanha (Normi/Hopeakettu) lista jäi voimaan.

### Wiki-kortit

Vanha Meilisearch-DB esti dump-importin (mtime-logiikka). Korjattu SHA-256-markkerilla + uudelleenasennuksella.

### Lisenssit

`license.html` sisälsi edelleen koko Servo cargo-about -dumpin (~1,1 MB) JSON-UI:n alla.

### Yle-otsikko

Meilisearch upsert ei korvannut vanhaa `documents`-dokumenttia; tarvitaan delete + reload.

---

## Muutetut tiedostot — yhteenveto

### Kierros 1

| Repo | Tiedosto | Muutos |
|------|----------|--------|
| Kotisatama | `ports/servoshell/egl/android/mod.rs` | Enter-keycode ensin; first-run → `servo:config` |
| Katselin | `build.gradle.kts`, `build-android.sh`, `sync-android-wiki-data.sh` | Wiki-data + Meili APK:han |
| Katselin | `SettingsActivity.kt`, `MainActivity.kt`, `KotisatamaUi.java` | Profiili + `servo:` URL |

### Kierros 2

| Repo | Tiedosto | Muutos |
|------|----------|--------|
| Kotisatama | `resources/resource_protocol/config.html` | Unlock-overlay, Normi-paluu emoji, SVG-picker, ei prefs |
| Kotisatama | `resources/resource_protocol/config.css` | Overlay + SVG-painikkeet |
| Kotisatama | `resources/resource_protocol/config-emoji-icons.js` | Uusi SVG-emoji-valitsin |
| Kotisatama | `resources/resource_protocol/kotisatama-i18n.js` | Unlock/Normi-vihjeet |
| Kotisatama | `resources/resource_protocol/license.html` | Vain kuratoitu lista |
| Kotisatama | `resources/resource_protocol/licenses.json` | Kotisatama, Meilisearch, Consent-O-Matic |
| Kotisatama | `ports/servoshell/protocols/servo.rs` | `profile/verify` + whitelist reload `profile/set`:ssä |
| Kotisatama | `components/kotisatama/whitelist/src/state.rs` | `reload_for_profile()` |
| Kotisatama | `components/kotisatama/whitelist/src/resolve.rs` | `KOTISATAMA_DATA_DIR/{tag}-whitelist.json` |
| Kotisatama | `components/kotisatama/search/src/lib.rs` | SHA-256 dump-import; seed clear+reload |
| Kotisatama | `scripts/sync-android-wiki-test-data.ps1` | Wiki-indeksi-validointi |
| Suljettu | `whitelist-unified.json` | YouTube: ei `lapsi`-tagia |
| Suljettu | `scripts/export-profile-whitelist.py` | Sanity: youtube + wikipedia pois lapsista |

---

## Build ja asennus

### Emulaattori (x86_64)

```powershell
cd C:\Users\gigli\Kotisatama\Kotisatama
.\scripts\sync-android-wiki-test-data.ps1   # Meilisearch käynnissä

wsl -d Ubuntu-24.04 -- bash /mnt/c/Users/gigli/Kotisatama/Katselin/scripts/wsl-run-android-build.sh

adb uninstall org.servo.servoshell
adb install -r -d -g target\x86_64-linux-android\checked-release\servoapp.apk
```

### Oikea laite (arm64 / aarch64)

```powershell
# Wiki-data sama kuin yllä (Windows)

wsl -d Ubuntu-24.04 -- bash /mnt/c/Users/gigli/Kotisatama/Katselin/scripts/wsl-run-android-build-arm64.sh

# USB-debug päällä laitteessa:
adb devices
adb uninstall org.servo.servoshell
adb install -r -d -g C:\Users\gigli\Kotisatama\Kotisatama\target\aarch64-linux-android\checked-release\servoapp.apk
```

APK-polku (arm64):  
`C:\Users\gigli\Kotisatama\Katselin\servoapp-arm64.apk`  
(kopio buildista; myös `Kotisatama\target\aarch64-linux-android\checked-release\servoapp.apk` WSL:ssä)

**Huom:** Puhdas asennus (`uninstall` + `install`) varmistaa Meilisearch-DB:n ja whitelist-exporttien uudelleenpurkamisen.

---

## Testauslista (PASS emulaattorissa 2026-08-07)

| Testi | Tulos |
|-------|-------|
| Hopeakettu + emoji → `servo:config` lukitus | PASS |
| Normi-paluu samalla emojilla | PASS |
| Lapsi → natiiviasetukset / Consent-O-Matic ilman emojiä | PASS |
| Haku "Helsinki" → wiki-kortti | PASS |
| Lisenssit → vain kuratoidut | PASS |
| SVG-emoji-picker (ei tyhjiä laatikoita) | PASS |
| Ei Servo-prefs-taulukkoa configissa | PASS |
| Lapsi → YouTube estetty | PASS |
| Normi/Hopeakettu → Iltasanomat aukeaa | PASS |
| Haku "yle" → otsikko Yle | PASS |

### Profiilimuistutus

- **Lapsi:** vain `lapsi`-tagatut sivut; ei online-Wikipediaa, ei YouTubea, ei Avomeria; wiki vain offline-snapshot.
- **Hopeakettu:** `hopeakettu`-lista; Avomeri valittavissa.
- **Normi:** koko whitelist; Avomeri aina päällä.

Wiki-testidata APK:ssa on ~5000 artikkelin näyte (ei koko fiwiki). Haku toimii monilla suomenkielisillä aiheilla; täysi indeksi tulee myöhemmin CDN:n kautta.

---

## Regressiot (älä riko)

- Enter-näppäin Androidilla
- Avomeri-estot profiileittain
- Ensimmäinen käynnistys → `servo:config`
- Natiiviasetukset (Consent-O-Matic) ilman emoji-lukkoa
