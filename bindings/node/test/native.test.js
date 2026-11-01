const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const {
  CorrplotError,
  getRendererCapabilities,
  renderSceneSvg,
  renderSceneRgba,
  renderMatrixSvg,
  validateMatrix,
} = require("../index.js");

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

test("validates normalized correlation matrices in the shared Rust core", () => {
  assert.equal(validateMatrix({
    labels: ["alpha", "beta"],
    rows: 2,
    columns: 2,
    values: [1, 0.5, 0.5, 1],
    kind: "correlation",
  }), true);
});

test("maps shared matrix errors to diagnostic-v1", () => {
  assert.throws(
    () => validateMatrix({
      labels: ["alpha", "beta"],
      rows: 2,
      columns: 2,
      values: [1, 0.5, 0.2, 1],
      kind: "correlation",
    }),
    (error) => {
      assert.ok(error instanceof CorrplotError);
      assert.equal(error.diagnostic.schema_version, 1);
      assert.equal(error.diagnostic.code, "input.matrix.not_symmetric");
      assert.equal(error.diagnostic.stage, "input");
      return true;
    },
  );
});

test("renders a typed scene through the shared Rust SVG renderer", () => {
  const svg = renderSceneSvg(scene, {
    width: 100,
    height: 100,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  });

  assert.match(svg, /^<svg /);
  assert.match(svg, /<circle /);
  assert.match(svg, /viewBox="0 0 10 10"/);
});

test("returns owned RGBA pixels and renderer metadata from the shared Rust renderer", () => {
  const output = renderSceneRgba(scene, {
    widthPx: 16,
    heightPx: 12,
    dpi: 96,
    background: white,
  });

  assert.ok(Buffer.isBuffer(output.pixels));
  assert.equal(output.widthPx, 16);
  assert.equal(output.heightPx, 12);
  assert.equal(output.strideBytes, 64);
  assert.equal(output.pixelFormat, "rgba8");
  assert.equal(output.alphaMode, "straight");
  assert.equal(output.colorSpace, "srgb");
  assert.equal(output.dpi, 96);
  assert.equal(output.pixels.byteLength, output.strideBytes * output.heightPx);
});

test("maps invalid RGBA output options to diagnostic-v1", () => {
  assert.throws(
    () => renderSceneRgba(scene, {
      widthPx: 0,
      heightPx: 12,
      background: white,
    }),
    (error) => {
      assert.ok(error instanceof CorrplotError);
      assert.equal(error.diagnostic.schema_version, 1);
      assert.equal(error.diagnostic.code, "renderer.output.invalid_options");
      assert.equal(error.diagnostic.stage, "renderer");
      return true;
    },
  );
});

test("exposes the shared renderer capability declaration", () => {
  const capabilities = getRendererCapabilities();
  assert.equal(capabilities.capability_schema_version, 1);
  assert.ok(Array.isArray(capabilities.cell_methods));
});


const sceneV1 = fs.readFileSync(
  path.join(__dirname, "../../../prototypes/shared-svg/scene-v1.example.json"),
  "utf8",
);

test("constructs and renders a matrix through the shared core operation", () => {
  const svg = renderMatrixSvg(sceneV1, "ellipse");
  assert.match(svg, /^<svg /);
  assert.match(svg, /<ellipse /);
  assert.match(svg, /rotate\(-45 /);
});

test("maps invalid matrix construction options to diagnostic-v1", () => {
  assert.throws(
    () => renderMatrixSvg(sceneV1, "unknown"),
    (error) => {
      assert.ok(error instanceof CorrplotError);
      assert.equal(error.diagnostic.code, "configuration.invalid_field");
      assert.equal(error.diagnostic.path, "cellMethod");
      return true;
    },
  );
});
