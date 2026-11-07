import {
  createCorrplotEngine,
  type MatrixInput,
  type SceneBuildOptions,
  type SvgRenderOptions,
  type RgbaRenderOptions,
  type CanvasRenderOptions,
} from "@corrplot-ng/typescript";

const matrix: MatrixInput = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: new Float64Array([1, 0, 0, 1]),
  missingMask: [false, true, true, false],
  kind: "correlation",
};
const sceneOptions: SceneBuildOptions = {
  cellMethod: "circle",
  title: "Browser usage",
};
const svgOptions: SvgRenderOptions = { format: "svg", background: { red: 255, green: 255, blue: 255, alpha: 255 } };
const rgbaOptions: RgbaRenderOptions = {
  format: "rgba",
  widthPx: 256,
  heightPx: 256,
  background: { red: 255, green: 255, blue: 255, alpha: 255 },
};
const canvasOptions: CanvasRenderOptions = {
  width: 256,
  height: 256,
  devicePixelRatio: 2,
};

export async function browserUsageExample() {
  const engine = await createCorrplotEngine({ backend: "wasm" });
  const scene = engine.createScene(matrix, sceneOptions);
  const svg: string = engine.render(scene, svgOptions);
  const rgba = engine.render(scene, rgbaOptions);
  const canvas = document.createElement("canvas");
  const canvasResult = engine.renderToCanvas(scene, canvas, canvasOptions);
  const canvasCapabilities = engine.getCanvasRendererCapabilities();
  engine.dispose();
  return { svg, rgba, canvasResult, canvasCapabilities };
}
