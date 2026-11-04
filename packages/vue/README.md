# Vue integration

@corrplot-ng/vue provides a thin Vue 3 adapter over Corrplot-ng's browser WebAssembly backend. It exports a Corrplot component and a useCorrplot composable. The package does not import the Node-API backend or the runtime-selecting TypeScript facade.

The package depends on @corrplot-ng/wasm and declares Vue as a peer dependency. WASM initialization is asynchronous and shared by the underlying browser package. Rendering results from superseded inputs or unmounted components are ignored.

## Component

~~~vue
<script setup lang="ts">
import { Corrplot, type MatrixInput } from "@corrplot-ng/vue";

const matrix: MatrixInput = {
  labels: ["alpha", "beta", "gamma"],
  rows: 3,
  columns: 3,
  values: [
    1, 0.65, -0.25,
    0.65, 1, 0.1,
    -0.25, 0.1, 1,
  ],
  kind: "correlation",
};

function reportError(error: Error) {
  console.error("Could not render correlation plot:", error);
}
</script>

<template>
  <Corrplot
    :matrix="matrix"
    :scene-options="{
      cellMethod: 'circle',
      title: 'Correlation matrix',
      description: 'A three-variable correlation matrix.',
    }"
    :svg-options="{ width: 640, height: 640 }"
    aria-label="Correlation matrix for alpha, beta, and gamma"
    @error="reportError"
  />
</template>
~~~

The component renders inline SVG and emits an error event with a CorrplotError when initialization, validation, scene construction, or rendering fails. It also exposes an accessible role="alert" state while an error is present.

## Composable

Use useCorrplot when the application needs to place the SVG in its own layout or display loading/error state:

~~~vue
<script setup lang="ts">
import { computed, ref } from "vue";
import { useCorrplot, type MatrixInput } from "@corrplot-ng/vue";

const revision = ref(0);
const matrix = computed<MatrixInput>(() => ({
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: revision.value % 2 === 0
    ? [1, 0.6, 0.6, 1]
    : [1, -0.3, -0.3, 1],
  kind: "correlation",
}));

const { svg, loading, error } = useCorrplot(matrix, () => ({
  title: "Revision " + revision.value,
  cellMethod: "ellipse",
}));
</script>

<template>
  <button @click="revision += 1">Update matrix</button>
  <p v-if="loading">Rendering…</p>
  <p v-else-if="error" role="alert">{{ error.message }}</p>
  <div
    v-else-if="svg"
    role="img"
    aria-label="Correlation matrix"
    v-html="svg"
  />
</template>
~~~

Each argument may be a plain value, a ref/computed ref, or a getter. Reactive object and array changes are observed. Replace typed-array values rather than mutating their indexed elements in place, because JavaScript typed-array index writes are not a reliable Vue reactivity signal.

## Compatibility

The package requires Vue 3.3 or later and an evergreen browser with WebAssembly MVP, ES modules, typed arrays, and Promises. The package does not require SharedArrayBuffer, browser threads, Web Workers, or OffscreenCanvas.

## Errors and lifecycle

- WASM initialization and rendering are asynchronous.
- Failures are returned as CorrplotError instances with a diagnostic-v1 record.
- The component emits one error event for each new failed render and exposes an accessible alert state.
- The composable stops its watcher on unmount; late results from superseded requests are discarded.
- The package does not create workers, Canvas contexts, or Node.js resources.

## Browser-only dependency boundary

The only runtime package dependency is @corrplot-ng/wasm; Vue is a peer dependency. No native Node package, Node built-in, or Node-only runtime import is part of the browser dependency graph. Production bundling is checked with Vite and esbuild. Browser smoke tests exercise reactive updates, mount/unmount, error handling, and recovery in Chromium, Firefox, and WebKit.

Public npm publication remains deferred. The package is built and packed as part of the repository's non-publishing workspace and staging workflows.
