#!/usr/bin/env bash
# Copy GStreamer Android .so files into jniLibs for APK packaging.
set -euo pipefail
ABI="${1:?usage: $0 <gst-abi> <ndk-abi> <target-dir>}"
NDK_ABI="${2:?}"
TARGET_DIR="${3:?}"
: "${GSTREAMER_ROOT_ANDROID:?GSTREAMER_ROOT_ANDROID is not set}"

GST_LIB="$GSTREAMER_ROOT_ANDROID/$ABI/lib"
DEST="$TARGET_DIR/jniLibs/$NDK_ABI"
mkdir -p "$DEST"
n=0
shopt -s nullglob
for f in "$GST_LIB"/*.so; do
  cp -f "$f" "$DEST/"
  n=$((n + 1))
done
echo "Copied $n GStreamer .so files to $DEST"
test -f "$DEST/libffi.so"
test -f "$DEST/libgstreamer-1.0.so"
