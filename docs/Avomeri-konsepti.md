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

Käyttöliittymä

Kotisataman pääkäyttöliittymä ei perustu URL-osoitteisiin.

Käyttäjä käyttää:

- Hakua
- Sataman sivuja
- Varustamon sovelluksia

URL on tekninen yksityiskohta.

Kotisatama kysyy:

«"Mitä haluat tehdä?"»

eikä

«"Minkä osoitteen haluat avata?"»

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

Ei URL-vartijaa

Kotisatama ei seuraa tai tarkista jokaisen URL:n whitelist-statusta.

Sivustojen hallinta tapahtuu Sataman ja Avomeren tasolla.

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
