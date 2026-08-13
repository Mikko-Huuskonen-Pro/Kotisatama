# Kotisatama Android

> **Android-sovellus elää Katselin-päärepossa.**  
> Repo: https://github.com/Mikko-Huuskonen-Pro/Katselin  
> Polku: `android/apk/`, orkestrointi: `scripts/build-android.sh`  
> Suunnitelma: sisaruscheckoutissa `../Katselin/docs/REPO-JAKO-SUUNNITELMA.md`

Servo EGL / JNI pysyy tässä forkissa (`ports/servoshell/egl/android/`).
Gradle-projekti **ei** ole enää tämän repon lähdepuussa.

## Paikallinen junction (mach tarvitsee polun)

`mach build --target …-android` odottaa edelleen polkua
`support/android/apk`. Paikallisesti se on **Windows-junction**
(tai WSL-symlink) Katselin-apk:hon:

```
support/android/apk  →  ../Katselin/android/apk
```

Luo / korjaa:

```powershell
# Windows (repo juuresta)
.\scripts\link-android-apk.ps1
```

```bash
# WSL / Linux
./scripts/link-android-apk.sh
```

Junction **ei** kuulu gittiin (`support/android/apk/` on gitignoressa).
Kloonin jälkeen aja linkitysskripti kerran.

## Build

Suosi Katselin-orkestroijaa:

```bash
cd ../Katselin
./scripts/build-android.sh --skip-bootstrap --target x86_64-linux-android
```

Täydet ohjeet: `../Katselin/android/README.md`

Vanhat integraatiosuunnitelmat ja testiraportit tästä hakemistosta on siirretty:
`../Kotisataman-suljetut-osat/Docs/legacy/` (ajantasaiset testit: `../Katselin/android/Testit/`).

## GStreamer / HTML5-video (Android)

Katselimen `<video>`/`<audio>` käyttää Servon GStreamer-pinon koodia (`components/media/`).
Android-build vaatii GStreamerin mukaan — **älä käytä dummy-media-stackia**.

### Build

Katselin-orkestroija välittää GStreamerin automaattisesti:

```bash
cd ../Katselin
./scripts/build-android.sh --skip-bootstrap --target aarch64-linux-android
```

Tai suoraan moottorirepossa:

```bash
./mach build --target aarch64-linux-android --profile checked-release --media-stack=gstreamer
```

### GStreamer Android SDK

Cross-käännös tarvitsee GStreamer 1.18+ Android NDK -sysrootin (`pkg-config` löytää
`gstreamer-1.0`). Asenna GStreamer Android -paketti ja varmista, että `PKG_CONFIG_PATH`
osoittaa Android-ABI:n mukaiseen `.pc`-hakemistoon ennen `mach build` -ajoa.

Tarvittavat pluginit (minimi): `core`, `base`, `good`, `bad`, `ugly`, `libav` —
erityisesti `glsinkbin` videon GL-texture -polulle.

Paketoidaan APK:n `lib/` / `jniLibs/` mukaan build-vaiheessa (Gradle + mach package).

### GL-video

Embedder rekisteröi EGL-kontekstin GStreamerille:
`ports/servoshell/egl/android/media.rs` → `Servo::initialize_gl_accelerated_media()`.

Pref `media_glvideo_enabled` on päällä Kotisatama-Android-buildissa oletuksena.

### Rajoitteet

- MSE/EME (YouTube upotettuna, Netflix) **ei tueta** — odotetaan Servo-upstreamia
- Natiivi HLS: `<video src="*.m3u8">` (M5, GStreamer URI)
- Suunnitelma: [docs/Katselin_GStreamer_Android_ensin.md](../docs/Katselin_GStreamer_Android_ensin.md)

