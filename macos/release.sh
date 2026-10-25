#!/usr/bin/env bash
# Build a signed, notarized, universal release of the keyflip input method.
#
#   KEYFLIP_SIGN_ID="Developer ID Application: <name> (<team>)" \
#   KEYFLIP_NOTARY_PROFILE=keyflip \
#   macos/release.sh
#
# One-time setup, done by the maintainer (needs the Apple Developer account):
#   1. Xcode > Settings > Accounts > Manage Certificates > + > Developer ID Application
#   2. xcrun notarytool store-credentials keyflip --apple-id <id> --team-id <team>
#      (asks for an app-specific password from account.apple.com)
#   3. rustup (https://rustup.rs) for the universal build and macOS 12 support
#
# Output: dist/keyflip-<version>-macos.zip, stapled, ready to publish.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

: "${KEYFLIP_SIGN_ID:?set KEYFLIP_SIGN_ID to a Developer ID Application identity}"
: "${KEYFLIP_NOTARY_PROFILE:?set KEYFLIP_NOTARY_PROFILE to a notarytool keychain profile}"
case "$KEYFLIP_SIGN_ID" in
  "Developer ID Application:"*) ;;
  *) echo "notarization needs a Developer ID Application identity, not: $KEYFLIP_SIGN_ID" >&2; exit 2 ;;
esac
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi
if ! command -v rustup >/dev/null 2>&1; then
  echo "releases need rustup for the universal build" >&2; exit 2
fi

VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' macos/Info.plist)"
APP="macos/build/Keyflip.app"
ZIP="dist/keyflip-$VERSION-macos.zip"

bash macos/build.sh --sign "$KEYFLIP_SIGN_ID"

# Every slice must load on macOS 12 (no object built for a newer system).
for ARCH in arm64 x86_64; do
  MINOS="$(vtool -arch "$ARCH" -show-build "$APP/Contents/MacOS/Keyflip" | awk '/minos/ {print $2}')"
  echo "$ARCH minos $MINOS"
  [ "$MINOS" = "12.0" ] || { echo "unexpected minimum macOS for $ARCH: $MINOS" >&2; exit 1; }
done

mkdir -p dist
rm -f "$ZIP"
ditto -c -k --keepParent "$APP" "$ZIP"

echo "== Notarize"
xcrun notarytool submit "$ZIP" --keychain-profile "$KEYFLIP_NOTARY_PROFILE" --wait

echo "== Staple"
xcrun stapler staple "$APP"
xcrun stapler validate "$APP"
rm -f "$ZIP"
ditto -c -k --keepParent "$APP" "$ZIP"
spctl --assess --type execute --verbose "$APP"

shasum -a 256 "$ZIP"
echo "Release: $ZIP"
