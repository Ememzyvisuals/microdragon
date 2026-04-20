# Publishing MICRODRAGON to npm as @ememzyvisuals/microdragon

## Complete Step-by-Step Guide for Emmanuel Ariyo / EMEMZYVISUALS DIGITALS

---

## 1. Create npm Account (if not done)

```bash
# Go to: https://www.npmjs.com/signup
# Username: ememzyvisuals
# This creates the @ememzyvisuals scope automatically
```

---

## 2. Login to npm on your machine

```bash
npm login
# Enter: username, password, email
# For 2FA: use npm login --auth-type=web
```

---

## 3. Set up the package

```bash
# Navigate to the npm package directory
cd microdragon/npm

# Install the package locally to test it
npm install

# Verify the package looks correct
npm pack --dry-run
# This shows exactly what will be published
```

---

## 4. Test locally before publishing

```bash
# Create a test directory
mkdir /tmp/microdragon_test && cd /tmp/microdragon_test
npm init -y

# Install from local path
npm install /path/to/microdragon/npm

# Test it works
./node_modules/.bin/microdragon --version
# OR
npx @ememzyvisuals/microdragon --version
```

---

## 5. Build pre-compiled binaries (for fast installs)

For best user experience, build binaries for each platform and upload to GitHub Releases:

```bash
# On Linux (x64)
cd microdragon/core
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/microdragon ../npm/bin/microdragon-linux-x64

# On macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/microdragon ../npm/bin/microdragon-macos-arm64

# On macOS (Intel)
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/microdragon ../npm/bin/microdragon-macos-x64

# On Windows
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/microdragon.exe ../npm/bin/microdragon-windows-x64.exe

# Cross-compilation (on Linux, for all targets):
# cargo install cross
# cross build --release --target aarch64-apple-darwin
```

---

## 6. GitHub Actions: Auto-build on release (RECOMMENDED)

Create `.github/workflows/release.yml` to auto-build and publish:

```yaml
name: Build and Release MICRODRAGON

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: microdragon-linux-x64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: microdragon-macos-arm64
          - os: windows-latest
            target: x86_64-pc-windows-gnu
            artifact: microdragon-windows-x64.exe

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cd core && cargo build --release --target ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: core/target/${{ matrix.target }}/release/microdragon*

  publish-npm:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          registry-url: 'https://registry.npmjs.org'
      - run: |
          mkdir -p npm/bin
          # Download artifacts into npm/bin/
      - run: cd npm && npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

---

## 7. First publish

```bash
cd microdragon/npm

# IMPORTANT: Make sure version in package.json matches your release
# Version format: MAJOR.MINOR.PATCH (e.g., 0.1.0 → 0.2.0 → 1.0.0)

# Publish publicly (--access public is required for scoped packages)
npm publish --access public

# Output should say:
# + @ememzyvisuals/microdragon@0.1.0
```

---

## 8. Verify it's live

```bash
# Check on npmjs.com
open https://www.npmjs.com/package/@ememzyvisuals/microdragon

# Test install from npm
npm install -g @ememzyvisuals/microdragon
microdragon --version
microdragon setup
```

---

## 9. Releasing updates

```bash
# Bump version (pick one):
npm version patch    # 0.1.0 → 0.1.1 (bug fixes)
npm version minor    # 0.1.0 → 0.2.0 (new features)
npm version major    # 0.1.0 → 1.0.0 (breaking changes)

# Publish update
npm publish --access public
```

---

## 10. Important files to configure

### package.json - update these fields:
```json
{
  "version": "0.1.0",          ← increment each release
  "author": {
    "email": "emmanuel@ememzyvisuals.com"  ← your real email
  },
  "repository": {
    "url": "https://github.com/ememzyvisuals/microdragon.git"  ← your GitHub
  }
}
```

### .npmignore - prevent publishing unnecessary files:
```
core/target/
*.log
.env
**/__pycache__/
*.pyc
tests/
docs/
```

---

## 11. npm badge for README

After publishing, add this to your README:

```markdown
[![npm version](https://img.shields.io/npm/v/@ememzyvisuals/microdragon.svg)](https://www.npmjs.com/package/@ememzyvisuals/microdragon)
[![npm downloads](https://img.shields.io/npm/dm/@ememzyvisuals/microdragon.svg)](https://www.npmjs.com/package/@ememzyvisuals/microdragon)
```

---

## Users can then install MICRODRAGON with simply:

```bash
npm install -g @ememzyvisuals/microdragon
microdragon setup
```

**That's it. No manual Rust setup, no Python install, no config — just npm.**

---

## One-liner install via npx (no install needed):

```bash
npx @ememzyvisuals/microdragon setup
npx @ememzyvisuals/microdragon ask "hello"
```

---

## Troubleshooting

**"403 Forbidden"**: Make sure `npm login` is done and `--access public` is set
**"404 Not Found"**: Package name typo — verify `@ememzyvisuals/microdragon` is exact
**Binary not found**: The postinstall script failed — user can run `pip install -r requirements.txt` manually
**Token issues**: Create an Automation token at npmjs.com/settings/tokens for CI/CD
