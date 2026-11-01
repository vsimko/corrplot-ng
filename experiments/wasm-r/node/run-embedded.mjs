#!/usr/bin/env node
// Invoke the same host-neutral wasm32-unknown-unknown module in-process.
import { readFile } from "node:fs/promises";
import { createHash } from "node:crypto";
import { performance } from "node:perf_hooks";

const [wasmPath, fixturePath] = process.argv.slice(2);
if (!wasmPath || !fixturePath) {
  console.error("usage: node run-embedded.mjs MODULE.wasm FIXTURE.json");
  process.exit(2);
}

const wasmBytes = await readFile(wasmPath);
const fixtureBytes = await readFile(fixturePath);
const request = new TextEncoder().encode(fixtureBytes.toString("utf8"));
const { instance } = await WebAssembly.instantiate(wasmBytes, {});
const { memory, corrplot_alloc, corrplot_free, corrplot_validate_json } = instance.exports;
if (!(memory instanceof WebAssembly.Memory)) {
  throw new Error("module does not export linear memory");
}

const started = performance.now();
const inputPointer = corrplot_alloc(request.length);
new Uint8Array(memory.buffer, inputPointer, request.length).set(request);
const packed = corrplot_validate_json(inputPointer, request.length);
corrplot_free(inputPointer, request.length);

const result = BigInt(packed);
const outputPointer = Number(result & 0xffffffffn);
const outputLength = Number(result >> 32n);
if (!outputPointer || !outputLength) throw new Error("WASM validation returned an empty result");
const outputBytes = new Uint8Array(memory.buffer, outputPointer, outputLength).slice();
corrplot_free(outputPointer, outputLength);
const response = JSON.parse(new TextDecoder().decode(outputBytes));

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
console.log(JSON.stringify({
  runtime: process.version,
  wasmSha256: sha256(wasmBytes),
  fixtureSha256: sha256(fixtureBytes),
  elapsedMs: performance.now() - started,
  response
}));
