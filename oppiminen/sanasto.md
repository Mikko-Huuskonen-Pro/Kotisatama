# Sanasto

Servon lähdekoodi ja upstream-dokumentaatio ovat englanniksi. Tässä taulukossa on keskeiset termit suomeksi — käytä näitä oppimisdokumenteissa, älä koodissa.

## Yleiset

| Englanti | Suomi | Lyhyt selitys |
|----------|-------|---------------|
| browser engine | selainmoottori | Ohjelmisto, joka tulkitsee HTML/CSS/JS ja piirtää sivun |
| embedder | upotuskerros | Sovellus, joka käyttää Servoa (esim. servoshell) |
| pipeline | putki | Sivun elinkaaren vaiheet peräkkäin (lataus → näyttö) |
| crate | crate | Rust-kirjaston/yksikön paketti (`components/layout/` jne.) |
| upstream | upstream | Alkuperäinen Servo-projekti (servo/servo) |
| fork | fork | Haara alkuperäisestä reposta (Katselin = Servo-fork) |

## Arkkitehtuuri

| Englanti | Suomi | Lyhyt selitys |
|----------|-------|---------------|
| constellation | tähtikuvio | Prosessien ja välilehtien orkestrointi |
| browsing context | selauskonteksti | Yksi "selainikkuna" tai välilehti logiikassa |
| navigation | navigointi | Siirtyminen uuteen URL:iin |
| session history | istuntohistoria | Takaisin/eteen-pino |
| sandbox | hiekkalaatikko | Eristetty suoritusympäristö (turvallisuus) |

## Verkko ja sisältö

| Englanti | Suomi | Lyhyt selitys |
|----------|-------|---------------|
| fetch | haku (verkko) | HTTP-pyyntö resurssin hakemiseen |
| resource | resurssi | Ladattava tiedosto (HTML, CSS, kuva…) |
| cookie | eväste | Pieni tiedosto istunnon tunnistamiseen |
| TLS / HTTPS | salattu yhteys | Turvallinen HTTP |
| CORS | rajatettu jakaminen | Sääntö, milloin sivu saa hakea toisen domainin dataa |

## DOM ja skriptit

| Englanti | Suomi | Lyhyt selitys |
|----------|-------|---------------|
| DOM | dokumenttiobjektimalli | Sivun puurakenne (elementit, attribuutit) |
| script | skripti | JavaScript-moottori ja DOM-API |
| event loop | tapahtumasilmukka | JS-tehtävien ja tapahtumien käsittely |
| binding | sidos | Rust ↔ JavaScript -rajapinta |

## Ulkoasu

| Englanti | Suomi | Lyhyt selitys |
|----------|-------|---------------|
| layout | asettelu | Elementtien sijainnit ja koot (CSS) |
| style | tyyli | CSS-sääntöjen laskenta |
| paint | piirto | Pikselien tuottaminen ruudulle |
| compositor | koostaja | Kerrosten yhdistäminen (jos käytössä) |
| reflow | uudelleenlayout | Asettelun uudelleenlaskenta |

## Testaus

| Englanti | Suomi | Lyhyt selitys |
|----------|-------|---------------|
| WPT | web-platform-testit | Standardien mukaiset automaattitestit |
| reftest | referenssitesti | Visuaalinen vertailu kahden sivun välillä |
| testharness | testikehys | WPT:n JS-testien ajuri |

## Katselin-spesifiset

| Englanti | Suomi | Konteksti |
|----------|-------|-----------|
| whitelist | whitelist / satama | Sallittujen sivujen lista |
| Kotisatama | kotisatama | Turvallinen etusivu ja suljettu tila |
| Telakka | telakka | Kehitysmalli Servo-puutteiden korjaamiseen |
| Avomeri | avomeri | Avoin internet (ei whitelist) |

## Lisää termejä

Kun törmäät uuteen termiin, lisää se tähän tiedostoon lyhyellä selityksellä. Pidä englanninkielinen vastine sarakkeessa, jotta koodin lukeminen pysyy helppona.
