import { CorrplotError, createEngine, backendUnavailable, backendInitializationFailed } from "./factory.js";
import { renderSceneToCanvas, getCanvasRendererCapabilities } from "./canvas.js";

export { CorrplotError };

const canvasRenderer = Object.freeze({
  render: renderSceneToCanvas,
  getCapabilities: getCanvasRendererCapabilities,
});

export async function createCorrplotEngine(options = {}) {
  const { backend = "auto", adapter } = options;
  if (adapter) return createEngine(adapter, "injected", canvasRenderer);
  if (backend !== "auto" && backend !== "wasm") backendUnavailable(backend, "wasm");
  try {
    const wasm = await import("@corrplot-ng/wasm");
    await wasm.default();
    return createEngine(wasm, "wasm", canvasRenderer);
  } catch (error) {
    backendInitializationFailed("wasm", error);
  }
}
