# Turvallisuus ja profiilit

*Päivitetty: kesäkuu 2026*

Tämä dokumentti kuvaa Kotisataman whitelist-pohjaisen turvallisuusmallin, profiilimatriisin ja v1-toteutusprioriteetit.

---

## Threat model (lyhyesti)

Whitelist on **navigointisuodatin** Satama-tilassa — ei sandbox eikä sisällönsuodatin. Se estää suoran navigoinnin kuratoimattomille domaineille, mutta ei rajoita whitelisted sivun aliresursseja (skriptit, upotukset, fetch).

| Uhkamalli | Whitelist auttaa? |
|-----------|-------------------|
| Satunnainen netin selaaminen | Kyllä (Satama) |
| Phishing-linkki Satamassa | Osittain |
| Haitallinen sisältö sallitulla sivulla | Ei |
| CDN/indeksin manipulointi | Vain jos integriteettitarkistus päällä |
| Määrätietoinen ohitus (devtools, env) | Ei |

---

## Profiilimatriisi

Kaksi erillistä käsitettä:

| Käsite | Ympäristömuuttuja | Tehtävä |
|--------|-------------------|---------|
| **Tuoteprofiili** | `KOTISATAMA_PRODUCT_PROFILE` | Käyttäytymissäännöt (Avomeri, käyttäjän lisäykset) |
| **Whitelist-profiili** | Johdetaan tuoteprofiilista | Mitkä `domains[]`-merkinnät ovat aktiivisia (`tags`) |

| Tuoteprofiili | Whitelist-tag | Avomeri | Käyttäjän domain-lisäys | Kohderyhmä |
|---------------|---------------|---------|-------------------------|------------|
| `normaali` (oletus) | kaikki (`free`) | ✅ | ✅ | Aikuinen v1 |
| `hopeakettu` | `hopeakettu` | ✅ | ✅ | Hopeakettu-tilaus |
| `lapsi` | `lapsi` | ❌ | ❌ (omaisen kautta myöhemmin) | Junior |
| `seniori` | `seniori` | ❌ oletuksena (`KOTISATAMA_SENIORI_AVOMERI=1` → ✅) | ❌ (omaisen kautta myöhemmin) | Senior |

Toteutus: `components/kotisatama/whitelist/src/product_profile.rs`

---

## v1-toteutus: aikuinen (`normaali`)

### Toteutetaan ensimmäiseen julkaisuun

| # | Kohde | Tila | Kuvaus |
|---|-------|------|--------|
| 1 | CDN-manifest + SHA-256 | ✅ | `free/manifest.json` + tarkistus ennen cache-päivitystä |
| 1b | Ed25519-allekirjoitus | ✅ | Manifest allekirjoitetaan CI:ssä; julkinen avain `config/cdn-signing-public.hex` |
| 2 | Fail-safe fallback | ✅ | Bundlattu `config/whitelist.json`, ei `init_empty` |
| 3 | Profiili-infra | ✅ | `ProductProfile` + portit servoshellissa |
| 4 | FTN-domainit (data) | 🔄 jatkuva | `suomi.fi` + auth-ketju whitelist-dataan |

### Ei aikuisen v1:een (lapsi/seniori)

| Kohde | Syy |
|-------|-----|
| `load_web_resource`-suodatus | Rikkoo whitelisted sivujen toiminnan |
| `type: yellow` navigointiesto | UI-metatieto, ei turvaraja |
| Avomerin lukitus | Normaali-profiilissa Avomeri on tuoteominaisuus |
| Sisällönsuodatus | Erillinen lapsiprofiilin työ |

---

## CDN-integriteetti

Julkaisupaketti (`output/cdn/free/`):

```
free/
  manifest.json     ← SHA-256-tiivisteet (crawler generoi)
  whitelist.json
  index.dump
```

`manifest.json`-esimerkki:

```json
{
  "version": "1",
  "updated": "2026-06-30",
  "files": {
    "whitelist.json": { "sha256": "…" },
    "index.dump": { "sha256": "…" }
  },
  "signature": "ed25519-hex…"
}
```

### Allekirjoitus

1. Crawler muodostaa kanonisen JSON-payloadin (`version`, `updated`, `files` — ei `signature`-kenttää)
2. CI allekirjoittaa 32-tavuisella Ed25519-avaimella (`KOTISATAMA_CDN_SIGNING_KEY_HEX`)
3. Asiakas tarkistaa allekirjoituksen upotetulla tai `KOTISATAMA_CDN_PUBLIC_KEY`-julkisella avaimella

Kehitys: `KOTISATAMA_CDN_SKIP_INTEGRITY=1` ohittaa kaiken. Julkaisuun **ei** skip-lippua.

Tuotantoavain: CI-salaisuus `KOTISATAMA_CDN_SIGNING_KEY_HEX`. Julkinen avain tuotantobuildissa `KOTISATAMA_CDN_PUBLIC_KEY` tai `config/cdn-signing-public.hex`.

---

## Whitelist-lataus (fail-safe)

Prioriteettijärjestys `init_with_fallback`:

1. CDN-cache (`index-data/cache/whitelist.json`) — vain jos integriteetti OK
2. `KOTISATAMA_WHITELIST_PATH`
3. `config/whitelist.json` (kehitys)
4. Paketin rinnalla oleva `config/whitelist.json` / `whitelist.json`

Jos mikään ei onnistu → **ei tyhjää listaa**. Navigointi estää kaikki paitsi `servo:` / `about:` / `data:`.

---

## FTN / tunnistautumisketju

Whitelistin wildcard (`kela.fi` → `*.kela.fi`) kattaa Kelan alidomainit. Erilliset merkinnät tarvitaan ketjun ulkopuolisille domaineille — ks. `docs/FILOSOFIA.md`.

v1-minimi whitelist-datassa:

- `suomi.fi`, `vero.fi`, `tunnistus.fi`, `mobiilivarmenne.fi`
- Suurimmat pankit (`nordea.fi`, `op.fi`, `aktia.fi`, …)
- Pankki- ja FTN-välittäjädomainit lisätään iteratiivisesti Telakka-testien mukaan

---

## Liittyvät tiedostot

| Tiedosto | Rooli |
|----------|-------|
| `components/kotisatama/whitelist/src/product_profile.rs` | Profiilipolitiikat |
| `components/kotisatama/whitelist/src/resolve.rs` | Fallback-lataus |
| `components/kotisatama/search/src/cdn_integrity.rs` | Manifest + SHA-256 |
| `crawler/crawl.js` | Generoi `manifest.json` |
| `SECURITY.md` | Haavoittuvuuksien ilmoitus |
