import init, * as wasm from "@corrplot-ng/browser-wasm";
import { CorrplotError, createEngine, backendUnavailable } from "./factory.js";

export { CorrplotError };

export async function createCorrplotEngine(options = {}) {
  const { backend = "auto", adapter } = options;
  if (adapter) return createEngine(adapter, "injected");
  if (backend !== "auto" && backend !== "wasm") backendUnavailable(backend, "wasm");
  await init();
  return createEngine(wasm, "wasm");
}
