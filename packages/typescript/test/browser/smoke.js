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

  const canvas = document.createElement("canvas");
  const canvasResult = engine.renderToCanvas(scene, canvas, {
    width: 64,
    height: 64,
    devicePixelRatio: 2,
  });
  const canvasContext = canvas.getContext("2d");
  if (!canvasContext) throw new Error("direct Canvas renderer did not provide a 2D context");
  const canvasPixels = canvasContext.getImageData(0, 0, canvas.width, canvas.height).data;
  let paintedPixels = 0;
  for (let index = 0; index < canvasPixels.length; index += 4) {
    if (canvasPixels[index] < 245 || canvasPixels[index + 1] < 245 || canvasPixels[index + 2] < 245) {
      paintedPixels += 1;
    }
  }
  if (paintedPixels === 0) throw new Error("direct Canvas renderer did not paint any scene pixels");


  const fixtureResponse = await fetch("../fixtures/canvas-conformance.json");
  if (!fixtureResponse.ok) throw new Error("could not load the Canvas visual conformance fixture");
  const visualFixture = await fixtureResponse.json();
  const visualCanvas = document.createElement("canvas");
  engine.renderToCanvas(visualFixture.scene, visualCanvas, {
    width: visualFixture.scene.width,
    height: visualFixture.scene.height,
    devicePixelRatio: 1,
  });
  const visualContext = visualCanvas.getContext("2d");
  if (!visualContext) throw new Error("visual conformance Canvas has no 2D context");
  const visualCanvasPixels = visualContext.getImageData(0, 0, visualCanvas.width, visualCanvas.height).data;
  const visualRgba = engine.render(visualFixture.scene, {
    format: "rgba",
    widthPx: visualFixture.scene.width,
    heightPx: visualFixture.scene.height,
    background: { red: 255, green: 255, blue: 255, alpha: 255 },
  });
  const readPixel = (pixels, stride, x, y) => {
    const offset = y * stride + x * 4;
    return Array.from(pixels.slice(offset, offset + 4));
  };
  for (const sample of visualFixture.expectedSamples) {
    const canvasPixel = readPixel(visualCanvasPixels, visualCanvas.width * 4, sample.x, sample.y);
    const rgbaPixel = readPixel(visualRgba.pixels, visualRgba.strideBytes, sample.x, sample.y);
    if (canvasPixel.some((channel, index) => channel !== sample.color[index]) ||
        rgbaPixel.some((channel, index) => channel !== sample.color[index])) {
      throw new Error("Canvas/RGBA visual fixture mismatch at " + sample.label +
        ": Canvas=" + canvasPixel.join(",") + " RGBA=" + rgbaPixel.join(","));
    }
  }
  const countInk = (pixels, stride, region) => {
    let count = 0;
    for (let y = region.y; y < region.y + region.height; y += 1) {
      for (let x = region.x; x < region.x + region.width; x += 1) {
        const pixel = readPixel(pixels, stride, x, y);
        if (pixel[0] < 230 || pixel[1] < 230 || pixel[2] < 230) count += 1;
      }
    }
    return count;
  };
  const numberInkPixels = countInk(visualCanvasPixels, visualCanvas.width * 4, visualFixture.inkRegions.number);
  const shadeInkPixels = countInk(visualCanvasPixels, visualCanvas.width * 4, visualFixture.inkRegions.shade);
  if (numberInkPixels === 0 || shadeInkPixels === 0) {
    throw new Error("Canvas visual fixture did not render its number and shade glyphs");
  }

  const canvasCapabilities = engine.getCanvasRendererCapabilities();
  if (!canvasCapabilities.host_targets.includes("browser-canvas-2d")) {
    throw new Error("Canvas capability declaration omitted browser-canvas-2d");
  }

  let offscreenSupported = false;
  let offscreenWidthPx = null;
  let offscreenHeightPx = null;
  if (canvasCapabilities.host_targets.includes("browser-offscreen-canvas-2d")) {
    const offscreen = new OffscreenCanvas(1, 1);
    const offscreenResult = engine.renderToCanvas(scene, offscreen, {
      width: 64,
      height: 64,
      devicePixelRatio: 1,
    });
    const offscreenContext = offscreen.getContext("2d");
    if (!offscreenContext) throw new Error("OffscreenCanvas 2D context disappeared after capability detection");
    const offscreenPixels = offscreenContext.getImageData(0, 0, 64, 64).data;
    let offscreenPaintedPixels = 0;
    for (let index = 0; index < offscreenPixels.length; index += 4) {
      if (offscreenPixels[index] < 245 || offscreenPixels[index + 1] < 245 || offscreenPixels[index + 2] < 245) {
        offscreenPaintedPixels += 1;
      }
    }
    if (offscreenPaintedPixels === 0) throw new Error("OffscreenCanvas renderer did not paint scene pixels");
    offscreenSupported = true;
    offscreenWidthPx = offscreenResult.widthPx;
    offscreenHeightPx = offscreenResult.heightPx;
  }

  if (engine.backend !== "wasm") {
    throw new Error("expected WASM backend, received " + engine.backend);
  }
  engine.dispose();
  window.__corrplotTypeScriptFacade = {
    status: "ready",
    backend: engine.backend,
    svgLength: svg.length,
    widthPx: rgba.widthPx,
    heightPx: rgba.heightPx,
    pixelLength: rgba.pixels.length,
    canvasWidthPx: canvasResult.widthPx,
    canvasHeightPx: canvasResult.heightPx,
    visualSampleCount: visualFixture.expectedSamples.length,
    numberInkPixels,
    shadeInkPixels,
    paintedPixels,
    canvasHostTargets: canvasCapabilities.host_targets,
    offscreenSupported,
    offscreenWidthPx,
    offscreenHeightPx,
  };
  status.textContent = "TypeScript WASM facade ready";
} catch (error) {
  window.__corrplotTypeScriptFacade = { status: "failed", error: error?.stack ?? String(error) };
  status.textContent = "TypeScript WASM facade failed: " + (error?.message ?? error);
  throw error;
}
