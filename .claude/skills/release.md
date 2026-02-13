---
name: release
description: Complete E-Fees release workflow - from commit to publish. Use when ready to release a new version.
---

# E-Fees Release Workflow

This skill handles the complete release process. **Run this entire workflow to completion without stopping to ask.**

## Prerequisites Check

Before starting, verify:
- [ ] All changes committed
- [ ] Tests passing (`npm run check`)
- [ ] On `main` branch

## Step 1: Version Bump

```bash
# Bump version (patch/minor/major)
npm version patch --no-git-tag-version

# Get the new version
VERSION=$(node -p "require('./package.json').version")
echo "New version: $VERSION"

# Update Cargo.toml
sed -i '' "s/^version = \".*\"/version = \"$VERSION\"/" src-tauri/Cargo.toml

# Update tauri.conf.json
sed -i '' "s/\"version\": \".*\"/\"version\": \"$VERSION\"/" src-tauri/tauri.conf.json

# Commit version bump
git add package.json package-lock.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: Bump version to $VERSION"
```

## Step 2: Tag and Push

```bash
VERSION=$(node -p "require('./package.json').version")

# Create tag and push to both remotes
git tag "v$VERSION"
git push origin main
git push origin "v$VERSION"
git push github main
git push github "v$VERSION"
```

## Step 3: Monitor Build

```bash
# Wait for build to start
sleep 10

# Check build status (repeat until completed)
gh run list --repo newillusions/e-fees --limit 1 --json status,conclusion,databaseId

# If build fails, check logs:
# gh run view <RUN_ID> --repo newillusions/e-fees --log-failed | tail -50
```

**Common build failures:**
- Missing exports: Check both `src/lib/api.ts` AND `src/lib/api/index.ts`
- Type errors: Run `npm run check` locally first

## Step 4: Mount Web Server

```bash
# Mount the www share
open "smb://martin@server.internal/www"
sleep 4

# Verify mount
ls /Volumes/www/e-fees-releases | tail -3
```

## Step 5: Publish Release

```bash
VERSION=$(node -p "require('./package.json').version")
WEB_ROOT="/Volumes/www/e-fees-releases"
RELEASE_DIR="$WEB_ROOT/$VERSION"
WEB_BASE_URL="https://apache.mms.name/e-fees-releases"

# Get the successful run ID
RUN_ID=$(gh run list --repo newillusions/e-fees --limit 5 --json databaseId,headBranch,conclusion \
    --jq ".[] | select(.headBranch == \"v$VERSION\") | select(.conclusion == \"success\") | .databaseId" | head -1)

if [ -z "$RUN_ID" ]; then
    echo "ERROR: No successful build found for v$VERSION"
    exit 1
fi

# Create temp directory and download artifacts
TEMP_DIR=$(mktemp -d)
mkdir -p "$TEMP_DIR/arm64" "$TEMP_DIR/x64" "$TEMP_DIR/windows"

gh run download $RUN_ID --repo newillusions/e-fees -n macos-aarch64 -D "$TEMP_DIR/arm64"
gh run download $RUN_ID --repo newillusions/e-fees -n macos-x64 -D "$TEMP_DIR/x64"
gh run download $RUN_ID --repo newillusions/e-fees -n windows -D "$TEMP_DIR/windows"

# Create release directories
mkdir -p "$RELEASE_DIR/macos-aarch64" "$RELEASE_DIR/macos-x64" "$RELEASE_DIR/windows"

# Copy macOS ARM64
cp $(find "$TEMP_DIR/arm64" -name "*.app.tar.gz" ! -name "*.sig") "$RELEASE_DIR/macos-aarch64/e-fees.app.tar.gz"
cp $(find "$TEMP_DIR/arm64" -name "*.app.tar.gz.sig") "$RELEASE_DIR/macos-aarch64/e-fees.app.tar.gz.sig"

# Copy macOS x64
cp $(find "$TEMP_DIR/x64" -name "*.app.tar.gz" ! -name "*.sig") "$RELEASE_DIR/macos-x64/e-fees.app.tar.gz"
cp $(find "$TEMP_DIR/x64" -name "*.app.tar.gz.sig") "$RELEASE_DIR/macos-x64/e-fees.app.tar.gz.sig"

# Copy Windows
cp $(find "$TEMP_DIR/windows" -name "*-setup.exe" ! -name "*.sig") "$RELEASE_DIR/windows/e-fees_x64-setup.exe"
cp $(find "$TEMP_DIR/windows" -name "*-setup.exe.sig") "$RELEASE_DIR/windows/e-fees_x64-setup.exe.sig" 2>/dev/null

# Cleanup
rm -rf "$TEMP_DIR"

echo "✅ Artifacts published to $RELEASE_DIR"
```

## Step 6: Update Manifest

```bash
VERSION=$(node -p "require('./package.json').version")
WEB_ROOT="/Volumes/www/e-fees-releases"
WEB_BASE_URL="https://apache.mms.name/e-fees-releases"

# Read signatures
MACOS_ARM64_SIG=$(cat "$WEB_ROOT/$VERSION/macos-aarch64/e-fees.app.tar.gz.sig")
MACOS_X64_SIG=$(cat "$WEB_ROOT/$VERSION/macos-x64/e-fees.app.tar.gz.sig")
WINDOWS_SIG=$(cat "$WEB_ROOT/$VERSION/windows/e-fees_x64-setup.exe.sig")

# Generate update.json
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

echo "✅ update.json written"
cat "$WEB_ROOT/update.json"
```

## Step 7: Verify

```bash
VERSION=$(node -p "require('./package.json').version")

# Verify web server files
echo "📁 Release files:"
ls -la /Volumes/www/e-fees-releases/$VERSION/*/

# Verify update.json is accessible
echo "🌐 Testing update endpoint:"
curl -s "https://apache.mms.name/e-fees-releases/update.json" | head -5

echo "✅ Release v$VERSION complete!"
```

## Troubleshooting

### Build fails with export error
Both `src/lib/api.ts` (legacy) AND `src/lib/api/index.ts` (modular) need to export new functions.

### Git push rejected
GitHub Actions may push manifest updates. Run:
```bash
git stash && git pull --rebase origin main && git stash pop
```

### Volume not mounting
```bash
open "smb://martin@server.internal/www"
```

### Need to re-run failed build
```bash
# Delete old tag
git tag -d v$VERSION
git push origin :refs/tags/v$VERSION
git push github :refs/tags/v$VERSION

# Fix the issue, then recreate tag
git tag v$VERSION
git push origin v$VERSION
git push github v$VERSION
```

## Key Paths

| Item | Location |
|------|----------|
| Web releases | `/Volumes/www/e-fees-releases/` |
| Update manifest | `https://apache.mms.name/e-fees-releases/update.json` |
| GitHub repo | `newillusions/e-fees` |
| Forgejo repo | `forge.mms.name/emittiv/fee-prop` |
