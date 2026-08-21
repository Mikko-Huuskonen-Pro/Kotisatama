#!/usr/bin/env bash
# Relink servoshell with GStreamer Android sysroot and package x64 APK.
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
test -f "$GSTREAMER_ROOT_ANDROID/x86_64/lib/libffi.a"

./mach build --target x86_64-linux-android --profile checked-release --media-stack=gstreamer --no-package -p servoshell

export SERVO_TARGET_DIR="$ENGINE/target/x86_64-linux-android/checked-release"
cd "$ENGINE/support/android/apk"
./gradlew --no-daemon -PskipWikiIndexCheck :servoapp:assembleX64Release :servoview:assembleX64Release

APK="$KATSELIN/android/apk/servoapp/build/outputs/apk/x64Release/servoapp-x64Release.apk"
cp -f "$APK" "$KATSELIN/Katselin-x64-emulator.apk"
ls -lh "$KATSELIN/Katselin-x64-emulator.apk"
echo DONE
