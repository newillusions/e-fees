#!/bin/bash
# Automatically codesign the debug binary with entitlements after each build

BINARY="/Volumes/base/dev/e-fees/src-tauri/target/debug/app"
ENTITLEMENTS="/Volumes/base/dev/e-fees/src-tauri/entitlements.plist"

if [ -f "$BINARY" ]; then
    echo "Codesigning $BINARY with entitlements..."
    codesign --entitlements "$ENTITLEMENTS" -s - -f "$BINARY"
    if [ $? -eq 0 ]; then
        echo "✅ Codesigning successful"
    else
        echo "❌ Codesigning failed"
        exit 1
    fi
else
    echo "❌ Binary not found at $BINARY"
    exit 1
fi
