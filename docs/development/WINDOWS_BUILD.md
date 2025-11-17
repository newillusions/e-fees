# Windows Build Instructions

## Current Status

The project is currently built and tested on macOS (Apple Silicon). Building for Windows requires additional setup.

## Why Cross-Compilation is Complex

Tauri applications cannot be easily cross-compiled from macOS to Windows because:
1. Windows-specific dependencies (Win32 APIs, WebView2, etc.)
2. Different system libraries and linking requirements
3. Platform-specific Rust compilation
4. Different bundle formats (MSI, NSIS installers vs DMG)

## Options for Windows Builds

### Option 1: Native Windows Build (Recommended)

Build on a Windows machine or VM:

```powershell
# Install Rust
winget install Rustlang.Rustup

# Install Node.js
winget install OpenJS.NodeJS

# Install Visual Studio Build Tools (required)
winget install Microsoft.VisualStudio.2022.BuildTools

# Clone repository
git clone https://git.mms.name/martin/fee-prop.git
cd fee-prop

# Install dependencies
npm install

# Build
npm run tauri:build
```

Output locations:
- `src-tauri/target/release/bundle/msi/E-Fees_0.10.0_x64_en-US.msi`
- `src-tauri/target/release/bundle/nsis/E-Fees_0.10.0_x64-setup.exe`

### Option 2: GitHub Actions (Multi-Platform CI/CD)

Use GitHub Actions to build for multiple platforms automatically:

```yaml
# .github/workflows/release.yml
name: Release Build

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [macos-latest, windows-latest, ubuntu-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install dependencies
        run: npm install
      - name: Build
        run: npm run tauri:build
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.platform }}-build
          path: src-tauri/target/release/bundle/
```

### Option 3: Cross-Compilation with cargo-xwin (Experimental)

**Warning**: This is experimental and may not work fully.

```bash
# Install cargo-xwin
cargo install cargo-xwin

# Add Windows target
rustup target add x86_64-pc-windows-msvc

# Install additional dependencies (via Homebrew)
brew install llvm mingw-w64

# Attempt build (may fail due to Tauri-specific issues)
cargo xwin build --release --target x86_64-pc-windows-msvc
```

**Known issues**:
- WebView2 dependencies not available on macOS
- Windows SDK headers required
- Linker errors with Win32 APIs

### Option 4: Docker Windows Container

Use Docker to run a Windows build environment:

```bash
# Pull Windows Server Core image
docker pull mcr.microsoft.com/windows/servercore:ltsc2022

# Run build container
docker run -it -v $(pwd):/workspace mcr.microsoft.com/windows/servercore:ltsc2022

# Inside container
cd /workspace
# Install Rust, Node.js, and build tools
# Run npm run tauri:build
```

### Option 5: Cloud Build Service

Use a cloud CI/CD service that supports Windows:
- GitHub Actions (recommended)
- Azure Pipelines
- AppVeyor
- CircleCI

## Recommended Approach

**For v0.10.0 and future releases**, we recommend:

1. **GitHub Actions** for automated multi-platform builds
2. **Native Windows build** for one-off releases
3. **Document the process** so any team member can build on Windows

## What's Needed for Windows Support

### System Requirements
- Windows 10/11
- Visual Studio 2022 Build Tools
- Rust toolchain
- Node.js 20+
- WebView2 Runtime (bundled with installer)

### Additional Tauri Config
Already configured in `tauri.conf.json`:
- ✅ Windows icon (icons/icon.ico)
- ✅ Bundle targets: "all"
- ✅ MSI and NSIS installer support

### Build Artifacts Expected
- `E-Fees_0.10.0_x64_en-US.msi` (~15-20 MB)
- `E-Fees_0.10.0_x64-setup.exe` (~15-20 MB)
- Checksums (SHA256)

## Next Steps

1. **Immediate**: Set up GitHub Actions for automated builds
2. **Short-term**: Test on a Windows machine to ensure compatibility
3. **Long-term**: Add Windows to CI/CD pipeline for every release

## Resources

- [Tauri Prerequisites (Windows)](https://tauri.app/v1/guides/getting-started/prerequisites/#windows)
- [Tauri Building (Windows)](https://tauri.app/v1/guides/building/windows)
- [cargo-xwin Documentation](https://github.com/rust-cross/cargo-xwin)
- [GitHub Actions Tauri Action](https://github.com/tauri-apps/tauri-action)

## Status

- ✅ macOS (Apple Silicon): Built and tested
- ⚠️ Windows (x64): Requires native build or CI/CD
- ⚠️ Linux: Not yet tested (should work via GitHub Actions)
