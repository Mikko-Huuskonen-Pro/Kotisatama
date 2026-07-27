# Content-blocking network audit (Vaihe 0)

**Päivitetty:** 27.7.2026  
**Tila:** kytkentäpiste valittu — ei Servo-hookia vielä (Vaihe 2)

## Valittu kytkentäpiste

Embedder-hook: `WebViewDelegate::load_web_resource` → `WebResourceLoad`.

- Polku: `main_fetch` → `RequestInterceptor::intercept_request` → `NetToEmbedderMsg::WebResourceRequested`
- Ajoitus: **ennen** `http_network_fetch` (ennen verkkoa ja cachea)
- Toteutuspaikka Vaiheessa 2: [`ports/servoshell/running_app_state.rs`](../ports/servoshell/running_app_state.rs) (`RunningAppState: WebViewDelegate`)
- Ei tarvita `components/net/`-patchia MVP:lle (AGENT.md)

## Konteksti hookissa

| Kenttä | Saatavilla |
|---|---|
| Kohde-URL | `WebResourceRequest.url` |
| Method | kyllä |
| Headers | kyllä |
| Resurssityyppi | `destination` (CSP Destination) |
| Main frame | `is_for_main_frame` |
| Redirect | `is_redirect` |
| Lähde-/dokumentti-URL | osittain: `referrer_url` tai `webview.url()` |
| WebView | `WebView`-argumentti |

## Estomekanismi

- Salli: pudota `WebResourceLoad` → oletus `DoNotIntercept`
- Estä: `load.intercept(...).cancel()` → `LoadCancelled`, ei verkkoa
- **Älä pidä** `WebResourceLoad`-kahvaa auki — fetch jumittuu

## `request_navigation`

Vain navigoinneille (whitelist). **Ei** korvaa alipyyntösuodatusta.

## Windows vs Android

Sama `components/net/` + sama `RunningAppState`. Yksi hook riittää molempiin.

## Fail-open

Ilman toteutettua `load_web_resource`-metodia Servo sallii kaiken. Virhetilanteessa sama: salli pyyntö.
