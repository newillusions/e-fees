#!/bin/bash

# Script to prepare repository for public release
# Removes sensitive information and credentials

set -e

echo "🔒 Preparing repository for public release..."
echo ""

# 1. Remove database backups (contains production data)
echo "📦 Removing database backups..."
if [ -d "db backups" ]; then
    git rm -r "db backups/"
    echo "  ✓ Removed db backups/"
fi

# 2. Remove environment backup files
echo "🔐 Removing environment backups..."
if [ -f "archive/env-files/.env.backup" ]; then
    git rm "archive/env-files/.env.backup"
    echo "  ✓ Removed .env.backup"
fi

# 3. Check for any .env files that shouldn't be committed
echo "🔍 Checking for committed .env files..."
COMMITTED_ENVS=$(git ls-files | grep -E "^\.env$|^\.env\.local$|^\.env\.dev$" || true)
if [ -n "$COMMITTED_ENVS" ]; then
    echo "  ⚠️  WARNING: Found committed .env files:"
    echo "$COMMITTED_ENVS"
    echo "  These should be in .gitignore. Remove them manually if needed."
fi

# 4. List files that contain sensitive patterns (for manual review)
echo ""
echo "📋 Files containing potentially sensitive information:"
echo "   (forge.mms.name, private IPs, tokens - review manually)"
echo ""

# Gitea token pattern
echo "Files with Gitea tokens:"
git grep -l "ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f" || echo "  ✓ No hardcoded tokens found"

echo ""
echo "⚠️  Manual actions required:"
echo ""
echo "1. Regenerate Forgejo API token:"
echo "   - Go to https://forge.mms.name/user/settings/applications"
echo "   - Revoke token: ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f"
echo "   - Generate new token"
echo "   - Update GitHub secrets: GITEA_TOKEN"
echo ""
echo "2. Review and sanitize documentation files:"
echo "   - docs/development/GITHUB_ACTIONS_SETUP.md"
echo "   - .claude/commands/gitea-release.md"
echo "   - docs/development/UNRAID_GITEA_ACTIONS.md"
echo "   Replace tokens with: 'your_gitea_token_here'"
echo ""
echo "3. Consider if forge.mms.name references need to be genericized"
echo ""
echo "Run: git status"
echo "Then: git commit -m 'chore: Remove sensitive data for public release'"
