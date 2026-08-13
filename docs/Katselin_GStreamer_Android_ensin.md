# Katselin – GStreamer-media Android ensin

## Tavoiteet

- 1. Freenet-sovellukset ja varustamo näkyviin. https://github.com/Mikko-Huuskonen-Pro/Varustamo readme.md
- 2. Consent-o-matic rustiksi ja perustumaan vain valkoisiin sivuihin
- 3. Katselimeen saadaan GStreamer-pohjainen HTML5-video toimimaan siten, että **Android on ensimmäinen kohdealusta**.

Tavoitteena ei ole korvata Servoa GStreamerilla. Servo hoitaa edelleen selaimen ja web-sisällön. GStreamer hoitaa median toiston siellä, missä se on Katselimen kannalta järkevää.

Ensimmäinen tavoite on saada tämä ketju toimimaan mahdollisimman pienellä Servo-forkilla.

---

## Nykytila Kotisatamassa

**Keskeinen havainto:** Kotisatama perii jo Servon valmiin GStreamer-media-pinon. Uutta backendia ei rakenneta alusta — työ on build-, GL- ja embedder-kytkentää.

### Toimiva ketju (desktop)

```text
HTML <video>/<audio>
        │
  HTMLMediaElement          components/script/dom/html/htmlmediaelement.rs
        │
    ServoMedia              components/media/servo-media/
        │
  GStreamerBackend          components/media/backends/gstreamer/
        │
    ServoSrc                progressive HTTP → playbin3
        │
  RenderUnix / CPU          GL-texture tai BGRA → WebRender
        │
      Näyttö
```

### Android-puutteet tänään

| Ongelma | Sijainti | Vaikutus |
|---------|----------|----------|
| Cross-build oletus = dummy | `python/servo/command_base.py` | Ei mediaa ilman `--media-stack=gstreamer` |
| GL-init puuttuu | `ports/servoshell/desktop/accelerated_gl_media.rs` (stub muilla alustoilla) | `RenderAndroid` ei saa EGL-kontekstia |
| `media_glvideo_enabled = false` | `components/config/prefs.rs` | GL-texture -polku pois päältä |
| Progressive download pois | `components/media/backends/gstreamer/player.rs` FIXME #282 | Seek/buffering-heikkouksia |
| GStreamer Android SDK | puuttuu Katselin-buildista | Runtime-pluginit puuttuvat APK:sta |

`RenderAndroid` (`components/media/backends/gstreamer/render-android/`) on jo olemassa — se tarvitsee vain EGL-kontekstin embedderiltä.

---

## 1. Hermes `gs` — päätös

Alkuperäinen suunnitelma tutki Hermes Browserin `gs`-moduulia. Repo ei viittaa siihen, eikä vastaavaa GStreamer-wrapperia löydy julkisesta lähteestä.

**Päätös:** käytetään olemassa olevaa `servo-media-gstreamer` -pinon koodia (MPL-2.0). Hermes-tutkimus lykätään — ei uutta wrapperia.

---

## 2. Android ensin: itsenäinen GStreamer-smoke test (M0)

Ennen Servo-integraatiota rakennetaan pieni testi **Katselin-repoon** (`../Katselin`):

```text
HTTPS MP4 URL → playbin → glimagesink/glsinkbin → SurfaceView
```

Testattavat: H.264/AAC MP4, WebM, play/pause/seek, verkkokatkos.

Tämä validoi GStreamer Android SDK:n ja laitteistodekoodauksen ilman Servon monimutkaisuutta. Erillinen Activity APK:ssa.

**Onnistumiskriteeri:**

> Android-laite pystyy toistamaan verkkovideon GStreamerilla.

---

## 3. Embedder Media Wiring (ei erillistä MediaBackendiä)

**Älä rakenna rinnakkaista `MediaBackend`-kerrosta**, joka duplikoi `ServoMedia` + `GStreamerBackend`.

| Kerros | Missä | Tehtävä |
|--------|-------|---------|
| **Servo media** (olemassa) | `components/media/` | HTML5-media-API, GStreamer-pipeline |
| **Embedder wiring** (uusi) | `ports/servoshell/egl/android/media.rs` | EGL/GL-init, GStreamer-bootstrap, Android-prefit |
| **Kotisatama policy** (myöhemmin, M7) | `components/kotisatama/media/` | Verkkopolitiikka, media-URL-suodatus |

`HTMLMediaElement → ServoMedia → GStreamer` -ketju **ei tarvitse muutosta** M1:een.

---

## 4. Servo → embedder-kytkentä

Servon `<video>`/`<audio>`-polku on jo valmis:

```text
HTMLMediaElement
       ↓
ServoMedia::create_player()
       ↓
GStreamerBackend (playbin3 + ServoSrc)
       ↓
RenderAndroid (glsinkbin + GL-texture)
       ↓
WebRender external image
```

JavaScriptin näkökulmasta `video.play()`, `video.pause()`, `video.currentTime`, `video.volume` jne. toimivat normaalisti — suurin osa on jo `htmlmediaelement.rs`:ssä.

Embedderin tehtävä: antaa EGL-konteksti ja varmistaa GStreamer-build.

---

## 5. Android video-output: GL-texture, ei suora Surface

**Inline `<video>` selaimessa** oikea polku on:

```text
GStreamer → glsinkbin → GL-texture (OES/2D) → WebRender → sivun layout
```

Toteutettu `components/media/backends/gstreamer/render-android/lib.rs`:ssä.

Suora `GStreamer → Android Surface` -output sopii vain **nativiin fullscreen-soittimeen** (MediaSession), ei upotettuun web-videoon. Vältetään turhaa mallia:

```text
GStreamer → videoframe → Rust → Java → Surface   ← EI tätä inline-videolle
```

---

## 6. Hardware decoding

Kun perus-GStreamer-toisto toimii, varmistetaan Androidin laitteistodekoodaus:

- H.264 (`amcviddec`)
- VP8/VP9
- laitteen tukemat muut codecit GStreamer-paketissa

GStreamer hoitaa pipeline-valinnan — Katselimeen ei rakenneta codec-kohtaista ehtologiikkaa.

---

## 7. Verkko ja media

Tuplalatausta **ei synny** — `ServoSrc` hakee bytet Servon verkko-stackista:

```text
Servo verkko-stack → ServoSrc (appsrc) → GStreamer playbin3
```

Myöhemmin (M7) samaan kokonaisuuteen liitetään Katselimen verkkopolitiikka ja whitelist.

---

## 8. Ensimmäiset formaatit

### Tier 1 (M1–M4)

- MP4, H.264, AAC
- WebM, VP8/VP9, Opus

### Tier 2 (M5–M6) — natiivi GStreamer URI

- HLS (`<video src="playlist.m3u8">`)
- DASH (manifest URL suoraan GStreamerille)

### Tier 3

- Muut codec/container-yhdistelmät playbin3:n kautta

---

## 9. HTML5-mediaominaisuudet (M2)

Suurin osa on jo toteutettu upstreamissa. Testattava Androidilla:

- play, pause, seek, volume, mute
- duration, currentTime, ended
- buffering, error
- fullscreen (Android Activity + MediaSession JNI)

---

## 10. Streamit (M5–M6)

Natiivi HLS/DASH GStreamer URI -polulla:

```text
<video src="*.m3u8"> → playbin3 → hlssrc2/dashdemux → RenderAndroid
```

JS-pohjaiset demuxerit (hls.js, dash.js) **eivät kuulu nykyiseen toteutukseen** — ks. §14.

---

## 11. Servo-forkin minimointi

> GStreameria ei lisätä Servoon tavalla, joka kasvattaa forkkia tarpeettomasti.

| Muutos | Koko | Upstream-potentiaali |
|--------|------|---------------------|
| Android GL media init | ~50–100 riviä embedderissä | Korkea |
| Cross-build media-stack default | 1 rivi Python | Keskinkertainen |
| HLS suora URI | Pieni HTMLMediaElement-haara | Korkea |
| Kotisatama media policy | 0 Servo-muutosta | — |

Upstream-muutokset merkitään `KOTISATAMA-PATCH`-kommenteilla (ks. [AGENT.md](../AGENT.md)).

---

## 12. Milestone 1 — onnistumiskriteeri

> **Android-Katselin avaa HTML-sivun, jonka `<video>`-elementti toistaa HTTPS:n yli tulevan H.264/AAC MP4-videon GStreamerilla.**

Tässä vaiheessa ei tarvita: HLS:ää, DASHia, DRM:ää, YouTube-upotusta, Windows-toteutusta.

Kun tämä toimii, ketju on todistettu:

```text
HTMLMediaElement → ServoMedia → GStreamer → RenderAndroid → WebRender → Video
```

---

## 13. Milestone-vaiheet

| Milestone | Sisältö | Tila |
|-----------|---------|------|
| **M0** | Build + GStreamer SDK + smoke test APK | Toteutettava |
| **M1** | HTML `<video>` MP4 Android | Toteutettava |
| **M2** | HTML5-media testattu Androidilla | Toteutettava (suurin osa valmis) |
| **M3** | HW decode varmistettu | Toteutettava |
| **M4** | WebM/VP9/Opus | Toteutettava |
| **M5–M6** | HLS/DASH natiivi (GStreamer URI) | Toteutettava |
| **M7** | Verkkopolitiikka (`components/kotisatama/media/`) | Toteutettava |
| **M8** | Windows (jo lähes valmis desktopilla) | Toteutettava |
| **M9** | Upstream PR (GL-init, HLS URI) | Toteutettava |
| ~~MSE / EME~~ | Media Source Extensions, DRM | **Ulkopuolella — odotetaan upstreamia** |

---

## 14. MSE — tietoisesti ulkopuolella

**Päätös:** MSE (Media Source Extensions) **ei toteuteta Kotisatamassa nyt**. Odotetaan Servo-upstreamin kehitystä (XLarge-työ).

**Perustelu:**

- Androidilla YouTube ja muut suoratoistopalvelut toimivat natiivisovelluksissa
- MSE vaatii uudet DOM-luokat (`MediaSource`, `SourceBuffer`) — ~90 WPT-testiä, ei järkevää forkata
- Backend `Player::push_data()` on jo osittain valmis upstreamissa

| Toimii Katselimessa | Ei toimi (odotetaan upstreamia) |
|---------------------|--------------------------------|
| `<video src="video.mp4">` | YouTube upotettuna selaimessa |
| `<video src="playlist.m3u8">` natiivi | hls.js / dash.js -sivustot |
| WebM, audio | Netflix, DRM (EME) |
| Whitelist-sivujen omat videot | Twitch selaimessa |

**Seuranta:** tarkista Servo-wiki ja WPT media-source -testit fork-päivityksissä. Kun upstream toteuttaa MSE:n, cherry-pick / merge forkkiin.

---

## Toteutusvaiheet

### A — Android-build: GStreamer päälle

- `../Katselin/scripts/build-android.sh`: `--media-stack=gstreamer` oletukseksi
- Harkitse `python/servo/command_base.py`: Android cross-build → gstreamer (ei dummy)
- Dokumentoi GStreamer Android NDK -asennus `support/android/README.md`:hen
- GStreamer SDK: `gstreamer-1.0`, plugins-base/good/bad/ugly, libav → APK `lib/`

### B — Smoke test (M0)

Katselin-repossa: erillinen Activity, HTTPS MP4 → playbin → SurfaceView.

### C — Android GL-kytkentä (kriittisin muutos)

1. Kun Android Surface luodaan (`ports/servoshell/egl/android/mod.rs`), kutsu:
   `Servo::initialize_gl_accelerated_media(EGL display, Gles2, EGL context)`
2. Ota `media_glvideo_enabled` käyttöön Katselimessa (`ports/servoshell/prefs.rs`)
3. Varmista `initialize_image_handler()` painter-käynnistyksessä
4. Uusi tiedosto: `ports/servoshell/egl/android/media.rs`

### D — M1: end-to-end HTML-video

Testisivu Katselin-repossa. Verkko: ServoSrc + Servon verkko-stack (ei tuplalatausta).

### E — Android-spesifiset korjaukset

- Tutki player.rs FIXME #282 (progressive download)
- Varmista hardware decode -pluginien saatavuus
- Testaa MediaSession JNI (lock screen -kontrollit)

### F — HTML5-media (M2)

Testaa Androidilla olemassa oleva HTMLMediaElement-käyttäytyminen.

### G — Formaatit (M4–M6)

Tier 1 → Tier 2 natiivi HLS/DASH. Ei MSE:tä.

### H — Verkkokerros (M7)

`components/kotisatama/media/` — URL-validointi, integrointi whitelist/content-blockingiin.

---

## Testisuunnitelma

| Vaihe | Testi | Onnistumiskriteeri |
|-------|-------|-------------------|
| M0 | Smoke test APK, HTTPS MP4 | Video näkyy SurfaceView:ssä |
| M1 | HTML `<video src="https://...mp4">` Katselimessa | Video renderöityy sivun layoutissa |
| M2 | play/pause/seek/volume/mute | HTML5-media-API toimii |
| M3 | H.264 MP4 laitteistodekoodauksella | CPU-kuorma alhainen, sujuva toisto |
| M4 | WebM VP9/Opus | Toisto onnistuu |
| M5 | `<video src="*.m3u8">` | Natiivi HLS toistuu |
| M7 | Video whitelist-sivulla | Verkkopolitiikka ei estä mediaa |

Laitteet: Android ARM64 -puhelin + x86_64-emulaattori.

---

## Lopullinen arkkitehtuuri

```text
                         KATSELIN (../Katselin)
                            │
              ┌─────────────┼─────────────┐
              │             │             │
         Servoshell      Privacy        Media policy
         (EGL/JNI)                         (M7)
              │
    HTMLMediaElement
              │
         ServoMedia
              │
      GStreamerBackend
              │
    RenderAndroid (GL-texture)
              │
         WebRender
              │
         Android-näyttö
```

### Perusperiaate

**Android ensin, hyödynnetään olemassa olevaa Servo-GStreamer-pinon koodia, embedder-kytkentä pienellä forkilla, MSE/EME odotetaan upstreamista.**
