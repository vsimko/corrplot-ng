export type BackendSelection = "auto" | "native" | "wasm";
export interface MatrixInput {
  labels: string[];
  rows: number;
  columns: number;
  values: Array<number | null>;
  kind: "correlation" | "general_numeric";
}
export interface RgbaColor { red: number; green: number; blue: number; alpha: number; }
export interface SvgRenderOptions { format: "svg"; method?: "circle" | "square" | "ellipse" | "number" | "shade" | "color" | "pie"; }
export interface RgbaRenderOptions {
  format: "rgba";
  method?: "circle" | "square" | "ellipse" | "number" | "shade" | "color" | "pie";
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
  validateMatrix(input: MatrixInput): true | { valid: boolean; diagnostic?: DiagnosticV1 | null };
  renderMatrixSvg(sceneV1Json: string, method: string): string | { svg?: string; diagnostic?: DiagnosticV1 | null };
  renderMatrixRgba(sceneV1Json: string, method: string, options: Omit<RgbaRenderOptions, "format" | "method">): RgbaOutput | (Partial<RgbaOutput> & { diagnostic?: DiagnosticV1 | null });
  getRendererCapabilities(): unknown | string;
  dispose?(): void;
}
export interface Engine {
  readonly backend: "native" | "wasm" | "injected";
  validate(input: MatrixInput): true;
  render(sceneV1Json: string, options: SvgRenderOptions): string;
  render(sceneV1Json: string, options: RgbaRenderOptions): RgbaOutput;
  getRendererCapabilities(): unknown;
  dispose(): void;
}
export function createCorrplotEngine(options?: { backend?: BackendSelection; adapter?: BackendAdapter }): Promise<Engine>;
