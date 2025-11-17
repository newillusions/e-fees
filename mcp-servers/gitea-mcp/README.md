# Gitea MCP Server

A Model Context Protocol (MCP) server for interacting with Gitea's API.

## Usage Pattern

**This MCP server is designed to be invoked on-demand via slash command, NOT as an always-loaded MCP server.**

**Why?** Token optimization - release management is infrequent, so we avoid loading this server on every Claude Code startup.

**How to use:** Type `/gitea-release` in Claude Code to access release management functionality.

## Features

- **create_release**: Create new releases on Gitea
- **upload_release_asset**: Upload binaries and files to releases
- **list_releases**: List all releases in the repository
- **get_release**: Get detailed information about a specific release
- **delete_release**: Delete a release
- **list_tags**: List all git tags in the repository

## Installation

```bash
cd mcp-servers/gitea-mcp
npm install
```

## Configuration

Set these environment variables:

```bash
export GITEA_TOKEN="your_gitea_access_token"
export GITEA_SERVER="https://git.mms.name"  # Optional, defaults to this
export GITEA_OWNER="martin"                   # Optional, defaults to this
export GITEA_REPO="fee-prop"                  # Optional, defaults to this
```

Or use `.env.local` file (already configured in the main project).

## Usage

### As MCP Server

The server runs on stdio and is configured in `.mcp-config.json`:

```json
{
  "mcpServers": {
    "gitea": {
      "command": "node",
      "args": ["mcp-servers/gitea-mcp/index.js"],
      "env": {
        "GITEA_TOKEN": "from-.env.local",
        "GITEA_SERVER": "https://git.mms.name",
        "GITEA_OWNER": "martin",
        "GITEA_REPO": "fee-prop"
      }
    }
  }
}
```

### Available Tools

#### create_release
```typescript
{
  tag_name: string;      // e.g., "v1.0.0"
  name: string;          // e.g., "E-Fees v1.0.0"
  body?: string;         // Release notes
  draft?: boolean;       // Default: false
  prerelease?: boolean;  // Default: false
}
```

#### upload_release_asset
```typescript
{
  release_id: number;    // Release ID from create_release
  file_path: string;     // Local path to file
  file_name?: string;    // Optional custom name
}
```

#### list_releases
```typescript
{
  page?: number;   // Default: 1
  limit?: number;  // Default: 10
}
```

#### get_release
```typescript
{
  tag_name: string;  // e.g., "v1.0.0"
}
```

#### delete_release
```typescript
{
  release_id: number;
}
```

#### list_tags
```typescript
{
  page?: number;   // Default: 1
  limit?: number;  // Default: 50
}
```

## Example Usage with Claude Code

```typescript
// Create a release
const result = await use_mcp_tool({
  server_name: "gitea",
  tool_name: "create_release",
  arguments: {
    tag_name: "v1.0.0",
    name: "E-Fees v1.0.0",
    body: "Initial release with bug fixes"
  }
});

// Upload a binary
await use_mcp_tool({
  server_name: "gitea",
  tool_name: "upload_release_asset",
  arguments: {
    release_id: result.release_id,
    file_path: "./releases/v1.0.0/E-Fees_1.0.0_aarch64.dmg"
  }
});

// List all releases
await use_mcp_tool({
  server_name: "gitea",
  tool_name: "list_releases",
  arguments: { limit: 5 }
});
```

## Security

- Token is read from environment variables (never hardcoded)
- Token stored in `.env.local` (excluded from git)
- All API requests use HTTPS
- Bearer token authentication

## Development

```bash
# Install dependencies
npm install

# Run server
npm start

# Test with MCP Inspector
npx @modelcontextprotocol/inspector node index.js
```

## Troubleshooting

### "GITEA_TOKEN environment variable is required"
Make sure `GITEA_TOKEN` is set in your environment or `.env.local` file.

### "Gitea API error (401)"
Your token may be expired or invalid. Generate a new one at:
https://git.mms.name/user/settings/applications

### "Failed to upload asset"
Check that:
- File path is correct and readable
- Release ID is valid
- File size is within Gitea's limits

## Links

- [MCP Specification](https://modelcontextprotocol.io/)
- [Gitea API Documentation](https://docs.gitea.com/api/1.20/)
- [E-Fees Project](https://git.mms.name/martin/fee-prop)
