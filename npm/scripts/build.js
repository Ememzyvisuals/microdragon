#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

console.log('🔨 Building npm package...');

// Ensure bin directory exists
const binDir = path.join(__dirname, '../bin');
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
  console.log('✓ Created bin directory');
}

// Ensure lib directory exists
const libDir = path.join(__dirname, '../lib');
if (!fs.existsSync(libDir)) {
  fs.mkdirSync(libDir, { recursive: true });
  console.log('✓ Created lib directory');
}

console.log('✓ Build complete');
