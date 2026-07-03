# Rust-käytännöt Servo-opiskelussa

Mitä Rust-taitoja kannattaa olla (tai opetella rinnalla) kun luet Servo-koodia.

## Suositeltu taso ennen syväsukellusta

- Funktiot, structit, enumit, `match`
- `Result` ja `?`-operaattori virheenkäsittelyyn
- Moduulit (`mod`, `use`) ja tiedostorakenne
- Omistajuus: lainaus (`&`), `Box`, perusteet `Arc`:sta
- Traitit ja `impl`

## Mitä opetella tarpeen mukaan

| Aihe | Miksi Servossa |
|------|----------------|
| `Arc<Mutex<...>>` | Jaettu tila säikeissä |
| Kanavat (`std::sync::mpsc`, ipc) | Prosessiviestit |
| Makrot (`macro_rules!`, derive) | DOM-makrot, serialisointi |
| `unsafe` | Matalan tason optimoinnit — harvemmin ensimmäinen lukukohde |

## Lukemisen järjestys

1. Älä lue koko `components/script/` kerralla.
2. Aloita yhdestä polusta: esim. navigointi `constellation` → `net`.
3. Käytä `cargo doc --open` tai IDE:n "go to definition".
4. Kirjoita ylös termit [sanasto.md](../sanasto.md):hen.

## Työkalut

- **rust-analyzer** (VS Code / Cursor) — hyppää määritelmiin
- **`rg` / grep** — etsi tyyppinimiä ja viestejä
- **`./mach build`** — varmista että ymmärrys vastaa käännöstä

## Yleisiä virheitä aloittelijalla

| Virhe | Parempi tapa |
|-------|--------------|
| Muokata upstream-tiedostoa suoraan | Oma crate tai embedder-hook ([AGENT.md](../../AGENT.md)) |
| Kääntää termejä koodiin | Suomi vain `oppiminen/`-dokumenteissa |
| Lukea ilman tavoitetta | Valitse yksi bugi tai yksi URL poluksi |

## Resurssit

- [Rust-ohjelmointikieli](https://mikko-huuskonen-pro.github.io/Kirja/) (suomennos)
- [The Rust Book](https://doc.rust-lang.org/book/) (englanti)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

## Seuraavaksi

- [repossa.md](repossa.md) — crate-rakenne
- [00-aloitus.md](../00-aloitus.md) — ympäristön pystytys
