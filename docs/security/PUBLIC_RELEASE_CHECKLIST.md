# Public Release Checklist

## ✅ Completed

- [x] Removed database backups (contained production data)
- [x] Removed .env.backup files
- [x] Sanitized Gitea token from documentation
- [x] Added cleanup scripts

## ⚠️ CRITICAL: Before Pushing or Making Public

### 1. Revoke Exposed Gitea Token (REQUIRED)

The token `ba7daa3d4ab6fd3ac6825b2bf939fb58bc7ab01f` was committed to git history and **MUST** be revoked:

```bash
# 1. Go to Gitea settings
open https://git.mms.name/user/settings/applications

# 2. Find and revoke the exposed token

# 3. Generate new token with same permissions

# 4. Update GitHub secret
gh secret set GITEA_TOKEN --repo newillusions/e-fees
# (Enter new token when prompted)
```

### 2. Remove Git History (REQUIRED)

The sensitive data is still in git history. Two options:

**Option A: Fresh Start (Recommended for Public)**
```bash
# Create new repo without history
rm -rf .git
git init
git add .
git commit -m "Initial commit - cleaned for public release"
git remote add origin https://github.com/newillusions/e-fees.git
git branch -M main
git push -u origin main --force
```

**Option B: Use BFG Repo Cleaner**
```bash
# Install BFG
brew install bfg

# Clone fresh copy
cd /tmp
git clone --mirror https://github.com/newillusions/e-fees.git

# Remove sensitive files from history
bfg --delete-files "*.surql" e-fees.git
bfg --delete-files ".env.backup" e-fees.git
bfg --replace-text passwords.txt e-fees.git  # Create passwords.txt with token

# Clean up
cd e-fees.git
git reflog expire --expire=now --all
git gc --prune=now --aggressive

# Force push
git push --force
```

### 3. Final Security Check

```bash
# Search for any remaining sensitive patterns
git log -p | grep -E "(password|token|secret|10\.0\.[01]\.|th38ret3ch)" || echo "✓ Clean"

# Check repo visibility
gh repo view newillusions/e-fees --json visibility

# If private, make public ONLY after above steps complete
gh repo edit newillusions/e-fees --visibility public
```

## Acceptable Public Information

These **can** remain in the public repo:
- ✅ `git.mms.name` - Your Gitea server URL (for workflow documentation)
- ✅ Username `martin` - Public GitHub username
- ✅ Workflow files - Use GitHub Secrets, not hardcoded values
- ✅ Example/template .env files
- ✅ Documentation about architecture

## Privacy Considerations

**Consider sanitizing** (optional):
- Private server URLs → Could use `example.com` in docs
- Private IP addresses → Use `192.0.2.1` (documentation IP)
- Specific usernames → Use `your-username` placeholders

**Must keep** for functionality:
- GitHub workflow files
- Tauri configuration
- Package dependencies

## Post-Public Steps

Once public:

1. **Update README.md** - Add public-facing description
2. **Add LICENSE** - Choose appropriate license
3. **Update documentation** - Assume readers don't have private server access
4. **Monitor for leaked secrets** - Use GitHub secret scanning

## Testing Before Public

```bash
# Check what would be public
git log --oneline | head -20

# Verify no sensitive files
git ls-files | grep -E "\.env$|secret|password|\.surql$" || echo "✓ Clean"

# Final repo size (should be smaller after cleanup)
du -sh .git
```

## If Secrets Leaked

If you accidentally push secrets:

1. **Immediately** rotate/revoke them
2. Consider them compromised permanently
3. Use `git-secrets` tool to prevent future leaks
4. Enable GitHub secret scanning

## Questions?

- Will this be used by others? Consider documentation improvements
- Need Windows builds? Currently no updater (see RELEASE_AUTOMATION.md)
- Want GitHub Sponsors? Add funding.yml after public
