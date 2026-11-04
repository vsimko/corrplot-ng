import { computed, h, ref } from "vue";
import {
  Corrplot,
  useCorrplot,
  type CorrplotProps,
  type MatrixInput,
  type SceneBuildOptions,
} from "@corrplot-ng/vue";

const matrix: MatrixInput = {
  labels: ["alpha", "beta"],
  rows: 2,
  columns: 2,
  values: new Float64Array([1, 0, 0, 1]),
  missingMask: [false, true, true, false],
  kind: "correlation",
};

const matrixRef = ref(matrix);
const sceneOptions: SceneBuildOptions = {
  cellMethod: "ellipse",
  title: "Typed Vue figure",
};

const reactiveMatrix = computed(() => ({
  ...matrixRef.value,
  values: matrixRef.value.values,
}));

const state = useCorrplot(reactiveMatrix, () => ({
  ...sceneOptions,
  title: "Revision " + matrixRef.value.rows,
}));

const props: CorrplotProps = {
  matrix,
  sceneOptions,
  svgOptions: { width: 320, height: 240 },
  ariaLabel: "Typed correlation figure",
  onError: error => console.error(error.message),
};

h(Corrplot, props);
h(Corrplot, {
  matrix: matrixRef.value,
  onError: error => console.error(error.message),
});

if (state.error.value) console.error(state.error.value.message);
matrixRef.value = { ...matrix, values: [1, 0.2, 0.2, 1] };
