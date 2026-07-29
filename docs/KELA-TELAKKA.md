# Kela Telakka

Tämä dokumentti rajaa ensimmäisen Kela-kierroksen. Tavoite ei ole tehdä sivukohtaisia selainmoottorihackeja, vaan löytää ensimmäinen konkreettinen Servo-puute ja korjata se yleisesti.

## MVP-rajaus

Kela MVP on valmis, kun nämä toimivat Kotisatamassa:

1. `https://www.kela.fi/` latautuu.
2. Etusivun keskeinen sisältö on luettavissa.
3. Päänavigaation linkki avautuu samassa satamassa.
4. Kelan hakuun pääsee tai hakulomake antaa ymmärrettävän virheen.
5. `https://asiointi.kela.fi/` avautuu tunnistautumisen alkuun asti.

## Whitelist-lähtötila

`config/whitelist.json` sisältää `kela.fi`-merkinnän. Whitelist-logiikka sallii myös alidomainit, joten ainakin nämä ovat satamassa:

- `www.kela.fi`
- `asiointi.kela.fi`

Lookalike-domainit eivät saa mennä läpi:

- `kela.fi.example.com`
- `example-kela.fi`
- `kelafi.example`

## Ensimmäinen testikierros

Kirjaa jokainen hajoamiskohta näin:

- URL: tarkka osoite.
- Toisto: lyhyt askel, jolla virhe näkyy.
- Odotettu: mitä selaimen pitäisi tehdä.
- Toteutunut: mitä Servo/Kotisatama teki.
- Konsoli/loki: olennainen virheviesti.
- Epäilty puute: esimerkiksi layout, fetch, evästeet, lomakkeet, dialogit tai PDF.
- Patch-status: `upstreamable`, `local-only`, `submitted` tai `remove-when-upstreamed`.

## Seuraava tekninen askel

Kun ensimmäinen rikkomus on toistettu, korjaus tehdään pienimpänä yleisenä muutoksena. Älä lisää ehtoja muodossa `if url contains kela.fi`; Kela toimii testikohteena, ei poikkeuksena.

---

## Toteutettu korjaus: Next.js / TCP-yhteysraja

**Patch-status:** `local-only` (yleinen verkko-optimointi; upstreamattavissa erikseen)

**Tiedosto:** `components/net/connector.rs` (`KOTISATAMA-PATCH`)

### Ongelma

`https://www.kela.fi/` palautti HTML:n (200), mutta Next.js-chunkit (`/_next/static/chunks/…`) kaatuivat toistuvasti:

- `ConnectionReset` / Windows 10054
- `Fetching classic script failed`
- React ei hydratoitunut → sivu jäi "yksinkertaiseen tilaan" (staattinen HTML ilman interaktiivisuutta)

**Syy:** Next.js-sivut avaavat kymmeniä chunk-GETejä rinnakkain. Servon Hyper-connector avaa ilman rajaa yhtä monta HTTP/1.1-TCP-yhteyttä per host. CDN tai Windows katkaisee liian monta yhteyttä → chunkit epäonnistuvat → JS ei lataudu.

### Korjaus

Selainmainen TCP-yhteysraja per host (oletus **6**, kuten HTTP/1.1-käytännössä):

- `host_connect_semaphore()` — yksi `Semaphore` per host-avain
- `LimitedTcpStream` — pitää `OwnedSemaphorePermit`in streamin elinkaaren ajan (myös idle-poolissa)
- Säätö: ympäristömuuttuja `KOTISATAMA_MAX_CONN_PER_HOST`

Patch ei ole Kelakohtainen. Sama mekanismi auttaa kaikkia Next.js- ja muuta rinnakkaislatausta tekeviä sivuja.

### Miksi juuri `components/net/`, ei Kotisatama-crate

`WebResourceLoad` voi vain sallia tai estää pyynnön — se ei voi jonottaa TCP-yhteyksiä. Yhteyden elinkaari elää Hyper-connectorissa, joten pieni patch tähän tiedostoon on ainoa toimiva paikka ilman laajempaa net-/script-diffiä.

Kotisatama-kerroksessa (`ports/servoshell/`, `components/kotisatama/`) ei ole sivukohtaista injektiota, user-agent-spooffausta eikä CSS/JS-polyfilliä Kelalle. Whitelist, alias (`kela` → `kela.fi`) ja hakuhitit ovat navigointia — eivät sivun toimivuuden korjausta.

### Testikierroksen merkintä (esimerkki)

| Kenttä | Arvo |
|---|---|
| URL | `https://www.kela.fi/henkiloasiakkaat` |
| Toisto | Avaa etusivu tai alisivu Kotisatamassa |
| Odotettu | Hydratoitu React-sivu, navigaatio toimii |
| Toteutunut (ennen patchia) | HTML näkyi, interaktiivisuus puuttui |
| Konsoli/loki | `ConnectionReset`, `Fetching classic script failed` |
| Epäilty puute | Verkko — liian monta rinnakkaista TCP-yhteyttä |
| Patch-status | `local-only` |

---

## YouTube ja muut moottoritason puutteet

YouTube on hyvä vastakohta Kelalle: siellä ongelma **ei** ratkea samanlaisella Kotisatama-kerroksen korjauksella.

### YouTube (2026-07-28): scroll kaatuu

| | Kela.fi | YouTube |
|---|---|---|
| Kerros | Verkko (`components/net/`) | Layout/paint (`components/shared/paint/`) |
| Ongelma | Liikaa TCP-yhteyksiä | Vanhentunut `ScrollTreeNodeId` → panic |
| Korjaustapa | Yleinen yhteysraja | Moottoribugi Servossa |
| Kotisatama-kerros auttaa? | Kyllä (patch jo tehty) | **Ei** |

Scrollaus aiheuttaa fatal panicin:

```text
index out of bounds: the len is 42 but the index is 44
components\shared\paint\display_list.rs:473  (ScrollTree::get_node)
```

Layout pitää viittausta scroll-nodeen, jota ei enää ole puussa. Sama bugiperhe on upstreamissa (esim. [servo#42215](https://github.com/servo/servo/issues/42215), [servo#42593](https://github.com/servo/servo/issues/42593)).

Lisäksi lokissa näkyy erillisiä Servo-puutteita (eivät aiheuta tätä kaatumista):

- `IDBIndex.openCursor` puuttuu → `TypeError: … openCursor is not a function`
- CSS-varoituksia (`-moz-box-orient`, `width: NaNpx`)
- Google Sign-In / `googlevideo.com` 403 — erillinen yhteensopivuus- ja mediakysymys

### Arvio: odota upstreamia

**Kannattaa jäädä odottamaan upstream-korjauksia eikä lähteä ratkomaan YouTube-ongelmaa Kotisatama-kerroksessa.**

Perustelut:

1. **Kaatumispiste on syvällä Servossa** — scroll-puu, layout, display list. Kotisatama ei voi turvallisesti "kiertää" paniikkia injektiolla tai UI-patchilla.
2. **Kelamalli ei skaalaudu** — TCP-yhteysraja on geneerinen selainkäytös, ei `if url contains youtube.com`. YouTubelle vastaava "pieni yleinen korjaus" olisi itse moottorin korjaus upstreamissa.
3. **Sivukohtaiset hackit rikkovat filosofian** — ks. [FILOSOFIA.md](FILOSOFIA.md): puuttuva osa korjataan yleisesti, ei piiloteta Chromea tai sivukohtaisia poikkeuksia.
4. **Upstream tietää ongelman** — ScrollTree-panicit on jo raportoitu; oikea paikka korjaukselle on Servo/Telakka, ei `ports/servoshell/`.

**Telakan toimenpide YouTubelle:** kirjaa rikkomus testikierrokseen, seuraa upstream-issueja, testaa uudelleen upstream-mergejen jälkeen. Älä aloita Kotisatama-kerroksen site-compat -hackeja.

**Telakan toimenpide Kelalle:** jatka seuraavia MVP-testejä (navigaatio, haku, asiointi.kela.fi). Verkkopatch on paikallaan; jäljellä olevat esteet ovat todennäköisesti muita Servo-puutteita (lomakkeet, evästeet, tunnistautumisketju).
