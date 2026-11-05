import assert from "node:assert/strict";
import { chromium, firefox, webkit } from "playwright";

const browserName = process.env.CORRPLOT_BROWSER ?? "chromium";
const browserType = { chromium, firefox, webkit }[browserName];
assert.ok(browserType, "Unsupported CORRPLOT_BROWSER: " + browserName);

const launchOptions = { headless: true };
if (browserName === "chromium") launchOptions.channel = "chrome";
const browser = await browserType.launch(launchOptions);
try {
  const page = await browser.newPage();
  const errors = [];
  page.on("pageerror", error => errors.push(error.message));
  await page.goto("http://127.0.0.1:4173/packages/typescript/test/browser/index.html", {
    waitUntil: "domcontentloaded",
  });

  await page.waitForFunction(() => {
    const status = window.__corrplotTypeScriptFacade?.status;
    return status === "ready" || status === "failed";
  }, null, { timeout: 30_000 });

  const result = await page.evaluate(() => window.__corrplotTypeScriptFacade);
  assert.equal(
    result.status,
    "ready",
    "TypeScript browser facade failed: " + (result.error ?? "unknown error"),
  );
  assert.equal(result.backend, "wasm", "browser facade must select the WASM backend");
  assert.ok(result.svgLength > 0, "browser facade did not render SVG");
  assert.ok(result.pixelLength > 0, "browser facade did not return RGBA pixels");
  assert.ok(result.paintedPixels > 0, "direct Canvas renderer did not paint pixels");
  assert.ok(result.canvasWidthPx > 0 && result.canvasHeightPx > 0);
  assert.ok(result.canvasHostTargets.includes("browser-canvas-2d"));
  assert.ok(result.visualSampleCount > 0, "Canvas visual fixture was not checked");
  assert.equal(
    result.highDpiSampleCount,
    result.visualSampleCount,
    "high-DPI Canvas visual fixture was not checked for every sample",
  );
  assert.ok(result.numberInkPixels > 0, "number glyph was not rendered");
  assert.ok(result.shadeInkPixels > 0, "shade glyph was not rendered");
  assert.equal(result.highDpiWidthPx, 320);
  assert.equal(result.highDpiHeightPx, 80);
  assert.ok(result.highDpiNumberInkPixels > 0);
  assert.ok(result.highDpiShadeInkPixels > 0);
  assert.equal(
    await page.locator("#status").textContent(),
    "TypeScript WASM facade ready",
  );

  if (result.offscreenSupported) {
    assert.equal(result.offscreenWidthPx, 64);
    assert.equal(result.offscreenHeightPx, 64);
  }
  assert.deepEqual(errors, [], "TypeScript browser smoke page raised an uncaught browser error");
  console.log(browserName + " TypeScript browser facade and Canvas conformance smoke passed.");
} finally {
  await browser.close();
}
