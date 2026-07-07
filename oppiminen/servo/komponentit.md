# Komponentit (`components/`)

Kartta Servo-moottorin Rust-crateista tässä repossa. Polku on suhteessa repojuureen.

> Tämä lista ei ole täydellinen API-dokumentaatio — se auttaa löytämään oikean hakemiston. Päivitä taulukkoa kun opit uusia yhteyksiä.

## Ydin ja orkestrointi

| Hakemisto | Mitä tekee | Katselimen näkökulma |
|-----------|------------|----------------------|
| `components/servo/` | Moottorin julkinen API embedderille | Aloita tästä jos tutkit integraatiota |
| `components/constellation/` | Välilehdet, prosessit, navigointi, historia | Sivun avautuminen, uusi välilehti |
| `components/config/` | Asetukset ja preferenssit | Kokeelliset featuret, debug-asetukset |

## Verkko ja tallennus

| Hakemisto | Mitä tekee | Katselimen näkökulma |
|-----------|------------|----------------------|
| `components/net/` | HTTP, TLS, evästeet, välimuisti | Kela-asiointi, kirjautuminen, API-kutsut |
| `components/storage/` | Paikallinen tallennus (localStorage jne.) | Istunto, asetukset sivuilla |

## DOM, skriptit ja sidokset

| Hakemisto | Mitä tekee | Katselimen näkökulma |
|-----------|------------|----------------------|
| `components/script/` | HTML-parseri, DOM, JavaScript-moottori | Monimutkaiset sivut, lomakkeet, SPA:t |
| `components/script_bindings/` | Web API -sidokset JS:lle | `fetch`, `console`, DOM-metodit |
| `components/dom_struct/` | DOM-rakenteiden makrot | Syvä sukellus — myöhemmin |

## Ulkoasu ja piirto

| Hakemisto | Mitä tekee | Katselimen näkökulma |
|-----------|------------|----------------------|
| `components/layout/` | CSS, asettelupuu, flex/grid | Lukettavuus, responsiivisuus, Kela-layout |
| `components/paint/` | Piirto, tekstit, kuvat | Visuaaliset bugit, fontit |
| `components/fonts/` | Fonttien lataus ja rasterointi | Erikoisfontit, ääkköset |
| `components/canvas/` | Canvas 2D -API | Kaaviot, piirtoelementit |
| `components/pixels/` | Pikselipuskurit | Matalan tason piirto |

## Grafiikka-API:t

| Hakemisto | Mitä tekee | Katselimen näkökulma |
|-----------|------------|----------------------|
| `components/webgl/` | WebGL | Harvemmin kriittinen viranomaissivuilla |
| `components/webgpu/` | WebGPU | Uudempi grafiikka-API |

## Apu ja infrastruktuuri

| Hakemisto | Mitä tekee |
|-----------|------------|
| `components/url/` | URL-jäsennys |
| `components/geometry/` | Geometriatyypit |
| `components/timers/` | Ajastimet |
| `components/metrics/` | Mittarit |
| `components/profile/` | Profilointi |
| `components/devtools/` | Kehittäjätyökalut |
| `components/allocator/` | Muistinhallinta |

## Kotisatama (ei upstream)

| Hakemisto | Mitä tekee |
|-----------|------------|
| `components/kotisatama/whitelist/` | Whitelist-logiikka |
| `components/kotisatama/search/` | Haku (Meilisearch-client) |
| `components/kotisatama/pulloposti/` | Pulloposti-daemon-client |

Nämä eivät ole osa upstream-Servoa — opiskellessa moottoria erota ne upstream-komponenteista.

## Embedder (ei `components/`)

| Hakemisto | Mitä tekee |
|-----------|------------|
| `ports/servoshell/` | Desktop/Android-kuori, navigointihookit |

Lue: [embedder-ja-ports.md](embedder-ja-ports.md).

## Miten käyttää tätä karttaa

1. Toista bugi (esim. Kela-sivu).
2. Arvioi **mikä kerros** epäonnistuu (ei lataudu → `net`, väärä asettelu → `layout`, JS-virhe → `script`).
3. Avaa vastaava hakemisto ja etsi WPT-testi aiheesta: [testaus-wpt.md](testaus-wpt.md).

## Syvä dokumentaatio komponenteittain

| Kerros | Dokumentti |
|--------|------------|
| Constellation | [constellation-ja-navigointi.md](constellation-ja-navigointi.md) |
| Iframe / nested BC | [iframe-ja-upotetut-kontekstit.md](iframe-ja-upotetut-kontekstit.md) |
| JavaScript | [javascript-moottori.md](javascript-moottori.md) |
| Script + layout (yleinen) | [script-layout-ja-reflow.md](script-layout-ja-reflow.md) |
| Flex + grid | [css-flex-ja-grid.md](css-flex-ja-grid.md) |
| Net + paint | [verkko-ja-piirto.md](verkko-ja-piirto.md) |
| Prosessit | [prosessit-ja-säikeet.md](prosessit-ja-säikeet.md) |
