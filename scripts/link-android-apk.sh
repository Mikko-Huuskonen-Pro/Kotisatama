#!/usr/bin/env bash
# Luo / korjaa support/android/apk → Katselin/android/apk (symlink)
#
# Käyttö (Kotisatama-repo juuresta, WSL/Linux):
#   ./scripts/link-android-apk.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LINK="$ROOT/support/android/apk"
TARGET="$(cd "$ROOT/../Katselin/android/apk" && pwd)"

if [[ ! -f "$TARGET/settings.gradle.kts" ]]; then
  echo "Katselin apk not found at $TARGET" >&2
  exit 1
fi

mkdir -p "$(dirname "$LINK")"

if [[ -L "$LINK" ]]; then
  rm -f "$LINK"
elif [[ -d "$LINK" ]]; then
  echo "Refusing to remove real directory at $LINK — move it aside first." >&2
  exit 1
elif [[ -e "$LINK" ]]; then
  echo "Unexpected file at $LINK" >&2
  exit 1
fi

ln -sfn "$TARGET" "$LINK"
test -f "$LINK/settings.gradle.kts"
echo "OK: $LINK → $TARGET"
