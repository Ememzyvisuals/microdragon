#!/usr/bin/env node
// bin/microdragon.js — MICRODRAGON CLI entry point
// Finds the correct platform binary and hands off to it.
"use strict";
const { spawnSync } = require("child_process");
const path = require("path");
const fs   = require("fs");

function findBinary() {
  const plat = process.platform;
  const arch = process.arch;
  const isWin = plat === "win32";

  // Exact filenames as bundled by the release workflow
  const platformMap = {
    "win32-x64":    "microdragon-windows-x64.exe",
    "linux-x64":    "microdragon-linux-x64",
    "linux-arm64":  "microdragon-linux-arm64",
    "darwin-x64":   "microdragon-macos-x64",
    "darwin-arm64": "microdragon-macos-arm64",
  };

  const platformBin  = platformMap[`${plat}-${arch}`];
  const genericBin   = isWin ? "microdragon.exe" : "microdragon";
  const binDir       = __dirname;

  // Search order: bundled platform-specific → generic → built from source
  const candidates = [
    platformBin && path.join(binDir, platformBin),
    path.join(binDir, genericBin),
    path.join(binDir, "..", "..", "core", "target", "release", genericBin),
    path.join(binDir, "..", "core", "target", "release", genericBin),
  ].filter(Boolean);

  return candidates.find(p => {
    try { return fs.existsSync(p); }
    catch { return false; }
  }) || genericBin;
}

const binary = findBinary();
const result = spawnSync(binary, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false,
  env: process.env,
});

if (result.error) {
  const isWin = process.platform === "win32";
  console.error("\n  🐉  MICRODRAGON binary not found.\n");
  console.error("  Quick fixes:\n");
  console.error("  A — Reinstall:");
  console.error("      npm install -g @ememzyvisuals/microdragon\n");
  console.error("  B — Build from source (needs Rust 1.75+):");
  if (isWin) {
    console.error("      cd %APPDATA%\\npm\\node_modules\\@ememzyvisuals\\microdragon\\core");
    console.error("      cargo build --release");
    console.error("      copy target\\release\\microdragon.exe ..\\bin\\microdragon-windows-x64.exe");
  } else {
    console.error("      cd $(npm root -g)/@ememzyvisuals/microdragon/core");
    const plat = process.platform === "darwin" ? "macos" : "linux";
    const arch = process.arch;
    console.error(`      cargo build --release`);
    console.error(`      cp target/release/microdragon ../bin/microdragon-${plat}-${arch}`);
  }
  console.error("\n  C — Run setup wizard:");
  console.error("      microdragon setup\n");
  process.exit(1);
}

process.exit(result.status ?? 0);
