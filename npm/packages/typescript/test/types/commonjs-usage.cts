import facade = require("@corrplot-ng/typescript");

const engineFactory: typeof facade.createCorrplotEngine = facade.createCorrplotEngine;
const input: facade.MatrixInput = {
  labels: ["a"],
  rows: 1,
  columns: 1,
  values: [1],
  kind: "correlation",
};

export async function commonJsUsageExample() {
  const engine = await engineFactory({ backend: "native" });
  const scene: facade.SceneInput = engine.createScene(input);
  const svg = engine.render(scene, { format: "svg" });
  engine.dispose();
  return svg;
}
