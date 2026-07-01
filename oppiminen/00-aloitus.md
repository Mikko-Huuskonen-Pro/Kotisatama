# Aloitus

Tämä sivu auttaa aloittamaan Servo-moottorin opiskelun Katselin-repossa.

## Mitä sinun pitää tietää etukäteen

- **Rustin perusteet** — omistajuus, `Result`, moduulit, crate-rakenne
- **Webin perusteet** — HTML, CSS, JavaScript, HTTP (ei tarvitse olla asiantuntija)
- **Git** — haarat, commitit, upstream-merge

Jos Rust on uutta, lue ensin [rust/käytännöt.md](rust/käytännöt.md) ja palaa sitten tänne.

## Kehitysympäristö

Seuraa Servon virallista asennusohjetta:

- [Setting up your environment](https://book.servo.org/hacking/setting-up-your-environment.html)

Katselin-spesifiset lisäohjeet: [README.md](../README.md#kehitysympäristö).

### Nopea tarkistus

```bash
./mach build --release
./mach run
```

Jos build onnistuu ja selain avautuu, ympäristö on kunnossa.

## Ensimmäiset askeleet

1. Lue [sanasto.md](sanasto.md) — tutustu termeihin ennen koodin lukemista.
2. Valitse polku tavoitteesi mukaan:
   - **Katselinin oma kerros** (whitelist, haku): [kotisatama/README.md](kotisatama/README.md)
   - **Servo-moottori** (sivun lataus, layout, JS): [servo/sivun-lataus.md](servo/sivun-lataus.md)
3. Selaa [servo/komponentit.md](servo/komponentit.md) tai [kotisatama/cratet.md](kotisatama/cratet.md) — löydä oikea hakemisto.
4. Avaa lähdekoodi rinnalla — aloita esim. `ports/servoshell/kotisatama.rs` tai `components/constellation/`.

## Reporakenne (lyhyt)

```
Katselin/
├── components/          ← Servo-moottorin Rust-cratet
│   ├── kotisatama/      ← Omat muutokset (älä sekoita upstream-opiskeluun)
│   └── [servo-upstream] ← Moottorin ydin
├── ports/servoshell/    ← Embedder (sovelluskuori)
├── oppiminen/           ← Tämä hakemisto
└── docs/                ← Tuotedokumentaatio
```

## Työkalut

| Työkalu | Käyttö |
|---------|--------|
| `./mach build` | Käännä moottori |
| `./mach run` | Käynnistä selain |
| `./mach test-wpt` | Aja web-platform-testejä |
| `cargo doc --open` | Rust-dokumentaatio crateille |

Lisää: [servo/testaus-wpt.md](servo/testaus-wpt.md), [linkit.md](linkit.md).

## Seuraavaksi

- **Katselinin kerros:** [kotisatama/README.md](kotisatama/README.md)
- **Käytännönläheinen (moottori):** [servo/sivun-lataus.md](servo/sivun-lataus.md)
- **Teoreettinen:** [servo/arkkitehtuuri.md](servo/arkkitehtuuri.md)
- **Kela-debuggaus:** [telakka/miten-debugataan.md](telakka/miten-debugataan.md)
