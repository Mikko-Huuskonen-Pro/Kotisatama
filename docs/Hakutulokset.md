Kotisatama hakutulossivu

Tarkoitus

Kotisataman nykyinen haku avaa suoraan parhaan osuman. Tämä toimii silloin, kun käyttäjän tarkoitus on selvä.

Esimerkiksi:

eläke → kela.fi

Tarvitaan kuitenkin myös graafinen hakutulossivu tilanteisiin, joissa:

- osumia on useita
- paras osuma ei ole varma
- käyttäjä haluaa itse valita
- käyttäjä painaa hakupainiketta eikä enteriä
- käyttäjä hakee yleisellä sanalla, kuten “apteekki”, “parturi”, “vero” tai “koulu”

Hakutulossivu näyttää Sataman sisäisen hakemiston tulokset klassisessa hakukonenäkymässä.

Sijainti

Hakutulossivu toteutetaan julkiseen Katselin/Kotisatama-repoon.

Whitelist- ja yellowlist-data voivat pysyä suljetussa repossa.

Julkiseen repoon kuuluu:

- käyttöliittymä
- hakutuloksen komponentit
- hakutuloksen visuaalinen malli
- tyhjän haun näkymä
- virhenäkymä
- dokumentoitu dataformaatti

Suljettuun repoon kuuluu:

- varsinainen domain-lista
- Meilisearch-indeksi
- kuratoitu data
- mahdolliset sisäiset laatumerkinnät

Perusmalli

Hakutulossivu saa hakusanan selaimen sisäisestä hausta.

Esimerkki:

katselin://haku?q=eläke

tai myöhemmin:

https://katselin.fi/haku?q=eläke

Ensimmäisessä vaiheessa haku toimii selaimen sisällä paikallista Meilisearch-indeksiä vasten.

Käyttötapa

Osoite-/hakukentän toiminta:

Enter
→ avaa paras osuma, jos varmuus on korkea

Hakupainike
→ avaa hakutulossivu

Epävarma haku
→ avaa hakutulossivu

Useita vahvoja osumia
→ avaa hakutulossivu

Myöhemmin asetuksissa voidaan tarjota valinta:

[ ] Avaa paras osuma suoraan
[x] Näytä aina hakutulokset

Hakutuloksen rakenne

Yksi hakutulos sisältää vähintään:

{
  "domain": "kela.fi",
  "label": "Kela",
  "description": "Eläke, tuet, lapsiperheet ja sosiaaliturvan asiointi",
  "category": "government",
  "tags": ["eläke", "tuet", "sosiaaliturva", "asiointi"],
  "type": "white"
}

Näytettävä hakutulos

Hakutuloskortti:

🦊 Kela
kela.fi
Eläke, tuet, lapsiperheet ja sosiaaliturvan asiointi

[white] [viranomainen] [eläke] [tuet]

Keltaisten sivujen tulos:

💊 247Apteekkiin
247apteekkiin.fi
Apteekki ja terveyspalvelut

[yellow] [terveys] [apteekki]

Type-symbolit

Hakutuloksessa pitää näkyä selvästi, minkä tyyppinen tulos on.

white  = Sataman luotettu sivu
yellow = Keltaisten sivujen kaupallinen tai arjen palvelu

Ehdotettu esitystapa:

🦊 Satama
🟡 Keltaiset sivut

Tai tekstinä:

Satamassa
Keltaisissa sivuissa

Kategoriat

Hakutuloksessa voidaan näyttää kategorian symboli.

Esimerkkejä:

emergency     🚨
government    🏛️
municipality  🏘️
health        🏥
education     🎓
library       📚
transport     🚌
banking       🏦
commerce      🛒
services      ✂️
culture       🎭
sports        ⚽
nature        🌲
work          💼
media         📰
housing       🏠
organization  🤝
other         🔎

Hakusivun näkymä

Sivun yläosa:

Kotisatama-haku

[ eläke                                      🔍 ]

Löytyi 6 tulosta

Tulokset:

1. Kela
2. Työeläke.fi
3. Eläketurvakeskus
4. Suomi.fi
5. Verohallinto
6. Keva

Tyhjä hakutulos

Jos tuloksia ei löydy:

Ei tuloksia Satamasta.

Voit:
- tarkistaa kirjoitusasun
- kokeilla yleisempää hakusanaa
- avata haun Avomerellä
- ehdottaa sivua lisättäväksi Satamaan

Tärkeää: käyttäjälle ei saa tulla tunnetta, että selain meni rikki.

Turvallisuus

Hakutulossivu ei saa ohittaa whitelist-logiikkaa.

Vaikka tulos näkyy hakusivulla, avaaminen tarkistetaan edelleen normaalin Satama-säännön kautta.

Hakutulos näyttää vaihtoehdot.
Whitelist päättää, saako sivun avata.

Ensimmäisen version rajaus

Ensimmäiseen versioon riittää:

- hakukenttä
- hakutulosten lista
- label
- domain
- type
- category
- tags
- klikkaus domainiin
- tyhjän haun näkymä

Ei vielä tarvita:

- logoja
- favicon-hakua
- käyttäjän sijaintia
- mainoksia
- ulkoista serveriä
- katselin.fi-hakua
- kirjautumista
- ehdota sivua -lomaketta

Miksi tämä on julkisessa repossa

Hakutulossivu on osa Katselimen käyttöliittymää, ei salainen datalista.

Avoimessa repossa voidaan näyttää:

- miten Satama esittää hakutulokset
- miten valkoiset ja keltaiset sivut erotetaan
- miten käyttäjä valitsee tuloksen
- miten turvallinen hakukokemus toimii

Suljettu data voidaan syöttää samaan näkymään buildissä tai ajonaikaisesti.

Pitkän aikavälin tavoite

Hakutulossivusta voi myöhemmin tulla myös Katselin.fi:n julkinen hakemisto.

Ensimmäinen vaihe:

Katselin-selain → paikallinen Meilisearch → sisäinen hakutulossivu

Myöhempi vaihe:

katselin.fi → staattinen hakemisto tai palvelinhaku

Tärkeintä nyt on rakentaa graafinen ikkuna nykyisen toimivan hakumoottorin päälle.