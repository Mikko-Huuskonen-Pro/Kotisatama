# Miten debugataan (Telakka)

Käytännön ohje whitelist-sivun (esim. Kela) debuggaukseen niin, että opit moottoria ja dokumentoit löydökset.

## Yhteys Telakkaan

[Telakka](../../docs/FILOSOFIA.md) tarkoittaa: löydä Servo-puute, korjaa yleisesti, Kela toimii sivuvaikutuksena.

Tarkempi MVP-rajaus: [docs/KELA-TELAKKA.md](../../docs/KELA-TELAKKA.md).

## Työjärjestys

### 1. Toista ongelma

- Käynnistä: `./mach run`
- Kirjaa **tarkka URL**, mitä teit, mitä näit
- Ota talteen konsoli/loki jos mahdollista

### 2. Rajaa kerros

Käytä [sivun-lataus.md](../servo/sivun-lataus.md) -ketjua:

| Oire | Epäillä |
|------|---------|
| Ei lataudu | `net` |
| Tyhjä / virheilmoitus sivulla | `script` |
| Rikkinäinen ulkoasu | `layout` / `paint` |
| Ei pääse sivulle lainkaan | `whitelist` / embedder |

### 3. Erottele Kotisatama vs. Servo

- Jos sama URL toimii upstream-Servossa mutta ei Katselimessä → Kotisatama-patch
- Jos ei toimi kummassakaan → upstream-bugi → Telakka-korjaus

### 4. Etsi testi tai minimiesimerkki

- WPT: [testaus-wpt.md](../servo/testaus-wpt.md)
- Pienin HTML-tiedosto joka näyttää saman bugin (paikallinen testi)

### 5. Korjaa pienin yleinen muutos

**Älä** tee:

```rust
if url.contains("kela.fi") { ... }
```

**Tee** yleinen korjaus komponentissa ja dokumentoi [oppimispäiväkirjaan](oppimispäiväkirja/).

## Kirjausmalli (KELA-TELAKKA)

Jokaisesta hajoamiskohdasta:

| Kenttä | Sisältö |
|--------|---------|
| URL | Tarkka osoite |
| Toisto | Lyhyet askeleet |
| Odotettu | Mitä selaimen pitäisi tehdä |
| Toteutunut | Mitä tapahtui |
| Konsoli/loki | Olennainen virhe |
| Epäilty puute | layout, fetch, evästeet, … |
| Patch-status | `upstreamable`, `local-only`, `submitted`, `remove-when-upstreamed` |

## Kirjaa myös oppiminen

Kun ymmärrät **miksi** bugi syntyi, tee erillinen merkintä [oppimispäiväkirjaan](oppimispäiväkirja/) — oppimisfokus, ei vain bugilista.

## Hyödylliset komennot

```bash
./mach run
./mach build --release
# WPT-esimerkki (polku vaihtelee)
./mach test-wpt --help
```

## Seuraavaksi

- [oppimispäiväkirja/README.md](oppimispäiväkirja/README.md) — malli päiväkirjamerkinnälle
- [oppimispäiväkirja/2026-06-29-kela-etusivu.md](oppimispäiväkirja/2026-06-29-kela-etusivu.md) — esimerkki: Kela-etusivu
- [kotisatama-vs-servo.md](../kotisatama-vs-servo.md) — miten fork eroaa Servosta
- [servo/komponentit.md](../servo/komponentit.md)
