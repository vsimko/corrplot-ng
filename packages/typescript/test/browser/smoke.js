import { createCorrplotEngine, CorrplotError } from "../../src/browser.js";

const status = document.querySelector("#status");
const benchmarkEnabled = new URLSearchParams(window.location.search).get("benchmark") === "1";

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
  const statisticalScene = {
    width: 100,
    height: 50,
    cells: [
      {
        x: 0, y: 0, size: 20, visible: true,
        fill: { red: 255, green: 255, blue: 255, alpha: 255 },
        stroke: { red: 0, green: 0, blue: 0, alpha: 255 },
        confidenceInterval: {
          lower: 0.2, upper: 0.8, x1: 4, x2: 16, y: 10,
          capHeight: 4, strokeWidth: 2,
          color: { red: 220, green: 20, blue: 60, alpha: 255 },
        },
        significance: { pValue: 0.01, status: "significant", annotation: "**", blank: false },
      },
      {
        x: 20, y: 0, size: 20, visible: true,
        fill: { red: 255, green: 255, blue: 255, alpha: 255 },
        stroke: { red: 0, green: 0, blue: 0, alpha: 255 },
        glyph: { kind: "circle", radius: 4, color: { red: 0, green: 0, blue: 0, alpha: 255 } },
        significance: { status: "unknown", annotation: "?", blank: true },
        confidenceInterval: {
          lower: -0.5, upper: 0.5, x1: 24, x2: 36, y: 10,
          capHeight: 4, strokeWidth: 2,
          color: { red: 220, green: 20, blue: 60, alpha: 255 },
        },
      },
      {
        x: 40, y: 0, size: 20, visible: false,
        fill: { red: 255, green: 255, blue: 255, alpha: 255 },
        stroke: { red: 0, green: 0, blue: 0, alpha: 255 },
        significance: { pValue: 0.5, status: "not_significant", annotation: "ns", blank: false },
      },
    ],
    texts: [],
    significance: { mode: "stars", thresholds: [0.05, 0.01], symbol: "*" },
  };
  const overlaySvg = engine.render(statisticalScene, { format: "svg" });
  if (!overlaySvg.includes('class="confidence-interval"') ||
      !overlaySvg.includes('class="significance-annotation"')) {
    throw new Error("WASM typed-scene adapter dropped statistical overlays");
  }
  for (const invalidScene of [
    { ...statisticalScene, significance: { mode: "unsupported", thresholds: [0.05], symbol: "*" } },
    { ...statisticalScene, cells: [{ ...statisticalScene.cells[0], pValue: 1.5 }] },
  ]) {
    let rejected = false;
    try {
      engine.render(invalidScene, { format: "svg" });
    } catch (error) {
      rejected = error instanceof CorrplotError &&
        error.diagnostic.code === "renderer.output.invalid_options";
    }
    if (!rejected) throw new Error("WASM adapter dropped or failed to validate statistical metadata");
  }
  const overlayRgba = engine.render(statisticalScene, {
    format: "rgba",
    widthPx: 100,
    heightPx: 50,
    background: { red: 255, green: 255, blue: 255, alpha: 255 },
  });
  const overlayIntervalOffset = (10 * overlayRgba.widthPx + 10) * 4;
  if (overlayRgba.pixels[overlayIntervalOffset] !== 220 ||
      overlayRgba.pixels[overlayIntervalOffset + 1] !== 20 ||
      overlayRgba.pixels[overlayIntervalOffset + 2] !== 60 ||
      overlayRgba.pixels[(10 * overlayRgba.widthPx + 30) * 4] === 220) {
    throw new Error("WASM RGBA renderer did not preserve statistical overlay and blank-cell semantics");
  }
  const overlayCanvas = document.createElement("canvas");
  engine.renderToCanvas(statisticalScene, overlayCanvas, {
    width: 100, height: 50, devicePixelRatio: 1,
  });
  const overlayPixels = overlayCanvas.getContext("2d").getImageData(0, 0, 100, 50).data;
  const intervalPixelOffset = (10 * 100 + 10) * 4;
  if (overlayPixels[intervalPixelOffset] !== 220 ||
      overlayPixels[intervalPixelOffset + 1] !== 20 ||
      overlayPixels[intervalPixelOffset + 2] !== 60) {
    throw new Error("Canvas did not render the supplied confidence interval");
  }
  if (overlayPixels[(10 * 100 + 30) * 4] === 220) {
    throw new Error("blank cell unexpectedly rendered its confidence interval");
  }
  const countDarkInk = (pixels, width, region) => {
    let count = 0;
    for (let y = region.y; y < region.y + region.height; y += 1) {
      for (let x = region.x; x < region.x + region.width; x += 1) {
        const offset = (y * width + x) * 4;
        if (pixels[offset] < 60 && pixels[offset + 1] < 60 && pixels[offset + 2] < 60) count += 1;
      }
    }
    return count;
  };
  if (countDarkInk(overlayPixels, 100, { x: 11, y: 1, width: 8, height: 8 }) === 0) {
    throw new Error("Canvas did not render the supplied significance annotation");
  }
  const highDpiOverlayCanvas = document.createElement("canvas");
  engine.renderToCanvas(statisticalScene, highDpiOverlayCanvas, {
    width: 100, height: 50, devicePixelRatio: 2,
  });
  const highDpiOverlayPixels = highDpiOverlayCanvas.getContext("2d").getImageData(
    0, 0, highDpiOverlayCanvas.width, highDpiOverlayCanvas.height,
  ).data;
  const highDpiIntervalOffset = (20 * highDpiOverlayCanvas.width + 20) * 4;
  if (highDpiOverlayPixels[highDpiIntervalOffset] !== 220 ||
      highDpiOverlayPixels[highDpiIntervalOffset + 1] !== 20 ||
      highDpiOverlayPixels[highDpiIntervalOffset + 2] !== 60 ||
      countDarkInk(highDpiOverlayPixels, highDpiOverlayCanvas.width, {
        x: 22, y: 2, width: 16, height: 14,
      }) === 0) {
    throw new Error("2x Canvas did not preserve confidence interval and annotation ink");
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

  // Exercise the same visual contract at a larger logical viewport and 2x
  // backing-store resolution. The narrower viewport also verifies centered
  // letterboxing rather than cropping.
  const highDpiCanvas = document.createElement("canvas");
  const highDpiResult = engine.renderToCanvas(visualFixture.scene, highDpiCanvas, {
    width: visualFixture.scene.width * 2,
    height: visualFixture.scene.height * 4,
    devicePixelRatio: 2,
  });
  const highDpiContext = highDpiCanvas.getContext("2d");
  if (!highDpiContext) throw new Error("high-DPI conformance Canvas has no 2D context");
  const highDpiCanvasPixels = highDpiContext.getImageData(0, 0, highDpiCanvas.width, highDpiCanvas.height).data;
  const highDpiRgba = engine.render(visualFixture.scene, {
    format: "rgba",
    widthPx: highDpiResult.widthPx,
    heightPx: highDpiResult.heightPx,
    background: { red: 255, green: 255, blue: 255, alpha: 255 },
  });
  if (highDpiResult.widthPx !== 320 || highDpiResult.heightPx !== 240 ||
      highDpiRgba.widthPx !== highDpiResult.widthPx ||
      highDpiRgba.heightPx !== highDpiResult.heightPx) {
    throw new Error("high-DPI Canvas/RGBA output dimensions did not match the requested viewport");
  }
  const highDpiScale = Math.min(
    highDpiResult.widthPx / visualFixture.scene.width,
    highDpiResult.heightPx / visualFixture.scene.height,
  );
  const highDpiOffsetX = (highDpiResult.widthPx - visualFixture.scene.width * highDpiScale) / 2;
  const highDpiOffsetY = (highDpiResult.heightPx - visualFixture.scene.height * highDpiScale) / 2;
  for (const sample of visualFixture.expectedSamples) {
    const x = Math.floor(highDpiOffsetX + sample.x * highDpiScale);
    const y = Math.floor(highDpiOffsetY + sample.y * highDpiScale);
    const canvasPixel = readPixel(highDpiCanvasPixels, highDpiCanvas.width * 4, x, y);
    const rgbaPixel = readPixel(highDpiRgba.pixels, highDpiRgba.strideBytes, x, y);
    if (canvasPixel.some((channel, index) => channel !== sample.color[index]) ||
        rgbaPixel.some((channel, index) => channel !== sample.color[index])) {
      throw new Error("high-DPI Canvas/RGBA visual fixture mismatch at " + sample.label +
        ": Canvas=" + canvasPixel.join(",") + " RGBA=" + rgbaPixel.join(","));
    }
  }
  const scaleRegion = region => ({
    x: Math.floor(highDpiOffsetX + region.x * highDpiScale),
    y: Math.floor(highDpiOffsetY + region.y * highDpiScale),
    width: Math.ceil(region.width * highDpiScale),
    height: Math.ceil(region.height * highDpiScale),
  });
  const highDpiNumberInkPixels = countInk(
    highDpiCanvasPixels, highDpiCanvas.width * 4, scaleRegion(visualFixture.inkRegions.number),
  );
  const highDpiShadeInkPixels = countInk(
    highDpiCanvasPixels, highDpiCanvas.width * 4, scaleRegion(visualFixture.inkRegions.shade),
  );
  if (highDpiNumberInkPixels === 0 || highDpiShadeInkPixels === 0) {
    throw new Error("high-DPI Canvas fixture did not render its number and shade glyphs");
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

  let canvasPerformance = null;
  if (benchmarkEnabled) {
    const benchmarkSize = 64;
    const benchmarkLabels = Array.from(
      { length: benchmarkSize },
      (_, index) => "variable-" + String(index + 1).padStart(2, "0"),
    );
    const benchmarkValues = Array.from({ length: benchmarkSize * benchmarkSize }, (_, index) => {
      const row = Math.floor(index / benchmarkSize);
      const column = index % benchmarkSize;
      if (row === column) return 1;
      const first = Math.min(row, column);
      const second = Math.max(row, column);
      return ((first * 31 + second * 17) % 201 - 100) / 200;
    });
    const benchmarkScene = engine.createScene({
      labels: benchmarkLabels,
      rows: benchmarkSize,
      columns: benchmarkSize,
      values: benchmarkValues,
      kind: "correlation",
    }, {
      cellMethod: "ellipse",
      title: "Browser Canvas performance fixture",
      description: "Deterministic 64-variable correlation matrix.",
      displayType: "upper",
      showDiagonal: false,
    });
    if (!benchmarkScene || !Array.isArray(benchmarkScene.cells)) {
      throw new Error("could not construct browser Canvas performance fixture");
    }
    const benchmarkWidthPx = 512;
    const benchmarkHeightPx = 512;
    const benchmarkCanvas = document.createElement("canvas");
    const renderCanvas = () => engine.renderToCanvas(benchmarkScene, benchmarkCanvas, {
      width: benchmarkWidthPx,
      height: benchmarkHeightPx,
      devicePixelRatio: 1,
    });
    const renderRgba = () => engine.render(benchmarkScene, {
      format: "rgba",
      widthPx: benchmarkWidthPx,
      heightPx: benchmarkHeightPx,
      background: { red: 255, green: 255, blue: 255, alpha: 255 },
    });
    const measureBrowserOperation = (operation, iterations = 10, warmupIterations = 3) => {
      for (let index = 0; index < warmupIterations; index += 1) operation();
      const samples = [];
      for (let index = 0; index < iterations; index += 1) {
        const start = performance.now();
        operation();
        samples.push(performance.now() - start);
      }
      const sorted = [...samples].sort((left, right) => left - right);
      const percentile = p => {
        const position = (sorted.length - 1) * p;
        const lower = Math.floor(position);
        const upper = Math.ceil(position);
        return lower === upper
          ? sorted[lower]
          : sorted[lower] + (sorted[upper] - sorted[lower]) * (position - lower);
      };
      return {
        iterations,
        warmupIterations,
        medianMs: Number(percentile(0.5).toFixed(4)),
        iqrMs: Number((percentile(0.75) - percentile(0.25)).toFixed(4)),
        p95Ms: Number(percentile(0.95).toFixed(4)),
        minMs: Number(sorted[0].toFixed(4)),
        maxMs: Number(sorted[sorted.length - 1].toFixed(4)),
        samplesMs: samples.map(value => Number(value.toFixed(4))),
      };
    };
    const canvasTiming = measureBrowserOperation(renderCanvas);
    const rgbaTiming = measureBrowserOperation(renderRgba);
    const canvasOutput = renderCanvas();
    const rgbaOutput = renderRgba();
    if (canvasOutput.widthPx !== benchmarkWidthPx ||
        canvasOutput.heightPx !== benchmarkHeightPx ||
        rgbaOutput.widthPx !== benchmarkWidthPx ||
        rgbaOutput.heightPx !== benchmarkHeightPx ||
        rgbaOutput.pixels.byteLength !== benchmarkWidthPx * benchmarkHeightPx * 4) {
      throw new Error("browser Canvas/RGBA benchmark output dimensions are inconsistent");
    }
    canvasPerformance = {
      workload: {
        rows: benchmarkSize,
        columns: benchmarkSize,
        cells: benchmarkScene.cells.length,
        widthPx: benchmarkWidthPx,
        heightPx: benchmarkHeightPx,
        devicePixelRatio: 1,
      },
      outputBytes: {
        canvasBackingStore: benchmarkCanvas.width * benchmarkCanvas.height * 4,
        rgbaBuffer: rgbaOutput.pixels.byteLength,
      },
      directCanvas2d: canvasTiming,
      wasmRgba: rgbaTiming,
    };
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
    highDpiSampleCount: visualFixture.expectedSamples.length,
    highDpiNumberInkPixels,
    highDpiShadeInkPixels,
    highDpiWidthPx: highDpiResult.widthPx,
    highDpiHeightPx: highDpiResult.heightPx,
    highDpiScale,
    highDpiOffsetY,
    canvasPerformance,
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
