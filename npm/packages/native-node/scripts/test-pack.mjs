import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmCli = process.env.npm_execpath;
if (!npmCli) throw new Error("npm_execpath is not available; run this smoke test through npm");
const platformPackages = {
  "linux-x64": "linux-x64-gnu",
  "darwin-x64": "darwin-x64",
  "darwin-arm64": "darwin-arm64",
  "win32-x64": "win32-x64-msvc",
};
const platformKey = process.platform + "-" + process.arch;
const platformPackage = platformPackages[platformKey];
if (!platformPackage) throw new Error("No package fixture is defined for " + platformKey);

const temporaryRoot = mkdtempSync(join(tmpdir(), "corrplot-node-package-"));
const packDirectory = join(temporaryRoot, "pack");
const installDirectory = join(temporaryRoot, "install");
const platformDirectory = join(temporaryRoot, "platform");
mkdirSync(packDirectory, { recursive: true });
mkdirSync(installDirectory, { recursive: true });
mkdirSync(platformDirectory, { recursive: true });
writeFileSync(join(installDirectory, "package.json"), JSON.stringify({ private: true }));

function pack(directory) {
  const args = directory === packageRoot
    ? [join(packageRoot, "scripts", "pack.mjs"), "--json", "--pack-destination", packDirectory]
    : [npmCli, "pack", "--json", "--pack-destination", packDirectory];
  const output = execFileSync(process.execPath, args, {
    cwd: directory, encoding: "utf8", stdio: ["ignore", "pipe", "inherit"],
  });
  const jsonStart = output.indexOf("[");
  assert.notEqual(jsonStart, -1, "npm pack must return JSON metadata");
  return JSON.parse(output.slice(jsonStart))[0];
}

try {
  const basePackage = pack(packageRoot);
  assert.ok(!basePackage.files.some((file) => file.path === "index.node"),
    "the base tarball must not contain a host-specific native binary");

  copyFileSync(join(packageRoot, "index.node"), join(platformDirectory, "index.node"));
  copyFileSync(join(packageRoot, "..", "native-node-" + platformPackage, "package.json"), join(platformDirectory, "package.json"));
  const nativePackage = pack(platformDirectory);
  assert.ok(nativePackage.files.some((file) => file.path === "index.node"),
    "the platform tarball must contain the native addon");

  const baseTarball = join(packDirectory, basePackage.filename);
  const packedMetadata = JSON.parse(execFileSync("tar", ["-xOf", baseTarball, "package/package.json"], { encoding: "utf8" }));
  const expectedOptionalDependencies = JSON.parse(readFileSync(join(packageRoot, "platform-dependencies.json"), "utf8"));
  assert.deepEqual(packedMetadata.optionalDependencies, expectedOptionalDependencies,
    "the published base package must declare all four optional platform packages");
  const nativeTarball = join(packDirectory, nativePackage.filename);
  execFileSync(process.execPath, [npmCli, "install", "--ignore-scripts", "--omit=optional", "--no-audit", "--no-fund", "--prefix", installDirectory, baseTarball],
    { cwd: temporaryRoot, stdio: "inherit" });
  execFileSync(process.execPath, [npmCli, "install", "--ignore-scripts", "--no-audit", "--no-fund", "--prefix", installDirectory, nativeTarball],
    { cwd: temporaryRoot, stdio: "inherit" });

  const smokeScript = `const assert = require("node:assert/strict");
const { createRequire } = require("node:module");
const { join } = require("node:path");
const requireFromInstall = createRequire(join(process.argv[2], "package.json"));
const api = requireFromInstall("@corrplot-ng/native-node");
assert.equal(api.validateMatrix({ labels: ["alpha", "beta"], rows: 2, columns: 2, values: [1, 0.5, 0.5, 1], kind: "correlation" }), true);
assert.equal(api.getRendererCapabilities().capability_schema_version, 1);
const white = { red: 255, green: 255, blue: 255, alpha: 255 };
const svg = api.renderSceneSvg({ width: 1, height: 1, cells: [], texts: [] }, { width: 10, height: 10, unit: "px", fontFamily: "sans-serif", fontSizePt: 10, background: white });
assert.match(svg, /^<svg /);
console.log("Clean npm base + platform package install passed.");
`;
  const smokePath = join(temporaryRoot, "smoke.cjs");
  writeFileSync(smokePath, smokeScript);
  execFileSync(process.execPath, [smokePath, installDirectory], { stdio: "inherit" });
} finally {
  rmSync(temporaryRoot, { recursive: true, force: true });
}
