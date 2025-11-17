# Gitea Release Management

Create and manage releases on the Gitea server (https://git.mms.name/martin/fee-prop).

## What You Should Do

When this command is invoked, help the user with Gitea release operations using the standalone Gitea MCP server located at `mcp-servers/gitea-mcp/`.

### Available Operations

1. **Create a new release**
2. **Upload release assets (DMG, checksums, etc.)**
3. **List existing releases**
4. **Get release details**
5. **Delete a release**
6. **List repository tags**

### How to Execute

The Gitea MCP server is NOT loaded by default (to save tokens). You need to run it directly:

```bash
# All credentials are in .env.local
cd /Volumes/base/dev/e-fees

# Run Gitea MCP operations by executing the server directly
GITEA_TOKEN="$(grep GITEA_TOKEN .env.local | cut -d= -f2 | tr -d '"')" \
GITEA_SERVER="$(grep GITEA_SERVER .env.local | cut -d= -f2 | tr -d '"')" \
GITEA_OWNER="$(grep GITEA_OWNER .env.local | cut -d= -f2 | tr -d '"')" \
GITEA_REPO="$(grep GITEA_REPO .env.local | cut -d= -f2 | tr -d '"')" \
node mcp-servers/gitea-mcp/index.js
```

### Alternative: Use the Bash Script

For common operations, use the automated script:

```bash
# Create release and upload assets
./scripts/create-gitea-release.sh 0.10.0

# The script will:
# - Read credentials from .env.local
# - Create the release on Gitea
# - Upload DMG file from releases/v{VERSION}/
# - Upload SHA256 checksum
# - Use release notes from releases/v{VERSION}/RELEASE_NOTES.md
```

### When to Use This Command

- **After building a new release** - Upload binaries to Gitea
- **Version tagging** - Create formal releases from git tags
- **Release management** - List, update, or delete releases
- **Asset uploads** - Add DMG, MSI, AppImage files to releases

### Files to Reference

- **MCP Server**: `mcp-servers/gitea-mcp/index.js` - Full MCP implementation
- **README**: `mcp-servers/gitea-mcp/README.md` - Complete documentation and examples
- **Script**: `scripts/create-gitea-release.sh` - Automated release creation
- **Credentials**: `.env.local` - Gitea token and server config (NOT in git)
- **Docs**: `docs/development/GITEA_RELEASES.md` - Comprehensive guide

### Example: Create Release for v0.11.0

```bash
# 1. Build the release first
npm run version:set 0.11.0
npm run tauri:build

# 2. Create release on Gitea using the script
./scripts/create-gitea-release.sh 0.11.0

# Or manually using the MCP server (for more control):
# - Start the server
# - Call create_release tool
# - Call upload_release_asset for each file
```

### Security Note

- Gitea token is stored in `.env.local` (excluded from git)
- Token: `ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f`
- Server: `https://git.mms.name`
- Repository: `martin/fee-prop`

**For Gitea Actions workflows:**
- Secret name: `RELEASE_TOKEN` (NOT `GITEA_TOKEN` - that prefix is reserved)
- Add at: `https://git.mms.name/martin/fee-prop/settings/secrets`

### Token Optimization

This command is implemented as a slash command (not an always-loaded MCP server) because:
- ✅ Only used occasionally (once per release)
- ✅ Saves tokens when not in use
- ✅ Still provides full MCP functionality when needed
- ✅ Credentials securely stored in `.env.local`
