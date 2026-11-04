import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const esbuildRoot = fileURLToPath(new URL("./esbuild/", import.meta.url));
const viteRoot = fileURLToPath(new URL("./vite-dist/", import.meta.url));

async function allFiles(directory) {
  const files = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...await allFiles(path));
    else if (entry.isFile()) files.push(path);
  }
  return files;
}

async function assertBrowserOnly(directory, label) {
  const files = await allFiles(directory);
  const javascript = files.filter(path => path.endsWith(".js"));
  assert.ok(javascript.length > 0, `${label} must contain a JavaScript bundle`);
  assert.ok(!files.some(path => path.endsWith(".node") || path.includes("native-node")), `${label} must not contain a Node-API binary`);
  const source = (await Promise.all(javascript.map(path => readFile(path, "utf8")))).join("\n");
  assert.ok(!source.includes("@corrplot-ng/native-node"), `${label} must not include the native Node package`);
  assert.ok(!source.includes("bindings/node"), `${label} must not include Node binding sources`);
  assert.doesNotMatch(source, /["']node:[^"']+["']/, `${label} must not contain Node built-in imports`);
}

await assertBrowserOnly(esbuildRoot, "esbuild React package bundle");
await assertBrowserOnly(viteRoot, "Vite React application bundle");
console.log("React browser bundles are isolated from Node-only dependencies.");
