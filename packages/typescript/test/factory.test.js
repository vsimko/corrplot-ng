import test from "node:test";
import assert from "node:assert/strict";
import { CorrplotError, createEngine } from "../src/factory.js";

const pixels = new Uint8Array([1, 2, 3, 4]);
function adapter(overrides = {}) {
  return {
    validateMatrix: () => true,
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
  const engine = createEngine(adapter({ validateMatrix: () => ({ valid: false, diagnostic: { schema_version: 1, code: "input.matrix.invalid", severity: "error", stage: "input", message: "bad matrix" } }) }), "injected");
  assert.throws(() => engine.validate({}), error => error instanceof CorrplotError && error.diagnostic.code === "input.matrix.invalid");
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
  assert.throws(() => engine.validate({}), error => error instanceof CorrplotError && error.diagnostic.code === "runtime.disposed");
});

test("normalizes diagnostics thrown by the native adapter", () => {
  const nativeError = new Error("bad input");
  nativeError.diagnostic = { schema_version: 1, code: "input.matrix.not_symmetric", severity: "error", stage: "input", message: "bad input" };
  const engine = createEngine(adapter({ validateMatrix: () => { throw nativeError; } }), "injected");
  assert.throws(() => engine.validate({}), error => error instanceof CorrplotError && error.diagnostic.code === "input.matrix.not_symmetric");
});
