# VAIHE7-TEEMAT.md — Satama/Avomeri/Myrsky-tilanvaihto

*Suunnitelma, ei vielä toteutettu. Tarkoitettu Cursorille toteutuksen pohjaksi.*

---

## Mikä on jo olemassa

Tilan tunnistamiseen tarvittavat signaalit ovat **jo koodissa**, vain yhdistämättä mihinkään visuaaliseen tilaan:

| Tila | Signaali | Missä |
|---|---|---|
| **Satama** (normaali) | Oletus — ei avomeri, ei myrsky | — |
| **Avomeri** | `kotisatama_whitelist::is_avomeri_gateway(&url)` palauttaa `true`, tai `kotisatama::is_blocked_page(location)` ja käyttäjä on jatkanut Startpageen | `components/kotisatama/whitelist/src/lib.rs:120`, `ports/servoshell/kotisatama.rs:121` |
| **Myrsky** | `SearchOutcome::Error(_)` haun yhteydessä (Meilisearch ei vastaa / offline) | `components/kotisatama/search/src/lib.rs:41-45`, palautuu `kotisatama::search()`-funktiosta |

Eli ei tarvita uutta tunnistuslogiikkaa — tarvitaan **yksi yhdistävä tila-enum** ja **kytkentä taustakuvaan**.

---

## Mitä puuttuu

### 1. Asset-siivous

`assets/themes/{Satama,Avomeri,Myrsky}/` sisältää tällä hetkellä puhelimen kuvakaappauksia (`Screenshot_20260613-231349.Kuvat.png` -tyyppisiä tiedostonimiä), ei lopullisia optimoituja taustakuvia. Konseptidokumentin mukaan lopputulosten pitäisi olla `kotisatama.webp`, `avomeri.webp`, `myrsky.webp`.

- [ ] Päätä lopulliset taustakuvat (nykyiset screenshotit placeholdereina vai uudet suunnitellut kuvat?)
- [ ] Optimoi/konvertoi PNG → WebP, nimeä uudelleen selkeästi: `assets/themes/satama.webp`, `avomeri.webp`, `myrsky.webp`
- [ ] **Myrsky-kuva pitää bundlata appiin** (ei CDN:n kautta) — se täytyy näkyä myös ilman verkkoyhteyttä (tämä periaate on jo kirjattu muistioihin, vain ei vielä toteutettu assetin sijoittelussa)

### 2. Tila-enum (Kotisatama-omistettu, ei upstream-riskiä)

Lisätään `ports/servoshell/kotisatama.rs` (joka on jo Kotisatama-spesifinen tiedosto, ei upstream):

```rust
// KOTISATAMA: UI-taustateema nykyisen selaustilan mukaan.
pub enum KotisatamaTheme {
    Satama,
    Avomeri,
    Myrsky,
}

pub fn current_theme(location: &str, last_search: Option<&SearchOutcome>) -> KotisatamaTheme {
    if matches!(last_search, Some(SearchOutcome::Error(_))) {
        return KotisatamaTheme::Myrsky;
    }
    if is_blocked_page(location) {
        return KotisatamaTheme::Avomeri; // käyttäjä jatkamassa avomerelle
    }
    if Url::parse(location)
        .map(|u| is_avomeri_gateway(&u))
        .unwrap_or(false)
    {
        return KotisatamaTheme::Avomeri;
    }
    KotisatamaTheme::Satama
}
```

*(Luonnos — Cursor tarkistaa tarkat tyypit/importit ennen käyttöä.)*

### 3. Kytkentä `gui.rs`:ään — minimaalisella otteella

`gui.rs` on upstream-jaettu tiedosto (sama periaate kuin kieliroadmapin vaihe 4). Tähän ei pidä viedä piirtologiikkaa kokonaisuudessaan, vain:

- [ ] Yksi kutsu `kotisatama::current_theme(...)` -> palauttaa enumin
- [ ] Yksi funktio (Kotisatama-omistetussa tiedostossa) joka piirtää taustan `egui::Context`-tasolle teeman mukaan — itse piirtologiikka pidetään `ports/servoshell/kotisatama.rs`:ssä, `gui.rs`:ään vain kutsu
- [ ] Merkitse kosketuskohta `KOTISATAMA-PATCH`-kommentilla (sama puute korjataan samalla kuin kieliroadmapin Vaihe 0:ssa — kannattaa tehdä molemmat yhdellä kertaa, koska kosketetaan samaa tiedostoa)

### 4. Android-puoli

- [ ] Sama `current_theme()`-logiikka kutsuttavissa JNI:n kautta (`egl/android/kotisatama.rs`), uusi natiivimetodi esim. `kotisatamaCurrentTheme`
- [ ] Taustakuvat myös Android-resursseihin (`res/drawable/`) — webp toimii natiivisti Androidilla

---

## Järjestysehdotus

1. Asset-siivous ensin (ei riipu koodista, voi tehdä rinnakkain)
2. `KotisatamaTheme`-enum + `current_theme()` `ports/servoshell/kotisatama.rs`:ään (täysin Kotisatama-omistettu, ei riskiä)
3. **Samalla kertaa** kun kosketetaan `gui.rs`:ää: lisää sekä teema-kutsu että puuttuvat `KOTISATAMA-PATCH`-merkinnät (yhdistä kieliroadmapin Vaihe 0:n kanssa — säästää yhden ylimääräisen upstream-kosketuskerran)
4. Desktop-testaus kaikilla kolmella tilalla
5. Android-kytkentä

---

## Avoimet päätökset

- [ ] Näytetäänkö tausta koko ikkunan takana vai vain tietyssä paneelissa (esim. uuden välilehden tausta)?
- [ ] Siirtymäanimaatio tilojen välillä, vai suora vaihto?
- [ ] Myrsky-tila: pitääkö sen pysyä päällä koko session ajan kunnes haku onnistuu uudelleen, vai tarkistetaanko jokaisella haulla erikseen?

---

*Täydentää `ROADMAP-1.md`:n vaihetta 7. Yhdistä `gui.rs`-kosketus kieliroadmapin (`KIELIROADMAP.md`) Vaihe 0:n kanssa, jos molemmat tehdään lähekkäin.*
