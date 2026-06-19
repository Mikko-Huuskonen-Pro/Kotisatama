# KIELIROADMAP.md — Kotisatama (suomi + ruotsi)
*Kesäkuu 2026*

Tämä roadmap kattaa lokalisoinnin (i18n) rakentamisen Kotisatamaan, suomi oletuksena ja ruotsi ensimmäisenä lisäkielenä. Tällä hetkellä reposta **ei löydy minkäänlaista i18n-mekanismia** — kaikki teksti on kovakoodattu neljään eri paikkaan, kolmella eri tavalla. Tämä roadmap käy ne läpi pinta kerrallaan.

---

## Nykytila ennen tätä roadmapia

| Pinta | Mekanismi | Kieli nyt |
|---|---|---|
| Android (`strings.xml`) | Android-resurssijärjestelmä (oikea, valmis pohja) | Sekakielinen: osa suomea, osa englantia |
| Sisäiset sivut (`resources/resource_protocol/*.html`) | Ei mitään — teksti suoraan HTML:ssä | Suomi, `lang="fi"` kovakoodattu |
| Desktop-UI (`ports/servoshell/desktop/gui.rs`, egui) | Ei mitään — ~49 merkkijonoa suoraan Rust-koodissa | Suomi |
| Blokkaussivu (`kotisatama-whitelist`-craten `lib.rs`) | Ei mitään — HTML generoidaan `format!`-makrolla | Suomi |

---

## Vaihe 1 — Android: `strings.xml` siivous + `values-sv/`

**Tavoite:** Android-puolen oletuskieli yhtenäisesti suomeksi, ruotsi rinnalle natiivilla Android-mekanismilla.

- [ ] Korvaa `support/android/apk/servoapp/src/main/res/values/strings.xml` siivotulla, täysin suomenkielisellä versiolla
- [ ] Lisää `support/android/apk/servoapp/src/main/res/values-sv/strings.xml` (ruotsinkielinen)
- [ ] Tarkista `idle`-merkkijonon näkyvyys (debug-label vai oikeasti käyttäjälle näkyvä?)
- [ ] Tarkista `options` vs. `settings_title` -kaksinaisuus — sama teksti, kaksi eri avainta
- [ ] Testaa laitteella/emulaattorilla molemmilla kieliasetuksilla
- [ ] Commit + PR

**Tiedostot valmiina odottamassa käyttöönottoa:** `strings.xml` (suomi) ja `strings-sv.xml` (ruotsi, nimetään `values-sv/strings.xml`:ksi kohteessa) — luotu, **ei vielä viety repoon eikä testattu**.

**Valmis kun:** Android näyttää oikean kielen laitteen asetuksen mukaan, ei sekakielisiä näkymiä.

---

## Vaihe 2 — Sisäiset HTML-sivut (`resource_protocol`)

**Tavoite:** Avomeri, Pulloposti, uusi välilehti, asetukset ja lisenssisivu tukevat kieliasetusta.

Koskee: `avomeri.html`, `pulloposti.html`, `newtab.html`, `config.html`, `license.html` (+ vastaavat `.css`).

- [ ] Päätä mekanismi: kevyt JS-pohjainen tekstikartta (`{ fi: {...}, sv: {...} }`) vs. erilliset `.html`-tiedostot per kieli vs. palvelinpuolen (Rust) templating ennen `resource:///`-tarjoilua
- [ ] Päätä mistä kieliasetus luetaan (selaimen oma asetus? käyttöjärjestelmän locale? erillinen Kotisatama-asetus?)
- [ ] Poista kovakoodattu `lang="fi"` — korvaa dynaamisella tai per-kieli-tiedostolla
- [ ] Käännä tekstit ruotsiksi (mm. "Olet siirtymässä avomerelle...", "Haluatko viestiä pullopostilla?", "Myrsky: verkkoyhteyttä ei ole...")
- [ ] Testaa molemmat kielet kaikilla viidellä sivulla

**Valmis kun:** Kaikki viisi sisäistä sivua näyttävät oikean kielen ilman koodin kovakoodattua suomea.

---

## Vaihe 3 — Blokkaussivu (`kotisatama-whitelist`)

**Tavoite:** "Tätä sivua ei löydy kotisatamassa" -sivu (data: URL) tukee kieliasetusta.

- [ ] Erota HTML-templaatti `format!`-makrosta omaksi tekstikartaksi (sama päätös kuin vaihe 2 — yhtenäinen mekanismi koko reposssa olisi siistein)
- [ ] Käännä: otsikko, "Jatka avomerelle", raportointiohje
- [ ] Varmista että kielivalinta kulkee samalla tavalla kuin muu UI (ei oma erillinen logiikka)

**Valmis kun:** Blokkaussivu noudattaa samaa kieliasetusta kuin loppu selain.

---

## Vaihe 4 — Desktop-UI (`gui.rs`, egui)

**Tavoite:** ~49 kovakoodattua merkkijonoa irti Rust-koodista, käännettävissä.

Tämä on työläin vaihe — egui on immediate-mode-UI, merkkijonot ovat suoraan koodissa kutsupaikoissa, ei erillistä resurssitiedostoa.

- [ ] Valitse käännösmekanismi Rust-puolelle. Vaihtoehdot:
  - **Fluent** (`fluent-rs`) — Mozilla/Servo-ekosysteemin oma, luonteva valinta forkille
  - Kevyt oma ratkaisu: `match kieli { Fi => "...", Sv => "..." }` -funktiot per merkkijono
- [ ] Kerää kaikki 49 merkkijonoa yhteen paikkaan (esim. `components/kotisatama/i18n/`)
- [ ] Korvaa suorat literaalit funktiokutsuilla/avaimilla
- [ ] Käännä ruotsiksi
- [ ] Kielivalinta: sama lähde kuin vaiheissa 2–3 (yhtenäinen asetus koko selaimelle)

**Valmis kun:** Desktop-UI vaihtaa kielen yhdestä asetuksesta, ei kovakoodattua suomea jäljellä `gui.rs`:ssä.

---

## Vaihe 5 — Kielivalinta käyttäjälle

**Tavoite:** Käyttäjä voi vaihtaa kielen (ei vain seuraa käyttöjärjestelmän localea).

- [ ] Päätä: automaattinen (OS-locale) vai manuaalinen valitsin asetuksissa, vai molemmat (OS oletuksena, manuaalinen ohitus)
- [ ] Tallennus: missä kieliasetus pysyy (tiedosto? env-muuttuja? Android `SharedPreferences`?)
- [ ] UI: kielivalitsin `config.html`/asetussivulle (desktop) ja Android-asetuksiin

**Valmis kun:** Käyttäjä löytää kielivalinnan ja se pysyy seuraavalla käynnistyksellä.

---

## Avoimet päätökset (koko roadmapin yli)

- [ ] Yksi yhtenäinen i18n-mekanismi koko reposssa vai pintakohtainen (Android natiivi, muualla Fluent/oma)?
- [ ] Oletuskieli aina suomi, vai seurataanko laitteen localea heti alusta?
- [ ] Käännösten ylläpito: kuka kääntää jatkossa, missä tekstit pidetään ajan tasalla kun ominaisuuksia lisätään?
- [ ] Lisätäänkö englanti kolmanneksi kieleksi myöhemmin (esim. testaajia/kehittäjiä varten)?

---

## Ei kuulu tähän roadmapiin (myöhemmin)

- Hopeakettu/Lapsi-profiilien mahdolliset kielikohtaiset whitelistat
- Crawlerin/hakuindeksin monikielisyys (haku toimii nyt vain sillä kielellä millä sivusto on kirjoitettu)
- Pulloposti-daemonin (suljettu repo) omat tekstit

---

*Kotisatama on osa Ilio-toiminimeä (Y-tunnus 2010). Kieliroadmap täydentää `ROADMAP-1.md`:ää, ei korvaa sitä.*
