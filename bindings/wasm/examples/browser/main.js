import init, {
  getRendererCapabilities,
  renderSceneSvg,
  renderSceneRgba,
  validateMatrix,
} from "../../pkg/corrplot_ng_wasm.js";

const status = document.querySelector("#status");
const figure = document.querySelector("#figure");

const black = { red: 0, green: 0, blue: 0, alpha: 255 };
const white = { red: 255, green: 255, blue: 255, alpha: 255 };

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

  figure.replaceChildren(image, canvas);
  status.textContent = `WASM ready; renderer schema ${capabilities.capability_schema_version}; SVG and RGBA pixel checks passed (${rgba.pixels.byteLength} bytes)`;
} catch (error) {
  status.textContent = `WASM smoke test failed: ${error.message}`;
  status.setAttribute("aria-live", "assertive");
  throw error;
}
