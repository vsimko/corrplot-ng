import test from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import { createCorrplotEngine, CorrplotError } from "../src/node.js";

const background = { red: 255, green: 255, blue: 255, alpha: 255 };
const matrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, -0.5, -0.5, 1],
  kind: "correlation",
};

test("Node facade creates a Rust scene and renders it to SVG and owned RGBA", async () => {
  const engine = await createCorrplotEngine({ backend: "native" });
  try {
    assert.equal(engine.backend, "native");
    assert.equal(engine.validate(matrix), true);
    const scene = engine.createScene(matrix, {
      cellMethod: "ellipse",
      title: "Native facade",
      description: "Built by the shared Rust scene builder.",
      displayType: "upper",
    });
    assert.equal(scene.description, "Built by the shared Rust scene builder.");
    assert.equal(scene.cells.length, 4);
    assert.equal(scene.cells[1].visible, true);
    assert.equal(scene.cells[2].visible, false);

    const svg = engine.render(scene, { format: "svg" });
    assert.match(svg, /^<svg /);
    assert.match(svg, /<ellipse /);
    assert.match(svg, /Built by the shared Rust scene builder/);

    const rgba = engine.render(scene, {
      format: "rgba",
      widthPx: 96,
      heightPx: 96,
      background,
    });
    assert.ok(rgba.pixels instanceof Uint8Array);
    assert.equal(rgba.widthPx, 96);
    assert.equal(rgba.heightPx, 96);
    assert.equal(rgba.pixels.byteLength, rgba.strideBytes * rgba.heightPx);
  } finally {
    engine.dispose();
  }
});

test("Node facade preserves the matrix-bearing scene-v1 compatibility path", async () => {
  const engine = await createCorrplotEngine({ backend: "native" });
  try {
    const sceneV1Json = await readFile(
      new URL("../../../prototypes/shared-svg/scene-v1.example.json", import.meta.url),
      "utf8",
    );
    const svg = engine.render(sceneV1Json, { format: "svg", method: "circle" });
    assert.match(svg, /^<svg /);
  } finally {
    engine.dispose();
  }
});

test("Node facade maps normalized-matrix construction failures to CorrplotError", async () => {
  const engine = await createCorrplotEngine({ backend: "native" });
  try {
    assert.throws(
      () => engine.createScene({
        labels: ["alpha", "beta"],
        rows: 2,
        columns: 1,
        values: [1, 0.5],
        kind: "correlation",
      }),
      error => error instanceof CorrplotError &&
        error.diagnostic.code === "input.matrix.invalid_shape" &&
        error.diagnostic.stage === "input",
    );
  } finally {
    engine.dispose();
  }
});
