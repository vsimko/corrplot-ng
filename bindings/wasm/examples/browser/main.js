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

try {
  await init();

  const validation = validateMatrix({
    labels: ["alpha", "beta"],
    rows: 2,
    columns: 2,
    values: [1, 0.5, 0.5, 1],
    kind: "correlation",
  });
  if (!validation.valid) {
    throw new Error(validation.diagnostic.message);
  }

  const capabilities = JSON.parse(getRendererCapabilities());
  const svgResult = renderSceneSvg({
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
  }, {
    width: 160,
    height: 160,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  });

  if (svgResult.diagnostic) {
    throw new Error(svgResult.diagnostic.message);
  }
  if (typeof svgResult.svg !== "string" || !svgResult.svg.startsWith("<svg ")) {
    throw new Error("SVG renderer did not return a valid SVG string");
  }

  const image = document.createElement("img");
  image.alt = "A single black circle rendered as SVG by the Rust WebAssembly backend";
  image.src = URL.createObjectURL(new Blob([svgResult.svg], { type: "image/svg+xml" }));

  const rgba = renderSceneRgba({
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
  }, {
    widthPx: 160,
    heightPx: 160,
    dpi: 96,
    background: white,
  });
  if (rgba.diagnostic) {
    throw new Error(rgba.diagnostic.message);
  }
  if (!(rgba.pixels instanceof Uint8Array) ||
      rgba.pixels.byteLength !== rgba.strideBytes * rgba.heightPx) {
    throw new Error("RGBA output did not return a complete Uint8Array buffer");
  }

  const canvas = document.createElement("canvas");
  canvas.width = rgba.widthPx;
  canvas.height = rgba.heightPx;
  canvas.setAttribute("aria-label", "A single black circle rendered as RGBA by the Rust WebAssembly backend");
  const context = canvas.getContext("2d");
  context.putImageData(
    new ImageData(new Uint8ClampedArray(rgba.pixels), rgba.widthPx, rgba.heightPx),
    0,
    0,
  );
  figure.replaceChildren(image, canvas);
  status.textContent = `WASM ready; renderer schema ${capabilities.capability_schema_version}; RGBA buffer ${rgba.pixels.byteLength} bytes`;
} catch (error) {
  status.textContent = `WASM smoke test failed: ${error.message}`;
  status.setAttribute("aria-live", "assertive");
  throw error;
}
