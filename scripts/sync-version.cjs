#!/usr/bin/env node

/**
 * Version Synchronization Script
 *
 * This script ensures version numbers are consistent across:
 * - package.json (source of truth)
 * - src-tauri/Cargo.toml
 * - src-tauri/tauri.conf.json
 *
 * Usage: node scripts/sync-version.js [version]
 * Example: node scripts/sync-version.js 0.10.0
 *
 * If no version is provided, it reads from package.json and syncs to other files.
 */

const fs = require('fs');
const path = require('path');

// ANSI color codes for output
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  red: '\x1b[31m'
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function readJsonFile(filePath) {
  const content = fs.readFileSync(filePath, 'utf8');
  return JSON.parse(content);
}

function writeJsonFile(filePath, data) {
  const content = JSON.stringify(data, null, 2) + '\n';
  fs.writeFileSync(filePath, content, 'utf8');
}

function readTomlFile(filePath) {
  return fs.readFileSync(filePath, 'utf8');
}

function writeTomlFile(filePath, content) {
  fs.writeFileSync(filePath, content, 'utf8');
}

function updateCargoToml(filePath, version) {
  let content = readTomlFile(filePath);

  // Update version in [package] section
  content = content.replace(
    /^version\s*=\s*"[^"]*"/m,
    `version = "${version}"`
  );

  // Update description to remove -dev suffix
  content = content.replace(
    /^description\s*=\s*"E-Fees DEV.*"/m,
    `description = "E-Fees - Fee Proposal Management System"`
  );

  writeTomlFile(filePath, content);
  return true;
}

function updateTauriConfig(filePath, version) {
  const config = readJsonFile(filePath);

  // Update version
  config.version = version;

  // Update product name to remove DEV
  config.productName = "E-Fees";

  // Update identifier to remove .dev
  config.identifier = "com.emittiv.e-fees";

  // Update window title
  if (config.app && config.app.windows && config.app.windows[0]) {
    config.app.windows[0].title = `E-Fees v${version}`;
  }

  writeJsonFile(filePath, config);
  return true;
}

function updatePackageJson(filePath, version) {
  const pkg = readJsonFile(filePath);
  pkg.version = version;
  writeJsonFile(filePath, pkg);
  return true;
}

function main() {
  const projectRoot = path.join(__dirname, '..');

  // Determine target version
  let targetVersion = process.argv[2];

  if (!targetVersion) {
    // Read from package.json
    const packageJsonPath = path.join(projectRoot, 'package.json');
    const pkg = readJsonFile(packageJsonPath);
    targetVersion = pkg.version;
    log(`\n📦 Reading version from package.json: ${targetVersion}`, 'blue');
  } else {
    log(`\n📦 Using provided version: ${targetVersion}`, 'blue');
  }

  // Validate version format (semver)
  const versionRegex = /^\d+\.\d+\.\d+(-[a-z0-9.-]+)?$/;
  if (!versionRegex.test(targetVersion)) {
    log(`\n❌ Error: Invalid version format '${targetVersion}'`, 'red');
    log('Expected format: X.Y.Z or X.Y.Z-suffix (e.g., 1.0.0 or 1.0.0-beta.1)\n', 'yellow');
    process.exit(1);
  }

  log('\n🔄 Synchronizing version across files...\n', 'yellow');

  const files = [
    {
      name: 'package.json',
      path: path.join(projectRoot, 'package.json'),
      updater: updatePackageJson
    },
    {
      name: 'src-tauri/Cargo.toml',
      path: path.join(projectRoot, 'src-tauri', 'Cargo.toml'),
      updater: updateCargoToml
    },
    {
      name: 'src-tauri/tauri.conf.json',
      path: path.join(projectRoot, 'src-tauri', 'tauri.conf.json'),
      updater: updateTauriConfig
    }
  ];

  let successCount = 0;
  let errorCount = 0;

  for (const file of files) {
    try {
      if (!fs.existsSync(file.path)) {
        log(`⚠️  ${file.name} - File not found`, 'yellow');
        continue;
      }

      file.updater(file.path, targetVersion);
      log(`✅ ${file.name} - Updated to v${targetVersion}`, 'green');
      successCount++;
    } catch (error) {
      log(`❌ ${file.name} - Error: ${error.message}`, 'red');
      errorCount++;
    }
  }

  log('\n' + '─'.repeat(50), 'blue');
  log(`\n📊 Summary:`, 'blue');
  log(`   ✅ Successfully updated: ${successCount} files`, 'green');
  if (errorCount > 0) {
    log(`   ❌ Failed: ${errorCount} files`, 'red');
  }
  log(`   🎯 Target version: ${targetVersion}\n`, 'blue');

  if (errorCount === 0) {
    log('✨ Version synchronization complete!\n', 'green');
    log('📝 Next steps:', 'yellow');
    log('   1. Run: npm install (to update package-lock.json)', 'reset');
    log('   2. Run: cargo check (to verify Cargo.toml)', 'reset');
    log('   3. Commit changes: git add -A && git commit -m "chore: bump version to v' + targetVersion + '"', 'reset');
    log('   4. Create tag: git tag v' + targetVersion + '\n', 'reset');
    process.exit(0);
  } else {
    process.exit(1);
  }
}

// Run the script
if (require.main === module) {
  main();
}
