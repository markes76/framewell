#!/bin/sh
# Verify that changing a bundle does not change its macOS permission identity.
set -eu
task_bundle=${1:-'dist/Framewell Rust.app'}
task_identity=${FRAMEWELL_SIGN_IDENTITY:?'Set FRAMEWELL_SIGN_IDENTITY to an existing code-signing certificate'}
if [ "$task_identity" = '-' ]; then
    echo 'A certificate is required: ad hoc identities change with each build.' >&2
    exit 1
fi
task_check=$(mktemp -d /tmp/framewell-signing-check.XXXXXX)
trap 'rm -rf "$task_check"' EXIT HUP INT TERM
ditto "$task_bundle" "$task_check/Framewell.app"
codesign --force --sign "$task_identity" "$task_check/Framewell.app"
codesign -d -r- "$task_check/Framewell.app" > "$task_check/before-all.txt" 2>&1
sed -n '/^designated =>/p' "$task_check/before-all.txt" > "$task_check/before.txt"
test -s "$task_check/before.txt"
/usr/libexec/PlistBuddy -c 'Set :CFBundleVersion 999' "$task_check/Framewell.app/Contents/Info.plist"
codesign --force --sign "$task_identity" "$task_check/Framewell.app"
codesign --verify --deep --strict "$task_check/Framewell.app"
codesign -d -r- "$task_check/Framewell.app" > "$task_check/after-all.txt" 2>&1
sed -n '/^designated =>/p' "$task_check/after-all.txt" > "$task_check/after.txt"
test -s "$task_check/after.txt"
cmp "$task_check/before.txt" "$task_check/after.txt"
echo 'PASS: bundle changes retain the same certificate-based permission identity.'
