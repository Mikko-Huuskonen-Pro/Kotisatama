# Kotisatama Android build

Kotisatama uses Servo's servoshell EGL path (`ports/servoshell/egl/android/`).

## Prerequisites

- Android NDK (via Servo `./mach bootstrap`)
- Rust Android targets: `aarch64-linux-android`

## Build servoshell for Android

From repo root:

```powershell
# Windows — suositus
.\scripts\build-android.ps1

# Tai manuaalisesti
.\mach build --target aarch64-linux-android --profile checked-release
.\mach package --android --target aarch64-linux-android --profile checked-release
```

```bash
# Linux / macOS
./scripts/build-android.sh
```

APK output:

```
target/aarch64-linux-android/checked-release/servoapp.apk
```

## Kotisatama assets in APK

Gradle copies the local curated whitelist from `index-data/cache/whitelist.json`
when present, otherwise it falls back to `config/whitelist.json`. Search seed
documents are copied from `config/search-index/documents.json`.
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
| `KOTISATAMA_RESOURCES_DIR` | Extracted `resources/` (servo:haku / resource_protocol) |

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
