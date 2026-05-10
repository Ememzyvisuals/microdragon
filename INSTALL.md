# MICRODRAGON — Install Guide

**Every install method gives you the same result: type `microdragon` anywhere and it works.**

---

## Choose your install method

| Method | Who it's for | Time |
|---|---|---|
| [npm install -g](#method-a-npm-recommended) | Everyone — easiest | 2 minutes |
| [git clone + cargo build](#method-b-build-from-source) | Developers who want the source | 5 minutes |
| [curl installer](#method-c-curl-one-liner) | Linux/macOS quick install | 3 minutes |

---

## Method A: npm (Recommended)

Works on Windows, macOS, Linux. No Rust needed.

### 1. Install Node.js 18+

```bash
# Check if you already have it:
node --version
# Must show v18.0.0 or higher.

# If not installed:

# Windows:
winget install OpenJS.NodeJS.LTS
# OR download from: https://nodejs.org

# macOS:
brew install node
# OR download from: https://nodejs.org

# Ubuntu/Debian:
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify:
node --version   # v20.x.x or similar
```

### 2. Install MICRODRAGON

```bash
npm install -g @ememzyvisuals/microdragon
```

This automatically:
- Downloads the `microdragon` binary for your platform (Linux/macOS/Windows)
- Installs Python dependencies for all modules
- Creates `~/.local/share/microdragon/` data directory

### 3. Verify it works

```bash
microdragon --version
# microdragon v0.1.0-beta
```

### 4. Configure

```bash
microdragon setup
```

### 5. Start

```bash
microdragon
```

You will see:
```
  Microdragon v0.1.0-beta
  by EMEMZYVISUALS DIGITALS — Emmanuel Ariyo
  https://x.com/ememzyvisuals

  What do you want to achieve today?
  (type /help for examples, /pro for full CLI, /exit to quit)

  >
```

**That's it. You're running MICRODRAGON.**

---

### If `microdragon` is not found after npm install

This means npm's global bin directory is not in your PATH.

**Linux/macOS — fix:**
```bash
# Add to your ~/.bashrc or ~/.zshrc:
export PATH="$PATH:$(npm root -g)/../.bin"

# Apply immediately:
source ~/.bashrc   # or: source ~/.zshrc

# Verify:
microdragon --version
```

**Windows — fix:**
```powershell
# npm global bin is usually already in PATH after installing Node.js
# If not, find where npm installs global packages:
npm root -g
# Output example: C:\Users\YOU\AppData\Roaming\npm\node_modules

# The bin folder is one level up:
# C:\Users\YOU\AppData\Roaming\npm\
# Add this to your System Environment Variables → PATH
```

---

## Method B: Build from Source

Use this if you want to modify MICRODRAGON or if you prefer building from source.
**The result is identical to Method A** — `microdragon` command works everywhere.

### 1. Install prerequisites

```bash
# Rust (required)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow prompts, press 1 to install
source ~/.cargo/env   # or restart terminal

# Verify:
rustc --version   # rustc 1.75.0 or higher

# Python 3.9+ (likely already installed)
python3 --version

# Node.js 18+ (for WhatsApp bridge only)
node --version
```

### 2. Clone the repo

```bash
git clone https://github.com/ememzyvisuals/microdragon
cd microdragon
```

### 3. Build the Rust binary

```bash
cd core
cargo build --release
# This takes 2-5 minutes the first time (downloads and compiles deps)
# Subsequent builds take 10-30 seconds
```

The binary is now at: `core/target/release/microdragon`

### 4. Install the binary to your PATH

```bash
# Linux/macOS — choose one:

# Option 1: ~/.local/bin (no sudo needed, recommended)
mkdir -p ~/.local/bin
cp core/target/release/microdragon ~/.local/bin/microdragon

# Make sure ~/.local/bin is in PATH (add to ~/.bashrc or ~/.zshrc if needed):
export PATH="$PATH:$HOME/.local/bin"
source ~/.bashrc

# Option 2: /usr/local/bin (needs sudo, available to all users)
sudo cp core/target/release/microdragon /usr/local/bin/microdragon

# Option 3: Cargo bin (already in PATH if you have Rust)
cp core/target/release/microdragon ~/.cargo/bin/microdragon
```

```powershell
# Windows (PowerShell):
# Copy to Cargo bin (already in PATH):
Copy-Item core\target\release\microdragon.exe "$env:USERPROFILE\.cargo\bin\microdragon.exe"

# OR copy anywhere and add that folder to PATH:
mkdir "$env:USERPROFILE\bin"
Copy-Item core\target\release\microdragon.exe "$env:USERPROFILE\bin\microdragon.exe"
# Then add %USERPROFILE%\bin to your System PATH environment variable
```

### 5. Install Python modules

```bash
# From the project root (not the core/ directory):
cd ..   # back to microdragon/ root if you were in core/
pip install -r requirements.txt
playwright install chromium   # for browser automation
```

### 6. Configure

```bash
microdragon setup
```

### 7. Start

```bash
microdragon
```

**From any directory, any terminal, `microdragon` now works.**

You do NOT need to be in the project folder.
You do NOT need to use `./microdragon` or `npm run`.
Just type `microdragon` anywhere.

---

## Method C: curl One-liner

Linux and macOS only.

```bash
curl -fsSL https://raw.githubusercontent.com/ememzyvisuals/microdragon/main/scripts/install.sh | bash
```

This script:
1. Detects your OS and architecture
2. Downloads the correct pre-built binary
3. Installs it to `~/.local/bin/microdragon`
4. Adds `~/.local/bin` to PATH (in your shell config)
5. Installs Python dependencies
6. Prints `microdragon setup` as the next step

---

## After Any Install — Same Steps

Regardless of which method you used:

```bash
# Step 1: Confirm it's working
microdragon --version

# Step 2: Configure your AI provider (one-time)
microdragon setup

# Step 3: Start
microdragon
```

---

## Updating MICRODRAGON

```bash
# If you installed via npm:
npm install -g @ememzyvisuals/microdragon@latest

# If you built from source:
cd ~/path/to/microdragon
git pull
cd core && cargo build --release
cp target/release/microdragon ~/.local/bin/
```

---

## Uninstalling

```bash
# If installed via npm:
npm uninstall -g @ememzyvisuals/microdragon

# If installed manually:
rm ~/.local/bin/microdragon           # Linux/macOS
# Windows: delete microdragon.exe from wherever you copied it

# Remove data (conversations, settings):
rm -rf ~/.local/share/microdragon     # Linux/macOS
# Windows: %LOCALAPPDATA%\microdragon
```

---

## Troubleshooting

### `command not found: microdragon`

The binary is installed but not in your PATH.

```bash
# Find where it is:
npm root -g    # look in ../bin/ from this path

# Add npm bin to PATH:
export PATH="$PATH:$(npm root -g)/../.bin"
echo 'export PATH="$PATH:$(npm root -g)/../.bin"' >> ~/.bashrc
source ~/.bashrc
```

### `microdragon: /usr/bin/env: bad interpreter`

Permissions issue on Linux/macOS:
```bash
chmod +x $(which microdragon)
```

### npm postinstall says "binary not available"

Pre-built binary not yet on GitHub Releases for your platform.
It will fall back to building from source automatically if Rust is installed.
Otherwise:

```bash
# Build manually:
cd $(npm root -g)/@ememzyvisuals/microdragon
cd core && cargo build --release
cp target/release/microdragon ../npm/bin/microdragon
```

### Windows: `'microdragon' is not recognized`

```powershell
# Verify npm global bin is in PATH:
npm config get prefix
# Output: C:\Users\YOU\AppData\Roaming\npm
# This folder should be in your PATH. Check:
# System Properties → Environment Variables → PATH
```

### Python dependencies failed

```bash
pip install -r requirements.txt --break-system-packages
# If still fails:
pip3 install aiohttp pydantic python-dotenv rich httpx
```

---

*© 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo*
*[@ememzyvisuals](https://x.com/ememzyvisuals)*
