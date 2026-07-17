#!/usr/bin/env node

const { execSync } = require("child_process");
const fs = require("fs");
const path = require("path");
const https = require("https");
const { promisify } = require("util");

const REPO = "krauzX/devcore";
const VERSION = require("./package.json").version;

const PLATFORMS = {
  "linux-x64": { target: "x86_64-unknown-linux-gnu", ext: "tar.gz" },
  "darwin-x64": { target: "x86_64-apple-darwin", ext: "tar.gz" },
  "darwin-arm64": { target: "aarch64-apple-darwin", ext: "tar.gz" },
  "win32-x64": { target: "x86_64-pc-windows-msvc", ext: "zip" },
};

function getPlatformKey() {
  const platform = process.platform;
  const arch = process.arch;
  return `${platform}-${arch}`;
}

function getDownloadUrl(platformKey) {
  const info = PLATFORMS[platformKey];
  if (!info) {
    throw new Error(
      `Unsupported platform: ${platformKey}. Supported: ${Object.keys(PLATFORMS).join(", ")}`
    );
  }
  const tag = `v${VERSION}`;
  const filename = `devcore-${tag}-${info.target}.${info.ext}`;
  return `https://github.com/${REPO}/releases/download/${tag}/${filename}`;
}

function download(url) {
  return new Promise((resolve, reject) => {
    const follow = (url) => {
      https
        .get(url, (res) => {
          if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
            return follow(res.headers.location);
          }
          if (res.statusCode !== 200) {
            return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
          }
          const chunks = [];
          res.on("data", (chunk) => chunks.push(chunk));
          res.on("end", () => resolve(Buffer.concat(chunks)));
          res.on("error", reject);
        })
        .on("error", reject);
    };
    follow(url);
  });
}

function extract(archivePath, destDir, ext) {
  if (ext === "tar.gz") {
    execSync(`tar xzf "${archivePath}" -C "${destDir}"`, { stdio: "ignore" });
  } else if (ext === "zip") {
    execSync(`powershell -Command "Expand-Archive -Path '${archivePath}' -DestinationPath '${destDir}' -Force"`, {
      stdio: "ignore",
    });
  }
}

async function main() {
  const platformKey = getPlatformKey();
  const info = PLATFORMS[platformKey];
  const binDir = path.join(__dirname, "bin");

  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  const url = getDownloadUrl(platformKey);
  console.log(`Downloading devcore ${VERSION} for ${platformKey}...`);

  try {
    const data = await download(url);
    const tmpArchive = path.join(__dirname, `.devcore-download.${info.ext}`);
    fs.writeFileSync(tmpArchive, data);
    extract(tmpArchive, binDir, info.ext);
    fs.unlinkSync(tmpArchive);

    // Make binaries executable on unix
    if (process.platform !== "win32") {
      for (const name of ["shipforge", "codetrail", "devpulse"]) {
        const binPath = path.join(binDir, name);
        if (fs.existsSync(binPath)) {
          fs.chmodSync(binPath, 0o755);
        }
      }
    }

    console.log("devcore installed successfully!");
  } catch (err) {
    console.warn(`Warning: Could not download prebuilt binary: ${err.message}`);
    console.warn("You can build from source with: cargo build --release");
  }
}

main();
