# Katselin Android – Testausraportti
**Päivämäärä:** 1.8.2026  
**Versio:** Android (testiversio)

---

# Yhteenveto

| Taso | Määrä |
|------|------:|
| ✅ OK | 4 |
| 🟡 Keskitaso | 5 |
| 🔴 Kriittinen | 2 |
| 💡 Kehitys | 5 |

---

# Testit

## 1. Huijausviestin blokkaus
**Tila:** ✅ OK (10/10)

### Testi
```
Suomi.fi: SinuIIa on uusi viesti koskien terveydenhuollon ajanvarausta.
Tarkista viesti osoitteessa:
https://www.suomi.viestipalvelut.info
```

### Tulos

- Sivu ei avaudu whitelistin kautta.
- Selain ei vie käyttäjää huijaussivulle.
- Toimii odotetusti.

**Arvosana:** 10/10

---

## 2. Linkkien blokkaus

**Tila:** ✅ OK

Testisivu:

https://www.timesofisrael.com/

### Tulos

Linkkien blokkaus toimii odotetulla tavalla.

---

## 3. Google kiertää Sataman

**Tila:** 🟡 Keskitaso

### Havainto

Googlen hakutulosten kautta pääsee lähes kaikille sivuille, jolloin Sataman rajoitukset voidaan kiertää.

Mielenkiintoisesti ChatGPT-linkeissä esto toimii eri tavalla.Se selvästi käyttää selaimen oman osoitepalkin kautta, taas kun google ei toimi samoin. 

### Ehdotus

Väliaikaisena ratkaisuna poistetaan Google whitelistiltä.

---

## 4. Emojit hakutuloksissa

**Tila:** 🟡 Keskitaso

### Havainto

Hakutuloksissa olevat kuvakkeet/emojit eivät näy oikein.

(Kuvankaappaus liitteenä.)

---

## 5. Oikotie.fi

**Tila:** ✅ OK

### Tulos

- Oikotie toimii.
- "Avaa sovelluksessa" toimii hyvin.
- Youtube ei pelaa, mutta avaa sovelluksessa pelastaa tilanteen. sama Yt music kanssa

---

## 6. Satama / Telakka

**Tila:** 🟡 Keskitaso

### Havainto 1

Lisää Satamaan ei toimi järkevästi.

Vaikka sivun lisää Satamaan, sitä ei löydä hausta.

### Ehdotus

Telakka vaatii jatkokehitystä.

Mahdollinen ratkaisu:

- Satama toimii suosikkilistana.
- Meilisearch indeksoi myös käyttäjän omat kohteet.
- Käyttäjä voi lisätä omia hakukohteita.

### Havainto 2

Kun Satama-portin avaa, selain menee Qwantiin eikä varsinaiselle sivulle.

Tarvitsee lisäselvitystä.

---

## 7. Sovelluksen nimi

**Tila:** 🔴 Kriittinen

Sovelluksen nimi näkyy edelleen:

**Kotisatama**

Pitäisi vaihtaa muotoon:

**Katselin**

---

## 8. Cover Your Tracks

**Tila:** 🔴 Kriittinen

EFF:n Cover Your Tracks -testi epäonnistuu.

Nykyinen tulos:

```
Blocking tracking ads?
No

Blocking invisible trackers?
No
```

### Ratkaisu

Android-versioon tulee integroida:

https://github.com/Mikko-Huuskonen-Pro/adblock-Katselin

Lokikirja alapalkissa, sen tilalle laskuri kuinka paljon seuraajia estetty. Kun sitä klikkaa saa lisätietoa estoista. Lokikirja voi siirtyä yläpalkkiin kolmen pisteen taakse, eli sieltä voi valita lokirjan. 
Windows-versiossa seuraimen estot pelaa jo. 

**Huomio**

Tämä liittyy suoraan Katselimen tuotelupaukseen, joten korjaus on kriittinen.

---

## 9. Yle Areena

**Tila:** ✅ OK

"Avaa sovellus" ei avaa Yle Areenaa.

Ei tarvitse korjata.

---

## 10. Sovelluskuvake

**Tila:** 🟡 Keskitaso

### Havainto

Nykyinen kuvake näyttää keskeneräiseltä.

### Ehdotus

Taustan tulisi olla kokonaan musta.

(Liite: App kuvake.png)

---

## 11. Hakuwidget

**Tila:** 💡 Kehitys

Lisätään Androidin työpöydälle widget, joka avaa suoraan Katselimen haun.

Parantaa päivittäistä käyttöä.

---

## 12. Whitelist-lisäykset

**Tila:** 💡 Kehitys

Lisätään whitelistille:

- https://mikko-huuskonen-pro.github.io/Servo-kirja/
- https://mikko-huuskonen-pro.github.io/Kirja/
- https://kirjapino.fi
- Finlandia Kirja

---

## 13. Välilehtien hallinta

**Tila:** 💡 Kehitys

Androidilla sopiva maksimi voisi olla:

**20 välilehteä**

Kun avataan 21. välilehti:

- vanhin suljetaan automaattisesti.

Tämä todennäköisesti parantaa myös välilehtien muistamista.

Suunnitelma:

Etsitään avoimen lähdekoodin ratkaisu. Rust toteutus, suunnittele

---

## 14. Evästeiden hyväksyntä

**Tila:** 💡 Kehitys

Lisätään automatiikka:

- hyväksyy vain pakolliset evästeet
- vähentää käyttäjän klikkailua

Tähän etsitään avoimen lähdekoodin ratkaisu. Ehdotus: Consent-O-Matic ja sille asetuksiin valinta vaihtoehdot. Oletuksena "Hyväksy pakolliset" 

---

## 15. Whitelist

**Tila:** 💡 Kehitys

Lisätään uusia whitelist-kohteita tarpeen mukaan.

---

## 16. Katselin.fi

**Tila:** 💡 Kehitys

Sivustolle tehtävät muutokset:

- uusi logo käyttöön
- poistetaan Varustamo-maininnat
- lisätään seurannanesto näkyvästi esille
- lisätään Android-sideloading-ohjeet
- huomioidaan, ettei sovellus ole vielä Play Kaupassa

---

# Prioriteetit

## 🔴 Kriittinen

- Sovelluksen nimi → Katselin
- Adblock-Katselin Androidiin (Cover Your Tracks)

## 🟡 Keskitaso

- Google whitelist
- Satama / Telakka
- Emojit hakutuloksiin
- App-kuvake
- Satama-portin avautuminen

## 💡 Kehitys

- Hakuwidget
- Whitelist-laajennukset
- 20 välilehden hallinta
- Automaattinen pakollisten evästeiden hyväksyntä
- Katselin.fi:n päivitykset

---

# Kokonaisarvio

Selvitä meilisearch ei onnistu android versiossa, mitä kimi k3 tarkoitti sillä? 

Perustoiminnot toimivat hyvin ja tärkein turvallisuusominaisuus (huijausviestien blokkaus) toimii erinomaisesti.

Suurimmat puutteet liittyvät Android-version viimeistelyyn, Sataman toimintaan sekä seurannaneston integrointiin. Näistä erityisesti **Adblock-Katselinin integrointi Androidiin** on kriittinen, koska se liittyy suoraan Katselimen keskeiseen tuotelupaukseen. Myöskin meilisearch pitää saada toimimaan androidilla
