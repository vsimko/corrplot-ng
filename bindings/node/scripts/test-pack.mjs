import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmCli = process.env.npm_execpath;
if (!npmCli) throw new Error("npm_execpath is not available; run this smoke test through npm");

const temporaryRoot = mkdtempSync(join(tmpdir(), "corrplot-node-package-"));
const packDirectory = join(temporaryRoot, "pack");
const installDirectory = join(temporaryRoot, "install");
mkdirSync(packDirectory, { recursive: true });
mkdirSync(installDirectory, { recursive: true });
writeFileSync(join(installDirectory, "package.json"), JSON.stringify({ private: true }));

try {
  const packOutput = execFileSync(process.execPath, [npmCli, "pack", "--json", "--pack-destination", packDirectory], {
    cwd: packageRoot,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "inherit"],
  });
  // Lifecycle scripts (prepack) may write their own stdout before npm's JSON.
  const jsonStart = packOutput.indexOf("[");
  assert.notEqual(jsonStart, -1, "npm pack must return JSON metadata");
  const packed = JSON.parse(packOutput.slice(jsonStart))[0];
  assert.ok(packed.files.some((file) => file.path === "index.node"), "npm tarball must contain the native addon");
  const tarball = join(packDirectory, packed.filename);

  execFileSync(process.execPath, [npmCli, "install", "--ignore-scripts", "--no-audit", "--no-fund", "--prefix", installDirectory, tarball], {
    cwd: temporaryRoot,
    stdio: "inherit",
  });

  const requireFromInstall = createRequire(join(installDirectory, "package.json"));
  const api = requireFromInstall("@corrplot-ng/native-node");
  assert.equal(api.validateMatrix({
    labels: ["alpha", "beta"],
    rows: 2,
    columns: 2,
    values: [1, 0.5, 0.5, 1],
    kind: "correlation",
  }), true);
  assert.equal(api.getRendererCapabilities().capability_schema_version, 1);

  const white = { red: 255, green: 255, blue: 255, alpha: 255 };
  const svg = api.renderSceneSvg({ width: 1, height: 1, cells: [], texts: [] }, {
    width: 10,
    height: 10,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  });
  assert.match(svg, /^<svg /);
  process.stdout.write("Clean npm tarball install and native-addon smoke test passed.\n");
} finally {
  rmSync(temporaryRoot, { recursive: true, force: true });
}
