# Kotisatamaan osallistuminen

Kotisatama on Servo-fork. Ennen kuin teet muutoksia, lue [`AGENT.md`](./AGENT.md) — siellä on projektin tärkeimmät säännöt.

---

## Missä voi auttaa

Tällä hetkellä hyödyllisintä:

- **Whitelist-täydennykset** — puuttuvia domaineja Hopeakettu- tai Lapsi-profiiliin
- **Crawler-testaus** — Playwright-indeksoinnin toimivuuden varmistaminen eri sivustoilla
- **Bugiilmoitukset** — erityisesti whitelist-ohitukset tai hakuindeksin ongelmat

---

## Whitelist-muutokset (helpoin tapa osallistua)

Whitelist-tiedostot ovat `config/`-hakemistossa:

```
config/
├── whitelist-hopeakettu.json
├── whitelist-lapsi.json
└── whitelist-hyvinvointialueet.json
```

Uuden domainin lisääminen:

```json
{
  "domain": "esimerkki.fi",
  "label": "Esimerkki — lyhyt kuvaus",
  "tags": ["kategoria1", "kategoria2"]
}
```

Tee PR jossa kerrot miksi domain kuuluu kyseiseen profiiliin. Ylläpitäjä hyväksyy tai hylkää.

---

## Rust-muutokset

Kaikki Kotisatama-spesifinen Rust-koodi kuuluu `components/kotisatama/`-hakemistoon. **Älä muokkaa Servo-upstream-tiedostoja** ilman pakottavaa syytä — katso `AGENT.md`.

Tarkistuslista ennen PR:ää:

```bash
cargo build                          # Pitää toimia ilman featurea
cargo build --features kotisatama    # Pitää toimia featurella
cd tauri && cargo build              # Tauri buildaa itsenäisesti
```

---

## Bugiilmoitukset

Avaa GitHub Issue. Kerro:

1. Mitä teit
2. Mitä odotit tapahtuvan
3. Mitä tapahtui
4. Käyttöjärjestelmä ja versio

Tietoturva-aukoista: katso [SECURITY.md](./SECURITY.md) — **ei julkisena issuena**.

---

## Servo-upstream

Kotisatama on Servo-fork. Servo-projektin omiin muutoksiin osallistuminen tapahtuu [servo/servo-repon kautta](https://github.com/servo/servo/blob/main/CONTRIBUTING.md).
