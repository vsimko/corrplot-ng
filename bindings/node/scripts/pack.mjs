import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const npmCli = process.env.npm_execpath;
if (!npmCli) throw new Error("npm_execpath is not available; run this script through npm");

const metadata = JSON.parse(readFileSync(join(packageRoot, "package.json"), "utf8"));
metadata.optionalDependencies = JSON.parse(
  readFileSync(join(packageRoot, "platform-dependencies.json"), "utf8"),
);
const staging = mkdtempSync(join(tmpdir(), "corrplot-node-pack-"));

try {
  for (const entry of metadata.files ?? []) {
    cpSync(join(packageRoot, entry), join(staging, entry), { recursive: true });
  }
  writeFileSync(join(staging, "package.json"), JSON.stringify(metadata, null, 2) + "\n");
  execFileSync(process.execPath, [npmCli, "pack", ...process.argv.slice(2)], {
    cwd: staging,
    stdio: "inherit",
  });
} finally {
  rmSync(staging, { recursive: true, force: true });
}
