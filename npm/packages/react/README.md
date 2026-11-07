# React integration

@corrplot-ng/react provides a small React adapter for Corrplot-ng's browser WebAssembly backend. It exports a Corrplot component and a useCorrplot hook. The package does not import the Node-API backend or the runtime-selecting TypeScript facade.

The package depends on @corrplot-ng/wasm and declares React as a peer dependency. WASM initialization is asynchronous and shared across component instances. The component starts rendering after mount, ignores results from effects that have been cleaned up, and can safely mount again under React Strict Mode.

## Usage

~~~tsx
import { Corrplot } from "@corrplot-ng/react";

const matrix = {
  labels: ["alpha", "beta", "gamma"],
  rows: 3,
  columns: 3,
  values: [
    1, 0.65, -0.25,
    0.65, 1, 0.1,
    -0.25, 0.1, 1,
  ],
  kind: "correlation" as const,
};

export function CorrelationFigure() {
  return (
    <Corrplot
      matrix={matrix}
      sceneOptions={{
        cellMethod: "circle",
        title: "Correlation matrix",
        description: "A three-variable correlation matrix.",
      }}
      svgOptions={{ width: 640, height: 640 }}
      ariaLabel="Correlation matrix for alpha, beta, and gamma"
    />
  );
}
~~~

The component renders inline SVG. It accepts the same row-major matrix shape and scene-building options as the TypeScript facade. Numeric typed arrays are supported; use missingMask with typed arrays to mark missing cells (true means missing). Ordinary arrays may use null values.

## Hook

Use useCorrplot when the application needs to place the SVG into its own layout or expose loading/error state:

~~~tsx
import { useCorrplot, type MatrixInput } from "@corrplot-ng/react";

function Figure({ matrix }: { matrix: MatrixInput }) {
  const { svg, loading, error } = useCorrplot(matrix, {
    title: "Correlation matrix",
    cellMethod: "ellipse",
  });

  if (loading) return <p>Rendering…</p>;
  if (error) return <p role="alert">{error.message}</p>;
  if (!svg) return null;
  return <div role="img" aria-label="Correlation matrix" dangerouslySetInnerHTML={{ __html: svg }} />;
}
~~~

The hook accepts optional SVG settings as its third argument. Memoize matrix and options objects when they are derived during a component render, as with other effect-driven React hooks.

## Errors and lifecycle

- WASM initialization and rendering are asynchronous.
- Failures are returned as CorrplotError instances with a diagnostic-v1 record.
- The component exposes failures through role="alert" and the optional onError callback.
- Unmounted or superseded effects do not update component state.
- The package does not create workers, Canvas contexts, or Node.js resources.

## Browser-only dependency boundary

The runtime dependency is @corrplot-ng/wasm; React is a peer dependency. No native Node package, Node built-in, or Node-only runtime import is part of the browser dependency graph. Production bundling is checked with Vite and esbuild. The Playwright browser smoke test runs under Chromium, Firefox, and WebKit. It verifies the initial Strict Mode render, matrix updates, unmount/remount, invalid-input error reporting, and recovery.

Public npm publication remains deferred. The package is built and packed as part of the repository's non-publishing workspace and staging workflows.
