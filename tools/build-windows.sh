#!/usr/bin/env bash
# Cross-compile the Windows binaries on Linux/WSL and collect them in dist/.
#
#   tools/build-windows.sh
#
# Needs: rustup target x86_64-pc-windows-gnu and mingw-w64.
# Output: dist/keyflip_tsf.dll, dist/keyflip-server.exe, dist/keyflip-pipe-client.exe
set -euo pipefail

TARGET=x86_64-pc-windows-gnu
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

# Keep build output outside the repository.
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/keyflip-target-win}"

# Smaller binaries; these environment variables override the release profile
# without touching Cargo.toml.
export CARGO_PROFILE_RELEASE_LTO=fat
export CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1
export CARGO_PROFILE_RELEASE_STRIP=symbols

# Separate invocations: building both at once would unify keyflip-proto's
# `engine` feature into the DLL and bloat it with the lexicon.
cargo build --release --target "$TARGET" -p keyflip-tsf
cargo build --release --target "$TARGET" -p keyflip-server

REL="$CARGO_TARGET_DIR/$TARGET/release"
DIST="$ROOT/dist"
mkdir -p "$DIST"

for f in keyflip_tsf.dll keyflip-server.exe keyflip-pipe-client.exe; do
  if [ ! -f "$REL/$f" ]; then
    echo "missing build output: $REL/$f" >&2
    exit 1
  fi
  cp -f "$REL/$f" "$DIST/$f"
done

echo
echo "dist/ ready:"
(cd "$DIST" && sha256sum keyflip_tsf.dll keyflip-server.exe keyflip-pipe-client.exe)
