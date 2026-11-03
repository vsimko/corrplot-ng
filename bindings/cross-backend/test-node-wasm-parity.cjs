const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const native = require("../../bindings/node");
const wasm = require("../../bindings/wasm/pkg-node/corrplot_ng_wasm.js");

const black = { red: 0, green: 0, blue: 0, alpha: 255 };
const white = { red: 255, green: 255, blue: 255, alpha: 255 };
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
const sceneV1 = fs.readFileSync(
  path.join(__dirname, "../../../prototypes/shared-svg/scene-v1.example.json"),
  "utf8",
);
const rgbaOptions = {
  widthPx: 380,
  heightPx: 392,
  dpi: 96,
  background: white,
};
const matrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.5, 0.5, 1],
  kind: "correlation",
};

function unwrapSvg(result) {
  assert.equal(result.diagnostic ?? null, null);
  assert.equal(typeof result.svg, "string");
  return result.svg;
}

function diagnosticFrom(operation) {
  try {
    operation();
  } catch (error) {
    assert.ok(error.diagnostic, "native backend error must carry diagnostic-v1");
    return error.diagnostic;
  }
  assert.fail("native backend unexpectedly accepted invalid input");
}

test("Node and WASM validate the same normalized matrix", () => {
  assert.equal(native.validateMatrix(matrix), true);
  const result = wasm.validateMatrix(matrix);
  assert.equal(result.valid, true);
  assert.equal(result.diagnostic ?? null, null);
});

test("Node and WASM return the same diagnostic for invalid matrices", () => {
  const invalid = { ...matrix, values: [1, 0.5, 0.2, 1] };
  const nativeDiagnostic = diagnosticFrom(() => native.validateMatrix(invalid));
  const wasmDiagnostic = wasm.validateMatrix(invalid).diagnostic;
  assert.deepEqual(wasmDiagnostic, nativeDiagnostic);
});

test("Node and WASM render identical SVG for typed scenes", () => {
  const options = {
    width: 160,
    height: 160,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  };
  const nativeSvg = native.renderSceneSvg(scene, options);
  const wasmSvg = unwrapSvg(wasm.renderSceneSvg(scene, options));
  assert.equal(wasmSvg, nativeSvg);
});

test("Node and WASM render identical RGBA bytes for typed scenes", () => {
  const options = { widthPx: 160, heightPx: 160, dpi: 96, background: white };
  const nativeOutput = native.renderSceneRgba(scene, options);
  const wasmOutput = wasm.renderSceneRgba(scene, options);
  assert.equal(wasmOutput.diagnostic ?? null, null);
  assert.equal(wasmOutput.widthPx, nativeOutput.widthPx);
  assert.equal(wasmOutput.heightPx, nativeOutput.heightPx);
  assert.equal(wasmOutput.strideBytes, nativeOutput.strideBytes);
  assert.equal(wasmOutput.pixelFormat, nativeOutput.pixelFormat);
  assert.equal(wasmOutput.alphaMode, nativeOutput.alphaMode);
  assert.equal(wasmOutput.colorSpace, nativeOutput.colorSpace);
  assert.deepEqual(Buffer.from(wasmOutput.pixels), nativeOutput.pixels);
});

test("Node and WASM construct identical matrix SVG", () => {
  const nativeSvg = native.renderMatrixSvg(sceneV1, "ellipse");
  const wasmSvg = unwrapSvg(wasm.renderMatrixSvg(sceneV1, "ellipse"));
  assert.equal(wasmSvg, nativeSvg);
});

test("Node and WASM construct identical matrix RGBA bytes and metadata", () => {
  const nativeOutput = native.renderMatrixRgba(sceneV1, "circle", rgbaOptions);
  const wasmOutput = wasm.renderMatrixRgba(sceneV1, "circle", rgbaOptions);
  assert.equal(wasmOutput.diagnostic ?? null, null);
  for (const key of ["widthPx", "heightPx", "strideBytes", "pixelFormat", "alphaMode", "colorSpace", "dpi"]) {
    assert.equal(wasmOutput[key], nativeOutput[key], key);
  }
  assert.deepEqual(Buffer.from(wasmOutput.pixels), nativeOutput.pixels);
});

test("Node and WASM expose the same renderer capabilities", () => {
  assert.deepEqual(JSON.parse(wasm.getRendererCapabilities()), native.getRendererCapabilities());
});

test("Node and WASM return the same diagnostic for unsupported matrix methods", () => {
  const nativeDiagnostic = diagnosticFrom(() => native.renderMatrixSvg(sceneV1, "unknown"));
  const wasmDiagnostic = wasm.renderMatrixSvg(sceneV1, "unknown").diagnostic;
  assert.deepEqual(wasmDiagnostic, nativeDiagnostic);
});

test("Node and WASM return the same diagnostic for invalid RGBA options", () => {
  const invalidOptions = { ...rgbaOptions, widthPx: 0 };
  const nativeDiagnostic = diagnosticFrom(() => native.renderMatrixRgba(sceneV1, "circle", invalidOptions));
  const wasmDiagnostic = wasm.renderMatrixRgba(sceneV1, "circle", invalidOptions).diagnostic;
  assert.deepEqual(wasmDiagnostic, nativeDiagnostic);
});
