import type {
  DefineComponent,
  MaybeRefOrGetter,
  ShallowRef,
} from "vue";

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

export interface RgbaColor {
  red: number;
  green: number;
  blue: number;
  alpha: number;
}

export interface SvgRenderOptions {
  width?: number;
  height?: number;
  unit?: "px" | "mm" | "in";
  fontFamily?: string;
  fontSizePt?: number;
  background?: RgbaColor;
}

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

export class CorrplotError extends Error {
  readonly diagnostic: DiagnosticV1;
}

export interface CorrplotState {
  readonly svg: Readonly<ShallowRef<string | null>>;
  readonly loading: Readonly<ShallowRef<boolean>>;
  readonly error: Readonly<ShallowRef<Error | null>>;
}

export interface CorrplotProps {
  matrix: MatrixInput;
  sceneOptions?: SceneBuildOptions;
  svgOptions?: SvgRenderOptions;
  ariaLabel?: string;
  onError?: (error: Error) => void;
}

export declare const Corrplot: DefineComponent<CorrplotProps>;

export declare function useCorrplot(
  matrix: MaybeRefOrGetter<MatrixInput>,
  sceneOptions?: MaybeRefOrGetter<SceneBuildOptions>,
  svgOptions?: MaybeRefOrGetter<SvgRenderOptions>,
): CorrplotState;
