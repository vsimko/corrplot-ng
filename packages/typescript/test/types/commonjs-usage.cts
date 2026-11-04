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
  const scene: facade.SceneInput = {
    width: 80,
    height: 30,
    cells: [{
      x: 0, y: 0, size: 10, visible: true,
      fill: { red: 255, green: 255, blue: 255, alpha: 255 },
      stroke: { red: 0, green: 0, blue: 0, alpha: 255 },
    }],
    texts: [],
    legend: [{ kind: "missing", label: "NA", color: { red: 242, green: 242, blue: 242, alpha: 255 } }],
    clusterRegions: [{
      id: "group-a", variables: ["a"], sourceIndices: [0],
      rowStart: 0, rowEnd: 0, columnStart: 0, columnEnd: 0,
      x: 1, y: 1, width: 8, height: 8,
      color: { red: 33, green: 102, blue: 172, alpha: 255 }, lineWidth: 2,
    }],
  };
  const svg = engine.render(scene, { format: "svg" });
  engine.dispose();
  return svg;
}
