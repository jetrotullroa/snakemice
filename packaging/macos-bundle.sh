#!/usr/bin/env bash
# Build SnakeMice and package target/release/snakemice as a double-clickable SnakeMice.app (macOS).
# Usage (from repo root): ./packaging/macos-bundle.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo build --release

VERSION="$(
  grep -E '^[[:space:]]*version[[:space:]]*=' "$ROOT/Cargo.toml" | head -1 | sed -E 's/.*"([^"]+)".*/\1/'
)"

BIN="$ROOT/target/release/snakemice"
APP="$ROOT/target/release/SnakeMice.app"

if [[ ! -f "$BIN" ]]; then
  echo "error: missing $BIN (build failed?)" >&2
  exit 1
fi

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS"
mkdir -p "$APP/Contents/Resources"

cp "$BIN" "$APP/Contents/MacOS/snakemice"
chmod +x "$APP/Contents/MacOS/snakemice"

cat >"$APP/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>snakemice</string>
  <key>CFBundleIdentifier</key>
  <string>com.github.jetrotullroa.snakemice</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>SnakeMice</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF

echo "Built: $APP"
echo "Zip for release: (cd target/release && ditto -c -k --sequesterRsrc --keepParent SnakeMice.app SnakeMice-macos-$(uname -m).app.zip)"
