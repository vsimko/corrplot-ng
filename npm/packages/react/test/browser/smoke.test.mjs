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
  await page.goto("http://127.0.0.1:4174/", { waitUntil: "domcontentloaded" });

  const plot = page.locator('[data-testid="corrplot"]');
  const waitForStatus = status => page.waitForFunction(
    expected => document.querySelector('[data-testid="corrplot"]')?.getAttribute("data-status") === expected,
    status,
    { timeout: 30_000 },
  );
  const waitForSvgDescription = description => page.waitForFunction(
    expected => document.querySelector('[data-testid="corrplot"] svg')?.textContent.includes(expected),
    description,
    { timeout: 30_000 },
  );

  await waitForStatus("ready");
  let svg = await plot.locator("svg").evaluate(element => element.outerHTML);
  assert.ok(svg.includes("React integration"), "React component did not render its figure title");
  assert.ok(svg.includes("React smoke revision 0"), "React component did not render the initial figure");
  assert.equal(await page.locator("#revision").textContent(), "0");
  assert.equal(await page.locator("#error-count").textContent(), "0");

  await page.locator("#update").click();
  await waitForSvgDescription("React smoke revision 1");
  svg = await plot.locator("svg").evaluate(element => element.outerHTML);
  assert.ok(svg.includes("gamma"), "React matrix update did not reach the rendered SVG");
  assert.equal(await page.locator("#revision").textContent(), "1");

  await page.locator("#toggle").click();
  await page.waitForFunction(
    () => document.querySelector('[data-testid="corrplot"]') === null,
    null,
    { timeout: 10_000 },
  );
  await page.locator("#toggle").click();
  await waitForStatus("ready");
  await waitForSvgDescription("React smoke revision 1");

  await page.locator("#invalid").click();
  await waitForStatus("error");
  assert.equal(await page.locator('[data-testid="corrplot"][role="alert"]').count(), 1);
  await page.waitForFunction(
    () => document.querySelector("#error-count")?.textContent === "1" &&
      document.querySelector("#error-message")?.textContent.trim().length > 0,
    null,
    { timeout: 10_000 },
  );
  await page.waitForTimeout(100);
  assert.equal(
    await page.locator("#error-count").textContent(),
    "1",
    "React onError must not repeat for the same failed render under Strict Mode",
  );

  await page.locator("#recover").click();
  await waitForStatus("ready");
  await waitForSvgDescription("React smoke revision 1");
  assert.equal(await page.locator("#error-count").textContent(), "1");

  assert.deepEqual(errors, [], "React smoke page raised an uncaught browser error");
  console.log(browserName + " React Strict Mode component smoke passed.");
} finally {
  await browser.close();
}
