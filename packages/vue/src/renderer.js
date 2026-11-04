const DEFAULT_BACKGROUND = Object.freeze({ red: 255, green: 255, blue: 255, alpha: 255 });

function diagnostic(code, stage, message, path) {
  return {
    schema_version: 1,
    code,
    severity: "error",
    stage,
    message,
    ...(path ? { path } : {}),
  };
}

export class CorrplotError extends Error {
  constructor(value) {
    const record = value && typeof value === "object" ? value : diagnostic(
      "configuration.invalid_field",
      "configuration",
      String(value),
      "backend",
    );
    super(record.message ?? "Corrplot operation failed");
    this.name = "CorrplotError";
    this.diagnostic = record;
  }
}

function asCorrplotError(error, stage = "configuration", path = "backend") {
  if (error instanceof CorrplotError) return error;
  if (error?.diagnostic && typeof error.diagnostic === "object") {
    return new CorrplotError(error.diagnostic);
  }
  const message = error instanceof Error ? error.message : String(error);
  return new CorrplotError(diagnostic(
    "configuration.invalid_field",
    stage,
    message,
    path,
  ));
}

function fail(code, stage, message, path) {
  throw new CorrplotError(diagnostic(code, stage, message, path));
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
  if (Array.isArray(source)) values = source.slice();
  else if (isNumericTypedArray(source)) values = Array.from(source);
  else fail(
    "configuration.invalid_field",
    "configuration",
    "matrix values must be a number/null array or a numeric typed array",
    "values",
  );

  if (input.missingMask !== undefined) {
    const mask = input.missingMask;
    if (!mask || typeof mask.length !== "number" || mask.length !== values.length) {
      fail("configuration.invalid_field", "configuration", "missingMask length must match matrix values", "missingMask");
    }
    values = values.map((value, index) => {
      if (typeof mask[index] !== "boolean") {
        fail(
          "configuration.invalid_field",
          "configuration",
          "missingMask entries must be booleans",
          `missingMask[${index}]`,
        );
      }
      return mask[index] ? null : value;
    });
  }

  for (let index = 0; index < values.length; index += 1) {
    if (values[index] !== null && typeof values[index] !== "number") {
      fail(
        "configuration.invalid_field",
        "configuration",
        "matrix values must be numbers or null",
        `values[${index}]`,
      );
    }
  }

  return {
    labels: input.labels,
    rows: input.rows,
    columns: input.columns,
    values,
    kind: input.kind,
  };
}

function normalizeSceneOptions(options = {}) {
  return {
    cellMethod: options.cellMethod ?? "circle",
    title: options.title ?? "",
    description: options.description ?? "",
    annotation: options.annotation ?? "",
    displayType: options.displayType ?? "full",
    showDiagonal: options.showDiagonal ?? true,
  };
}

function normalizeSvgOptions(scene, options = {}) {
  const width = options.width ?? scene.width;
  const height = options.height ?? scene.height;
  const fontSizePt = options.fontSizePt ?? 10;

  if (!Number.isFinite(width) || width <= 0 || !Number.isFinite(height) || height <= 0) {
    fail("configuration.invalid_field", "configuration", "SVG width and height must be positive finite numbers", "output");
  }
  if (!Number.isFinite(fontSizePt) || fontSizePt <= 0) {
    fail("configuration.invalid_field", "configuration", "fontSizePt must be a positive finite number", "output.fontSizePt");
  }

  return {
    width,
    height,
    unit: options.unit ?? "px",
    fontFamily: options.fontFamily ?? "sans-serif",
    fontSizePt,
    background: options.background ?? DEFAULT_BACKGROUND,
  };
}

let wasmModulePromise;

async function loadWasmModule() {
  if (!wasmModulePromise) {
    wasmModulePromise = import("@corrplot-ng/wasm")
      .then(async wasm => {
        if (typeof wasm.default !== "function") {
          throw new Error("the browser WASM package does not expose its initializer");
        }
        await wasm.default();
        return wasm;
      })
      .catch(error => {
        wasmModulePromise = undefined;
        throw asCorrplotError(error, "configuration", "backend");
      });
  }
  return wasmModulePromise;
}

export async function renderMatrixToSvg(input, sceneOptions = {}, svgOptions = {}, loadWasm = loadWasmModule) {
  const matrix = normalizeMatrixInput(input);

  let wasm;
  try {
    wasm = await loadWasm();
  } catch (error) {
    throw asCorrplotError(error, "configuration", "backend");
  }

  for (const method of ["validateMatrix", "createScene", "renderSceneSvg"]) {
    if (typeof wasm?.[method] !== "function") {
      fail(
        "configuration.invalid_field",
        "configuration",
        `browser WASM backend is missing '${method}'`,
        `backend.${method}`,
      );
    }
  }

  let validation;
  try {
    validation = wasm.validateMatrix(matrix);
  } catch (error) {
    throw asCorrplotError(error, "input", "matrix");
  }
  if (!validation?.valid) {
    throw new CorrplotError(validation?.diagnostic ?? diagnostic(
      "input.matrix.invalid_shape",
      "input",
      "matrix validation failed",
      "matrix",
    ));
  }

  let sceneResult;
  try {
    sceneResult = wasm.createScene(matrix, normalizeSceneOptions(sceneOptions));
  } catch (error) {
    throw asCorrplotError(error, "input", "matrix");
  }
  if (sceneResult?.diagnostic) throw new CorrplotError(sceneResult.diagnostic);
  const scene = sceneResult?.scene;
  if (!scene || typeof scene !== "object") {
    fail("renderer.output.invalid_scene", "export", "browser WASM backend did not return a scene", "output.scene");
  }

  let svgResult;
  try {
    svgResult = wasm.renderSceneSvg(scene, normalizeSvgOptions(scene, svgOptions));
  } catch (error) {
    throw asCorrplotError(error, "renderer", "output");
  }
  if (svgResult?.diagnostic) throw new CorrplotError(svgResult.diagnostic);

  const svg = typeof svgResult === "string" ? svgResult : svgResult?.svg;
  if (typeof svg !== "string" || !svg.startsWith("<svg")) {
    fail("renderer.output.format_mismatch", "export", "browser WASM backend did not return an SVG document", "output.svg");
  }
  return svg;
}
