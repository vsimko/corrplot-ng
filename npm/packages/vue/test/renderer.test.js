import test from "node:test";
import assert from "node:assert/strict";
import { CorrplotError, renderMatrixToSvg } from "../src/renderer.js";

const matrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.5, 0.5, 1],
  kind: "correlation",
};

const scene = { width: 200, height: 160, cells: [], texts: [] };

function fakeWasm(overrides = {}) {
  return {
    validateMatrix: () => ({ valid: true, diagnostic: null }),
    createScene: () => ({ scene, diagnostic: null }),
    renderSceneSvg: () => ({ svg: "<svg />", diagnostic: null }),
    ...overrides,
  };
}

test("builds and renders a scene through the browser WASM operations", async () => {
  let receivedMatrix;
  let receivedSceneOptions;
  let receivedSvgOptions;
  const wasm = fakeWasm({
    validateMatrix: input => {
      receivedMatrix = input;
      return { valid: true, diagnostic: null };
    },
    createScene: (input, options) => {
      receivedMatrix = input;
      receivedSceneOptions = options;
      return { scene, diagnostic: null };
    },
    renderSceneSvg: (input, options) => {
      assert.equal(input, scene);
      receivedSvgOptions = options;
      return { svg: "<svg data-rendered='true' />", diagnostic: null };
    },
  });

  const svg = await renderMatrixToSvg({
    ...matrix,
    values: new Float64Array([1, 0, 0, 1]),
    missingMask: [false, true, true, false],
  }, {
    cellMethod: "ellipse",
    title: "Test figure",
  }, {
    width: 320,
    height: 240,
    fontSizePt: 12,
  }, async () => wasm);

  assert.match(svg, /data-rendered/);
  assert.deepEqual(receivedMatrix.values, [1, null, null, 1]);
  assert.equal("missingMask" in receivedMatrix, false);
  assert.equal(receivedSceneOptions.cellMethod, "ellipse");
  assert.equal(receivedSceneOptions.title, "Test figure");
  assert.equal(receivedSceneOptions.showDiagonal, true);
  assert.deepEqual(receivedSvgOptions, {
    width: 320,
    height: 240,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 12,
    background: { red: 255, green: 255, blue: 255, alpha: 255 },
  });
});

test("uses scene dimensions when SVG dimensions are omitted", async () => {
  let options;
  await renderMatrixToSvg(matrix, {}, {}, async () => fakeWasm({
    renderSceneSvg: (_scene, received) => {
      options = received;
      return { svg: "<svg />", diagnostic: null };
    },
  }));
  assert.equal(options.width, scene.width);
  assert.equal(options.height, scene.height);
});

test("maps matrix validation diagnostics to CorrplotError", async () => {
  const value = {
    schema_version: 1,
    code: "input.matrix.invalid_shape",
    severity: "error",
    stage: "input",
    message: "matrix is not square",
    path: "matrix",
  };
  await assert.rejects(
    renderMatrixToSvg(matrix, {}, {}, async () => fakeWasm({
      validateMatrix: () => ({ valid: false, diagnostic: value }),
    })),
    error => error instanceof CorrplotError && error.diagnostic === value,
  );
});

test("maps scene-construction and SVG-rendering diagnostics", async () => {
  const sceneDiagnostic = {
    schema_version: 1,
    code: "input.matrix.invalid_shape",
    severity: "error",
    stage: "input",
    message: "scene could not be built",
  };
  await assert.rejects(
    renderMatrixToSvg(matrix, {}, {}, async () => fakeWasm({
      createScene: () => ({ scene: null, diagnostic: sceneDiagnostic }),
    })),
    error => error instanceof CorrplotError && error.diagnostic === sceneDiagnostic,
  );

  const renderDiagnostic = {
    schema_version: 1,
    code: "renderer.unsupported_feature",
    severity: "error",
    stage: "renderer",
    message: "unsupported",
  };
  await assert.rejects(
    renderMatrixToSvg(matrix, {}, {}, async () => fakeWasm({
      renderSceneSvg: () => ({ svg: null, diagnostic: renderDiagnostic }),
    })),
    error => error instanceof CorrplotError && error.diagnostic === renderDiagnostic,
  );
});

test("reports WASM initialization failures as actionable diagnostics", async () => {
  await assert.rejects(
    renderMatrixToSvg(matrix, {}, {}, async () => {
      throw new Error("missing WASM asset");
    }),
    error => error instanceof CorrplotError &&
      error.diagnostic.code === "configuration.invalid_field" &&
      error.diagnostic.path === "backend" &&
      error.message.includes("missing WASM asset"),
  );
});

test("rejects malformed values and invalid SVG dimensions before rendering", async () => {
  await assert.rejects(
    renderMatrixToSvg({ ...matrix, values: [1, undefined, 0.5, 1] }, {}, {}, async () => fakeWasm()),
    error => error instanceof CorrplotError && error.diagnostic.path === "values[1]",
  );
  await assert.rejects(
    renderMatrixToSvg(matrix, {}, { width: 0 }, async () => fakeWasm()),
    error => error instanceof CorrplotError && error.diagnostic.path === "output",
  );
});

test("fails clearly when the WASM package does not expose required operations", async () => {
  await assert.rejects(
    renderMatrixToSvg(matrix, {}, {}, async () => ({ validateMatrix() {} })),
    error => error instanceof CorrplotError && error.diagnostic.path === "backend.createScene",
  );
});
