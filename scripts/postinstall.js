#!/usr/bin/env node
const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

const binDir = path.join(__dirname, "..", "bin");
const isWin = process.platform === "win32";

if (!isWin) {
  for (const name of ["shipforge", "codetrail", "devpulse"]) {
    const binPath = path.join(binDir, name);
    if (fs.existsSync(binPath)) {
      try { fs.chmodSync(binPath, 0o755); } catch {}
    }
  }
}

console.log("devcore binaries ready.");
