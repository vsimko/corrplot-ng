const test = require("node:test");
const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");

const native = require("../../bindings/node");
const wasm = require("../../bindings/wasm/pkg-node/corrplot_ng_wasm.js");

const black = { red: 0, green: 0, blue: 0, alpha: 255 };
const white = { red: 255, green: 255, blue: 255, alpha: 255 };
const scene = {
  width: 80,
  height: 30,
  legend: [
    { kind: "missing", label: "NA", color: { red: 242, green: 242, blue: 242, alpha: 255 } },
    { kind: "stop", label: "Sig", color: { red: 33, green: 102, blue: 172, alpha: 255 } },
  ],
  clusterRegions: [{
    id: "group-a",
    variables: ["alpha"],
    sourceIndices: [0],
    rowStart: 0,
    rowEnd: 0,
    columnStart: 0,
    columnEnd: 0,
    x: 1,
    y: 1,
    width: 10,
    height: 10,
    color: { red: 33, green: 102, blue: 172, alpha: 255 },
    lineWidth: 2,
  }],
  cells: [{
    x: 0,
    y: 0,
    size: 10,
    visible: true,
    fill: white,
    stroke: black,
    glyph: { kind: "circle", radius: 3, color: black },
  }],
  texts: [{
    text: "Typed title",
    semanticRole: "document-title",
    x: 40,
    y: 1,
    fontFamily: "Arial",
    fontSize: 8,
    fontWeight: "bold",
    fontStyle: "normal",
    color: black,
    anchor: "middle",
    rotationDegrees: 0,
  }],
};
const sceneV1 = fs.readFileSync(
  path.join(__dirname, "../../prototypes/shared-svg/scene-v1.example.json"),
  "utf8",
);
const sceneV1Construction = fs.readFileSync(
  path.join(__dirname, "../../prototypes/shared-svg/scene-v1-construction.example.json"),
  "utf8",
);
const sceneV1Statistical = fs.readFileSync(
  path.join(__dirname, "../../prototypes/shared-svg/scene-v1-statistical-overlays.example.json"),
  "utf8",
);
const sceneV1LegendCluster = fs.readFileSync(
  path.join(__dirname, "../../prototypes/shared-svg/scene-v1-legend-cluster.example.json"),
  "utf8",
);
const matrixFixtures = [
  { id: "canonical-missing-values", json: sceneV1 },
  { id: "construction-overrides-and-figure-text", json: sceneV1Construction },
  { id: "statistical-overlays", json: sceneV1Statistical },
  { id: "legend-and-cluster-regions", json: sceneV1LegendCluster },
];
const rgbaOptions = {
  widthPx: 380,
  heightPx: 392,
  dpi: 96,
  background: white,
};
const matrix = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, 0.5, 0.5, 1],
  kind: "correlation",
};
const cellMethods = ["circle", "square", "ellipse", "number", "shade", "color", "pie"];

const correlationCorpus = JSON.parse(fs.readFileSync(
  path.join(__dirname, "../../tests/fixtures/correlation-cases.json"),
  "utf8",
));
const generalMatrixCorpus = JSON.parse(fs.readFileSync(
  path.join(__dirname, "../../tests/fixtures/general-matrix-cases.json"),
  "utf8",
));
const sharedMatrixFixtures = [
  ...correlationCorpus.cases.map(fixture => ({ ...fixture, kind: "correlation" })),
  ...generalMatrixCorpus.cases.map(fixture => ({ ...fixture, kind: "general_numeric" })),
];

function matrixInputFromFixture(fixture) {
  const rows = fixture.matrix.length;
  const columns = fixture.matrix[0]?.length ?? 0;
  assert.ok(rows > 0 && columns > 0, `${fixture.id}: fixture matrix must not be empty`);
  assert.ok(fixture.matrix.every(row => row.length === columns), `${fixture.id}: fixture matrix must be rectangular`);
  return {
    labels: fixture.labels ?? Array.from({ length: rows }, (_, index) => `v${index + 1}`),
    rows,
    columns,
    values: fixture.matrix.flat(),
    kind: fixture.kind,
  };
}

function assertRgbaParity(nativeOutput, wasmOutput, context) {
  assert.equal(wasmOutput.diagnostic ?? null, null, `${context}: WASM RGBA diagnostic`);
  for (const key of ["widthPx", "heightPx", "strideBytes", "pixelFormat", "alphaMode", "colorSpace", "dpi"]) {
    assert.equal(wasmOutput[key], nativeOutput[key], `${context}: ${key}`);
  }
  assert.deepEqual(Buffer.from(wasmOutput.pixels), nativeOutput.pixels, `${context}: RGBA bytes differ`);
}

function unwrapSvg(result) {
  assert.equal(result.diagnostic ?? null, null);
  assert.equal(typeof result.svg, "string");
  return result.svg;
}

function diagnosticFrom(operation) {
  try {
    operation();
  } catch (error) {
    assert.ok(error.diagnostic, "native backend error must carry diagnostic-v1");
    return error.diagnostic;
  }
  assert.fail("native backend unexpectedly accepted invalid input");
}

test("Node and WASM validate the same normalized matrix", () => {
  assert.equal(native.validateMatrix(matrix), true);
  const result = wasm.validateMatrix(matrix);
  assert.equal(result.valid, true);
  assert.equal(result.diagnostic ?? null, null);
});

test("Node and WASM return the same diagnostic for invalid matrices", () => {
  const invalid = { ...matrix, values: [1, 0.5, 0.2, 1] };
  const nativeDiagnostic = diagnosticFrom(() => native.validateMatrix(invalid));
  const wasmDiagnostic = wasm.validateMatrix(invalid).diagnostic;
  assert.deepEqual(wasmDiagnostic, nativeDiagnostic);
});

test("Node and WASM render identical SVG for typed scenes", () => {
  const options = {
    width: 160,
    height: 160,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  };
  const nativeSvg = native.renderSceneSvg(scene, options);
  const wasmSvg = unwrapSvg(wasm.renderSceneSvg(scene, options));
  assert.equal(wasmSvg, nativeSvg);
  assert.ok(nativeSvg.includes('class="cluster-region"'));
  assert.ok(nativeSvg.includes('class="legend-entry"'));
  assert.ok(nativeSvg.includes('data-semantic-role="document-title"'));
  assert.ok(nativeSvg.includes('font-family="Arial"'));
  assert.ok(nativeSvg.includes('font-weight="bold"'));
});

test("Node and WASM render identical RGBA bytes for typed scenes", () => {
  const options = { widthPx: 160, heightPx: 160, dpi: 96, background: white };
  const nativeOutput = native.renderSceneRgba(scene, options);
  const wasmOutput = wasm.renderSceneRgba(scene, options);
  assert.equal(wasmOutput.diagnostic ?? null, null);
  assert.equal(wasmOutput.widthPx, nativeOutput.widthPx);
  assert.equal(wasmOutput.heightPx, nativeOutput.heightPx);
  assert.equal(wasmOutput.strideBytes, nativeOutput.strideBytes);
  assert.equal(wasmOutput.pixelFormat, nativeOutput.pixelFormat);
  assert.equal(wasmOutput.alphaMode, nativeOutput.alphaMode);
  assert.equal(wasmOutput.colorSpace, nativeOutput.colorSpace);
  assert.deepEqual(Buffer.from(wasmOutput.pixels), nativeOutput.pixels);
});

test("Node and WASM build the same normalized-matrix scene and render it identically", () => {
  const options = {
    cellMethod: "ellipse",
    title: "Cross-runtime scene",
    description: "Built from the same normalized matrix.",
    displayType: "upper",
    showDiagonal: true,
  };
  const nativeResult = native.createScene(matrix, options);
  const wasmResult = wasm.createScene(matrix, options);
  assert.equal(nativeResult.diagnostic ?? null, null);
  assert.equal(wasmResult.diagnostic ?? null, null);
  assert.deepEqual(JSON.parse(JSON.stringify(wasmResult.scene)), JSON.parse(JSON.stringify(nativeResult.scene)));

  const svgOptions = {
    width: nativeResult.scene.width,
    height: nativeResult.scene.height,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  };
  assert.ok(nativeResult.scene.texts.some(text => text.semanticRole === "document-title"));
  const nativeSvg = native.renderSceneSvg(nativeResult.scene, svgOptions);
  const wasmSvg = unwrapSvg(wasm.renderSceneSvg(wasmResult.scene, svgOptions));
  assert.equal(wasmSvg, nativeSvg);
  assert.ok(nativeSvg.includes("<title id=\"scene-title\">Cross-runtime scene</title>"));

  const rasterOptions = { widthPx: 128, heightPx: 128, dpi: 96, background: white };
  const nativeRgba = native.renderSceneRgba(nativeResult.scene, rasterOptions);
  const wasmRgba = wasm.renderSceneRgba(wasmResult.scene, rasterOptions);
  assert.equal(wasmRgba.diagnostic ?? null, null);
  assert.deepEqual(Buffer.from(wasmRgba.pixels), nativeRgba.pixels);
});

test("Node and WASM return the same diagnostic for invalid scene construction", () => {
  const invalid = { ...matrix, columns: 1, values: [1, 0.5] };
  const nativeResult = native.createScene(invalid);
  const wasmResult = wasm.createScene(invalid, {});
  assert.equal(nativeResult.scene ?? null, null);
  assert.deepEqual(wasmResult.diagnostic, nativeResult.diagnostic);
  assert.equal(nativeResult.diagnostic.code, "input.matrix.invalid_shape");
});

test("Node and WASM construct identical matrix SVG across fixtures and cell methods", () => {
  for (const fixture of matrixFixtures) {
    for (const method of cellMethods) {
      const nativeSvg = native.renderMatrixSvg(fixture.json, method);
      const wasmSvg = unwrapSvg(wasm.renderMatrixSvg(fixture.json, method));
      assert.equal(wasmSvg, nativeSvg, `${fixture.id}: SVG output differs for ${method}`);
      const expectedTitle = JSON.parse(fixture.json).title;
      assert.ok(nativeSvg.includes(`<title id="scene-title">${expectedTitle}</title>`), `${fixture.id}: accessible SVG title missing`);
    }
  }
});

test("Node and WASM construct identical matrix RGBA across fixtures and cell methods", () => {
  for (const fixture of matrixFixtures) {
    for (const method of cellMethods) {
      const nativeOutput = native.renderMatrixRgba(fixture.json, method, rgbaOptions);
      const wasmOutput = wasm.renderMatrixRgba(fixture.json, method, rgbaOptions);
      assert.equal(wasmOutput.diagnostic ?? null, null, `${fixture.id}: WASM RGBA failed for ${method}`);
      for (const key of ["widthPx", "heightPx", "strideBytes", "pixelFormat", "alphaMode", "colorSpace", "dpi"]) {
        assert.equal(wasmOutput[key], nativeOutput[key], `${fixture.id} / ${method}: ${key}`);
      }
      assert.deepEqual(Buffer.from(wasmOutput.pixels), nativeOutput.pixels, `${fixture.id} / ${method}: RGBA bytes differ`);
    }
  }
});

test("Node and WASM render scene-v1 statistical overlays identically", () => {
  const nativeSvg = native.renderMatrixSvg(sceneV1Statistical, "circle");
  const wasmSvg = unwrapSvg(wasm.renderMatrixSvg(sceneV1Statistical, "circle"));
  assert.equal(wasmSvg, nativeSvg);
  assert.ok(nativeSvg.includes('class="confidence-interval"'));
  assert.ok(nativeSvg.includes('class="confidence-interval-cap"'));
  assert.ok(nativeSvg.includes('class="significance-annotation"'));
  assert.ok(nativeSvg.includes(">**</text>"));
  assert.equal((nativeSvg.match(/class="significance-annotation"/g) ?? []).length, 1);
  assert.ok(!nativeSvg.includes("HIDDEN"));
  assert.ok(!nativeSvg.includes('x1="112.00" y1="280.00" x2="180.00" y2="280.00"'));
  assert.ok(!nativeSvg.includes('x1="280.00" y1="280.00" x2="348.00" y2="280.00"'));

  const nativeOutput = native.renderMatrixRgba(sceneV1Statistical, "circle", rgbaOptions);
  const wasmOutput = wasm.renderMatrixRgba(sceneV1Statistical, "circle", rgbaOptions);
  assertRgbaParity(nativeOutput, wasmOutput, "scene-v1 statistical overlays");
  const pixelAt = (output, x, y) => {
    const offset = (y * output.widthPx + x) * 4;
    return Array.from(output.pixels.subarray(offset, offset + 4));
  };
  assert.deepEqual(pixelAt(nativeOutput, 146, 112), [34, 34, 34, 255]);
  assert.deepEqual(pixelAt(nativeOutput, 146, 280), [255, 255, 255, 255]);
  assert.deepEqual(pixelAt(nativeOutput, 320, 280), [255, 255, 255, 255]);
  const annotationInk = [];
  for (let y = 86; y < 102; y++) {
    for (let x = 166; x < 178; x++) {
      if (pixelAt(nativeOutput, x, y).every((channel, index) => channel === [34, 34, 34, 255][index])) {
        annotationInk.push([x, y]);
      }
    }
  }
  assert.ok(annotationInk.length > 0, "RGBA output should contain the supplied significance annotation");
});


test("Node and WASM build and render the shared matrix fixture corpus across every cell method", () => {
  for (const fixture of sharedMatrixFixtures) {
    const input = matrixInputFromFixture(fixture);
    assert.equal(native.validateMatrix(input), true, `${fixture.id}: native validation`);
    const wasmValidation = wasm.validateMatrix(input);
    assert.equal(wasmValidation.valid, true, `${fixture.id}: WASM validation`);
    assert.equal(wasmValidation.diagnostic ?? null, null, `${fixture.id}: WASM validation diagnostic`);

    for (const method of cellMethods) {
      const options = {
        cellMethod: method,
        title: fixture.id,
        description: fixture.description ?? `Shared fixture ${fixture.id}`,
        annotation: fixture.id,
        displayType: fixture.id === "three-variable-reordered"
          ? "lower"
          : fixture.id === "missing-value-pattern"
            ? "upper"
            : "full",
        showDiagonal: fixture.diag ?? true,
      };
      const nativeResult = native.createScene(input, options);
      const wasmResult = wasm.createScene(input, options);
      const context = `${fixture.id} / ${method}`;
      assert.equal(nativeResult.diagnostic ?? null, null, `${context}: native scene diagnostic`);
      assert.equal(wasmResult.diagnostic ?? null, null, `${context}: WASM scene diagnostic`);
      assert.deepEqual(
        JSON.parse(JSON.stringify(wasmResult.scene)),
        JSON.parse(JSON.stringify(nativeResult.scene)),
        `${context}: normalized scene differs`,
      );

      const svgOptions = {
        width: nativeResult.scene.width,
        height: nativeResult.scene.height,
        unit: "px",
        fontFamily: "sans-serif",
        fontSizePt: 10,
        background: white,
      };
      const nativeSvg = native.renderSceneSvg(nativeResult.scene, svgOptions);
      const wasmSvg = unwrapSvg(wasm.renderSceneSvg(wasmResult.scene, svgOptions));
      assert.equal(wasmSvg, nativeSvg, `${context}: SVG output differs`);

      const rasterOptions = { widthPx: 96, heightPx: 96, dpi: 96, background: white };
      const nativeRgba = native.renderSceneRgba(nativeResult.scene, rasterOptions);
      const wasmRgba = wasm.renderSceneRgba(wasmResult.scene, rasterOptions);
      assertRgbaParity(nativeRgba, wasmRgba, context);
    }
  }
});

test("Node and WASM preserve scene and renderer parity for a deterministic 64x64 correlation matrix", () => {
  const size = 64;
  const labels = Array.from({ length: size }, (_, index) => `variable-${String(index + 1).padStart(2, "0")}`);
  const values = Array.from({ length: size * size }, (_, index) => {
    const row = Math.floor(index / size);
    const column = index % size;
    if (row === column) return 1;
    const first = Math.min(row, column);
    const second = Math.max(row, column);
    return ((first * 31 + second * 17) % 201 - 100) / 200;
  });
  const input = { labels, rows: size, columns: size, values, kind: "correlation" };

  assert.equal(native.validateMatrix(input), true);
  const wasmValidation = wasm.validateMatrix(input);
  assert.equal(wasmValidation.valid, true);
  assert.equal(wasmValidation.diagnostic ?? null, null);

  const options = {
    cellMethod: "ellipse",
    title: "64-variable parity fixture",
    description: "Deterministic large correlation matrix for cross-backend regression coverage.",
    displayType: "upper",
    showDiagonal: false,
  };
  const nativeResult = native.createScene(input, options);
  const wasmResult = wasm.createScene(input, options);
  assert.equal(nativeResult.diagnostic ?? null, null);
  assert.equal(wasmResult.diagnostic ?? null, null);
  assert.equal(nativeResult.scene.cells.length, size * size);
  assert.deepEqual(
    JSON.parse(JSON.stringify(wasmResult.scene)),
    JSON.parse(JSON.stringify(nativeResult.scene)),
  );

  const svgOptions = {
    width: nativeResult.scene.width,
    height: nativeResult.scene.height,
    unit: "px",
    fontFamily: "sans-serif",
    fontSizePt: 10,
    background: white,
  };
  assert.equal(
    unwrapSvg(wasm.renderSceneSvg(wasmResult.scene, svgOptions)),
    native.renderSceneSvg(nativeResult.scene, svgOptions),
  );
  const rasterOptions = { widthPx: 128, heightPx: 128, dpi: 96, background: white };
  assertRgbaParity(
    native.renderSceneRgba(nativeResult.scene, rasterOptions),
    wasm.renderSceneRgba(wasmResult.scene, rasterOptions),
    "64x64 correlation matrix",
  );
});

test("Node and WASM expose the same implemented renderer capabilities", () => {
  const capabilities = JSON.parse(wasm.getRendererCapabilities());
  assert.deepEqual(capabilities, native.getRendererCapabilities());
  for (const feature of [
    "significance",
    "confidence_intervals",
    "legend",
    "figure_text",
    "cluster_regions",
  ]) {
    assert.equal(capabilities.features[feature], true, `${feature} must be declared supported`);
  }
  assert.equal(capabilities.features.plotmath_labels, false);
  assert.equal(capabilities.features.accessible_metadata, false);
});

test("Node and WASM return the same diagnostic for unsupported matrix methods", () => {
  const nativeDiagnostic = diagnosticFrom(() => native.renderMatrixSvg(sceneV1, "unknown"));
  const wasmDiagnostic = wasm.renderMatrixSvg(sceneV1, "unknown").diagnostic;
  assert.deepEqual(wasmDiagnostic, nativeDiagnostic);
});

test("Node and WASM return the same diagnostic for malformed scene-v1 JSON", () => {
  const nativeDiagnostic = diagnosticFrom(() => native.renderMatrixSvg("{", "circle"));
  const wasmDiagnostic = wasm.renderMatrixSvg("{", "circle").diagnostic;
  assert.deepEqual(wasmDiagnostic, nativeDiagnostic);
  assert.equal(wasmDiagnostic.code, "input.scene.invalid_json");
  assert.equal(wasmDiagnostic.path, "$");
});

test("Node and WASM return the same diagnostic for invalid RGBA options", () => {
  const invalidOptions = { ...rgbaOptions, widthPx: 0 };
  const nativeDiagnostic = diagnosticFrom(() => native.renderMatrixRgba(sceneV1, "circle", invalidOptions));
  const wasmDiagnostic = wasm.renderMatrixRgba(sceneV1, "circle", invalidOptions).diagnostic;
  assert.deepEqual(wasmDiagnostic, nativeDiagnostic);
});
