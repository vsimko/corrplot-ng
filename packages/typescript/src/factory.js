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

function invoke(operation) {
  try {
    return operation();
  } catch (error) {
    if (error instanceof CorrplotError) throw error;
    if (error?.diagnostic) throw new CorrplotError(error.diagnostic);
    throw error;
  }
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
    if (disposed) fail("configuration.invalid_field", "configuration", "corrplot engine has been disposed", "engine");
  };

  return Object.freeze({
    backend,
    validate(input) {
      ensureActive();
      const result = invoke(() => adapter.validateMatrix(input));
      if (result && typeof result === "object" && "valid" in result) {
        if (!result.valid) {
          if (result.diagnostic) unwrap(result);
          fail("input.matrix.invalid_shape", "input", "matrix validation failed", "matrix");
        }
        return true;
      }
      if (result === false) fail("input.matrix.invalid_shape", "input", "matrix validation failed", "matrix");
      return true;
    },
    render(sceneV1Json, options) {
      ensureActive();
      if (typeof sceneV1Json !== "string") {
        fail("configuration.invalid_field", "configuration", "sceneV1Json must be a JSON string", "sceneV1Json");
      }
      const method = options?.method ?? "circle";
      if (options?.format === "svg") {
        const svg = unwrap(invoke(() => adapter.renderMatrixSvg(sceneV1Json, method)), "svg");
        if (typeof svg !== "string") fail("renderer.output.format_mismatch", "export", "SVG backend did not return an SVG string", "output.svg");
        return svg;
      }
      if (options?.format === "canvas") {
        if (typeof adapter.renderMatrixCanvas !== "function") {
          fail("renderer.unsupported_feature", "renderer", "Canvas output is unavailable in this backend", "output.format");
        }
        return invoke(() => adapter.renderMatrixCanvas(sceneV1Json, method, options));
      }
      if (options?.format === "rgba") {
        const { widthPx, heightPx, dpi, background } = options;
        if (!Number.isInteger(widthPx) || widthPx <= 0 || !Number.isInteger(heightPx) || heightPx <= 0 || !background) {
          fail("configuration.invalid_field", "configuration", "RGBA output requires positive integer widthPx/heightPx and a background color", "output");
        }
        const output = unwrap(invoke(() => adapter.renderMatrixRgba(sceneV1Json, method, { widthPx, heightPx, dpi, background })));
        if (!(output?.pixels instanceof Uint8Array) || output.widthPx !== widthPx || output.heightPx !== heightPx || !Number.isInteger(output.strideBytes) || output.strideBytes < widthPx * 4 || output.pixels.byteLength !== output.strideBytes * heightPx) {
          fail("renderer.output.invalid_buffer", "export", "RGBA backend returned an invalid owned pixel buffer or metadata", "output.pixels");
        }
        return output;
      }
      fail("renderer.unsupported_feature", "renderer", `unsupported output format '${options?.format}'`, "output.format");
    },
    getRendererCapabilities() {
      ensureActive();
      const value = invoke(() => adapter.getRendererCapabilities());
      return typeof value === "string" ? JSON.parse(value) : value;
    },
    dispose() {
      if (disposed) return;
      disposed = true;
      if (adapter.dispose) invoke(() => adapter.dispose());
    },
  });
}

export function backendInitializationFailed(backend, cause) {
  const detail = cause instanceof Error ? cause.message : String(cause);
  fail("configuration.invalid_field", "configuration", `could not initialize '${backend}' backend: ${detail}`, "backend");
}

export function backendUnavailable(requested, available) {
  fail(
    "configuration.invalid_field",
    "configuration",
    `backend '${requested}' is unavailable in this runtime; available backend: '${available}'`,
    "backend",
  );
}
