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
    const bin   = isWin ? "microdragon.exe" : "microdragon";
    return [
        // 1. Downloaded by postinstall (npm install -g)
        path.join(__dirname, bin),
        // 2. Built from source (git clone + cargo build)
        path.join(__dirname, "..", "..", "core", "target", "release", bin),
        // 3. PATH — user installed manually
    ].find(p => fs.existsSync(p)) || bin;
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
