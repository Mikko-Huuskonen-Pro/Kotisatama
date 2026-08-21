#!/usr/bin/env bash
# Build arm64 APK with GStreamer Android static link flags for real devices.
set -euo pipefail
GST_ROOT="${GSTREAMER_ROOT_ANDROID:-$HOME/Android/gstreamer-1.0-android-universal-1.22.12}"
export GSTREAMER_ROOT_ANDROID="$GST_ROOT"
export PKG_CONFIG_ALLOW_CROSS=1
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}"
export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$ANDROID_SDK_ROOT/ndk/28.2.13676358}"

ENGINE=/mnt/c/Users/gigli/Kotisatama/Kotisatama
KATSELIN=/mnt/c/Users/gigli/Kotisatama/Katselin
cd "$ENGINE"

echo "GSTREAMER_ROOT_ANDROID=$GSTREAMER_ROOT_ANDROID"
test -f "$GSTREAMER_ROOT_ANDROID/arm64/lib/libffi.a"

./mach build --target aarch64-linux-android --profile checked-release --media-stack=gstreamer --no-package

SO="$ENGINE/target/aarch64-linux-android/checked-release/libservoshell.so"
UNDEF=$(llvm-nm -D --undefined-only "$SO" | awk '/^(gst_|orc_|pcre2_|g_module_|egl|__clear_cache|nice_|BZ2_|graphene_)/ {print $NF}' | sort -u)
if [[ -n "$UNDEF" ]]; then
  echo "ERROR: unresolved media/GL symbols in libservoshell.so:" >&2
  echo "$UNDEF" >&2
  exit 1
fi
echo "libservoshell.so: no undefined gst_/orc_/pcre2_/g_module_/egl symbols"

export SERVO_TARGET_DIR="$ENGINE/target/aarch64-linux-android/checked-release"
cd "$ENGINE/support/android/apk"
./gradlew --no-daemon -PskipWikiIndexCheck :servoapp:assembleArm64Release :servoview:assembleArm64Release

APK="$KATSELIN/android/apk/servoapp/build/outputs/apk/arm64Release/servoapp-arm64Release.apk"
OUT="$KATSELIN/Katselin-arm64.apk"
cp -f "$APK" "$OUT"
ls -lh "$OUT"
echo DONE
