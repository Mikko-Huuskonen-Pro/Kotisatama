# Oppiminen — Servo-moottori

Tämä hakemisto on suomenkielinen oppimateriaali **Servo-selainmoottorin** ymmärtämiseen. Se täydentää tuotedokumentaatiota (`docs/`), joka kuvaa Katselinta ja Kotisatamaa.

> **Servo on moottori, Kotisatama on satama.** Moottorin koodi on englanniksi; tässä kansiossa selitetään käsitteet ja rakenteet suomeksi. Katso [docs/FILOSOFIA.md](../docs/FILOSOFIA.md).

## Mitä tästä löytyy

| Tiedosto | Sisältö |
|----------|---------|
| [00-aloitus.md](00-aloitus.md) | Esitiedot, työkalut, ensimmäiset askeleet |
| [sanasto.md](sanasto.md) | Keskeiset termit suomeksi ja englanniksi |
| [linkit.md](linkit.md) | Kuratoitu lista upstream-lähteistä |
| [kotisatama-vs-servo.md](kotisatama-vs-servo.md) | Miten fork eroaa upstream-Servosta |
| [servo/](servo/) | Moottorin arkkitehtuuri ja komponentit |
| [rust/](rust/) | Rust- ja repokohtaiset käytännöt |
| [telakka/](telakka/) | Debuggaus ja oppimispäiväkirja (Kela-työ) |

## Oppimispolut

Valitse polku lähtötasosi mukaan. Voit yhdistää polkuja vapaasti.

### Polku A — "Mitä tapahtuu kun avaan sivun?"

Käytännönläheinen polku: yksi URL, koko ketju selitettynä.

1. [servo/sivun-lataus.md](servo/sivun-lataus.md)
2. [servo/embedder-ja-ports.md](servo/embedder-ja-ports.md)
3. [telakka/miten-debugataan.md](telakka/miten-debugataan.md)

### Polku B — "Haluan lukea koodia järjestelmällisesti"

Rakenne ensin, sitten yksityiskohdat.

1. [servo/arkkitehtuuri.md](servo/arkkitehtuuri.md)
2. [servo/komponentit.md](servo/komponentit.md)
3. [servo/prosessit-ja-säikeet.md](servo/prosessit-ja-säikeet.md)
4. Valitse yksi komponentti `components/`-hakemistosta ja lue lähdekoodi rinnalla

### Polku C — "Haluan korjata ensimmäisen upstream-bugin"

Telakka-malli: Kela testikohteena, yleinen korjaus tuloksena.

1. [servo/testaus-wpt.md](servo/testaus-wpt.md)
2. [docs/KELA-TELAKKA.md](../docs/KELA-TELAKKA.md)
3. [telakka/miten-debugataan.md](telakka/miten-debugataan.md)
4. Kirjaa havainnot [telakka/oppimispäiväkirja/](telakka/oppimispäiväkirja/)

## Miten dokumentaatiota täydennetään

- **Älä kopioi** [book.servo.org](https://book.servo.org) kokonaan — tee tiivistelmiä ja linkitä alkuperäiseen.
- **Kirjoita oppimispäiväkirjaan** kun debuggaat whitelist-sivua tai ajat WPT-testejä.
- **Pidä koodi englanniksi** — suomi vain tässä hakemistossa (ks. [AGENT.md](../AGENT.md)).
- **Upstream-korjaukset** dokumentoidaan yleisinä, ei sivukohtaisina hackeina.

## Esimerkkimerkinnät

- [telakka/oppimispäiväkirja/2026-06-29-kela-etusivu.md](telakka/oppimispäiväkirja/2026-06-29-kela-etusivu.md) — Kela-etusivu, Kotisatama vs. Servo debuggauksessa

## Liittyvät dokumentit

- [AGENT.md](../AGENT.md) — kehityssäännöt (älä koske upstreamia suoraan)
- [docs/FILOSOFIA.md](../docs/FILOSOFIA.md) — Servo vs. Kotisatama
- [docs/KELA-TELAKKA.md](../docs/KELA-TELAKKA.md) — ensimmäinen Telakka-kierros
- [kotisatama-vs-servo.md](kotisatama-vs-servo.md) — fork-erot yhdellä sivulla
