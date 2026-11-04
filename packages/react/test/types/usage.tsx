import { createElement } from "react";
import {
  Corrplot,
  useCorrplot,
  type CorrplotProps,
  type MatrixInput,
  type SceneBuildOptions,
} from "@corrplot-ng/react";

const matrix: MatrixInput = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: new Float64Array([1, 0, 0, 1]),
  missingMask: [false, true, true, false],
  kind: "correlation",
};

const sceneOptions: SceneBuildOptions = {
  cellMethod: "ellipse",
  title: "Typed React figure",
};

const props: CorrplotProps = {
  matrix,
  sceneOptions,
  svgOptions: { width: 320, height: 240 },
  ariaLabel: "Typed correlation figure",
  onError: error => console.error(error.message),
};

createElement(Corrplot, props);

function Example() {
  const state = useCorrplot(matrix, sceneOptions, { width: 320, height: 240 });
  if (state.error) console.error(state.error.message);
  return createElement(Corrplot, { matrix, sceneOptions });
}

void Example;
