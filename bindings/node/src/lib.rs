use napi::bindgen_prelude::Buffer;
use corrplot_core::output::{OutputOptions, RasterOptions, RenderOutput, Renderer, RgbaColor, SvgOptions, SvgUnit};
use corrplot_core::raster::{CellGlyph, RgbaRenderer, Scene, SceneCell, SceneCellSignificance, SceneConfidenceInterval, SceneSignificancePolicy, SceneSvgRenderer, SceneText, TextAnchor};
use corrplot_core::{validate_matrix, MatrixKind, RENDERER_CAPABILITIES_V1};
use corrplot_core::scene::{matrix_to_scene_v1, scene_v1_to_scene, CellMethod, DisplayType, MatrixSemantics, NormalizedMatrixV1, SceneBuildOptions};
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
pub struct RgbaOptionsInput {
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: Option<f64>,
    pub background: RgbaColorInput,
}

#[napi(object)]
pub struct RgbaResult {
    pub width_px: Option<u32>,
    pub height_px: Option<u32>,
    pub stride_bytes: Option<u32>,
    pub pixel_format: Option<String>,
    pub alpha_mode: Option<String>,
    pub color_space: Option<String>,
    pub dpi: Option<f64>,
    pub pixels: Option<Buffer>,
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
    pub rotation_degrees: Option<f64>,
    pub start_radians: Option<f64>,
    pub end_radians: Option<f64>,
    pub text: Option<String>,
    pub font_size: Option<f64>,
    pub color: Option<RgbaColorInput>,
}

#[napi(object)]
pub struct SceneCellSignificanceInput {
    pub p_value: Option<f64>,
    pub status: String,
    pub annotation: Option<String>,
    pub blank: bool,
}

#[napi(object)]
pub struct SceneConfidenceIntervalInput {
    pub lower: f64,
    pub upper: f64,
    pub x1: f64,
    pub x2: f64,
    pub y: f64,
    pub cap_height: f64,
    pub stroke_width: f64,
    pub color: RgbaColorInput,
}

#[napi(object)]
pub struct SceneSignificancePolicyInput {
    pub mode: String,
    pub thresholds: Vec<f64>,
    pub symbol: String,
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
    pub p_value: Option<f64>,
    pub normalized_value: Option<f64>,
    pub significance: Option<SceneCellSignificanceInput>,
    pub confidence_interval: Option<SceneConfidenceIntervalInput>,
}

#[napi(object)]
pub struct SceneTextInput {
    pub text: String,
    pub semantic_role: Option<String>,
    pub x: f64,
    pub y: f64,
    pub font_size: f64,
    pub color: RgbaColorInput,
    pub anchor: String,
    pub rotation_degrees: Option<f64>,
}

#[napi(object)]
pub struct SceneInput {
    pub width: f64,
    pub height: f64,
    pub description: Option<String>,
    pub cells: Vec<SceneCellInput>,
    pub texts: Vec<SceneTextInput>,
    pub significance: Option<SceneSignificancePolicyInput>,
}

#[napi(object)]
pub struct SceneBuildOptionsInput {
    pub cell_method: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub annotation: Option<String>,
    pub display_type: Option<String>,
    pub show_diagonal: Option<bool>,
}

impl Default for SceneBuildOptionsInput {
    fn default() -> Self {
        Self {
            cell_method: None,
            title: None,
            description: None,
            annotation: None,
            display_type: None,
            show_diagonal: None,
        }
    }
}

#[napi(object)]
pub struct SceneResult {
    pub scene: Option<SceneInput>,
    pub diagnostic: Option<DiagnosticV1>,
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
                    "configuration.invalid_field",
                    "configuration",
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
            rotation_degrees: input.rotation_degrees.unwrap_or(0.0),
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
            p_value: cell.p_value,
            normalized_value: cell.normalized_value,
            significance: cell.significance.map(|value| SceneCellSignificance {
                p_value: value.p_value,
                status: value.status,
                annotation: value.annotation,
                blank: value.blank,
            }),
            confidence_interval: cell.confidence_interval.map(|value| {
                Ok::<_, DiagnosticV1>(SceneConfidenceInterval {
                    lower: value.lower,
                    upper: value.upper,
                    x1: value.x1,
                    x2: value.x2,
                    y: value.y,
                    cap_height: value.cap_height,
                    stroke_width: value.stroke_width,
                    color: rgba(value.color, &format!("cells[{index}].confidenceInterval.color"))?,
                })
            }).transpose()?,
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
            semantic_role: text.semantic_role,
            x: text.x,
            y: text.y,
            font_size: text.font_size,
            color: rgba(text.color, &format!("texts[{index}].color"))?,
            anchor,
            rotation_degrees: text.rotation_degrees.unwrap_or(0.0),
            font_family: String::new(),
            font_weight: "normal".into(),
            font_style: "normal".into(),
        });
    }

    let significance = input.significance.map(|value| SceneSignificancePolicy {
        mode: value.mode,
        thresholds: value.thresholds,
        symbol: value.symbol,
    });
    Ok(Scene { width: input.width, height: input.height, description: input.description.unwrap_or_default(), cells, texts, significance, legend: Vec::new(), cluster_regions: Vec::new() })
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


fn rgba_error(diagnostic: DiagnosticV1) -> RgbaResult {
    RgbaResult {
        width_px: None,
        height_px: None,
        stride_bytes: None,
        pixel_format: None,
        alpha_mode: None,
        color_space: None,
        dpi: None,
        pixels: None,
        diagnostic: Some(diagnostic),
    }
}

fn render_rgba_scene(scene: &Scene, options: RgbaOptionsInput) -> RgbaResult {
    let background = match rgba(options.background, "output.background") {
        Ok(background) => background,
        Err(diagnostic) => return rgba_error(diagnostic),
    };
    let raster = RasterOptions {
        width_px: options.width_px,
        height_px: options.height_px,
        dpi: options.dpi,
        background,
    };

    match RgbaRenderer.render(scene, &OutputOptions::Rgba(raster)) {
        Ok(RenderOutput::Rgba(buffer)) => {
            let stride_bytes = match u32::try_from(buffer.stride_bytes) {
                Ok(stride) => stride,
                Err(_) => return rgba_error(diagnostic(
                    "renderer.output.invalid_buffer",
                    "export",
                    "RGBA stride cannot be represented by the Node.js binding",
                    Some("output.stride_bytes"),
                )),
            };
            RgbaResult {
                width_px: Some(buffer.width_px),
                height_px: Some(buffer.height_px),
                stride_bytes: Some(stride_bytes),
                pixel_format: Some("rgba8".to_string()),
                alpha_mode: Some("straight".to_string()),
                color_space: Some("srgb".to_string()),
                dpi: buffer.dpi,
                // Move the Rust-owned allocation into a Node Buffer. No second
                // pixel-vector copy is made at this ownership boundary.
                pixels: Some(Buffer::from(buffer.pixels)),
                diagnostic: None,
            }
        }
        Ok(_) => rgba_error(diagnostic(
            "renderer.output.format_mismatch",
            "export",
            "RGBA renderer returned a non-RGBA output",
            Some("output"),
        )),
        Err(error) => rgba_error(diagnostic(error.code, "renderer", error.message, Some("output"))),
    }
}

#[napi(js_name = "renderSceneRgba")]
pub fn render_scene_rgba_native(input: SceneInput, options: RgbaOptionsInput) -> RgbaResult {
    let scene = match scene(input) {
        Ok(scene) => scene,
        Err(diagnostic) => return rgba_error(diagnostic),
    };
    render_rgba_scene(&scene, options)
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

fn rgba_input(color: &RgbaColor) -> RgbaColorInput {
    RgbaColorInput {
        red: color.red as u32,
        green: color.green as u32,
        blue: color.blue as u32,
        alpha: color.alpha as u32,
    }
}

fn glyph_input(glyph: CellGlyph) -> CellGlyphInput {
    let mut input = CellGlyphInput {
        kind: String::new(),
        radius: None,
        side: None,
        rx: None,
        ry: None,
        rotation_degrees: None,
        start_radians: None,
        end_radians: None,
        text: None,
        font_size: None,
        color: None,
    };
    match glyph {
        CellGlyph::Circle { radius, color } => {
            input.kind = "circle".into();
            input.radius = Some(radius);
            input.color = Some(rgba_input(&color));
        }
        CellGlyph::Square { side, color } => {
            input.kind = "square".into();
            input.side = Some(side);
            input.color = Some(rgba_input(&color));
        }
        CellGlyph::Ellipse { rx, ry, rotation_degrees, color } => {
            input.kind = "ellipse".into();
            input.rx = Some(rx);
            input.ry = Some(ry);
            input.rotation_degrees = Some(rotation_degrees);
            input.color = Some(rgba_input(&color));
        }
        CellGlyph::Pie { radius, start_radians, end_radians, color } => {
            input.kind = "pie".into();
            input.radius = Some(radius);
            input.start_radians = Some(start_radians);
            input.end_radians = Some(end_radians);
            input.color = Some(rgba_input(&color));
        }
        CellGlyph::Number { text, font_size, color } => {
            input.kind = "number".into();
            input.text = Some(text);
            input.font_size = Some(font_size);
            input.color = Some(rgba_input(&color));
        }
        CellGlyph::Shade { color } => {
            input.kind = "shade".into();
            input.color = Some(rgba_input(&color));
        }
        CellGlyph::Color => input.kind = "color".into(),
    }
    input
}

fn scene_input(scene: Scene) -> SceneInput {
    let cells = scene.cells.into_iter().map(|cell| SceneCellInput {
        x: cell.x,
        y: cell.y,
        size: cell.size,
        visible: cell.visible,
        fill: rgba_input(&cell.fill),
        stroke: rgba_input(&cell.stroke),
        glyph: cell.glyph.map(glyph_input),
        p_value: cell.p_value,
        normalized_value: cell.normalized_value,
        significance: cell.significance.map(|value| SceneCellSignificanceInput {
            p_value: value.p_value,
            status: value.status,
            annotation: value.annotation,
            blank: value.blank,
        }),
        confidence_interval: cell.confidence_interval.map(|value| SceneConfidenceIntervalInput {
            lower: value.lower,
            upper: value.upper,
            x1: value.x1,
            x2: value.x2,
            y: value.y,
            cap_height: value.cap_height,
            stroke_width: value.stroke_width,
            color: rgba_input(&value.color),
        }),
    }).collect();
    let texts = scene.texts.into_iter().map(|text| SceneTextInput {
        text: text.text,
        semantic_role: text.semantic_role,
        x: text.x,
        y: text.y,
        font_size: text.font_size,
        color: rgba_input(&text.color),
        anchor: match text.anchor {
            TextAnchor::Start => "start",
            TextAnchor::Middle => "middle",
            TextAnchor::End => "end",
        }.into(),
        rotation_degrees: Some(text.rotation_degrees),
    }).collect();
    SceneInput {
        width: scene.width,
        height: scene.height,
        description: Some(scene.description),
        cells,
        texts,
        significance: scene.significance.map(|value| SceneSignificancePolicyInput {
            mode: value.mode,
            thresholds: value.thresholds,
            symbol: value.symbol,
        }),
    }
}

fn build_matrix_scene(input: MatrixInput, options: SceneBuildOptionsInput) -> Result<Scene, DiagnosticV1> {
    let semantics = match input.kind.as_str() {
        "correlation" => MatrixSemantics::Correlation,
        "general_numeric" => MatrixSemantics::GeneralNumeric,
        other => return Err(diagnostic(
            "configuration.invalid_field",
            "configuration",
            format!("unsupported matrix kind '{other}'"),
            Some("kind"),
        )),
    };
    let rows = input.rows as usize;
    let columns = input.columns as usize;
    if rows == 0 || rows != columns {
        return Err(diagnostic(
            "input.matrix.invalid_shape",
            "input",
            "scene construction requires a non-empty square matrix",
            Some("matrix"),
        ));
    }
    let expected = rows.checked_mul(columns).ok_or_else(|| diagnostic(
        "input.matrix.invalid_shape",
        "input",
        "matrix dimensions overflow",
        Some("matrix"),
    ))?;
    if input.values.len() != expected {
        return Err(diagnostic(
            "input.matrix.invalid_shape",
            "input",
            format!("matrix has {} values; expected {expected}", input.values.len()),
            Some("values"),
        ));
    }
    let values = input.values.chunks(columns).map(|row| row.to_vec()).collect();
    let display_type = match options.display_type.as_deref().unwrap_or("full") {
        "full" => DisplayType::Full,
        "upper" => DisplayType::Upper,
        "lower" => DisplayType::Lower,
        "diagonal" => DisplayType::Diagonal,
        other => return Err(diagnostic(
            "configuration.invalid_field",
            "configuration",
            format!("unsupported display type '{other}'"),
            Some("displayType"),
        )),
    };
    let method = cell_method(options.cell_method.as_deref().unwrap_or("circle"))?;
    let normalized = NormalizedMatrixV1 {
        schema_version: 1,
        kind: "normalized_matrix".into(),
        labels: input.labels,
        values,
        source_indices: (0..rows).collect(),
        semantics,
        display_type,
        show_diagonal: options.show_diagonal.unwrap_or(true),
    };
    matrix_to_scene_v1(&normalized, &SceneBuildOptions {
        cell_method: method,
        title: options.title.unwrap_or_default(),
        description: options.description.unwrap_or_default(),
        annotation: options.annotation.filter(|value| !value.is_empty()),
    }).map_err(scene_build_diagnostic)
}

#[napi(js_name = "createScene")]
pub fn create_scene_native(input: MatrixInput, options: Option<SceneBuildOptionsInput>) -> SceneResult {
    match build_matrix_scene(input, options.unwrap_or_default()) {
        Ok(scene) => SceneResult { scene: Some(scene_input(scene)), diagnostic: None },
        Err(diagnostic) => SceneResult { scene: None, diagnostic: Some(diagnostic) },
    }
}

/// End-to-end matrix construction through the shared Rust core.
#[napi(js_name = "renderMatrixSvg")]
pub fn render_matrix_svg_native(scene_v1_json: String, method: String) -> SvgResult {
    let method = match cell_method(&method) {
        Ok(method) => method,
        Err(error) => return SvgResult { svg: None, diagnostic: Some(error) },
    };
    let scene = match scene_v1_to_scene(&scene_v1_json, method) {
        Ok(scene) => scene,
        Err(error) => return SvgResult { svg: None, diagnostic: Some(scene_build_diagnostic(error)) },
    };
    let options = SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels);
    match SceneSvgRenderer.render(&scene, &OutputOptions::Svg(options)) {
        Ok(RenderOutput::Svg(svg)) => SvgResult { svg: Some(svg), diagnostic: None },
        Ok(_) => SvgResult {
            svg: None,
            diagnostic: Some(diagnostic("renderer.output.format_mismatch", "export", "SVG renderer returned a non-SVG output", Some("output"))),
        },
        Err(error) => SvgResult {
            svg: None,
            diagnostic: Some(diagnostic(error.code, "renderer", error.message, Some("output"))),
        },
    }
}

/// End-to-end matrix construction through the shared Rust core and RGBA renderer.
#[napi(js_name = "renderMatrixRgba")]
pub fn render_matrix_rgba_native(
    scene_v1_json: String,
    method: String,
    options: RgbaOptionsInput,
) -> RgbaResult {
    let method = match cell_method(&method) {
        Ok(method) => method,
        Err(error) => return rgba_error(error),
    };
    let scene = match scene_v1_to_scene(&scene_v1_json, method) {
        Ok(scene) => scene,
        Err(error) => return rgba_error(scene_build_diagnostic(error)),
    };
    render_rgba_scene(&scene, options)
}

