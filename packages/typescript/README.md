# TypeScript facade

The facade selects one execution backend at module resolution time: native Node-API in Node.js or browser WebAssembly in browser builds. It never silently falls back to a different backend.

Install the facade together with only the backend required by the target runtime:

```sh
# Node.js
npm install @corrplot-ng/typescript @corrplot-ng/native-node

# Browser
npm install @corrplot-ng/typescript @corrplot-ng/wasm
```

The backend packages are optional peer dependencies, not ordinary dependencies. npm therefore does not install both backends when the facade is installed. A Node.js application does not need the browser WASM package, and a browser application does not need native Node binaries. The runtime-specific export loads only its matching backend.

```ts
import { createCorrplotEngine } from "@corrplot-ng/typescript";

const engine = await createCorrplotEngine({ backend: "auto" });
const matrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.65, 0.65, 1],
  kind: "correlation",
};

engine.validate(matrix);
const scene = engine.createScene(matrix, {
  cellMethod: "circle",
  title: "Example correlation matrix",
  description: "A two-variable correlation matrix.",
});
const svg = engine.render(scene, { format: "svg" });
const rgba = engine.render(scene, {
  format: "rgba",
  widthPx: 640,
  heightPx: 640,
  background: { red: 255, green: 255, blue: 255, alpha: 255 },
});
engine.dispose();
```

The facade exposes matrix validation, Rust-backed normalized-matrix-to-scene construction through `createScene()`, rendering of renderer-neutral scenes to SVG or RGBA, renderer capabilities, diagnostic errors, and idempotent disposal. `MatrixInput` uses row-major numeric values; ordinary arrays may contain explicit `null` cells, while numeric typed arrays use an optional boolean `missingMask` (`true` means missing). Scene construction currently requires a non-empty square matrix. The older `render(sceneV1Json, options)` path remains available for callers with a matrix-bearing scene-v1 JSON document. RGBA results contain JavaScript-owned `Uint8Array` pixels.

## Direct Canvas 2D rendering (browser only)

The browser facade can draw the Rust-built renderer-neutral scene directly into a caller-owned `HTMLCanvasElement`, `OffscreenCanvas`, or Canvas 2D context. This path draws scene primitives with Canvas 2D; it does not serialize SVG or transfer an intermediate RGBA image.

```ts
const canvas = document.querySelector("canvas");
if (!canvas) throw new Error("Canvas element is missing");

const result = engine.renderToCanvas(scene, canvas, {
  width: 640,
  height: 640,
  devicePixelRatio: window.devicePixelRatio,
  background: { red: 255, green: 255, blue: 255, alpha: 255 },
});

console.log(result.widthPx, result.heightPx);
const canvasCapabilities = engine.getCanvasRendererCapabilities();
console.log(canvasCapabilities.host_targets);
```

Width and height are logical CSS pixels and default to the scene dimensions. The backing store is scaled by `devicePixelRatio`, which defaults to the current environment's value or `1`. For an HTML canvas, CSS width and height are set to the logical dimensions by default; pass `setCssSize: false` to leave CSS layout to the caller. The renderer preserves the context's previous state, letterboxes rather than crops, clips drawing to the viewport, and sets the canvas role and accessible label from the scene description when available.

The canvas is cleared before each render by default. Its background defaults to opaque white; use `background: null` for a transparent cleared canvas. With `clear: false` and no explicit background, existing pixels are preserved when the backing dimensions already match. Resizing a canvas necessarily resets its backing store.

`OffscreenCanvas` uses the same API and can be used where its 2D context is supported, including worker contexts:

```ts
const offscreen = new OffscreenCanvas(640, 640);
engine.renderToCanvas(scene, offscreen, {
  width: 640,
  height: 640,
  devicePixelRatio: 1,
});
const bitmap = offscreen.transferToImageBitmap();
```

A worker can initialize the browser WASM engine, build the scene from transferred matrix data, render to an OffscreenCanvas, and transfer an ImageBitmap back to the main thread. OffscreenCanvas has no DOM element for CSS sizing or ARIA attributes. The runtime capability declaration includes `browser-offscreen-canvas-2d` only when the current environment can create an OffscreenCanvas 2D context. The Canvas renderer supports the seven cell glyph methods, labels, scene text, and missing cells. It currently reports and rejects significance annotations, confidence intervals, legends, cluster regions, and plotmath labels rather than silently dropping them. Canvas rendering is unavailable from the Node runtime and fails with a `renderer.unsupported_feature` diagnostic.

`backend: "auto"` selects native Node-API in Node.js and WebAssembly in browsers. Explicit unavailable backends fail with diagnostic-v1 errors. `getRendererCapabilities("rust-core-scene")` optionally verifies the selected shared Rust renderer ID; `getCanvasRendererCapabilities()` describes the separate browser-owned Canvas target. Tests and advanced integrations may inject an adapter through `adapter`.

## CommonJS on Node.js

Node.js CommonJS consumers resolve to the same native backend and share the same diagnostic error class:

```js
const { createCorrplotEngine, CorrplotError } = require("@corrplot-ng/typescript");

async function main() {
  const engine = await createCorrplotEngine({ backend: "auto" });
  // Use engine.validate(), engine.render(), and engine.dispose().
}
main().catch((error) => {
  if (error instanceof CorrplotError) console.error(error.diagnostic);
  else throw error;
});
```

The browser export remains ESM. Node.js import and require conditions both select the native adapter.
