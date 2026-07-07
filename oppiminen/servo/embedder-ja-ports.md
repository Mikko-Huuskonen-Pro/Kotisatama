# Embedder ja ports

**Embedder** on sovellus, joka upottaa Servo-moottorin. Katselimessä se on käytännössä **servoshell** (`ports/servoshell/`).

## Miksi embedder on tärkeä?

Moottori (`components/`) ei tiedä mitään ikkunoista, osoitepalkista tai Kotisataman whitelististä. Embedder:

- näyttää ikkunan ja piirtää kehyksen
- välittää käyttäjän syötteen moottorille
- voi **hyväksyä tai hylätä navigoinnin** ennen kuin pyyntö menee moottoriin

Tämä on Katselimin ensisijainen paikka whitelist-logiikalle (ks. [AGENT.md](../../AGENT.md)).

## Navigointihook

Upstream tarjoaa delegaatin, jota embedder toteuttaa. Katselin käyttää sitä suunnilleen näin:

```rust
// ports/servoshell/ — konseptuaalinen esimerkki (KOTISATAMA-PATCH)
impl WebViewDelegate for RunningAppState {
    fn request_navigation(&self, _webview: WebView, request: NavigationRequest) {
        if kotisatama_whitelist::is_allowed(&request.url) {
            request.allow();
        } else {
            request.deny();
        }
    }
}
```

Logiikka pysyy `components/kotisatama/whitelist/`-cratessa; servoshell vain kutsuu sitä.

## `ports/`-hakemisto

| Polku | Käyttö |
|-------|--------|
| `ports/servoshell/` | Desktop- ja Android-embedder |
| `ports/servoshell/egl/android/` | Android APK -polku |

Androidissa Katselin käyttää Servon omaa embedderia, ei Tauria (ks. [README.md](../../README.md)).

## Mitä embedder **ei** tee

- Ei jäsentä HTML:ää → `components/script/`
- Ei tee HTTP-pyyntöjä → `components/net/`
- Ei laske CSS-asettelua → `components/layout/`

Kun bugi on "sivu latautuu mutta näyttää väärältä", syy on harvoin embedderissä.

## Opiskelujärjestys

1. Lue [sivun-lataus.md](sivun-lataus.md) — missä hook on ketjussa
2. Selaa `ports/servoshell/` — etsi `request_navigation` tai `WebViewDelegate`
3. Vertaa `components/kotisatama/whitelist/` — mitä `is_allowed` tarkistaa

## Seuraavaksi

- [constellation-ja-navigointi.md](constellation-ja-navigointi.md) — mitä tapahtuu allow/deny:n jälkeen
- [komponentit.md](komponentit.md) — moottorin cratet hookin jälkeen
- [kotisatama-vs-servo.md](../kotisatama-vs-servo.md) — kaikki fork-erot yhdessä
- [telakka/miten-debugataan.md](../telakka/miten-debugataan.md)
