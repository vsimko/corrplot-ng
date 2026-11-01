//! Shared normalized-matrix to renderer-neutral scene construction.
//!
//! This is the first plot-construction boundary shared by native and WebAssembly
//! consumers. Runtime adapters convert host values to these plain Rust types;
//! layout and glyph semantics remain in this module.

use crate::output::RgbaColor;
use crate::raster::{CellGlyph, Scene, SceneCell, SceneText, TextAnchor};
use crate::{validate_matrix, MatrixKind};
use std::collections::HashSet;

const CELL_SIZE: f64 = 64.0;
const CELL_GAP: f64 = 4.0;
const LEFT_MARGIN: f64 = 112.0;
const TOP_MARGIN: f64 = 92.0;
const BOTTOM_MARGIN: f64 = 24.0;
const RIGHT_MARGIN: f64 = 24.0;
const LABEL_FONT_SIZE: f64 = 12.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixSemantics { Correlation, GeneralNumeric }

impl MatrixSemantics {
    fn validation_kind(self) -> MatrixKind {
        match self { Self::Correlation => MatrixKind::Correlation, Self::GeneralNumeric => MatrixKind::GeneralNumeric }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedMatrixV1 {
    pub schema_version: u32,
    pub kind: String,
    pub labels: Vec<String>,
    /// Row-major matrix values. None is the only missing-value marker.
    pub values: Vec<Vec<Option<f64>>>,
    /// Displayed position to original source position.
    pub source_indices: Vec<usize>,
    pub semantics: MatrixSemantics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellMethod { Circle, Square, Ellipse, Number, Shade, Color, Pie }

#[derive(Debug, Clone, PartialEq)]
pub struct SceneBuildOptions {
    pub cell_method: CellMethod,
    pub title: String,
    pub description: String,
    pub annotation: Option<String>,
}

impl Default for SceneBuildOptions {
    fn default() -> Self {
        Self { cell_method: CellMethod::Circle, title: String::new(), description: String::new(), annotation: None }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneBuildError {
    pub code: &'static str,
    pub message: String,
    pub path: String,
    pub row: Option<usize>,
    pub column: Option<usize>,
}

impl SceneBuildError {
    fn new(code: &'static str, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self { code, message: message.into(), path: path.into(), row: None, column: None }
    }
}

impl std::fmt::Display for SceneBuildError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { formatter.write_str(&self.message) }
}
impl std::error::Error for SceneBuildError {}

/// Construct a deterministic logical scene from normalized matrix data.
///
/// This initial slice supports matrix semantics, source-index validation,
/// labels, missing values, seven cell encodings, title/description input and a
/// plain annotation. Cluster regions, plotmath, confidence intervals,
/// significance and publication-text layout remain follow-up work.
pub fn matrix_to_scene_v1(input: &NormalizedMatrixV1, options: &SceneBuildOptions) -> Result<Scene, SceneBuildError> {
    if input.schema_version != 1 || input.kind != "normalized_matrix" {
        return Err(SceneBuildError::new("input.invalid_schema_version", "expected normalized_matrix schema_version 1", "schema_version"));
    }
    let n = input.labels.len();
    if n == 0 || input.values.len() != n || input.values.iter().any(|row| row.len() != n) {
        return Err(SceneBuildError::new("input.matrix.invalid_shape", "normalized matrix values must be a non-empty square matrix matching labels", "values"));
    }
    if input.source_indices.len() != n {
        return Err(SceneBuildError::new("input.matrix.invalid_shape", "source_indices length must match labels length", "source_indices"));
    }
    let mut seen = HashSet::with_capacity(n);
    for (index, source_index) in input.source_indices.iter().copied().enumerate() {
        if !seen.insert(source_index) {
            return Err(SceneBuildError::new("input.matrix.invalid_source_indices", format!("source_indices contains duplicate index {source_index}"), format!("source_indices[{index}]")));
        }
    }
    let flat: Vec<Option<f64>> = input.values.iter().flatten().copied().collect();
    validate_matrix(&input.labels, n, n, &flat, input.semantics.validation_kind()).map_err(|error| SceneBuildError {
        code: error.code,
        message: error.message,
        path: error.row.zip(error.column).map(|(row, column)| format!("values[{row}][{column}]")).unwrap_or_else(|| "labels".to_string()),
        row: error.row,
        column: error.column,
    })?;

    let max_abs = flat.iter().flatten().copied().map(|value| value.abs()).fold(0.0_f64, f64::max);
    let pitch = CELL_SIZE + CELL_GAP;
    let width = LEFT_MARGIN + n as f64 * pitch + RIGHT_MARGIN;
    let height = TOP_MARGIN + n as f64 * pitch + BOTTOM_MARGIN;
    let positive = RgbaColor { red: 33, green: 102, blue: 172, alpha: 255 };
    let negative = RgbaColor { red: 178, green: 24, blue: 43, alpha: 255 };
    let neutral = RgbaColor { red: 189, green: 189, blue: 189, alpha: 255 };
    let missing_fill = RgbaColor { red: 242, green: 242, blue: 242, alpha: 255 };
    let grid = RgbaColor { red: 224, green: 224, blue: 224, alpha: 255 };
    let black = RgbaColor { red: 35, green: 35, blue: 35, alpha: 255 };
    let white = RgbaColor::WHITE;
    let mut cells = Vec::with_capacity(n * n);
    let mut texts = Vec::with_capacity(n * 2 + 2);

    for (row, values) in input.values.iter().enumerate() {
        for (column, value) in values.iter().copied().enumerate() {
            let x = LEFT_MARGIN + column as f64 * pitch;
            let y = TOP_MARGIN + row as f64 * pitch;
            let magnitude = match value {
                Some(value) => {
                    let denominator = match input.semantics { MatrixSemantics::Correlation => 1.0, MatrixSemantics::GeneralNumeric => max_abs };
                    if denominator == 0.0 { 0.0 } else { (value.abs() / denominator).clamp(0.0, 1.0) }
                }
                None => 0.0,
            };
            let color = match value { Some(value) if value > 0.0 => positive, Some(value) if value < 0.0 => negative, Some(_) => neutral, None => missing_fill };
            let glyph = value.map(|raw| match options.cell_method {
                CellMethod::Circle => CellGlyph::Circle { radius: magnitude * CELL_SIZE * 0.43, color },
                CellMethod::Square => CellGlyph::Square { side: magnitude * CELL_SIZE * 0.86, color },
                CellMethod::Ellipse => CellGlyph::Ellipse { rx: CELL_SIZE * 0.43, ry: (CELL_SIZE * 0.43 * (1.0 - magnitude)).max(CELL_SIZE * 0.015), color },
                CellMethod::Number => CellGlyph::Number { text: format!("{raw:.3}"), font_size: 12.0, color: black },
                CellMethod::Shade => CellGlyph::Shade { color },
                CellMethod::Color => CellGlyph::Color,
                CellMethod::Pie => {
                    let sign = if raw < 0.0 { -1.0 } else { 1.0 };
                    CellGlyph::Pie { radius: CELL_SIZE * 0.43, start_radians: -std::f64::consts::FRAC_PI_2, end_radians: -std::f64::consts::FRAC_PI_2 + sign * std::f64::consts::TAU * magnitude, color }
                }
            });
            cells.push(SceneCell {
                x, y, size: CELL_SIZE, visible: true,
                fill: match value { None => missing_fill, Some(_) if options.cell_method == CellMethod::Color => color, Some(_) => white },
                stroke: grid, glyph,
            });
            if value.is_none() {
                texts.push(SceneText { text: "NA".into(), x: x + CELL_SIZE / 2.0, y: y + CELL_SIZE / 2.0 - 5.0, font_size: 10.0, color: black, anchor: TextAnchor::Middle });
            }
        }
    }

    for (index, label) in input.labels.iter().enumerate() {
        let center = TOP_MARGIN + index as f64 * pitch + CELL_SIZE / 2.0;
        texts.push(SceneText { text: label.clone(), x: LEFT_MARGIN - 12.0, y: center - LABEL_FONT_SIZE / 2.0, font_size: LABEL_FONT_SIZE, color: black, anchor: TextAnchor::End });
        texts.push(SceneText { text: label.clone(), x: LEFT_MARGIN + index as f64 * pitch + CELL_SIZE / 2.0, y: TOP_MARGIN - LABEL_FONT_SIZE - 8.0, font_size: LABEL_FONT_SIZE, color: black, anchor: TextAnchor::Middle });
    }
    if !options.title.is_empty() {
        texts.push(SceneText { text: options.title.clone(), x: width / 2.0, y: 12.0, font_size: 18.0, color: black, anchor: TextAnchor::Middle });
    }
    if let Some(annotation) = &options.annotation {
        if !annotation.is_empty() {
            texts.push(SceneText { text: annotation.clone(), x: LEFT_MARGIN, y: height - BOTTOM_MARGIN + 2.0, font_size: 10.0, color: black, anchor: TextAnchor::Start });
        }
    }
    // The current renderer Scene type has no accessibility metadata fields.
    let _description = &options.description;
    let scene = Scene { width, height, cells, texts };
    scene.validate().map_err(|error| SceneBuildError::new(error.code, error.message, "scene"))?;
    Ok(scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> NormalizedMatrixV1 {
        NormalizedMatrixV1 {
            schema_version: 1, kind: "normalized_matrix".into(),
            labels: vec!["alpha".into(), "beta".into(), "gamma".into()],
            values: vec![vec![Some(1.0), Some(-0.5), None], vec![Some(-0.5), Some(1.0), None], vec![None, None, Some(1.0)]],
            source_indices: vec![0, 1, 2], semantics: MatrixSemantics::Correlation,
        }
    }

    #[test]
    fn builds_deterministic_scene_and_preserves_missing_values() {
        let scene = matrix_to_scene_v1(&input(), &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells.len(), 9);
        assert_eq!(scene.cells[0].glyph, Some(CellGlyph::Circle { radius: CELL_SIZE * 0.43, color: RgbaColor { red: 33, green: 102, blue: 172, alpha: 255 } }));
        assert!(scene.texts.iter().any(|text| text.text == "NA"));
        assert!(scene.texts.iter().any(|text| text.text == "alpha"));
        assert_eq!(scene.width, LEFT_MARGIN + 3.0 * (CELL_SIZE + CELL_GAP) + RIGHT_MARGIN);
    }

    #[test]
    fn supports_all_seven_declared_cell_methods() {
        for method in [CellMethod::Circle, CellMethod::Square, CellMethod::Ellipse, CellMethod::Number, CellMethod::Shade, CellMethod::Color, CellMethod::Pie] {
            let options = SceneBuildOptions { cell_method: method, ..Default::default() };
            let scene = matrix_to_scene_v1(&input(), &options).unwrap();
            assert_eq!(scene.cells.len(), 9);
            scene.validate().unwrap();
        }
    }

    #[test]
    fn general_numeric_values_use_a_data_derived_magnitude() {
        let mut matrix = input();
        matrix.semantics = MatrixSemantics::GeneralNumeric;
        matrix.labels = vec!["alpha".into(), "beta".into()];
        matrix.source_indices = vec![0, 1];
        matrix.values = vec![vec![Some(0.0), Some(5.0)], vec![Some(-10.0), Some(2.0)]];
        let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells[2].glyph, Some(CellGlyph::Circle { radius: CELL_SIZE * 0.43, color: RgbaColor { red: 178, green: 24, blue: 43, alpha: 255 } }));
    }

    #[test]
    fn built_scene_is_consumed_by_both_existing_renderers() {
        use crate::output::{OutputOptions, RasterOptions, RenderOutput, Renderer, SvgOptions, SvgUnit};
        use crate::raster::{RgbaRenderer, SceneSvgRenderer};
        let scene = matrix_to_scene_v1(&input(), &SceneBuildOptions::default()).unwrap();
        let svg_options = OutputOptions::Svg(SvgOptions::new(scene.width, scene.height, SvgUnit::Pixels));
        let svg = SceneSvgRenderer.render(&scene, &svg_options).unwrap();
        assert!(matches!(svg, RenderOutput::Svg(_)));
        let rgba_options = OutputOptions::Rgba(RasterOptions::new(320, 320));
        let rgba = RgbaRenderer.render(&scene, &rgba_options).unwrap();
        assert!(matches!(rgba, RenderOutput::Rgba(_)));
    }

    #[test]
    fn rejects_duplicate_source_indices_with_stable_path() {
        let mut matrix = input();
        matrix.source_indices = vec![0, 0, 2];
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.code, "input.matrix.invalid_source_indices");
        assert_eq!(error.path, "source_indices[1]");
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut matrix = input();
        matrix.schema_version = 2;
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.path, "schema_version");
    }
}
