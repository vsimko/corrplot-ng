import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync, statSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { cpus, totalmem } from "node:os";
import { performance } from "node:perf_hooks";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { brotliCompressSync, gzipSync } from "node:zlib";

const require = createRequire(import.meta.url);
const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "../../..");
const native = require("../../packages/native-node");
const wasm = require("../../packages/wasm/pkg-node/corrplot_ng_wasm.js");
const smoke = process.argv.includes("--smoke");

const nativeAddonPath = resolve(repoRoot, "npm/packages/native-node/index.node");
const wasmModulePath = resolve(repoRoot, "npm/packages/wasm/pkg-node/corrplot_ng_wasm_bg.wasm");
const nativeAddonBytes = statSync(nativeAddonPath).size;

const white = { red: 255, green: 255, blue: 255, alpha: 255 };
const smallMatrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.5, 0.5, 1],
  kind: "correlation",
};
const largeSize = 64;
const largeLabels = Array.from(
  { length: largeSize },
  (_, index) => "variable-" + String(index + 1).padStart(2, "0"),
);
const largeValues = Array.from({ length: largeSize * largeSize }, (_, index) => {
  const row = Math.floor(index / largeSize);
  const column = index % largeSize;
  if (row === column) return 1;
  const first = Math.min(row, column);
  const second = Math.max(row, column);
  return ((first * 31 + second * 17) % 201 - 100) / 200;
});
const largeMatrix = {
  labels: largeLabels,
  rows: largeSize,
  columns: largeSize,
  values: largeValues,
  kind: "correlation",
};
const sceneOptions = {
  cellMethod: "ellipse",
  title: "Node/WASM performance fixture",
  description: "Deterministic 64-variable correlation matrix.",
  displayType: "upper",
  showDiagonal: false,
};
const rgbaOptions = {
  widthPx: 512,
  heightPx: 512,
  dpi: 96,
  background: white,
};

function percentile(sorted, p) {
  const position = (sorted.length - 1) * p;
  const lower = Math.floor(position);
  const upper = Math.ceil(position);
  if (lower === upper) return sorted[lower];
  return sorted[lower] + (sorted[upper] - sorted[lower]) * (position - lower);
}

function summarize(samples, unit) {
  const sorted = [...samples].sort((a, b) => a - b);
  return {
    unit,
    iterations: samples.length,
    median: Number(percentile(sorted, 0.5).toFixed(4)),
    iqr: Number((percentile(sorted, 0.75) - percentile(sorted, 0.25)).toFixed(4)),
    p95: Number(percentile(sorted, 0.95).toFixed(4)),
    min: Number(sorted[0].toFixed(4)),
    max: Number(sorted[sorted.length - 1].toFixed(4)),
    samples: samples.map(value => Number(value.toFixed(4))),
  };
}

function measure(operation, iterations, warmupIterations) {
  for (let index = 0; index < warmupIterations; index += 1) operation();
  const samples = [];
  for (let index = 0; index < iterations; index += 1) {
    const start = performance.now();
    operation();
    samples.push(performance.now() - start);
  }
  return {
    ...summarize(samples, "ms"),
    warmupIterations,
  };
}

function coldLoad(modulePath) {
  const source = [
    'const { performance } = require("node:perf_hooks");',
    "const before = process.memoryUsage();",
    "const start = performance.now();",
    "require(" + JSON.stringify(modulePath) + ");",
    "const elapsedMs = performance.now() - start;",
    "const after = process.memoryUsage();",
    "process.stdout.write(JSON.stringify({",
    "  elapsedMs,",
    "  rssDeltaBytes: after.rss - before.rss,",
    "  heapUsedDeltaBytes: after.heapUsed - before.heapUsed,",
    "  externalDeltaBytes: after.external - before.external",
    "}));",
  ].join("\n");
  const child = spawnSync(process.execPath, ["-e", source], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  if (child.status !== 0) {
    throw new Error(
      "cold-load subprocess failed for " + modulePath + ": " +
      (child.stderr || child.stdout || "unknown child-process error"),
    );
  }
  const output = child.stdout.trim().split(/\r?\n/).at(-1);
  return JSON.parse(output);
}

function measureColdLoads(modulePath, iterations) {
  const samples = Array.from({ length: iterations }, () => coldLoad(modulePath));
  const summarizeMetric = key => summarize(samples.map(sample => sample[key]), "bytes");
  return {
    elapsed: summarize(samples.map(sample => sample.elapsedMs), "ms"),
    memorySnapshotDeltas: {
      rss: summarizeMetric("rssDeltaBytes"),
      heapUsed: summarizeMetric("heapUsedDeltaBytes"),
      external: summarizeMetric("externalDeltaBytes"),
    },
  };
}

function unwrapWasm(result, field) {
  assert.equal(result.diagnostic ?? null, null);
  return field === undefined ? result : result[field];
}

function validateWasm(input) {
  const result = wasm.validateMatrix(input);
  assert.equal(result.valid, true);
  assert.equal(result.diagnostic ?? null, null);
}

const nativeSceneResult = native.createScene(largeMatrix, sceneOptions);
const wasmSceneResult = wasm.createScene(largeMatrix, sceneOptions);
assert.equal(nativeSceneResult.diagnostic ?? null, null);
assert.equal(wasmSceneResult.diagnostic ?? null, null);
const nativeScene = nativeSceneResult.scene;
const wasmScene = wasmSceneResult.scene;
assert.deepEqual(
  JSON.parse(JSON.stringify(wasmScene)),
  JSON.parse(JSON.stringify(nativeScene)),
  "Node/WASM scene construction must pass before timing",
);

const svgOptions = {
  width: nativeScene.width,
  height: nativeScene.height,
  unit: "px",
  fontFamily: "sans-serif",
  fontSizePt: 10,
  background: white,
};
const nativeSvg = native.renderSceneSvg(nativeScene, svgOptions);
const wasmSvg = unwrapWasm(wasm.renderSceneSvg(wasmScene, svgOptions), "svg");
assert.equal(wasmSvg, nativeSvg, "Node/WASM SVG must pass before timing");

const nativeRgba = native.renderSceneRgba(nativeScene, rgbaOptions);
const wasmRgba = unwrapWasm(wasm.renderSceneRgba(wasmScene, rgbaOptions));
for (const key of ["widthPx", "heightPx", "strideBytes", "pixelFormat", "alphaMode", "colorSpace", "dpi"]) {
  assert.equal(wasmRgba[key], nativeRgba[key], "Node/WASM RGBA metadata differs: " + key);
}
assert.deepEqual(
  Buffer.from(wasmRgba.pixels),
  nativeRgba.pixels,
  "Node/WASM RGBA must pass before timing",
);

assert.equal(native.validateMatrix(smallMatrix), true);
validateWasm(smallMatrix);
assert.equal(native.validateMatrix(largeMatrix), true);
validateWasm(largeMatrix);

const iterations = smoke
  ? { smallValidation: 3, largeValidation: 2, sceneBuild: 2, svg: 2, rgba: 2, coldLoads: 2, warmup: 1 }
  : { smallValidation: 500, largeValidation: 100, sceneBuild: 40, svg: 30, rgba: 10, coldLoads: 7, warmup: 5 };

const operations = {
  validateMatrix2x2: {
    native: measure(() => native.validateMatrix(smallMatrix), iterations.smallValidation, iterations.warmup),
    wasm: measure(() => wasm.validateMatrix(smallMatrix), iterations.smallValidation, iterations.warmup),
  },
  validateMatrix64x64: {
    native: measure(() => native.validateMatrix(largeMatrix), iterations.largeValidation, iterations.warmup),
    wasm: measure(() => wasm.validateMatrix(largeMatrix), iterations.largeValidation, iterations.warmup),
  },
  createScene64x64: {
    native: measure(() => native.createScene(largeMatrix, sceneOptions), iterations.sceneBuild, iterations.warmup),
    wasm: measure(() => wasm.createScene(largeMatrix, sceneOptions), iterations.sceneBuild, iterations.warmup),
  },
  renderSceneSvg64x64: {
    native: measure(() => native.renderSceneSvg(nativeScene, svgOptions), iterations.svg, iterations.warmup),
    wasm: measure(() => wasm.renderSceneSvg(wasmScene, svgOptions), iterations.svg, iterations.warmup),
  },
  renderSceneRgba64x64_512: {
    native: measure(() => native.renderSceneRgba(nativeScene, rgbaOptions), iterations.rgba, iterations.warmup),
    wasm: measure(() => wasm.renderSceneRgba(wasmScene, rgbaOptions), iterations.rgba, iterations.warmup),
  },
};

const wasmModule = readFileSync(wasmModulePath);
const results = {
  schemaVersion: 1,
  measurementMode: smoke ? "smoke" : "baseline",
  measuredAt: new Date().toISOString(),
  sourceCommit: process.env.GITHUB_SHA ?? null,
  environment: {
    runtime: process.version,
    napiVersion: process.versions.napi ?? null,
    platform: process.platform,
    architecture: process.arch,
    cpuModel: cpus()[0]?.model ?? null,
    logicalCpuCount: cpus().length,
    totalMemoryBytes: totalmem(),
  },
  artifacts: {
    nativeAddonBytes,
    wasmModuleBytes: wasmModule.byteLength,
    wasmGzipBytes: gzipSync(wasmModule).byteLength,
    wasmBrotliBytes: brotliCompressSync(wasmModule).byteLength,
  },
  initialization: {
    nativeNodeApi: measureColdLoads(resolve(repoRoot, "npm/packages/native-node"), iterations.coldLoads),
    nodeHostedWasm: measureColdLoads(resolve(repoRoot, "npm/packages/wasm/pkg-node/corrplot_ng_wasm.js"), iterations.coldLoads),
  },
  workloads: {
    matrix2x2: { rows: 2, columns: 2, values: smallMatrix.values.length },
    matrix64x64: { rows: largeSize, columns: largeSize, values: largeValues.length },
    scene64x64: {
      width: nativeScene.width,
      height: nativeScene.height,
      cells: nativeScene.cells.length,
      svgBytes: Buffer.byteLength(nativeSvg),
      rgbaWidthPx: nativeRgba.widthPx,
      rgbaHeightPx: nativeRgba.heightPx,
      rgbaBytes: nativeRgba.pixels.byteLength,
    },
  },
  operations,
  methodology: {
    correctnessGate: "Exact Node/WASM scene, SVG, RGBA metadata, and RGBA byte parity is required before timing.",
    timing: "Each backend is warmed independently; raw samples, median, IQR, p95, minimum, and maximum are retained.",
    memory: "Cold-load RSS/heap/external values are before/after snapshots in fresh Node subprocesses, not peak-memory measurements.",
    initialization: "Cold module-load samples exclude Node process startup and time require() of the native addon or Node-hosted WASM module.",
    limits: "Informational single-runner evidence only; no performance thresholds or cross-machine claims.",
  },
};

console.log("Node native vs Node-hosted WASM benchmark (informational; no performance gate):");
console.log(JSON.stringify(results, null, 2));

const outputPath = process.env.CORRPLOT_BENCHMARK_OUTPUT;
if (outputPath) writeFileSync(outputPath, JSON.stringify(results, null, 2) + "\n");
