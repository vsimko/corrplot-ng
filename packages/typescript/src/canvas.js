import { CorrplotError } from "./factory.js";

const DEFAULT_BACKGROUND = Object.freeze({ red: 255, green: 255, blue: 255, alpha: 255 });
const FULL_CIRCLE = Math.PI * 2;
const SUPPORTED_GLYPHS = new Set(["circle", "square", "ellipse", "pie", "number", "shade", "color"]);

function fail(code, message, path) {
  throw new CorrplotError({
    schema_version: 1,
    code,
    severity: "error",
    stage: "renderer",
    message,
    path,
  });
}

function finite(value) {
  return typeof value === "number" && Number.isFinite(value);
}

function positive(value) {
  return finite(value) && value > 0;
}

function colorCss(color, path) {
  if (!color || !["red", "green", "blue", "alpha"].every(key =>
    Number.isInteger(color[key]) && color[key] >= 0 && color[key] <= 255
  )) {
    fail("renderer.output.invalid_scene", "Canvas scene contains an invalid RGBA color", path);
  }
  return `rgba(${color.red}, ${color.green}, ${color.blue}, ${color.alpha / 255})`;
}

function validateScene(scene) {
  if (!scene || typeof scene !== "object" ||
      !positive(scene.width) || !positive(scene.height) ||
      !Array.isArray(scene.cells) || !Array.isArray(scene.texts)) {
    fail("renderer.output.invalid_scene", "Canvas renderer received an invalid renderer-neutral scene", "scene");
  }
  if (scene.description !== undefined && typeof scene.description !== "string") {
    fail("renderer.output.invalid_scene", "scene description must be a string", "scene.description");
  }

  if (scene.legend !== undefined && scene.legend !== null && !Array.isArray(scene.legend)) {
    fail("renderer.output.invalid_scene", "legend must be an array", "scene.legend");
  }
  for (const [index, entry] of (scene.legend ?? []).entries()) {
    const path = `scene.legend[${index}]`;
    if (!entry || !["stop", "interval", "missing"].includes(entry.kind) || typeof entry.label !== "string") {
      fail("renderer.output.invalid_scene", "legend entry has invalid kind or label", path);
    }
    colorCss(entry.color, `${path}.color`);
  }

  if (scene.clusterRegions !== undefined && scene.clusterRegions !== null && !Array.isArray(scene.clusterRegions)) {
    fail("renderer.output.invalid_scene", "clusterRegions must be an array", "scene.clusterRegions");
  }
  for (const [index, region] of (scene.clusterRegions ?? []).entries()) {
    const path = `scene.clusterRegions[${index}]`;
    const validIndices = values => Array.isArray(values) && values.length > 0 &&
      values.every(value => Number.isSafeInteger(value) && value >= 0 && value <= 0xffff_ffff) &&
      new Set(values).size === values.length;
    const validNames = values => Array.isArray(values) && values.length > 0 &&
      values.every(value => typeof value === "string" && value.length > 0) &&
      new Set(values).size === values.length;
    if (!region || typeof region.id !== "string" || !region.id.length ||
        !validNames(region.variables) || !validIndices(region.sourceIndices) ||
        region.variables.length !== region.sourceIndices.length ||
        !Number.isSafeInteger(region.rowStart) || region.rowStart < 0 || region.rowStart > 0xffff_ffff ||
        !Number.isSafeInteger(region.rowEnd) || region.rowEnd < region.rowStart || region.rowEnd > 0xffff_ffff ||
        !Number.isSafeInteger(region.columnStart) || region.columnStart < 0 || region.columnStart > 0xffff_ffff ||
        !Number.isSafeInteger(region.columnEnd) || region.columnEnd < region.columnStart || region.columnEnd > 0xffff_ffff ||
        !finite(region.x) || !finite(region.y) || !positive(region.width) ||
        !positive(region.height) || !positive(region.lineWidth)) {
      fail("renderer.output.invalid_scene", "cluster region has invalid identity, indices, or geometry", path);
    }
    colorCss(region.color, `${path}.color`);
  }
  if (scene.significance != null) {
    const policy = scene.significance;
    if (!policy || !["pch", "p-value", "blank", "stars"].includes(policy.mode) ||
        !Array.isArray(policy.thresholds) || policy.thresholds.length === 0 ||
        policy.thresholds.some(value => !finite(value) || value < 0 || value > 1) ||
        typeof policy.symbol !== "string" || !policy.symbol.length ||
        [...policy.symbol].length > 8) {
      fail("renderer.output.invalid_scene", "scene has invalid plot-wide significance policy", "scene.significance");
    }
  }

  for (const [index, cell] of scene.cells.entries()) {
    const path = `scene.cells[${index}]`;
    if (!cell || !finite(cell.x) || !finite(cell.y) || !positive(cell.size) || typeof cell.visible !== "boolean") {
      fail("renderer.output.invalid_scene", "scene cell has invalid geometry or visibility", path);
    }
    colorCss(cell.fill, `${path}.fill`);
    colorCss(cell.stroke, `${path}.stroke`);
    if (cell.pValue != null && (!finite(cell.pValue) || cell.pValue < 0 || cell.pValue > 1)) {
      fail("renderer.output.invalid_scene", "cell pValue must be between 0 and 1", `${path}.pValue`);
    }
    if (cell.normalizedValue != null && !finite(cell.normalizedValue)) {
      fail("renderer.output.invalid_scene", "cell normalizedValue must be finite", `${path}.normalizedValue`);
    }
    if (cell.significance != null) {
      const value = cell.significance;
      if (!value || (value.pValue != null && (!finite(value.pValue) || value.pValue < 0 || value.pValue > 1)) ||
          !["unknown", "significant", "not_significant"].includes(value.status) ||
          (value.annotation != null && typeof value.annotation !== "string") ||
          typeof value.blank !== "boolean") {
        fail("renderer.output.invalid_scene", "cell has invalid significance metadata", `${path}.significance`);
      }
    }
    if (cell.confidenceInterval != null) {
      const value = cell.confidenceInterval;
      if (!value || !finite(value.lower) || !finite(value.upper) ||
          value.lower < -1 || value.upper > 1 || value.lower > value.upper ||
          !finite(value.x1) || !finite(value.x2) || !finite(value.y) ||
          !finite(value.capHeight) || value.capHeight < 0 ||
          !finite(value.strokeWidth) || value.strokeWidth < 0) {
        fail("renderer.output.invalid_scene", "cell has invalid confidence interval metadata", `${path}.confidenceInterval`);
      }
      colorCss(value.color, `${path}.confidenceInterval.color`);
    }
    const glyph = cell.glyph;
    if (glyph === undefined || glyph === null) continue;
    if (!glyph || !SUPPORTED_GLYPHS.has(glyph.kind)) {
      fail("renderer.unsupported_feature", `Canvas 2D does not support glyph '${glyph?.kind ?? "unknown"}'`, `${path}.glyph.kind`);
    }
    if (glyph.kind !== "color") colorCss(glyph.color, `${path}.glyph.color`);
    switch (glyph.kind) {
      case "circle":
      case "pie":
        if (!finite(glyph.radius) || glyph.radius < 0) {
          fail("renderer.output.invalid_scene", "glyph radius must be finite and non-negative", `${path}.glyph.radius`);
        }
        if (glyph.kind === "pie" && (!finite(glyph.startRadians) || !finite(glyph.endRadians) || !finite(glyph.endRadians - glyph.startRadians))) {
          fail("renderer.output.invalid_scene", "pie angles and their difference must be finite", `${path}.glyph`);
        }
        break;
      case "square":
        if (!finite(glyph.side) || glyph.side < 0) {
          fail("renderer.output.invalid_scene", "square side must be finite and non-negative", `${path}.glyph.side`);
        }
        break;
      case "ellipse":
        if (!finite(glyph.rx) || glyph.rx < 0 || !finite(glyph.ry) || glyph.ry < 0 ||
            (glyph.rotationDegrees !== undefined && !finite(glyph.rotationDegrees))) {
          fail("renderer.output.invalid_scene", "ellipse geometry must be finite and non-negative", `${path}.glyph`);
        }
        break;
      case "number":
        if (typeof glyph.text !== "string" || !positive(glyph.fontSize)) {
          fail("renderer.output.invalid_scene", "number glyph requires text and a positive font size", `${path}.glyph`);
        }
        break;
      case "shade":
      case "color":
        break;
    }
  }

  for (const [index, text] of scene.texts.entries()) {
    const path = `scene.texts[${index}]`;
    if (!text || typeof text.text !== "string" || !finite(text.x) || !finite(text.y) ||
        !positive(text.fontSize) || !["start", "middle", "end"].includes(text.anchor) ||
        (text.rotationDegrees !== undefined && !finite(text.rotationDegrees))) {
      fail("renderer.output.invalid_scene", "scene text has invalid content, position, anchor, or font size", path);
    }
    colorCss(text.color, `${path}.color`);
    if (text.semanticRole === "plotmath") {
      fail("renderer.unsupported_feature", "Canvas 2D does not yet support plotmath labels", `${path}.semanticRole`);
    }
    for (const field of ["fontFamily", "fontWeight", "fontStyle"]) {
      if (text[field] !== undefined && typeof text[field] !== "string") {
        fail("renderer.output.invalid_scene", `${field} must be a string`, `${path}.${field}`);
      }
    }
  }
}

function resolveTarget(target) {
  const isContext = Boolean(target && target.canvas && typeof target.save === "function");
  const canvas = isContext ? target.canvas : target;
  if (!canvas || typeof canvas !== "object") {
    fail("configuration.invalid_field", "provide an HTMLCanvasElement, OffscreenCanvas, or Canvas 2D context", "canvas");
  }

  let context;
  try {
    context = isContext ? target : canvas.getContext?.("2d");
  } catch (error) {
    fail("renderer.unsupported_feature", `could not acquire a Canvas 2D context: ${error instanceof Error ? error.message : String(error)}`, "canvas");
  }
  if (!context) {
    fail("renderer.unsupported_feature", "the supplied canvas does not provide a Canvas 2D context", "canvas");
  }
  if (context.canvas && context.canvas !== canvas) {
    fail("configuration.invalid_field", "Canvas context does not belong to the supplied canvas", "context");
  }

  const requiredMethods = [
    "save", "restore", "setTransform", "clearRect", "fillRect", "strokeRect",
    "beginPath", "rect", "clip", "arc", "ellipse", "moveTo", "lineTo",
    "closePath", "fill", "stroke", "translate", "rotate", "fillText",
  ];
  for (const method of requiredMethods) {
    if (typeof context[method] !== "function") {
      fail("renderer.unsupported_feature", `Canvas 2D context is missing '${method}'`, `context.${method}`);
    }
  }
  return { canvas, context };
}

function drawGlyph(context, cell, glyph, fontFamily, path) {
  const cx = cell.x + cell.size / 2;
  const cy = cell.y + cell.size / 2;
  if (glyph.kind === "color") return;
  const fill = colorCss(glyph.color, `${path}.color`);

  switch (glyph.kind) {
    case "circle":
      context.beginPath();
      context.arc(cx, cy, glyph.radius, 0, FULL_CIRCLE);
      context.fillStyle = fill;
      context.fill();
      break;
    case "square": {
      const half = glyph.side / 2;
      context.fillStyle = fill;
      context.fillRect(cx - half, cy - half, glyph.side, glyph.side);
      break;
    }
    case "ellipse":
      context.beginPath();
      context.ellipse(
        cx,
        cy,
        glyph.rx,
        glyph.ry,
        (glyph.rotationDegrees ?? 0) * Math.PI / 180,
        0,
        FULL_CIRCLE,
      );
      context.fillStyle = fill;
      context.fill();
      break;
    case "pie":
      context.beginPath();
      context.moveTo(cx, cy);
      if (Math.abs(glyph.endRadians - glyph.startRadians) >= FULL_CIRCLE) {
        context.arc(cx, cy, glyph.radius, 0, FULL_CIRCLE);
      } else {
        context.arc(cx, cy, glyph.radius, glyph.startRadians, glyph.endRadians, glyph.endRadians < glyph.startRadians);
      }
      context.closePath();
      context.fillStyle = fill;
      context.fill();
      break;
    case "number":
      context.save();
      try {
        context.font = `normal normal ${glyph.fontSize}px ${fontFamily}`;
        context.textAlign = "center";
        context.textBaseline = "middle";
        context.fillStyle = fill;
        context.fillText(glyph.text, cx, cy);
      } finally {
        context.restore();
      }
      break;
    case "shade":
      context.beginPath();
      context.moveTo(cell.x, cell.y + cell.size);
      context.lineTo(cell.x + cell.size, cell.y);
      context.strokeStyle = fill;
      context.lineWidth = 1;
      context.stroke();
      break;
  }
}

function drawClusterRegions(context, regions) {
  for (const [index, region] of regions.entries()) {
    context.save();
    try {
      context.strokeStyle = colorCss(region.color, `scene.clusterRegions[${index}].color`);
      context.lineWidth = region.lineWidth;
      context.strokeRect(region.x, region.y, region.width, region.height);
    } finally {
      context.restore();
    }
  }
}

function drawLegend(context, entries, sceneHeight, fontFamily) {
  if (!entries.length) return;
  context.save();
  try {
    context.font = `normal normal 8px ${fontFamily}`;
    context.textAlign = "left";
    context.textBaseline = "hanging";
    let x = 8;
    const y = sceneHeight - 13;
    for (const [index, entry] of entries.entries()) {
      context.fillStyle = colorCss(entry.color, `scene.legend[${index}].color`);
      context.fillRect(x, y + 1, 7, 7);
      x += 11;
      context.fillStyle = "rgba(0, 0, 0, 1)";
      context.fillText(entry.label, x, y);
      x += [...entry.label].length * 8 * 6 / 7 + 10;
    }
  } finally {
    context.restore();
  }
}

function drawText(context, text, defaultFontFamily, path) {
  const fontFamily = text.fontFamily || defaultFontFamily;
  const fontWeight = text.fontWeight || "normal";
  const fontStyle = text.fontStyle || "normal";
  context.save();
  try {
    context.translate(text.x, text.y);
    // Canvas and SVG both use positive angles for clockwise rotation in screen coordinates.
    context.rotate((text.rotationDegrees ?? 0) * Math.PI / 180);
    context.font = `${fontStyle} ${fontWeight} ${text.fontSize}px ${fontFamily}`;
    context.textAlign = text.anchor === "middle" ? "center" : text.anchor;
    context.textBaseline = "hanging";
    context.fillStyle = colorCss(text.color, `${path}.color`);
    text.text.split("\n").forEach((line, index) => {
      context.fillText(line, 0, index * text.fontSize * 1.2);
    });
  } finally {
    context.restore();
  }
}

/**
 * Draw a renderer-neutral Rust scene directly into a caller-owned Canvas 2D
 * target. Width and height are logical CSS pixels; devicePixelRatio controls
 * the canvas backing-store resolution.
 */
export function renderSceneToCanvas(scene, target, options = {}) {
  validateScene(scene);
  options ??= {};
  const width = options.width ?? scene.width;
  const height = options.height ?? scene.height;
  const detectedDpr = globalThis.devicePixelRatio;
  const devicePixelRatio = options.devicePixelRatio ??
    (positive(detectedDpr) ? detectedDpr : 1);
  const clear = options.clear ?? true;
  const setCssSize = options.setCssSize ?? true;
  const fontFamily = options.fontFamily ?? "sans-serif";

  if (!positive(width) || !positive(height)) {
    fail("renderer.output.invalid_options", "Canvas width and height must be positive finite numbers", "output");
  }
  if (!positive(devicePixelRatio)) {
    fail("renderer.output.invalid_options", "devicePixelRatio must be positive and finite", "devicePixelRatio");
  }
  if (typeof clear !== "boolean" || typeof setCssSize !== "boolean") {
    fail("renderer.output.invalid_options", "clear and setCssSize must be booleans", "options");
  }
  if (typeof fontFamily !== "string" || !fontFamily.trim()) {
    fail("renderer.output.invalid_options", "fontFamily must be a non-empty string", "fontFamily");
  }

  const background = options.background === undefined
    ? (clear ? DEFAULT_BACKGROUND : null)
    : options.background;
  const backgroundCss = background === null ? null : colorCss(background, "output.background");
  const widthPx = Math.max(1, Math.round(width * devicePixelRatio));
  const heightPx = Math.max(1, Math.round(height * devicePixelRatio));
  if (!Number.isSafeInteger(widthPx) || !Number.isSafeInteger(heightPx)) {
    fail("renderer.output.invalid_options", "Canvas backing dimensions exceed the safe integer range", "output");
  }

  const { canvas, context } = resolveTarget(target);
  try {
    if (canvas.width !== widthPx) canvas.width = widthPx;
    if (canvas.height !== heightPx) canvas.height = heightPx;
    if (canvas.width !== widthPx || canvas.height !== heightPx) {
      fail("renderer.output.invalid_buffer", "Canvas rejected the requested backing-store dimensions", "canvas");
    }
    if (setCssSize && canvas.style) {
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
    }
    if (typeof canvas.setAttribute === "function") {
      if (typeof canvas.getAttribute !== "function" || !canvas.getAttribute("role")) {
        canvas.setAttribute("role", "img");
      }
      if (scene.description && (typeof canvas.getAttribute !== "function" || !canvas.getAttribute("aria-label"))) {
        canvas.setAttribute("aria-label", scene.description);
      }
    }
  } catch (error) {
    if (error instanceof CorrplotError) throw error;
    fail("renderer.output.invalid_buffer", `could not size or annotate the canvas: ${error instanceof Error ? error.message : String(error)}`, "canvas");
  }

  let saved = false;
  try {
    context.save();
    saved = true;
    context.setTransform(1, 0, 0, 1, 0, 0);
    context.globalAlpha = 1;
    context.globalCompositeOperation = "source-over";
    if (clear) context.clearRect(0, 0, widthPx, heightPx);
    if (backgroundCss !== null) {
      context.fillStyle = backgroundCss;
      context.fillRect(0, 0, widthPx, heightPx);
    }

    const scale = Math.min(widthPx / scene.width, heightPx / scene.height);
    const offsetX = (widthPx - scene.width * scale) / 2;
    const offsetY = (heightPx - scene.height * scale) / 2;
    context.setTransform(scale, 0, 0, scale, offsetX, offsetY);
    context.beginPath();
    context.rect(0, 0, scene.width, scene.height);
    context.clip();

    for (const [index, cell] of scene.cells.entries()) {
      if (!cell.visible) continue;
      const path = `scene.cells[${index}]`;
      context.fillStyle = colorCss(cell.fill, `${path}.fill`);
      context.fillRect(cell.x, cell.y, cell.size, cell.size);
      context.strokeStyle = colorCss(cell.stroke, `${path}.stroke`);
      context.lineWidth = 1;
      context.strokeRect(cell.x, cell.y, cell.size, cell.size);
      const blank = cell.significance?.blank === true;
      if (!blank && cell.glyph) drawGlyph(context, cell, cell.glyph, fontFamily, `${path}.glyph`);
      if (!blank && cell.confidenceInterval) {
        const interval = cell.confidenceInterval;
        context.save();
        try {
          context.strokeStyle = colorCss(interval.color, `${path}.confidenceInterval.color`);
          context.lineWidth = interval.strokeWidth;
          context.beginPath();
          context.moveTo(interval.x1, interval.y);
          context.lineTo(interval.x2, interval.y);
          context.stroke();
          for (const endpoint of [interval.x1, interval.x2]) {
            context.beginPath();
            context.moveTo(endpoint, interval.y - interval.capHeight / 2);
            context.lineTo(endpoint, interval.y + interval.capHeight / 2);
            context.stroke();
          }
        } finally {
          context.restore();
        }
      }
    }

    for (const [index, cell] of scene.cells.entries()) {
      if (!cell.visible) continue;
      const annotation = cell.significance?.annotation;
      if (typeof annotation !== "string" || annotation.length === 0) continue;
      context.save();
      try {
        context.font = `normal bold 10px ${fontFamily}`;
        context.textAlign = "center";
        context.textBaseline = "middle";
        context.fillStyle = "rgba(34, 34, 34, 1)";
        context.fillText(annotation, cell.x + cell.size * 0.8, cell.y + cell.size * 0.2);
      } finally {
        context.restore();
      }
    }

    drawClusterRegions(context, scene.clusterRegions ?? []);
    drawLegend(context, scene.legend ?? [], scene.height, fontFamily);

    for (const [index, text] of scene.texts.entries()) {
      drawText(context, text, fontFamily, `scene.texts[${index}]`);
    }
  } catch (error) {
    if (error instanceof CorrplotError) throw error;
    fail("renderer.output.invalid_buffer", `Canvas 2D rendering failed: ${error instanceof Error ? error.message : String(error)}`, "canvas");
  } finally {
    if (saved) context.restore();
  }

  return { canvas, context, widthPx, heightPx, devicePixelRatio };
}

function supportsOffscreenCanvas2d() {
  if (typeof globalThis.OffscreenCanvas !== "function") return false;
  try {
    return Boolean(new globalThis.OffscreenCanvas(1, 1).getContext("2d"));
  } catch {
    return false;
  }
}

export function getCanvasRendererCapabilities() {
  const hostTargets = ["browser-canvas-2d"];
  if (supportsOffscreenCanvas2d()) hostTargets.push("browser-offscreen-canvas-2d");
  return {
    capability_schema_version: 1,
    renderer: {
      id: "typescript-canvas-2d",
      name: "Corrplot-ng browser Canvas 2D renderer",
      version: "0.1.0",
    },
    scene_schema_versions: [1],
    output_formats: [],
    host_targets: hostTargets,
    matrix_kinds: ["correlation_plot", "general_matrix_plot"],
    cell_methods: ["circle", "square", "ellipse", "number", "shade", "color", "pie"],
    features: {
      row_labels: true,
      column_labels: true,
      plotmath_labels: false,
      missing_cells: true,
      cell_text_annotations: true,
      significance: true,
      confidence_intervals: true,
      legend: true,
      figure_text: true,
      accessible_metadata: false,
      cluster_regions: true,
    },
    negotiation_modes: ["strict"],
  };
}
