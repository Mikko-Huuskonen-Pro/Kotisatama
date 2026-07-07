# Servo-moottori — oppimateriaali

Suomenkielinen opas Servo-selainmoottorin ymmärtämiseen tässä repossa. Kotisatama-kerros on erillisessä hakemistossa: [kotisatama/](../kotisatama/).

> **Servo on moottori, Kotisatama on satama.** Moottorin koodi on englanniksi `components/`-hakemistossa; tässä selitetään käsitteet suomeksi.

## Dokumentit

### Aloitus ja kartta

| Tiedosto | Taso | Sisältö |
|----------|------|---------|
| [arkkitehtuuri.md](arkkitehtuuri.md) | Yleiskuva | Kerrokset, mermaid-kaavio |
| [komponentit.md](komponentit.md) | Kartta | `components/`-hakemistot taulukkona |
| [sivun-lataus.md](sivun-lataus.md) | Käytännöllinen | URL:sta näytölle — koko ketju yhdellä sivulla |

### Syvä sukellus — orkestrointi ja verkko

| Tiedosto | Taso | Sisältö |
|----------|------|---------|
| [constellation-ja-navigointi.md](constellation-ja-navigointi.md) | Syvä | Pipeline, IPC-viestit, top-level navigointi |
| [iframe-ja-upotetut-kontekstit.md](iframe-ja-upotetut-kontekstit.md) | Syvä | Iframe, nested BC, Kotisatama-whitelist |
| [prosessit-ja-säikeet.md](prosessit-ja-säikeet.md) | Syvä | Prosessit, säikeet, viestikanavat |
| [verkko-ja-piirto.md](verkko-ja-piirto.md) | Syvä | HTTP-fetch, WebRender, embedder-piirto |

### Syvä sukellus — DOM, JS ja layout

| Tiedosto | Taso | Sisältö |
|----------|------|---------|
| [script-layout-ja-reflow.md](script-layout-ja-reflow.md) | Syvä | DOM-parseri, reflow-ketju |
| [javascript-moottori.md](javascript-moottori.md) | Syvä | SpiderMonkey, event loop, sidokset |
| [css-flex-ja-grid.md](css-flex-ja-grid.md) | Syvä | Flex (natiivi), Grid (Taffy), fragment tree |

### Embedder ja testaus

| Tiedosto | Taso | Sisältö |
|----------|------|---------|
| [embedder-ja-ports.md](embedder-ja-ports.md) | Keskitaso | servoshell, `WebViewDelegate` |
| [testaus-wpt.md](testaus-wpt.md) | Käytännöllinen | Web Platform Tests |

## Oppimispolut

### Polku 1 — "Mitä tapahtuu kun avaan sivun?"

1. [sivun-lataus.md](sivun-lataus.md) — kokonaiskuva
2. [constellation-ja-navigointi.md](constellation-ja-navigointi.md) — navigointi ja pipeline
3. [verkko-ja-piirto.md](verkko-ja-piirto.md) — verkko ja pikselit
4. [script-layout-ja-reflow.md](script-layout-ja-reflow.md) — DOM ja asettelu
5. [embedder-ja-ports.md](embedder-ja-ports.md) — embedder-hook

### Polku 2 — "Haluan lukea koodia järjestelmällisesti"

1. [arkkitehtuuri.md](arkkitehtuuri.md)
2. [komponentit.md](komponentit.md)
3. [prosessit-ja-säikeet.md](prosessit-ja-säikeet.md)
4. [constellation-ja-navigointi.md](constellation-ja-navigointi.md)
5. [javascript-moottori.md](javascript-moottori.md)
6. [css-flex-ja-grid.md](css-flex-ja-grid.md)
7. [iframe-ja-upotetut-kontekstit.md](iframe-ja-upotetut-kontekstit.md)
8. [verkko-ja-piirto.md](verkko-ja-piirto.md)

### Polku 3 — "Korjaan layout- tai JS-bugin"

1. [telakka/miten-debugataan.md](../telakka/miten-debugataan.md)
2. Arvioi kerros (taulukko alla)
3. [css-flex-ja-grid.md](css-flex-ja-grid.md) tai [javascript-moottori.md](javascript-moottori.md)
4. [testaus-wpt.md](testaus-wpt.md) — vastaava WPT-testi

### Polku 4 — "Iframe tai upotettu sisältö"

1. [iframe-ja-upotetut-kontekstit.md](iframe-ja-upotetut-kontekstit.md)
2. [kotisatama/navigointi.md](../kotisatama/navigointi.md) — whitelist
3. [constellation-ja-navigointi.md](constellation-ja-navigointi.md) — NavigateIframe

## Missä bugi usein piilee?

| Oire | Aloita tästä |
|------|--------------|
| Sivu ei lataudu | [verkko-ja-piirto.md](verkko-ja-piirto.md) → `net` |
| Tyhjä sivu, JS-virhe | [javascript-moottori.md](javascript-moottori.md) → `script` |
| Flex/grid layout rikki | [css-flex-ja-grid.md](css-flex-ja-grid.md) → `layout` |
| Yleinen layout-ongelma | [script-layout-ja-reflow.md](script-layout-ja-reflow.md) → reflow |
| Pikselit väärin | [verkko-ja-piirto.md](verkko-ja-piirto.md) → paint |
| Iframe tyhjä / väärä sisältö | [iframe-ja-upotetut-kontekstit.md](iframe-ja-upotetut-kontekstit.md) |
| Linkki ei avaudu | [embedder-ja-ports.md](embedder-ja-ports.md) + [kotisatama/navigointi.md](../kotisatama/navigointi.md) |
| Navigointi jumittaa | [constellation-ja-navigointi.md](constellation-ja-navigointi.md) |

## Liittyvät dokumentit

- [oppiminen/README.md](../README.md) — pääindeksi
- [kotisatama/](../kotisatama/) — Katselinin oma kerros
- [book.servo.org](https://book.servo.org) — upstream-dokumentaatio (englanniksi)
