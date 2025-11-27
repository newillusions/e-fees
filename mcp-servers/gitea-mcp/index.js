#!/usr/bin/env node

/**
 * Gitea MCP Server
 *
 * Provides Model Context Protocol tools for interacting with Gitea API:
 * - Create releases
 * - Upload release assets
 * - List releases
 * - Get release details
 * - Manage tags
 */

import { Server } from '@modelcontextprotocol/sdk/server/index.js';
import { StdioServerTransport } from '@modelcontextprotocol/sdk/server/stdio.js';
import {
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from '@modelcontextprotocol/sdk/types.js';
import fetch from 'node-fetch';

// Configuration from environment variables
const GITEA_SERVER = process.env.GITEA_SERVER || 'https://git.mms.name';
const GITEA_TOKEN = process.env.GITEA_TOKEN;
const GITEA_OWNER = process.env.GITEA_OWNER || 'martin';
const GITEA_REPO = process.env.GITEA_REPO || 'fee-prop';

if (!GITEA_TOKEN) {
  console.error('Error: GITEA_TOKEN environment variable is required');
  process.exit(1);
}

// Helper function to make Gitea API requests
async function giteaRequest(endpoint, options = {}) {
  const url = `${GITEA_SERVER}/api/v1${endpoint}`;
  const headers = {
    'Authorization': `token ${GITEA_TOKEN}`,
    'Content-Type': 'application/json',
    ...options.headers
  };

  const response = await fetch(url, {
    ...options,
    headers
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Gitea API error (${response.status}): ${error}`);
  }

  return response.json();
}

// Helper function to upload file to release
async function uploadReleaseAsset(releaseId, filePath, fileName) {
  const fs = await import('fs');
  const FormData = (await import('form-data')).default;

  const form = new FormData();
  form.append('attachment', fs.createReadStream(filePath), fileName);

  const response = await fetch(
    `${GITEA_SERVER}/api/v1/repos/${GITEA_OWNER}/${GITEA_REPO}/releases/${releaseId}/assets`,
    {
      method: 'POST',
      headers: {
        'Authorization': `token ${GITEA_TOKEN}`,
        ...form.getHeaders()
      },
      body: form
    }
  );

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to upload asset: ${error}`);
  }

  return response.json();
}

// Create MCP server
const server = new Server(
  {
    name: 'gitea-mcp-server',
    version: '1.0.0',
  },
  {
    capabilities: {
      tools: {},
    },
  }
);

// List available tools
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [
      {
        name: 'create_release',
        description: 'Create a new release on Gitea',
        inputSchema: {
          type: 'object',
          properties: {
            tag_name: {
              type: 'string',
              description: 'Git tag for the release (e.g., v1.0.0)'
            },
            name: {
              type: 'string',
              description: 'Release title'
            },
            body: {
              type: 'string',
              description: 'Release notes/description'
            },
            draft: {
              type: 'boolean',
              description: 'Create as draft release',
              default: false
            },
            prerelease: {
              type: 'boolean',
              description: 'Mark as pre-release',
              default: false
            }
          },
          required: ['tag_name', 'name']
        }
      },
      {
        name: 'upload_release_asset',
        description: 'Upload a file to a Gitea release',
        inputSchema: {
          type: 'object',
          properties: {
            release_id: {
              type: 'number',
              description: 'Release ID to upload to'
            },
            file_path: {
              type: 'string',
              description: 'Local file path to upload'
            },
            file_name: {
              type: 'string',
              description: 'Name for the uploaded file (optional, uses filename if not provided)'
            }
          },
          required: ['release_id', 'file_path']
        }
      },
      {
        name: 'list_releases',
        description: 'List all releases for the repository',
        inputSchema: {
          type: 'object',
          properties: {
            page: {
              type: 'number',
              description: 'Page number for pagination',
              default: 1
            },
            limit: {
              type: 'number',
              description: 'Number of releases per page',
              default: 10
            }
          }
        }
      },
      {
        name: 'get_release',
        description: 'Get details of a specific release',
        inputSchema: {
          type: 'object',
          properties: {
            tag_name: {
              type: 'string',
              description: 'Git tag of the release (e.g., v1.0.0)'
            }
          },
          required: ['tag_name']
        }
      },
      {
        name: 'delete_release',
        description: 'Delete a release',
        inputSchema: {
          type: 'object',
          properties: {
            release_id: {
              type: 'number',
              description: 'Release ID to delete'
            }
          },
          required: ['release_id']
        }
      },
      {
        name: 'list_tags',
        description: 'List all git tags in the repository',
        inputSchema: {
          type: 'object',
          properties: {
            page: {
              type: 'number',
              description: 'Page number for pagination',
              default: 1
            },
            limit: {
              type: 'number',
              description: 'Number of tags per page',
              default: 50
            }
          }
        }
      }
    ]
  };
});

// Handle tool calls
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  const { name, arguments: args } = request.params;

  try {
    switch (name) {
      case 'create_release': {
        const release = await giteaRequest(
          `/repos/${GITEA_OWNER}/${GITEA_REPO}/releases`,
          {
            method: 'POST',
            body: JSON.stringify({
              tag_name: args.tag_name,
              name: args.name,
              body: args.body || '',
              draft: args.draft || false,
              prerelease: args.prerelease || false
            })
          }
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                success: true,
                release_id: release.id,
                tag_name: release.tag_name,
                name: release.name,
                url: release.html_url,
                created_at: release.created_at
              }, null, 2)
            }
          ]
        };
      }

      case 'upload_release_asset': {
        const path = await import('path');
        const fileName = args.file_name || path.basename(args.file_path);

        const asset = await uploadReleaseAsset(
          args.release_id,
          args.file_path,
          fileName
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                success: true,
                asset_id: asset.id,
                name: asset.name,
                size: asset.size,
                download_url: asset.browser_download_url
              }, null, 2)
            }
          ]
        };
      }

      case 'list_releases': {
        const releases = await giteaRequest(
          `/repos/${GITEA_OWNER}/${GITEA_REPO}/releases?page=${args.page || 1}&limit=${args.limit || 10}`
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                success: true,
                count: releases.length,
                releases: releases.map(r => ({
                  id: r.id,
                  tag_name: r.tag_name,
                  name: r.name,
                  draft: r.draft,
                  prerelease: r.prerelease,
                  created_at: r.created_at,
                  assets_count: r.assets?.length || 0,
                  url: r.html_url
                }))
              }, null, 2)
            }
          ]
        };
      }

      case 'get_release': {
        const release = await giteaRequest(
          `/repos/${GITEA_OWNER}/${GITEA_REPO}/releases/tags/${args.tag_name}`
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                success: true,
                release: {
                  id: release.id,
                  tag_name: release.tag_name,
                  name: release.name,
                  body: release.body,
                  draft: release.draft,
                  prerelease: release.prerelease,
                  created_at: release.created_at,
                  assets: release.assets?.map(a => ({
                    id: a.id,
                    name: a.name,
                    size: a.size,
                    download_count: a.download_count,
                    download_url: a.browser_download_url
                  })) || [],
                  url: release.html_url
                }
              }, null, 2)
            }
          ]
        };
      }

      case 'delete_release': {
        await giteaRequest(
          `/repos/${GITEA_OWNER}/${GITEA_REPO}/releases/${args.release_id}`,
          { method: 'DELETE' }
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                success: true,
                message: `Release ${args.release_id} deleted successfully`
              }, null, 2)
            }
          ]
        };
      }

      case 'list_tags': {
        const tags = await giteaRequest(
          `/repos/${GITEA_OWNER}/${GITEA_REPO}/tags?page=${args.page || 1}&limit=${args.limit || 50}`
        );

        return {
          content: [
            {
              type: 'text',
              text: JSON.stringify({
                success: true,
                count: tags.length,
                tags: tags.map(t => ({
                  name: t.name,
                  commit_sha: t.commit?.sha,
                  message: t.message
                }))
              }, null, 2)
            }
          ]
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  } catch (error) {
    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify({
            success: false,
            error: error.message
          }, null, 2)
        }
      ],
      isError: true
    };
  }
});

// Start the server
async function main() {
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error('Gitea MCP Server running on stdio');
  console.error(`Connected to: ${GITEA_SERVER}`);
  console.error(`Repository: ${GITEA_OWNER}/${GITEA_REPO}`);
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
