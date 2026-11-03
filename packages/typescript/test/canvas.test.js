import test from "node:test";
import assert from "node:assert/strict";
import { CorrplotError } from "../src/factory.js";
import { drawCanvasCommands } from "../src/canvas.js";

function mockCanvas() {
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
    arc: (...args) => calls.push(["arc", ...args]),
    ellipse: (...args) => calls.push(["ellipse", ...args]),
    moveTo: (...args) => calls.push(["moveTo", ...args]),
    lineTo: (...args) => calls.push(["lineTo", ...args]),
    closePath: () => calls.push(["closePath"]),
    fill: () => calls.push(["fill"]),
    stroke: () => calls.push(["stroke"]),
    rect: (...args) => calls.push(["rect", ...args]),
    clip: () => calls.push(["clip"]),
    translate: (...args) => calls.push(["translate", ...args]),
    rotate: (...args) => calls.push(["rotate", ...args]),
    fillText: (...args) => calls.push(["fillText", ...args]),
  };
  const canvas = {
    width: 0,
    height: 0,
    getContext: () => context,
    setAttribute: (...args) => calls.push(["setAttribute", ...args]),
  };
  context.canvas = canvas;
  return { canvas, context, calls };
}

const scene = {
  width: 100,
  height: 50,
  description: "test scene",
  commands: [
    { op: "fillRect", x: 0, y: 0, width: 10, height: 10, color: { red: 255, green: 0, blue: 0, alpha: 255 } },
    { op: "text", text: "label", x: 4, y: 5, fontSize: 8, color: { red: 0, green: 0, blue: 0, alpha: 255 }, anchor: "middle", rotationDegrees: 30, fontFamily: "sans-serif", fontWeight: "normal", fontStyle: "normal" },
  ],
};

test("draws direct scene commands at device-pixel-ratio backing resolution", () => {
  const { canvas, calls } = mockCanvas();
  const result = drawCanvasCommands(scene, { canvas, width: 100, height: 50, devicePixelRatio: 2, background: { red: 255, green: 255, blue: 255, alpha: 255 } });
  assert.equal(result.widthPx, 200);
  assert.equal(result.heightPx, 100);
  assert.equal(canvas.width, 200);
  assert.equal(canvas.height, 100);
  assert.ok(calls.some(call => call[0] === "fillText" && call[1] === "label"));
  assert.ok(calls.some(call => call[0] === "clip"));
  assert.ok(calls.some(call => call[0] === "clearRect"));
});

test("accepts a caller-provided OffscreenCanvas-like context", () => {
  const { context, canvas } = mockCanvas();
  const result = drawCanvasCommands(scene, { context, width: 50, height: 25 });
  assert.equal(result.canvas, canvas);
  assert.equal(canvas.width, 50);
  assert.equal(canvas.height, 25);
});

test("rejects invalid dimensions and mismatched context ownership", () => {
  const { canvas, context } = mockCanvas();
  assert.throws(() => drawCanvasCommands(scene, { canvas, width: 0, height: 10 }), CorrplotError);
  assert.throws(() => drawCanvasCommands(scene, { canvas: { ...canvas }, context, width: 10, height: 10 }), CorrplotError);
});

test("rejects unsupported drawing commands with a diagnostic", () => {
  const { canvas } = mockCanvas();
  assert.throws(
    () => drawCanvasCommands({ ...scene, commands: [{ op: "webgl" }] }, { canvas, width: 10, height: 10 }),
    error => error instanceof CorrplotError && error.diagnostic.code === "renderer.unsupported_feature",
  );
});
