import assert from "node:assert/strict";
import { appendFileSync } from "node:fs";
import { chromium, firefox, webkit } from "playwright";

const browserName = process.env.CORRPLOT_BROWSER ?? "chromium";
const benchmarkOutputPath = process.env.CORRPLOT_BROWSER_BENCHMARK_OUTPUT;
const browserTypes = { chromium, firefox, webkit };
const browserType = browserTypes[browserName];
assert(browserType, `unsupported CORRPLOT_BROWSER "${browserName}"`);

const launchOptions = { headless: true };
if (browserName === "chromium") launchOptions.channel = "chrome";

const browser = await browserType.launch(launchOptions);
try {
  const page = await browser.newPage();
  const pageErrors = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await page.goto(
    "http://127.0.0.1:4173/bindings/wasm/examples/browser/index.html",
    { waitUntil: "networkidle" },
  );
  await page.waitForFunction(
    () => {
      const status = document.querySelector("#status")?.textContent ?? "";
      return status.startsWith("WASM ready;") || status.startsWith("WASM smoke test failed:");
    },
    null,
    { timeout: 30_000 },
  );
  const statusText = await page.locator("#status").textContent();
  assert(
    statusText?.startsWith("WASM ready;"),
    `${browserName} WASM initialization failed: ${statusText}; page errors: ${pageErrors.join(" | ") || "none"}`,
  );

  const svg = page.locator("#figure img");
  await svg.waitFor({ state: "visible" });
  await svg.evaluate((image) => image.decode());

  const canvas = page.locator("#figure canvas").first();
  await canvas.waitFor({ state: "visible" });
  const canvasInfo = await canvas.evaluate((element) => ({
    width: element.width,
    height: element.height,
    center: Array.from(
      element.getContext("2d").getImageData(80, 80, 1, 1).data,
    ),
  }));

  assert.deepEqual(canvasInfo, {
    width: 160,
    height: 160,
    center: [0, 0, 0, 255],
  });
  const matrixCanvas = page.locator("#matrix-rgba-canvas");
  await matrixCanvas.waitFor({ state: "visible" });
  const matrixInfo = await matrixCanvas.evaluate((element) => {
    const context = element.getContext("2d");
    const pixel = (x, y) => Array.from(context.getImageData(x, y, 1, 1).data);
    return {
      width: element.width,
      height: element.height,
      positive: pixel(146, 118),
      negative: pixel(314, 118),
      missing: pixel(280, 180),
    };
  });
  assert.deepEqual(matrixInfo, {
    width: 380,
    height: 392,
    positive: [33, 102, 172, 255],
    negative: [178, 24, 43, 255],
    missing: [242, 242, 242, 255],
  });
  assert.deepEqual(pageErrors, [], "browser page raised an uncaught error");

  const facadePage = await browser.newPage();
  const facadeErrors = [];
  facadePage.on("pageerror", (error) => facadeErrors.push(error.message));
  const facadeUrl = "http://127.0.0.1:4173/packages/typescript/test/browser/index.html" +
    (benchmarkOutputPath ? "?benchmark=1" : "");
  await facadePage.goto(facadeUrl, { waitUntil: "networkidle" });
  await facadePage.waitForFunction(
    () => {
      const status = document.querySelector("#status")?.textContent ?? "";
      return status.startsWith("TypeScript WASM facade ready") || status.startsWith("TypeScript WASM facade failed:");
    },
    null,
    { timeout: 30_000 },
  );
  const facadeStatus = await facadePage.locator("#status").textContent();
  assert(
    facadeStatus?.startsWith("TypeScript WASM facade ready"),
    `${browserName} TypeScript facade failed: ${facadeStatus}; page errors: ${facadeErrors.join(" | ") || "none"}`,
  );
  const facadeResult = await facadePage.evaluate(() => window.__corrplotTypeScriptFacade);
  assert.equal(facadeResult.status, "ready");
  assert.equal(facadeResult.backend, "wasm");
  assert.ok(facadeResult.svgLength > 0);
  assert.equal(facadeResult.widthPx, 64);
  assert.equal(facadeResult.heightPx, 64);
  assert.equal(facadeResult.pixelLength, 64 * 64 * 4);
  assert.equal(facadeResult.canvasWidthPx, 128);
  assert.equal(facadeResult.canvasHeightPx, 128);
  assert.equal(facadeResult.visualSampleCount, 7);
  assert.ok(facadeResult.numberInkPixels > 0, "number glyph visual fixture must contain ink");
  assert.ok(facadeResult.shadeInkPixels > 0, "shade glyph visual fixture must contain ink");
  assert.equal(facadeResult.highDpiSampleCount, 7);
  assert.equal(facadeResult.highDpiWidthPx, 320);
  assert.equal(facadeResult.highDpiHeightPx, 80);
  assert.equal(facadeResult.highDpiScale, 4);
  assert.equal(facadeResult.highDpiOffsetY, 20);
  assert.ok(facadeResult.highDpiNumberInkPixels > 0);
  assert.ok(facadeResult.highDpiShadeInkPixels > 0);
  assert.ok(facadeResult.paintedPixels > 0, "direct Canvas renderer must paint scene pixels");
  assert.ok(facadeResult.canvasHostTargets.includes("browser-canvas-2d"));
  assert.equal(
    facadeResult.offscreenSupported,
    facadeResult.canvasHostTargets.includes("browser-offscreen-canvas-2d"),
    "OffscreenCanvas capability must reflect the current browser runtime",
  );
  if (facadeResult.offscreenSupported) {
    assert.equal(facadeResult.offscreenWidthPx, 64);
    assert.equal(facadeResult.offscreenHeightPx, 64);
  }
  if (benchmarkOutputPath) {
    const metrics = facadeResult.canvasPerformance;
    assert.ok(metrics, "scheduled browser run must produce Canvas/RGBA performance metrics");
    assert.equal(metrics.workload.rows, 64);
    assert.equal(metrics.workload.columns, 64);
    assert.equal(metrics.workload.cells, 4096);
    assert.equal(metrics.outputBytes.canvasBackingStore, 512 * 512 * 4);
    assert.equal(metrics.outputBytes.rgbaBuffer, 512 * 512 * 4);
    assert.equal(metrics.directCanvas2d.iterations, 10);
    assert.equal(metrics.wasmRgba.iterations, 10);
    assert.ok(metrics.directCanvas2d.medianMs > 0);
    assert.ok(metrics.wasmRgba.medianMs > 0);
    const userAgent = await facadePage.evaluate(() => navigator.userAgent);
    appendFileSync(benchmarkOutputPath, JSON.stringify({
      browser: browserName,
      userAgent,
      ...metrics,
    }) + "\n");
  }
  assert.deepEqual(facadeErrors, [], "TypeScript facade page raised an uncaught error");

  const reactPage = await browser.newPage();
  const reactErrors = [];
  reactPage.on("pageerror", error => reactErrors.push(error.message));
  await reactPage.goto(
    "http://127.0.0.1:4173/packages/react/test/browser/vite-dist/index.html",
    { waitUntil: "networkidle" },
  );
  const reactPlot = reactPage.locator('[data-testid="corrplot"]');
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "ready",
    null,
    { timeout: 30_000 },
  );
  let reactSvg = await reactPlot.locator("svg").evaluate(element => element.outerHTML);
  assert.ok(reactSvg.includes("React smoke revision 0"), "React Strict Mode did not render the initial figure");

  await reactPage.locator("#update").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"] svg')?.outerHTML.includes("React smoke revision 1"),
    null,
    { timeout: 15_000 },
  );
  reactSvg = await reactPlot.locator("svg").evaluate(element => element.outerHTML);
  assert.ok(reactSvg.includes("gamma"), "React matrix update did not reach the rendered SVG");

  await reactPage.locator("#toggle").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]') === null,
    null,
    { timeout: 10_000 },
  );
  await reactPage.locator("#toggle").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "ready",
    null,
    { timeout: 30_000 },
  );

  await reactPage.locator("#invalid").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "error",
    null,
    { timeout: 15_000 },
  );
  assert.ok(await reactPage.locator('[data-testid="corrplot"][role="alert"]').count());
  await reactPage.waitForFunction(
    () => document.querySelector("#error-count")?.textContent === "1",
    null,
    { timeout: 10_000 },
  );
  await reactPage.waitForTimeout(100);
  assert.equal(await reactPage.locator("#error-count").textContent(), "1", "onError must not repeat when its callback rerenders the parent");

  await reactPage.locator("#recover").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "ready",
    null,
    { timeout: 30_000 },
  );
  await reactPage.locator("#invalid").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "error",
    null,
    { timeout: 15_000 },
  );
  await reactPage.waitForFunction(
    () => document.querySelector("#error-count")?.textContent === "2",
    null,
    { timeout: 10_000 },
  );
  await reactPage.waitForTimeout(100);
  assert.equal(await reactPage.locator("#error-count").textContent(), "2", "a later error should notify once again");
  await reactPage.locator("#recover").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "ready",
    null,
    { timeout: 30_000 },
  );
  assert.deepEqual(reactErrors, [], "React production integration page raised an uncaught error");

  const strictModePage = await browser.newPage();
  const strictModeErrors = [];
  strictModePage.on("pageerror", error => strictModeErrors.push(error.message));
  await strictModePage.goto("http://127.0.0.1:4174/", { waitUntil: "domcontentloaded" });
  await strictModePage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "ready",
    null,
    { timeout: 30_000 },
  );
  assert.ok(await strictModePage.locator('[data-testid="corrplot"] svg').count());

  await strictModePage.locator("#toggle").click();
  await strictModePage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]') === null,
    null,
    { timeout: 10_000 },
  );
  await strictModePage.locator("#toggle").click();
  await strictModePage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "ready",
    null,
    { timeout: 30_000 },
  );
  assert.deepEqual(strictModeErrors, [], "React Strict Mode development page raised an uncaught error");

  console.log(`${browserName} WASM, TypeScript facade, direct Canvas/OffscreenCanvas, and React production/Strict Mode smoke passed.`);
} finally {
  await browser.close();
}
