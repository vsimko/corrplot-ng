import test from "node:test";
import assert from "node:assert/strict";
import { CorrplotError, createEngine } from "../src/factory.js";

const pixels = new Uint8Array([1, 2, 3, 4]);
function adapter(overrides = {}) {
  return {
    validateMatrix: () => true,
    createScene: () => ({ scene: { width: 10, height: 10, description: "test scene", cells: [], texts: [] }, diagnostic: null }),
    renderSceneSvg: () => ({ svg: "<svg scene />", diagnostic: null }),
    renderSceneRgba: () => ({ widthPx: 1, heightPx: 1, strideBytes: 4, pixelFormat: "rgba8", alphaMode: "straight", colorSpace: "srgb", pixels }),
    renderMatrixSvg: () => "<svg />",
    renderMatrixRgba: () => ({ widthPx: 1, heightPx: 1, strideBytes: 4, pixelFormat: "rgba8", alphaMode: "straight", colorSpace: "srgb", pixels }),
    getRendererCapabilities: () => JSON.stringify({ capability_schema_version: 1 }),
    ...overrides,
  };
}

test("validates and renders through the injected backend", () => {
  const engine = createEngine(adapter(), "injected");
  assert.equal(engine.validate({}), true);
  assert.equal(engine.render("{}", { format: "svg" }), "<svg />");
  assert.equal(engine.render("{}", { format: "rgba", widthPx: 1, heightPx: 1, background: { red: 0, green: 0, blue: 0, alpha: 255 } }).pixels, pixels);
  assert.deepEqual(engine.getRendererCapabilities(), { capability_schema_version: 1 });
});

test("converts backend validation diagnostics to CorrplotError", () => {
  const engine = createEngine(adapter({ validateMatrix: () => ({ valid: false, diagnostic: { schema_version: 1, code: "input.matrix.invalid_shape", severity: "error", stage: "input", message: "bad matrix" } }) }), "injected");
  assert.throws(() => engine.validate({}), error => error instanceof CorrplotError && error.diagnostic.code === "input.matrix.invalid_shape");
});

test("converts renderer diagnostics to CorrplotError", () => {
  const engine = createEngine(adapter({ renderMatrixSvg: () => ({ svg: null, diagnostic: { schema_version: 1, code: "renderer.unsupported_feature", severity: "error", stage: "renderer", message: "unsupported" } }) }), "injected");
  assert.throws(() => engine.render("{}", { format: "svg" }), error => error instanceof CorrplotError && error.diagnostic.stage === "renderer");
});

test("rejects malformed RGBA dimensions before calling the backend", () => {
  let called = false;
  const engine = createEngine(adapter({ renderMatrixRgba: () => { called = true; return {}; } }), "injected");
  assert.throws(() => engine.render("{}", { format: "rgba", widthPx: 0, heightPx: 1, background: {} }), CorrplotError);
  assert.equal(called, false);
});

test("dispose is idempotent and prevents later operations", () => {
  let calls = 0;
  const engine = createEngine(adapter({ dispose: () => { calls += 1; } }), "injected");
  engine.dispose();
  engine.dispose();
  assert.equal(calls, 1);
  assert.throws(() => engine.validate({}), error => error instanceof CorrplotError && error.diagnostic.code === "configuration.invalid_field");
});

test("normalizes diagnostics thrown by a native-style adapter", () => {
  const nativeError = new Error("bad input");
  nativeError.diagnostic = { schema_version: 1, code: "input.matrix.not_symmetric", severity: "error", stage: "input", message: "bad input" };
  const engine = createEngine(adapter({ validateMatrix: () => { throw nativeError; } }), "injected");
  assert.throws(() => engine.validate({}), error => error instanceof CorrplotError && error.diagnostic.code === "input.matrix.not_symmetric");
});

test("creates and renders a renderer-neutral scene through the injected backend", () => {
  const engine = createEngine(adapter(), "injected");
  const scene = engine.createScene({
    labels: ["alpha"],
    rows: 1,
    columns: 1,
    values: [1],
    kind: "correlation",
  }, { title: "test" });
  assert.equal(scene.description, "test scene");
  assert.equal(engine.render(scene, { format: "svg" }), "<svg scene />");
  const output = engine.render(scene, {
    format: "rgba",
    widthPx: 1,
    heightPx: 1,
    background: { red: 255, green: 255, blue: 255, alpha: 255 },
  });
  assert.equal(output.pixels, pixels);
});

test("converts scene-construction diagnostics to CorrplotError", () => {
  const engine = createEngine(adapter({
    createScene: () => ({
      scene: null,
      diagnostic: { schema_version: 1, code: "input.matrix.invalid_shape", severity: "error", stage: "input", message: "not square" },
    }),
  }), "injected");
  assert.throws(
    () => engine.createScene({ labels: [], rows: 0, columns: 0, values: [], kind: "correlation" }),
    error => error instanceof CorrplotError && error.diagnostic.code === "input.matrix.invalid_shape",
  );
});

test("normalizes typed-array matrix values with an explicit missing mask", () => {
  let received;
  const engine = createEngine(adapter({
    createScene: input => {
      received = input;
      return { scene: { width: 1, height: 1, cells: [], texts: [] }, diagnostic: null };
    },
  }), "injected");
  engine.createScene({
    labels: ["alpha", "beta"],
    rows: 2,
    columns: 2,
    values: new Float64Array([1, 0, 0, 1]),
    missingMask: [false, true, true, false],
    kind: "correlation",
  });
  assert.deepEqual(received.values, [1, null, null, 1]);
  assert.equal("missingMask" in received, false);
});

test("validates an optional renderer id against the backend capability declaration", () => {
  const engine = createEngine(adapter({
    getRendererCapabilities: () => ({ renderer: { id: "rust-core-scene" }, capability_schema_version: 1 }),
  }), "injected");
  assert.equal(engine.getRendererCapabilities("rust-core-scene").renderer.id, "rust-core-scene");
  assert.throws(
    () => engine.getRendererCapabilities("unknown"),
    error => error instanceof CorrplotError && error.diagnostic.code === "renderer.unsupported_feature",
  );
});
