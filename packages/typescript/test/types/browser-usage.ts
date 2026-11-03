import {
  createCorrplotEngine,
  type MatrixInput,
  type SceneBuildOptions,
  type SvgRenderOptions,
  type RgbaRenderOptions,
} from "@corrplot-ng/typescript";

const matrix: MatrixInput = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: [1, null, null, 1],
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

export async function browserUsageExample() {
  const engine = await createCorrplotEngine({ backend: "wasm" });
  const scene = engine.createScene(matrix, sceneOptions);
  const svg: string = engine.render(scene, svgOptions);
  const rgba = engine.render(scene, rgbaOptions);
  engine.dispose();
  return { svg, rgba };
}
