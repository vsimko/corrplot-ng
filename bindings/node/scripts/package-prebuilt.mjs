import { copyFileSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmCli = process.env.npm_execpath;
if (!npmCli) throw new Error("npm_execpath is not available; run this script through npm");
const packageName = process.env.CORRPLOT_NODE_PLATFORM_PACKAGE;
const outputDirectory = process.env.CORRPLOT_NODE_PACKAGE_OUTPUT;
const supported = new Set(["linux-x64-gnu", "darwin-x64", "darwin-arm64", "win32-x64-msvc"]);
if (!supported.has(packageName)) throw new Error("Unsupported platform package: " + packageName);
if (!outputDirectory) throw new Error("CORRPLOT_NODE_PACKAGE_OUTPUT must point to the artifact output directory");

const staging = mkdtempSync(join(tmpdir(), "corrplot-node-prebuilt-"));
mkdirSync(outputDirectory, { recursive: true });
try {
  copyFileSync(join(packageRoot, "index.node"), join(staging, "index.node"));
  copyFileSync(join(packageRoot, "npm", packageName, "package.json"), join(staging, "package.json"));
  execFileSync(process.execPath, [npmCli, "pack", "--json", "--pack-destination", outputDirectory], { cwd: staging, stdio: "inherit" });
} finally {
  rmSync(staging, { recursive: true, force: true });
}
