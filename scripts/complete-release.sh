#!/bin/bash

# Complete Release Automation Script
# This script handles the entire release process if GitHub Actions fails
# Usage: ./scripts/complete-release.sh <github-run-id>

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GITEA_SERVER="https://forge.mms.name"
GITEA_OWNER="emittiv"
GITEA_REPO="fee-prop"

echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     E-Fees Complete Release Automation System            ║${NC}"
echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check required environment variables
if [ -z "$GITEA_TOKEN" ]; then
    echo -e "${RED}❌ Error: GITEA_TOKEN environment variable not set${NC}"
    echo ""
    echo "Please set your Gitea token:"
    echo "  export GITEA_TOKEN='your_token_here'"
    exit 1
fi

# Get GitHub run ID
if [ -z "$1" ]; then
    echo -e "${YELLOW}⚠️  No GitHub run ID provided${NC}"
    echo "Fetching latest workflow run..."
    RUN_ID=$(gh run list --repo newillusions/e-fees --limit 1 --json databaseId --jq '.[0].databaseId')
    echo -e "${GREEN}Using run ID: $RUN_ID${NC}"
else
    RUN_ID=$1
fi

# Get version from the workflow
echo -e "${YELLOW}📦 Fetching version information...${NC}"
VERSION=$(gh run view $RUN_ID --repo newillusions/e-fees --json headBranch --jq '.headBranch' | sed 's/refs\/tags\/v//')
if [ -z "$VERSION" ]; then
    # Fallback to package.json
    VERSION=$(node -p "require('./package.json').version")
fi
TAG="v${VERSION}"

echo -e "${GREEN}Version: $VERSION${NC}"
echo -e "${GREEN}Tag: $TAG${NC}"
echo ""

# Download artifacts from GitHub Actions
echo -e "${YELLOW}📥 Downloading artifacts from GitHub Actions...${NC}"
ARTIFACT_DIR="/tmp/e-fees-release-$VERSION"
rm -rf "$ARTIFACT_DIR"
mkdir -p "$ARTIFACT_DIR"
cd "$ARTIFACT_DIR"
gh run download $RUN_ID --repo newillusions/e-fees
cd -

echo -e "${GREEN}✅ Artifacts downloaded to: $ARTIFACT_DIR${NC}"
echo ""

# Check if Gitea release exists
echo -e "${YELLOW}🔍 Checking Gitea release status...${NC}"
RELEASE_JSON=$(curl -s -H "Authorization: token ${GITEA_TOKEN}" \
  "${GITEA_SERVER}/api/v1/repos/${GITEA_OWNER}/${GITEA_REPO}/releases/tags/${TAG}")

RELEASE_ID=$(echo "$RELEASE_JSON" | jq -r '.id // empty')

if [ -z "$RELEASE_ID" ] || [ "$RELEASE_ID" = "null" ]; then
    echo -e "${YELLOW}📝 Creating new release...${NC}"

    RELEASE_RESPONSE=$(curl -s -X POST \
      -H "Authorization: token ${GITEA_TOKEN}" \
      -H "Content-Type: application/json" \
      "${GITEA_SERVER}/api/v1/repos/${GITEA_OWNER}/${GITEA_REPO}/releases" \
      -d "{\"tag_name\": \"${TAG}\", \"name\": \"E-Fees ${TAG}\", \"body\": \"Release ${TAG}\", \"draft\": false, \"prerelease\": false}")

    RELEASE_ID=$(echo "$RELEASE_RESPONSE" | jq -r '.id // empty')

    if [ -z "$RELEASE_ID" ] || [ "$RELEASE_ID" = "null" ]; then
        echo -e "${RED}❌ Failed to create release${NC}"
        echo "Response: $RELEASE_RESPONSE"
        exit 1
    fi

    echo -e "${GREEN}✅ Release created (ID: $RELEASE_ID)${NC}"
else
    echo -e "${GREEN}✅ Release already exists (ID: $RELEASE_ID)${NC}"
fi
echo ""

# Upload artifacts function
upload_artifact() {
    local file_path=$1
    local asset_name=$2

    if [ ! -f "$file_path" ]; then
        echo -e "${YELLOW}⚠️  File not found: $file_path (skipping)${NC}"
        return 1
    fi

    echo -e "${BLUE}Uploading: $asset_name${NC}"

    curl -s -X POST \
      -H "Authorization: token ${GITEA_TOKEN}" \
      -F "attachment=@${file_path}" \
      "${GITEA_SERVER}/api/v1/repos/${GITEA_OWNER}/${GITEA_REPO}/releases/${RELEASE_ID}/assets?name=${asset_name}" \
      > /dev/null

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}  ✓ Uploaded: $asset_name${NC}"
        return 0
    else
        echo -e "${RED}  ✗ Failed: $asset_name${NC}"
        return 1
    fi
}

# Upload macOS artifacts
echo -e "${YELLOW}📤 Uploading macOS artifacts...${NC}"
upload_artifact "$ARTIFACT_DIR/macos-aarch64/dmg/E-Fees_${VERSION}_aarch64.dmg" "E-Fees_${VERSION}_aarch64.dmg"
upload_artifact "$ARTIFACT_DIR/macos-aarch64/macos/E-Fees.app.tar.gz" "E-Fees_${VERSION}_aarch64.app.tar.gz"
upload_artifact "$ARTIFACT_DIR/macos-x64/dmg/E-Fees_${VERSION}_x64.dmg" "E-Fees_${VERSION}_x64.dmg"
upload_artifact "$ARTIFACT_DIR/macos-x64/macos/E-Fees.app.tar.gz" "E-Fees_${VERSION}_x64.app.tar.gz"
echo ""

# Upload Windows artifacts
echo -e "${YELLOW}📤 Uploading Windows artifacts...${NC}"
upload_artifact "$ARTIFACT_DIR/windows/nsis/E-Fees_${VERSION}_x64-setup.exe" "E-Fees_${VERSION}_x64-setup.exe"
upload_artifact "$ARTIFACT_DIR/windows/msi/E-Fees_${VERSION}_x64_en-US.msi" "E-Fees_${VERSION}_x64.msi"
upload_artifact "$ARTIFACT_DIR/windows/nsis/E-Fees_${VERSION}_x64-setup.nsis.zip" "E-Fees_${VERSION}_x64-setup.nsis.zip" || echo -e "${YELLOW}  ⚠️  Windows updater not available${NC}"
echo ""

# Extract signatures
echo -e "${YELLOW}🔑 Extracting signatures...${NC}"
SIG_AARCH64=$(cat "$ARTIFACT_DIR/macos-aarch64/macos/E-Fees.app.tar.gz.sig" 2>/dev/null || echo "")
SIG_X64=$(cat "$ARTIFACT_DIR/macos-x64/macos/E-Fees.app.tar.gz.sig" 2>/dev/null || echo "")
SIG_WIN=$(cat "$ARTIFACT_DIR/windows/nsis/E-Fees_${VERSION}_x64-setup.nsis.zip.sig" 2>/dev/null || echo "")

if [ -n "$SIG_AARCH64" ]; then
    echo -e "${GREEN}  ✓ macOS ARM signature${NC}"
fi
if [ -n "$SIG_X64" ]; then
    echo -e "${GREEN}  ✓ macOS Intel signature${NC}"
fi
if [ -n "$SIG_WIN" ]; then
    echo -e "${GREEN}  ✓ Windows signature${NC}"
else
    echo -e "${YELLOW}  ⚠️  Windows signature not available${NC}"
fi
echo ""

# Create update.json
echo -e "${YELLOW}📝 Creating update.json manifest...${NC}"
PUB_DATE=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Build platforms JSON
PLATFORMS_JSON='{'
PLATFORMS_JSON+='"darwin-aarch64": {"signature": "'"$SIG_AARCH64"'", "url": "'"${GITEA_SERVER}/${GITEA_OWNER}/${GITEA_REPO}/releases/download/${TAG}/E-Fees_${VERSION}_aarch64.app.tar.gz"'"},'
PLATFORMS_JSON+='"darwin-x86_64": {"signature": "'"$SIG_X64"'", "url": "'"${GITEA_SERVER}/${GITEA_OWNER}/${GITEA_REPO}/releases/download/${TAG}/E-Fees_${VERSION}_x64.app.tar.gz"'"}'

if [ -n "$SIG_WIN" ]; then
    PLATFORMS_JSON+=',"windows-x86_64": {"signature": "'"$SIG_WIN"'", "url": "'"${GITEA_SERVER}/${GITEA_OWNER}/${GITEA_REPO}/releases/download/${TAG}/E-Fees_${VERSION}_x64-setup.nsis.zip"'"}'
fi

PLATFORMS_JSON+='}'

cat > update.json << EOF
{
  "version": "$VERSION",
  "notes": "Release $TAG",
  "pub_date": "$PUB_DATE",
  "platforms": $PLATFORMS_JSON
}
EOF

echo -e "${GREEN}✅ update.json created${NC}"
cat update.json | jq '.'
echo ""

# Push to repository
echo -e "${YELLOW}🚀 Pushing update.json to repository...${NC}"
git add update.json
git commit -m "chore: Update manifest for $TAG

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>"

git push origin main
git push github main

echo -e "${GREEN}✅ update.json pushed to both remotes${NC}"
echo ""

# Cleanup
echo -e "${YELLOW}🧹 Cleaning up temporary files...${NC}"
rm -rf "$ARTIFACT_DIR"
echo -e "${GREEN}✅ Cleanup complete${NC}"
echo ""

echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║              Release Complete!                            ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "Release URL: ${BLUE}${GITEA_SERVER}/${GITEA_OWNER}/${GITEA_REPO}/releases/tag/${TAG}${NC}"
echo -e "Update manifest: ${BLUE}${GITEA_SERVER}/${GITEA_OWNER}/${GITEA_REPO}/raw/branch/main/update.json${NC}"
echo ""
