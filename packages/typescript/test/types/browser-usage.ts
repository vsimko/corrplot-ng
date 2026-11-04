import {
  createCorrplotEngine,
  type MatrixInput,
  type SceneInput,
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
const typedScene: SceneInput = {
  width: 80,
  height: 30,
  description: "Typed scene with overlays.",
  cells: [{
    x: 0,
    y: 0,
    size: 10,
    visible: true,
    fill: { red: 255, green: 255, blue: 255, alpha: 255 },
    stroke: { red: 0, green: 0, blue: 0, alpha: 255 },
    glyph: { kind: "circle", radius: 3, color: { red: 0, green: 0, blue: 0, alpha: 255 } },
  }],
  texts: [],
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
  const canvasResult = engine.renderToCanvas(typedScene, canvas, canvasOptions);
  const canvasCapabilities = engine.getCanvasRendererCapabilities();
  engine.dispose();
  return { svg, rgba, canvasResult, canvasCapabilities };
}
