use corrplot_core::output::{OutputOptions, RasterOptions, RenderOutput, Renderer, RgbaColor, SvgOptions, SvgUnit};
use corrplot_core::raster::{CellGlyph, RgbaRenderer, Scene, SceneCell, SceneSvgRenderer, SceneText, TextAnchor};
use js_sys::{Array, Reflect, Uint8Array};
use corrplot_core::{validate_matrix, MatrixKind, RENDERER_CAPABILITIES_V1};
use corrplot_core::scene::{scene_v1_to_scene, CellMethod};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub stage: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,


}

#[derive(Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub diagnostic: Option<DiagnosticV1>,
}

#[derive(Serialize, Deserialize)]
pub struct SvgResult {
    pub svg: Option<String>,
    pub diagnostic: Option<DiagnosticV1>,
}

#[derive(Serialize, Deserialize)]
pub struct MatrixInput {
    pub labels: Vec<String>,
    pub rows: u32,
    pub columns: u32,
    pub values: Vec<Option<f64>>,
    pub kind: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RgbaColorInput {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellGlyphInput {
    pub kind: String,
    pub radius: Option<f64>,
    pub side: Option<f64>,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
    pub rotation_degrees: Option<f64>,
    pub start_radians: Option<f64>,
    pub end_radians: Option<f64>,
    pub text: Option<String>,
    pub font_size: Option<f64>,
    pub color: Option<RgbaColorInput>,
}

#[derive(Serialize, Deserialize)]
pub struct SceneCellInput {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub visible: bool,
    pub fill: RgbaColorInput,
    pub stroke: RgbaColorInput,
    pub glyph: Option<CellGlyphInput>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneTextInput {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub color: RgbaColorInput,
    pub anchor: String,
    pub rotation_degrees: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct SceneInput {
    pub width: f64,
    pub height: f64,
    pub cells: Vec<SceneCellInput>,
    pub texts: Vec<SceneTextInput>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgOptionsInput {
    pub width: f64,
    pub height: f64,
    pub unit: String,
    pub font_family: String,
    pub font_size_pt: f64,
    pub background: RgbaColorInput,
}

fn diagnostic(code: &str, stage: &str, message: impl Into<String>, path: Option<&str>) -> DiagnosticV1 {
    DiagnosticV1 {
        schema_version: 1,
        code: code.to_string(),
        severity: "error".to_string(),
        stage: stage.to_string(),
        message: message.into(),
        path: path.map(str::to_string),
        row: None,
        column: None,
    }
}

fn matrix_diagnostic(error: corrplot_core::MatrixValidationError) -> DiagnosticV1 {
    DiagnosticV1 {
        schema_version: 1,
        code: error.code.to_string(),
        severity: "error".to_string(),
        stage: "input".to_string(),
        message: error.message,
        path: Some("matrix".to_string()),
        row: error.row.map(|value| value as u32),
        column: error.column.map(|value| value as u32),
    }
}

fn to_js<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|error| JsValue::from_str(&error.to_string()))
}

fn from_js<T: for<'de> Deserialize<'de>>(value: JsValue, name: &str) -> Result<T, DiagnosticV1> {
    serde_wasm_bindgen::from_value(value).map_err(|error| diagnostic(
        "configuration.invalid_field",
        "configuration",
        format!("invalid {name}: {error}"),
        Some(name),
    ))
}

#[wasm_bindgen(js_name = "validateMatrix")]
pub fn validate_matrix_wasm(input: JsValue) -> Result<JsValue, JsValue> {
    let input: MatrixInput = match from_js(input, "matrix") {
        Ok(input) => input,
        Err(diagnostic) => return to_js(&ValidationResult { valid: false, diagnostic: Some(diagnostic) }),
    };
    let kind = match input.kind.as_str() {
        "correlation" => MatrixKind::Correlation,
        "general_numeric" => MatrixKind::GeneralNumeric,
        other => {
            let diagnostic = diagnostic(
                "configuration.invalid_field",
                "configuration",
                format!("unsupported matrix kind '{other}'"),
                Some("kind"),
            );
            return to_js(&ValidationResult { valid: false, diagnostic: Some(diagnostic) });
        }
    };
    let result = validate_matrix(
        &input.labels,
        input.rows as usize,
        input.columns as usize,
        &input.values,
        kind,
    );
    match result {
        Ok(()) => to_js(&ValidationResult { valid: true, diagnostic: None }),
        Err(error) => to_js(&ValidationResult { valid: false, diagnostic: Some(matrix_diagnostic(error)) }),
    }
}

fn rgba(input: RgbaColorInput, path: &str) -> Result<RgbaColor, DiagnosticV1> {
    let channel = |value: u32, name: &str| {
        u8::try_from(value).map_err(|_| diagnostic(
            "scene.invalid_content",
            "scene",
            format!("{path}.{name} must be an integer from 0 to 255"),
            Some(path),
        ))
    };
    Ok(RgbaColor {
        red: channel(input.red, "red")?,
        green: channel(input.green, "green")?,
        blue: channel(input.blue, "blue")?,
        alpha: channel(input.alpha, "alpha")?,
    })
}

fn required<T>(value: Option<T>, name: &str) -> Result<T, DiagnosticV1> {
    value.ok_or_else(|| diagnostic(
        "scene.invalid_content",
        "scene",
        format!("scene glyph requires '{name}'"),
        Some("cells.glyph"),
    ))
}

fn glyph(input: CellGlyphInput) -> Result<CellGlyph, DiagnosticV1> {
    match input.kind.as_str() {
        "circle" => Ok(CellGlyph::Circle {
            radius: required(input.radius, "radius")?,
            color: rgba(required(input.color, "color")?, "cells.glyph.color")?,
        }),
        "square" => Ok(CellGlyph::Square {
            side: required(input.side, "side")?,
            color: rgba(required(input.color, "color")?, "cells.glyph.color")?,
        }),
        "ellipse" => Ok(CellGlyph::Ellipse {
            rx: required(input.rx, "rx")?,
            ry: required(input.ry, "ry")?,
            rotation_degrees: input.rotation_degrees.unwrap_or(0.0),
            color: rgba(required(input.color, "color")?, "cells.glyph.color")?,
        }),
        "pie" => Ok(CellGlyph::Pie {
            radius: required(input.radius, "radius")?,
            start_radians: required(input.start_radians, "startRadians")?,
            end_radians: required(input.end_radians, "endRadians")?,
            color: rgba(required(input.color, "color")?, "cells.glyph.color")?,
        }),
        "number" => Ok(CellGlyph::Number {
            text: required(input.text, "text")?,
            font_size: required(input.font_size, "fontSize")?,
            color: rgba(required(input.color, "color")?, "cells.glyph.color")?,
        }),
        "shade" => Ok(CellGlyph::Shade {
            color: rgba(required(input.color, "color")?, "cells.glyph.color")?,
        }),
        "color" => Ok(CellGlyph::Color),
        other => Err(diagnostic(
            "renderer.unsupported_feature",
            "renderer",
            format!("unsupported cell glyph '{other}'"),
            Some("cells.glyph.kind"),
        )),
    }
}

fn scene(input: SceneInput) -> Result<Scene, DiagnosticV1> {
    let mut cells = Vec::with_capacity(input.cells.len());
    for (index, cell) in input.cells.into_iter().enumerate() {
        let fill = rgba(cell.fill, &format!("cells[{index}].fill"))?;
        let stroke = rgba(cell.stroke, &format!("cells[{index}].stroke"))?;
        cells.push(SceneCell {
            x: cell.x,
            y: cell.y,
            size: cell.size,
            visible: cell.visible,
            fill,
            stroke,
            glyph: cell.glyph.map(glyph).transpose()?,
            p_value: None, normalized_value: None, significance: None, confidence_interval: None,
        });
    }

    let mut texts = Vec::with_capacity(input.texts.len());
    for (index, text) in input.texts.into_iter().enumerate() {
        let anchor = match text.anchor.as_str() {
            "start" => TextAnchor::Start,
            "middle" => TextAnchor::Middle,
            "end" => TextAnchor::End,
            other => return Err(diagnostic(
                "scene.invalid_content",
                "scene",
                format!("scene text anchor '{other}' is not supported"),
                Some(&format!("texts[{index}].anchor")),
            )),
        };
        texts.push(SceneText {
            text: text.text,
            semantic_role: None,
            x: text.x,
            y: text.y,
            font_size: text.font_size,
            color: rgba(text.color, &format!("texts[{index}].color"))?,
            anchor,
            font_family: String::new(),
            font_weight: "normal".into(),
            font_style: "normal".into(),
            rotation_degrees: text.rotation_degrees.unwrap_or(0.0),
        });
    }

    Ok(Scene { width: input.width, height: input.height, description: String::new(), cells, texts, significance: None, legend: Vec::new(), cluster_regions: Vec::new() })
}

#[wasm_bindgen(js_name = "renderSceneSvg")]
pub fn render_scene_svg_wasm(scene_input: JsValue, options_input: JsValue) -> Result<JsValue, JsValue> {
    let input: SceneInput = match from_js(scene_input, "scene") {
        Ok(input) => input,
        Err(diagnostic) => return to_js(&SvgResult { svg: None, diagnostic: Some(diagnostic) }),
    };
    let options: SvgOptionsInput = match from_js(options_input, "options") {
        Ok(options) => options,
        Err(diagnostic) => return to_js(&SvgResult { svg: None, diagnostic: Some(diagnostic) }),
    };
    let scene = match scene(input) {
        Ok(scene) => scene,
        Err(diagnostic) => return to_js(&SvgResult { svg: None, diagnostic: Some(diagnostic) }),
    };
    let unit = match options.unit.as_str() {
        "px" => SvgUnit::Pixels,
        "mm" => SvgUnit::Millimeters,
        "in" => SvgUnit::Inches,
        other => {
            let diagnostic = diagnostic(
                "renderer.output.invalid_options",
                "renderer",
                format!("unsupported SVG unit '{other}'"),
                Some("output.unit"),
            );
            return to_js(&SvgResult { svg: None, diagnostic: Some(diagnostic) });
        }
    };
    let background = match rgba(options.background, "output.background") {
        Ok(color) => color,
        Err(diagnostic) => return to_js(&SvgResult { svg: None, diagnostic: Some(diagnostic) }),
    };
    let mut svg = SvgOptions::new(options.width, options.height, unit);
    if !options.font_family.trim().is_empty() {
        svg.font_family = options.font_family;
    }
    svg.font_size_pt = options.font_size_pt;
    svg.background = background;

    let result = match SceneSvgRenderer.render(&scene, &OutputOptions::Svg(svg)) {
        Ok(RenderOutput::Svg(svg)) => SvgResult { svg: Some(svg), diagnostic: None },
        Ok(_) => SvgResult {
            svg: None,
            diagnostic: Some(diagnostic(
                "renderer.output.format_mismatch",
                "export",
                "SVG renderer returned a non-SVG output",
                Some("output"),
            )),
        },
        Err(error) => SvgResult {
            svg: None,
            diagnostic: Some(diagnostic(error.code, "renderer", error.message, Some("output"))),
        },
    };
    to_js(&result)
}

#[wasm_bindgen(js_name = "getRendererCapabilities")]
pub fn get_renderer_capabilities_wasm() -> String {
    RENDERER_CAPABILITIES_V1.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn validates_using_the_shared_rust_core() {
        let input = MatrixInput {
            labels: vec!["alpha".into(), "beta".into()],
            rows: 2,
            columns: 2,
            values: vec![Some(1.0), Some(0.5), Some(0.5), Some(1.0)],
            kind: "correlation".into(),
        };
        let result = validate_matrix_wasm(serde_wasm_bindgen::to_value(&input).unwrap()).unwrap();
        let result: ValidationResult = serde_wasm_bindgen::from_value(result).unwrap();
        assert!(result.valid);
        assert!(result.diagnostic.is_none());
    }

    #[wasm_bindgen_test]
    fn maps_shared_matrix_errors_to_diagnostic_v1() {
        let input = MatrixInput {
            labels: vec!["alpha".into(), "beta".into()],
            rows: 2,
            columns: 2,
            values: vec![Some(1.0), Some(0.5), Some(0.2), Some(1.0)],
            kind: "correlation".into(),
        };
        let result = validate_matrix_wasm(serde_wasm_bindgen::to_value(&input).unwrap()).unwrap();
        let result: ValidationResult = serde_wasm_bindgen::from_value(result).unwrap();
        let diagnostic = result.diagnostic.unwrap();
        assert_eq!(diagnostic.code, "input.matrix.not_symmetric");
        assert_eq!(diagnostic.stage, "input");
    }

    #[wasm_bindgen_test]
    fn renders_scene_v1_matrix_and_maps_invalid_method_to_diagnostic_v1() {
        let scene_v1 = include_str!("../../../prototypes/shared-svg/scene-v1.example.json");

        let output = render_matrix_svg_wasm(scene_v1.into(), "ellipse".into()).unwrap();
        let output: SvgResult = serde_wasm_bindgen::from_value(output).unwrap();
        let svg = output.svg.unwrap();
        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("<circle "));

        let output = render_matrix_svg_wasm(scene_v1.into(), "unknown".into()).unwrap();
        let output: SvgResult = serde_wasm_bindgen::from_value(output).unwrap();
        let diagnostic = output.diagnostic.unwrap();
        assert_eq!(diagnostic.schema_version, 1);
        assert_eq!(diagnostic.code, "configuration.invalid_field");
        assert_eq!(diagnostic.severity, "error");
        assert_eq!(diagnostic.stage, "configuration");
        assert_eq!(diagnostic.message, "unsupported cell method 'unknown'");
        assert_eq!(diagnostic.path.as_deref(), Some("cellMethod"));
        assert_eq!(diagnostic.row, None);
        assert_eq!(diagnostic.column, None);
        assert!(output.svg.is_none());
    }

    #[wasm_bindgen_test]
    fn renders_shared_scene_v1_construction_fixture() {
        let fixture = include_str!("../../../prototypes/shared-svg/scene-v1-construction.example.json");
        let output = render_matrix_svg_wasm(fixture.into(), "circle".into()).unwrap();
        let output: SvgResult = serde_wasm_bindgen::from_value(output).unwrap();
        let svg = output.svg.unwrap();
        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("A caption"));
        assert!(svg.contains("data-semantic-role=\"caption\""));
        assert!(svg.contains("second line"));
    }

    #[wasm_bindgen_test]
    fn renders_typed_scene_with_the_shared_svg_renderer() {
        let scene = SceneInput {
            width: 10.0,
            height: 10.0,
            cells: vec![SceneCellInput {
                x: 0.0,
                y: 0.0,
                size: 10.0,
                visible: true,
                fill: RgbaColorInput { red: 255, green: 255, blue: 255, alpha: 255 },
                stroke: RgbaColorInput { red: 0, green: 0, blue: 0, alpha: 255 },
                glyph: Some(CellGlyphInput {
                    kind: "circle".into(),
                    radius: Some(3.0),
                    side: None,
                    rx: None,
                    ry: None,
                    rotation_degrees: None,
                    start_radians: None,
                    end_radians: None,
                    text: None,
                    font_size: None,
                    color: Some(RgbaColorInput { red: 0, green: 0, blue: 0, alpha: 255 }),
                }),
            }],
            texts: vec![],
        };
        let options = SvgOptionsInput {
            width: 100.0,
            height: 100.0,
            unit: "px".into(),
            font_family: String::new(),
            font_size_pt: 10.0,
            background: RgbaColorInput { red: 255, green: 255, blue: 255, alpha: 255 },
        };
        let output = render_scene_svg_wasm(
            serde_wasm_bindgen::to_value(&scene).unwrap(),
            serde_wasm_bindgen::to_value(&options).unwrap(),
        ).unwrap();
        let output: SvgResult = serde_wasm_bindgen::from_value(output).unwrap();
        let svg = output.svg.unwrap();
        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("<circle "));
    }

    #[wasm_bindgen_test]
    fn renders_owned_rgba_typed_array_with_shared_raster_renderer() {
        let scene = SceneInput {
            width: 10.0,
            height: 10.0,
            cells: vec![SceneCellInput {
                x: 0.0,
                y: 0.0,
                size: 10.0,
                visible: true,
                fill: RgbaColorInput { red: 255, green: 255, blue: 255, alpha: 255 },
                stroke: RgbaColorInput { red: 0, green: 0, blue: 0, alpha: 255 },
                glyph: Some(CellGlyphInput {
                    kind: "circle".into(),
                    radius: Some(3.0),
                    side: None,
                    rx: None,
                    ry: None,
                    rotation_degrees: None,
                    start_radians: None,
                    end_radians: None,
                    text: None,
                    font_size: None,
                    color: Some(RgbaColorInput { red: 0, green: 0, blue: 0, alpha: 255 }),
                }),
            }],
            texts: vec![],
        };
        let options = RgbaOptionsInput {
            width_px: 16,
            height_px: 12,
            dpi: Some(96.0),
            background: RgbaColorInput { red: 255, green: 255, blue: 255, alpha: 255 },
        };
        let output = render_scene_rgba_wasm(
            serde_wasm_bindgen::to_value(&scene).unwrap(),
            serde_wasm_bindgen::to_value(&options).unwrap(),
        ).unwrap();
        let pixels = Reflect::get(&output, &JsValue::from_str("pixels")).unwrap()
            .dyn_into::<Uint8Array>().unwrap();
        assert_eq!(pixels.length(), 16 * 12 * 4);
        assert_eq!(Reflect::get(&output, &JsValue::from_str("widthPx")).unwrap().as_f64(), Some(16.0));
        assert_eq!(Reflect::get(&output, &JsValue::from_str("heightPx")).unwrap().as_f64(), Some(12.0));
        assert_eq!(Reflect::get(&output, &JsValue::from_str("strideBytes")).unwrap().as_f64(), Some(64.0));
        assert_eq!(Reflect::get(&output, &JsValue::from_str("pixelFormat")).unwrap().as_string().as_deref(), Some("rgba8"));
        let diagnostic = Reflect::get(&output, &JsValue::from_str("diagnostic")).unwrap();
        assert!(diagnostic.is_undefined() || diagnostic.is_null());
    }

    #[wasm_bindgen_test]
    fn maps_invalid_rgba_options_to_diagnostic_v1() {
        let scene = SceneInput { width: 1.0, height: 1.0, cells: vec![], texts: vec![] };
        let options = RgbaOptionsInput {
            width_px: 0,
            height_px: 12,
            dpi: None,
            background: RgbaColorInput { red: 255, green: 255, blue: 255, alpha: 255 },
        };
        let output = render_scene_rgba_wasm(
            serde_wasm_bindgen::to_value(&scene).unwrap(),
            serde_wasm_bindgen::to_value(&options).unwrap(),
        ).unwrap();
        let diagnostic: DiagnosticV1 = serde_wasm_bindgen::from_value(
            Reflect::get(&output, &JsValue::from_str("diagnostic")).unwrap()
        ).unwrap();
        assert_eq!(diagnostic.code, "renderer.output.invalid_options");
        assert_eq!(diagnostic.stage, "renderer");
        assert!(Reflect::get(&output, &JsValue::from_str("pixels")).unwrap().is_null());
    }

}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RgbaOptionsInput {
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: Option<f64>,
    pub background: RgbaColorInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RgbaMetadata {
    width_px: Option<u32>,
    height_px: Option<u32>,
    stride_bytes: Option<u32>,
    pixel_format: Option<String>,
    alpha_mode: Option<String>,
    color_space: Option<String>,
    dpi: Option<f64>,
    diagnostic: Option<DiagnosticV1>,
}

fn rgba_result(metadata: RgbaMetadata, pixels: Option<Uint8Array>) -> Result<JsValue, JsValue> {
    let result = to_js(&metadata)?;
    let pixels = pixels.map(JsValue::from).unwrap_or(JsValue::NULL);
    Reflect::set(&result, &JsValue::from_str("pixels"), &pixels)?;
    Ok(result)
}

fn rgba_failure(diagnostic: DiagnosticV1) -> Result<JsValue, JsValue> {
    rgba_result(RgbaMetadata {
        width_px: None,
        height_px: None,
        stride_bytes: None,
        pixel_format: None,
        alpha_mode: None,
        color_space: None,
        dpi: None,
        diagnostic: Some(diagnostic),
    }, None)
}

#[wasm_bindgen(js_name = "renderSceneRgba")]
pub fn render_scene_rgba_wasm(scene_input: JsValue, options_input: JsValue) -> Result<JsValue, JsValue> {
    let input: SceneInput = match from_js(scene_input, "scene") {
        Ok(input) => input,
        Err(diagnostic) => return rgba_failure(diagnostic),
    };
    let options: RgbaOptionsInput = match from_js(options_input, "options") {
        Ok(options) => options,
        Err(diagnostic) => return rgba_failure(diagnostic),
    };
    let scene = match scene(input) {
        Ok(scene) => scene,
        Err(diagnostic) => return rgba_failure(diagnostic),
    };
    let background = match rgba(options.background, "output.background") {
        Ok(background) => background,
        Err(diagnostic) => return rgba_failure(diagnostic),
    };
    let raster = RasterOptions {
        width_px: options.width_px,
        height_px: options.height_px,
        dpi: options.dpi,
        background,
    };

    match RgbaRenderer.render(&scene, &OutputOptions::Rgba(raster)) {
        Ok(RenderOutput::Rgba(buffer)) => {
            let stride_bytes = match u32::try_from(buffer.stride_bytes) {
                Ok(stride) => stride,
                Err(_) => return rgba_failure(diagnostic(
                    "renderer.output.invalid_buffer",
                    "export",
                    "RGBA stride cannot be represented by the WebAssembly binding",
                    Some("output.strideBytes"),
                )),
            };
            // wasm memory cannot safely back a long-lived JS typed array: Rust
            // allocations may move or be freed after this call. Copy once into
            // JS-owned memory so the returned Uint8Array has stable ownership.
            let pixels = Uint8Array::from(buffer.pixels.as_slice());
            rgba_result(RgbaMetadata {
                width_px: Some(buffer.width_px),
                height_px: Some(buffer.height_px),
                stride_bytes: Some(stride_bytes),
                pixel_format: Some("rgba8".to_string()),
                alpha_mode: Some("straight".to_string()),
                color_space: Some("srgb".to_string()),
                dpi: buffer.dpi,
                diagnostic: None,
            }, Some(pixels))
        }
        Ok(_) => rgba_failure(diagnostic(
            "renderer.output.format_mismatch",
            "export",
            "RGBA renderer returned a non-RGBA output",
            Some("output"),
        )),
        Err(error) => rgba_failure(diagnostic(error.code, "renderer", error.message, Some("output"))),
    }
}


fn scene_build_diagnostic(error: corrplot_core::scene::SceneBuildError) -> DiagnosticV1 {
    DiagnosticV1 {
        schema_version: 1,
        code: error.code.to_string(),
        severity: "error".to_string(),
        stage: "input".to_string(),
        message: error.message,
        path: Some(error.path),
        row: error.row.and_then(|value| u32::try_from(value).ok()),
        column: error.column.and_then(|value| u32::try_from(value).ok()),
    }
}

fn cell_method(value: &str) -> Result<CellMethod, DiagnosticV1> {
    match value {
        "circle" => Ok(CellMethod::Circle),
        "square" => Ok(CellMethod::Square),
        "ellipse" => Ok(CellMethod::Ellipse),
        "number" => Ok(CellMethod::Number),
        "shade" => Ok(CellMethod::Shade),
        "color" => Ok(CellMethod::Color),
        "pie" => Ok(CellMethod::Pie),
        other => Err(diagnostic(
            "configuration.invalid_field",
            "configuration",
            format!("unsupported cell method '{other}'"),
            Some("cellMethod"),
        )),
    }
}

/// End-to-end matrix construction through the shared Rust core.
#[wasm_bindgen(js_name = "renderMatrixSvg")]
pub fn render_matrix_svg_wasm(scene_v1_json: String, method: String) -> Result<JsValue, JsValue> {
    let method = match cell_method(&method) {
        Ok(method) => method,
        Err(error) => return to_js(&SvgResult { svg: None, diagnostic: Some(error) }),
    };
    let scene = match scene_v1_to_scene(&scene_v1_json, method) {
        Ok(scene) => scene,
        Err(error) => return to_js(&SvgResult { svg: None, diagnostic: Some(scene_build_diagnostic(error)) }),
    };
    let options = SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels);
    let result = match SceneSvgRenderer.render(&scene, &OutputOptions::Svg(options)) {
        Ok(RenderOutput::Svg(svg)) => SvgResult { svg: Some(svg), diagnostic: None },
        Ok(_) => SvgResult {
            svg: None,
            diagnostic: Some(diagnostic("renderer.output.format_mismatch", "export", "SVG renderer returned a non-SVG output", Some("output"))),
        },
        Err(error) => SvgResult {
            svg: None,
            diagnostic: Some(diagnostic(error.code, "renderer", error.message, Some("output"))),
        },
    };
    to_js(&result)
}


/// End-to-end matrix construction through the shared Rust core and RGBA renderer.
#[wasm_bindgen(js_name = "renderMatrixRgba")]
pub fn render_matrix_rgba_wasm(
    scene_v1_json: String,
    method: String,
    options_input: JsValue,
) -> Result<JsValue, JsValue> {
    let method = match cell_method(&method) {
        Ok(method) => method,
        Err(error) => return rgba_failure(error),
    };
    let scene = match scene_v1_to_scene(&scene_v1_json, method) {
        Ok(scene) => scene,
        Err(error) => return rgba_failure(scene_build_diagnostic(error)),
    };
    let options: RgbaOptionsInput = match from_js(options_input, "options") {
        Ok(options) => options,
        Err(error) => return rgba_failure(error),
    };
    let background = match rgba(options.background, "output.background") {
        Ok(background) => background,
        Err(error) => return rgba_failure(error),
    };
    let raster = RasterOptions {
        width_px: options.width_px,
        height_px: options.height_px,
        dpi: options.dpi,
        background,
    };

    match RgbaRenderer.render(&scene, &OutputOptions::Rgba(raster)) {
        Ok(RenderOutput::Rgba(buffer)) => {
            let stride_bytes = match u32::try_from(buffer.stride_bytes) {
                Ok(stride) => stride,
                Err(_) => return rgba_failure(diagnostic(
                    "renderer.output.invalid_buffer",
                    "export",
                    "RGBA stride cannot be represented by the WebAssembly binding",
                    Some("output.strideBytes"),
                )),
            };
            // Return JS-owned bytes so the result remains valid after later WASM calls.
            let pixels = Uint8Array::from(buffer.pixels.as_slice());
            rgba_result(RgbaMetadata {
                width_px: Some(buffer.width_px),
                height_px: Some(buffer.height_px),
                stride_bytes: Some(stride_bytes),
                pixel_format: Some("rgba8".to_string()),
                alpha_mode: Some("straight".to_string()),
                color_space: Some("srgb".to_string()),
                dpi: buffer.dpi,
                diagnostic: None,
            }, Some(pixels))
        }
        Ok(_) => rgba_failure(diagnostic(
            "renderer.output.format_mismatch",
            "export",
            "RGBA renderer returned a non-RGBA output",
            Some("output"),
        )),
        Err(error) => rgba_failure(diagnostic(error.code, "renderer", error.message, Some("output"))),
    }
}


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl From<RgbaColor> for CanvasColor {
    fn from(color: RgbaColor) -> Self {
        Self { red: color.red, green: color.green, blue: color.blue, alpha: color.alpha }
    }
}

#[derive(Serialize)]
#[serde(tag = "op", rename_all = "camelCase")]
enum CanvasCommand {
    FillRect { x: f64, y: f64, width: f64, height: f64, color: CanvasColor },
    StrokeRect { x: f64, y: f64, width: f64, height: f64, line_width: f64, color: CanvasColor },
    Circle { cx: f64, cy: f64, radius: f64, color: CanvasColor },
    Square { cx: f64, cy: f64, side: f64, color: CanvasColor },
    Ellipse { cx: f64, cy: f64, rx: f64, ry: f64, rotation_degrees: f64, color: CanvasColor },
    Pie { cx: f64, cy: f64, radius: f64, start_radians: f64, end_radians: f64, color: CanvasColor },
    Line { x1: f64, y1: f64, x2: f64, y2: f64, line_width: f64, color: CanvasColor },
    Text {
        text: String,
        x: f64,
        y: f64,
        font_size: f64,
        color: CanvasColor,
        anchor: String,
        rotation_degrees: f64,
        font_family: String,
        font_weight: String,
        font_style: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasSceneCommands {
    width: f64,
    height: f64,
    description: String,
    commands: Vec<CanvasCommand>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CanvasCommandsResult {
    scene: Option<CanvasSceneCommands>,
    diagnostic: Option<DiagnosticV1>,
}

fn canvas_text(
    text: String,
    x: f64,
    y: f64,
    font_size: f64,
    color: RgbaColor,
    anchor: TextAnchor,
    rotation_degrees: f64,
    font_family: String,
    font_weight: String,
    font_style: String,
) -> CanvasCommand {
    CanvasCommand::Text {
        text,
        x,
        y,
        font_size,
        color: color.into(),
        anchor: match anchor {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        }.to_string(),
        rotation_degrees,
        font_family,
        font_weight,
        font_style,
    }
}

fn scene_canvas_commands(scene: &Scene) -> CanvasSceneCommands {
    let mut commands = Vec::new();
    for cell in &scene.cells {
        if !cell.visible { continue; }
        let x = cell.x;
        let y = cell.y;
        let size = cell.size;
        let cx = x + size / 2.0;
        let cy = y + size / 2.0;
        commands.push(CanvasCommand::FillRect {
            x, y, width: size, height: size, color: cell.fill.into(),
        });
        commands.push(CanvasCommand::StrokeRect {
            x, y, width: size, height: size, line_width: 1.0, color: cell.stroke.into(),
        });
        match &cell.glyph {
            Some(CellGlyph::Circle { radius, color }) => commands.push(CanvasCommand::Circle {
                cx, cy, radius: *radius, color: (*color).into(),
            }),
            Some(CellGlyph::Square { side, color }) => commands.push(CanvasCommand::Square {
                cx, cy, side: *side, color: (*color).into(),
            }),
            Some(CellGlyph::Ellipse { rx, ry, rotation_degrees, color }) => commands.push(CanvasCommand::Ellipse {
                cx, cy, rx: *rx, ry: *ry, rotation_degrees: *rotation_degrees, color: (*color).into(),
            }),
            Some(CellGlyph::Pie { radius, start_radians, end_radians, color }) => commands.push(CanvasCommand::Pie {
                cx, cy, radius: *radius, start_radians: *start_radians, end_radians: *end_radians, color: (*color).into(),
            }),
            Some(CellGlyph::Number { text, font_size, color }) => commands.push(canvas_text(
                text.clone(), cx, cy - font_size / 2.0, *font_size, *color, TextAnchor::Middle,
                0.0, "sans-serif".into(), "normal".into(), "normal".into(),
            )),
            Some(CellGlyph::Shade { color }) => commands.push(CanvasCommand::Line {
                x1: x, y1: y + size, x2: x + size, y2: y, line_width: 1.0, color: (*color).into(),
            }),
            Some(CellGlyph::Color) | None => {}
        }
    }

    for region in &scene.cluster_regions {
        commands.push(CanvasCommand::StrokeRect {
            x: region.x, y: region.y, width: region.width, height: region.height,
            line_width: region.line_width, color: region.color.into(),
        });
    }

    let black = RgbaColor { red: 0, green: 0, blue: 0, alpha: 255 };
    let mut legend_x = 8.0;
    let legend_y = scene.height - 13.0;
    for entry in &scene.legend {
        commands.push(CanvasCommand::FillRect {
            x: legend_x, y: legend_y + 1.0, width: 7.0, height: 7.0, color: entry.color.into(),
        });
        commands.push(canvas_text(
            entry.label.clone(), legend_x + 11.0, legend_y, 8.0, black, TextAnchor::Start,
            0.0, "sans-serif".into(), "normal".into(), "normal".into(),
        ));
        legend_x += 18.0 + entry.label.chars().count() as f64 * 8.0 * 6.0 / 7.0 + 10.0;
    }

    for text in &scene.texts {
        commands.push(canvas_text(
            text.text.clone(), text.x, text.y, text.font_size, text.color, text.anchor,
            text.rotation_degrees,
            if text.font_family.is_empty() { "sans-serif".into() } else { text.font_family.clone() },
            text.font_weight.clone(),
            text.font_style.clone(),
        ));
    }

    CanvasSceneCommands {
        width: scene.width,
        height: scene.height,
        description: scene.description.clone(),
        commands,
    }
}

/// Return renderer-neutral Canvas 2D drawing commands from the shared Rust scene.
/// Canvas and DOM objects remain owned by the browser adapter.
#[wasm_bindgen(js_name = "renderMatrixCanvasCommands")]
pub fn render_matrix_canvas_commands_wasm(scene_v1_json: String, method: String) -> Result<JsValue, JsValue> {
    let method = match cell_method(&method) {
        Ok(method) => method,
        Err(error) => return to_js(&CanvasCommandsResult { scene: None, diagnostic: Some(error) }),
    };
    let scene = match scene_v1_to_scene(&scene_v1_json, method) {
        Ok(scene) => scene,
        Err(error) => return to_js(&CanvasCommandsResult {
            scene: None,
            diagnostic: Some(scene_build_diagnostic(error)),
        }),
    };
    to_js(&CanvasCommandsResult {
        scene: Some(scene_canvas_commands(&scene)),
        diagnostic: None,
    })
}


#[cfg(test)]
mod canvas_commands_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn exports_scene_v1_canvas_commands_without_raster_pixels() {
        let fixture = include_str!("../../../prototypes/shared-svg/scene-v1.example.json");
        let output = render_matrix_canvas_commands_wasm(fixture.into(), "circle".into()).unwrap();
        let scene = Reflect::get(&output, &JsValue::from_str("scene")).unwrap();
        assert!(!scene.is_null());
        assert_eq!(Reflect::get(&scene, &JsValue::from_str("width")).unwrap().as_f64(), Some(380.0));
        let commands = Array::from(&Reflect::get(&scene, &JsValue::from_str("commands")).unwrap());
        assert!(commands.length() > 0);
        let operations: Vec<String> = commands.iter()
            .filter_map(|command| Reflect::get(&command, &JsValue::from_str("op")).ok()?.as_string())
            .collect();
        assert!(operations.iter().any(|op| op == "fillRect"));
        assert!(operations.iter().any(|op| op == "strokeRect"));
        assert!(operations.iter().any(|op| op == "circle"));
        assert!(operations.iter().any(|op| op == "text"));
        let diagnostic = Reflect::get(&output, &JsValue::from_str("diagnostic")).unwrap();
        assert!(diagnostic.is_undefined() || diagnostic.is_null());
    }

    #[wasm_bindgen_test]
    fn maps_invalid_canvas_method_to_diagnostic_v1() {
        let fixture = include_str!("../../../prototypes/shared-svg/scene-v1.example.json");
        let output = render_matrix_canvas_commands_wasm(fixture.into(), "unknown".into()).unwrap();
        let diagnostic = Reflect::get(&output, &JsValue::from_str("diagnostic")).unwrap();
        assert_eq!(
            Reflect::get(&diagnostic, &JsValue::from_str("code")).unwrap().as_string().as_deref(),
            Some("configuration.invalid_field"),
        );
        assert!(Reflect::get(&output, &JsValue::from_str("scene")).unwrap().is_null());
    }
}

#[cfg(test)]
mod matrix_rgba_tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn renders_scene_v1_matrix_to_owned_rgba_pixels() {
        let fixture = include_str!("../../../prototypes/shared-svg/scene-v1.example.json");
        let options = RgbaOptionsInput {
            width_px: 380,
            height_px: 392,
            dpi: Some(96.0),
            background: RgbaColorInput { red: 255, green: 255, blue: 255, alpha: 255 },
        };
        let output = render_matrix_rgba_wasm(
            fixture.into(),
            "circle".into(),
            serde_wasm_bindgen::to_value(&options).unwrap(),
        ).unwrap();
        let pixels = Reflect::get(&output, &JsValue::from_str("pixels")).unwrap()
            .dyn_into::<Uint8Array>().unwrap();
        assert_eq!(pixels.length(), 380 * 392 * 4);
        assert_eq!(Reflect::get(&output, &JsValue::from_str("widthPx")).unwrap().as_f64(), Some(380.0));
        assert_eq!(Reflect::get(&output, &JsValue::from_str("heightPx")).unwrap().as_f64(), Some(392.0));
        assert_eq!(Reflect::get(&output, &JsValue::from_str("strideBytes")).unwrap().as_f64(), Some(380.0 * 4.0));
        let diagnostic = Reflect::get(&output, &JsValue::from_str("diagnostic")).unwrap();
        assert!(diagnostic.is_undefined() || diagnostic.is_null());

        // The positive diagonal cell has a visible glyph at its center.
        let offset = (118 * 380 + 146) * 4;
        assert_eq!(pixels.get_index(offset as u32 + 3), 255);
        assert!(
            pixels.get_index(offset as u32) != 255 ||
            pixels.get_index(offset as u32 + 1) != 255 ||
            pixels.get_index(offset as u32 + 2) != 255
        );
    }

    #[wasm_bindgen_test]
    fn maps_invalid_matrix_rgba_options_to_diagnostic_v1() {
        let fixture = include_str!("../../../prototypes/shared-svg/scene-v1.example.json");
        let options = RgbaOptionsInput {
            width_px: 0,
            height_px: 392,
            dpi: None,
            background: RgbaColorInput { red: 255, green: 255, blue: 255, alpha: 255 },
        };
        let output = render_matrix_rgba_wasm(
            fixture.into(),
            "circle".into(),
            serde_wasm_bindgen::to_value(&options).unwrap(),
        ).unwrap();
        let diagnostic: DiagnosticV1 = serde_wasm_bindgen::from_value(
            Reflect::get(&output, &JsValue::from_str("diagnostic")).unwrap()
        ).unwrap();
        assert_eq!(diagnostic.schema_version, 1);
        assert_eq!(diagnostic.code, "renderer.output.invalid_options");
        assert_eq!(diagnostic.stage, "renderer");
        assert!(Reflect::get(&output, &JsValue::from_str("pixels")).unwrap().is_null());
    }
}
