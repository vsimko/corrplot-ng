import init, {
  getRendererCapabilities,
  renderSceneSvg,
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
  const svg = renderSceneSvg({
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

  const image = document.createElement("img");
  image.alt = "A single black circle rendered by the Rust WebAssembly backend";
  image.src = URL.createObjectURL(new Blob([svg], { type: "image/svg+xml" }));
  figure.replaceChildren(image);
  status.textContent = `WASM ready; renderer schema ${capabilities.capability_schema_version}`;
} catch (error) {
  status.textContent = `WASM smoke test failed: ${error.message}`;
  status.setAttribute("aria-live", "assertive");
  throw error;
}
