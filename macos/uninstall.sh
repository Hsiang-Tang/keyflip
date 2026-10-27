#!/usr/bin/env bash
# Remove the keyflip input method for the current user.
set -euo pipefail
killall Keyflip 2>/dev/null || true
rm -rf "$HOME/Library/Input Methods/Keyflip.app"
rm -f "$HOME/Library/LaunchAgents/com.hsiangtang.keyflip.finish-setup.plist"
echo "Removed. Remove the 'keyflip 注音' entry from System Settings > Keyboard > Input Sources, then log out and back in."
