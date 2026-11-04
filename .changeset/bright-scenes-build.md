---
"@corrplot-ng/native-node": minor
"@corrplot-ng/wasm": minor
"@corrplot-ng/typescript": minor
---

Add Rust-backed normalized-matrix scene construction to the Node and WASM bindings and expose it through the shared TypeScript facade. The facade now renders returned scenes to SVG/RGBA, accepts numeric typed arrays with an explicit missing-cell mask, and supports renderer-ID capability checks.
