export class CorrplotError extends Error {
  constructor(diagnostic) {
    super(diagnostic.message);
    this.name = "CorrplotError";
    this.diagnostic = Object.freeze({ ...diagnostic });
  }
}

function fail(code, stage, message, path) {
  throw new CorrplotError({
    schema_version: 1,
    code,
    severity: "error",
    stage,
    message,
    ...(path ? { path } : {}),
  });
}

function unwrap(result, field) {
  if (result?.diagnostic) throw new CorrplotError(result.diagnostic);
  if (field === undefined) return result;
  return typeof result === "string" ? result : result?.[field];
}

export function createEngine(adapter, backend) {
  for (const method of ["validateMatrix", "renderMatrixSvg", "renderMatrixRgba", "getRendererCapabilities"]) {
    if (typeof adapter?.[method] !== "function") {
      fail("configuration.invalid_field", "configuration", `backend adapter is missing '${method}'`, `adapter.${method}`);
    }
  }

  let disposed = false;
  const ensureActive = () => {
    if (disposed) fail("runtime.disposed", "runtime", "corrplot engine has been disposed", "engine");
  };

  return Object.freeze({
    backend,
    validate(input) {
      ensureActive();
      const result = adapter.validateMatrix(input);
      if (result && typeof result === "object" && "valid" in result) {
        if (!result.valid) unwrap(result);
        return true;
      }
      if (result === false) fail("input.matrix.invalid", "input", "matrix validation failed", "matrix");
      return true;
    },
    render(sceneV1Json, options) {
      ensureActive();
      if (typeof sceneV1Json !== "string") {
        fail("configuration.invalid_field", "configuration", "sceneV1Json must be a JSON string", "sceneV1Json");
      }
      const method = options?.method ?? "circle";
      if (options?.format === "svg") {
        return unwrap(adapter.renderMatrixSvg(sceneV1Json, method), "svg");
      }
      if (options?.format === "rgba") {
        const { widthPx, heightPx, dpi, background } = options;
        if (!Number.isInteger(widthPx) || widthPx <= 0 || !Number.isInteger(heightPx) || heightPx <= 0 || !background) {
          fail("configuration.invalid_field", "configuration", "RGBA output requires positive integer widthPx/heightPx and a background color", "output");
        }
        const output = unwrap(adapter.renderMatrixRgba(sceneV1Json, method, { widthPx, heightPx, dpi, background }));
        if (!(output?.pixels instanceof Uint8Array)) {
          fail("renderer.output.invalid_buffer", "export", "RGBA backend did not return JavaScript-owned Uint8Array pixels", "output.pixels");
        }
        return output;
      }
      fail("renderer.unsupported_feature", "renderer", `unsupported output format '${options?.format}'`, "output.format");
    },
    getRendererCapabilities() {
      ensureActive();
      const value = adapter.getRendererCapabilities();
      return typeof value === "string" ? JSON.parse(value) : value;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      adapter.dispose?.();
    },
  });
}

export function backendUnavailable(requested, available) {
  fail(
    "configuration.invalid_field",
    "configuration",
    `backend '${requested}' is unavailable in this runtime; available backend: '${available}'`,
    "backend",
  );
}
