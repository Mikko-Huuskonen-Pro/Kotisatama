# KIELIROADMAP.md — Kotisatama (suomi + ruotsi)
*Kesäkuu 2026*

Tämä roadmap kattaa lokalisoinnin (i18n) rakentamisen Kotisatamaan, suomi oletuksena ja ruotsi ensimmäisenä lisäkielenä. Tällä hetkellä reposta **ei löydy minkäänlaista i18n-mekanismia** — kaikki teksti on kovakoodattu neljään eri paikkaan, kolmella eri tavalla. Tämä roadmap käy ne läpi pinta kerrallaan.

## Periaate: ei yhtenäistä i18n-kehystä koko repoon

Reposta löytyy kaksi eri tyyppistä tiedostoa, ja niitä **ei** kannata kohdella samalla tavalla:

- **Kotisatama-omistamat tiedostot** (`components/kotisatama/`, `resources/resource_protocol/*.html`, Android `res/values*/`) — eivät koskaan saa upstream-merge-konflikteja. Näihin voi rakentaa minkä tahansa i18n-ratkaisun vapaasti.
- **Upstream-jaetut tiedostot** (`ports/servoshell/desktop/gui.rs`) — synkronoituvat aktiivisesti Servon upstreamista (`git log` näyttää suoran `Merge upstream/servo main` -commitin tiedostoon). Jokainen rivi joka muutetaan tässä tiedostossa on potentiaalinen konfliktipiste jokaisessa tulevassa upstream-mergessä.

**Havainto matkan varrelta:** `gui.rs`:n ~49 suomenkielistä merkkijonoa eivät ole tällä hetkellä merkitty `KOTISATAMA-PATCH`-kommentilla, vaikka `AGENT.md` vaatii sen kaikille upstream-tiedostoihin tehdyille muutoksille. Tämä on olemassa oleva riski jo ilman kieliä — kannattaa korjata joka tapauksessa (ks. Vaihe 0).

Tämän vuoksi roadmap on järjestetty **upstream-konfliktiriskin mukaan**, ei alkuperäisen pinta-listan mukaan: ensin Kotisatama-omistetut pinnat (matala/ei riski), `gui.rs` viimeisenä ja kevyimmällä mahdollisella otteella.

---

## Nykytila ennen tätä roadmapia

| Pinta | Mekanismi | Kieli nyt |
|---|---|---|
| Android (`strings.xml`) | Android-resurssijärjestelmä (oikea, valmis pohja) | Sekakielinen: osa suomea, osa englantia |
| Sisäiset sivut (`resources/resource_protocol/*.html`) | Ei mitään — teksti suoraan HTML:ssä | Suomi, `lang="fi"` kovakoodattu |
| Desktop-UI (`ports/servoshell/desktop/gui.rs`, egui) | Ei mitään — ~49 merkkijonoa suoraan Rust-koodissa | Suomi |
| Blokkaussivu (`kotisatama-whitelist`-craten `lib.rs`) | Ei mitään — HTML generoidaan `format!`-makrolla | Suomi |

---

## Vaihe 0 — `gui.rs`: merkitse olemassa olevat muutokset (ei vielä kieliä)

**Tavoite:** Korjaa olemassa oleva piilevä riski ennen kuin tiedostoon kosketaan enempää.

- [ ] Käy läpi `ports/servoshell/desktop/gui.rs` ja lisää `KOTISATAMA-PATCH`-kommentit jokaisen kohdan päälle, jossa on suomenkielistä tekstiä tai muuta Kotisatama-spesifistä logiikkaa (AGENT.md:n vaatima käytäntö)
- [ ] Vertaa upstream-Servon `gui.rs`-vastineeseen, jotta nähdään tarkka diffi
- [ ] Ei vielä mitään käännösmuutoksia tässä vaiheessa — vain merkinnät

**Valmis kun:** Seuraava `git fetch upstream && git merge upstream/main` näyttää konfliktit selkeästi merkityissä kohdissa, ei yllätyksinä.

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

- [ ] Erota HTML-templaatti `format!`-makrosta omaksi tekstikartaksi (samalla mekanismilla kuin vaihe 2 — molemmat Kotisatama-omistettuja tiedostoja, joten sama ratkaisu sopii molempiin ilman upstream-riskiä)
- [ ] Käännä: otsikko, "Jatka avomerelle", raportointiohje
- [ ] Varmista että kielivalinta kulkee samalla tavalla kuin muu UI (ei oma erillinen logiikka)

**Valmis kun:** Blokkaussivu noudattaa samaa kieliasetusta kuin loppu selain.

---

## Vaihe 4 — Desktop-UI (`gui.rs`, egui) — viimeisenä, kevyimmällä otteella

**Tavoite:** ~49 kovakoodattua merkkijonoa irti Rust-koodista — **mutta ilman että tiedostoon koskettava diffi paisuu**, koska se on upstream-jaettu tiedosto.

Tämä on tarkoituksella roadmapin viimeinen vaihe. Suositus: **älä** rakenna Fluentia tai mitään muuta kehystä suoraan `gui.rs`:ään. Sen sijaan:

- [ ] Tee `components/kotisatama/i18n/`-crate (Kotisatama-omistettu, ei upstream-riskiä) joka sisältää kaiken käännöslogiikan ja -datan
- [ ] Tarjoa sieltä yksi minimaalinen funktio, esim. `t("avomeri_jatka")` — yksi rivi per kutsupaikka `gui.rs`:ssä
- [ ] Korvaa kukin 49 literaalista yhdellä `t(...)`-kutsulla — pidä jokainen muutosrivi mahdollisimman pieni ja merkitse `KOTISATAMA-PATCH`
- [ ] Käännä ruotsiksi `i18n`-craten sisällä (ei `gui.rs`:ssä)
- [ ] Kielivalinta: sama lähde kuin vaiheissa 2–3

**Miksi viimeisenä:** Jokainen `gui.rs`:ään tehty muutos on upstream-mergessä mahdollinen konfliktipiste. Kannattaa odottaa että vaiheet 0–3 ovat valmiit ja käännösmekanismi (tekstikartta, kielivalinnan tallennus) on jo testattu Kotisatama-omistetuilla pinnoilla, ennen kuin samaa kuviota tuodaan upstream-tiedostoon.

**Valmis kun:** Desktop-UI vaihtaa kielen yhdestä asetuksesta, `gui.rs`-diffi pysyy minimissä (yksi `t(...)`-kutsu per rivi, ei käännöslogiikkaa itse tiedostossa).

---

## Vaihe 5 — Kielivalinta käyttäjälle

**Tavoite:** Käyttäjä voi vaihtaa kielen (ei vain seuraa käyttöjärjestelmän localea).

- [ ] Päätä: automaattinen (OS-locale) vai manuaalinen valitsin asetuksissa, vai molemmat (OS oletuksena, manuaalinen ohitus)
- [ ] Tallennus: missä kieliasetus pysyy (tiedosto? env-muuttuja? Android `SharedPreferences`?)
- [ ] UI: kielivalitsin `config.html`/asetussivulle (desktop) ja Android-asetuksiin

**Valmis kun:** Käyttäjä löytää kielivalinnan ja se pysyy seuraavalla käynnistyksellä.

---

## Avoimet päätökset (koko roadmapin yli)

- [x] ~~Yksi yhtenäinen i18n-mekanismi koko repossa?~~ — **Päätetty: ei.** Pintakohtainen ratkaisu, upstream-konfliktiriski ohjaa (ks. yllä "Periaate").
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
