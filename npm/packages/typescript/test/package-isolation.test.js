import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const packageRoot = new URL("../", import.meta.url);
const manifest = JSON.parse(await readFile(new URL("package.json", packageRoot), "utf8"));
const wasmManifest = JSON.parse(await readFile(new URL("../../bindings/wasm/package.json", packageRoot), "utf8"));

test("backend packages are optional peers, never installed as facade dependencies", () => {
  assert.equal(manifest.dependencies, undefined);
  assert.equal(manifest.optionalDependencies, undefined);

  for (const name of ["@corrplot-ng/native-node", "@corrplot-ng/wasm"]) {
    assert.equal(manifest.peerDependencies?.[name], "0.1.0");
    assert.equal(manifest.peerDependenciesMeta?.[name]?.optional, true);
  }
});

test("browser facade and WASM package use the same canonical package name", async () => {
  assert.equal(wasmManifest.name, "@corrplot-ng/wasm");
  const browser = await readFile(new URL("src/browser.js", packageRoot), "utf8");
  assert.ok(browser.includes("@corrplot-ng/wasm"));
  assert.ok(!browser.includes("@corrplot-ng/native-node"));
  assert.ok(browser.includes("./canvas.js"));
});

test("runtime entry points import only their matching backend", async () => {
  const [node, commonJs] = await Promise.all([
    readFile(new URL("src/node.js", packageRoot), "utf8"),
    readFile(new URL("src/node.cjs", packageRoot), "utf8"),
  ]);

  assert.ok(node.includes("@corrplot-ng/native-node"));
  assert.ok(!node.includes("@corrplot-ng/wasm"));
  assert.ok(!node.includes("./canvas.js"));
  assert.ok(commonJs.includes("@corrplot-ng/native-node"));
  assert.ok(!commonJs.includes("@corrplot-ng/wasm"));
});
