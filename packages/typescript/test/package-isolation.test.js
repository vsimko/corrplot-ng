import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const packageRoot = new URL("../", import.meta.url);
const manifest = JSON.parse(await readFile(new URL("package.json", packageRoot), "utf8"));

test("backend packages are optional peers, never installed as facade dependencies", () => {
  assert.equal(manifest.dependencies, undefined);
  assert.equal(manifest.optionalDependencies, undefined);

  for (const name of ["@corrplot-ng/native-node", "@corrplot-ng/wasm"]) {
    assert.equal(manifest.peerDependencies?.[name], "0.1.0");
    assert.equal(manifest.peerDependenciesMeta?.[name]?.optional, true);
  }
});

test("runtime entry points import only their matching backend", async () => {
  const [browser, node, commonJs] = await Promise.all([
    readFile(new URL("src/browser.js", packageRoot), "utf8"),
    readFile(new URL("src/node.js", packageRoot), "utf8"),
    readFile(new URL("src/node.cjs", packageRoot), "utf8"),
  ]);

  assert.match(browser, /@corrplot-ng\/browser-wasm/);
  assert.doesNotMatch(browser, /@corrplot-ng\/native-node/);
  assert.match(node, /@corrplot-ng\/native-node/);
  assert.doesNotMatch(node, /@corrplot-ng\/browser-wasm/);
  assert.match(commonJs, /@corrplot-ng\/native-node/);
  assert.doesNotMatch(commonJs, /@corrplot-ng\/browser-wasm/);
});
