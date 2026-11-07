const test = require("node:test");
const assert = require("node:assert/strict");
const { CorrplotError, createCorrplotEngine } = require("../src/node.cjs");

const pixels = new Uint8Array([1, 2, 3, 4]);
const validMatrix = { labels: ["alpha"], rows: 1, columns: 1, values: [1], kind: "correlation" };
function adapter(overrides = {}) {
  return {
    validateMatrix: () => true,
    createScene: () => ({ scene: { width: 10, height: 10, description: "", cells: [], texts: [] }, diagnostic: null }),
    renderSceneSvg: () => ({ svg: "<svg scene />", diagnostic: null }),
    renderSceneRgba: () => ({ widthPx: 1, heightPx: 1, strideBytes: 4, pixelFormat: "rgba8", alphaMode: "straight", colorSpace: "srgb", pixels }),
    renderMatrixSvg: () => "<svg />",
    renderMatrixRgba: () => ({ widthPx: 1, heightPx: 1, strideBytes: 4, pixelFormat: "rgba8", alphaMode: "straight", colorSpace: "srgb", pixels }),
    getRendererCapabilities: () => ({ capability_schema_version: 1 }),
    ...overrides,
  };
}

test("CommonJS entry creates an engine using an injected backend", async () => {
  const engine = await createCorrplotEngine({ adapter: adapter() });
  assert.equal(engine.backend, "injected");
  assert.equal(engine.validate(validMatrix), true);
  assert.equal(engine.render("{}", { format: "svg" }), "<svg />");
});

test("CommonJS entry shares the facade CorrplotError class", async () => {
  const nativeError = new Error("bad input");
  nativeError.diagnostic = {
    schema_version: 1,
    code: "input.matrix.not_symmetric",
    severity: "error",
    stage: "input",
    message: "bad input",
  };
  const engine = await createCorrplotEngine({
    adapter: adapter({ validateMatrix: () => { throw nativeError; } }),
  });
  assert.throws(() => engine.validate(validMatrix), error =>
    error instanceof CorrplotError &&
    error.diagnostic.code === "input.matrix.not_symmetric");
});

test("CommonJS entry rejects an unavailable backend with diagnostic-v1", async () => {
  await assert.rejects(
    createCorrplotEngine({ backend: "wasm" }),
    error => error instanceof CorrplotError &&
      error.diagnostic.code === "configuration.invalid_field" &&
      error.diagnostic.path === "backend",
  );
});
