#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
cargo build --locked --release
bundle='dist/Framewell Rust.app'
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
cp target/release/framewell "$bundle/Contents/MacOS/FramewellRust"
cat > "$bundle/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>Framewell Rust</string>
<key>CFBundleDisplayName</key><string>Framewell</string>
<key>CFBundleIdentifier</key><string>org.framewell.rust</string>
<key>CFBundleExecutable</key><string>FramewellRust</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>2</string>
<key>LSMinimumSystemVersion</key><string>15.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSScreenCaptureUsageDescription</key><string>Capture the region, window or display you choose to explain.</string>
</dict></plist>
PLIST
codesign --force --sign - "$bundle"
echo "Built $bundle (development signature; not notarized)"
