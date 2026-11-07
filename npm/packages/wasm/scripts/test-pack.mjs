import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);
const packageDirectory = fileURLToPath(new URL("../", import.meta.url));
const { stdout } = await execFileAsync("npm", ["pack", "--dry-run", "--json"], {
  cwd: packageDirectory,
  maxBuffer: 1024 * 1024,
});
const [packageInfo] = JSON.parse(stdout);
const files = new Set(packageInfo.files.map(file => file.path));

for (const required of ["package.json", "pkg/corrplot_ng_wasm.js", "pkg/corrplot_ng_wasm_bg.wasm"]) {
  assert.ok(files.has(required), `WASM npm package is missing ${required}`);
}
assert.ok(!files.has("pkg/.gitignore"), "generated .gitignore must not be included in the WASM package");
console.log("WASM npm package contains the generated ESM entry point and binary.");
