# Kotisatama Android build

Kotisatama uses Servo's servoshell EGL path (`ports/servoshell/egl/android/`).

## Prerequisites

- Android NDK (via Servo `./mach bootstrap`)
- Rust Android targets: `aarch64-linux-android`

## Build servoshell for Android

From repo root:

```bash
./mach build --target aarch64-linux-android --profile checked-release
```

APK output (after Gradle assemble):

```
target/aarch64-linux-android/checked-release/servoapp.apk
```

## Kotisatama assets in APK

Gradle copies `config/whitelist.json` and search seed documents into APK assets.
Optional: bundle Meilisearch + index dump for offline search.

```bash
# Linux/macOS
./support/android/fetch-meilisearch.sh

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

## UI

- URL field: enter address or search Kotisatama index
- **Ilmoita** button: anonymous report (requires `KOTISATAMA_REPORT_URL`)
- Whitelist navigation enforced in Rust (same as desktop)

## Gradle APK build

```bash
cd support/android/apk
./gradlew :servoapp:assembleArm64Release
```

Use `servoview-local` when building against a local `./mach` servoshell output (see `settings.gradle.kts`).
