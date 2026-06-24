Avomeri-konsepti

Kotisatama-projekti
Versio 1.0 – kesäkuu 2026

---

Visio

«"Satamassa asutaan. Avomerellä käydään."»

Kotisataman ensisijainen tehtävä on tarjota turvallinen, ennustettava ja luotettava tapa käyttää verkkoa.

Kaikkea internetiä ei tarvitse tuoda satamaan.

Kun käyttäjä haluaa poistua kuratoidusta ympäristöstä tutkimaan muuta verkkoa, hän voi lähteä Avomerelle.

Avomeri ei ole toinen selain.

Avomeri on Kotisataman erityinen selaustila.

---

Perusajatus

Kotisatamassa on kaksi maailmaa:

Satama

Luotettu ympäristö.

Ominaisuudet:

- Kuratoidut sivustot
- Varustamon sovellukset
- Pysyvät kirjautumiset
- Evästeet sallittu
- Käyttäjäprofiili säilyy
- Meilisearch-pohjainen haku

Satama on Kotisataman oletustila.

---

Avomeri

Tutkimusmatka avoimeen internetiin.

Ominaisuudet:

- Ei pysyvää profiilia
- Ei historiaa
- Ei tallennettuja evästeitä
- Ei pysyvää localStoragea
- Ei pysyvää IndexedDB:tä
- Ei automaattisia kirjautumisia
- Oikeudet estetty oletuksena

Kun Avomeri suljetaan, kaikki Avomeren data poistetaan.


---

Avomerelle lähteminen

Kotisatamassa näkyy toiminto:

«Lähde Avomerelle»

Toiminto avaa erillisen muistittoman selausistunnon.

Kun käyttäjä poistuu Avomereltä:

- evästeet poistetaan
- välimuisti poistetaan
- localStorage poistetaan
- IndexedDB poistetaan
- käyttöoikeudet nollataan
- historia poistetaan

Avomeri ei jätä jälkiä.

---
Laituri

Laituri on käyttäjän oma alue Sataman ja Avomeren välissä.

Se on tarkoitettu sivustoille, joita käyttäjä käyttää säännöllisesti, mutta joita ei vielä ole hyväksytty osaksi Satamaa.

Laituri toimii kokeilu- ja siirtymäalueena.

---

Tarkoitus

Kaikkia hyödyllisiä sivustoja ei voida ennakolta lisätä Satamaan.

Käyttäjä voi lisätä omia sivustojaan Laiturille ja käyttää niitä ilman, että koko avoin internet muuttuu osaksi Satamaa.

Esimerkkejä:

- Harrastussivustot
- Paikalliset yhdistykset
- Pankki tai palvelu, jota Kotisatama ei vielä tue
- Käyttäjän omat verkkopalvelut

---

Ominaisuudet

Laiturin sivustot ovat käyttäjän hyväksymiä.

Niille voidaan sallia:

- Evästeet
- Kirjautumiset
- Pysyvät asetukset

Mutta ne eivät ole osa Kotisataman kuratoitua ydinsisältöä.

---

Sivuston lisääminen

Käyttäjä voi valita:

«Lisää Laiturille»

Tämän jälkeen sivusto ilmestyy Kotisataman hakuihin ja pikavalintoihin.

Sivustoa voidaan käyttää lähes samalla tavalla kuin Sataman sivustoja.

---

Suhde Satamaan

Satama sisältää Ilion hyväksymät ja ylläpitämät kohteet.

Laituri sisältää käyttäjän itse hyväksymät kohteet.

Satama = yhteisön luottamus
Laituri = käyttäjän luottamus

---

Suhde Avomereen

Avomeri on väliaikainen tutkimusmatka.

Laituri on paikka, johon tutkimusmatkalta löydetty hyödyllinen kohde voidaan kiinnittää myöhempää käyttöä varten.

Avomeri → Laituri → Satama

Kaikkien sivustojen ei tarvitse koskaan päästä Satamaan.

Monille käyttäjille Laituri on lopullinen ja riittävä paikka.

---

Filosofia

Satama on koti.

Laituri on oma vene.

Avomeri on maailma kotisataman ulkopuolella.

---

Ei URL-vartijaa

Kotisatama ei seuraa tai tarkista jokaisen URL:n whitelist-statusta.

Sivustojen hallinta tapahtuu Sataman ja Avomeren tasolla. Jos olet satamassa, meilisearch löytyy valkoisista sivuista oikean palvelun. Avomeren hakuja ei sotketa kotisatamaan ja whitelistiin, vaan pelaa normaalin selaimen tavoin

Arkkitehtuuri:

Kotisatama
├─ Satama
└─ Avomeri

Käyttäjä valitsee ympäristön.

Ympäristö määrittää säännöt.

---

Avomeri eri käyttäjäryhmille

Junior

Avomeri = pois käytöstä

Junior näkee vain Sataman.

---

Senior

Avomeri = pois käytöstä oletuksena

Avomeri voidaan ottaa käyttöön asetuksista tai ylläpitäjän toimesta.

---

Normaali

Avomeri = käytössä

Käyttäjä voi siirtyä Avomerelle milloin tahansa.

---

Hakukone

Avomeren oletushakukone on Qwant.

Perustelut:

- Eurooppalainen palvelu
- Yksityisyyttä kunnioittava
- Ei Googlen käyttöliittymä
- Sopii Kotisataman filosofiaan

Vaihtoehtoiset hakukoneet:

- Startpage
- DuckDuckGo

Oletus:

Qwant

---

Filosofia

Kotisatama ei pyri estämään internetiä.

Kotisatama pyrkii tekemään internetin käytöstä ymmärrettävää.

Satama on koti.

Avomeri on tutkimusmatka.

Käyttäjä tietää aina kummassa maailmassa hän on.
