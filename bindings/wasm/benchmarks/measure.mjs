import assert from "node:assert/strict";
import { readFileSync, statSync } from "node:fs";
import { brotliCompressSync, gzipSync } from "node:zlib";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";
import init, {
  renderSceneRgba,
  renderSceneSvg,
  validateMatrix,
} from "../pkg/corrplot_ng_wasm.js";

const wasmPath = fileURLToPath(new URL("../pkg/corrplot_ng_wasm_bg.wasm", import.meta.url));
const wasmBytes = readFileSync(wasmPath);
assert.equal(statSync(wasmPath).size, wasmBytes.byteLength);

const black = { red: 0, green: 0, blue: 0, alpha: 255 };
const white = { red: 255, green: 255, blue: 255, alpha: 255 };
const matrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.5, 0.5, 1],
  kind: "correlation",
};
const scene = {
  width: 10,
  height: 10,
  cells: [{
    x: 0,
    y: 0,
    size: 10,
    visible: true,
    fill: white,
    stroke: black,
    glyph: { kind: "circle", radius: 3, color: black },
  }],
  texts: [],
};
const svgOptions = {
  width: 256,
  height: 256,
  unit: "px",
  fontFamily: "sans-serif",
  fontSizePt: 10,
  background: white,
};
const rgbaOptions = {
  widthPx: 256,
  heightPx: 256,
  dpi: 96,
  background: white,
};

function summarize(samples) {
  const sorted = [...samples].sort((a, b) => a - b);
  const percentile = (p) => sorted[Math.min(sorted.length - 1, Math.ceil(p * sorted.length) - 1)];
  return {
    iterations: samples.length,
    medianMs: Number(percentile(0.5).toFixed(4)),
    p95Ms: Number(percentile(0.95).toFixed(4)),
  };
}

function measure(operation, iterations, warmup) {
  for (let index = 0; index < warmup; index += 1) operation();
  const samples = [];
  for (let index = 0; index < iterations; index += 1) {
    const start = performance.now();
    operation();
    samples.push(performance.now() - start);
  }
  return summarize(samples);
}

const rssBefore = process.memoryUsage().rss;
const initStart = performance.now();
const wasm = await init(wasmBytes);
const initializationMs = performance.now() - initStart;
const rssAfterInit = process.memoryUsage().rss;

const validation = validateMatrix(matrix);
assert.equal(validation.valid, true);
const svg = renderSceneSvg(scene, svgOptions);
assert.equal(typeof svg.svg, "string");
assert.equal(svg.diagnostic, undefined);
const rgba = renderSceneRgba(scene, rgbaOptions);
assert.equal(rgba.diagnostic, undefined);
assert.ok(rgba.pixels instanceof Uint8Array);
assert.equal(rgba.pixels.byteLength, rgba.strideBytes * rgba.heightPx);

const results = {
  runtime: process.version,
  platform: process.platform,
  architecture: process.arch,
  wasmModule: {
    bytes: wasmBytes.byteLength,
    gzipBytes: gzipSync(wasmBytes).byteLength,
    brotliBytes: brotliCompressSync(wasmBytes).byteLength,
  },
  initialization: {
    coldStartMs: Number(initializationMs.toFixed(3)),
    wasmLinearMemoryBytes: wasm.memory?.buffer?.byteLength ?? null,
    processRssDeltaBytes: rssAfterInit - rssBefore,
  },
  operations: {
    validateMatrix: measure(() => validateMatrix(matrix), 500, 25),
    renderSceneSvg256: measure(() => renderSceneSvg(scene, svgOptions), 100, 10),
    renderSceneRgba256: measure(() => renderSceneRgba(scene, rgbaOptions), 40, 5),
  },
  rgbaOutput: {
    widthPx: rgba.widthPx,
    heightPx: rgba.heightPx,
    strideBytes: rgba.strideBytes,
    pixelBytes: rgba.pixels.byteLength,
  },
};

console.log("WASM baseline (informational; no performance gate):");
console.log(JSON.stringify(results, null, 2));
