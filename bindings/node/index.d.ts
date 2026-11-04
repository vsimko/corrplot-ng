export interface DiagnosticV1 {
  schema_version: 1;
  code: string;
  severity: "error" | "warning";
  stage: string;
  message: string;
  path?: string;
  row?: number;
  column?: number;
}

export interface MatrixInput {
  labels: string[];
  rows: number;
  columns: number;
  values: Array<number | null>;
  kind: "correlation" | "general_numeric";
}

export interface RgbaColorInput {
  red: number;
  green: number;
  blue: number;
  alpha: number;
}

export interface CellGlyphInput {
  kind: "circle" | "square" | "ellipse" | "pie" | "number" | "shade" | "color";
  radius?: number;
  side?: number;
  rx?: number;
  ry?: number;
  rotationDegrees?: number;
  startRadians?: number;
  endRadians?: number;
  text?: string;
  fontSize?: number;
  color?: RgbaColorInput;
}

export interface SceneCellSignificanceInput {
  pValue?: number;
  status: "unknown" | "significant" | "not_significant";
  annotation?: string;
  blank: boolean;
}

export interface SceneConfidenceIntervalInput {
  lower: number;
  upper: number;
  x1: number;
  x2: number;
  y: number;
  capHeight: number;
  strokeWidth: number;
  color: RgbaColorInput;
}

export interface SceneCellInput {
  x: number;
  y: number;
  size: number;
  visible: boolean;
  fill: RgbaColorInput;
  stroke: RgbaColorInput;
  glyph?: CellGlyphInput | null;
  pValue?: number | null;
  normalizedValue?: number | null;
  significance?: SceneCellSignificanceInput | null;
  confidenceInterval?: SceneConfidenceIntervalInput | null;
}

export interface SceneTextInput {
  text: string;
  semanticRole?: string;
  x: number;
  y: number;
  fontFamily?: string;
  fontSize: number;
  fontWeight?: string;
  fontStyle?: string;
  color: RgbaColorInput;
  anchor: "start" | "middle" | "end";
  rotationDegrees?: number;
}

export interface SceneSignificancePolicyInput {
  mode: "pch" | "p-value" | "blank" | "stars";
  thresholds: number[];
  symbol: string;
}

export interface SceneLegendEntryInput {
  kind: "stop" | "interval" | "missing";
  label: string;
  color: RgbaColorInput;
}

export interface SceneClusterRegionInput {
  id: string;
  variables: string[];
  sourceIndices: number[];
  rowStart: number;
  rowEnd: number;
  columnStart: number;
  columnEnd: number;
  x: number;
  y: number;
  width: number;
  height: number;
  color: RgbaColorInput;
  lineWidth: number;
}

export interface SceneInput {
  width: number;
  height: number;
  description?: string | null;
  cells: SceneCellInput[];
  texts: SceneTextInput[];
  significance?: SceneSignificancePolicyInput | null;
  legend?: SceneLegendEntryInput[] | null;
  clusterRegions?: SceneClusterRegionInput[] | null;
}

export interface SceneBuildOptionsInput {
  cellMethod?: "circle" | "square" | "ellipse" | "number" | "shade" | "color" | "pie";
  title?: string;
  description?: string;
  annotation?: string;
  displayType?: "full" | "upper" | "lower" | "diagonal";
  showDiagonal?: boolean;
}

export interface SceneResult {
  scene?: SceneInput | null;
  diagnostic?: DiagnosticV1 | null;
}

export interface SvgOptionsInput {
  width: number;
  height: number;
  unit: "px" | "mm" | "in";
  fontFamily: string;
  fontSizePt: number;
  background: RgbaColorInput;
}

export interface RgbaOptionsInput {
  widthPx: number;
  heightPx: number;
  dpi?: number;
  background: RgbaColorInput;
}

export interface RgbaOutput {
  widthPx: number;
  heightPx: number;
  strideBytes: number;
  pixelFormat: "rgba8";
  alphaMode: "straight";
  colorSpace: "srgb";
  dpi?: number;
  /** Owned pixel bytes; returned as a Node Buffer (a Uint8Array subclass). */
  pixels: Uint8Array;
}

export class CorrplotError extends Error {
  readonly diagnostic: DiagnosticV1;
}

export function validateMatrix(input: MatrixInput): true;
export function createScene(input: MatrixInput, options?: SceneBuildOptionsInput): SceneResult;
export function renderSceneSvg(scene: SceneInput, options: SvgOptionsInput): string;
export function renderSceneRgba(scene: SceneInput, options: RgbaOptionsInput): RgbaOutput;
export function renderMatrixSvg(sceneV1Json: string, method?: "circle" | "square" | "ellipse" | "number" | "shade" | "color" | "pie"): string;
export function renderMatrixRgba(sceneV1Json: string, method: "circle" | "square" | "ellipse" | "number" | "shade" | "color" | "pie", options: RgbaOptionsInput): RgbaOutput;
export function getRendererCapabilities(): unknown;
