#!/usr/bin/env bash
# Download Meilisearch for Android (bundled into APK assets).
#
# Meilisearch does not publish official Android binaries. This script downloads a
# Linux binary as a starting point; for production you may need an NDK build.
#
# Usage:
#   ./support/android/fetch-meilisearch.sh            # aarch64 (physical device)
#   MEILISEARCH_ARCH=amd64 ./support/android/fetch-meilisearch.sh   # x86_64 emulator
#
# Output:
#   support/android/apk/servoapp/src/main/assets/kotisatama/bin/meilisearch

set -euo pipefail

VERSION="${MEILISEARCH_VERSION:-1.12.8}"
ARCH="${MEILISEARCH_ARCH:-aarch64}"
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DEST="$ROOT/support/android/apk/servoapp/src/main/assets/kotisatama/bin"
TMP="$(mktemp -d)"

mkdir -p "$DEST"
ARCHIVE="meilisearch-linux-${ARCH}"
URL="https://github.com/meilisearch/meilisearch/releases/download/v${VERSION}/${ARCHIVE}"

echo "Downloading Meilisearch v${VERSION} (linux ${ARCH})..."
curl -fsSL "$URL" -o "$TMP/meilisearch"
chmod +x "$TMP/meilisearch"
mv "$TMP/meilisearch" "$DEST/meilisearch"
rm -rf "$TMP"

echo "Installed: $DEST/meilisearch"
echo "Note: verify the binary runs on your Android device (bionic vs glibc)."
