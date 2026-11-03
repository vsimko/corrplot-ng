import native from "@corrplot-ng/native-node";
import { CorrplotError, createEngine, backendUnavailable } from "./factory.js";

export { CorrplotError };

export async function createCorrplotEngine(options = {}) {
  const { backend = "auto", adapter } = options;
  if (adapter) return createEngine(adapter, "injected");
  if (backend !== "auto" && backend !== "native") backendUnavailable(backend, "native");
  return createEngine(native, "native");
}
