import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";

const [moduleArg, fixtureArg] = process.argv.slice(2);
if (!moduleArg || !fixtureArg) {
  throw new Error("usage: node run.mjs <module.wasm> <fixture.json>");
}

const modulePath = resolve(moduleArg);
const fixturePath = resolve(fixtureArg);
const wasm = readFileSync(modulePath);
const request = readFileSync(fixturePath);
const runtime = process.env.CORRPLOT_WASMTIME || "wasmtime";
const started = performance.now();
const child = spawnSync(runtime, ["run", modulePath], {
  input: request,
  encoding: "utf8",
  maxBuffer: 16 * 1024 * 1024,
});
const elapsedMs = performance.now() - started;

if (child.error) throw child.error;
if (child.status !== 0) {
  throw new Error(`wasmtime exited with ${child.status}: ${child.stderr}`);
}

const response = JSON.parse(child.stdout);
console.log(JSON.stringify({
  runtime: child.stderr.match(/Wasmtime[^\\n]*/)?.[0] ?? "external Wasmtime",
  wasmSha256: createHash("sha256").update(wasm).digest("hex"),
  fixtureSha256: createHash("sha256").update(request).digest("hex"),
  elapsedMs: Number(elapsedMs.toFixed(3)),
  response,
}, null, 2));
