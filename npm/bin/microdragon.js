#!/usr/bin/env node
// bin/microdragon.js
// @ememzyvisuals/microdragon — CLI entry point
// Finds the Rust binary and hands off to it directly.
"use strict";
const { spawnSync } = require("child_process");
const path = require("path");
const fs = require("fs");

function findBinary() {
    const isWin = process.platform === "win32";
    const arch  = process.arch; // x64 or arm64

    // Map to the exact filenames the CI release workflow copies into bin/
    const platformMap = {
        "win32-x64":    "microdragon-windows-x64.exe",
        "linux-x64":    "microdragon-linux-x64",
        "linux-arm64":  "microdragon-linux-arm64",
        "darwin-x64":   "microdragon-macos-x64",
        "darwin-arm64": "microdragon-macos-arm64",
    };
    const platformKey    = `${process.platform}-${arch}`;
    const platformBinary = platformMap[platformKey];

    // Generic fallback names (postinstall download path)
    const genericBin = isWin ? "microdragon.exe" : "microdragon";

    const candidates = [
        // 1. Bundled in npm package by CI (primary path)
        platformBinary && path.join(__dirname, platformBinary),
        // 2. Downloaded by postinstall to bin/ with generic name
        path.join(__dirname, genericBin),
        // 3. Built from source
        path.join(__dirname, "..", "..", "core", "target", "release", genericBin),
    ].filter(Boolean);

    return candidates.find(p => fs.existsSync(p)) || genericBin;
}

const result = spawnSync(findBinary(), process.argv.slice(2), {
    stdio: "inherit",
    windowsHide: false,
});

if (result.error) {
    const isWin = process.platform === "win32";
    console.error("\n  🐉 Microdragon binary not found.\n");
    console.error("  Fix:\n");
    console.error("  A — Reinstall (downloads binary automatically):");
    console.error("      npm install -g @ememzyvisuals/microdragon\n");
    console.error("  B — Build from source (Rust 1.75+ required):");
    if (isWin) {
        console.error("      cd core && cargo build --release");
        console.error("      copy target\\release\\microdragon.exe %USERPROFILE%\\.cargo\\bin\\");
    } else {
        console.error("      cd core && cargo build --release");
        console.error("      cp target/release/microdragon ~/.local/bin/");
    }
    console.error("\n  Then: microdragon setup\n");
    process.exit(1);
}
process.exit(result.status ?? 0);

