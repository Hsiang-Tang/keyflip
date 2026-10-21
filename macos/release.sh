#!/usr/bin/env bash
# Build a signed, notarized release of the keyflip input method (Apple Silicon).
#
#   KEYFLIP_SIGN_ID="Developer ID Application: <name> (<team>)" \
#   KEYFLIP_NOTARY_PROFILE=keyflip \
#   macos/release.sh
#
# One-time setup, done by the maintainer (needs the Apple Developer account):
#   1. Xcode > Settings > Accounts > Manage Certificates > + > Developer ID Application
#   2. xcrun notarytool store-credentials keyflip --apple-id <id> --team-id <team>
#      (asks for an app-specific password from account.apple.com)
#
# Output: dist/keyflip-<version>-macos-arm64.zip, stapled, ready to publish.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

: "${KEYFLIP_SIGN_ID:?set KEYFLIP_SIGN_ID to a Developer ID Application identity}"
: "${KEYFLIP_NOTARY_PROFILE:?set KEYFLIP_NOTARY_PROFILE to a notarytool keychain profile}"
case "$KEYFLIP_SIGN_ID" in
  "Developer ID Application:"*) ;;
  *) echo "notarization needs a Developer ID Application identity, not: $KEYFLIP_SIGN_ID" >&2; exit 2 ;;
esac
if [ "$(uname -m)" != arm64 ]; then
  echo "releases are built on Apple Silicon" >&2; exit 2
fi

VERSION="$(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' macos/Info.plist)"
APP="macos/build/Keyflip.app"
ZIP="dist/keyflip-$VERSION-macos-arm64.zip"

bash macos/build.sh --sign "$KEYFLIP_SIGN_ID"

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
