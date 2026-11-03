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
engine.validate({ labels: ["a"], rows: 1, columns: 1, values: [1], kind: "correlation" });
const svg = engine.render(sceneV1Json, { format: "svg", method: "circle" });
engine.dispose();
```

The initial facade exposes matrix validation, scene-v1 matrix-to-SVG, scene-v1 matrix-to-RGBA, renderer capabilities, and idempotent disposal. The current `render` input is the matrix-bearing scene-v1 JSON string accepted by the existing Rust adapters; it is not yet a general normalized-matrix plotting API. RGBA results contain JavaScript-owned `Uint8Array` pixels. Direct Canvas is not advertised until the browser adapter in #144 is implemented.

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
