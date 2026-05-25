#!/usr/bin/env node
// scripts/postinstall.js
// Runs after: npm install @ememzyvisuals/microdragon
// Downloads pre-built binary for the user's platform OR builds from source

const { execSync, spawnSync } = require("child_process");
const https = require("https");
const fs = require("fs");
const path = require("path");
const os = require("os");

const VERSION = "0.1.0";
const GITHUB_RELEASES = "https://github.com/ememzyvisuals/microdragon/releases/download";

// ─── Platform detection ───────────────────────────────────────────────────────

function getPlatformTarget() {
  const arch = process.arch;
  const plat = process.platform;
  const targets = {
    "linux-x64":   "microdragon-linux-x64",
    "linux-arm64": "microdragon-linux-arm64",
    "darwin-x64":  "microdragon-macos-x64",
    "darwin-arm64":"microdragon-macos-arm64",
    "win32-x64":   "microdragon-windows-x64.exe",
  };
  return targets[`${plat}-${arch}`] || null;
}

function getBinaryName() {
  return process.platform === "win32" ? "microdragon.exe" : "microdragon";
}

function getBinDir() {
  return path.join(__dirname, "..", "bin");
}

// ─── Chalk-free colored output (no deps during install) ──────────────────────

const c = {
  green:  (s) => `\x1b[32m${s}\x1b[0m`,
  cyan:   (s) => `\x1b[36m${s}\x1b[0m`,
  yellow: (s) => `\x1b[33m${s}\x1b[0m`,
  red:    (s) => `\x1b[31m${s}\x1b[0m`,
  bold:   (s) => `\x1b[1m${s}\x1b[0m`,
  dim:    (s) => `\x1b[2m${s}\x1b[0m`,
};

// ─── Main ─────────────────────────────────────────────────────────────────────

async function main() {
  console.log();
  console.log(c.cyan(c.bold("  ⬡ MICRODRAGON Universal AI Agent")));
  console.log(c.dim("    by EMEMZYVISUALS DIGITALS — Emmanuel Ariyo"));
  console.log();

  const binDir = getBinDir();
  if (!fs.existsSync(binDir)) fs.mkdirSync(binDir, { recursive: true });

  const target = getPlatformTarget();
  const binaryPath = path.join(binDir, getBinaryName());

  // Check if binary already exists (re-install scenario)
  if (fs.existsSync(binaryPath)) {
    console.log(c.green("  ✓ MICRODRAGON binary already installed"));
    await installPythonDeps();
    printSuccess();
    return;
  }

  // Try downloading pre-built binary
  if (target) {
    console.log(`  ${c.cyan("▸")} Downloading pre-built binary for ${process.platform}-${process.arch}...`);
    const downloaded = await downloadBinary(target, binaryPath);
    if (downloaded) {
      if (process.platform !== "win32") fs.chmodSync(binaryPath, 0o755);
      console.log(c.green("  ✓ Binary downloaded"));
      await installPythonDeps();
      printSuccess();
      return;
    }
  }

  // Fallback: build from source
  console.log(`  ${c.yellow("▸")} Pre-built binary not available, building from source...`);
  const built = buildFromSource(binaryPath);
  if (built) {
    await installPythonDeps();
    printSuccess();
  } else {
    printManualInstructions();
  }
}

async function downloadBinary(target, destPath) {
  const url = `${GITHUB_RELEASES}/v${VERSION}/${target}`;
  return new Promise((resolve) => {
    const tmpPath = destPath + ".tmp";
    const file = fs.createWriteStream(tmpPath);
    const request = https.get(url, { followRedirects: true }, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        file.close();
        // Follow redirect
        https.get(res.headers.location, (res2) => {
          if (res2.statusCode !== 200) { fs.unlinkSync(tmpPath); resolve(false); return; }
          res2.pipe(file);
          file.on("finish", () => {
            file.close();
            fs.renameSync(tmpPath, destPath);
            resolve(true);
          });
        }).on("error", () => { fs.unlinkSync(tmpPath); resolve(false); });
        return;
      }
      if (res.statusCode !== 200) { file.close(); fs.unlinkSync(tmpPath); resolve(false); return; }
      res.pipe(file);
      file.on("finish", () => { file.close(); fs.renameSync(tmpPath, destPath); resolve(true); });
    });
    request.on("error", () => { try { fs.unlinkSync(tmpPath); } catch {} resolve(false); });
    request.setTimeout(30000, () => { request.destroy(); resolve(false); });
  });
}

function buildFromSource(binaryPath) {
  // Check for Rust
  const rustCheck = spawnSync("rustc", ["--version"], { stdio: "pipe" });
  if (rustCheck.status !== 0) {
    console.log(c.yellow("  ℹ Rust not found — skipping source build"));
    return false;
  }

  const srcDir = path.join(__dirname, "..", "..", "core");
  if (!fs.existsSync(path.join(srcDir, "Cargo.toml"))) {
    console.log(c.yellow("  ℹ Rust source not found in package"));
    return false;
  }

  try {
    console.log("    Building... (this takes 1-3 minutes on first run)");
    execSync("cargo build --release", { cwd: srcDir, stdio: "inherit" });
    const builtBin = path.join(srcDir, "target", "release", getBinaryName());
    if (fs.existsSync(builtBin)) {
      fs.copyFileSync(builtBin, binaryPath);
      if (process.platform !== "win32") fs.chmodSync(binaryPath, 0o755);
      console.log(c.green("  ✓ Built from source"));
      return true;
    }
  } catch (e) {
    console.log(c.red(`  ✗ Build failed: ${e.message}`));
  }
  return false;
}

async function installPythonDeps() {
  // Check for Python
  const pythonCmd = process.platform === "win32" ? "python" : "python3";
  const pyCheck = spawnSync(pythonCmd, ["--version"], { stdio: "pipe" });

  if (pyCheck.status !== 0) {
    console.log(c.yellow("  ℹ Python not found — skipping Python module install"));
    console.log(c.dim("    Install Python 3.9+ for full functionality"));
    return;
  }

  const reqFile = path.join(__dirname, "..", "..", "requirements.txt");
  if (!fs.existsSync(reqFile)) return;

  console.log(`  ${c.cyan("▸")} Installing Python modules (background)...`);
  try {
    // Install quietly in background — don't block npm install
    spawnSync(pythonCmd, ["-m", "pip", "install", "-r", reqFile, "--quiet", "--no-warn-script-location"],
      { stdio: "pipe", timeout: 120000 });
    console.log(c.green("  ✓ Python modules installed"));
  } catch (e) {
    console.log(c.yellow("  ℹ Python modules not auto-installed"));
    console.log(c.dim("    Run: pip install -r requirements.txt"));
  }
}

function printSuccess() {
  console.log();
  console.log(c.green(c.bold("  ✓ MICRODRAGON installed successfully!")));
  console.log();
  console.log(`  ${c.cyan("▸")} Run ${c.bold("microdragon setup")} to configure your AI provider`);
  console.log(`  ${c.cyan("▸")} Run ${c.bold("microdragon")} to start the interactive session`);
  console.log(`  ${c.cyan("▸")} Run ${c.bold("microdragon --help")} for all commands`);
  console.log();
  console.log(c.dim("  Documentation: https://github.com/ememzyvisuals/microdragon"));
  console.log(c.dim("  © 2026 EMEMZYVISUALS DIGITALS — Emmanuel Ariyo"));
  console.log();
}

function printManualInstructions() {
  console.log();
  console.log(c.yellow("  ⚠ Automatic install incomplete. Manual steps:"));
  console.log();
  console.log("  1. Install Rust: https://rustup.rs");
  console.log(`  2. Build: ${c.bold("cd node_modules/@ememzyvisuals/microdragon/core && cargo build --release")}`);
  console.log(`  3. Copy binary: ${c.bold("cp target/release/microdragon ~/.local/bin/")}`);
  console.log(`  4. Python deps: ${c.bold("pip install -r requirements.txt")}`);
  console.log();
}

main().catch((e) => {
  console.error(c.red("  Postinstall error:"), e.message);
  // Don't exit non-zero — don't block npm install
});
