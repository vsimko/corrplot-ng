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
  start_radians?: number;
  end_radians?: number;
  text?: string;
  font_size?: number;
  color?: RgbaColorInput;
}

export interface SceneCellInput {
  x: number;
  y: number;
  size: number;
  visible: boolean;
  fill: RgbaColorInput;
  stroke: RgbaColorInput;
  glyph?: CellGlyphInput;
}

export interface SceneTextInput {
  text: string;
  x: number;
  y: number;
  font_size: number;
  color: RgbaColorInput;
  anchor: "start" | "middle" | "end";
}

export interface SceneInput {
  width: number;
  height: number;
  cells: SceneCellInput[];
  texts: SceneTextInput[];
}

export interface SvgOptionsInput {
  width: number;
  height: number;
  unit: "px" | "mm" | "in";
  font_family: string;
  font_size_pt: number;
  background: RgbaColorInput;
}

export class CorrplotError extends Error {
  readonly diagnostic: DiagnosticV1;
}

export function validateMatrix(input: MatrixInput): true;
export function renderSceneSvg(scene: SceneInput, options: SvgOptionsInput): string;
export function getRendererCapabilities(): unknown;
