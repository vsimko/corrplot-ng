import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const bundle = await readFile(new URL("./dist/smoke.js", import.meta.url), "utf8");
assert.ok(bundle.length > 0, "browser facade bundle must not be empty");
assert.ok(!bundle.includes("@corrplot-ng/native-node"), "browser bundle must not contain the native Node package");
assert.ok(!bundle.includes("bindings/node"), "browser bundle must not contain Node binding sources");
assert.doesNotMatch(
  bundle,
  /(?:from\s*|import\s*\()\s*["']node:/,
  "browser bundle must not contain Node built-in imports",
);
