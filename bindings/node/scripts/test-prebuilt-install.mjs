import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmCli = process.env.npm_execpath;
const packageOutput = process.env.CORRPLOT_NODE_PACKAGE_OUTPUT;
if (!npmCli) throw new Error("npm_execpath is not available; run this smoke test through npm");
if (!packageOutput) throw new Error("CORRPLOT_NODE_PACKAGE_OUTPUT must point to the prebuilt tarball directory");

const platformTarballs = readdirSync(packageOutput).filter((name) => name.endsWith(".tgz"));
if (platformTarballs.length !== 1) {
  throw new Error("Expected exactly one platform tarball in " + packageOutput + ", found " + platformTarballs.length);
}

const temporaryRoot = mkdtempSync(join(tmpdir(), "corrplot-node-prebuilt-install-"));
const packDirectory = join(temporaryRoot, "pack");
const installDirectory = join(temporaryRoot, "consumer");
mkdirSync(packDirectory, { recursive: true });
mkdirSync(installDirectory, { recursive: true });
const platformTarball = join(packageOutput, platformTarballs[0]);

function npm(args, cwd) {
  return execFileSync(process.execPath, [npmCli, ...args], {
    cwd,
    stdio: "inherit",
  });
}

try {
  npm(["pack", "--pack-destination", packDirectory], packageRoot);
  const baseTarballs = readdirSync(packDirectory).filter((name) => name.endsWith(".tgz"));
  assert.equal(baseTarballs.length, 1, "expected exactly one base package tarball");
  const baseTarball = join(packDirectory, baseTarballs[0]);

  npm([
    "install",
    "--ignore-scripts",
    "--omit=optional",
    "--no-audit",
    "--no-fund",
    "--prefix",
    installDirectory,
    baseTarball,
    platformTarball,
  ], temporaryRoot);

  const smoke = `const assert = require("node:assert/strict");
const api = require("@corrplot-ng/native-node");
assert.equal(api.validateMatrix({
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.5, 0.5, 1],
  kind: "correlation"
}), true);
assert.equal(api.getRendererCapabilities().capability_schema_version, 1);
const svg = api.renderSceneSvg(
  { width: 1, height: 1, cells: [], texts: [] },
  {
    width: 10,
    height: 10,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: { red: 255, green: 255, blue: 255, alpha: 255 }
  }
);
assert.match(svg, /^<svg /);
console.log("Prebuilt tarball clean-install smoke test passed.");
`;
  const smokePath = join(installDirectory, "smoke.cjs");
  writeFileSync(smokePath, smoke);
  execFileSync(process.execPath, [smokePath], {
    cwd: installDirectory,
    stdio: "inherit",
  });
} finally {
  rmSync(temporaryRoot, { recursive: true, force: true });
}
