import { CorrplotError, createEngine, backendUnavailable, backendInitializationFailed } from "./factory.js";

export { CorrplotError };

export async function createCorrplotEngine(options = {}) {
  const { backend = "auto", adapter } = options;
  if (adapter) return createEngine(adapter, "injected");
  if (backend !== "auto" && backend !== "native") backendUnavailable(backend, "native");
  try {
    const module = await import("@corrplot-ng/native-node");
    return createEngine(module.default ?? module, "native");
  } catch (error) {
    backendInitializationFailed("native", error);
  }
}
