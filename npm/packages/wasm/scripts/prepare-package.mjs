import { access, rm } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const packageDirectory = fileURLToPath(new URL("../", import.meta.url));
const outputDirectory = join(packageDirectory, "pkg");

// wasm-pack writes a nested .gitignore that excludes every generated file from
// npm pack. The package root .gitignore still keeps pkg/ out of Git.
await rm(join(outputDirectory, ".gitignore"), { force: true });

for (const file of ["corrplot_ng_wasm.js", "corrplot_ng_wasm_bg.wasm"]) {
  await access(join(outputDirectory, file));
}
