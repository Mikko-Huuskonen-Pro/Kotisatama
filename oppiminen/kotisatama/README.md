# Kotisatama — Katselinin oma kerros

Tämä hakemisto selittää **Katselinin tuotekerroksen**: whitelist, haku, sisäiset sivut ja subprocess-sovellukset. Se ei korvaa Servo-moottorin oppimateriaalia (`oppiminen/servo/`), vaan täydentää sitä.

> **Servo on moottori, Kotisatama on satama.** Moottorin koodi on englanniksi upstream-hakemistoissa; Kotisatama-spesifinen koodi on `components/kotisatama/` ja embedder-hook `ports/servoshell/kotisatama.rs`. Katso [AGENT.md](../../AGENT.md).

## Mitä tästä löytyy

| Tiedosto | Sisältö |
|----------|---------|
| [arkkitehtuuri.md](arkkitehtuuri.md) | Kokonaiskuva: embedder → cratet → subprocessit → CDN |
| [cratet.md](cratet.md) | Jokainen `kotisatama-*`-crate, ympäristömuuttujat |
| [navigointi.md](navigointi.md) | Osoitepalkki, whitelist, alias, haku, Avomeri |
| [sisaiset-sivut.md](sisaiset-sivut.md) | Kaikki `servo:`-URL:t ja niiden käyttötarkoitus |

## Milloin lukea tätä vs. Servo-dokumentaatiota

| Ongelma | Aloita täältä | Sitten |
|---------|---------------|--------|
| Blokkaussivu (`servo:blocked`) | [navigointi.md](navigointi.md) | [kotisatama-vs-servo.md](../kotisatama-vs-servo.md) |
| Haku ei toimi / tyhjät tulokset | [cratet.md](cratet.md) → `search` | [arkkitehtuuri.md](arkkitehtuuri.md) |
| Sisäinen sivu näyttää väärältä | [sisaiset-sivut.md](sisaiset-sivut.md) | `resources/resource_protocol/` |
| Sivu latautuu mutta sisältö rikki | [servo/sivun-lataus.md](../servo/sivun-lataus.md) | [telakka/miten-debugataan.md](../telakka/miten-debugataan.md) |

## Oppimispolku D — "Haluan ymmärtää Katselinin oman kerroksen"

1. [arkkitehtuuri.md](arkkitehtuuri.md) — missä Kotisatama istuu Servon päällä
2. [navigointi.md](navigointi.md) — mitä tapahtuu osoitepalkissa
3. [cratet.md](cratet.md) — mitä kukin crate tekee
4. [sisaiset-sivut.md](sisaiset-sivut.md) — `servo:haku`, `servo:blocked` jne.
5. [kotisatama-vs-servo.md](../kotisatama-vs-servo.md) — erot upstreamiin
6. [servo/sivun-lataus.md](../servo/sivun-lataus.md) — mitä tapahtuu whitelistin jälkeen

## Liittyvät dokumentit

- [README.md](../../README.md) — tuotekuvaus ja arkkitehtuuri
- [docs/FILOSOFIA.md](../../docs/FILOSOFIA.md) — miksi whitelist ja Telakka
- [AGENT.md](../../AGENT.md) — kehityssäännöt (älä koske upstreamia)
- [kotisatama-vs-servo.md](../kotisatama-vs-servo.md) — fork-erot yhdellä sivulla
