import errors from "./errors.cjs";

export const { CorrplotError } = errors;

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

function isNumericTypedArray(value) {
  if (!ArrayBuffer.isView(value) || value instanceof DataView) return false;
  if (typeof BigInt64Array !== "undefined" && value instanceof BigInt64Array) return false;
  if (typeof BigUint64Array !== "undefined" && value instanceof BigUint64Array) return false;
  return true;
}

function normalizeMatrixInput(input) {
  if (!input || typeof input !== "object") {
    fail("configuration.invalid_field", "configuration", "matrix input must be an object", "matrix");
  }
  const source = input.values;
  let values;
  if (Array.isArray(source)) values = source;
  else if (isNumericTypedArray(source)) values = Array.from(source);
  else fail("configuration.invalid_field", "configuration", "matrix values must be a number/null array or a numeric typed array", "values");

  if (input.missingMask !== undefined) {
    const mask = input.missingMask;
    if (!mask || typeof mask.length !== "number" || mask.length !== values.length) {
      fail("configuration.invalid_field", "configuration", "missingMask length must match matrix values", "missingMask");
    }
    values = values.map((value, index) => {
      if (typeof mask[index] !== "boolean") {
        fail("configuration.invalid_field", "configuration", "missingMask entries must be booleans", `missingMask[${index}]`);
      }
      return mask[index] ? null : value;
    });
  }
  return {
    labels: input.labels,
    rows: input.rows,
    columns: input.columns,
    values,
    kind: input.kind,
  };
}

export function createEngine(adapter, backend) {
  for (const method of ["validateMatrix", "createScene", "renderSceneSvg", "renderSceneRgba", "renderMatrixSvg", "renderMatrixRgba", "getRendererCapabilities"]) {
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
      const result = invoke(() => adapter.validateMatrix(normalizeMatrixInput(input)));
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
    createScene(input, options = {}) {
      ensureActive();
      const normalizedOptions = {
        cellMethod: options.cellMethod ?? "circle",
        title: options.title ?? "",
        description: options.description ?? "",
        annotation: options.annotation ?? "",
        displayType: options.displayType ?? "full",
        showDiagonal: options.showDiagonal ?? true,
      };
      const scene = unwrap(invoke(() => adapter.createScene(normalizeMatrixInput(input), normalizedOptions)), "scene");
      if (!scene || typeof scene !== "object" || !Number.isFinite(scene.width) || !Number.isFinite(scene.height) || !Array.isArray(scene.cells) || !Array.isArray(scene.texts)) {
        fail("renderer.output.invalid_scene", "export", "scene builder returned an invalid renderer-neutral scene", "output.scene");
      }
      return scene;
    },
    render(sceneOrSceneV1Json, options) {
      ensureActive();
      const method = options?.method ?? "circle";
      const isSceneV1Json = typeof sceneOrSceneV1Json === "string";
      if (!isSceneV1Json && (!sceneOrSceneV1Json || typeof sceneOrSceneV1Json !== "object")) {
        fail("configuration.invalid_field", "configuration", "render input must be a renderer-neutral scene or a scene-v1 JSON string", "scene");
      }
      if (options?.format === "svg") {
        const result = invoke(() => isSceneV1Json
          ? adapter.renderMatrixSvg(sceneOrSceneV1Json, method)
          : adapter.renderSceneSvg(sceneOrSceneV1Json, {
              width: options.width ?? sceneOrSceneV1Json.width,
              height: options.height ?? sceneOrSceneV1Json.height,
              unit: options.unit ?? "px",
              fontFamily: options.fontFamily ?? "",
              fontSizePt: options.fontSizePt ?? 10,
              background: options.background ?? { red: 255, green: 255, blue: 255, alpha: 255 },
            }));
        const svg = unwrap(result, "svg");
        if (typeof svg !== "string") fail("renderer.output.format_mismatch", "export", "SVG backend did not return an SVG string", "output.svg");
        return svg;
      }
      if (options?.format === "rgba") {
        const { widthPx, heightPx, dpi, background } = options;
        if (!Number.isInteger(widthPx) || widthPx <= 0 || !Number.isInteger(heightPx) || heightPx <= 0 || !background) {
          fail("configuration.invalid_field", "configuration", "RGBA output requires positive integer widthPx/heightPx and a background color", "output");
        }
        const output = unwrap(invoke(() => isSceneV1Json
          ? adapter.renderMatrixRgba(sceneOrSceneV1Json, method, { widthPx, heightPx, dpi, background })
          : adapter.renderSceneRgba(sceneOrSceneV1Json, { widthPx, heightPx, dpi, background })));
        if (!(output?.pixels instanceof Uint8Array) || output.widthPx !== widthPx || output.heightPx !== heightPx || !Number.isInteger(output.strideBytes) || output.strideBytes < widthPx * 4 || output.pixels.byteLength !== output.strideBytes * heightPx) {
          fail("renderer.output.invalid_buffer", "export", "RGBA backend returned an invalid owned pixel buffer or metadata", "output.pixels");
        }
        return output;
      }
      fail("renderer.unsupported_feature", "renderer", `unsupported output format '${options?.format}'`, "output.format");
    },
    getRendererCapabilities(rendererId) {
      ensureActive();
      const value = invoke(() => adapter.getRendererCapabilities());
      const capabilities = typeof value === "string" ? JSON.parse(value) : value;
      if (rendererId !== undefined && capabilities?.renderer?.id !== rendererId) {
        fail("renderer.unsupported_feature", "renderer", `renderer '${rendererId}' is not available in the selected backend`, "rendererId");
      }
      return capabilities;
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
