# Rust tässä repossa

Lyhyt johdanto siihen, miten Rust-organisaatio toimii Katselin/Servo-monorepossa.

## Monorepo ja workspace

Juuren `Cargo.toml` määrittelee **workspace**:n. Jokainen `components/*/Cargo.toml` on erillinen crate, mutta ne käännätään yhdessä.

```
Cargo.toml          ← workspace-juuri
components/
  layout/Cargo.toml ← yksi crate
  script/Cargo.toml ← toinen crate
```

## Tyypillinen crate-rakenne

| Tiedosto | Sisältö |
|----------|---------|
| `lib.rs` | Julkinen API, moduulit |
| `*.rs` | Toteutukset |
| `Cargo.toml` | Riippuvuudet, featuret |

## Featuret ja Kotisatama

Upstream-crateissa voi olla feature `kotisatama` (ks. [AGENT.md](../../AGENT.md)). Oma koodi elää mieluummin `components/kotisatama/`-cratena ilman upstream-muutoksia.

## `./mach` vs. `cargo`

| Komento | Käyttö |
|---------|--------|
| `./mach build` | Koko Servo/Katselin — käytä tätä |
| `cargo build -p layout` | Yksittäinen crate (nopeampi kokeilu) |
| `cargo doc -p script --open` | Dokumentaatio yhdelle crateille |

## Mistä aloittaa lukeminen

1. `ports/servoshell/` — miten sovellus käynnistyy
2. `components/servo/` — moottorin API embedderille
3. Ongelman mukaan: [servo/komponentit.md](../servo/komponentit.md)

## Hyödylliset Rust-käsitteet Servossa

- **`Arc` / `Mutex`** — jaettu tila prosessien/säikeiden välillä
- **`enum` viesteille** — IPC-viestityypit
- **`#[derive(...)]`** — DOM- ja IPC-rakenteet
- **Traitit** — embedder-delegaatit (`WebViewDelegate`)

## Seuraavaksi

- [käytännöt.md](käytännöt.md) — mitä osata ennen syväsukellusta
- [servo/arkkitehtuuri.md](../servo/arkkitehtuuri.md)
