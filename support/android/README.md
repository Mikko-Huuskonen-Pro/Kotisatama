# Kotisatama Android build

Kotisatama uses Servo's servoshell EGL path (`ports/servoshell/egl/android/`).

## Prerequisites

- **Linux or macOS** (or **WSL2** on Windows — native Windows `mach` does not support Android cross builds)
- Android NDK **r28** (`28.2.13676358`) — newer NDKs are rejected; see below
- Rust Android targets: `aarch64-linux-android` (physical device) and/or `x86_64-linux-android` (emulator)

## Emulator vs physical device

| Target | Use for | APK variant |
|---|---|---|
| `x86_64-linux-android` | **x86_64 emulator (AVD)** — recommended for emulator testing | `servoapp-x64Release.apk` |
| `aarch64-linux-android` | Physical phones/tablets (arm64) | `servoapp-arm64Release.apk` |

> **Note:** An arm64 APK on an x86_64 emulator runs through the Berberis
> translator and currently crashes in `JNIServo.init`
> (`Rust error: Java exception was thrown`). Build for `x86_64-linux-android`
> when testing in the emulator.

## Build (WSL2 on Windows)

```bash
# WSL2, repo root (Linux filesystem clone preferred; /mnt/c works but is slower)
cd ~/Kotisatama/Kotisatama

export ANDROID_SDK_ROOT=~/Android/Sdk
export ANDROID_NDK_ROOT=$ANDROID_SDK_ROOT/ndk/28.2.13676358
export PATH="$ANDROID_SDK_ROOT/platform-tools:$PATH"

# First time only: install Linux NDK r28 into WSL (a Windows NDK will not work in WSL)
./scripts/build-android.sh --install-ndk --target x86_64-linux-android

# Emulator build + install (every subsequent run)
./scripts/build-android.sh --skip-bootstrap --target x86_64-linux-android --install --emulator
```

For a physical arm64 device:

```bash
./scripts/build-android.sh --skip-bootstrap --target aarch64-linux-android --install --usb
```

The script runs unit tests, syncs the whitelist, downloads the matching
Meilisearch binary (`amd64` for x86_64, `aarch64` for arm64), runs
`mach build` (which invokes Gradle assemble), and installs the APK via `adb`.

Useful flags: `--skip-tests`, `--skip-meilisearch`, `--skip-whitelist-sync`.

### Manual commands (equivalent)

```bash
./mach bootstrap --yes
MEILISEARCH_ARCH=amd64 ./support/android/fetch-meilisearch.sh   # aarch64 for a physical device
./mach build --target x86_64-linux-android --profile checked-release
adb install -r support/android/apk/servoapp/build/outputs/apk/x64Release/servoapp-x64Release.apk
```

APK outputs:

```
support/android/apk/servoapp/build/outputs/apk/x64Release/servoapp-x64Release.apk     # emulator
support/android/apk/servoapp/build/outputs/apk/arm64Release/servoapp-arm64Release.apk # device
target/<target>/checked-release/servoapp.apk                                          # symlink/copy
```

> **Note:** `./mach package --android --target <t>` currently fails with
> "Please specify either --target or --android" — `mach build` already runs
> the Gradle assemble step and produces the APK, so `mach package` is not
> needed. Install with `adb install -r <apk>` (start the emulator from
> Android Studio first).

## Meilisearch Androidilla

Meilisearchin viralliset Linux-binäärit ovat **dynaamisesti linkitettyjä glibc:hen**
(interpreter `/lib64/ld-linux-x86-64.so.2`), eikä Androidin bionic-libc suorita niitä.
Siksi `KOTISATAMA_MEILISEARCH_BIN` yksin ei riitä: spawn epäonnistuu.

**Ratkaisu:** Androidilla (ja aina kun Meilisearch ei käynnisty) haku käyttää
muistissa toimivaa **seed-varahakua** (`kotisatama_search::seed_search`), joka
osumat hakee `documents.json`:n ja whitelistin kuratoiduista dokumenteista
(case-insensitive substring, pisteytetty). Sivulla ei näy virhettä, tulokset tulevat
kuratoidusta aineistosta.

> Tuotantotason ratkaisu olisi NDK:lla käännetty Meilisearch; se on oma työkokonaisuutensa.

## Kotisatama assets in APK

Gradle copies the local curated whitelist from `index-data/cache/whitelist.json`
when present, otherwise it falls back to `config/whitelist.json`. Search seed
documents are copied from `config/search-index/documents.json`.
Optional: bundle Meilisearch + index dump for offline search.

```bash
# Linux/macOS — aarch64 (device)
./support/android/fetch-meilisearch.sh

# Linux/macOS — amd64 (x86_64 emulator)
MEILISEARCH_ARCH=amd64 ./support/android/fetch-meilisearch.sh

# Windows
./support/android/fetch-meilisearch.ps1
```

Place `index-data/index.dump` in repo root before building APK to bundle a pre-built index.

## Runtime

On first launch, `KotisatamaAssets` extracts assets to app private storage and sets:

| Variable | Purpose |
|---|---|
| `KOTISATAMA_WHITELIST_PATH` | Whitelist JSON |
| `KOTISATAMA_MEILISEARCH_BIN` | Extracted Meilisearch binary |
| `KOTISATAMA_MEILISEARCH_DB` | Local index database |
| `KOTISATAMA_INDEX_DUMP` | Optional dump import |
| `KOTISATAMA_SEARCH_DOCUMENTS` | Seed documents fallback |
| `KOTISATAMA_RESOURCES_DIR` | Extracted `resources/` (servo:haku / resource_protocol) |

## UI

- URL field: enter address or search Kotisatama index
- Top bar: **Avaa sovelluksessa** (opens current URL in a native app via `ACTION_VIEW`)
- Bottom bar: **Lokikirja** anonymous report (requires `KOTISATAMA_REPORT_URL` or `KOTISATAMA_GITHUB_TOKEN`)
- Experimental web prefs: on by default
- Whitelist navigation enforced in Rust (same as desktop)

First successful emulator run notes (commands): `oppiminen/telakka/oppimispäiväkirja/2026-07-31-android-ensimmainen-build.md`

## Gradle APK build

```bash
cd support/android/apk
./gradlew :servoapp:assembleArm64Release
```

Use `servoview-local` when building against a local `./mach` servoshell output (see `settings.gradle.kts`).
