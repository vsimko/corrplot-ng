import assert from "node:assert/strict";
import { chromium } from "playwright";

const browser = await chromium.launch({ channel: "chrome", headless: true });
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
    `browser WASM initialization failed: ${statusText}; page errors: ${pageErrors.join(" | ") || "none"}`,
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

  console.log("Browser WASM smoke passed: SVG decoded and Canvas pixels match.");
} finally {
  await browser.close();
}
