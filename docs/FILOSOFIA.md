# Kotisatama-filosofia

*Versio 0.2 — kesäkuu 2026*

## Perusajatus

Kotisatama ei ole vain käyttöliittymä selaimen päällä. Kotisatama on turvallinen arkiympäristö, jossa tärkeät digipalvelut tehdään ymmärrettäviksi, rajatuiksi ja luotettaviksi.

Tavoite ei ole tehdä koko internetistä helppoa. Tavoite on tehdä tärkeistä asioista turvallisia.

## Ei Google-riippuvuuksia

Kotisataman lähtökohta on, ettei ratkaisun ytimeen rakenneta Google-riippuvuutta.

Jos jokin tärkeä sivu ei toimi Servolla, ratkaisu ei ole piilottaa taustalle Chromiumia tai Chromea. Ratkaisu on selvittää, mitä Servosta puuttuu, ja korjata puuttuva osa mahdollisimman yleisesti ja standardien mukaisesti.

Periaate:

> Kela ei toimi → selvitetään puuttuva web-alustan osa → korjataan Servo/Kotisatama-yhteensopivuus.

Ei:

> Kela ei toimi → käytetään Chromea.

## Servo on moottori, Kotisatama on satama

Servo on Kotisataman selainmoottori. Kotisatama ei yritä muuttaa Servoa omaksi tuotteekseen eikä forkata sitä pysyvästi erilleen upstreamista.

Roolit:

- **Servo** on selainmoottori.
- **Kotisatama** on turvallinen käyttöympäristö.
- **Varustamo** on luotettujen sovellusten jakelupaikka.
- **Viranomaisväylä** (ei toteuteta) on tärkeiden viranomaispalvelujen käyttäjälle näkyvä sovellus. Perus whitelistaus nyt kuitenkin ajaa sen yli
- **Telakka** on kehitysmalli, jolla korjataan Servosta puuttuvia osia.

## Whitelist ensin

Kotisatamassa ei tarvitse aloittaa koko avoimesta webistä. Aluksi määritellään tärkeimmät palvelut, joiden täytyy toimia.

Ensimmäinen whitelist voi sisältää esimerkiksi:

- Kela
- Suomi.fi
- OmaKanta
- Vero
- eläke- ja viranomaispalvelut
- pankkitunnistuksen kannalta välttämättömät sivut

Whitelist ei ole kiertotie Servon ohi. Se on testilista sille, mitä Kotisataman pitää osata.

## Kela on ensisijainen testi

Ikäihmiselle Kela ei ole sivuasia. Se voi olla yksi tärkeimmistä digipalveluista.

Siksi Kela.fi ja Kelan asiointi ovat Kotisatamalle erityisen tärkeitä testikohteita. Jos Kela ei toimi, Kotisatama ei ole vielä valmis niille käyttäjille, joita varten sitä rakennetaan.

Ensimmäiset testit:

1. Kela.fi etusivu latautuu.
2. Navigaatio ja haku toimivat.
3. Asiointi.kela.fi pääsee tunnistautumisen alkuun.
4. Tunnistautumisvirta toimii.
5. Lomakkeet, viestit ja PDF:t toimivat.

### Whitelist ei tee sivusta toimivaa

Whitelist ratkaisee turvallisuuden ja fokuksen: käyttäjä pysyy satamassa ja tärkeät palvelut ovat tarkoituksella saatavilla. Se ei korvaa selainmoottorin yhteensopivuutta.

`kela.fi` whitelistissa tarkoittaa, että navigointi sinne on sallittu — ei sitä, että sivu latautuu, JavaScript toimii tai asiointi onnistuu. Suurin osa Kelan työstä tapahtuu **Servossa** (Telakka), ei Kotisataman UI- tai hakukerroksessa.

### Testitasojen realistisuus

Ensimmäiset testit eivät ole saman vaikeisia. Telakka-taktiikalla realistinen eteneminen näyttää tältä:

| Testi | Tavoite | Realistisuus |
|---|---|---|
| 1. Etusivu latautuu | `www.kela.fi` avautuu ja sisältö on luettavissa | Kohtuullinen — ensimmäinen saavutettava taso |
| 2. Navigaatio ja haku | Linkit, valikot ja haku toimivat riittävän luotettavasti | Mahdollinen — iteratiivista Telakka-työtä |
| 3. Asiointi.kela.fi → tunnistautumisen alku | Eri subdomain, istunto, lomakkeet | Vaikea |
| 4. Koko tunnistautumisvirta | FTN-ketju pankkien ja välittäjien kautta | Erittäin vaikea — ks. tunnistautumisketju alla |
| 5. Lomakkeet, viestit, PDF:t | Kirjautunut asiointi arjessa | Erittäin vaikea — riippuu istunnosta, tiedostoista ja PDF-tuesta |

Osittainen onnistuminen on siis odotettavaa ja hyväksyttävää välivaihe: käyttäjä löytää Kelan satamasta ja lukee tietoa ennen kuin täysi asiointi on mahdollista.

### Tunnistautumisketju (FTN)

Kela-asiointi käyttää vahvaa tunnistautumista (FTN). Virta ei pysy `kela.fi`-domainissa vaan kulkee useiden välittäjien kautta — esimerkiksi `asiointi.kela.fi`, `suomi.fi`, pankit ja tunnistuspalvelut.

Suljettu satama ja avoin tunnistautumisketju ovat ristiriidassa, ellei jokin näistä päätetä eksplisiittisesti:

1. **Laajennettu whitelist** — FTN-ketjun domainit whitelistataan tarkoituksella (satama kasvaa, mutta pysyy kuratoituna).
2. **Tilapäinen tunnistuspolku** — auth-vaiheessa sallitaan rajattu poikkeus satamasta (käyttäjälle selkeästi merkitty “tunnistautuminen”).
3. **Erillinen Viranomaisväylä-sovellus** — kuratoitu käyttöliittymä, joka hoitaa auth-polun (ei toteutettu; whitelist korvaa osittain).

Ilman tämän mallin valintaa testi #4 ei käytännössä onnistu, vaikka moottori olisi kunnossa. Auth-ketjun domainit on kirjattava whitelist-suunnitelmaan — pelkkä `kela.fi` ei riitä.

Whitelistissä huomioitava vähintään:

- `kela.fi` ja alidomainit (`www`, `asiointi`, …)
- `suomi.fi` ja tunnistautumiseen liittyvät alidomainit
- pankkitunnistuksen ja mobiilivarmenteen välittäjät (FTN-osapuolet)

## Ei sivukohtaisia selainmoottorihackeja

Kotisatama ei lisää Servo-koodiin sivukohtaisia poikkeuksia kuten:

```text
if url contains kela.fi
```

Jos Kela paljastaa puutteen, korjaus tehdään niin, että se hyödyttää myös muita sivuja.

Hyvä korjaus on:

- pieni
- testattava
- standardin mukainen
- upstreamattavissa Servoon
- poistettavissa Kotisataman patch-sarjasta, kun se hyväksytään upstreamiin

Kaikki Kelan rikkomukset eivät kuitenkaan ratkea yhdellä pienellä korjauksella. Yksi näkymä voi paljastaa useita puutteita (evästeet, layout, fetch, kolmannen osapuolen skripti). Telakka etenee silti samaa polkua — mutta aikajänne voi venyä, eikä jokainen este ole heti upstreamattavissa (`status: local-only` sallittu väliaikaisesti).

## Upstreamia ei rikota

Kotisataman omat osat pidetään erillään Servon koodista.

Suositeltu rakenne:

```text
servo/                 # upstream, mahdollisimman puhdas
kotisatama-runtime/    # UI, profiilit, whitelist, oikeudet
kotisatama-patches/    # väliaikaiset Servo-korjaukset
kotisatama-tests/      # Kela/Suomi.fi/OmaKanta-testit
```

Patchit merkitään selkeästi:

```text
status: upstreamable
status: submitted
status: local-only
status: remove-when-upstreamed
```

Tavoite on, että Kotisatama ja Servo tukevat toisiaan, mutta eivät korvaa toisiaan.

## Viranomaisväylä (Ei toteutukseen) 

Viranomaisväylä on Varustamon sovellus, joka tarjoaa turvallisen pääsyn tärkeisiin viranomaispalveluihin.

Se ei ole yleinen selain. Se on rajattu palveluvalikko.

Esimerkkejä:

- Kela
- Suomi.fi
- OmaKanta
- Vero
- eläkeasiointi
- pankkitunnistus

Käyttäjälle tämä näkyy yksinkertaisesti:

> Avaa Kela turvallisesti.

Teknisesti Viranomaisväylä käyttää Kotisataman whitelistia ja Servon päälle rakennettua turvallista ajotilaa.

## Telakka

Telakka on tapa kehittää Kotisatamaa arjen tarpeista käsin.

Työnkulku:

1. Valitaan tärkeä sivu, joka ei toimi.
2. Toistetaan ongelma Servossa.
3. Kirjataan ensimmäinen konkreettinen hajoamiskohta.
4. Selvitetään puuttuva API, standardiominaisuus tai bugi.
5. Tehdään pienin mahdollinen korjaus.
6. Lisätään testi.
7. Ajetaan Kotisatama-testit.
8. Tarjotaan korjaus upstreamiin.
9. Poistetaan paikallinen patch, kun upstream sisältää korjauksen.

Jos sama sivu paljastaa useita hajoamiskohtia, toistetaan vaiheet 3–7 per kohta. Yksi Telakka-kierros = yksi konkreettinen korjaus, ei koko sivun kerralla ratkaiseminen.

## Kotisatama Ready

Selain tai sovellus on Kotisatama Ready vasta, kun tärkeimmät arjen palvelut toimivat luotettavasti ja ovat whitelistillä satamassa.

**Huom:** Ready-taso on pitkän aikavälin tavoite. Servo ei ole vielä valmis yleiseen verkkoon; Kotisatama etenee sivu kerrallaan. Kela voi olla osittain toimiva (etusivu, tiedonhaku) ennen kuin täysi asiointi ja tunnistautuminen on mahdollista.

### Portaittaiset Kelan milestone’t

| Taso | Kriteeri | Merkitys käyttäjälle |
|---|---|---|
| **Kela MVP** | `www.kela.fi` latautuu, navigaatio ja haku toimivat | Löytää tiedon turvallisesti satamasta |
| **Kela asiointiin** | `asiointi.kela.fi` avautuu, pääsee tunnistautumisen alkuun | Ymmärtää miten asiointi alkaa |
| **Kela kirjautunut** | FTN-tunnistautuminen onnistuu, viestit ja lomakkeet toimivat | Voi hoitaa asiansa |
| **Kela Ready** | PDF:t, liitteet ja pitkät istunnot luotettavasti | Arjen digipalvelu valmis |

Ensimmäinen tavoitetaso (Kotisatama Ready kokonaisuutena):

- Kela toimii.
- Suomi.fi toimii.
- OmaKanta toimii.
- pankkitunnistus toimii.
- yleisimmät PDF:t ja lomakkeet toimivat.
- käyttäjä ei joudu ymmärtämään selainmoottoreita, evästeitä tai teknisiä virheitä.

### Miten “toimii” mitataan

- **Automaattiset smoke-testit** — etusivu latautuu, kriittiset linkit vastaavat (kotisatama-tests).
- **Manuaalinen checklist** — yllä olevat viisi Kelan testiä ja milestone-taulukko.
- **Telakkakirjaus** — jokaisesta hajoamiskohdasta: URL, konsolivirhe, puuttuva API, korjaus, patch-status.

Onnistuminen kirjataan milestone-tasolla, ei binäärisesti “Kela toimii / ei toimi”.

## Ydinlause

> Servo on moottori. Kotisatama on satama. Telakka korjaa moottoria, mutta ei muuta sitä satamaksi.

## Lyhyt periaate

Kaikkea nettiä ei tarvitse tehdä helpoksi.  
Tärkeät asiat pitää tehdä turvallisiksi.
