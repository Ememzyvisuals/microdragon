#!/usr/bin/env bash
# scripts/install.sh — MICRODRAGON Universal Installer
# Linux and macOS only. Windows: use npm install -g @ememzyvisuals/microdragon

set -e

VERSION="0.1.0"
INSTALL_DIR="$HOME/.local/bin"
DATA_DIR="$HOME/.local/share/microdragon"
GITHUB="https://github.com/ememzyvisuals/microdragon"
RELEASES="$GITHUB/releases/download/v$VERSION"

GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"
BOLD="\033[1m"

ok()   { echo -e "  ${GREEN}✓${RESET} $1"; }
info() { echo -e "  ${CYAN}→${RESET} $1"; }
warn() { echo -e "  ${YELLOW}!${RESET} $1"; }
fail() { echo -e "  ${RED}✗${RESET} $1"; exit 1; }

echo ""
echo -e "  ${BOLD}Microdragon v$VERSION${RESET}"
echo -e "  ${CYAN}by EMEMZYVISUALS DIGITALS — Emmanuel Ariyo${RESET}"
echo ""

# ── Detect platform ──────────────────────────────────────────────────────────

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS-$ARCH" in
    linux-x86_64)   TARGET="microdragon-linux-x64" ;;
    linux-aarch64)  TARGET="microdragon-linux-arm64" ;;
    darwin-x86_64)  TARGET="microdragon-macos-x64" ;;
    darwin-arm64)   TARGET="microdragon-macos-arm64" ;;
    *)
        warn "Unsupported platform: $OS-$ARCH"
        warn "Attempting source build instead..."
        TARGET=""
        ;;
esac

# ── Install dir ───────────────────────────────────────────────────────────────

mkdir -p "$INSTALL_DIR"
mkdir -p "$DATA_DIR"

BINARY="$INSTALL_DIR/microdragon"

# ── Download binary ───────────────────────────────────────────────────────────

if [ -n "$TARGET" ]; then
    URL="$RELEASES/$TARGET"
    info "Downloading $TARGET..."

    if command -v curl &>/dev/null; then
        curl -fsSL "$URL" -o "$BINARY" && chmod +x "$BINARY"
    elif command -v wget &>/dev/null; then
        wget -q "$URL" -O "$BINARY" && chmod +x "$BINARY"
    else
        fail "curl or wget required. Install one and retry."
    fi

    ok "Binary downloaded to $BINARY"
else
    # Source build fallback
    if ! command -v cargo &>/dev/null; then
        info "Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    info "Building from source (1-5 minutes)..."
    TMP_DIR=$(mktemp -d)
    git clone --depth=1 "$GITHUB" "$TMP_DIR/microdragon"
    cd "$TMP_DIR/microdragon/core"
    cargo build --release
    cp target/release/microdragon "$BINARY"
    chmod +x "$BINARY"
    cd /
    rm -rf "$TMP_DIR"
    ok "Built and installed to $BINARY"
fi

# ── PATH setup ────────────────────────────────────────────────────────────────

add_to_path() {
    local rcfile="$1"
    if [ -f "$rcfile" ] && ! grep -q 'microdragon\|\.local/bin' "$rcfile"; then
        echo '' >> "$rcfile"
        echo '# MICRODRAGON' >> "$rcfile"
        echo 'export PATH="$PATH:$HOME/.local/bin"' >> "$rcfile"
        ok "Added ~/.local/bin to PATH in $rcfile"
    fi
}

if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    add_to_path "$HOME/.bashrc"
    add_to_path "$HOME/.zshrc"
    export PATH="$PATH:$HOME/.local/bin"
    warn "PATH updated. Run: source ~/.bashrc  (or restart terminal)"
fi

# ── Python deps ───────────────────────────────────────────────────────────────

if command -v pip3 &>/dev/null; then
    info "Installing Python modules (quiet)..."
    pip3 install aiohttp pydantic python-dotenv rich httpx -q \
         --break-system-packages 2>/dev/null || \
    pip3 install aiohttp pydantic python-dotenv rich httpx -q 2>/dev/null || \
    warn "pip install failed — run: pip install -r requirements.txt"
    ok "Python modules installed"
fi

# ── Playwright ────────────────────────────────────────────────────────────────

if command -v playwright &>/dev/null; then
    playwright install chromium --with-deps --quiet 2>/dev/null || true
fi

# ── Done ──────────────────────────────────────────────────────────────────────

echo ""
ok "MICRODRAGON v$VERSION installed"
echo ""
echo -e "  ${BOLD}Next steps:${RESET}"
echo ""
echo "  microdragon setup     ← configure your AI provider (do this first)"
echo "  microdragon           ← start the interactive session"
echo "  microdragon --help    ← see all commands"
echo ""
echo -e "  ${CYAN}Questions → @ememzyvisuals on X${RESET}"
echo ""
