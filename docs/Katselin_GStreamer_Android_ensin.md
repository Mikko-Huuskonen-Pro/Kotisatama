# Katselin – GStreamer-media Android ensin

## Tavoiteet

- 1.Freenet-sovellukset ja varustamo näkyviin. https://github.com/Mikko-Huuskonen-Pro/Varustamo readme.md

- 2.Consent-o-matic rustiksi ja perustumaan vain valkoisiin sivuihin

- 3. Katselimeen rakennetaan GStreamer-pohjainen media-backend siten, että **Android on ensimmäinen kohdealusta**.

Tavoitteena ei ole korvata Servoa GStreamerilla. Servo hoitaa edelleen selaimen ja web-sisällön. GStreamer hoitaa median toiston siellä, missä se on Katselimen kannalta järkevää.

Perusketju:

```text
HTML <video>/<audio>
        │
       Servo
        │
Katselin Media Adapter
        │
GStreamer Backend
        │
Android Surface / audio output
```

Ensimmäinen tavoite on saada tämä ketju toimimaan mahdollisimman pienellä Servo-forkilla.

---

## 1. Ensimmäinen vaihe: selvitä Hermesin `gs`

Hermes Browserin yhteydessä oleva `gs` tutkitaan ennen kuin sitä otetaan suoraan käyttöön.

Selvitetään:

- mitä `gs` tarkalleen tekee
- onko se GStreamer-wrapper vai oma media-abstraktio
- mitä riippuvuuksia sillä on
- tukeeko se Androidia
- miten media-URL annetaan sille
- miten video-output toteutetaan
- miten audio käsitellään
- millainen lisenssi ja uudelleenkäyttömahdollisuus sillä on

Tässä vaiheessa ei vielä tehdä muutoksia Servoon.

**Tulos:** selkeä päätös siitä, käytetäänkö Hermesin `gs`:ää sellaisenaan, hyödynnetäänkö siitä osia vai rakennetaanko Katselimen oma GStreamer-adapteri.

---

## 2. Android ensin: itsenäinen GStreamer-testi

Ennen Servo-integraatiota rakennetaan pieni Android-testi.

Sen ainoa tehtävä on:

```text
Media URL
    ↓
GStreamer
    ↓
Android Surface
```

Testataan ensin tavallinen HTTPS:n yli tuleva H.264/AAC MP4.

Sen jälkeen:

- MP4
- WebM
- audio
- pause/play
- seek
- buffering
- verkkoyhteyden katkeaminen
- fullscreen / orientation

Ensimmäinen onnistumiskriteeri:

> Android-laite pystyy toistamaan verkkovideon GStreamerilla suoraan Androidin Surfaceen.

---

## 3. Katselimeen oma MediaBackend

GStreameria ei sidota suoraan Servon jokaiseen media-API:n kohtaan.

Katselimeen tehdään oma abstraktiokerros:

```text
media/
├── mod.rs
├── player.rs
├── source.rs
├── surface.rs
├── state.rs
└── gstreamer.rs
```

Ajatus:

```text
MediaBackend
      │
      └── GStreamerBackend
```

Tällä pidetään Servo-riippuvuus pienenä.

Myöhemmin backend voidaan tarvittaessa vaihtaa tai laajentaa ilman, että koko media-integraatio rakennetaan uudelleen.

---

## 4. Servo → Katselin Media Adapter

Seuraavaksi selvitetään Servon nykyinen `<video>`/`<audio>`-mediaelementin polku.

Tavoite:

```text
HTMLMediaElement
       ↓
Servo
       ↓
Katselin Media Adapter
       ↓
GStreamer
```

Katselin ei saa rikkoa normaalia HTML5-media-API:a.

JavaScriptin näkökulmasta:

```text
video.play()
video.pause()
video.currentTime = ...
video.volume = ...
```

toimii normaalisti.

Taustalla media voidaan ohjata GStreamerille.

---

## 5. Android Surface

Video pyritään viemään mahdollisimman suoraviivaisesti GStreamerista Androidin native Surfaceen.

Ensisijainen tavoite:

```text
GStreamer
    ↓
hardware decoder
    ↓
Android Surface
```

Vältetään turhaa mallia:

```text
GStreamer
    ↓
videoframe
    ↓
Rust
    ↓
Java/Kotlin
    ↓
Surface
```

Koska tämä aiheuttaisi ylimääräistä muistinkopiointia ja CPU-kuormaa.

---

## 6. Hardware decoding

Kun perus-GStreamer-toisto toimii, otetaan Androidin laitteistodekoodaus käyttöön.

Ensisijaisesti testataan:

- H.264
- VP8/VP9
- laitteen tukemat muut codecit

Tavoitteena on hyödyntää Android-laitteen omaa media-rautaa aina kun mahdollista.

---

## 7. Verkko ja media

Tavoitteena ei ole ladata samaa mediaa kahteen kertaan:

```text
Servo → video
GStreamer → sama video uudelleen
```

vaan rakentaa hallittu media-polku.

Alustava malli:

```text
              Servo
                │
          media resource
                │
       Katselin Media Adapter
                │
           GStreamer
                │
              network
```

Myöhemmin samaan kokonaisuuteen voidaan liittää Katselimen muut verkkopolitiikat ja suodatukset.

---

## 8. Ensimmäiset formaatit

Ensimmäiseen toimivaan versioon ei yritetä toteuttaa kaikkea.

### Tier 1

- MP4
- H.264
- AAC
- WebM
- VP8/VP9
- Opus

### Tier 2

- HLS
- DASH

### Tier 3

- muut erikoisemmat codec/container-yhdistelmät

Tärkeää on, että GStreamer hoitaa pipeline-valinnan eikä Katselimeen rakenneta valtavaa määrää codec-kohtaista ehtologiikkaa.

---

## 9. HTML5-mediaominaisuudet

Kun perusvideo toimii, toteutetaan normaali mediaelementin käyttäytyminen:

- play
- pause
- seek
- volume
- mute
- duration
- currentTime
- ended
- buffering
- error
- fullscreen

Tässä vaiheessa Katselimen pitäisi käyttäytyä verkkosivulle mahdollisimman normaalina selaimena.

---

## 10. Streamit

Vasta tavallisen MP4-toiston jälkeen siirrytään adaptiivisiin streameihin.

### HLS

```text
HLS
 ↓
GStreamer
 ↓
Android
```

### DASH

```text
DASH
 ↓
GStreamer
 ↓
Android
```

Tämän jälkeen voidaan tutkia adaptive bitrate -toimintaa.

---

## 11. Servo-forkin minimointi

Tärkeä periaate:

> GStreameria ei lisätä Servoon tavalla, joka kasvattaa Katselimen ylläpitämää Servo-forkkia tarpeettomasti.

Ensisijainen järjestys:

```text
Katselin adapter
      ↓
Servo API / hook
      ↓
tarvittaessa pieni Servo-muutos
      ↓
upstream PR
```

Jos Servo tarjoaa tarvittavan rajapinnan, käytetään sitä.

Jos rajapinta puuttuu, tehdään mahdollisimman pieni muutos ja pyritään viemään se upstreamiin.

---

## 12. Milestone 1

Ensimmäinen todellinen onnistumiskriteeri on:

> **Android-Katselin avaa HTML-sivun, jonka `<video>`-elementti toistaa HTTPS:n yli tulevan H.264/AAC MP4-videon GStreamerilla Androidin Surfaceen.**

Tässä vaiheessa ei vielä tarvita:

- HLS:ää
- DASHia
- DRM:ää
- YouTube-spesifisiä ratkaisuja
- kaikkien codecien tukea
- Windows-toteutusta

Kun tämä toimii, koko tärkein ketju on todistettu:

```text
Servo
  ↓
Katselin
  ↓
Media Adapter
  ↓
GStreamer
  ↓
Android
  ↓
Video
```

---

## 13. Seuraavat milestone-vaiheet

### M1 – Android MP4

Servo + Katselin + GStreamer + Android Surface.

### M2 – HTML5-media

play/pause/seek/volume/fullscreen/error/buffering.

### M3 – hardware decoding

Androidin laitteistodekoodaus.

### M4 – WebM ja muut perusformaatit

VP8/VP9/Opus jne.

### M5 – HLS

### M6 – DASH

### M7 – verkkokerroksen integraatio

Media osaksi Katselimen hallittua verkkopolitiikkaa.

### M8 – Windows

Sama MediaBackend-ajatus Windowsille.

### M9 – mahdollinen upstream

Servo-muutosten siirtäminen upstreamiin, jos niistä on hyötyä myös muille Servo-embeddaajille.

---

## Lopullinen arkkitehtuuri

```text
                         KATSELIN
                            │
              ┌─────────────┼─────────────┐
              │             │             │
            Servo        Privacy        Media
              │             │             │
              │             │       Media Adapter
              │             │             │
              └─────────────┴─────────────┤
                                          │
                                  GStreamer Backend
                                          │
                              ┌───────────┴───────────┐
                              │                       │
                           Android                 Desktop
                              │
                    Android Surface / audio
```

### Perusperiaate

**Android ensin, GStreamer erillisenä media-backendinä ja Servo mahdollisimman vähän forkattuna.**

Näin GStreamer-integraatio voidaan rakentaa ensin konkreettisesti toimivaksi Androidissa ilman, että Katselimen selainmoottorin arkkitehtuuria tarvitsee ratkaista kerralla kokonaan.
