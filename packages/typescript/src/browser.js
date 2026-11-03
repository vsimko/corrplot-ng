import { CorrplotError, createEngine, backendUnavailable, backendInitializationFailed } from "./factory.js";
import { createBrowserAdapter } from "./browser-adapter.js";

export { CorrplotError };

export async function createCorrplotEngine(options = {}) {
  const { backend = "auto", adapter } = options;
  if (adapter) return createEngine(adapter, "injected");
  if (backend !== "auto" && backend !== "wasm") backendUnavailable(backend, "wasm");
  try {
    const wasm = await import("@corrplot-ng/browser-wasm");
    await wasm.default();
    return createEngine(createBrowserAdapter(wasm), "wasm");
  } catch (error) {
    backendInitializationFailed("wasm", error);
  }
}
