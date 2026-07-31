# Ensimmäinen Android-build emulaattorissa

**Päivä:** 2026-07-31  
**Konteksti:** Kotisatama APK aarch64, Servo servoshell EGL  
**Komponentti:** `ports/servoshell/egl/android/`, `support/android/`

## Mitä yritin

- Saada Kotisatama-APK käyntiin Android-emulaattorissa Windows-koneella.
- Suora `.\mach build --target aarch64-linux-android` Windowsissa epäonnistuu:
  `Android cross builds are only supported on Linux and macOS.`

## Onnistunut polku (WSL2)

Windowsissa `scripts/build-android.ps1` varoittaa ja ohjaa WSL:ään. Rakenna Linux-puolella:

```bash
# WSL2, repon juuressa (Linux-polku, ei /mnt/... jos mahdollista)
cd ~/Kotisatama/Kotisatama   # tai vastaava klooni WSL:ssä

# Ympäristö (Servo vaatii NDK r28)
export ANDROID_SDK_ROOT=~/Android/Sdk
export ANDROID_NDK_ROOT=$ANDROID_SDK_ROOT/ndk/28.2.13676358

# Ensimmäinen kerta: bootstrap + NDK tarvittaessa
./scripts/build-android.sh --install-ndk

# Seuraavat kerrat (nopeampi)
./scripts/build-android.sh --skip-bootstrap

# APK + asennus emulaattoriin
./scripts/build-android.sh --skip-bootstrap --install --emulator
```

Manuaalisesti sama asia:

```bash
./mach bootstrap --yes
./support/android/fetch-meilisearch.sh
./mach build --target aarch64-linux-android --profile checked-release
# HUOM: ./mach package --android --target <t> epäonnistuu ("Please specify either
# --target or --android") — mach build ajaa jo Gradle-assemblen ja tuottaa APK:n.
# Asenna suoraan adb:llä:
adb install -r support/android/apk/servoapp/build/outputs/apk/arm64Release/servoapp-arm64Release.apk
```

APK:

```
support/android/apk/servoapp/build/outputs/apk/arm64Release/servoapp-arm64Release.apk   # fyysinen laite
support/android/apk/servoapp/build/outputs/apk/x64Release/servoapp-x64Release.apk       # x86_64-emulaattori
target/<target>/checked-release/servoapp.apk                                            # symlink/kopio
```

adb suoraan (jos APK on jo valmis):

```bash
adb install -r support/android/apk/servoapp/build/outputs/apk/x64Release/servoapp-x64Release.apk
```

### WSL-huomiot

- `mach build` tarvitsee **Linux-NDK**:n (`prebuilt/linux-x86_64`). Windowsin Android Studion NDK ei kelpaa WSL-käännökseen.
- Jos SDK/NDK puuttuu: `./scripts/build-android.sh --install-ndk`
- Whitelist: skripti synkkaa suljetuista osista → `index-data/cache/whitelist.json` (tai `config/whitelist.json`)
- Haku: bundlaa Meilisearch `fetch-meilisearch.sh`:llä; valinnainen `index-data/index.dump`

Lisää: [support/android/README.md](../../../support/android/README.md)

## Emulaattoritestaus 2026-07-31 (uusi x64-APK, iltapäivä)

| Testi | Tulos |
|---|---|
| katselin.fi avautuu | OK |
| Takaisin (Android + sovelluksen oma) | OK |
| Osoitepalkista haku `kela` | **Toimii** — 6 tulosta (Kela.fi, Kanta.fi, OmaKanta, Asiointi, Eläke, Toimeentulotuki), ei virheviestiä |
| Yläpalkki: Avaa sovelluksessa | OK — näkyy suoraan yläpalkissa, kolmen pisteen valikko poistettu |
| Lokikirja | OK — vain alapalkissa (📜) |

### Miksi Meilisearch ei lataudu Androidilla (kaksi syytä)

1. **glibc vs bionic:** virallinen Linux-binääri on dynaamisesti linkitetty
   (`interpreter /lib64/ld-linux-x86-64.so.2`), Androidin bionic ei aja sitä.
2. **SELinux:** `avc: denied { execute_no_trans }` — sovellus ei saa suorittaa
   binääriä `files/`-hakemistosta (os error 13, Permission denied).

→ **Ratkaisu:** `kotisatama_search::seed_search` varahaku (muistissa, kuratoitu
`documents.json` + whitelist). Käytössä aina kun Meilisearch-client ei käynnisty.
Emulaattorissa haku toimii nyt ilman virheilmoitusta. Tuotantotason Meilisearch
vaatisi NDK-käännöksen ja appin nativen hakemiston (`nativeLibraryDir`).

### Tarkennus: "kela-haku toimi" -väite oli virheellinen

- Käyttäjän kuvakaappaus (x64-APK 30.7.) osoitti: sivu näytti "Paikallinen haku ei
  käytettävissä" -virheen. Tulokset eivät tulleet Meilisearchista.
- Lisäksi käyttäjän emulaattorissa pyöri **vanha 30.7. x64-APK**: uusi UI
  (Avaa sovelluksessa yläpalkissa, Lokikirja vain alapalkissa) ja Meilisearch-bundlaus
  eivät olleet siinä mukana.
- Oivallus: Meilisearchin Linux-binääri on glibc-dynaaminen → **ei käynnisty Androidin
  bionicissa lainkaan** (`file`: interpreter `/lib64/ld-linux-x86-64.so.2`).
- **Ratkaisu:** seed-varahaku (`kotisatama_search::seed_search`) — muistissa oleva
  substring-haku kuratoidusta `documents.json` + whitelist-aineistosta, käytössä aina
  kun Meilisearch-client ei käynnisty. Androidilla tämä on käytännössä ainoa polku
  ilman NDK-käännettyä Meilisearchiä.
- `KotisatamaAssets.extractAssetIfPresent` ei korvannut vanhaa binääriä (dest olemassa) →
  lisätty kokotarkistus, jotta väärän arkkitehtuurin binääri korvautuu.

### arm64-APK kaatuu emulaattorilla

- x86_64-AVD kääntää arm64-koodia Berberis-tulkilla.
- arm64-APK kaatuu heti: `java.lang.RuntimeException: Rust error: Java exception was thrown` osoitteessa `JNIServo.init`.
- Sama crash toistui 30.7. ja 31.7. arm64-APK:lla.
- **x86_64-APK (`servoapp-x64Release.apk`) toimii emulaattorissa** — se on suositeltu debug-tavoite.
- Fyysisellä laitteella (puhelin) käytetään arm64-APK:ta.

Korjaus seuraavaan buildiin: aja `./support/android/fetch-meilisearch.sh` (älä käytä `--skip-meilisearch`) ennen packagea, jotta binääri menee APK-assetteihin.

### Lokikirja / ehdotukset

Lähetys vaatii `KOTISATAMA_GITHUB_TOKEN` (kehitys) tai `KOTISATAMA_REPORT_URL` (tuotanto-worker). Ilman näitä toast: aseta token/URL. Tuotanto-APK:lle ei pidä upottaa GitHub-tokenia — worker-URL.

## Mitä opin

- Android-käännös Kotisatamassa = Servon oma EGL-polku, ei Tauri.
- Windows → WSL2 on käytännön polku; natiivi Windows-mach ei tue Android-crossia.
- `headers`-crate tarvitaan servoshellissä myös Androidilla (`ports/servoshell/Cargo.toml`).
- Predictive Back: `enableOnBackInvokedCallback` + Compose `BackHandler` → sama kuin alapalkin Takaisin.

## Seuraava askel

- [x] x64-APK uusilla muutoksilla (UI + seed-varahaku) — asennettu emulaattoriin 31.7. iltapäivällä, testattu
- [x] arm64-APK fyysiselle laitteelle — `servoapp-arm64Release.apk` (171 Mt, ilman Meilisearch-binaaria),
  kopio jakelua varten: `~/Downloads/Kotisatama-2026-07-31-arm64.apk`
- [ ] Ensimmäinen ajo oikealla puhelimella (arm64 natiivi — ei Berberis-ongelmaa)
- [x] Kokeelliset prefs oletuksena päälle Androidissa
- [x] Yläpalkki: **Avaa sovelluksessa** Lokikirjan tilalle (Lokikirja jää alapalkkiin)
- [ ] Lokikirjan lähetys emulaattorissa: `KOTISATAMA_REPORT_URL` tai GitHub-lomake ilman tokenia
- [ ] (Myöhemmin) NDK-käännetty Meilisearch, jos seed-haku ei riitä tuotantoon
