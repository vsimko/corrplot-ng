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
const rssBeforeInstantiation = process.memoryUsage().rss;
const initializationStarted = performance.now();
const { instance } = await WebAssembly.instantiate(wasmBytes, {});
const initializationMs = performance.now() - initializationStarted;
const rssAfterInstantiation = process.memoryUsage().rss;
const { memory, corrplot_alloc, corrplot_free, corrplot_validate_json } = instance.exports;
if (!(memory instanceof WebAssembly.Memory)) {
  throw new Error("module does not export linear memory");
}
const wasmLinearMemoryBytesAfterInstantiation = memory.buffer.byteLength;

const emptyPhases = () => ({
  allocationMs: 0,
  inputCopyMs: 0,
  validationMs: 0,
  outputCopyMs: 0,
  releaseMs: 0,
});
let phases = emptyPhases();

function invoke() {
  let started = performance.now();
  const inputPointer = corrplot_alloc(request.length);
  phases.allocationMs += performance.now() - started;
  if (!inputPointer) throw new Error("WASM input allocation failed");

  started = performance.now();
  new Uint8Array(memory.buffer, inputPointer, request.length).set(request);
  phases.inputCopyMs += performance.now() - started;

  started = performance.now();
  const packed = corrplot_validate_json(inputPointer, request.length);
  phases.validationMs += performance.now() - started;

  const result = BigInt(packed);
  const outputPointer = Number(result & 0xffffffffn);
  const outputLength = Number(result >> 32n);
  if (!outputPointer || !outputLength) {
    corrplot_free(inputPointer, request.length);
    throw new Error("WASM validation returned an empty result");
  }

  started = performance.now();
  const outputBytes = new Uint8Array(memory.buffer, outputPointer, outputLength).slice();
  phases.outputCopyMs += performance.now() - started;

  started = performance.now();
  corrplot_free(outputPointer, outputLength);
  corrplot_free(inputPointer, request.length);
  phases.releaseMs += performance.now() - started;
  return outputBytes;
}

const firstCallStarted = performance.now();
const firstOutputBytes = invoke();
const firstCallMs = performance.now() - firstCallStarted;
const firstCallPhases = { ...phases };
const rssAfterFirstCall = process.memoryUsage().rss;
const wasmLinearMemoryBytesAfterFirstCall = memory.buffer.byteLength;

phases = emptyPhases();
invoke(); // warm-up before the repeated-call sample
phases = emptyPhases();
const calls = 100;
const repeatedStarted = performance.now();
let outputBytes;
for (let i = 0; i < calls; i++) outputBytes = invoke();
const elapsedMs = performance.now() - repeatedStarted;
const repeatedPhases = { ...phases };
const rssAfterBenchmark = process.memoryUsage().rss;
const wasmLinearMemoryBytesAfterBenchmark = memory.buffer.byteLength;
const response = JSON.parse(new TextDecoder().decode(outputBytes));
const firstResponse = JSON.parse(new TextDecoder().decode(firstOutputBytes));
if (JSON.stringify(firstResponse) !== JSON.stringify(response)) {
  throw new Error("cold and repeated WASM responses differ");
}

const perCall = (value) => Number((value / calls).toFixed(6));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
console.log(JSON.stringify({
  runtime: process.version,
  wasmSha256: sha256(wasmBytes),
  fixtureSha256: sha256(fixtureBytes),
  coldStart: {
    initializationMs: Number(initializationMs.toFixed(3)),
    firstCallMs: Number(firstCallMs.toFixed(3)),
    firstCallPhasesMs: Object.fromEntries(
      Object.entries(firstCallPhases).map(([name, value]) => [name, Number(value.toFixed(4))]),
    ),
  },
  memory: {
    processRssBytesBeforeInstantiation: rssBeforeInstantiation,
    processRssBytesAfterInstantiation: rssAfterInstantiation,
    processRssBytesAfterFirstCall: rssAfterFirstCall,
    processRssBytesAfterBenchmark: rssAfterBenchmark,
    processRssDeltaAfterInstantiationBytes: rssAfterInstantiation - rssBeforeInstantiation,
    processRssDeltaAfterFirstCallBytes: rssAfterFirstCall - rssBeforeInstantiation,
    processRssDeltaAfterBenchmarkBytes: rssAfterBenchmark - rssBeforeInstantiation,
    wasmLinearMemoryBytesAfterInstantiation,
    wasmLinearMemoryBytesAfterFirstCall,
    wasmLinearMemoryBytesAfterBenchmark,
  },
  transferAndExecution: {
    inputBytesPerCall: request.byteLength,
    outputBytesPerCall: outputBytes.byteLength,
    repeatedCallPhaseMsPerCall: Object.fromEntries(
      Object.entries(repeatedPhases).map(([name, value]) => [name, perCall(value)]),
    ),
  },
  calls,
  elapsedMs,
  perCallMs: elapsedMs / calls,
  response
}, null, 2));
