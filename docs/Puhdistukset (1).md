# Puhdistukset

Tahan tiedostoon kirjataan Kotisataman repo- ja koodipohjan puhdistuskierrokset.
Tarkoitus on erottaa Kotisataman toimintaan kuuluvat osat vanhoista Servo-projektin
ja upstream-infran jaanneista.

## 2026-08-03: V3 — Android-APK pois forkista (Katselin-päärepo)

Branch: paikallinen työ (ei vielä commitoitu; osana Katselin-repojakoa)  
Liittyy: `../Katselin/docs/REPO-JAKO-SUUNNITELMA.md` (V3), `support/android/README.md`

### Tavoite

Hillitä Servo-forkin (Kotisatama) muutospintaa: Android-sovelluksen Gradle-projekti
ei elä enää tässä repossa. Ainoa lähde on Katselin-päärepo
(`https://github.com/Mikko-Huuskonen-Pro/Katselin`, polku `android/apk/`).
Upstream-merge ei koske APK/UI-tiedostoja; mach löytää polun paikallisella
junctionilla/symlinkillä.

### Poistetut / siirretty pois tämän repon lähdepuusta

- `support/android/apk/` **kokonaan** (servoapp, servoview, Gradle, buildSrc, jne.)
  - Levyllä: Windows-**junction** → `../Katselin/android/apk` (ei gittiin).
  - Git-indeksistä: `git rm -r --cached support/android/apk` (staged deletions,
    commit myöhemmin yhdessä muun reporakenteen kanssa).
  - `.gitignore`: `/support/android/apk/` — kloonin jälkeen junction luodaan
    skriptillä, ei seurata gittiin.
- Vanhasta apk-puusta poistettu myös paikallinen backup `apk.bak-v3` (~3.7 GB)
  junctionin varmistuksen jälkeen.

### Muutetut asiat (tässä forkissa)

- `support/android/README.md`
  - Korvattu osoittimella Katselin-repoon + junction-ohjeisiin.
- `support/android/fetch-meilisearch.sh` / `.ps1`
  - Deprecated-wrapperit; ohjaavat `../Katselin/android/fetch-meilisearch.*`.
- `AGENT.md` (Android-kohta)
  - APK-paketointi = Katselin; paikallinen linkitys `scripts/link-android-apk.*`.
- `.gitignore`
  - Lisätty `/support/android/apk/`.

### Lisätyt asiat

- `scripts/link-android-apk.ps1` — `mklink /J` Windowsilla
- `scripts/link-android-apk.sh` — `ln -s` WSL/Linuxissa

### Varmistus

- WSL-build Katselin-orkestroijalla (`x86_64-linux-android`): exit 0;
  logi `Engine apk path OK`, Gradle-polut `Katselin/android/apk/...`,
  ei pre-V3 -varoitusta.
- Emulaattori: uninstall + install + `MainActivity` OK
  (`target/x86_64-linux-android/checked-release/servoapp.apk`).

### Jää tähän forkkiin (tarkoituksella)

- `ports/servoshell/egl/android/` — JNI + EGL
- `components/kotisatama/*` — whitelist, search, content-blocking, …
- `resources/resource_protocol/`, `config/` — jaettu desktopin kanssa
- `scripts/build-android.sh` — legacy; suositus: `../Katselin/scripts/build-android.sh`

### Commit-huomio

Kun kokonaisuus commitoitaan: staged `D`-rivit `support/android/apk/**` + yllä
olevat dokumentti-/skriptimuutokset. Junction/symlink **ei** kuulu committiin.

---

## 2026-06-13: Servo-projektijaanteiden ensimmainen siivous

github/workflows poistettu kaikki muut paitsi kotisatamaan liittyvä crawler. 

Branch: `cursor/remove-servo-leftovers-5fdf`  
Commit: `3772c92ba6b Remove obsolete Servo project leftovers`

### Tavoite

Poistaa tai rebrandata sellaiset Servo-projektiin liittyvat asiat, jotka eivat
vaikuta Kotisataman selain-, whitelist-, crawler- tai build-toimintaan. Esimerkki
tasta oli GitHubin sponsorointitiedosto, joka pyysi lahjoituksia upstream-Servolle.

### Poistetut asiat

- `.github/FUNDING.yml`
  - Poisti GitHub Sponsors / Open Collective -linkit upstream-Servolle.
- `etc/doc.servo.org/`
  - Poisti vanhan `doc.servo.org`-julkaisuhakemiston sisallon.
- Servo Bookiin ohjaavat dokumenttistubit:
  - `docs/HACKING_QUICKSTART.md`
  - `docs/STYLE_GUIDE.md`
  - `docs/glossary.md`
  - `docs/debugging.md`
  - `docs/ORGANIZATION.md`
  - `docs/COMMAND_LINE_ARGS.md`
  - `docs/components/style.md`
  - `docs/components/webxr.md`
- Servo Media -upstream-dokumentit:
  - `docs/media/overview.md`
  - `docs/media/webaudio.md`
  - `docs/media/avplayback.md`
- Vanhoja upstream-yllapitotiedostoja:
  - `etc/show-stale-intermittent-issues.sh`
  - `etc/ci/performance/README.md`
  - `tests/power/README.md`
  - `tests/power/PowerMeasure.py`
- Servo.orgiin kovakoodatut scenario-testit:
  - `etc/ci/scenario/servo_test_open_page_servo.py`
  - `etc/ci/scenario/servo_test_open_page_servo_plot.py`
- Kayttamattomat Servo-brandiassetit:
  - `resources/servo.svg`
  - `resources/resource_protocol/servo-color-positive-no-container.svg`
  - `resources/resource_protocol/servo-color-negative-no-container.svg`
  - `resources/org.servo.Servo.desktop`

### Muutetut asiat

- `.github/ISSUE_TEMPLATE/roadmap.md`
  - Roadmap-template puhuu nyt Kotisataman roadmapista, ei Servon roadmapista.
- `.github/release.yml`
  - Poistettiin `servo-wpt-sync` automaattisista release note -poissulkuista.
- `.github/actions/parse_msrv/action.yml`
  - Vaihdettiin GitHub UI:ssa nakyvat `libservo`/`servo`-tekstit
    neutraaliksi browser engine -sanastoksi. Varsinainen crate-haku
    `select(.name == "servo")` jatettiin, koska se on toiminnallinen.
- `.github/CODEOWNERS`
  - Poistettiin viittaus puuttuvaan `.github/workflows/ohos.yml`-workflow'hun.
- `python/servo/post_build_commands.py`
  - Dokumentaatiobuild ohittaa nyt puuttuvan `etc/doc.servo.org`-hakemiston.
- `resources/resource_protocol/newtab.html`
  - Newtab rebrandattiin Kotisatamaksi.
  - Servo-logo ja Servo.org-linkit poistettiin.
- `resources/resource_protocol/newtab.css`
  - Poistettujen logoassetien tyylit korvattiin Kotisatama-otsikon tyyleilla.
- `ports/servoshell/prefs.rs`
  - Oletuskotisivu ja CLI:n URL-fallback vaihdettiin Servo.orgista `about:blank`iin.
- `resources/package-prefs.json`
  - Kommentti muutettiin pois Servo nightly build -sanastosta.
- `support/hitrace-bencher/runs.json`
  - Benchmark-esimerkin URL vaihdettiin `https://www.servo.org` -> `https://example.com`.
- OpenHarmony-labelit:
  - `support/openharmony/AppScope/resources/base/element/string.json`
  - `support/openharmony/entry/src/main/resources/en_US/element/string.json`
  - `support/openharmony/entry/src/main/resources/zh_CN/element/string.json`
  - Kayttajalle nakyvat labelit vaihdettiin Kotisatamaksi.
- `support/openharmony/entry/src/main/ets/entryability/EntryAbility.ets`
  - Oletus-URL vaihdettiin `https://servo.org` -> `about:blank`.
  - Lokitagit ja virhetekstit rebrandattiin Kotisatamaksi.
- `etc/ci/scenario/update_mitmproxy_dump.py`
  - Poistettuihin Servo.org-scenarioihin liittyva import ja ajokutsu poistettiin.

### Tarkoituksella paikalleen jatetyt Servo-viittaukset

Naiden poistaminen ei kuulu tahan siivouskierrokseen, koska ne ovat
toiminnallisia, upstream-testidataa tai juridisesti herkkaa attribuutiota:

- `servo-*` crate-nimet ja `servoshell`-nimet.
- WPT-testidata ja vendoroitu `tests/wpt/**`.
- Lisenssi- ja kolmannen osapuolen attribuutiot.
- Rustdoc-linkit `doc.servo.org`iin koodikommenteissa.
- WPT-sync-tyokalujen testidata ja botinimiin liittyvat yksikkotestit.
- Servo-urlit, joita kaytetaan vain parseri-/webview-/performance-testien
  esimerkkidatana.

### Validointi

- `git diff --check HEAD~1..HEAD`: OK
- `python3 -m py_compile python/servo/post_build_commands.py etc/ci/scenario/update_mitmproxy_dump.py`: OK
- `cargo test -p servoshell prefs --lib`: ei paassyt lahdekooditesteihin asti,
  koska ympariston C++-toolchain ei loytanyt `glsl-optimizer`-buildissa headeria
  `<new>`.

### Seuraavien kierrosten muistiinpanot

Seuraavilla kierroksilla kannattaa kasitella varovasti:

- `README.md`, `CONTRIBUTING.md`, `SECURITY.md` ja muut projektidokit:
  Servo-maininnat voivat olla fork-attribuutiota tai Kotisataman teknista
  todellisuutta.
- `Cargo.toml`, `pyproject.toml` ja lockfilet:
  nimimuutokset voivat vaikuttaa buildiin tai tooling-polkuun.
- `resources/resource_protocol/license.html` ja `etc/about.hbs`:
  sisaltavat lisenssi- ja attribuutiotekstia, joita ei pidä muuttaa ilman
  erillista paatosta.
- OpenHarmony- ja Android-paketointinimet:
  osa on kayttajalle nakyvaa brandia, osa taas buildin ja native-kirjastojen
  nimisidonnaista infraa.

## Tulevan puhdistuskierroksen pohja

### YYYY-MM-DD: Otsikko

Branch: `...`  
Commit: `...`

#### Tavoite

-

#### Poistetut asiat

-

#### Muutetut asiat

-

#### Tarkoituksella paikalleen jatetyt asiat

-

#### Validointi

-

## 2026-06-14: Upstream Servo merge + Pulloposti-integraatio

Branch: `main` (ei commitoitu — käyttäjä commitoi kerralla myöhemmin)

### Tavoite

- Synkronoida `upstream/main` (mozjs 140.12, Android Kotlin MainActivity, compile SDK 37, jne.)
- Säilyttää Kotisatama-PATCHit (whitelist, haku, raportti, teemat)
- Liittää Pulloposti subprocess-mallilla (`Ideoita.md`)

### Poistetut asiat

- `.github/workflows/release.yml` — pysyy pois (edellinen siivouskierros)
- `support/android/.../MainActivity.java` — korvattu upstream Kotlin + Kotisatama-PATCH

### Muutetut asiat

- Upstream merge ~20 committia (`upstream/main` → paikallinen `main`)
- `MainActivity.kt`: KotisatamaAssets, KotisatamaUi, raporttinappi (Java → Kotlin siirto)
- `components/kotisatama/pulloposti/` — subprocess-client (kuten Meilisearch)
- `servo:pulloposti` gateway + myrsky/offline-ehdotus avomeri-sivulla
- `scripts/sync-pulloposti-daemon.ps1` — daemon suljetusta reposta

### Suljetussa repossa

- `Pulloposti/daemon/` — `pulloposti-daemon` (/health, portti 7701)
- `docs/KOTISATAMA-INTEGRAATIO.md` — päivitetty subprocess-malliin

### Tarkoituksella paikalleen jätetyt asiat

- WPT, servo-crate-nimet, MPL-attribuutiot (kuten edellinen kierros)

### Validointi

- Merge-konfliktit ratkaistu (README, Cargo.lock, MainActivity, release.yml)
- `cargo test -p kotisatama-pulloposti -p kotisatama-whitelist -p kotisatama-search -p kotisatama-report` — 8/8 OK (2026-06-18)
- `scripts/sync-pulloposti-daemon.ps1` — daemon kopioitu `bin/pulloposti-daemon.exe`
