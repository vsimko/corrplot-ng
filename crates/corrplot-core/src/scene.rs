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

/// Parse the matrix-bearing portion of the established renderer-neutral scene-v1 JSON contract.
///
/// The returned value is normalized matrix input, not a renderer scene. Geometry
/// and presentation metadata are validated where required by the v1 schema and
/// deliberately remain owned by the scene consumer.
pub fn parse_scene_v1_matrix(source: &str) -> Result<NormalizedMatrixV1, SceneBuildError> {
    use crate::json::Value;
    let document = crate::json::parse(source).map_err(|error| SceneBuildError::new(
        "input.scene.invalid_json",
        format!("invalid JSON at byte {}: {}", error.offset, error.message),
        "$",
    ))?;
    let root = as_object(&document, "$")?;
    reject_unknown_fields(root, &["schema_version", "kind", "width", "height", "labels", "cells", "annotation", "style", "typography", "legend", "title", "description", "cluster_regions", "figure_text", "significance", "glyph_method", "text_annotations", "display_type", "show_diagonal", "variable_order", "matrix_semantics"], "$")?;
    let schema_version = as_usize(required(root, "schema_version", "$")?, "schema_version")?;
    if schema_version != 1 {
        return Err(SceneBuildError::new("input.scene.unsupported_version", "only scene schema_version 1 is supported", "schema_version"));
    }
    let kind = as_string(required(root, "kind", "$")?, "kind")?;
    let semantics = match kind {
        "correlation_plot" => MatrixSemantics::Correlation,
        "general_matrix_plot" => MatrixSemantics::GeneralNumeric,
        _ => return Err(SceneBuildError::new("input.scene.invalid_kind", "kind must be correlation_plot or general_matrix_plot", "kind")),
    };
    if let Some(matrix_semantics) = root.get("matrix_semantics") {
        validate_scene_matrix_semantics(matrix_semantics, semantics)?;
    } else if semantics == MatrixSemantics::GeneralNumeric {
        return Err(SceneBuildError::new("input.scene.missing_field", "general_matrix_plot requires matrix_semantics", "matrix_semantics"));
    }

    let width = as_number(required(root, "width", "$")?, "width")?;
    let height = as_number(required(root, "height", "$")?, "height")?;
    if width <= 0.0 || height <= 0.0 {
        return Err(SceneBuildError::new("input.scene.invalid_geometry", "scene width and height must be positive", "width"));
    }
    for field in ["title", "description", "annotation"] {
        as_string(required(root, field, "$")?, field)?;
    }
    let style = as_object(required(root, "style", "$")?, "style")?;
    reject_unknown_fields(style, &["positive_color", "negative_color", "zero_color", "missing_fill", "grid_color"], "style")?;
    for field in ["positive_color", "negative_color", "zero_color", "missing_fill", "grid_color"] {
        let path = format!("style.{field}");
        let color = as_string(required(style, field, "style")?, &path)?;
        if !is_srgb_hex(color) {
            return Err(SceneBuildError::new("input.scene.invalid_color", "expected a six-digit sRGB hex color", path));
        }
    }

    let label_values = as_array(required(root, "labels", "$")?, "labels")?;
    if label_values.is_empty() {
        return Err(SceneBuildError::new("input.matrix.invalid_shape", "scene labels must not be empty", "labels"));
    }
    let mut labels = Vec::with_capacity(label_values.len());
    for (index, value) in label_values.iter().enumerate() {
        let path = format!("labels[{index}]");
        let label = as_object(value, &path)?;
        reject_unknown_fields(label, &["text", "row_x", "row_y", "column_x", "column_y", "row_visible", "column_visible", "row_position", "column_position", "column_rotation", "offset", "plotmath"], &path)?;
        labels.push(as_string(required(label, "text", &path)?, &format!("{path}.text"))?.to_string());
        for field in ["row_x", "row_y", "column_x", "column_y"] {
            as_number(required(label, field, &path)?, &format!("{path}.{field}"))?;
        }
    }
    let n = labels.len();
    let cells = as_array(required(root, "cells", "$")?, "cells")?;
    let expected_cells = n.checked_mul(n).ok_or_else(|| SceneBuildError::new("input.matrix.invalid_shape", "matrix dimensions overflow", "cells"))?;
    if cells.len() != expected_cells {
        return Err(SceneBuildError::new("input.matrix.invalid_shape", format!("scene has {} cells; expected {expected_cells}", cells.len()), "cells"));
    }
    let mut values = vec![vec![None; n]; n];
    let mut seen = vec![false; expected_cells];
    for (index, value) in cells.iter().enumerate() {
        let path = format!("cells[{index}]");
        let cell = as_object(value, &path)?;
        reject_unknown_fields(cell, &["row", "column", "value", "x", "y", "center_x", "center_y", "size", "color", "visible", "encoding", "method", "region", "p_value", "significance", "confidence_interval", "normalized_value"], &path)?;
        let row = as_usize(required(cell, "row", &path)?, &format!("{path}.row"))?;
        let column = as_usize(required(cell, "column", &path)?, &format!("{path}.column"))?;
        if row >= n || column >= n {
            return Err(SceneBuildError::new("input.scene.cell_out_of_bounds", "cell row and column must refer to an existing label", format!("{path}.row")).with_cell(row, column));
        }
        let identity = row * n + column;
        if seen[identity] {
            return Err(SceneBuildError::new("input.scene.duplicate_cell", "duplicate row/column cell identity", format!("{path}.row")).with_cell(row, column));
        }
        seen[identity] = true;
        let cell_value = required(cell, "value", &path)?;
        values[row][column] = match cell_value {
            Value::Null => None,
            Value::Number(number) => Some(*number),
            _ => return Err(SceneBuildError::new("input.scene.invalid_cell_value", "cell value must be a finite number or null", format!("{path}.value")).with_cell(row, column)),
        };
        for field in ["x", "y", "center_x", "center_y"] {
            as_number(required(cell, field, &path)?, &format!("{path}.{field}"))?;
        }
        if as_number(required(cell, "size", &path)?, &format!("{path}.size"))? <= 0.0 {
            return Err(SceneBuildError::new("input.scene.invalid_geometry", "cell size must be positive", format!("{path}.size")).with_cell(row, column));
        }
    }
    if let Some(index) = seen.iter().position(|present| !present) {
        return Err(SceneBuildError::new("input.scene.missing_cell", "scene must contain every row/column cell identity", format!("cells[{index}]")));
    }

    let source_indices = if let Some(order) = root.get("variable_order") {
        let order = as_object(order, "variable_order")?;
        reject_unknown_fields(order, &["strategy", "indices", "labels"], "variable_order")?;
        as_string(required(order, "strategy", "variable_order")?, "variable_order.strategy")?;
        let ordered_labels = as_array(required(order, "labels", "variable_order")?, "variable_order.labels")?;
        if ordered_labels.len() != n {
            return Err(SceneBuildError::new("input.matrix.invalid_source_indices", "variable_order.labels length must match labels", "variable_order.labels"));
        }
        for (index, label) in ordered_labels.iter().enumerate() {
            as_string(label, &format!("variable_order.labels[{index}]"))?;
        }
        let indices = as_array(required(order, "indices", "variable_order")?, "variable_order.indices")?;
        if indices.len() != n {
            return Err(SceneBuildError::new("input.matrix.invalid_source_indices", "variable_order.indices length must match labels", "variable_order.indices"));
        }
        let mut result = Vec::with_capacity(n);
        let mut unique = HashSet::with_capacity(n);
        for (index, value) in indices.iter().enumerate() {
            let source_index = as_usize(value, &format!("variable_order.indices[{index}]"))?;
            if !unique.insert(source_index) {
                return Err(SceneBuildError::new("input.matrix.invalid_source_indices", "variable_order.indices must be unique", format!("variable_order.indices[{index}]")));
            }
            result.push(source_index);
        }
        result
    } else {
        (0..n).collect()
    };

    let flat: Vec<Option<f64>> = values.iter().flatten().copied().collect();
    validate_matrix(&labels, n, n, &flat, semantics.validation_kind()).map_err(|error| SceneBuildError {
        code: error.code,
        message: error.message,
        path: error.row.zip(error.column).map(|(row, column)| format!("cells[row={row},column={column}].value")).unwrap_or_else(|| "labels".to_string()),
        row: error.row,
        column: error.column,
    })?;

    Ok(NormalizedMatrixV1 {
        schema_version: 1,
        kind: "normalized_matrix".into(),
        labels,
        values,
        source_indices,
        semantics,
    })
}

impl SceneBuildError {
    fn with_cell(mut self, row: usize, column: usize) -> Self {
        self.row = Some(row);
        self.column = Some(column);
        self
    }
}

fn required<'a>(object: &'a std::collections::BTreeMap<String, crate::json::Value>, key: &str, parent: &str) -> Result<&'a crate::json::Value, SceneBuildError> {
    object.get(key).ok_or_else(|| SceneBuildError::new("input.scene.missing_field", format!("required field '{key}' is missing"), if parent == "$" { key.to_string() } else { format!("{parent}.{key}") }))
}

fn validate_scene_matrix_semantics(value: &crate::json::Value, expected: MatrixSemantics) -> Result<(), SceneBuildError> {
    let object = as_object(value, "matrix_semantics")?;
    reject_unknown_fields(object, &["mode", "color_limits", "color_mapping", "normalization", "palette", "has_observed_values"], "matrix_semantics")?;
    let mode = as_string(required(object, "mode", "matrix_semantics")?, "matrix_semantics.mode")?;
    let expected_mode = match expected {
        MatrixSemantics::Correlation => "correlation",
        MatrixSemantics::GeneralNumeric => "general_numeric",
    };
    if mode != expected_mode {
        return Err(SceneBuildError::new("input.scene.invalid_semantics", format!("matrix kind requires matrix_semantics.mode {expected_mode}"), "matrix_semantics.mode"));
    }
    if let Some(limits) = object.get("color_limits") {
        let limits = as_array(limits, "matrix_semantics.color_limits")?;
        if limits.len() != 2 {
            return Err(SceneBuildError::new("input.scene.invalid_semantics", "color_limits must contain exactly two values", "matrix_semantics.color_limits"));
        }
        let lower = as_number(&limits[0], "matrix_semantics.color_limits[0]")?;
        let upper = as_number(&limits[1], "matrix_semantics.color_limits[1]")?;
        if lower > upper {
            return Err(SceneBuildError::new("input.scene.invalid_semantics", "color_limits lower bound must not exceed upper bound", "matrix_semantics.color_limits"));
        }
    }
    if let Some(mapping) = object.get("color_mapping") {
        let mapping = as_string(mapping, "matrix_semantics.color_mapping")?;
        if !matches!(mapping, "sequential" | "diverging") {
            return Err(SceneBuildError::new("input.scene.invalid_semantics", "color_mapping must be sequential or diverging", "matrix_semantics.color_mapping"));
        }
    }
    if let Some(normalization) = object.get("normalization") {
        let normalization = as_object(normalization, "matrix_semantics.normalization")?;
        reject_unknown_fields(normalization, &["intercept", "zoom"], "matrix_semantics.normalization")?;
        as_number(required(normalization, "intercept", "matrix_semantics.normalization")?, "matrix_semantics.normalization.intercept")?;
        if as_number(required(normalization, "zoom", "matrix_semantics.normalization")?, "matrix_semantics.normalization.zoom")? <= 0.0 {
            return Err(SceneBuildError::new("input.scene.invalid_semantics", "normalization.zoom must be positive", "matrix_semantics.normalization.zoom"));
        }
    }
    if let Some(palette) = object.get("palette") {
        as_string(palette, "matrix_semantics.palette")?;
    }
    if let Some(observed) = object.get("has_observed_values") {
        if !matches!(observed, crate::json::Value::Bool(_)) {
            return Err(SceneBuildError::new("input.scene.invalid_type", "has_observed_values must be a boolean", "matrix_semantics.has_observed_values"));
        }
    }
    Ok(())
}

fn reject_unknown_fields(object: &std::collections::BTreeMap<String, crate::json::Value>, allowed: &[&str], path: &str) -> Result<(), SceneBuildError> {
    if let Some(key) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
        return Err(SceneBuildError::new("input.scene.unknown_field", format!("unknown field '{key}'"), format!("{path}.{key}")));
    }
    Ok(())
}

fn as_object<'a>(value: &'a crate::json::Value, path: &str) -> Result<&'a std::collections::BTreeMap<String, crate::json::Value>, SceneBuildError> {
    if let crate::json::Value::Object(object) = value { Ok(object) }
    else { Err(SceneBuildError::new("input.scene.invalid_type", "expected an object", path)) }
}

fn as_array<'a>(value: &'a crate::json::Value, path: &str) -> Result<&'a [crate::json::Value], SceneBuildError> {
    if let crate::json::Value::Array(array) = value { Ok(array) }
    else { Err(SceneBuildError::new("input.scene.invalid_type", "expected an array", path)) }
}

fn as_string<'a>(value: &'a crate::json::Value, path: &str) -> Result<&'a str, SceneBuildError> {
    if let crate::json::Value::String(string) = value { Ok(string) }
    else { Err(SceneBuildError::new("input.scene.invalid_type", "expected a string", path)) }
}

fn as_number(value: &crate::json::Value, path: &str) -> Result<f64, SceneBuildError> {
    if let crate::json::Value::Number(number) = value { Ok(*number) }
    else { Err(SceneBuildError::new("input.scene.invalid_type", "expected a finite number", path)) }
}

fn as_usize(value: &crate::json::Value, path: &str) -> Result<usize, SceneBuildError> {
    let number = as_number(value, path)?;
    if number < 0.0 || number.fract() != 0.0 || number > usize::MAX as f64 {
        return Err(SceneBuildError::new("input.scene.invalid_integer", "expected a non-negative integer", path));
    }
    Ok(number as usize)
}

fn is_srgb_hex(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
}

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
    fn maps_cell_validation_failures_to_stable_matrix_paths_and_coordinates() {
        let mut matrix = input();
        matrix.values[0][1] = Some(1.25);
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.code, "input.matrix.out_of_range");
        assert_eq!(error.path, "values[0][1]");
        assert_eq!(error.row, Some(0));
        assert_eq!(error.column, Some(1));

        matrix.values[0][1] = Some(f64::NAN);
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.path, "values[0][1]");
        assert_eq!(error.row, Some(0));
        assert_eq!(error.column, Some(1));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut matrix = input();
        matrix.schema_version = 2;
        let error = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap_err();
        assert_eq!(error.path, "schema_version");
    }

    #[test]
    fn parses_scene_v1_by_explicit_cell_identity_and_preserves_nulls() {
        let source = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[
            {"text":"alpha","row_x":1,"row_y":2,"column_x":3,"column_y":4},
            {"text":"beta","row_x":1,"row_y":2,"column_x":3,"column_y":4}
          ],
          "cells":[
            {"row":1,"column":1,"value":1,"x":10,"y":10,"center_x":15,"center_y":15,"size":10},
            {"row":0,"column":1,"value":null,"x":20,"y":10,"center_x":25,"center_y":15,"size":10},
            {"row":1,"column":0,"value":null,"x":10,"y":20,"center_x":15,"center_y":25,"size":10},
            {"row":0,"column":0,"value":1,"x":10,"y":10,"center_x":15,"center_y":15,"size":10}
          ]
        }"##;
        let matrix = parse_scene_v1_matrix(source).unwrap();
        assert_eq!(matrix.labels, vec!["alpha", "beta"]);
        assert_eq!(matrix.values, vec![vec![Some(1.0), None], vec![None, Some(1.0)]]);
        assert_eq!(matrix.source_indices, vec![0, 1]);
        assert_eq!(matrix.semantics, MatrixSemantics::Correlation);
    }

    #[test]
    fn parses_the_shared_canonical_scene_v1_fixture() {
        let matrix = parse_scene_v1_matrix(include_str!("../../../prototypes/shared-svg/scene-v1.example.json")).unwrap();
        assert_eq!(matrix.labels, vec!["alpha", "beta", "gamma"]);
        assert_eq!(matrix.values[0], vec![Some(1.0), Some(0.65), Some(-0.4)]);
        assert_eq!(matrix.values[1][2], None);
        assert_eq!(matrix.source_indices, vec![0, 1, 2]);
        assert_eq!(matrix.semantics, MatrixSemantics::Correlation);
        let scene = matrix_to_scene_v1(&matrix, &SceneBuildOptions::default()).unwrap();
        assert_eq!(scene.cells.len(), 9);
        scene.validate().unwrap();
    }

    #[test]
    fn scene_v1_parser_rejects_unsupported_versions_and_duplicate_cell_identity() {
        let source = r##"{
          "schema_version":2,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let error = parse_scene_v1_matrix(source).unwrap_err();
        assert_eq!(error.code, "input.scene.unsupported_version");
        assert_eq!(error.path, "schema_version");

        let duplicate = source.replace("\"schema_version\":2", "\"schema_version\":1")
            .replace("\"labels\":[{\"text\":\"a\",\"row_x\":1,\"row_y\":2,\"column_x\":3,\"column_y\":4}]",
                     "\"labels\":[{\"text\":\"a\",\"row_x\":1,\"row_y\":2,\"column_x\":3,\"column_y\":4},{\"text\":\"b\",\"row_x\":1,\"row_y\":2,\"column_x\":3,\"column_y\":4}]")
            .replace("\"cells\":[{\"row\":0,\"column\":0,\"value\":1,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1}]",
                     "\"cells\":[{\"row\":0,\"column\":0,\"value\":1,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1},{\"row\":0,\"column\":0,\"value\":1,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1},{\"row\":0,\"column\":1,\"value\":0,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1},{\"row\":1,\"column\":0,\"value\":0,\"x\":1,\"y\":1,\"center_x\":2,\"center_y\":2,\"size\":1}]");
        let error = parse_scene_v1_matrix(&duplicate).unwrap_err();
        assert_eq!(error.code, "input.scene.duplicate_cell");
        assert_eq!(error.row, Some(0));
        assert_eq!(error.column, Some(0));
    }

    #[test]
    fn scene_v1_parser_rejects_invalid_json_and_general_semantics_mismatch() {
        let error = parse_scene_v1_matrix("{").unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_json");

        let source = r##"{
          "schema_version":1,"kind":"general_matrix_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "matrix_semantics":{"mode":"correlation"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":2,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let error = parse_scene_v1_matrix(source).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_semantics");
        assert_eq!(error.path, "matrix_semantics.mode");

        let valid_general = source.replace("\"mode\":\"correlation\"", "\"mode\":\"general_numeric\"");
        let matrix = parse_scene_v1_matrix(&valid_general).unwrap();
        assert_eq!(matrix.semantics, MatrixSemantics::GeneralNumeric);
        assert_eq!(matrix.values, vec![vec![Some(2.0)]]);

        let unknown_field = valid_general.replace("\"title\":\"\"", "\"title\":\"\",\"unexpected\":true");
        let error = parse_scene_v1_matrix(&unknown_field).unwrap_err();
        assert_eq!(error.code, "input.scene.unknown_field");
        assert_eq!(error.path, "$.unexpected");
    }

}