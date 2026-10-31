import { copyFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { arch, platform } from "node:process";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const manifest = resolve(packageRoot, "Cargo.toml");

if (!["x64", "arm64"].includes(arch)) {
  throw new Error(`Unsupported Node native architecture: ${arch}`);
}

execFileSync("cargo", ["build", "--release", "--manifest-path", manifest], {
  cwd: packageRoot,
  stdio: "inherit",
});

const libraryName = platform === "win32"
  ? "corrplot_ng_node.dll"
  : platform === "darwin"
    ? "libcorrplot_ng_node.dylib"
    : platform === "linux"
      ? "libcorrplot_ng_node.so"
      : null;

if (!libraryName) {
  throw new Error(`Unsupported Node native platform: ${platform}`);
}

const source = resolve(packageRoot, "target", "release", libraryName);
const destination = resolve(packageRoot, "index.node");
copyFileSync(source, destination);
process.stdout.write(`Built native addon: ${destination}\n`);
