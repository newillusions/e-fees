#!/bin/bash
set -e

VERSION=$1
WEB_ROOT="/Volumes/user/www/e-fees-releases"
RELEASE_DIR="$WEB_ROOT/$VERSION"

if [ -z "$VERSION" ]; then
    echo "Usage: ./publish-release.sh <version>"
    echo "Example: ./publish-release.sh 0.10.16"
    exit 1
fi

echo "📦 Publishing e-fees v$VERSION"
echo "================================"

# Get the latest run ID for this version
echo "🔍 Finding GitHub Actions run for v$VERSION..."
RUN_ID=$(gh run list --repo newillusions/e-fees --limit 10 --json databaseId,headBranch,displayTitle,conclusion,status \
    --jq ".[] | select(.headBranch == \"v$VERSION\") | select(.conclusion == \"success\") | .databaseId" | head -1)

if [ -z "$RUN_ID" ]; then
    echo "❌ No successful run found for v$VERSION"
    exit 1
fi

echo "✅ Found run ID: $RUN_ID"

# Create temp directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

echo "📂 Downloading artifacts to $TEMP_DIR..."

# Download each artifact to its own subdirectory to avoid conflicts
mkdir -p "$TEMP_DIR/arm64" "$TEMP_DIR/x64" "$TEMP_DIR/windows" "$TEMP_DIR/manifest"

echo "⬇️  Downloading macOS arm64..."
gh run download $RUN_ID --repo newillusions/e-fees -n macos-aarch64 -D "$TEMP_DIR/arm64"

echo "⬇️  Downloading macOS x64..."
gh run download $RUN_ID --repo newillusions/e-fees -n macos-x64 -D "$TEMP_DIR/x64"

echo "⬇️  Downloading Windows..."
gh run download $RUN_ID --repo newillusions/e-fees -n windows -D "$TEMP_DIR/windows"

echo "⬇️  Downloading update manifest..."
gh run download $RUN_ID --repo newillusions/e-fees -n update-manifest -D "$TEMP_DIR/manifest"

# Create release directory
echo "📁 Creating release directory: $RELEASE_DIR"
mkdir -p "$RELEASE_DIR"

# Copy and rename files with proper architecture-specific names
echo "📋 Copying files to web server..."

# macOS arm64 - find the .app.tar.gz file dynamically (handles E-Fees or e-fees)
if [ -d "$TEMP_DIR/arm64/macos" ]; then
    mkdir -p "$RELEASE_DIR/macos-aarch64"
    MAC_ARM64_TAR=$(find "$TEMP_DIR/arm64/macos" -name "*.app.tar.gz" ! -name "*.sig" | head -1)
    MAC_ARM64_SIG=$(find "$TEMP_DIR/arm64/macos" -name "*.app.tar.gz.sig" | head -1)
    if [ -n "$MAC_ARM64_TAR" ]; then
        cp -X "$MAC_ARM64_TAR" "$RELEASE_DIR/macos-aarch64/e-fees.app.tar.gz"
        cp -X "$MAC_ARM64_SIG" "$RELEASE_DIR/macos-aarch64/e-fees.app.tar.gz.sig"
        echo "  ✓ macOS ARM64 binary copied"
    fi
fi

# macOS x64 - find the .app.tar.gz file dynamically
if [ -d "$TEMP_DIR/x64/macos" ]; then
    mkdir -p "$RELEASE_DIR/macos-x64"
    MAC_X64_TAR=$(find "$TEMP_DIR/x64/macos" -name "*.app.tar.gz" ! -name "*.sig" | head -1)
    MAC_X64_SIG=$(find "$TEMP_DIR/x64/macos" -name "*.app.tar.gz.sig" | head -1)
    if [ -n "$MAC_X64_TAR" ]; then
        cp -X "$MAC_X64_TAR" "$RELEASE_DIR/macos-x64/e-fees.app.tar.gz"
        cp -X "$MAC_X64_SIG" "$RELEASE_DIR/macos-x64/e-fees.app.tar.gz.sig"
        echo "  ✓ macOS x64 binary copied"
    fi
fi

# Windows - find the NSIS installer dynamically
if [ -d "$TEMP_DIR/windows" ]; then
    WIN_EXE=$(find "$TEMP_DIR/windows" -name "*_x64-setup.exe" | head -1)
    WIN_SIG=$(find "$TEMP_DIR/windows" -name "*_x64-setup.exe.sig" | head -1)

    if [ -n "$WIN_EXE" ]; then
        mkdir -p "$RELEASE_DIR/windows"
        cp -X "$WIN_EXE" "$RELEASE_DIR/windows/e-fees_x64-setup.exe"
        if [ -n "$WIN_SIG" ]; then
            cp -X "$WIN_SIG" "$RELEASE_DIR/windows/e-fees_x64-setup.exe.sig"
        fi
        echo "  ✓ Windows binary copied"
    fi
fi

# Get the web server base URL
WEB_BASE_URL="https://apache.mms.name/e-fees-releases"

# Read signatures from copied files (they are already base64 encoded by Tauri)
echo "🔐 Reading signatures..."
MACOS_ARM64_SIG=$(cat "$RELEASE_DIR/macos-aarch64/e-fees.app.tar.gz.sig" 2>/dev/null || echo "")
MACOS_X64_SIG=$(cat "$RELEASE_DIR/macos-x64/e-fees.app.tar.gz.sig" 2>/dev/null || echo "")
WINDOWS_SIG=$(cat "$RELEASE_DIR/windows/e-fees_x64-setup.exe.sig" 2>/dev/null || echo "")

# Generate update.json with web server URLs
echo "📝 Generating update.json..."
cat > "$WEB_ROOT/update.json" <<EOF
{
  "version": "$VERSION",
  "notes": "Release v$VERSION",
  "pub_date": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "platforms": {
    "darwin-aarch64": {
      "signature": "$MACOS_ARM64_SIG",
      "url": "$WEB_BASE_URL/$VERSION/macos-aarch64/e-fees.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "$MACOS_X64_SIG",
      "url": "$WEB_BASE_URL/$VERSION/macos-x64/e-fees.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "$WINDOWS_SIG",
      "url": "$WEB_BASE_URL/$VERSION/windows/e-fees_x64-setup.exe"
    }
  }
}
EOF

echo ""
echo "✅ Successfully published v$VERSION!"
echo ""
echo "📍 Files available at:"
echo "   $RELEASE_DIR/"
echo ""
echo "🌐 Update manifest:"
echo "   $WEB_ROOT/update.json"
echo ""
echo "📊 File listing:"
ls -lh "$RELEASE_DIR/"
