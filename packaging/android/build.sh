#!/usr/bin/env bash
# Build Wordpadx Android Tablet APK
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
WEB_DIR="$ROOT/apps/wordcraft-web"
DIST_WEB="$ROOT/dist/web"
ANDROID_DIR="$ROOT/mobile/android"
ANDROID_ASSETS="$ANDROID_DIR/app/src/main/assets/www"

SKIP_WEB=false
BUILD_TYPE="assembleDebug"

while [ $# -gt 0 ]; do
  case "$1" in
    --skip-web) SKIP_WEB=true; shift ;;
    --release) BUILD_TYPE="assembleRelease"; shift ;;
    *) shift ;;
  esac
done

echo "==> Building Wordpadx for Android Tablet..."

if [ "$SKIP_WEB" = "false" ]; then
  echo "--> Building WebAssembly bundle with Trunk..."
  (cd "$WEB_DIR" && trunk build --release)
fi

echo "--> Copying web bundle to Android assets..."
rm -rf "$ANDROID_ASSETS"
mkdir -p "$ANDROID_ASSETS"
cp -R "$DIST_WEB/." "$ANDROID_ASSETS/"

echo "--> Compiling Android app with Gradle ($BUILD_TYPE)..."
cd "$ANDROID_DIR"
if [ -f "./gradlew" ]; then
  chmod +x ./gradlew
  ./gradlew "$BUILD_TYPE"
else
  gradle "$BUILD_TYPE"
fi

echo "==> Android Tablet build complete!"
