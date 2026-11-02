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

  const canvas = page.locator("#figure canvas");
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
  assert.deepEqual(pageErrors, [], "browser page raised an uncaught error");

  console.log("Browser WASM smoke passed: SVG decoded and Canvas pixels match.");
} finally {
  await browser.close();
}
