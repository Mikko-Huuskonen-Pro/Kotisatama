#!/usr/bin/env bash
# DEPRECATED: APK assets live in Katselin. This wrapper forwards to Katselin.
# Prefer: ../Katselin/android/fetch-meilisearch.sh

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
FORWARD="$ROOT/../Katselin/android/fetch-meilisearch.sh"
if [[ ! -f "$FORWARD" ]]; then
  echo "Katselin fetch script not found: $FORWARD" >&2
  exit 1
fi
echo "Note: forwarding to Katselin android/fetch-meilisearch.sh"
exec bash "$FORWARD" "$@"
