# Katselin — adblock-rust-integraatiosuunnitelma

**Tila:** hyväksytty toteutettavaksi  
**Ensimmäinen kohde:** nykyinen Katselin (Windows + Android)  
**Myöhempi kohde:** Kotisatama OS  
**Päivitetty:** 17.7.2026

> **Tavoite ei ole lisätä selaimeen yhtä ominaisuutta lisää.**  
> Tavoite on tehdä verkosta rauhallisempi, nopeampi ja turvallisempi oletuksena.

---

## 1. Päätös

Katselimeen integroidaan Braven avoimen lähdekoodin **adblock-rust**-moottori. https://github.com/Mikko-Huuskonen-Pro/adblock-Katselin

Ensimmäinen julkaistava versio keskittyy verkkopyyntöjen estämiseen. Kosmeettinen suodatus, resurssien korvaukset ja tarkempi käyttöliittymä lisätään vasta vakaan verkkosuodatuksen jälkeen.

Integraatio tehdään omana Katselin-moduulina niin, että:

- Servo ei tunne käyttöliittymän asetuksia
- käyttöliittymä ei tunne suodatinmoottorin yksityiskohtia
- Android, Windows ja myöhemmin Linux käyttävät samaa suodatusydintä
- adblock-rust voidaan päivittää ilman laajoja muutoksia Katselimeen
- suodatus voidaan myöhemmin korvata toisella toteutuksella muuttamatta selaimen muita osia

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

1. adblock-rust-kirjaston lisäämisen Rust-riippuvuudeksi
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

```text
Katselin UI
   │
   │ asetukset, sivustokohtainen poikkeus, laskuri
   ▼
PrivacyController
   │
   ▼
ContentBlockingService
   ├── FilterListStore
   ├── SiteExceptionStore
   ├── BlockingStatistics
   └── adblock-rust Engine
             │
             ▼
Servo-verkkopyynnön tarkistuspiste
             │
       ┌─────┴─────┐
       │           │
     salli        estä
       │           │
       ▼           ▼
   verkkoon     paikallinen
                estovastaus
```

### Moduulien vastuut

#### `ContentBlockingService`

Katselimen oma julkinen rajapinta suodatukselle.

Vastaa:

- moottorin alustamisesta
- pyyntöjen tarkistamisesta
- moottorin turvallisesta vaihtamisesta listapäivityksen yhteydessä
- sivustokohtaisten poikkeusten huomioimisesta
- tilastojen välittämisestä

Muu Katselin ei käytä adblock-rustin tyyppejä suoraan.

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

---

## 5. Katselimen sisäinen rajapinta

Suositeltu oma abstraktio:

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

- Servo-integraatio pysyy riippumattomana adblock-rustin API-muutoksista
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
adblock-rust tarkistaa pyynnön
        ↓
Allow → normaali lataus
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

---

## 7. Resurssityyppien muunnos

Servon resurssityyppi muunnetaan adblock-rustin tuntemaan tyyppiin yhdessä paikassa.

Esimerkkitaulukko:

| Servo/Katselin | adblock-rust | Huomio |
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

adblock-rustin `Engine`-tyypin säieominaisuudet voivat riippua crate-versiosta ja feature-valinnoista. Toteutuksessa ei oleteta sokkona, että sama moottori voidaan jakaa kaikille säikeille.

Ennen integraatiota tarkistetaan käytetyn version:

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

Suojaus on oletuksena päällä.

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

Sanastossa vältetään teknisiä termejä kuten “filter engine”, “third-party request” ja “cosmetic rule”.

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

Ensimmäisen version ei pidä vaatia Androidin VPN- tai saavutettavuuspalvelua. Suodatus tapahtuu Katselimen oman Servo-liikenteen sisällä.

---

## 12. Kosmeettinen suodatus — vaihe 2

Verkkosuodatus estää resurssin lataamisen, mutta sivulle voi jäädä tyhjiä mainosalueita. Kosmeettinen suodatus ratkaisee tämän.

adblock-rust tarjoaa URL-kohtaisia kosmeettisia resursseja sekä dynaamisten luokkien ja tunnisteiden perusteella tuotettavia piilotussääntöjä.

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

adblock-rust on MPL-2.0-lisensoitu. Katselimen on säilytettävä kirjaston lisenssi- ja tekijänoikeustiedot jakelussa.

Tehtävät:

- lisää adblock-rust kolmansien osapuolten lisenssiluetteloon
- toimita MPL-2.0-lisenssiteksti jakelun mukana
- dokumentoi käytetty crate-versio
- julkaise adblock-rustin MPL-tiedostoihin tehdyt muutokset MPL-2.0:n mukaisesti
- pidä Katselimen oma integraatiokoodi erillään upstream-kirjaston lähdetiedostoista
- tarkista jokaisen mukana toimitettavan suodatinlistan oma lisenssi erikseen

Suositus: älä forkkaa adblock-rustia ensimmäisessä toteutuksessa. Käytä virallista cratea ja tee tarvittavat sovitukset Katselimen omassa adapterikerroksessa.

Tämä dokumentti ei korvaa oikeudellista neuvontaa.

---

## 15. Virhetilanteet ja turvalliset oletukset

| Tilanne | Toiminta |
|---|---|
| Paketoitu lista puuttuu | käynnistä selain ilman suodatusta ja näytä kehittäjäloki |
| Päivitetty lista vioittunut | käytä edellistä toimivaa listaa |
| Moottorin alustus epäonnistuu | selain toimii, suojaustila ilmoittaa virheestä |
| Pyynnön URL ei jäsenny | salli pyyntö ja kirjaa kehittäjätilassa |
| Resurssityyppi tuntematon | käsittele tyyppinä `other` |
| Poikkeustietokanta vioittunut | älä kaada selainta; palauta tyhjä poikkeuslista |
| Suodatuspalvelu ei vastaa | valitse dokumentoitu fail-open-käytäntö |

Ensimmäisessä versiossa käytetään **fail-open**-periaatetta teknisissä virheissä: selain ei lakkaa toimimasta suodatinmoottorin virheen vuoksi. Käyttäjälle ei kuitenkaan väitetä suojauksen olevan päällä, jos se ei ole.

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

**Valmis, kun:** yksi dokumentoitu kytkentäpiste on valittu.

### Vaihe 1 — erillinen suodatusmoduuli

- lisää riippuvuus lukitulla versiolla
- luo `ContentBlockingService`
- luo oma `BlockingRequest` ja `BlockingDecision`
- lataa pieni testisääntöjoukko
- kirjoita yksikkötestit

**Valmis, kun:** Rust-testi estää tunnetun testipyynnön ilman Servoa.

### Vaihe 2 — Servo-verkkosuodatus

- muodosta pyyntöadapteri
- tarkista pyyntö ennen lähettämistä
- toteuta turvallinen estovastaus
- varmista, ettei estetty pyyntö lähde verkkoon
- lisää kehittäjäloki

**Valmis, kun:** paikallinen integraatiotesti todistaa eston.

### Vaihe 3 — tuotantolista ja välimuisti

- valitse lisenssiltään sopiva oletuslista
- paketoi lista sovellukseen
- rakenna Engine käynnistyksessä
- serialisoi tai välimuistita moottori, jos mittaukset osoittavat tarpeen
- varmista palautuminen versiomuunnoksissa

**Valmis, kun:** suojaus toimii ensimmäisellä käynnistyksellä ilman verkkoyhteyttä.

### Vaihe 4 — käyttöliittymä

- suojaus oletuksena päälle
- estettyjen määrä nykyisellä sivulla
- “Salli tällä sivustolla”
- poikkeuksen poistaminen
- selkeä virhetila

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

- [ ] adblock-rust on eristetty Katselimen adapterikerroksen taakse
- [ ] paketoitu suodatinlista toimii ilman verkkoyhteyttä
- [ ] estettyä pyyntöä ei lähetetä verkkoon
- [ ] sallittu ensimmäisen osapuolen resurssi latautuu normaalisti
- [ ] päädokumenttien virhe-estot on minimoitu
- [ ] suojaus toimii Windowsissa
- [ ] suojaus toimii Androidissa
- [ ] sivustokohtainen sallinta toimii
- [ ] suodatusvirhe ei kaada selainta
- [ ] käyttäjälle ei näytetä väärää suojaustilaa
- [ ] lisenssitiedot toimitetaan jakelun mukana
- [ ] suodatinlistan lisenssi on tarkistettu
- [ ] suorituskykymittaukset on kirjattu
- [ ] suomalaiset kriittiset palveluluokat on savutestattu

---

## 19. Ehdotettu hakemistorakenne

Sovita nimet nykyiseen repoon; rakenne kuvaa vastuunjakoa.

```text
katselin/
├── content-blocking/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── service.rs
│   │   ├── request.rs
│   │   ├── decision.rs
│   │   ├── adblock_adapter.rs
│   │   ├── filter_store.rs
│   │   ├── exceptions.rs
│   │   └── statistics.rs
│   ├── assets/
│   │   ├── filters.txt
│   │   └── FILTER-LICENSES.md
│   └── tests/
│       ├── network_rules.rs
│       └── exception_rules.rs
├── servo-integration/
│   └── content_blocking_hook.rs
└── third_party/
    └── LICENSES.md
```

Pidä upstream-koodi riippuvuutena. Älä kopioi adblock-rustin lähdetiedostoja Katselimen omiin moduuleihin.

---

## 20. Ensimmäinen tehtävä Cursorille tai Codexille

```text
Audit Katselin's current Servo network loading path for an adblock-rust
integration. Do not implement blocking yet.

Deliver:
1. The exact code path used for top-level and subresource HTTP(S) requests.
2. The earliest safe interception point before a request is sent.
3. Which request context is available there: target URL, source/top-level URL,
   resource type, method, headers and tab/webview identity.
4. How a request can be cancelled without crashing or hanging the load.
5. Differences between Windows and Android paths.
6. A proposed minimal RequestBlocker trait and adapter boundary.
7. Tests that can prove a blocked request never reaches a local test server.

Constraints:
- Do not copy code from Brave or other browsers.
- Do not modify Servo upstream code unless no public embedding hook exists.
- Prefer a small Katselin-owned adapter layer.
- Do not add UI, list downloads or cosmetic filtering in this task.
- Document uncertainties instead of guessing APIs.
```

Auditin jälkeen varsinainen integraatio voidaan jakaa pieniin, tarkistettaviin muutoksiin.

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
- tarvittaessa pieni upstream-ystävällinen hook
- vältä laajaa Servo-forkkia

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

---

## 22. Ydinpäätös

Katselin ei rakenna omaa mainostenestomoottoria.

Katselin käyttää adblock-rustia infrastruktuurina ja rakentaa itse sen ympärille:

- selkeät oletukset
- Servo-integraation
- Android-kokemuksen
- sivustokohtaisen hallinnan
- suomalaisiin palveluihin sopivan laadunvarmistuksen

> **adblock-rust tekee suodatuksen. Katselin tekee verkosta rauhallisen.**

---

## 23. Tekniset lähteet tarkistusta varten

- Brave Software: `brave/adblock-rust`
- Rust crate: `adblock`
- Päärajapinta: `adblock::Engine`
- Verkkotarkistus: `Engine::check_network_request`
- Listojen kokoaminen: `FilterSet` ja `Engine::from_filter_set`
- Kosmeettinen suodatus: `Engine::url_cosmetic_resources` ja `Engine::hidden_class_id_selectors`
- Moottorin välimuisti: `Engine::serialize` ja `Engine::deserialize`
- Lisenssi: MPL-2.0

API:t ja featuret tarkistetaan uudelleen toteutushetkellä käytettävästä crate-versiosta.
