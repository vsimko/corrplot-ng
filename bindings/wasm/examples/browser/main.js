import init, {
  getRendererCapabilities,
  renderSceneSvg,
  renderSceneRgba,
  renderMatrixSvg,
  renderMatrixRgba,
  validateMatrix,
} from "../../pkg/corrplot_ng_wasm.js";

const status = document.querySelector("#status");
const figure = document.querySelector("#figure");

const black = { red: 0, green: 0, blue: 0, alpha: 255 };
const white = { red: 255, green: 255, blue: 255, alpha: 255 };
const cellMethods = ["circle", "square", "ellipse", "number", "shade", "color", "pie"];

function assert(condition, message) {
  if (!condition) throw new Error(message);
}

function assertPixel(pixels, width, x, y, expected, label) {
  const offset = (y * width + x) * 4;
  const actual = Array.from(pixels.slice(offset, offset + 4));
  assert(
    actual.every((channel, index) => channel === expected[index]),
    `${label}: expected rgba(${expected.join(", ")}), got rgba(${actual.join(", ")})`,
  );
}

try {
  await init();

  const validation = validateMatrix({
    labels: ["alpha", "beta"],
    rows: 2,
    columns: 2,
    values: [1, 0.5, 0.5, 1],
    kind: "correlation",
  });
  assert(validation.valid, validation.diagnostic?.message ?? "matrix validation failed");

  const fixtureResponse = await fetch("/prototypes/shared-svg/scene-v1.example.json");
  assert(fixtureResponse.ok, "could not load the canonical scene-v1 fixture");
  const matrixSceneV1 = await fixtureResponse.text();

  const matrixResult = renderMatrixSvg(matrixSceneV1, "ellipse");
  assert(!matrixResult.diagnostic, matrixResult.diagnostic?.message ?? "matrix-to-scene WASM operation failed");
  assert(
    typeof matrixResult.svg === "string" &&
      matrixResult.svg.includes("<circle "),
    "matrix-to-scene WASM operation did not return the expected circle SVG",
  );
  assert(matrixResult.svg.includes(">NA</text>"), "matrix-to-scene WASM operation did not preserve missing values");

  const matrixRgba = renderMatrixRgba(matrixSceneV1, "circle", {
    widthPx: 380,
    heightPx: 392,
    dpi: 96,
    background: white,
  });
  assert(!matrixRgba.diagnostic, matrixRgba.diagnostic?.message ?? "matrix-to-RGBA WASM operation failed");
  assert(
    matrixRgba.pixels instanceof Uint8Array &&
      matrixRgba.pixels.byteLength === matrixRgba.strideBytes * matrixRgba.heightPx &&
      matrixRgba.widthPx === 380 &&
      matrixRgba.heightPx === 392 &&
      matrixRgba.pixelFormat === "rgba8" &&
      matrixRgba.alphaMode === "straight" &&
      matrixRgba.colorSpace === "srgb",
    "matrix-to-RGBA WASM operation returned invalid buffer metadata",
  );
  const matrixCenterOffset = (118 * matrixRgba.widthPx + 146) * 4;
  const matrixCenter = matrixRgba.pixels.subarray(matrixCenterOffset, matrixCenterOffset + 4);
  assert(
    matrixCenter[3] === 255 &&
      (matrixCenter[0] !== 255 || matrixCenter[1] !== 255 || matrixCenter[2] !== 255),
    "matrix-to-RGBA WASM operation did not rasterize the first diagonal cell",
  );

  // Pin the fixture's positive, negative, and missing-cell semantics in the browser.
  assertPixel(matrixRgba.pixels, matrixRgba.widthPx, 146, 118, [33, 102, 172, 255], "positive cell");
  assertPixel(matrixRgba.pixels, matrixRgba.widthPx, 314, 118, [178, 24, 43, 255], "negative cell");
  assertPixel(matrixRgba.pixels, matrixRgba.widthPx, 280, 180, [242, 242, 242, 255], "missing-cell fill");

  // Exercise every advertised cell method in the actual browser runtime.
  for (const method of cellMethods) {
    const svgOutput = renderMatrixSvg(matrixSceneV1, method);
    assert(
      typeof svgOutput.svg === "string" && !svgOutput.diagnostic,
      `${method} matrix SVG failed: ${svgOutput.diagnostic?.message ?? "missing SVG output"}`,
    );
    const rgbaOutput = renderMatrixRgba(matrixSceneV1, method, {
      widthPx: 96,
      heightPx: 96,
      dpi: 96,
      background: white,
    });
    assert(
      rgbaOutput.pixels instanceof Uint8Array &&
        rgbaOutput.pixels.byteLength === rgbaOutput.strideBytes * rgbaOutput.heightPx &&
        rgbaOutput.widthPx === 96 &&
        rgbaOutput.heightPx === 96 &&
        !rgbaOutput.diagnostic,
      `${method} matrix RGBA failed: ${rgbaOutput.diagnostic?.message ?? "invalid pixel buffer"}`,
    );
  }

  const invalidJsonResult = renderMatrixSvg("{", "circle");
  assert(
    invalidJsonResult.diagnostic?.code === "input.scene.invalid_json" &&
      invalidJsonResult.diagnostic?.path === "$" &&
      invalidJsonResult.svg == null,
    "malformed scene-v1 JSON did not return the expected diagnostic-v1",
  );

  const invalidMethodResult = renderMatrixSvg(matrixSceneV1, "unknown");
  const invalidMethodDiagnostic = invalidMethodResult.diagnostic;
  assert(
    invalidMethodDiagnostic?.schema_version === 1 &&
      invalidMethodDiagnostic?.code === "configuration.invalid_field" &&
      invalidMethodDiagnostic?.severity === "error" &&
      invalidMethodDiagnostic?.stage === "configuration" &&
      invalidMethodDiagnostic?.message === "unsupported cell method 'unknown'" &&
      invalidMethodDiagnostic?.path === "cellMethod" &&
      invalidMethodDiagnostic?.row == null &&
      invalidMethodDiagnostic?.column == null &&
      invalidMethodResult.svg == null,
    "matrix-to-scene WASM operation did not return the complete expected diagnostic-v1",
  );

  const capabilities = JSON.parse(getRendererCapabilities());
  const scene = {
    width: 10,
    height: 10,
    cells: [{
      x: 0,
      y: 0,
      size: 10,
      visible: true,
      fill: white,
      stroke: black,
      glyph: { kind: "circle", radius: 3, color: black },
    }],
    texts: [],
  };

  const svgResult = renderSceneSvg(scene, {
    width: 160,
    height: 160,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  });
  assert(!svgResult.diagnostic, svgResult.diagnostic?.message ?? "SVG rendering failed");
  assert(
    typeof svgResult.svg === "string" && svgResult.svg.startsWith("<svg"),
    "SVG renderer did not return an SVG document",
  );

  const rgba = renderSceneRgba(scene, {
    widthPx: 160,
    heightPx: 160,
    dpi: 96,
    background: white,
  });
  assert(!rgba.diagnostic, rgba.diagnostic?.message ?? "RGBA rendering failed");
  assert(
    rgba.pixels instanceof Uint8Array &&
      rgba.pixels.byteLength === rgba.strideBytes * rgba.heightPx,
    "RGBA output did not return a complete Uint8Array buffer",
  );

  // The fixture has a black circle at its center and a white cell interior.
  assertPixel(rgba.pixels, rgba.widthPx, 80, 80, [0, 0, 0, 255], "circle center");
  assertPixel(rgba.pixels, rgba.widthPx, 80, 10, [255, 255, 255, 255], "cell interior");

  // A later WASM call must not invalidate the JS-owned RGBA result.
  const retainedPixels = new Uint8Array(rgba.pixels);
  validateMatrix({
    labels: ["one"],
    rows: 1,
    columns: 1,
    values: [1],
    kind: "correlation",
  });
  assert(
    rgba.pixels.every((channel, index) => channel === retainedPixels[index]),
    "RGBA bytes changed after a subsequent WebAssembly call",
  );

  const image = document.createElement("img");
  image.alt = "A single black circle rendered as SVG by the Rust WebAssembly backend";
  image.src = URL.createObjectURL(new Blob([svgResult.svg], { type: "image/svg+xml" }));

  const canvas = document.createElement("canvas");
  canvas.width = rgba.widthPx;
  canvas.height = rgba.heightPx;
  canvas.setAttribute("aria-label", "A single black circle rendered as RGBA by the Rust WebAssembly backend");
  const context = canvas.getContext("2d");
  assert(context, "2D canvas context is unavailable");
  context.putImageData(
    new ImageData(new Uint8ClampedArray(rgba.pixels), rgba.widthPx, rgba.heightPx),
    0,
    0,
  );
  assertPixel(context.getImageData(80, 80, 1, 1).data, 1, 0, 0, [0, 0, 0, 255], "canvas circle center");

  const matrixCanvas = document.createElement("canvas");
  matrixCanvas.id = "matrix-rgba-canvas";
  matrixCanvas.width = matrixRgba.widthPx;
  matrixCanvas.height = matrixRgba.heightPx;
  matrixCanvas.setAttribute("aria-label", "A 3 by 3 correlation matrix rendered as RGBA by the Rust WebAssembly backend");
  const matrixContext = matrixCanvas.getContext("2d");
  assert(matrixContext, "matrix Canvas 2D context is unavailable");
  matrixContext.putImageData(
    new ImageData(new Uint8ClampedArray(matrixRgba.pixels), matrixRgba.widthPx, matrixRgba.heightPx),
    0,
    0,
  );
  assertPixel(matrixContext.getImageData(146, 118, 1, 1).data, 1, 0, 0, [33, 102, 172, 255], "matrix Canvas positive cell");
  assertPixel(matrixContext.getImageData(314, 118, 1, 1).data, 1, 0, 0, [178, 24, 43, 255], "matrix Canvas negative cell");

  figure.replaceChildren(image, canvas, matrixCanvas);
  status.textContent = `WASM ready; renderer schema ${capabilities.capability_schema_version}; matrix SVG/RGBA semantics, all cell methods, diagnostics, and browser pixel checks passed (${matrixRgba.pixels.byteLength} matrix bytes)`;
} catch (error) {
  status.textContent = `WASM smoke test failed: ${error.message}`;
  status.setAttribute("aria-live", "assertive");
  throw error;
}
