import { CorrplotError, createEngine, backendUnavailable, backendInitializationFailed } from "./factory.js";
import { drawCanvasCommands } from "./canvas.js";

export { CorrplotError };

export async function createCorrplotEngine(options = {}) {
  const { backend = "auto", adapter } = options;
  if (adapter) return createEngine(adapter, "injected");
  if (backend !== "auto" && backend !== "wasm") backendUnavailable(backend, "wasm");
  try {
    const wasm = await import("@corrplot-ng/browser-wasm");
    await wasm.default();
    const browserAdapter = {
      ...wasm,
      renderMatrixCanvas(sceneV1Json, method, renderOptions) {
        const result = wasm.renderMatrixCanvasCommands(sceneV1Json, method);
        if (result?.diagnostic) throw new CorrplotError(result.diagnostic);
        if (!result?.scene) {
          throw new CorrplotError({
            schema_version: 1,
            code: "renderer.output.format_mismatch",
            severity: "error",
            stage: "export",
            message: "WASM Canvas command export did not return a scene",
            path: "output.scene",
          });
        }
        return drawCanvasCommands(result.scene, renderOptions);
      },
      getRendererCapabilities() {
        const capabilities = JSON.parse(wasm.getRendererCapabilities());
        return {
          ...capabilities,
          output_formats: [...new Set([...(capabilities.output_formats ?? []), "canvas"])],
        };
      },
    };
    return createEngine(browserAdapter, "wasm");
  } catch (error) {
    backendInitializationFailed("wasm", error);
  }
}
