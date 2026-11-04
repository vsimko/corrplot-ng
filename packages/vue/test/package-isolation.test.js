import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const packageRoot = new URL("../", import.meta.url);
const manifest = JSON.parse(await readFile(new URL("package.json", packageRoot), "utf8"));

test("Vue package has only the browser WASM runtime dependency", () => {
  assert.deepEqual(Object.keys(manifest.dependencies ?? {}).sort(), ["@corrplot-ng/wasm"]);
  assert.equal(manifest.peerDependencies?.vue, "^3.3.0");
  assert.equal(manifest.peerDependencies?.react, undefined);
  assert.equal(manifest.dependencies?.["@corrplot-ng/native-node"], undefined);
  assert.equal(manifest.dependencies?.["@corrplot-ng/typescript"], undefined);
});

test("Vue runtime source has no Node-only imports or native binding references", async () => {
  const [index, renderer] = await Promise.all([
    readFile(new URL("src/index.js", packageRoot), "utf8"),
    readFile(new URL("src/renderer.js", packageRoot), "utf8"),
  ]);
  const source = index + "\n" + renderer;
  assert.ok(!source.includes("@corrplot-ng/native-node"));
  assert.ok(!source.includes("bindings/node"));
  assert.ok(!source.includes("@corrplot-ng/react"));
  assert.doesNotMatch(source, /from\s+["']node:/);
  assert.doesNotMatch(source, /import\(["']node:/);
});
