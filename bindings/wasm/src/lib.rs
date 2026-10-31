use corrplot_core::output::{OutputOptions, RenderOutput, Renderer, RgbaColor, SvgOptions, SvgUnit};
use corrplot_core::raster::{CellGlyph, Scene, SceneCell, SceneSvgRenderer, SceneText, TextAnchor};
use corrplot_core::{validate_matrix, MatrixKind, RENDERER_CAPABILITIES_V1};
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

#[derive(Deserialize)]
pub struct MatrixInput {
    pub labels: Vec<String>,
    pub rows: u32,
    pub columns: u32,
    pub values: Vec<Option<f64>>,
    pub kind: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RgbaColorInput {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellGlyphInput {
    pub kind: String,
    pub radius: Option<f64>,
    pub side: Option<f64>,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
    pub start_radians: Option<f64>,
    pub end_radians: Option<f64>,
    pub text: Option<String>,
    pub font_size: Option<f64>,
    pub color: Option<RgbaColorInput>,
}

#[derive(Deserialize)]
pub struct SceneCellInput {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub visible: bool,
    pub fill: RgbaColorInput,
    pub stroke: RgbaColorInput,
    pub glyph: Option<CellGlyphInput>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SceneTextInput {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub color: RgbaColorInput,
    pub anchor: String,
}

#[derive(Deserialize)]
pub struct SceneInput {
    pub width: f64,
    pub height: f64,
    pub cells: Vec<SceneCellInput>,
    pub texts: Vec<SceneTextInput>,
}

#[derive(Deserialize)]
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

#[wasm_bindgen(js_name = validateMatrix)]
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
            x: text.x,
            y: text.y,
            font_size: text.font_size,
            color: rgba(text.color, &format!("texts[{index}].color"))?,
            anchor,
        });
    }

    Ok(Scene { width: input.width, height: input.height, cells, texts })
}

#[wasm_bindgen(js_name = renderSceneSvg)]
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
    svg.font_family = options.font_family;
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

#[wasm_bindgen(js_name = getRendererCapabilities)]
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
            font_family: "sans-serif".into(),
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
}
