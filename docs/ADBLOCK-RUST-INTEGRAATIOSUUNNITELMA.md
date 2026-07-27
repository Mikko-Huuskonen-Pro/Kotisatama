# Katselin — adblock-rust-integraatiosuunnitelma

**Tila:** hyväksytty toteutettavaksi  
**Ensimmäinen kohde:** nykyinen Katselin (Windows + Android)  
**Myöhempi kohde:** Kotisatama OS  
**Päivitetty:** 27.7.2026

**Lukitut päätökset (27.7.2026):**

- Moottori on Katselimen fork [adblock-Katselin](https://github.com/Mikko-Huuskonen-Pro/adblock-Katselin) (`adblock` crate), ei crates.io:n Brave-upstream suoraan.
- Integraatio sijoitetaan Kotisatama-malliin (`components/kotisatama/`, feature-flag, minimaaliset `KOTISATAMA-PATCH`-hookit) niin, että nykyinen whitelist/haku/UI-polku ei rikkoudu.

> **Tavoite ei ole lisätä selaimeen yhtä ominaisuutta lisää.**  
> Tavoite on tehdä verkosta rauhallisempi, nopeampi ja turvallisempi oletuksena.

---

## 1. Päätös

Katselimeen integroidaan Braven avoimen lähdekoodin **adblock-rust**-moottorin fork **adblock-Katselin**:

- Fork-repo: https://github.com/Mikko-Huuskonen-Pro/adblock-Katselin  
- Package-nimi Cargo:ssa: `adblock` (nykyinen fork-versio esim. `0.13.2`)  
- Riippuvuus: **git** forkkiseen (paikallisessa kehityksessä voi käyttää `path = "../adblock-Katselin"`)

Ensimmäinen julkaistava versio keskittyy verkkopyyntöjen estämiseen. Kosmeettinen suodatus, resurssien korvaukset ja tarkempi käyttöliittymä lisätään vasta vakaan verkkosuodatuksen jälkeen.

Integraatio tehdään omana Kotisatama-cratena niin, että:

- Servo ei tunne käyttöliittymän asetuksia
- käyttöliittymä ei tunne suodatinmoottorin yksityiskohtia
- Android, Windows ja myöhemmin Linux käyttävät samaa suodatusydintä
- forkkia voidaan päivittää ilman laajoja muutoksia Katselimeen
- suodatus voidaan myöhemmin korvata toisella toteutuksella muuttamatta selaimen muita osia
- ilman featurea / moottorin epäonnistuessa selain käyttäytyy kuten nykyinen toimiva malli (**fail-open**)

---

## 2. Miksi tämä kuuluu Katselimeen

Kotisataman periaatteiden mukaan käyttäjä käyttää palvelua, ei infrastruktuuria. Mainosteneston tulee siksi toimia pääosin näkymättömänä taustalla.

Integraatio tukee suoraan Katselimen tavoitteita:

- vähemmän visuaalista hälyä
- vähemmän seurantaa
- nopeammat sivulataukset
- pienempi datankulutus Androidilla
- kevyempi kuorma vanhoilla laitteilla
- turvalliset oletukset ilman asetusten opiskelua

Mainostenesto ei ole Katselimessa erillinen lisäosa. Se on osa rauhallista verkkokokemusta.

---

## 3. Rajaus

### Ensimmäinen MVP

MVP sisältää:

1. `adblock`-riippuvuuden (adblock-Katselin-fork) lisäämisen Rust-workspaceen
2. yhden valitun suodatinlistan lataamisen paikallisesta paketista
3. suodatinmoottorin alustamisen selaimen käynnistyessä
4. HTTP- ja HTTPS-alipyyntöjen tarkistamisen ennen verkkoon lähettämistä
5. estettyjen pyyntöjen laskurin välilehtikohtaisesti
6. mahdollisuuden poistaa suojaus käytöstä nykyiseltä sivustolta
7. lokituksen kehittäjätilassa
8. automaattiset yksikkö- ja integraatiotestit

### Ei ensimmäiseen MVP:hen

Seuraavat jätetään myöhemmäksi:

- kosmeettinen suodatus
- scriptlet-injektiot
- resurssien korvaaminen
- useiden listojen käyttöliittymä
- käyttäjän omat säännöt
- pilvisynkronointi
- suodatinlistojen taustapäivitykset
- yksityiskohtainen estoloki tavalliselle käyttäjälle
- aggressiivinen evästebannerien poisto

Näin ensimmäinen versio todistaa yhden asian luotettavasti: **estetäänkö verkkopyyntö oikein ennen sen lähettämistä?**

---

## 4. Ehdotettu arkkitehtuuri

Sovitetaan AGENT.md:n malliin: logiikka `components/kotisatama/`-cratessa, UI/asetukset servoshellissa, mahdollinen verkko-hook minimaalisena `KOTISATAMA-PATCH`-kohtana.

```text
Servoshell UI (asetukset / sivustopoikkeus / laskuri)
   │
   ▼
kotisatama-content-blocking  (ContentBlockingService)
   ├── FilterListStore
   ├── SiteExceptionStore
   ├── BlockingStatistics
   └── adblock-Katselin Engine  (crate: adblock)
             │
             ▼
Servo-verkkopyynnön tarkistuspiste  #[cfg(feature = "kotisatama")]
             │
       ┌─────┼─────┐
       │     │     │
     salli  estä  fail-open
       │     │     │
       ▼     ▼     ▼
   verkko  esto  verkko (nykyinen polku)
```

Whitelist-navigointihook (`request_navigation` / `ports/servoshell`) **ei korvaa** alipyyntösuodatusta. Navigointirajoitus ja sisältösuodatus ovat erilliset kerrokset.

### Moduulien vastuut

#### `ContentBlockingService` (`kotisatama-content-blocking`)

Katselimen oma julkinen rajapinta suodatukselle.

Vastaa:

- moottorin alustamisesta
- pyyntöjen tarkistamisesta
- moottorin turvallisesta vaihtamisesta listapäivityksen yhteydessä
- sivustokohtaisten poikkeusten huomioimisesta
- tilastojen välittämisestä

Muu Katselin ei käytä `adblock`-craten tyyppejä suoraan — vain adapteri (`adblock_adapter.rs`).

#### `FilterListStore`

Vastaa:

- paketoidun oletuslistan lukemisesta
- listan version ja lähteen tallentamisesta
- myöhemmin listapäivityksistä
- viimeisen toimivan listan säilyttämisestä
- listan koon ja eheyden tarkistamisesta

#### `SiteExceptionStore`

Vastaa käyttäjän hyväksymistä sivustokohtaisista poikkeuksista.

Ensimmäisessä versiossa tallennetaan vain normalisoitu sivustotunnus, esimerkiksi:

```text
example.fi
```

Ei koko URL-osoitetta, selaushistoriaa tai estettyjen resurssien nimiä pysyvään tallennukseen.

#### `BlockingStatistics`

Vastaa vain käyttöliittymälle tarpeellisista lyhytikäisistä tiedoista:

- estettyjen pyyntöjen määrä nykyisellä sivulla
- mahdollinen estoluokka kehittäjätilassa
- suodatuksen tila

Tilastot pidetään muistissa eikä niitä lähetetä palvelimelle.

#### Käyttöliittymä / asetukset (ei erillinen Servo-komponentti)

“PrivacyController”-rooli toteutetaan olemassa olevan servoshell-UI:n ja asetusten laajennuksena (desktop egui / Android Compose), ei uutena Servo-upstream-komponenttina.

---

## 4.1 Feature-flag ja nykyisen mallin säilytys

Noudatetaan AGENT.md:tä:

| Tilanne | Käyttäytyminen |
|---|---|
| `kotisatama`-feature pois | ei content-blocking-riippuvuutta; nykyinen Servo/Kotisatama-polku |
| Feature päällä, moottori OK | alipyynnöt tarkistetaan ennen lähetystä |
| Feature päällä, alustus epäonnistuu | **fail-open**: whitelist, haku, raportti, navigointi toimivat kuten nyt; UI ei väitä suojausta päälle |
| Verkkohook puuttuu vielä (Vaihe 0–1) | crate ja testit olemassa; selainbuild toimii ilman estoa |

Suositeltu kytkentä `ports/servoshell/Cargo.toml`:ssa:

- uusi valinnainen workspace-dep `kotisatama-content-blocking`
- joko osa olemassa olevaa `kotisatama`-featurea, tai alifeature `kotisatama-adblock` joka sisällytetään `kotisatama`-featureen

Workspace-merkintä samaan tapaan kuin muut Kotisatama-cratet:

```toml
kotisatama-content-blocking = { path = "components/kotisatama/content-blocking" }
```

Fork-riippuvuus craten `Cargo.toml`:ssa (esimerkki):

```toml
adblock = { git = "https://github.com/Mikko-Huuskonen-Pro/adblock-Katselin", default-features = false, features = ["embedded-domain-resolver", "full-regex-handling"] }
# paikallinen kehitys:
# adblock = { path = "../../../adblock-Katselin", default-features = false, features = ["..."] }
```

`single-thread`-feature forkin oletuksissa: säiemalli päätetään Vaihe 1:ssä forkin todellisen `Send`/`Sync`-tuen mukaan (§9).

---

## 5. Katselimen sisäinen rajapinta

Suositeltu oma abstraktio (`kotisatama-content-blocking`):

```rust
pub trait RequestBlocker {
    fn check(&self, request: &BlockingRequest) -> BlockingDecision;
}

pub struct BlockingRequest<'a> {
    pub url: &'a str,
    pub source_url: &'a str,
    pub resource_type: ResourceType,
}

pub enum BlockingDecision {
    Allow,
    Block,
    Redirect { resource: Vec<u8>, mime_type: String },
}
```

Ensimmäisessä MVP:ssä käytetään vain päätöksiä `Allow` ja `Block`. `Redirect` varataan myöhempää resurssien korvaamista varten.

Tämän rajapinnan hyöty:

- Servo-integraatio pysyy riippumattomana forkin API-muutoksista
- testit voivat käyttää vale-estäjää
- Android ja työpöytä käyttävät samaa sovelluslogiikkaa
- moottori voidaan tulevaisuudessa vaihtaa

---

## 6. Servo-integraation periaate

Suodatus on tehtävä kohdassa, jossa Katselin tai sen Servo-integraatiokerros tietää vähintään:

- pyydettävän resurssin URL:n
- ylimmän sivun tai pyynnön käynnistäneen dokumentin URL:n
- resurssityypin

Tarkistus tehdään **ennen verkkopyynnön lähettämistä**.

```text
Servo muodostaa pyynnön
        ↓
Katselin muodostaa BlockingRequestin
        ↓
adblock-Katselin (Engine) tarkistaa pyynnön
        ↓
Allow / fail-open → normaali lataus
Block → pyyntöä ei lähetetä
```

### Tärkeä tekninen selvitys ennen toteutusta

Nykyisestä Katselin/Servo-versiosta on tunnistettava todellinen verkkopyyntöjen välityspiste. Tarkkaa tiedosto- tai API-nimeä ei lukita tähän suunnitelmaan ennen lähdekoodin auditointia, koska Servo-rajapinnat muuttuvat.

Etsi ensisijaisesti kohta, jossa:

- `Request` tai vastaava verkkopyyntö rakennetaan
- resurssityyppi on vielä tiedossa
- pyyntö voidaan keskeyttää ilman keinotekoista verkkovirhettä
- päädokumentti ja alipyyntö voidaan erottaa toisistaan

Älä aloita DOM-tasolta tai JavaScript-injektiolla. Verkkosuodatus kuuluu verkkokerrokseen.

Jos hook tarvitaan `components/net/`-tasolla: vain minimaalinen `#[cfg(feature = "kotisatama")]`-kutsu + `KOTISATAMA-PATCH`-kommentti; logiikka pysyy `kotisatama-content-blocking`-cratessa. Älä muokkaa upstreamia ennen Vaihe 0 -auditointia.

---

## 7. Resurssityyppien muunnos

Servon resurssityyppi muunnetaan forkin tuntemaan tyyppiin yhdessä paikassa (`adblock_adapter.rs`).

Esimerkkitaulukko:

| Servo/Katselin | adblock (fork) | Huomio |
|---|---|---|
| päädokumentti | document | ei oletuksena estetä ilman vahvaa sääntöä |
| iframe | subdocument | säilytä lähdesivun tieto |
| JavaScript | script | keskeinen seurannan estossa |
| CSS | stylesheet | tarkista sivustojen rikkoutuminen |
| kuva | image | suuri vaikutus datankulutukseen |
| fontti | font | kolmannen osapuolen fontit voivat seurata |
| XHR/fetch | xmlhttprequest | keskeinen analytiikassa |
| media | media | video- ja äänipyynnöt |
| WebSocket | websocket | jos Servo-kytkentäpiste tukee sitä |
| muu | other | turvallinen oletus |

Tuntematonta tyyppiä ei pidä estää vain siksi, että muunnos epäonnistui. Se käsitellään tyyppinä `other` ja kirjataan kehittäjätilassa.

---

## 8. Suodatinlistat

### Ensimmäinen julkaisu

Ensimmäiseen julkaisuun valitaan yksi maltillinen oletuslista tai tarkkaan määritelty listakokonaisuus. Tavoite ei ole estää mahdollisimman paljon vaan rikkoa mahdollisimman vähän.

Valintakriteerit:

- lisenssi sallii jakelun Katselimen mukana
- lähde ja päivitystapa ovat dokumentoituja
- mainokset ja yleinen seuranta estyvät
- suomalaiset palvelut testataan erikseen
- lista ei sisällä tarpeettoman aggressiivisia häirintäsääntöjä

### Paketoitu lista

Selaimen mukana toimitetaan toimiva lista, jotta:

- suojaus toimii ensimmäisellä käynnistyksellä
- ensimmäinen käynnistys ei riipu verkkoyhteydestä
- epäonnistunut päivitys ei poista suojausta

Lista elää craten `assets/`-hakemistossa (§19), ei forkin repossa.

### Myöhempi päivitysmalli

```text
paketoitu lista
      ↓
käynnistys toimii aina
      ↓
Katselin tarkistaa uudemman listan
      ↓
lataa väliaikaiseen tiedostoon
      ↓
tarkistaa koon, muodon ja eheyden
      ↓
rakentaa uuden Enginen
      ↓
vaihtaa moottorin atomisesti
      ↓
säilyttää edellisen toimivan version
```

Moottori rakennetaan uudelleen listan vaihtuessa. Sääntöjä ei lisätä yksitellen elävään moottoriin.

---

## 9. Säikeistys ja suorituskyky

Forkin `Engine`-tyypin säieominaisuudet riippuvat crate-versiosta ja feature-valinnoista. Toteutuksessa ei oleteta sokkona, että sama moottori voidaan jakaa kaikille säikeille.

Ennen integraatiota tarkistetaan käytetyn fork-version:

- `Send`-tuki
- `Sync`-tuki
- `single-thread`- tai muut featuret
- WASM/Android-vaikutukset

Turvalliset toteutusvaihtoehdot:

1. yksi suodatuspalvelun omistama säie ja viestikanava
2. säiekohtainen moottori
3. jaettava moottori lukon takana, jos käytetty versio tukee sitä järkevästi

Pyyntökohtainen tarkistus ei saa:

- lukea listaa levyltä
- rakentaa moottoria uudelleen
- tehdä verkkopyyntöä
- kirjoittaa pysyvää lokia

### Suorituskykytavoitteet

Ensimmäisessä mittauksessa seurataan:

- moottorin alustusajan mediaani
- muistinkulutus listan latauksen jälkeen
- yhden pyynnön tarkistuksen mediaani ja 95. persentiili
- sivun kokonaislatausaika suojaus päällä ja pois
- estettyjen tavujen arvio Androidilla

Tavoite on, ettei suodatus aiheuta käyttäjän havaittavaa viivettä.

---

## 10. Käyttöliittymä

### Oletus

Suojaus on oletuksena päällä (kun moottori on alustettu onnistuneesti).

Käyttäjän ei tarvitse:

- valita suodatinlistaa
- ymmärtää sääntösyntaksia
- asentaa lisäosaa
- hyväksyä erillistä suojaustilaa

### Sivustokohtainen näkymä

Ensimmäinen käyttöliittymä voi sisältää:

```text
Suojaus tällä sivulla: päällä
Estetty: 12

[ Salli tällä sivustolla ]
```

Kun käyttäjä sallii sivuston:

- poikkeus koskee kyseistä sivustoa
- sivu ladataan uudelleen
- valinta voidaan perua samasta paikasta

Sanastossa vältetään teknisiä termejä kuten “filter engine”, “third-party request” ja “cosmetic rule”. Käyttäjätekstit suomeksi; crate- ja koodinimet englanniksi (`content-blocking`), kuten muissa Kotisatama-crateissa.

### Sivuston rikkoutuessa

Käyttäjälle tarjotaan yksinkertainen polku:

```text
Eikö sivu toimi oikein?
Kokeile sallia sisältö tällä sivustolla.
```

Vasta kehittäjätila näyttää tarkemmat tiedot.

---

## 11. Android

Android on integraation tärkeä ensimmäinen kohde, koska hyödyt näkyvät suoraan:

- pienempi datankulutus
- nopeampi lataus mobiiliverkossa
- pienempi akkukuorma
- vähemmän ruudun peittävää mainontaa

Android-toteutuksessa tarkistetaan lisäksi:

- sovelluksen pakettikoon kasvu
- alustus matalan tehon laitteella
- muistipaine sovelluksen siirtyessä taustalle
- suodatinmoottorin palautuminen prosessin uudelleenkäynnistyksen jälkeen
- listan tallennus sovelluksen yksityiseen hakemistoon

Ensimmäisen version ei pidä vaatia Androidin VPN- tai saavutettavuuspalvelua. Suodatus tapahtuu Katselimen oman Servo-liikenteen sisällä (servoshell EGL), sama `kotisatama-content-blocking`-ydin kuin desktopilla.

---

## 12. Kosmeettinen suodatus — vaihe 2

Verkkosuodatus estää resurssin lataamisen, mutta sivulle voi jäädä tyhjiä mainosalueita. Kosmeettinen suodatus ratkaisee tämän.

Fork tarjoaa URL-kohtaisia kosmeettisia resursseja sekä dynaamisten luokkien ja tunnisteiden perusteella tuotettavia piilotussääntöjä.

Ehdotettu eteneminen:

1. hae URL-kohtaiset kosmeettiset resurssit ennen sivun valmistumista
2. muodosta turvallinen käyttäjätyylisivu
3. injektoi CSS dokumenttiin Katselimen hallitsemassa kohdassa
4. kerää DOM:n luokat ja tunnisteet hallitusti
5. pyydä tarvittavat geneeriset selektorit
6. päivitä tyylit ilman sivun välkkymistä

Scriptletit jätetään erilliseksi myöhemmäksi työksi, koska ne vaikuttavat sivun JavaScript-käyttäytymiseen ja vaativat tiukemman turvallisuusarvioinnin.

---

## 13. Tietosuoja

Kaikki suodatus tehdään paikallisesti.

Katselin ei lähetä palvelimelle:

- avattuja URL-osoitteita
- estettyjä URL-osoitteita
- sivustokohtaisia tilastoja
- käyttäjän poikkeuslistaa

Suodatinlistojen päivityspyynnöt eivät saa sisältää selaushistoriaa. Päivityspalvelu näkee korkeintaan Katselimen version, listan version ja normaalit verkkoyhteyden metatiedot.

Jos telemetriaa joskus lisätään, se suunnitellaan erillisenä päätöksenä eikä tämän integraation sivutuotteena.

---

## 14. Lisenssit ja ilmoitukset

adblock-rust / adblock-Katselin on MPL-2.0-lisensoitu. Katselimen on säilytettävä kirjaston lisenssi- ja tekijänoikeustiedot jakelussa.

Tehtävät:

- lisää forkin `adblock` kolmansien osapuolten lisenssiluetteloon
- toimita MPL-2.0-lisenssiteksti jakelun mukana
- dokumentoi käytetty fork-commit / crate-versio
- julkaise forkin MPL-tiedostoihin tehdyt muutokset MPL-2.0:n mukaisesti **fork-repon kautta**
- pidä Katselimen oma integraatiokoodi (`kotisatama-content-blocking`) erillään forkin lähdetiedostoista
- tarkista jokaisen mukana toimitettavan suodatinlistan oma lisenssi erikseen

**Päätös:** käytetään Katselimen forkkia `adblock-Katselin`, ei crates.io:n Brave-upstreamia suoraan. Fork elää omassa repossaan; sitä ei kopioida `components/`-alle. Tarvittavat Servo/Kotisatama-sovitukset tehdään adapterikerroksessa.

Tämä dokumentti ei korvaa oikeudellista neuvontaa.

---

## 15. Virhetilanteet ja turvalliset oletukset

| Tilanne | Toiminta |
|---|---|
| Paketoitu lista puuttuu | käynnistä selain ilman suodatusta ja näytä kehittäjäloki |
| Päivitetty lista vioittunut | käytä edellistä toimivaa listaa |
| Moottorin alustus epäonnistuu | selain toimii (whitelist/haku/UI ennallaan), suojaustila ilmoittaa virheestä |
| Pyynnön URL ei jäsenny | salli pyyntö ja kirjaa kehittäjätilassa |
| Resurssityyppi tuntematon | käsittele tyyppinä `other` |
| Poikkeustietokanta vioittunut | älä kaada selainta; palauta tyhjä poikkeuslista |
| Suodatuspalvelu ei vastaa | **fail-open**: salli pyyntö, älä väitä suojausta päälle |

Ensimmäisessä versiossa käytetään **fail-open**-periaatetta teknisissä virheissä: selain ei lakkaa toimimasta suodatinmoottorin virheen vuoksi. Käyttäjälle ei kuitenkaan väitetä suojauksen olevan päällä, jos se ei ole. Nykyinen toimiva malli on turvallinen oletus.

---

## 16. Testaus

### Yksikkötestit

- tunnettu mainos-URL estyy
- tavallinen ensimmäisen osapuolen resurssi sallitaan
- poikkeussääntö sallii muuten estettävän pyynnön
- resurssityypit muunnetaan oikein
- virheellinen URL ei kaada palvelua
- sivuston normalisointi toimii aliverkkotunnuksilla
- moottorin vaihto ei kadota toimivaa versiota epäonnistuessa
- `cargo test -p kotisatama-content-blocking` (ilman Servoa)

### Integraatiotestit

Paikallinen testipalvelin tarjoaa sivun, jossa on:

- ensimmäisen osapuolen kuva
- kolmannen osapuolen mainosskripti
- analytiikkapyyntö
- iframe
- XHR/fetch
- WebSocket, jos kytkentäpiste tukee sitä

Testi varmistaa, että:

- sallitut resurssit latautuvat
- estettävät pyynnöt eivät saavu testipalvelimelle
- päädokumentti avautuu
- laskuri päivittyy
- sivustokohtainen sallinta toimii uudelleenlatauksen jälkeen
- whitelist-/haku-/raporttipolut eivät regressoidu

### Regressiotestit suomalaisilla palveluilla

Pidetään erillinen käsin testattava lista ainakin seuraavista luokista:

- pankit
- tunnistautuminen
- viranomaispalvelut
- terveyspalvelut
- uutiset
- verkkokaupat
- videopalvelut
- kartat
- sähköposti

Valkoisiin sivuihin hyväksytyille palveluille voidaan myöhemmin rakentaa automaattinen savutesti, mutta Valkoiset sivut eivät saa muodostua piilotetuksi yleiseksi sallittujen listaksi.

---

## 17. Toteutusvaiheet

### Vaihe 0 — lähdekoodin auditointi

- paikanna Katselimen nykyinen Servo-verkkokerros
- paikanna päädokumentin ja alipyyntöjen konteksti
- tarkista resurssityyppien saatavuus
- tarkista Android- ja Windows-polkujen erot
- kirjaa sopiva keskeytysmekanismi
- varmista, ettei ehdotettu hook riko whitelist/`request_navigation`-polkua

**Valmis, kun:** yksi dokumentoitu kytkentäpiste on valittu.

### Vaihe 1 — erillinen suodatusmoduuli (ei Servo-hookia)

- lisää `kotisatama-content-blocking` workspaceen
- lisää git-/path-riippuvuus forkkiseen `adblock-Katselin`
- luo `ContentBlockingService`, `BlockingRequest`, `BlockingDecision`
- lataa pieni testisääntöjoukko
- kirjoita yksikkötestit: `cargo test -p kotisatama-content-blocking`
- älä kytke vielä verkko-hookia — selainbuild säilyy ennallaan

**Valmis, kun:** Rust-testi estää tunnetun testipyynnön ilman Servoa.

### Vaihe 2 — Servo-verkkosuodatus

- muodosta pyyntöadapteri
- tarkista pyyntö ennen lähettämistä (`#[cfg(feature = "kotisatama")]`)
- toteuta turvallinen estovastaus + fail-open
- varmista, ettei estetty pyyntö lähde verkkoon
- lisää kehittäjäloki
- regressiotarkistus: whitelist, haku, raportti

**Valmis, kun:** paikallinen integraatiotesti todistaa eston eikä nykyinen malli regressoidu.

### Vaihe 3 — tuotantolista ja välimuisti

- valitse lisenssiltään sopiva oletuslista
- paketoi lista craten `assets/`-hakemistoon
- rakenna Engine käynnistyksessä
- serialisoi tai välimuistita moottori, jos mittaukset osoittavat tarpeen
- varmista palautuminen versiomuunnoksissa

**Valmis, kun:** suojaus toimii ensimmäisellä käynnistyksellä ilman verkkoyhteyttä.

### Vaihe 4 — käyttöliittymä

- suojaus oletuksena päälle (kun moottori OK)
- estettyjen määrä nykyisellä sivulla
- “Salli tällä sivustolla”
- poikkeuksen poistaminen
- selkeä virhetila (fail-open näkyy rehellisesti)

**Valmis, kun:** tavallinen käyttäjä pystyy korjaamaan rikkoutuneen sivun ilman teknisiä käsitteitä.

### Vaihe 5 — Android-validointi

- testaa vähintään yhdellä hitaalla ja yhdellä uudemmalla laitteella
- mittaa käynnistys, muisti, datankulutus ja latausaika
- testaa taustalta palautuminen
- testaa APK/AAB-kokomuutos

**Valmis, kun:** hyöty on mitattava eikä käyttökokemus hidastu havaittavasti.

### Vaihe 6 — kosmeettinen suodatus

- lisää URL-kohtaiset CSS-säännöt
- lisää dynaamisten selektorien käsittely
- estä sivun välkkyminen
- testaa saavutettavuus ja tulostusnäkymä

**Valmis, kun:** tyhjät mainosalueet poistuvat ilman laajaa sivustojen rikkoutumista.

### Vaihe 7 — listapäivitykset

- allekirjoitettu tai muuten eheystarkistettu jakelukanava
- atominen päivitys
- palautus edelliseen listaan
- kohtuullinen päivitysväli
- käyttöliittymässä vain viimeisin onnistunut päivitysaika

**Valmis, kun:** epäonnistunut tai viallinen päivitys ei heikennä selaimen toimintaa.

---

## 18. Ensimmäisen version hyväksymiskriteerit

Integraatio voidaan julkaista, kun kaikki seuraavat täyttyvät:

- [ ] adblock-Katselin on eristetty `kotisatama-content-blocking`-adapterin taakse
- [ ] paketoitu suodatinlista toimii ilman verkkoyhteyttä
- [ ] estettyä pyyntöä ei lähetetä verkkoon
- [ ] sallittu ensimmäisen osapuolen resurssi latautuu normaalisti
- [ ] päädokumenttien virhe-estot on minimoitu
- [ ] suojaus toimii Windowsissa
- [ ] suojaus toimii Androidissa
- [ ] sivustokohtainen sallinta toimii
- [ ] suodatusvirhe ei kaada selainta (fail-open)
- [ ] käyttäjälle ei näytetä väärää suojaustilaa
- [ ] whitelist / haku / raportti / navigointi eivät regressoidu
- [ ] lisenssitiedot toimitetaan jakelun mukana
- [ ] suodatinlistan lisenssi on tarkistettu
- [ ] suorituskykymittaukset on kirjattu
- [ ] suomalaiset kriittiset palveluluokat on savutestattu

---

## 19. Ehdotettu hakemistorakenne

AGENT.md:n mukainen sijoitus:

```text
components/kotisatama/
├── whitelist/              ← olemassa
├── search/                 ← olemassa
├── report/                 ← olemassa
├── ...
└── content-blocking/       ← uusi: kotisatama-content-blocking
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs
    │   ├── service.rs
    │   ├── request.rs
    │   ├── decision.rs
    │   ├── adblock_adapter.rs   ← ainoa paikka joka tuntee crate:n adblock
    │   ├── filter_store.rs
    │   ├── exceptions.rs
    │   └── statistics.rs
    ├── assets/
    │   ├── filters.txt
    │   └── FILTER-LICENSES.md
    └── tests/
        ├── network_rules.rs
        └── exception_rules.rs

ports/servoshell/           ← UI-asetukset + mahdollinen init-kutsu
components/net/             ← vain jos audit vaatii: minimaalinen KOTISATAMA-PATCH
```

Fork `adblock-Katselin` elää erillisessä repossa (git-riippuvuus). Älä kopioi forkin lähdetiedostoja Katselimen `components/`-alle.

Kolmansien osapuolten lisenssit: olemassa oleva / uusi kohta jakelun lisenssiluettelossa (esim. `third_party/LICENSES.md` tai vastaava).

---

## 20. Ensimmäinen tehtävä Cursorille tai Codexille

```text
Audit Katselin's current Servo network loading path for kotisatama-content-blocking
(adblock-Katselin fork). Do not implement blocking yet.

Deliver:
1. The exact code path used for top-level and subresource HTTP(S) requests.
2. The earliest safe interception point before a request is sent.
3. Which request context is available there: target URL, source/top-level URL,
   resource type, method, headers and tab/webview identity.
4. How a request can be cancelled without crashing or hanging the load.
5. Differences between Windows and Android paths.
6. A proposed minimal RequestBlocker trait and adapter boundary for
   components/kotisatama/content-blocking.
7. How the hook stays behind #[cfg(feature = "kotisatama")] / fail-open so
   whitelist, search and report paths keep working unchanged.
8. Tests that can later prove a blocked request never reaches a local test server.

Constraints:
- Depend on Mikko-Huuskonen-Pro/adblock-Katselin (git/path), not crates.io adblock.
- Do not copy fork sources into components/.
- Do not copy code from Brave or other browsers.
- Do not modify Servo upstream code unless no public embedding hook exists;
  if needed, only a minimal KOTISATAMA-PATCH.
- Prefer a small Kotisatama-owned adapter layer.
- Do not break existing whitelist / search / report / navigation behaviour.
- Do not add UI, list downloads or cosmetic filtering in this task.
- Document uncertainties instead of guessing APIs.
```

Auditin jälkeen varsinainen integraatio voidaan jakaa pieniin, tarkistettaviin muutoksiin (ensin Vaihe 1 crate + testit ilman hookia).

---

## 21. Riskit

### Sivustot rikkoutuvat

Torjunta:

- maltillinen oletuslista
- sivustokohtainen sallinta
- kriittisten suomalaisten palvelujen savutestit
- ei kosmeettisia scriptlettejä MVP:hen

### Servo-kytkentäpiste on liian myöhäinen

Torjunta:

- auditointi ennen toteutusta
- tarvittaessa pieni upstream-ystävällinen hook (`KOTISATAMA-PATCH`)
- vältä laajaa Servo-forkkia

### Nykyinen Kotisatama-malli regressoituu

Torjunta:

- feature-flag + fail-open
- Vaihe 1 ilman verkko-hookia
- regressiotestit whitelist/haku/raportti

### Suodatinlista kasvattaa käynnistysaikaa

Torjunta:

- mittaa ensin
- rakenna moottori taustalla vain, jos turvallinen oletustila säilyy
- käytä serialisointia välimuistina, ei pysyvänä formaattina

### Android-muisti kasvaa liikaa

Torjunta:

- yksi maltillinen lista
- mittaus oikeilla laitteilla
- featureiden minimointi
- tarvittaessa mobiilille optimoitu listakokonaisuus

### Lisenssit unohtuvat

Torjunta:

- kolmansien osapuolten lisenssit osaksi buildia
- suodatinlistojen lisenssit omassa tiedostossa
- riippuvuusauditointi julkaisuprosessiin
- forkin MPL-muutokset julkaistaan fork-repon kautta

---

## 22. Ydinpäätös

Katselin ei rakenna omaa mainostenestomoottoria.

Katselin käyttää **adblock-Katselin**-forkkia infrastruktuurina ja rakentaa itse sen ympärille Kotisatama-malliin sopivan kerroksen:

- `kotisatama-content-blocking`-crate
- feature-flag ja fail-open (nykyinen malli säilyy)
- Servo-integraation (minimaalinen patch tarvittaessa)
- Android-kokemuksen
- sivustokohtaisen hallinnan
- suomalaisiin palveluihin sopivan laadunvarmistuksen

> **adblock-Katselin tekee suodatuksen. Katselin tekee verkosta rauhallisen — rikkomatta nykyistä satamaa.**

---

## 23. Tekniset lähteet tarkistusta varten

- Katselin-fork: `Mikko-Huuskonen-Pro/adblock-Katselin`
- Upstream-viite: Brave Software `brave/adblock-rust`
- Rust crate (forkissa): `adblock`
- Päärajapinta: `adblock::Engine`
- Verkkotarkistus: `Engine::check_network_request`
- Listojen kokoaminen: `FilterSet` ja `Engine::from_filter_set`
- Kosmeettinen suodatus: `Engine::url_cosmetic_resources` ja `Engine::hidden_class_id_selectors`
- Moottorin välimuisti: `Engine::serialize` ja `Engine::deserialize`
- Lisenssi: MPL-2.0
- Kotisatama-ohje: `AGENT.md` (`components/kotisatama/`, `KOTISATAMA-PATCH`)

API:t ja featuret tarkistetaan uudelleen toteutushetkellä käytettävästä fork-versiosta.
