const test = require("node:test");
const assert = require("node:assert/strict");
const {
  CorrplotError,
  getRendererCapabilities,
  renderSceneSvg,
  validateMatrix,
} = require("../index.js");

const black = { red: 0, green: 0, blue: 0, alpha: 255 };
const white = { red: 255, green: 255, blue: 255, alpha: 255 };

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
  const svg = renderSceneSvg({
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
  }, {
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

test("exposes the shared renderer capability declaration", () => {
  const capabilities = getRendererCapabilities();
  assert.equal(capabilities.capability_schema_version, 1);
  assert.ok(Array.isArray(capabilities.cell_methods));
});
