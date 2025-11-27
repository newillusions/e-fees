#!/usr/bin/env node

/**
 * Generate update.json manifest for Tauri updater
 * This script creates the update manifest with signatures for all platforms
 */

const fs = require('fs');
const path = require('path');

// Get version from command line or package.json
const version = process.argv[2] || require('../package.json').version;

// Configuration
const GITEA_BASE_URL = 'https://git.mms.name/martin/fee-prop/releases/download';

// Build the update manifest
function generateManifest(version, signatures = {}) {
  const tag = `v${version}`;

  return {
    version: version,
    notes: `Release ${tag}`,
    pub_date: new Date().toISOString(),
    platforms: {
      'darwin-aarch64': {
        signature: signatures['darwin-aarch64'] || '',
        url: `${GITEA_BASE_URL}/${tag}/E-Fees_${version}_aarch64.app.tar.gz`
      },
      'darwin-x86_64': {
        signature: signatures['darwin-x86_64'] || '',
        url: `${GITEA_BASE_URL}/${tag}/E-Fees_${version}_x64.app.tar.gz`
      },
      'windows-x86_64': {
        signature: signatures['windows-x86_64'] || '',
        url: `${GITEA_BASE_URL}/${tag}/E-Fees_${version}_x64-setup.nsis.zip`
      }
    }
  };
}

// Read signatures from environment or files
function getSignatures() {
  const signatures = {};

  // Try to read from environment variables (set by CI)
  if (process.env.SIG_DARWIN_AARCH64) {
    signatures['darwin-aarch64'] = process.env.SIG_DARWIN_AARCH64;
  }
  if (process.env.SIG_DARWIN_X86_64) {
    signatures['darwin-x86_64'] = process.env.SIG_DARWIN_X86_64;
  }
  if (process.env.SIG_WINDOWS_X86_64) {
    signatures['windows-x86_64'] = process.env.SIG_WINDOWS_X86_64;
  }

  return signatures;
}

// Main
const signatures = getSignatures();
const manifest = generateManifest(version, signatures);

// Write to update.json
const outputPath = path.join(__dirname, '..', 'update.json');
fs.writeFileSync(outputPath, JSON.stringify(manifest, null, 2));

console.log(`Generated update.json for version ${version}`);
console.log(`Output: ${outputPath}`);

// Print signatures status
const platforms = Object.keys(manifest.platforms);
platforms.forEach(platform => {
  const hasSig = manifest.platforms[platform].signature ? '✓' : '✗';
  console.log(`  ${hasSig} ${platform}`);
});
