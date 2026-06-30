# Prosessit ja säikeet

Servo käyttää useita prosesseja ja säikeitä eristämään crashit, parantamaan turvallisuutta ja hyödyntämään rinnakkaisuutta. Tämä sivu on runko — täydennä kun opit lisää.

## Miksi useita prosesseja?

| Syy | Selitys |
|-----|---------|
| Turvallisuus | Epäluotettava sivu ei pääse kaatamaan koko selainta |
| Vakaus | Yhden välilehden kaatuminen ei välttämättä tapa muita |
| Rinnakkaisuus | Layout ja piirto voivat edetä eri säikeillä |

## Tyypillinen jako (yleiskuva)

> Tarkat nimet ja rajat voivat muuttua upstreamissa — tarkista `components/constellation/` ja book.servo.org.

| Osapuoli | Rooli |
|----------|-------|
| Pääprosessi / embedder | Ikkuna, käyttäjän syöte |
| Constellation | Koordinoi muita prosesseja |
| Script-prosessi | DOM ja JavaScript |
| Layout / paint | Asettelu ja piirto (säikeet tai omat prosessit riippuen versiosta) |

## Viestintä prosessien välillä

Prosessit kommunikoivat **viesteillä** (IPC), ei jaetulla muistilla. Kun luet koodia, etsi:

- `ipc`- tai `channel`-tyyppisiä rakenteita
- `ConstellationMsg`, `Pipeline`-tyyppisiä viestejä (`components/constellation/`)

## Mitä tämä tarkoittaa debuggauksessa

- Konsolivirhe voi tulla **eri prosessin** lokista kuin missä luulet olevasi.
- `./mach run` voi tukea lippuja lisälokitukselle — katso [linkit.md](../linkit.md) ja book.servo.org.
- WPT-testit ajavat usein yksinkertaisemmassa konfiguraatiossa kuin täysi desktop-UI.

## Harjoitustehtävä (täydennettävä)

- [ ] Etsi `components/constellation/pipeline.rs` (tai vastaava) ja kirjaa yksi lause: mitä pipeline tekee.
- [ ] Lisää havainto [telakka/oppimispäiväkirja/](../telakka/oppimispäiväkirja/).

## Seuraavaksi

- [arkkitehtuuri.md](arkkitehtuuri.md)
- [book.servo.org — design documentation](https://book.servo.org/design-documentation/)
