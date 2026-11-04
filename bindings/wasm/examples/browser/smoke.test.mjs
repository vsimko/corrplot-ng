import assert from "node:assert/strict";
import { chromium, firefox, webkit } from "playwright";

const browserName = process.env.CORRPLOT_BROWSER ?? "chromium";
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
  await facadePage.goto(
    "http://127.0.0.1:4173/packages/typescript/test/browser/index.html",
    { waitUntil: "networkidle" },
  );
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
  assert.deepEqual(facadeErrors, [], "TypeScript facade page raised an uncaught error");

  const reactPage = await browser.newPage();
  const reactErrors = [];
  reactPage.on("pageerror", error => reactErrors.push(error.message));
  await reactPage.goto(
    "http://127.0.0.1:4174/",
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
  await reactPage.locator("#recover").click();
  await reactPage.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === "ready",
    null,
    { timeout: 30_000 },
  );
  assert.deepEqual(reactErrors, [], "React integration page raised an uncaught error");

  console.log(`${browserName} WASM, TypeScript facade, and React integration smoke passed.`);
} finally {
  await browser.close();
}
