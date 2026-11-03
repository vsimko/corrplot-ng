import { CorrplotError } from "./factory.js";

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

function colorCss(color) {
  if (!color || !["red", "green", "blue", "alpha"].every(key => Number.isInteger(color[key]) && color[key] >= 0 && color[key] <= 255)) {
    fail("scene.invalid_content", "Canvas command contains an invalid RGBA color", "commands.color");
  }
  return `rgba(${color.red}, ${color.green}, ${color.blue}, ${color.alpha / 255})`;
}

function drawCommand(context, command) {
  switch (command.op) {
    case "fillRect":
      context.fillStyle = colorCss(command.color);
      context.fillRect(command.x, command.y, command.width, command.height);
      break;
    case "strokeRect":
      context.strokeStyle = colorCss(command.color);
      context.lineWidth = command.lineWidth;
      context.strokeRect(command.x, command.y, command.width, command.height);
      break;
    case "circle":
      context.beginPath();
      context.arc(command.cx, command.cy, command.radius, 0, Math.PI * 2);
      context.fillStyle = colorCss(command.color);
      context.fill();
      break;
    case "square":
      context.fillStyle = colorCss(command.color);
      context.fillRect(command.cx - command.side / 2, command.cy - command.side / 2, command.side, command.side);
      break;
    case "ellipse":
      context.beginPath();
      context.ellipse(command.cx, command.cy, command.rx, command.ry, -command.rotationDegrees * Math.PI / 180, 0, Math.PI * 2);
      context.fillStyle = colorCss(command.color);
      context.fill();
      break;
    case "pie":
      context.beginPath();
      context.moveTo(command.cx, command.cy);
      context.arc(command.cx, command.cy, command.radius, command.startRadians, command.endRadians);
      context.closePath();
      context.fillStyle = colorCss(command.color);
      context.fill();
      break;
    case "line":
      context.beginPath();
      context.moveTo(command.x1, command.y1);
      context.lineTo(command.x2, command.y2);
      context.strokeStyle = colorCss(command.color);
      context.lineWidth = command.lineWidth;
      context.stroke();
      break;
    case "text": {
      context.save();
      context.translate(command.x, command.y);
      context.rotate(-command.rotationDegrees * Math.PI / 180);
      context.font = `${command.fontStyle} ${command.fontWeight} ${command.fontSize}px ${command.fontFamily || "sans-serif"}`;
      context.textAlign = command.anchor === "middle" ? "center" : command.anchor === "end" ? "end" : "start";
      context.textBaseline = "top";
      context.fillStyle = colorCss(command.color);
      context.fillText(command.text, 0, 0);
      context.restore();
      break;
    }
    default:
      fail("renderer.unsupported_feature", `unsupported Canvas command '${command.op}'`, "commands.op");
  }
}

/**
 * Draw a shared-scene command stream into a caller-owned Canvas 2D context.
 * width/height are logical CSS pixels; devicePixelRatio controls backing-store resolution.
 */
export function drawCanvasCommands(scene, options = {}) {
  const {
    canvas: suppliedCanvas,
    context: suppliedContext,
    width,
    height,
    devicePixelRatio = 1,
    background = null,
    clear = true,
  } = options;

  if (!scene || !Number.isFinite(scene.width) || scene.width <= 0 ||
      !Number.isFinite(scene.height) || scene.height <= 0 || !Array.isArray(scene.commands)) {
    fail("scene.invalid_content", "Canvas renderer received an invalid shared scene command stream", "scene");
  }
  if (!Number.isFinite(width) || width <= 0 || !Number.isFinite(height) || height <= 0) {
    fail("configuration.invalid_field", "Canvas output requires positive logical width and height", "output");
  }
  if (!Number.isFinite(devicePixelRatio) || devicePixelRatio <= 0 || devicePixelRatio > 8) {
    fail("configuration.invalid_field", "devicePixelRatio must be greater than zero and no greater than eight", "devicePixelRatio");
  }

  const canvas = suppliedCanvas ?? suppliedContext?.canvas;
  if (!canvas) fail("configuration.invalid_field", "provide a canvas or a Canvas 2D context", "canvas");
  if (suppliedCanvas && suppliedContext && suppliedContext.canvas !== suppliedCanvas) {
    fail("configuration.invalid_field", "the supplied context does not belong to the supplied canvas", "context");
  }
  const context = suppliedContext ?? canvas.getContext?.("2d");
  if (!context) fail("renderer.unsupported_feature", "the supplied canvas does not provide a 2D context", "canvas");

  const widthPx = Math.max(1, Math.round(width * devicePixelRatio));
  const heightPx = Math.max(1, Math.round(height * devicePixelRatio));
  if (canvas.width !== widthPx) canvas.width = widthPx;
  if (canvas.height !== heightPx) canvas.height = heightPx;

  if (typeof canvas.setAttribute === "function") {
    canvas.setAttribute("role", "img");
    if (scene.description) canvas.setAttribute("aria-label", scene.description);
  }

  context.save();
  context.setTransform(1, 0, 0, 1, 0, 0);
  context.globalAlpha = 1;
  context.globalCompositeOperation = "source-over";
  if (clear) context.clearRect(0, 0, widthPx, heightPx);
  if (background !== null) {
    context.fillStyle = colorCss(background);
    context.fillRect(0, 0, widthPx, heightPx);
  }

  const scale = Math.min(widthPx / scene.width, heightPx / scene.height);
  const offsetX = (widthPx - scene.width * scale) / 2;
  const offsetY = (heightPx - scene.height * scale) / 2;
  context.setTransform(scale, 0, 0, scale, offsetX, offsetY);
  context.beginPath();
  context.rect(0, 0, scene.width, scene.height);
  context.clip();
  for (const command of scene.commands) drawCommand(context, command);
  context.restore();

  return { canvas, context, widthPx, heightPx, devicePixelRatio };
}
