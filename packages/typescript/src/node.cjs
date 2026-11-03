const { CorrplotError } = require("./errors.cjs");

function normalizeError(error) {
  if (error instanceof CorrplotError) return error;
  if (error?.diagnostic) return new CorrplotError(error.diagnostic);
  return error;
}

function wrapEngine(engine) {
  return Object.freeze({
    backend: engine.backend,
    validate(input) {
      try { return engine.validate(input); } catch (error) { throw normalizeError(error); }
    },
    render(input, options) {
      try { return engine.render(input, options); } catch (error) { throw normalizeError(error); }
    },
    getRendererCapabilities() {
      try { return engine.getRendererCapabilities(); } catch (error) { throw normalizeError(error); }
    },
    dispose() {
      try { return engine.dispose(); } catch (error) { throw normalizeError(error); }
    },
  });
}

async function createCorrplotEngine(options = {}) {
  const { createEngine, backendUnavailable, backendInitializationFailed } = await import("./factory.js");
  const { backend = "auto", adapter } = options;
  try {
    if (adapter) return wrapEngine(createEngine(adapter, "injected"));
    if (backend !== "auto" && backend !== "native") backendUnavailable(backend, "native");
    const module = await import("@corrplot-ng/native-node");
    return wrapEngine(createEngine(module.default ?? module, "native"));
  } catch (error) {
    const normalized = normalizeError(error);
    if (normalized !== error) throw normalized;
    if (error?.diagnostic) throw normalized;
    throw error;
  }
}

module.exports = { createCorrplotEngine, CorrplotError };
