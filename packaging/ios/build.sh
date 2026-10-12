#!/usr/bin/env bash
# Build Wordpadx for iPad / iOS
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
WEB_DIR="$ROOT/apps/wordcraft-web"
DIST_WEB="$ROOT/dist/web"
IOS_DIR="$ROOT/mobile/ios"
IOS_WWW="$IOS_DIR/Wordpadx/www"

SKIP_WEB=false
CONFIGURATION="Release"

while [ $# -gt 0 ]; do
  case "$1" in
    --skip-web) SKIP_WEB=true; shift ;;
    --debug) CONFIGURATION="Debug"; shift ;;
    *) shift ;;
  esac
done

echo "==> Building Wordpadx for iPad / iOS..."

if [ "$SKIP_WEB" = "false" ]; then
  echo "--> Building WebAssembly bundle with Trunk..."
  (cd "$WEB_DIR" && trunk build --release)
fi

echo "--> Copying web bundle to iOS app bundle..."
rm -rf "$IOS_WWW"
mkdir -p "$IOS_WWW"
cp -R "$DIST_WEB/." "$IOS_WWW/"

echo "--> Compiling with xcodebuild for iPad / iOS ($CONFIGURATION)..."
cd "$IOS_DIR"
xcodebuild \
  -project Wordpadx.xcodeproj \
  -scheme Wordpadx \
  -configuration "$CONFIGURATION" \
  -destination "generic/platform=iOS" \
  CODE_SIGNING_ALLOWED=NO \
  build

echo "==> iPad / iOS build complete!"
