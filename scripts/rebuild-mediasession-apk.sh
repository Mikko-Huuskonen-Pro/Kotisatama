#!/usr/bin/env bash
set -euo pipefail
ENGINE=/mnt/c/Users/gigli/Kotisatama/Kotisatama
KATSELIN=/mnt/c/Users/gigli/Kotisatama/Katselin
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}"
export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$ANDROID_SDK_ROOT/ndk/28.2.13676358}"
export SERVO_TARGET_DIR="$ENGINE/target/aarch64-linux-android/checked-release"
cd "$ENGINE/support/android/apk"
./gradlew --no-daemon -PskipWikiIndexCheck :servoapp:clean :servoapp:assembleArm64Release
APK="$KATSELIN/android/apk/servoapp/build/outputs/apk/arm64Release/servoapp-arm64Release.apk"
OUT="$KATSELIN/Katselin-arm64.apk"
cp -f "$APK" "$OUT"
ls -lh "$OUT"
unzip -p "$OUT" 'classes*.dex' 2>/dev/null | strings | grep -F 'Receiver already unregistered' | head -3 || echo "(string not found in dex — check packaging)"
echo DONE
