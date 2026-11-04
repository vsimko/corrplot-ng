import { createCorrplotEngine, CorrplotError } from "../../src/browser.js";

const status = document.querySelector("#status");

try {
  let unavailableBackendError;
  try {
    await createCorrplotEngine({ backend: "native" });
  } catch (error) {
    unavailableBackendError = error;
  }
  if (!(unavailableBackendError instanceof CorrplotError) ||
      unavailableBackendError.diagnostic.path !== "backend") {
    throw new Error("browser facade did not report an explicit native-backend request as unavailable");
  }

  const engine = await createCorrplotEngine({ backend: "auto" });
  const matrix = {
    labels: ["alpha", "beta"],
    rows: 2,
    columns: 2,
    values: [1, -0.5, -0.5, 1],
    kind: "correlation",
  };
  const scene = engine.createScene(matrix, {
    cellMethod: "ellipse",
    title: "Browser facade",
    description: "Created by the unified TypeScript API.",
  });
  const svg = engine.render(scene, { format: "svg" });
  const rgba = engine.render(scene, {
    format: "rgba",
    widthPx: 64,
    heightPx: 64,
    background: { red: 255, green: 255, blue: 255, alpha: 255 },
  });
  if (!svg.startsWith("<svg ") || !svg.includes("<ellipse ")) {
    throw new Error("browser facade did not render the expected SVG");
  }
  if (!(rgba.pixels instanceof Uint8Array) || rgba.pixels.byteLength !== rgba.strideBytes * rgba.heightPx) {
    throw new Error("browser facade returned invalid RGBA pixels");
  }
  if (engine.backend !== "wasm") {
    throw new Error(`expected WASM backend, received ${engine.backend}`);
  }
  engine.dispose();
  window.__corrplotTypeScriptFacade = {
    status: "ready",
    backend: engine.backend,
    svgLength: svg.length,
    widthPx: rgba.widthPx,
    heightPx: rgba.heightPx,
    pixelLength: rgba.pixels.length,
  };
  status.textContent = "TypeScript WASM facade ready";
} catch (error) {
  window.__corrplotTypeScriptFacade = { status: "failed", error: error?.stack ?? String(error) };
  status.textContent = `TypeScript WASM facade failed: ${error?.message ?? error}`;
  throw error;
}
