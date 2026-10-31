use corrplot_core::output::{OutputOptions, RenderOutput, Renderer, RgbaColor, SvgOptions, SvgUnit};
use corrplot_core::raster::{CellGlyph, Scene, SceneCell, SceneSvgRenderer, SceneText, TextAnchor};
use corrplot_core::{validate_matrix, MatrixKind, RENDERER_CAPABILITIES_V1};
use napi_derive::napi;

#[napi(object)]
pub struct DiagnosticV1 {
    pub schema_version: u32,
    pub code: String,
    pub severity: String,
    pub stage: String,
    pub message: String,
    pub path: Option<String>,
    pub row: Option<u32>,
    pub column: Option<u32>,
}

#[napi(object)]
pub struct ValidationResult {
    pub valid: bool,
    pub diagnostic: Option<DiagnosticV1>,
}

#[napi(object)]
pub struct SvgResult {
    pub svg: Option<String>,
    pub diagnostic: Option<DiagnosticV1>,
}

#[napi(object)]
pub struct MatrixInput {
    pub labels: Vec<String>,
    pub rows: u32,
    pub columns: u32,
    pub values: Vec<Option<f64>>,
    pub kind: String,
}

#[napi(object)]
pub struct RgbaColorInput {
    pub red: u32,
    pub green: u32,
    pub blue: u32,
    pub alpha: u32,
}

#[napi(object)]
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

#[napi(object)]
pub struct SceneCellInput {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub visible: bool,
    pub fill: RgbaColorInput,
    pub stroke: RgbaColorInput,
    pub glyph: Option<CellGlyphInput>,
}

#[napi(object)]
pub struct SceneTextInput {
    pub text: String,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub color: RgbaColorInput,
    pub anchor: String,
}

#[napi(object)]
pub struct SceneInput {
    pub width: f64,
    pub height: f64,
    pub cells: Vec<SceneCellInput>,
    pub texts: Vec<SceneTextInput>,
}

#[napi(object)]
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

#[napi(js_name = "validateMatrix")]
pub fn validate_matrix_native(input: MatrixInput) -> ValidationResult {
    let kind = match input.kind.as_str() {
        "correlation" => MatrixKind::Correlation,
        "general_numeric" => MatrixKind::GeneralNumeric,
        other => {
            return ValidationResult {
                valid: false,
                diagnostic: Some(diagnostic(
                    "input.matrix.invalid_kind",
                    "input",
                    format!("unsupported matrix kind '{other}'"),
                    Some("kind"),
                )),
            };
        }
    };

    match validate_matrix(
        &input.labels,
        input.rows as usize,
        input.columns as usize,
        &input.values,
        kind,
    ) {
        Ok(()) => ValidationResult { valid: true, diagnostic: None },
        Err(error) => ValidationResult { valid: false, diagnostic: Some(matrix_diagnostic(error)) },
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
            start_radians: required(input.start_radians, "start_radians")?,
            end_radians: required(input.end_radians, "end_radians")?,
            color: rgba(required(input.color, "color")?, "cells.glyph.color")?,
        }),
        "number" => Ok(CellGlyph::Number {
            text: required(input.text, "text")?,
            font_size: required(input.font_size, "font_size")?,
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

#[napi(js_name = "renderSceneSvg")]
pub fn render_scene_svg_native(input: SceneInput, options: SvgOptionsInput) -> SvgResult {
    let scene = match scene(input) {
        Ok(scene) => scene,
        Err(diagnostic) => return SvgResult { svg: None, diagnostic: Some(diagnostic) },
    };

    let unit = match options.unit.as_str() {
        "px" => SvgUnit::Pixels,
        "mm" => SvgUnit::Millimeters,
        "in" => SvgUnit::Inches,
        other => return SvgResult {
            svg: None,
            diagnostic: Some(diagnostic(
                "renderer.output.invalid_options",
                "renderer",
                format!("unsupported SVG unit '{other}'"),
                Some("output.unit"),
            )),
        },
    };
    let background = match rgba(options.background, "output.background") {
        Ok(color) => color,
        Err(diagnostic) => return SvgResult { svg: None, diagnostic: Some(diagnostic) },
    };
    let mut svg = SvgOptions::new(options.width, options.height, unit);
    svg.font_family = options.font_family;
    svg.font_size_pt = options.font_size_pt;
    svg.background = background;

    match SceneSvgRenderer.render(&scene, &OutputOptions::Svg(svg)) {
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
    }
}

#[napi(js_name = "getRendererCapabilities")]
pub fn get_renderer_capabilities_native() -> String {
    RENDERER_CAPABILITIES_V1.to_string()
}
