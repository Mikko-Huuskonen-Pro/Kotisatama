#!/usr/bin/env bash
# One-shot: build x86_64 APK in WSL and install to emulator.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}"
export ANDROID_HOME="$ANDROID_SDK_ROOT"
export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$ANDROID_SDK_ROOT/ndk/28.2.13676358}"

echo "ANDROID_SDK_ROOT=$ANDROID_SDK_ROOT"
echo "ANDROID_NDK_ROOT=$ANDROID_NDK_ROOT"
ls "$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/clang"
which java rustc cargo

LOG="$ROOT/index-data/android-build-wsl.log"
mkdir -p "$ROOT/index-data"

./scripts/build-android.sh \
  --skip-bootstrap \
  --skip-tests \
  --skip-meilisearch \
  --target x86_64-linux-android \
  --install \
  --emulator \
  2>&1 | tee "$LOG"

echo "DONE exit=${PIPESTATUS[0]}"
