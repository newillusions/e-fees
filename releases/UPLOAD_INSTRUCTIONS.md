# Release Upload Instructions

## Upload to Server

To upload the release binaries to your server, use one of the methods below:

### Method 1: SCP (Secure Copy)

```bash
# Upload specific version
scp -r releases/v0.10.0 user@10.0.0.131:/path/to/releases/

# Or upload all releases
scp -r releases user@10.0.0.131:/path/to/destination/
```

### Method 2: Rsync (Recommended for updates)

```bash
# Sync releases directory
rsync -avz --progress releases/ user@10.0.0.131:/path/to/releases/

# Sync specific version
rsync -avz --progress releases/v0.10.0/ user@10.0.0.131:/path/to/releases/v0.10.0/
```

### Method 3: Git LFS (Large File Storage)

If you want to include releases in git:

```bash
# Install Git LFS
brew install git-lfs
git lfs install

# Track DMG files
git lfs track "*.dmg"
git add .gitattributes

# Commit and push
git add releases/
git commit -m "release: Add v0.10.0 binaries"
git push
```

### Method 4: Web Server

If you have a web server for downloads:

```bash
# Example: Copy to web server document root
cp releases/v0.10.0/E-Fees_0.10.0_aarch64.dmg /var/www/html/downloads/
cp releases/v0.10.0/E-Fees_0.10.0_aarch64.dmg.sha256 /var/www/html/downloads/
cp releases/v0.10.0/RELEASE_NOTES.md /var/www/html/downloads/
```

## Verify Upload

After uploading, verify the integrity:

```bash
# On the server
cd /path/to/releases/v0.10.0
shasum -a 256 -c E-Fees_0.10.0_aarch64.dmg.sha256
```

Expected output:
```
E-Fees_0.10.0_aarch64.dmg: OK
```

## Current Release

- **Version**: v0.10.0
- **DMG File**: `E-Fees_0.10.0_aarch64.dmg` (10 MB)
- **SHA256**: `7e6074873ec0121b3c85450c1cfb7105ff7f4e15c78f4b4d277a18b3e7cd7494`
- **Location**: `releases/v0.10.0/`

## Distribution Options

### Internal Distribution
- Upload to internal file server
- Share via network drive
- Distribute via email (if allowed by policy)

### Public Distribution
- GitHub Releases (if repository is public)
- CDN or cloud storage (AWS S3, DigitalOcean Spaces, etc.)
- Self-hosted web server

### Example: Upload to GitHub Releases

```bash
# Using GitHub CLI
gh release create v0.10.0 \
  releases/v0.10.0/E-Fees_0.10.0_aarch64.dmg \
  releases/v0.10.0/E-Fees_0.10.0_aarch64.dmg.sha256 \
  --title "E-Fees v0.10.0" \
  --notes-file releases/v0.10.0/RELEASE_NOTES.md
```

## Security Notes

- Always verify checksums after upload
- Use secure transfer methods (SCP, SFTP, HTTPS)
- Keep upload credentials secure
- Don't commit large binaries to git (use Git LFS if needed)
- Consider code signing and notarization for public distribution
