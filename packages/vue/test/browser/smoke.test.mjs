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
  await page.goto("http://127.0.0.1:4175/", { waitUntil: "domcontentloaded" });

  const plot = page.locator('[data-testid="corrplot"]');
  const waitForStatus = status => page.waitForFunction(
    expected => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === expected,
    status,
    { timeout: 30_000 },
  );

  await waitForStatus("ready");
  let svg = await plot.locator("svg").evaluate(element => element.outerHTML);
  assert.ok(svg.includes("Vue smoke revision 0"), "Vue component did not render the initial figure");
  await page.waitForFunction(
    () => document.querySelector("#hook-status")?.getAttribute("data-status") === "ready" &&
      Number(document.querySelector("#hook-status")?.getAttribute("data-svg-length")) > 0,
    null,
    { timeout: 30_000 },
  );

  await page.locator("#update").click();
  await page.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"] svg')?.outerHTML.includes("Vue smoke revision 1"),
    null,
    { timeout: 15_000 },
  );
  svg = await plot.locator("svg").evaluate(element => element.outerHTML);
  assert.ok(svg.includes("gamma"), "Vue reactive matrix update did not reach the rendered SVG");

  await page.locator("#toggle").click();
  await page.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]') === null,
    null,
    { timeout: 10_000 },
  );
  await page.locator("#toggle").click();
  await waitForStatus("ready");

  await page.locator("#invalid").click();
  await waitForStatus("error");
  assert.ok(await page.locator('[data-testid="corrplot"][role="alert"]').count());
  await page.waitForFunction(
    () => document.querySelector("#error-count")?.textContent === "1",
    null,
    { timeout: 10_000 },
  );
  await page.waitForTimeout(100);
  assert.equal(await page.locator("#error-count").textContent(), "1", "Vue error event must not repeat without a new failure");
  await page.waitForFunction(
    () => document.querySelector("#hook-status")?.getAttribute("data-status") === "error",
    null,
    { timeout: 10_000 },
  );

  await page.locator("#recover").click();
  await waitForStatus("ready");
  await page.waitForFunction(
    () => document.querySelector("#hook-status")?.getAttribute("data-status") === "ready",
    null,
    { timeout: 15_000 },
  );

  await page.locator("#invalid").click();
  await waitForStatus("error");
  await page.waitForFunction(
    () => document.querySelector("#error-count")?.textContent === "2",
    null,
    { timeout: 10_000 },
  );
  await page.waitForTimeout(100);
  assert.equal(await page.locator("#error-count").textContent(), "2", "a later Vue error should emit once again");

  assert.deepEqual(errors, [], "Vue smoke page raised an uncaught browser error");
  console.log(browserName + " WASM and Vue component/composable smoke passed.");
} finally {
  await browser.close();
}
