# Testaus ja WPT

Servo testataan laajasti **Web Platform Tests (WPT)** -joukkoa vasten. Telakka-työssä WPT on tärkeä työkalu yleisten korjausten varmistamiseen.

## Mikä on WPT?

- Standardien mukaisia automaattitestejä selaimille
- Sama testisarja useille moottoreille (Chromium, Firefox, Servo…)
- Repossa: `tests/wpt/tests/` (iso — älä yritä lukea kerralla)

Dokumentaatio: <https://web-platform-tests.org>

## Peruskomennot

```bash
# Esimerkki: aja yksi testitiedosto (polku tarkista reposta)
./mach test-wpt tests/wpt/tests/html/...

# Apua
./mach test-wpt --help
```

Tarkat komennot ja liput: [book.servo.org — Testing](https://book.servo.org/contributing/testing.html).

## Milloin WPT auttaa Katselimessä

| Tilanne | WPT-käyttö |
|---------|------------|
| Epäilet layout-bugia | Etsi `css/` tai `html/` -testejä aiheesta |
| Fetch / CORS -ongelma | `fetch/` -hakemisto |
| Lomake tai input | `html/semantics/forms/` |
| Yleinen korjaus upstreamiin | Uusi tai olemassa oleva testi todistaa korjauksen |

Periaate [docs/KELA-TELAKKA.md](../../docs/KELA-TELAKKA.md): Kela on testikohde, korjaus on **yleinen**.

## Työskentelymalli

1. Toista bugi manuaalisesti (`./mach run`).
2. Arvioi kerros: [komponentit.md](komponentit.md).
3. Etsi vastaava WPT-testi tai kirjoita minimi testi (jos upstream-contribuutio).
4. Korjaa `components/`-koodissa (upstream-sääntöjen mukaan).
5. Aja testi uudelleen.

## Expectations / manifestit

Servo ylläpitää odotuksia siitä, mitkä WPT-testit menevät läpi tai epäonnistuvat. Tiedostot ovat repossa erillisissä manifesteissa — älä muokkaa niitä satunnaisesti; lue ensin Testing-luku book.servo.orgista.

## Kirjaa oppiminen

Kun ajat ensimmäisen WPT-testin, tee merkintä:

[telakka/oppimispäiväkirja/](../telakka/oppimispäiväkirja/)

## Seuraavaksi

- [linkit.md](../linkit.md) — upstream-linkit
- [telakka/miten-debugataan.md](../telakka/miten-debugataan.md)
