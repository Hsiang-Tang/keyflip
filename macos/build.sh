#!/usr/bin/env bash
# Build, sign and install the keyflip input method on macOS.
#
#   macos/build.sh                      build, sign ad hoc, install into ~/Library/Input Methods
#   macos/build.sh --no-install         build only (output in macos/build/)
#   macos/build.sh --sign "<identity>"  sign with a certificate and the hardened runtime
#                                       (used by macos/release.sh; implies --no-install)
#
# Needs: Rust (rustup), Xcode command line tools (swiftc, codesign). No admin rights.
set -euo pipefail

INSTALL=1
SIGN_ID=""
while [ $# -gt 0 ]; do
  case "$1" in
    --no-install) INSTALL=0 ;;
    --sign) SIGN_ID="${2:?--sign needs an identity}"; INSTALL=0; shift ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
  shift
done

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

ARCH="$(uname -m)"                       # arm64 or x86_64
case "$ARCH" in
  arm64)  RUST_TARGET=aarch64-apple-darwin ;;
  x86_64) RUST_TARGET=x86_64-apple-darwin ;;
  *) echo "unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

OUT="$ROOT/macos/build"
APP="$OUT/Keyflip.app"
LIBDIR="$OUT/rust/$RUST_TARGET/release"

export MACOSX_DEPLOYMENT_TARGET=12.0
echo "== Rust engine ($RUST_TARGET)"
if command -v rustup >/dev/null 2>&1; then rustup target add "$RUST_TARGET" >/dev/null; fi
CARGO_TARGET_DIR="$OUT/rust" cargo build --release --target "$RUST_TARGET" -p keyflip-ffi

echo "== Swift shell"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp macos/Info.plist "$APP/Contents/Info.plist"
cp -R macos/Resources/. "$APP/Contents/Resources/"
swiftc -O -swift-version 5 \
  -module-name Keyflip \
  -target "${ARCH}-apple-macos12.0" \
  -import-objc-header macos/Bridging.h \
  -Xcc "-I$ROOT/crates/keyflip-ffi/include" \
  -L "$LIBDIR" -lkeyflip_ffi \
  -framework Cocoa -framework InputMethodKit \
  macos/Sources/*.swift \
  -o "$APP/Contents/MacOS/Keyflip"

if [ -n "$SIGN_ID" ]; then
  echo "== Sign ($SIGN_ID, hardened runtime)"
  codesign --force --options runtime --timestamp --sign "$SIGN_ID" "$APP"
else
  echo "== Sign (ad hoc, local use only)"
  codesign --force --sign - "$APP"
fi
codesign --verify --strict --verbose "$APP"

if [ "$INSTALL" = 0 ]; then
  echo "Built $APP"
  exit 0
fi

DEST="$HOME/Library/Input Methods"
mkdir -p "$DEST"
# Replace a running copy cleanly.
killall Keyflip 2>/dev/null || true
rm -rf "$DEST/Keyflip.app"
cp -R "$APP" "$DEST/Keyflip.app"

cat <<MSG

Installed to $DEST/Keyflip.app

First time only: log out and back in, then open
  System Settings > Keyboard > Text Input > Input Sources > Edit... > +
  > Chinese, Traditional > keyflip 注音 > Add
and pick it from the input menu in the menu bar.

After rebuilding, the new build is picked up the next time you switch to it
(or run: killall Keyflip).
MSG
