import test from "node:test";
import assert from "node:assert/strict";
import { CorrplotError, createEngine } from "../src/factory.js";
import { getCanvasRendererCapabilities, renderSceneToCanvas } from "../src/canvas.js";

const black = { red: 0, green: 0, blue: 0, alpha: 255 };
const red = { red: 220, green: 30, blue: 40, alpha: 180 };
const white = { red: 255, green: 255, blue: 255, alpha: 255 };

function mockCanvas({ style = {} } = {}) {
  const calls = [];
  const context = {
    canvas: null,
    save: () => calls.push(["save"]),
    restore: () => calls.push(["restore"]),
    setTransform: (...args) => calls.push(["setTransform", ...args]),
    clearRect: (...args) => calls.push(["clearRect", ...args]),
    fillRect: (...args) => calls.push(["fillRect", ...args]),
    strokeRect: (...args) => calls.push(["strokeRect", ...args]),
    beginPath: () => calls.push(["beginPath"]),
    rect: (...args) => calls.push(["rect", ...args]),
    clip: () => calls.push(["clip"]),
    arc: (...args) => calls.push(["arc", ...args]),
    ellipse: (...args) => calls.push(["ellipse", ...args]),
    moveTo: (...args) => calls.push(["moveTo", ...args]),
    lineTo: (...args) => calls.push(["lineTo", ...args]),
    closePath: () => calls.push(["closePath"]),
    fill: () => calls.push(["fill"]),
    stroke: () => calls.push(["stroke"]),
    translate: (...args) => calls.push(["translate", ...args]),
    rotate: (...args) => calls.push(["rotate", ...args]),
    fillText: (...args) => calls.push(["fillText", ...args]),
  };
  const canvas = {
    width: 0,
    height: 0,
    style,
    getContext: kind => kind === "2d" ? context : null,
    setAttribute: (...args) => calls.push(["setAttribute", ...args]),
  };
  context.canvas = canvas;
  return { canvas, context, calls };
}

const scene = {
  width: 100,
  height: 50,
  description: "Canvas test scene",
  cells: [
    { x: 0, y: 0, size: 10, visible: true, fill: white, stroke: black, glyph: { kind: "circle", radius: 4, color: red } },
    { x: 10, y: 0, size: 10, visible: true, fill: white, stroke: black, glyph: { kind: "square", side: 6, color: red } },
    { x: 20, y: 0, size: 10, visible: true, fill: white, stroke: black, glyph: { kind: "ellipse", rx: 4, ry: 2, rotationDegrees: 25, color: red } },
    { x: 30, y: 0, size: 10, visible: true, fill: white, stroke: black, glyph: { kind: "pie", radius: 4, startRadians: 0, endRadians: Math.PI / 2, color: red } },
    { x: 40, y: 0, size: 10, visible: true, fill: white, stroke: black, glyph: { kind: "number", text: "0.5", fontSize: 6, color: black } },
    { x: 50, y: 0, size: 10, visible: true, fill: white, stroke: black, glyph: { kind: "shade", color: red } },
    { x: 60, y: 0, size: 10, visible: true, fill: red, stroke: black, glyph: { kind: "color" } },
    { x: 70, y: 0, size: 10, visible: false, fill: white, stroke: black },
  ],
  texts: [
    { text: "first\nsecond", x: 8, y: 25, fontSize: 8, color: black, anchor: "middle", rotationDegrees: 15 },
  ],
};

test("draws all shared-scene glyphs directly at device-pixel-ratio backing resolution", () => {
  const { canvas, calls } = mockCanvas();
  const result = renderSceneToCanvas(scene, canvas, { width: 100, height: 50, devicePixelRatio: 2 });

  assert.equal(result.canvas, canvas);
  assert.equal(result.widthPx, 200);
  assert.equal(result.heightPx, 100);
  assert.equal(canvas.width, 200);
  assert.equal(canvas.height, 100);
  assert.equal(canvas.style.width, "100px");
  assert.equal(canvas.style.height, "50px");
  assert.ok(calls.some(call => call[0] === "arc"));
  assert.ok(calls.some(call => call[0] === "ellipse"));
  assert.ok(calls.some(call => call[0] === "lineTo"));
  assert.ok(calls.some(call => call[0] === "fillText" && call[1] === "0.5"));
  assert.ok(calls.some(call => call[0] === "fillText" && call[1] === "first"));
  assert.ok(calls.some(call => call[0] === "fillText" && call[1] === "second"));
  assert.equal(calls.filter(call => call[0] === "strokeRect").length, 7);
  assert.ok(calls.some(call => call[0] === "clip"));
  assert.ok(calls.some(call => call[0] === "clearRect"));
  assert.ok(calls.some(call => call[0] === "setAttribute" && call[1] === "aria-label" && call[2] === scene.description));
  assert.equal(calls.at(-1)[0], "restore");
});

test("accepts a caller-provided Canvas 2D context and preserves pixels when clear is false", () => {
  const { context, canvas, calls } = mockCanvas({ style: null });
  const emptyScene = { width: 20, height: 10, cells: [], texts: [] };
  const result = renderSceneToCanvas(emptyScene, context, { width: 40, height: 20, clear: false });

  assert.equal(result.canvas, canvas);
  assert.equal(result.widthPx, 40);
  assert.equal(result.heightPx, 20);
  assert.equal(canvas.width, 40);
  assert.equal(canvas.height, 20);
  assert.ok(!calls.some(call => call[0] === "clearRect"));
  assert.ok(!calls.some(call => call[0] === "fillRect"));
});

test("supports transparent clearing and leaves CSS sizing to the caller when requested", () => {
  const { canvas, calls } = mockCanvas();
  renderSceneToCanvas({ width: 10, height: 10, cells: [], texts: [] }, canvas, {
    background: null,
    setCssSize: false,
  });
  assert.ok(calls.some(call => call[0] === "clearRect"));
  assert.ok(!calls.some(call => call[0] === "fillRect"));
  assert.equal(canvas.style.width, undefined);
});

test("rejects invalid scenes, dimensions, colors, and unsupported scene features with diagnostics", () => {
  const { canvas } = mockCanvas();
  const expectCode = (operation, code) => assert.throws(
    operation,
    error => error instanceof CorrplotError && error.diagnostic.code === code,
  );

  expectCode(() => renderSceneToCanvas(null, canvas), "renderer.output.invalid_scene");
  expectCode(() => renderSceneToCanvas(scene, canvas, { width: 0 }), "renderer.output.invalid_options");
  expectCode(() => renderSceneToCanvas({ ...scene, legend: [{ label: "x" }] }, canvas), "renderer.unsupported_feature");
  expectCode(() => renderSceneToCanvas({ ...scene, clusterRegions: [{}] }, canvas), "renderer.unsupported_feature");
  expectCode(() => renderSceneToCanvas({ ...scene, significance: {} }, canvas), "renderer.unsupported_feature");
  expectCode(() => renderSceneToCanvas({
    ...scene,
    cells: [{ ...scene.cells[0], confidenceInterval: {} }],
  }, canvas), "renderer.unsupported_feature");
  expectCode(() => renderSceneToCanvas({
    ...scene,
    cells: [{ ...scene.cells[0], significance: {} }],
  }, canvas), "renderer.unsupported_feature");
  expectCode(() => renderSceneToCanvas({
    ...scene,
    cells: [{ ...scene.cells[0], fill: { red: 256, green: 0, blue: 0, alpha: 255 } }],
  }, canvas), "renderer.output.invalid_scene");
});

test("reports browser host targets and conservative unsupported-feature capabilities", () => {
  const capabilities = getCanvasRendererCapabilities();
  assert.equal(capabilities.renderer.id, "typescript-canvas-2d");
  assert.ok(capabilities.host_targets.includes("browser-canvas-2d"));
  let offscreenSupported = false;
  try {
    offscreenSupported = typeof globalThis.OffscreenCanvas === "function" &&
      Boolean(new globalThis.OffscreenCanvas(1, 1).getContext("2d"));
  } catch {
    offscreenSupported = false;
  }
  assert.equal(capabilities.host_targets.includes("browser-offscreen-canvas-2d"), offscreenSupported);
  assert.equal(capabilities.output_formats.length, 0);
  assert.equal(capabilities.features.legend, false);
  assert.equal(capabilities.features.confidence_intervals, false);
  assert.equal(capabilities.features.missing_cells, true);
});

test("Engine exposes Canvas only when a browser renderer is supplied", () => {
  const adapter = {
    validateMatrix: () => true,
    renderMatrixSvg: () => "<svg />",
    renderMatrixRgba: () => ({ pixels: new Uint8Array(4), widthPx: 1, heightPx: 1, strideBytes: 4 }),
    getRendererCapabilities: () => ({}),
  };
  const renderer = {
    render: renderSceneToCanvas,
    getCapabilities: getCanvasRendererCapabilities,
  };
  const { canvas } = mockCanvas();
  const browserEngine = createEngine(adapter, "wasm", renderer);
  const result = browserEngine.renderToCanvas({ width: 10, height: 10, cells: [], texts: [] }, canvas);
  assert.equal(result.widthPx, 10);
  assert.equal(browserEngine.getCanvasRendererCapabilities().renderer.id, "typescript-canvas-2d");

  const nodeEngine = createEngine(adapter, "native");
  assert.throws(
    () => nodeEngine.renderToCanvas({ width: 10, height: 10, cells: [], texts: [] }, canvas),
    error => error instanceof CorrplotError && error.diagnostic.code === "renderer.unsupported_feature",
  );
  assert.throws(
    () => nodeEngine.getCanvasRendererCapabilities(),
    error => error instanceof CorrplotError && error.diagnostic.code === "renderer.unsupported_feature",
  );
  nodeEngine.dispose();
  assert.throws(
    () => browserEngine.renderToCanvas({ width: 10, height: 10, cells: [], texts: [] }, canvas),
    error => error instanceof CorrplotError && error.diagnostic.code === "configuration.invalid_field",
  );
});
