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
  values: [1, 0.5, 0.5, 1],
  kind: "correlation",
};
const sceneOptions: SceneBuildOptions = {
  cellMethod: "ellipse",
  title: "Node usage",
  description: "A typed API example.",
  displayType: "upper",
  showDiagonal: true,
};
const svgOptions: SvgRenderOptions = { format: "svg", unit: "px" };
const rgbaOptions: RgbaRenderOptions = {
  format: "rgba",
  widthPx: 320,
  heightPx: 320,
  background: { red: 255, green: 255, blue: 255, alpha: 255 },
};

export async function nodeUsageExample() {
  const engine = await createCorrplotEngine({ backend: "native" });
  engine.validate(matrix);
  const scene = engine.createScene(matrix, sceneOptions);
  const svg: string = engine.render(scene, svgOptions);
  const rgba = engine.render(scene, rgbaOptions);
  engine.dispose();
  return { svg, rgba };
}
