#!/bin/bash

# Create Gitea Release Script
# This script creates a release on Gitea and uploads the binaries

set -e

# Configuration
GITEA_SERVER="https://git.mms.name"
REPO_OWNER="martin"
REPO_NAME="fee-prop"
VERSION="${1:-0.10.0}"
TAG="v${VERSION}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║         Gitea Release Creator for E-Fees                 ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if GITEA_TOKEN is set
if [ -z "$GITEA_TOKEN" ]; then
    echo -e "${YELLOW}⚠️  GITEA_TOKEN environment variable not set${NC}"
    echo ""
    echo "To create a Gitea token:"
    echo "1. Go to ${GITEA_SERVER}/user/settings/applications"
    echo "2. Generate New Token"
    echo "3. Select scopes: write:repository"
    echo "4. Copy the token"
    echo ""
    echo "Then run:"
    echo "  export GITEA_TOKEN='your_token_here'"
    echo "  $0 $VERSION"
    echo ""
    exit 1
fi

# Paths
RELEASE_DIR="./releases/v${VERSION}"
DMG_FILE="${RELEASE_DIR}/E-Fees_${VERSION}_aarch64.dmg"
CHECKSUM_FILE="${RELEASE_DIR}/E-Fees_${VERSION}_aarch64.dmg.sha256"
NOTES_FILE="${RELEASE_DIR}/RELEASE_NOTES.md"

# Check if files exist
echo -e "${YELLOW}📦 Checking release files...${NC}"
if [ ! -f "$DMG_FILE" ]; then
    echo -e "${RED}❌ Error: DMG file not found: $DMG_FILE${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Found: $DMG_FILE${NC}"

if [ ! -f "$CHECKSUM_FILE" ]; then
    echo -e "${YELLOW}⚠️  Checksum file not found, skipping${NC}"
fi

if [ ! -f "$NOTES_FILE" ]; then
    echo -e "${YELLOW}⚠️  Release notes not found, using default${NC}"
    RELEASE_BODY="Release v${VERSION}"
else
    RELEASE_BODY=$(cat "$NOTES_FILE")
fi

echo ""
echo -e "${YELLOW}🔨 Creating release on Gitea...${NC}"

# Create release
RELEASE_RESPONSE=$(curl -s -X POST \
  "${GITEA_SERVER}/api/v1/repos/${REPO_OWNER}/${REPO_NAME}/releases" \
  -H "Authorization: token ${GITEA_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"tag_name\": \"${TAG}\",
    \"name\": \"E-Fees ${TAG}\",
    \"body\": $(jq -Rs . <<< "$RELEASE_BODY"),
    \"draft\": false,
    \"prerelease\": false
  }")

# Check if release was created successfully
RELEASE_ID=$(echo "$RELEASE_RESPONSE" | jq -r '.id // empty')

if [ -z "$RELEASE_ID" ]; then
    echo -e "${RED}❌ Failed to create release${NC}"
    echo "Response: $RELEASE_RESPONSE"
    exit 1
fi

echo -e "${GREEN}✅ Release created with ID: $RELEASE_ID${NC}"

# Upload DMG file
echo ""
echo -e "${YELLOW}📤 Uploading DMG file...${NC}"

UPLOAD_RESPONSE=$(curl -s -X POST \
  "${GITEA_SERVER}/api/v1/repos/${REPO_OWNER}/${REPO_NAME}/releases/${RELEASE_ID}/assets" \
  -H "Authorization: token ${GITEA_TOKEN}" \
  -F "attachment=@${DMG_FILE}")

ASSET_ID=$(echo "$UPLOAD_RESPONSE" | jq -r '.id // empty')

if [ -z "$ASSET_ID" ]; then
    echo -e "${RED}❌ Failed to upload DMG${NC}"
    echo "Response: $UPLOAD_RESPONSE"
else
    echo -e "${GREEN}✅ DMG uploaded successfully${NC}"
fi

# Upload checksum if exists
if [ -f "$CHECKSUM_FILE" ]; then
    echo ""
    echo -e "${YELLOW}📤 Uploading checksum file...${NC}"

    CHECKSUM_RESPONSE=$(curl -s -X POST \
      "${GITEA_SERVER}/api/v1/repos/${REPO_OWNER}/${REPO_NAME}/releases/${RELEASE_ID}/assets" \
      -H "Authorization: token ${GITEA_TOKEN}" \
      -F "attachment=@${CHECKSUM_FILE}")

    CHECKSUM_ASSET_ID=$(echo "$CHECKSUM_RESPONSE" | jq -r '.id // empty')

    if [ -z "$CHECKSUM_ASSET_ID" ]; then
        echo -e "${YELLOW}⚠️  Failed to upload checksum (non-critical)${NC}"
    else
        echo -e "${GREEN}✅ Checksum uploaded successfully${NC}"
    fi
fi

echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                  Release Created!                         ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Release URL: ${GITEA_SERVER}/${REPO_OWNER}/${REPO_NAME}/releases/tag/${TAG}"
echo ""
