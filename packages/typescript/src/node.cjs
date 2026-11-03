const { CorrplotError } = require("./errors.cjs");

async function createCorrplotEngine(options = {}) {
  const { createEngine, backendUnavailable, backendInitializationFailed } = await import("./factory.js");
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

module.exports = { createCorrplotEngine, CorrplotError };
