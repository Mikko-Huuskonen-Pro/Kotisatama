# Kela-etusivu — debuggaus ja kerrosten erottelu

**Päivä:** 2026-06-29  
**Konteksti:** Ensimmäinen Telakka-oppimispäiväkirja; Kela MVP:n kohta 1–2 ([docs/KELA-TELAKKA.md](../../../docs/KELA-TELAKKA.md))  
**Komponentit:** `kotisatama-whitelist`, `ports/servoshell`, mahdollisesti `net` / `script` / `layout`

## Lähtötila

[`../Kotisataman-suljetut-osat/Docs/legacy/NYKYTILA-2026-06-24.md`](../../../../Kotisataman-suljetut-osat/Docs/legacy/NYKYTILA-2026-06-24.md) kirjaa:

- `kela` ja `kela.fi` osoitekentässä avaavat oikean Kela-sivun.
- Aiempi ongelma "Kela ei auennut oikein" ei ollut enää aktiivinen tuona päivänä.

Tämä päiväkirja dokumentoi **miten ongelma jaetaan kerroksiin** ja miten etusivua debugataan jatkossa, vaikka navigointi toimisi.

---

## Mitä yritin

### 1. Navigointi kolmella tavalla

| Tapaus | Syöte | Odotettu | Mitä tarkistaa |
|--------|-------|----------|----------------|
| A | Osoitepalkki: `kela` | `https://kela.fi/` (tai www-redirect) | Kotisatama-alias, ei moottoria |
| B | Osoitepalkki: `kela.fi` | Kela etusivu latautuu | Whitelist + normaali navigointi |
| C | Osoitepalkki: `https://www.kela.fi/` | Sama | Suora URL, vain whitelist |

Käynnistys:

```bash
export KOTISATAMA_WHITELIST_PATH=config/whitelist.json
./mach run
```

### 2. Erotin Kotisatama vs. Servo

Ennen moottorin lukemista tarkistin:

1. **Päästäänkö sivulle?** Jos näkyy blokkaussivu (`servo:blocked`), syy on whitelistissä tai embedderissä — ei Kelan HTML:ssä.
2. **Onko URL oikea?** Jos osoitepalkki näyttää kelvollisen Kela-URL:n mutta sisältö on väärä, epäillään hakualiaslogiikkaa.
3. **Latautuuko HTML?** Jos URL on oikea mutta sivu tyhjä tai rikki, siirrytään Servo-kerrokseen.

### 3. Seurasin koodipolkua tapauksessa A (`kela`)

Polku on **Kotisatama-only**:

1. Käyttäjä painaa Enter osoitepalkissa → `window.rs` kutsuu `open_search_or_results`.
2. `resolve_address_alias("kela")` etsii whitelististä merkinnän, jossa `label` on "Kela" tai domainin ensimmäinen osa on `kela`.
3. `config/whitelist.json` sisältää `"domain": "kela.fi", "label": "Kela"`.
4. `load_url_or_blocked` sallii `https://kela.fi/` ja kutsuu `webview.load`.
5. Vasta tästä eteenpäin moottori (`constellation` → `net` → `script` → …) käsittelee sivua.

**Oppiminen:** `kela`-lyhenne ei ole DNS-haku eikä Google — se on curated whitelist -alias.

### 4. Seurasin polkua tapauksessa B/C (suora URL)

1. `request_navigation` (`running_app_state.rs`) kutsuu `should_allow_navigation`.
2. `is_navigation_allowed` tarkistaa hostin `kela.fi` / `www.kela.fi` whitelist-sääntöjä vasten (alidomainit sallittu).
3. `request.allow()` → moottori jatkaa normaalisti.

Jos tässä vaiheessa `deny`, näytetään `blocked_url_for` — käyttäjä ei koskaan näe Kela-sivua.

### 5. MVP-tarkistus etusivun sisällölle

[KELA-TELAKKA](../../../docs/KELA-TELAKKA.md) vaatii:

1. `https://www.kela.fi/` latautuu.
2. Keskeinen sisältö on luettavissa.

Kun navigointi toimii, tarkistuslistaus:

| Tarkistus | Kerros jos epäonnistuu |
|-----------|------------------------|
| Verkkopyyntö 200 / redirect OK | `components/net/` |
| Konsolissa JS-virheitä | `components/script/` |
| Sisältö näkyy mutta asettelu rikki | `components/layout/` |
| Fontit / värit väärin | `components/paint/`, `fonts/` |
| Evästebanneri ei toimi | `script` + DOM-tapahtumat |

Devtools: Servo kirjaa devtools-portin lokissa käynnistyksessä (`notify_devtools_server_started`).

---

## Mitä opin

### Kotisatama-kerros (ennen HTML:ää)

| Kohta | Tiedosto / moduuli | Huomio |
|-------|-------------------|--------|
| Whitelist JSON | `config/whitelist.json` | `kela.fi` + label "Kela" |
| Alias `kela` → URL | `kotisatama.rs` → `resolve_address_alias` | Ei pistettä domainissa → haku/alias |
| Sallittu navigointi | `kotisatama_whitelist::is_navigation_allowed` | Alidomainit, lookalike-estot |
| Embedder-hook | `running_app_state.rs` → `request_navigation` | Estetty → `servo:blocked`, ei verkko-virhettä |
| Ensimmäinen välilehti | `window.rs` → `create_toplevel_webview` | Sama whitelist ennen ensimmäistä loadia |

### Servo-kerros (Kela-sivu itsessään)

Kun URL on `https://www.kela.fi/` ja sivu on sallittu, **Kotisatama ei enää osallistu** sisällön renderöintiin. Kela käyttää tyypillistä modernia webiä (JS, CSS). Puutteet ovat yleisiä Servo-ongelmia — Telakka-korjaus ei saa olla `if url.contains("kela")`.

### Vertailu upstream-Servoon

Rakenna ilman Kotisatamaa:

```bash
cargo build -p servoshell --no-default-features --release
```

Jos Kela toimii ilman featurea mutta ei sen kanssa → bugi Kotisatama-kerroksessa.  
Jos ei toimi kummassakaan → upstream Telakka-työ (`components/*` + WPT).

Katso: [kotisatama-vs-servo.md](../../kotisatama-vs-servo.md).

---

## Havainnointilomake (täytä kun löydät bugin)

Kopioi [docs/KELA-TELAKKA.md](../../../docs/KELA-TELAKKA.md) -malli:

| Kenttä | Esimerkki (hypoteettinen) |
|--------|---------------------------|
| URL | `https://www.kela.fi/` |
| Toisto | Avaa `kela` osoitepalkista |
| Odotettu | Etusivu, päänavigaatio näkyvissä |
| Toteutunut | Tyhjä valkoinen alue |
| Konsoli | `Uncaught ReferenceError: …` |
| Epäilty puute | `script` (JS) |
| Patch-status | `upstreamable` |

---

## Avoimet kysymykset

- [ ] Toimiiko `https://asiointi.kela.fi/` MVP-kohdan 5 mukaan (KELA-TELAKKA)?
- [ ] Onko etusivun päänavigaation ensimmäinen linkki testattu (MVP kohta 3)?
- [ ] Vertaillaanko sama sivu upstream-Servon viimeisimpään tagiin systemaattisesti?

---

## Seuraava askel

- [ ] Aja `./mach test-wpt` yhdellä Kelan käyttämällä web-ominaisuudella kun ensimmäinen konkreettinen rikkomus löytyy (esim. `fetch`, `css-flexbox`).
- [ ] Kirjaa seuraava merkintä kun MVP-kohdista 3–5 testataan läpi.
- [ ] Pidä osoitepalkin regressiot (`kela`, `kela.fi`, `suomi`, `suomi.fi`) tallessa ennen Avomeri-muutoksia (NYKYTILA).

## Liittyvät dokumentit

- [kotisatama-vs-servo.md](../../kotisatama-vs-servo.md)
- [miten-debugataan.md](../miten-debugataan.md)
- [servo/sivun-lataus.md](../../servo/sivun-lataus.md)
