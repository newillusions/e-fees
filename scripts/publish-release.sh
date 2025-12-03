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

echo "📦 Publishing E-Fees v$VERSION"
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

# macOS arm64
if [ -d "$TEMP_DIR/arm64/macos" ]; then
    # Keep original filenames to preserve signature validity
    mkdir -p "$RELEASE_DIR/macos-aarch64"
    cp -X "$TEMP_DIR/arm64/macos/E-Fees.app.tar.gz" "$RELEASE_DIR/macos-aarch64/E-Fees.app.tar.gz"
    cp -X "$TEMP_DIR/arm64/macos/E-Fees.app.tar.gz.sig" "$RELEASE_DIR/macos-aarch64/E-Fees.app.tar.gz.sig"
    echo "  ✓ macOS ARM64 binary copied"
fi

# macOS x64
if [ -d "$TEMP_DIR/x64/macos" ]; then
    # Keep original filenames to preserve signature validity
    mkdir -p "$RELEASE_DIR/macos-x64"
    cp -X "$TEMP_DIR/x64/macos/E-Fees.app.tar.gz" "$RELEASE_DIR/macos-x64/E-Fees.app.tar.gz"
    cp -X "$TEMP_DIR/x64/macos/E-Fees.app.tar.gz.sig" "$RELEASE_DIR/macos-x64/E-Fees.app.tar.gz.sig"
    echo "  ✓ macOS x64 binary copied"
fi

# Windows
if [ -d "$TEMP_DIR/windows" ]; then
    # Find the Windows zip file (should be nsis.zip)
    WIN_ZIP=$(find "$TEMP_DIR/windows" -name "*.nsis.zip" | head -1)
    WIN_SIG=$(find "$TEMP_DIR/windows" -name "*.nsis.zip.sig" | head -1)

    if [ -n "$WIN_ZIP" ]; then
        cp -X "$WIN_ZIP" "$RELEASE_DIR/E-Fees_x64-setup.nsis.zip"
        cp -X "$WIN_SIG" "$RELEASE_DIR/E-Fees_x64-setup.nsis.zip.sig"
        echo "  ✓ Windows binary copied"
    fi
fi

# Get the web server base URL
WEB_BASE_URL="https://apache.mms.name/e-fees-releases"

# Read signatures from copied files (they are already base64 encoded by Tauri)
echo "🔐 Reading signatures..."
MACOS_ARM64_SIG=$(cat "$RELEASE_DIR/macos-aarch64/E-Fees.app.tar.gz.sig" 2>/dev/null || echo "")
MACOS_X64_SIG=$(cat "$RELEASE_DIR/macos-x64/E-Fees.app.tar.gz.sig" 2>/dev/null || echo "")
WINDOWS_SIG=$(cat "$RELEASE_DIR/windows/E-Fees.app.tar.gz.sig" 2>/dev/null || echo "")

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
      "url": "$WEB_BASE_URL/$VERSION/macos-aarch64/E-Fees.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "$MACOS_X64_SIG",
      "url": "$WEB_BASE_URL/$VERSION/macos-x64/E-Fees.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "$WINDOWS_SIG",
      "url": "$WEB_BASE_URL/$VERSION/windows/E-Fees.app.tar.gz"
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
