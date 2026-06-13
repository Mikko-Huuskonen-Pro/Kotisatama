# Puhdistukset

Tahan tiedostoon kirjataan Kotisataman repo- ja koodipohjan puhdistuskierrokset.
Tarkoitus on erottaa Kotisataman toimintaan kuuluvat osat vanhoista Servo-projektin
ja upstream-infran jaanneista.

## 2026-06-13: Servo-projektijaanteiden ensimmainen siivous

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
