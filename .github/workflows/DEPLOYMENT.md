# GitHub Actions Setup Guide
## How to Deploy & Publish MICRODRAGON via GitHub
### by EMEMZYVISUALS DIGITALS — Emmanuel Ariyo

---

## Overview

This is the complete pipeline:

```
git tag v0.1.0
    ↓
GitHub Actions triggers automatically
    ↓
1. Runs tests (Linux, macOS, Windows)
2. Builds Rust binary for ALL 5 platforms simultaneously
3. Creates GitHub Release with all binaries + checksums
4. Publishes @ememzyvisuals/microdragon to npm automatically
    ↓
Users install: npm install -g @ememzyvisuals/microdragon
```

---

## One-Time Setup

### Step 1: Create GitHub Repository

```bash
# On github.com: New Repository → ememzyvisuals/microdragon → Public → Create

cd /path/to/microdragon
git init
git add .
git commit -m "feat: initial MICRODRAGON release — Universal AI Agent v0.1.0"
git branch -M main
git remote add origin https://github.com/ememzyvisuals/microdragon.git
git push -u origin main
```

### Step 2: Get npm Token

```bash
# Login to npmjs.com
# Go to: npmjs.com → Profile → Access Tokens
# Click: Generate New Token → Automation (for CI)
# Copy the token (starts with npm_...)
```

### Step 3: Add npm Token to GitHub Secrets

```
GitHub → ememzyvisuals/microdragon repo
  → Settings
    → Secrets and variables
      → Actions
        → New repository secret

Name:  NPM_TOKEN
Value: npm_xxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

That's all the setup needed.

---

## How to Release a New Version

### Method A: Tag and push (recommended)

```bash
# Make your changes, commit them
git add .
git commit -m "feat: add game simulation module with GTA/NFS/MK support"

# Tag the release
git tag v0.1.0        # for initial release
# or
git tag v0.1.1        # for bug fixes (patch)
git tag v0.2.0        # for new features (minor)
git tag v1.0.0        # for breaking changes (major)

# Push everything — this triggers the full pipeline
git push origin main
git push origin v0.1.0

# GitHub Actions will:
# 1. Run tests on Linux, macOS, Windows
# 2. Build 5 platform binaries
# 3. Create GitHub Release
# 4. Publish to npm as @ememzyvisuals/microdragon@0.1.0
```

### Method B: Manual trigger from GitHub UI

```
GitHub → ememzyvisuals/microdragon
  → Actions
    → Build, Release & Publish MICRODRAGON
      → Run workflow
        → Enter version: 0.1.1
          → Run workflow
```

### Version naming convention

| Type | Version change | When to use |
|---|---|---|
| Bug fix | 0.1.0 → 0.1.1 | Small fix, no new features |
| New feature | 0.1.0 → 0.2.0 | New capability added |
| Breaking change | 0.1.0 → 1.0.0 | API changed, migration needed |
| Beta | 0.1.0-beta.1 | Pre-release test |

---

## What GitHub Actions Does (Detailed)

### CI Workflow (runs on every push/PR)

```
Trigger: push to main, or pull request
  ↓
rust-check:
  • cargo fmt -- --check   (code style)
  • cargo clippy           (lint)
  • cargo build            (compiles)
  • cargo test             (unit tests)

python-check:
  • python3 -m py_compile  (all .py files syntax check)
  • ruff check             (lint)

npm-check:
  • Validates package.json
  • node bin/microdragon.js --help (entry point check)
```

### Release Workflow (runs on git tag)

```
Trigger: git tag v* pushed
  ↓
test: Run full test suite on Ubuntu + macOS + Windows
  ↓
build: (5 parallel jobs)
  ┌─ Linux x64   (ubuntu-latest, native)
  ├─ Linux ARM64 (ubuntu-latest, cross-compile)
  ├─ macOS Intel (macos-latest, native)
  ├─ macOS ARM64 (macos-latest, native)
  └─ Windows x64 (windows-latest, native)
  ↓
release:
  • Downloads all 5 binaries
  • Creates SHA256SUMS.txt
  • Creates GitHub Release with:
    - All 5 binaries
    - Auto-generated changelog from git commits
    - Download table in release notes
  ↓
publish-npm:
  • Updates version in package.json
  • Copies all 5 binaries into npm/bin/
  • Runs: npm publish --access public --provenance
  • Verifies package is live on npmjs.com
```

---

## Monitoring Your Deployments

### Check GitHub Actions status

```
https://github.com/ememzyvisuals/microdragon/actions
```

You'll see:
- ✅ Green — all steps passed, npm published
- ❌ Red — something failed, click to see the log

### Check npm package

```bash
# Check it's live
npm view @ememzyvisuals/microdragon

# Check specific version
npm view @ememzyvisuals/microdragon@0.1.0

# Check download stats
open https://www.npmjs.com/package/@ememzyvisuals/microdragon
```

### Verify binaries in GitHub Releases

```
https://github.com/ememzyvisuals/microdragon/releases
```

---

## Troubleshooting

### "NPM_TOKEN not found" error

```
→ Go to: GitHub → Settings → Secrets → Actions
→ Make sure NPM_TOKEN exists
→ Token must start with npm_
→ Token type must be "Automation" (not classic)
```

### "403 Forbidden" from npm

```bash
# Make sure you're logged in as ememzyvisuals
npm whoami
# Should return: ememzyvisuals

# Re-authenticate
npm login --scope=@ememzyvisuals
```

### Cross-compilation fails (Linux ARM64)

```yaml
# The workflow installs gcc-aarch64-linux-gnu automatically
# If it still fails, add to the cross step:
- run: |
    echo '[target.aarch64-unknown-linux-gnu]' >> ~/.cargo/config.toml
    echo 'linker = "aarch64-linux-gnu-gcc"' >> ~/.cargo/config.toml
```

### Binary not found in npm package

```
→ Check: Actions → publish-npm job → "Prepare npm package" step
→ Verify all 5 artifact names match between build and publish jobs
→ artifact names must match exactly:
    microdragon-linux-x64
    microdragon-linux-arm64
    microdragon-macos-x64
    microdragon-macos-arm64
    microdragon-windows-x64.exe  ← note the .exe in the folder name
```

---

## Required GitHub Repository Settings

Go to: **Settings → Actions → General**

Enable:
- ✅ Allow GitHub Actions to create and approve pull requests
- ✅ Read and write permissions

Go to: **Settings → Pages** (optional, for documentation site)
- Source: Deploy from branch → main → /docs

---

## Gitignore — Important

Make sure these are in your `.gitignore`:

```
# Rust build output
core/target/

# Python
__pycache__/
*.pyc
*.pyo
.venv/
venv/

# npm
npm/node_modules/
npm/bin/microdragon-*    # binaries are downloaded by CI, not committed

# MICRODRAGON data
.local/share/microdragon/
*.db
*.key

# Environment
.env
.env.local

# OS
.DS_Store
Thumbs.db
```

---

## Complete Release Commands (copy-paste)

```bash
# Initial release
git init
git add .
git commit -m "feat: MICRODRAGON v0.1.0 — Universal AI Agent by EMEMZYVISUALS DIGITALS"
git branch -M main
git remote add origin https://github.com/ememzyvisuals/microdragon.git
git push -u origin main
git tag v0.1.0
git push origin v0.1.0
# ↑ This triggers the full build + npm publish pipeline

# Check status
open https://github.com/ememzyvisuals/microdragon/actions

# After ~10-15 minutes, verify on npm
npm view @ememzyvisuals/microdragon
```

---

## npm Package Badges (add to README)

After first publish, these badges will be live:

```markdown
[![npm version](https://img.shields.io/npm/v/@ememzyvisuals/microdragon.svg?color=00d4ff)](https://npmjs.com/package/@ememzyvisuals/microdragon)
[![npm downloads](https://img.shields.io/npm/dm/@ememzyvisuals/microdragon.svg)](https://npmjs.com/package/@ememzyvisuals/microdragon)
[![GitHub stars](https://img.shields.io/github/stars/ememzyvisuals/microdragon.svg?style=social)](https://github.com/ememzyvisuals/microdragon)
```

---

*© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo*
*Questions → [@ememzyvisuals on X](https://x.com/ememzyvisuals)*
