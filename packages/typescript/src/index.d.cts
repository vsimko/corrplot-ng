export type BackendSelection = "auto" | "native" | "wasm";
export type CellMethod = "circle" | "square" | "ellipse" | "number" | "shade" | "color" | "pie";
export type DisplayType = "full" | "upper" | "lower" | "diagonal";
export type NumericTypedArray =
  | Int8Array | Uint8Array | Uint8ClampedArray
  | Int16Array | Uint16Array
  | Int32Array | Uint32Array
  | Float32Array | Float64Array;

export interface MatrixInput {
  labels: string[];
  rows: number;
  columns: number;
  /** Row-major values. Use null in arrays or missingMask with numeric typed arrays for missing cells. */
  values: Array<number | null> | NumericTypedArray;
  /** Explicit missing-cell mask; true means the corresponding value is missing. */
  missingMask?: ArrayLike<boolean>;
  kind: "correlation" | "general_numeric";
}
export interface SceneBuildOptions {
  cellMethod?: CellMethod;
  title?: string;
  description?: string;
  annotation?: string;
  displayType?: DisplayType;
  showDiagonal?: boolean;
}
export interface RgbaColor { red: number; green: number; blue: number; alpha: number; }
export interface CellGlyphInput {
  kind: CellMethod;
  radius?: number;
  side?: number;
  rx?: number;
  ry?: number;
  rotationDegrees?: number;
  startRadians?: number;
  endRadians?: number;
  text?: string;
  fontSize?: number;
  color?: RgbaColor;
}
export interface SceneCellInput {
  x: number;
  y: number;
  size: number;
  visible: boolean;
  fill: RgbaColor;
  stroke: RgbaColor;
  glyph?: CellGlyphInput;
}
export interface SceneTextInput {
  text: string;
  x: number;
  y: number;
  fontSize: number;
  color: RgbaColor;
  anchor: "start" | "middle" | "end";
  rotationDegrees?: number;
}
export interface SceneInput {
  width: number;
  height: number;
  description?: string;
  cells: SceneCellInput[];
  texts: SceneTextInput[];
}
export interface SceneResult {
  scene?: SceneInput | null;
  diagnostic?: DiagnosticV1 | null;
}
export interface SvgRenderOptions {
  format: "svg";
  method?: CellMethod;
  width?: number;
  height?: number;
  unit?: "px" | "mm" | "in";
  fontFamily?: string;
  fontSizePt?: number;
  background?: RgbaColor;
}
export interface RgbaRenderOptions {
  format: "rgba";
  method?: CellMethod;
  widthPx: number;
  heightPx: number;
  dpi?: number;
  background: RgbaColor;
}
export interface RgbaOutput {
  widthPx: number; heightPx: number; strideBytes: number;
  pixelFormat: "rgba8"; alphaMode: "straight"; colorSpace: "srgb";
  dpi?: number; pixels: Uint8Array;
}
export interface DiagnosticV1 {
  schema_version: 1; code: string; severity: "error" | "warning";
  stage: string; message: string; path?: string; row?: number; column?: number;
}
export class CorrplotError extends Error { readonly diagnostic: DiagnosticV1; }
export interface BackendAdapter {
  validateMatrix(input: Omit<MatrixInput, "missingMask"> & { values: Array<number | null> }): true | { valid: boolean; diagnostic?: DiagnosticV1 | null };
  createScene(input: Omit<MatrixInput, "missingMask"> & { values: Array<number | null> }, options: SceneBuildOptions): SceneResult;
  renderSceneSvg(scene: SceneInput, options: Omit<SvgRenderOptions, "format" | "method"> & { width: number; height: number; unit: "px" | "mm" | "in"; fontFamily: string; fontSizePt: number; background: RgbaColor }): string | { svg?: string | null; diagnostic?: DiagnosticV1 | null };
  renderSceneRgba(scene: SceneInput, options: Omit<RgbaRenderOptions, "format" | "method">): RgbaOutput | (Partial<RgbaOutput> & { diagnostic?: DiagnosticV1 | null });
  renderMatrixSvg(sceneV1Json: string, method: string): string | { svg?: string | null; diagnostic?: DiagnosticV1 | null };
  renderMatrixRgba(sceneV1Json: string, method: string, options: Omit<RgbaRenderOptions, "format" | "method">): RgbaOutput | (Partial<RgbaOutput> & { diagnostic?: DiagnosticV1 | null });
  getRendererCapabilities(): unknown | string;
  dispose?(): void;
}
export interface Engine {
  readonly backend: "native" | "wasm" | "injected";
  validate(input: MatrixInput): true;
  createScene(input: MatrixInput, options?: SceneBuildOptions): SceneInput;
  render(scene: SceneInput, options: SvgRenderOptions): string;
  render(scene: SceneInput, options: RgbaRenderOptions): RgbaOutput;
  /** Compatibility path for callers that already hold a matrix-bearing scene-v1 JSON document. */
  render(sceneV1Json: string, options: SvgRenderOptions): string;
  render(sceneV1Json: string, options: RgbaRenderOptions): RgbaOutput;
  getRendererCapabilities(rendererId?: string): unknown;
  dispose(): void;
}
export function createCorrplotEngine(options?: { backend?: BackendSelection; adapter?: BackendAdapter }): Promise<Engine>;
