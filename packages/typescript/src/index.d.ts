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
export interface SceneCellSignificance {
  pValue?: number;
  status: "unknown" | "significant" | "not_significant";
  annotation?: string;
  blank: boolean;
}
export interface SceneConfidenceInterval {
  lower: number;
  upper: number;
  x1: number;
  x2: number;
  y: number;
  capHeight: number;
  strokeWidth: number;
  color: RgbaColor;
}
export interface SceneSignificancePolicy {
  mode: "pch" | "p-value" | "blank" | "stars";
  thresholds: number[];
  symbol: string;
}
export interface SceneLegendEntryInput {
  kind: "stop" | "interval" | "missing";
  label: string;
  color: RgbaColor;
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
  color: RgbaColor;
  lineWidth: number;
}
export interface SceneCellInput {
  x: number;
  y: number;
  size: number;
  visible: boolean;
  fill: RgbaColor;
  stroke: RgbaColor;
  glyph?: CellGlyphInput | null;
  pValue?: number | null;
  normalizedValue?: number | null;
  significance?: SceneCellSignificance;
  confidenceInterval?: SceneConfidenceInterval;
}
export interface SceneTextInput {
  text: string;
  semanticRole?: string;
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
  significance?: SceneSignificancePolicy;
  legend?: SceneLegendEntryInput[];
  clusterRegions?: SceneClusterRegionInput[];
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
export type CanvasTarget =
  | HTMLCanvasElement
  | OffscreenCanvas
  | CanvasRenderingContext2D
  | OffscreenCanvasRenderingContext2D;
export interface CanvasRenderOptions {
  /** Logical CSS-pixel width; defaults to the scene width. */
  width?: number;
  /** Logical CSS-pixel height; defaults to the scene height. */
  height?: number;
  /** Backing-store scale; defaults to globalThis.devicePixelRatio or 1. */
  devicePixelRatio?: number;
  /** Defaults to white when clear is true; null keeps the cleared canvas transparent. */
  background?: RgbaColor | null;
  /** Clear the backing store before drawing. Defaults to true. */
  clear?: boolean;
  /** Set HTMLCanvasElement CSS dimensions to logical width/height. Defaults to true. */
  setCssSize?: boolean;
  /** Font family used when scene text does not specify one. */
  fontFamily?: string;
}
export interface CanvasRenderResult {
  canvas: HTMLCanvasElement | OffscreenCanvas;
  context: CanvasRenderingContext2D | OffscreenCanvasRenderingContext2D;
  widthPx: number;
  heightPx: number;
  devicePixelRatio: number;
}
export interface CanvasRendererCapabilities {
  capability_schema_version: 1;
  renderer: { id: string; name: string; version: string };
  scene_schema_versions: number[];
  output_formats: string[];
  host_targets: Array<"browser-canvas-2d" | "browser-offscreen-canvas-2d">;
  matrix_kinds: Array<"correlation_plot" | "general_matrix_plot">;
  cell_methods: CellMethod[];
  features: Record<string, boolean>;
  negotiation_modes: Array<"strict" | "permissive">;
}
export interface DiagnosticV1 {
  schema_version: 1; code: string; severity: "error" | "warning";
  stage: string; message: string; path?: string; row?: number; column?: number;
}
export class CorrplotError extends Error { readonly diagnostic: DiagnosticV1; }
export interface BackendAdapter {
  validateMatrix(input: Omit<MatrixInput, "missingMask"> & { values: Array<number | null> }): true | { valid: boolean; diagnostic?: DiagnosticV1 | null };
  createScene?(input: Omit<MatrixInput, "missingMask"> & { values: Array<number | null> }, options: SceneBuildOptions): SceneResult;
  renderSceneSvg?(scene: SceneInput, options: Omit<SvgRenderOptions, "format" | "method"> & { width: number; height: number; unit: "px" | "mm" | "in"; fontFamily: string; fontSizePt: number; background: RgbaColor }): string | { svg?: string | null; diagnostic?: DiagnosticV1 | null };
  renderSceneRgba?(scene: SceneInput, options: Omit<RgbaRenderOptions, "format" | "method">): RgbaOutput | (Partial<RgbaOutput> & { diagnostic?: DiagnosticV1 | null });
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
  /** Browser-only direct Canvas 2D rendering; Node backends report renderer.unsupported_feature. */
  renderToCanvas(scene: SceneInput, target: CanvasTarget, options?: CanvasRenderOptions): CanvasRenderResult;
  getCanvasRendererCapabilities(): CanvasRendererCapabilities;
  getRendererCapabilities(rendererId?: string): unknown;
  dispose(): void;
}
export function createCorrplotEngine(options?: { backend?: BackendSelection; adapter?: BackendAdapter }): Promise<Engine>;
