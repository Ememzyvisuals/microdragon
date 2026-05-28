#!/usr/bin/env node
// scripts/postinstall.js
// Runs after: npm install -g @ememzyvisuals/microdragon
// The binary is bundled in bin/ by the release workflow.
// If it's missing (dev install from git), try downloading it.

const { spawnSync } = require("child_process");
const https = require("https");
const fs = require("fs");
const path = require("path");

// Read version dynamically from package.json — never hardcoded
const pkg = require("../package.json");
const VERSION = pkg.version;
const REPO = "Ememzyvisuals/microdragon";
const GITHUB_RELEASES = `https://github.com/${REPO}/releases/download`;

const c = {
  green:  (s) => `\x1b[32m${s}\x1b[0m`,
  cyan:   (s) => `\x1b[36m${s}\x1b[0m`,
  yellow: (s) => `\x1b[33m${s}\x1b[0m`,
  red:    (s) => `\x1b[31m${s}\x1b[0m`,
  bold:   (s) => `\x1b[1m${s}\x1b[0m`,
  dim:    (s) => `\x1b[2m${s}\x1b[0m`,
};

// Platform → exact filename as bundled by release.yml and as found on GitHub Releases
function getPlatformBinaryName() {
  const key = `${process.platform}-${process.arch}`;
  const map = {
    "win32-x64":    "microdragon-windows-x64.exe",
    "linux-x64":    "microdragon-linux-x64",
    "linux-arm64":  "microdragon-linux-arm64",
    "darwin-x64":   "microdragon-macos-x64",
    "darwin-arm64": "microdragon-macos-arm64",
  };
  return map[key] || null;
}

function getBinDir() {
  return path.join(__dirname, "..", "bin");
}

async function main() {
  console.log();
  console.log(c.cyan(c.bold("  ⬡ MICRODRAGON Universal AI Agent")));
  console.log(c.dim(`    v${VERSION} — EMEMZYVISUALS DIGITALS`));
  console.log();

  const binDir = getBinDir();
  if (!fs.existsSync(binDir)) fs.mkdirSync(binDir, { recursive: true });

  const platformBin = getPlatformBinaryName();

  if (!platformBin) {
    console.log(c.yellow(`  ⚠ Unsupported platform: ${process.platform}-${process.arch}`));
    console.log(c.dim("    Build from source: cd core && cargo build --release"));
    return;
  }

  const bundledPath = path.join(binDir, platformBin);

  // ── Case 1: Binary already bundled by release workflow ─────────────────────
  if (fs.existsSync(bundledPath)) {
    if (process.platform !== "win32") {
      try { fs.chmodSync(bundledPath, 0o755); } catch {}
    }
    console.log(c.green(`  ✓ Binary ready: ${platformBin}`));
    printSuccess();
    return;
  }

  // ── Case 2: Try downloading from GitHub Releases ───────────────────────────
  console.log(`  ${c.cyan("▸")} Binary not bundled — downloading v${VERSION} for ${process.platform}-${process.arch}...`);
  const url = `${GITHUB_RELEASES}/v${VERSION}/${platformBin}`;
  console.log(c.dim(`    ${url}`));

  const downloaded = await downloadFile(url, bundledPath);

  if (downloaded) {
    if (process.platform !== "win32") {
      try { fs.chmodSync(bundledPath, 0o755); } catch {}
    }
    console.log(c.green(`  ✓ Downloaded: ${platformBin}`));
    printSuccess();
    return;
  }

  // ── Case 3: Try building from source ─────────────────────────────────────
  console.log(c.yellow("  ▸ Download failed — checking for Rust to build from source..."));
  const rustCheck = spawnSync("rustc", ["--version"], { stdio: "pipe" });

  if (rustCheck.status === 0) {
    const srcDir = path.join(__dirname, "..", "..", "core");
    if (fs.existsSync(path.join(srcDir, "Cargo.toml"))) {
      console.log("    Building... (3–5 minutes first time)");
      try {
        spawnSync("cargo", ["build", "--release"], { cwd: srcDir, stdio: "inherit" });
        const builtName = process.platform === "win32" ? "microdragon.exe" : "microdragon";
        const builtPath = path.join(srcDir, "target", "release", builtName);
        if (fs.existsSync(builtPath)) {
          fs.copyFileSync(builtPath, bundledPath);
          if (process.platform !== "win32") fs.chmodSync(bundledPath, 0o755);
          console.log(c.green("  ✓ Built from source"));
          printSuccess();
          return;
        }
      } catch (e) {
        console.log(c.red(`  ✗ Build failed: ${e.message}`));
      }
    }
  }

  // ── Case 4: Nothing worked ────────────────────────────────────────────────
  console.log();
  console.log(c.yellow("  ⚠ Could not install binary automatically."));
  console.log();
  console.log("  Options:");
  console.log(`  A — Install Rust (https://rustup.rs) then run:`);
  console.log(c.bold("      npm install -g @ememzyvisuals/microdragon"));
  console.log();
  console.log(`  B — Build manually:`);
  console.log(c.bold("      cd node_modules/@ememzyvisuals/microdragon"));
  if (process.platform === "win32") {
    console.log(c.bold("      cd core && cargo build --release"));
    console.log(c.bold("      copy target\\release\\microdragon.exe ..\\bin\\microdragon-windows-x64.exe"));
  } else {
    console.log(c.bold("      cd core && cargo build --release"));
    console.log(c.bold(`      cp target/release/microdragon bin/${platformBin}`));
  }
  console.log();
}

function downloadFile(url, destPath) {
  return new Promise((resolve) => {
    const tmpPath = destPath + ".tmp";

    function fetch(url, redirects) {
      if (redirects <= 0) return resolve(false);
      https.get(url, (res) => {
        if (res.statusCode === 301 || res.statusCode === 302) {
          res.resume();
          return fetch(res.headers.location, redirects - 1);
        }
        if (res.statusCode !== 200) {
          res.resume();
          return resolve(false);
        }
        const file = fs.createWriteStream(tmpPath);
        res.pipe(file);
        file.on("finish", () => {
          file.close();
          try {
            fs.renameSync(tmpPath, destPath);
            resolve(true);
          } catch {
            resolve(false);
          }
        });
        file.on("error", () => {
          try { fs.unlinkSync(tmpPath); } catch {}
          resolve(false);
        });
      }).on("error", () => {
        try { fs.unlinkSync(tmpPath); } catch {}
        resolve(false);
      }).setTimeout(60000, function () {
        this.destroy();
        resolve(false);
      });
    }

    fetch(url, 5);
  });
}

function printSuccess() {
  console.log();
  console.log(c.green(c.bold("  ✓ MICRODRAGON installed successfully!")));
  console.log();
  console.log(`  ${c.cyan("▸")} Run ${c.bold("microdragon setup")} to configure your AI provider`);
  console.log(`  ${c.cyan("▸")} Run ${c.bold("microdragon")} to launch the agent`);
  console.log(`  ${c.cyan("▸")} Run ${c.bold("microdragon --help")} for all commands`);
  console.log();
  console.log(c.dim("  github.com/Ememzyvisuals/microdragon"));
  console.log();
}

main().catch((e) => {
  // Never block npm install
  console.error(c.red("  Postinstall error:"), e.message);
});
