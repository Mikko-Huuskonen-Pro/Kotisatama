#!/usr/bin/env bash
# Rakenna Kotisatama Android APK (aarch64, servoshell EGL).
#
# Käyttö (repo juuressa):
#   ./scripts/build-android.sh
#   ./scripts/build-android.sh --skip-bootstrap
#   ./scripts/build-android.sh --skip-bootstrap --install --usb
#
# Tulos:
#   target/aarch64-linux-android/checked-release/servoapp.apk

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

TARGET="${TARGET:-aarch64-linux-android}"
PROFILE="${PROFILE:-checked-release}"

SKIP_BOOTSTRAP=0
SKIP_TESTS=0
SKIP_MEILISEARCH=0
SKIP_WHITELIST=0
INSTALL_NDK=0
DO_INSTALL=0
INSTALL_USB=0
INSTALL_EMU=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --skip-bootstrap) SKIP_BOOTSTRAP=1 ;;
    --skip-tests) SKIP_TESTS=1 ;;
    --skip-meilisearch) SKIP_MEILISEARCH=1 ;;
    --skip-whitelist-sync) SKIP_WHITELIST=1 ;;
    --install-ndk) INSTALL_NDK=1 ;;
    --install) DO_INSTALL=1 ;;
    --usb) INSTALL_USB=1 ;;
    --emulator) INSTALL_EMU=1 ;;
    --target) TARGET="$2"; shift ;;
    --profile) PROFILE="$2"; shift ;;
    -h|--help)
      echo "Usage: $0 [--skip-bootstrap] [--install] [--usb|--emulator]"
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
  shift
done

step() { printf '\n==> %s\n' "$1"; }

SERVO_NDK_VERSION="28.2.13676358"

is_servo_ndk() {
  [[ -n "${1:-}" && -f "$1/source.properties" ]] || return 1
  local major
  major="$(grep -E '^Pkg\.Revision' "$1/source.properties" | sed -E 's/.*=\s*([0-9]+).*/\1/')"
  [[ "$major" == "28" ]]
}

find_servo_ndk_in_sdk() {
  local sdk_root="$1"
  local preferred="$sdk_root/ndk/$SERVO_NDK_VERSION"
  if is_servo_ndk "$preferred"; then
    echo "$preferred"
    return 0
  fi
  local dir
  for dir in "$sdk_root/ndk"/*; do
    [[ -d "$dir" ]] || continue
    is_servo_ndk "$dir" || continue
    echo "$dir"
    return 0
  done
  return 1
}

ensure_android_sdk() {
  if [[ -n "${ANDROID_NDK_ROOT:-}" ]] && ! is_servo_ndk "$ANDROID_NDK_ROOT"; then
    echo "Warning: ANDROID_NDK_ROOT is not NDK r28; Servo requires $SERVO_NDK_VERSION" >&2
    unset ANDROID_NDK_ROOT
  fi

  if is_servo_ndk "${ANDROID_NDK_ROOT:-}"; then
    if [[ -z "${ANDROID_SDK_ROOT:-}" ]]; then
      local ndk_parent
      ndk_parent="$(dirname "$ANDROID_NDK_ROOT")"
      if [[ "$(basename "$ndk_parent")" == "ndk" ]]; then
        export ANDROID_SDK_ROOT="$(dirname "$ndk_parent")"
      fi
    fi
    echo "ANDROID_NDK_ROOT=$ANDROID_NDK_ROOT"
    [[ -n "${ANDROID_SDK_ROOT:-}" ]] && echo "ANDROID_SDK_ROOT=$ANDROID_SDK_ROOT"
    return 0
  fi

  local props="$ROOT/support/android/apk/local.properties"
  local sdk_root="${ANDROID_SDK_ROOT:-${ANDROID_HOME:-}}"
  local ndk_root=""

  if [[ -f "$props" ]]; then
    while IFS='=' read -r key value; do
      [[ "$key" =~ ^[[:space:]]*# ]] && continue
      key="${key#"${key%%[![:space:]]*}"}"
      key="${key%"${key##*[![:space:]]}"}"
      value="${value#"${value%%[![:space:]]*}"}"
      value="${value%"${value##*[![:space:]]}"}"
      case "$key" in
        sdk.dir) sdk_root="${value//\\:/:}"; sdk_root="${sdk_root//\\\\/\\}" ;;
        ndk.dir)
          value="${value//\\:/:}"
          value="${value//\\\\/\\}"
          if is_servo_ndk "$value"; then
            ndk_root="$value"
          else
            echo "Warning: local.properties ndk.dir is not NDK r28; ignoring" >&2
          fi
          ;;
      esac
    done < "$props"
  fi

  if [[ -z "$sdk_root" && -d "${HOME}/Android/Sdk" ]]; then
    sdk_root="${HOME}/Android/Sdk"
  fi

  if [[ -z "$ndk_root" && -n "$sdk_root" ]]; then
    ndk_root="$(find_servo_ndk_in_sdk "$sdk_root" || true)"
  fi

  if [[ -z "$ndk_root" && "${INSTALL_NDK:-0}" -eq 1 && -n "$sdk_root" ]]; then
    local sdkmanager=""
    for candidate in \
      "$sdk_root/cmdline-tools/latest/bin/sdkmanager" \
      "$sdk_root/tools/bin/sdkmanager"; do
      if [[ -x "$candidate" ]]; then
        sdkmanager="$candidate"
        break
      fi
    done
    if [[ -n "$sdkmanager" ]]; then
      step "Installing Android NDK r28 ($SERVO_NDK_VERSION)"
      yes | "$sdkmanager" --install "ndk;$SERVO_NDK_VERSION"
      ndk_root="$(find_servo_ndk_in_sdk "$sdk_root" || true)"
    fi
  fi

  if ! is_servo_ndk "$ndk_root"; then
    cat >&2 <<EOF
Servo requires Android NDK r28 ($SERVO_NDK_VERSION), not newer NDK versions.

Install via Android Studio (support/android/apk):
  SDK Manager -> SDK Tools -> Show Package Details -> NDK (Side by side) -> $SERVO_NDK_VERSION

Or: sdkmanager --install "ndk;$SERVO_NDK_VERSION"
EOF
    exit 1
  fi

  export ANDROID_NDK_ROOT="$ndk_root"
  if [[ -z "${ANDROID_SDK_ROOT:-}" && -n "$sdk_root" ]]; then
    export ANDROID_SDK_ROOT="$sdk_root"
  elif [[ -z "${ANDROID_SDK_ROOT:-}" ]]; then
    local ndk_parent
    ndk_parent="$(dirname "$ndk_root")"
    if [[ "$(basename "$ndk_parent")" == "ndk" ]]; then
      export ANDROID_SDK_ROOT="$(dirname "$ndk_parent")"
    fi
  fi

  echo "ANDROID_NDK_ROOT=$ANDROID_NDK_ROOT"
  [[ -n "${ANDROID_SDK_ROOT:-}" ]] && echo "ANDROID_SDK_ROOT=$ANDROID_SDK_ROOT"
}

if [[ ! -f ./mach ]]; then
  echo "Run from Kotisatama repo root (mach not found)." >&2
  exit 1
fi

ensure_android_sdk

if [[ $SKIP_BOOTSTRAP -eq 0 ]]; then
  step "mach bootstrap"
  ./mach bootstrap --yes
fi

if [[ $SKIP_TESTS -eq 0 ]]; then
  step "Kotisatama unit tests"
  cargo test -p kotisatama-pulloposti -p kotisatama-whitelist -p kotisatama-search -p kotisatama-report
fi

if [[ $SKIP_WHITELIST -eq 0 ]]; then
  CLOSED="$(cd "$ROOT/../Kotisataman-suljetut-osat" 2>/dev/null && pwd || true)"
  if [[ -n "$CLOSED" && -f "$CLOSED/valkoiset-sivut/whitelist-unified.json" ]]; then
    step "Whitelist sync"
    mkdir -p "$ROOT/index-data/cache"
    cp "$CLOSED/valkoiset-sivut/whitelist-unified.json" "$ROOT/index-data/cache/whitelist.json"
    export KOTISATAMA_WHITELIST_PATH="$ROOT/index-data/cache/whitelist.json"
  else
    echo "Warning: closed whitelist not found - using config/whitelist.json or CDN cache"
  fi
fi

if [[ -f "$ROOT/index-data/cache/whitelist.json" ]]; then
  export KOTISATAMA_WHITELIST_PATH="$ROOT/index-data/cache/whitelist.json"
elif [[ ! -f config/whitelist.json ]]; then
  echo "Warning: whitelist not found. Run sync-whitelist.ps1 or restore config/whitelist.json"
fi

if [[ $SKIP_MEILISEARCH -eq 0 ]]; then
  step "Meilisearch for Android assets"
  ./support/android/fetch-meilisearch.sh
fi

step "mach build --target $TARGET --profile $PROFILE"
./mach build --target "$TARGET" --profile "$PROFILE"

step "mach package --android --target $TARGET --profile $PROFILE"
./mach package --android --target "$TARGET" --profile "$PROFILE"

APK="$ROOT/target/$TARGET/$PROFILE/servoapp.apk"
if [[ ! -f "$APK" ]]; then
  echo "APK not found: $APK" >&2
  exit 1
fi

echo ""
echo "Android build ready: $APK"

if [[ $DO_INSTALL -eq 1 ]]; then
  step "mach install --android"
  ARGS=(install --android --target "$TARGET" --profile "$PROFILE")
  [[ $INSTALL_USB -eq 1 ]] && ARGS+=(--usb)
  [[ $INSTALL_EMU -eq 1 ]] && ARGS+=(--emulator)
  ./mach "${ARGS[@]}"
fi
