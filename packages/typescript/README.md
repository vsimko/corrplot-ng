# TypeScript facade

The facade selects one execution backend at module resolution time: native Node-API in Node.js or browser WebAssembly in browser builds. It never silently falls back to a different backend.

```ts
import { createCorrplotEngine } from "@corrplot-ng/typescript";

const engine = await createCorrplotEngine({ backend: "auto" });
engine.validate({ labels: ["a"], rows: 1, columns: 1, values: [1], kind: "correlation" });
const svg = engine.render(sceneV1Json, { format: "svg", method: "circle" });
engine.dispose();
```

The initial facade exposes matrix validation, scene-v1 matrix-to-SVG, scene-v1 matrix-to-RGBA, renderer capabilities, and idempotent disposal. The current `render` input is the matrix-bearing scene-v1 JSON string accepted by the existing Rust adapters; it is not yet a general normalized-matrix plotting API. RGBA results contain JavaScript-owned `Uint8Array` pixels. Direct Canvas is available in browser builds through a Rust-generated renderer-neutral command stream; the TypeScript adapter owns the caller-provided Canvas 2D context.

`backend: "auto"` selects native Node-API in Node.js and WebAssembly in browsers. Explicit unavailable backends fail with diagnostic-v1 errors. Tests and advanced integrations may inject an adapter through `adapter`.

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


## Direct browser Canvas

```ts
const canvas = document.querySelector("canvas");
const output = engine.render(sceneV1Json, {
  format: "canvas",
  canvas,
  width: 380,
  height: 392,
  devicePixelRatio: window.devicePixelRatio,
  background: { red: 255, green: 255, blue: 255, alpha: 255 },
});
```

The browser adapter accepts an HTMLCanvasElement, OffscreenCanvas, or an existing 2D context. Width and height are logical pixels; devicePixelRatio controls the backing-store dimensions. The adapter clears by default, clips to the shared scene bounds, and applies an optional RGBA background. CSS sizing remains the caller's responsibility. OffscreenCanvas can be passed directly or through its 2D context in workers that support it.

The Rust WASM backend produces scene-derived drawing commands. Canvas/DOM objects are never passed into Rust, and the Canvas path does not serialize SVG or allocate an intermediate RGBA bitmap. Node's native backend reports Canvas as unsupported.
