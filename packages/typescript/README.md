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

The facade exposes matrix validation, Rust-backed normalized-matrix-to-scene construction through `createScene()`, rendering of renderer-neutral scenes to SVG or RGBA, renderer capabilities, and idempotent disposal. `MatrixInput` uses row-major `number | null` values and currently requires a non-empty square matrix for scene construction. The older `render(sceneV1Json, options)` path remains available for callers with a matrix-bearing scene-v1 JSON document. RGBA results contain JavaScript-owned `Uint8Array` pixels. Direct Canvas is not advertised until the browser adapter in #144 is implemented.

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
