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
    validate_scene_optional_metadata(root)?;
    validate_scene_extended_metadata(root)?;
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
        validate_scene_label_metadata(label, &path)?;
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
            let field = if row >= n { "row" } else { "column" };
            return Err(SceneBuildError::new("input.scene.cell_out_of_bounds", "cell row and column must refer to an existing label", format!("{path}.{field}")).with_cell(row, column));
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
        validate_scene_cell_metadata(cell, &path)?;
    }
    if let Some(index) = seen.iter().position(|present| !present) {
        return Err(SceneBuildError::new("input.scene.missing_cell", "scene must contain every row/column cell identity", format!("cells[row={},column={}]", index / n, index % n)).with_cell(index / n, index % n));
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

fn validate_scene_label_metadata(label: &std::collections::BTreeMap<String, crate::json::Value>, path: &str) -> Result<(), SceneBuildError> {
    use crate::json::Value;
    for field in ["row_visible", "column_visible"] {
        if let Some(value) = label.get(field) {
            if !matches!(value, Value::Bool(_)) { return Err(SceneBuildError::new("input.scene.invalid_type", "expected a boolean", format!("{path}.{field}"))); }
        }
    }
    for (field, allowed) in [
        ("row_position", &["left", "diagonal"][..]),
        ("column_position", &["top", "diagonal"][..]),
    ] {
        if let Some(value) = label.get(field) {
            let value = as_string(value, &format!("{path}.{field}"))?;
            if !allowed.contains(&value) { return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported label position", format!("{path}.{field}"))); }
        }
    }
    for field in ["column_rotation", "offset"] {
        if let Some(value) = label.get(field) { as_number(value, &format!("{path}.{field}"))?; }
    }
    if let Some(value) = label.get("plotmath") {
        let plotmath = as_object(value, &format!("{path}.plotmath"))?;
        let plotmath_path = format!("{path}.plotmath");
        reject_unknown_fields(plotmath, &["prefix", "source"], &plotmath_path)?;
        let prefix = as_string(required(plotmath, "prefix", &plotmath_path)?, &format!("{plotmath_path}.prefix"))?;
        if ![":", "=", "$"].contains(&prefix) { return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported plotmath prefix", format!("{plotmath_path}.prefix"))); }
        if as_string(required(plotmath, "source", &plotmath_path)?, &format!("{plotmath_path}.source"))?.is_empty() {
            return Err(SceneBuildError::new("input.scene.invalid_value", "plotmath source must not be empty", format!("{plotmath_path}.source")));
        }
    }
    Ok(())
}

fn validate_scene_cell_metadata(cell: &std::collections::BTreeMap<String, crate::json::Value>, path: &str) -> Result<(), SceneBuildError> {
    use crate::json::Value;
    if let Some(value) = cell.get("color") {
        let color = as_string(value, &format!("{path}.color"))?;
        if !is_srgb_hex(color) { return Err(SceneBuildError::new("input.scene.invalid_color", "expected a six-digit sRGB hex color", format!("{path}.color"))); }
    }
    if let Some(value) = cell.get("visible") {
        if !matches!(value, Value::Bool(_)) { return Err(SceneBuildError::new("input.scene.invalid_type", "visible must be a boolean", format!("{path}.visible"))); }
    }
    for (field, allowed) in [
        ("method", &["circle", "square", "ellipse", "number", "shade", "color", "pie"][..]),
        ("region", &["upper", "lower", "diagonal"][..]),
    ] {
        if let Some(value) = cell.get(field) {
            let value = as_string(value, &format!("{path}.{field}"))?;
            if !allowed.contains(&value) { return Err(SceneBuildError::new("input.scene.invalid_value", format!("unsupported {field} value"), format!("{path}.{field}"))); }
        }
    }
    for field in ["p_value", "normalized_value"] {
        if let Some(value) = cell.get(field) {
            if !matches!(value, Value::Null) { as_number(value, &format!("{path}.{field}"))?; }
        }
    }
    if let Some(value) = cell.get("p_value") {
        if let Value::Number(number) = value {
            if !(0.0..=1.0).contains(number) { return Err(SceneBuildError::new("input.scene.invalid_value", "p_value must be between 0 and 1", format!("{path}.p_value"))); }
        }
    }
    if let Some(value) = cell.get("encoding") {
        let encoding = as_object(value, &format!("{path}.encoding"))?;
        let ep = format!("{path}.encoding");
        reject_unknown_fields(encoding, &["kind", "magnitude", "sign", "color", "radius", "x", "y", "width", "height", "center_x", "center_y", "rx", "ry", "rotation", "fill_opacity", "fill", "line", "full_circle", "large_arc", "sweep_flag", "start_x", "start_y", "end_x", "end_y"], &ep)?;
        let kind = as_string(required(encoding, "kind", &ep)?, &format!("{ep}.kind"))?;
        if !["circle", "square", "ellipse", "number", "shade", "color", "pie"].contains(&kind) { return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported encoding kind", format!("{ep}.kind"))); }
        let magnitude = as_number(required(encoding, "magnitude", &ep)?, &format!("{ep}.magnitude"))?;
        if !(0.0..=1.0).contains(&magnitude) { return Err(SceneBuildError::new("input.scene.invalid_value", "magnitude must be between 0 and 1", format!("{ep}.magnitude"))); }
        let sign = as_number(required(encoding, "sign", &ep)?, &format!("{ep}.sign"))?;
        if ![-1.0, 0.0, 1.0].contains(&sign) { return Err(SceneBuildError::new("input.scene.invalid_value", "sign must be -1, 0 or 1", format!("{ep}.sign"))); }
        for field in ["color", "fill"] {
            if let Some(value) = encoding.get(field) {
                let color = as_string(value, &format!("{ep}.{field}"))?;
                if !is_srgb_hex(color) { return Err(SceneBuildError::new("input.scene.invalid_color", "expected a six-digit sRGB hex color", format!("{ep}.{field}"))); }
            } else if field == "color" {
                return Err(SceneBuildError::new("input.scene.missing_field", "required field 'color' is missing", format!("{ep}.color")));
            }
        }
        for field in ["radius", "width", "height", "rx", "ry"] {
            if let Some(value) = encoding.get(field) {
                if as_number(value, &format!("{ep}.{field}"))? < 0.0 { return Err(SceneBuildError::new("input.scene.invalid_value", "value must be non-negative", format!("{ep}.{field}"))); }
            }
        }
        for field in ["x", "y", "center_x", "center_y", "rotation", "start_x", "start_y", "end_x", "end_y"] {
            if let Some(value) = encoding.get(field) { as_number(value, &format!("{ep}.{field}"))?; }
        }
        for field in ["fill_opacity"] {
            if let Some(value) = encoding.get(field) {
                let number = as_number(value, &format!("{ep}.{field}"))?;
                if !(0.0..=1.0).contains(&number) { return Err(SceneBuildError::new("input.scene.invalid_value", "opacity must be between 0 and 1", format!("{ep}.{field}"))); }
            }
        }
        for field in ["full_circle", "large_arc"] {
            if let Some(value) = encoding.get(field) {
                if !matches!(value, Value::Bool(_)) { return Err(SceneBuildError::new("input.scene.invalid_type", "expected a boolean", format!("{ep}.{field}"))); }
            }
        }
        if let Some(value) = encoding.get("sweep_flag") {
            let number = as_number(value, &format!("{ep}.sweep_flag"))?;
            if number != 0.0 && number != 1.0 { return Err(SceneBuildError::new("input.scene.invalid_value", "sweep_flag must be 0 or 1", format!("{ep}.sweep_flag"))); }
        }
        if let Some(value) = encoding.get("line") {
            let line = as_object(value, &format!("{ep}.line"))?;
            let lp = format!("{ep}.line");
            reject_unknown_fields(line, &["x1", "y1", "x2", "y2", "stroke_width", "stroke_opacity"], &lp)?;
            for field in ["x1", "y1", "x2", "y2", "stroke_width", "stroke_opacity"] {
                let number = as_number(required(line, field, &lp)?, &format!("{lp}.{field}"))?;
                if field == "stroke_width" && number < 0.0 || field == "stroke_opacity" && !(0.0..=1.0).contains(&number) {
                    return Err(SceneBuildError::new("input.scene.invalid_value", "line width or opacity is out of range", format!("{lp}.{field}")));
                }
            }
        }
    }
    if let Some(value) = cell.get("significance") {
        let object = as_object(value, &format!("{path}.significance"))?;
        let p = format!("{path}.significance");
        reject_unknown_fields(object, &["p_value", "status", "annotation", "blank"], &p)?;
        let pv = required(object, "p_value", &p)?;
        if !matches!(pv, Value::Null) {
            let number = as_number(pv, &format!("{p}.p_value"))?;
            if !(0.0..=1.0).contains(&number) { return Err(SceneBuildError::new("input.scene.invalid_value", "p_value must be between 0 and 1", format!("{p}.p_value"))); }
        }
        let status = as_string(required(object, "status", &p)?, &format!("{p}.status"))?;
        if !["unknown", "significant", "not_significant"].contains(&status) { return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported significance status", format!("{p}.status"))); }
        let annotation = required(object, "annotation", &p)?;
        if !matches!(annotation, Value::Null) { as_string(annotation, &format!("{p}.annotation"))?; }
        if !matches!(required(object, "blank", &p)?, Value::Bool(_)) { return Err(SceneBuildError::new("input.scene.invalid_type", "blank must be a boolean", format!("{p}.blank"))); }
    }
    if let Some(value) = cell.get("confidence_interval") {
        let object = as_object(value, &format!("{path}.confidence_interval"))?;
        let p = format!("{path}.confidence_interval");
        reject_unknown_fields(object, &["lower", "upper", "x1", "x2", "y", "cap_height", "stroke_width", "color"], &p)?;
        let lower = as_number(required(object, "lower", &p)?, &format!("{p}.lower"))?;
        let upper = as_number(required(object, "upper", &p)?, &format!("{p}.upper"))?;
        for (field, number) in [("lower", lower), ("upper", upper)] {
            if !(-1.0..=1.0).contains(&number) { return Err(SceneBuildError::new("input.scene.invalid_value", "confidence interval bounds must be between -1 and 1", format!("{p}.{field}"))); }
        }
        if lower > upper { return Err(SceneBuildError::new("input.scene.invalid_value", "lower bound must not exceed upper bound", format!("{p}.lower"))); }
        for field in ["x1", "x2", "y", "cap_height", "stroke_width"] {
            let number = as_number(required(object, field, &p)?, &format!("{p}.{field}"))?;
            if (field == "cap_height" || field == "stroke_width") && number < 0.0 { return Err(SceneBuildError::new("input.scene.invalid_value", "value must be non-negative", format!("{p}.{field}"))); }
        }
        let color = as_string(required(object, "color", &p)?, &format!("{p}.color"))?;
        if !is_srgb_hex(color) { return Err(SceneBuildError::new("input.scene.invalid_color", "expected a six-digit sRGB hex color", format!("{p}.color"))); }
    }
    Ok(())
}


fn validate_scene_extended_metadata(root: &std::collections::BTreeMap<String, crate::json::Value>) -> Result<(), SceneBuildError> {
    use crate::json::Value;
    if let Some(value) = root.get("cluster_regions") {
        for (index, item) in as_array(value, "cluster_regions")?.iter().enumerate() {
            let path = format!("cluster_regions[{index}]");
            let object = as_object(item, &path)?;
            reject_unknown_fields(object, &["id", "variables", "source_indices", "row_start", "row_end", "column_start", "column_end", "x", "y", "width", "height", "color", "line_width"], &path)?;
            if as_string(required(object, "id", &path)?, &format!("{path}.id"))?.is_empty() {
                return Err(SceneBuildError::new("input.scene.invalid_value", "cluster region id must not be empty", format!("{path}.id")));
            }
            for field in ["variables", "source_indices"] {
                let entries = as_array(required(object, field, &path)?, &format!("{path}.{field}"))?;
                if entries.is_empty() {
                    return Err(SceneBuildError::new("input.scene.invalid_value", "cluster region arrays must not be empty", format!("{path}.{field}")));
                }
                let mut unique = HashSet::with_capacity(entries.len());
                for (entry_index, entry) in entries.iter().enumerate() {
                    let entry_path = format!("{path}.{field}[{entry_index}]");
                    let identity = if field == "variables" {
                        let text = as_string(entry, &entry_path)?;
                        if text.is_empty() { return Err(SceneBuildError::new("input.scene.invalid_value", "cluster variable must not be empty", entry_path)); }
                        text.to_string()
                    } else {
                        as_usize(entry, &entry_path)?.to_string()
                    };
                    if !unique.insert(identity) {
                        return Err(SceneBuildError::new("input.scene.invalid_value", "cluster region entries must be unique", entry_path));
                    }
                }
            }
            for field in ["row_start", "row_end", "column_start", "column_end"] {
                as_usize(required(object, field, &path)?, &format!("{path}.{field}"))?;
            }
            for field in ["x", "y"] {
                as_number(required(object, field, &path)?, &format!("{path}.{field}"))?;
            }
            for field in ["width", "height", "line_width"] {
                if as_number(required(object, field, &path)?, &format!("{path}.{field}"))? <= 0.0 {
                    return Err(SceneBuildError::new("input.scene.invalid_value", "cluster region dimensions and line_width must be positive", format!("{path}.{field}")));
                }
            }
            let color = as_string(required(object, "color", &path)?, &format!("{path}.color"))?;
            if !is_srgb_hex(color) {
                return Err(SceneBuildError::new("input.scene.invalid_color", "expected a six-digit sRGB hex color", format!("{path}.color")));
            }
        }
    }
    if let Some(value) = root.get("figure_text") {
        for (index, item) in as_array(value, "figure_text")?.iter().enumerate() {
            let path = format!("figure_text[{index}]");
            let object = as_object(item, &path)?;
            reject_unknown_fields(object, &["role", "text", "lines", "alignment", "font_family", "font_size", "font_weight", "font_style", "line_height", "spacing_after", "x", "y"], &path)?;
            let role = as_string(required(object, "role", &path)?, &format!("{path}.role"))?;
            if !["title", "subtitle", "caption", "footnote"].contains(&role) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported figure text role", format!("{path}.role")));
            }
            as_string(required(object, "text", &path)?, &format!("{path}.text"))?;
            let lines = as_array(required(object, "lines", &path)?, &format!("{path}.lines"))?;
            if lines.is_empty() { return Err(SceneBuildError::new("input.scene.invalid_value", "figure text lines must not be empty", format!("{path}.lines"))); }
            for (line_index, line) in lines.iter().enumerate() {
                as_string(line, &format!("{path}.lines[{line_index}]"))?;
            }
            let alignment = as_string(required(object, "alignment", &path)?, &format!("{path}.alignment"))?;
            if !["left", "center", "right"].contains(&alignment) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported figure text alignment", format!("{path}.alignment")));
            }
            if as_string(required(object, "font_family", &path)?, &format!("{path}.font_family"))?.is_empty() {
                return Err(SceneBuildError::new("input.scene.invalid_value", "font family must not be empty", format!("{path}.font_family")));
            }
            let font_size = as_number(required(object, "font_size", &path)?, &format!("{path}.font_size"))?;
            if !(6.0..=48.0).contains(&font_size) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "font size must be between 6 and 48", format!("{path}.font_size")));
            }
            let font_weight = as_string(required(object, "font_weight", &path)?, &format!("{path}.font_weight"))?;
            if !["normal", "bold"].contains(&font_weight) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported figure text font weight", format!("{path}.font_weight")));
            }
            let font_style = as_string(required(object, "font_style", &path)?, &format!("{path}.font_style"))?;
            if !["normal", "italic"].contains(&font_style) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported figure text font style", format!("{path}.font_style")));
            }
            if as_number(required(object, "line_height", &path)?, &format!("{path}.line_height"))? <= 0.0 {
                return Err(SceneBuildError::new("input.scene.invalid_value", "line_height must be positive", format!("{path}.line_height")));
            }
            if as_number(required(object, "spacing_after", &path)?, &format!("{path}.spacing_after"))? < 0.0 {
                return Err(SceneBuildError::new("input.scene.invalid_value", "spacing_after must be non-negative", format!("{path}.spacing_after")));
            }
            for field in ["x", "y"] { as_number(required(object, field, &path)?, &format!("{path}.{field}"))?; }
        }
    }
    if let Some(value) = root.get("significance") {
        let object = as_object(value, "significance")?;
        reject_unknown_fields(object, &["mode", "thresholds", "symbol"], "significance")?;
        let mode = as_string(required(object, "mode", "significance")?, "significance.mode")?;
        if !["pch", "p-value", "blank", "stars"].contains(&mode) {
            return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported significance mode", "significance.mode"));
        }
        let thresholds = as_array(required(object, "thresholds", "significance")?, "significance.thresholds")?;
        if thresholds.is_empty() {
            return Err(SceneBuildError::new("input.scene.invalid_value", "significance thresholds must not be empty", "significance.thresholds"));
        }
        for (index, threshold) in thresholds.iter().enumerate() {
            let number = as_number(threshold, &format!("significance.thresholds[{index}]"))?;
            if !(0.0..=1.0).contains(&number) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "significance thresholds must be between 0 and 1", format!("significance.thresholds[{index}]")));
            }
        }
        let symbol = as_string(required(object, "symbol", "significance")?, "significance.symbol")?;
        if symbol.is_empty() || symbol.chars().count() > 8 {
            return Err(SceneBuildError::new("input.scene.invalid_value", "significance symbol must contain 1 to 8 characters", "significance.symbol"));
        }
    }
    if let Some(value) = root.get("text_annotations") {
        for (index, item) in as_array(value, "text_annotations")?.iter().enumerate() {
            let path = format!("text_annotations[{index}]");
            let object = as_object(item, &path)?;
            reject_unknown_fields(object, &["row", "column", "role", "text", "color", "font_size", "font_weight", "font_style"], &path)?;
            as_usize(required(object, "row", &path)?, &format!("{path}.row"))?;
            as_usize(required(object, "column", &path)?, &format!("{path}.column"))?;
            let role = as_string(required(object, "role", &path)?, &format!("{path}.role"))?;
            if !["coefficient", "value", "missing"].contains(&role) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported text annotation role", format!("{path}.role")));
            }
            as_string(required(object, "text", &path)?, &format!("{path}.text"))?;
            if as_string(required(object, "color", &path)?, &format!("{path}.color"))?.is_empty() {
                return Err(SceneBuildError::new("input.scene.invalid_value", "text annotation color must not be empty", format!("{path}.color")));
            }
            if as_number(required(object, "font_size", &path)?, &format!("{path}.font_size"))? <= 0.0 {
                return Err(SceneBuildError::new("input.scene.invalid_value", "text annotation font_size must be positive", format!("{path}.font_size")));
            }
            let weight = as_string(required(object, "font_weight", &path)?, &format!("{path}.font_weight"))?;
            if !["normal", "bold"].contains(&weight) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported text annotation font weight", format!("{path}.font_weight")));
            }
            let style = as_string(required(object, "font_style", &path)?, &format!("{path}.font_style"))?;
            if !["normal", "italic"].contains(&style) {
                return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported text annotation font style", format!("{path}.font_style")));
            }
        }
    }
    Ok(())
}

fn validate_scene_optional_metadata(root: &std::collections::BTreeMap<String, crate::json::Value>) -> Result<(), SceneBuildError> {
    use crate::json::Value;
    if let Some(value) = root.get("typography") {
        let object = as_object(value, "typography")?;
        reject_unknown_fields(object, &["label_font_family", "label_font_size", "label_font_weight", "column_label_orientation", "label_collision"], "typography")?;
        if as_string(required(object, "label_font_family", "typography")?, "typography.label_font_family")?.is_empty() {
            return Err(SceneBuildError::new("input.scene.invalid_value", "font family must not be empty", "typography.label_font_family"));
        }
        let size = as_number(required(object, "label_font_size", "typography")?, "typography.label_font_size")?;
        if !(6.0..=48.0).contains(&size) { return Err(SceneBuildError::new("input.scene.invalid_value", "font size must be between 6 and 48", "typography.label_font_size")); }
        for (field, allowed) in [
            ("label_font_weight", &["normal", "bold"][..]),
            ("column_label_orientation", &["vertical", "horizontal"][..]),
            ("label_collision", &["preserve", "wrap", "truncate", "omit"][..]),
        ] {
            let value = as_string(required(object, field, "typography")?, &format!("typography.{field}"))?;
            if !allowed.contains(&value) { return Err(SceneBuildError::new("input.scene.invalid_value", format!("unsupported {field} value"), format!("typography.{field}"))); }
        }
    }
    if let Some(value) = root.get("legend") {
        for (index, item) in as_array(value, "legend")?.iter().enumerate() {
            let path = format!("legend[{index}]");
            let object = as_object(item, &path)?;
            reject_unknown_fields(object, &["kind", "label", "color"], &path)?;
            let kind = as_string(required(object, "kind", &path)?, &format!("{path}.kind"))?;
            if !["stop", "interval", "missing"].contains(&kind) { return Err(SceneBuildError::new("input.scene.invalid_value", "unsupported legend kind", format!("{path}.kind"))); }
            as_string(required(object, "label", &path)?, &format!("{path}.label"))?;
            let color = as_string(required(object, "color", &path)?, &format!("{path}.color"))?;
            if !is_srgb_hex(color) { return Err(SceneBuildError::new("input.scene.invalid_color", "expected a six-digit sRGB hex color", format!("{path}.color"))); }
        }
    }
    for (field, allowed) in [
        ("glyph_method", &["circle", "square", "ellipse", "number", "shade", "color", "pie"][..]),
        ("display_type", &["full", "upper", "lower", "diagonal"][..]),
    ] {
        if let Some(value) = root.get(field) {
            let value = as_string(value, field)?;
            if !allowed.contains(&value) { return Err(SceneBuildError::new("input.scene.invalid_value", format!("unsupported {field} value"), field)); }
        }
    }
    if let Some(value) = root.get("show_diagonal") {
        if !matches!(value, Value::Bool(_)) { return Err(SceneBuildError::new("input.scene.invalid_type", "show_diagonal must be a boolean", "show_diagonal")); }
    }
    Ok(())
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
    let limits = as_array(required(object, "color_limits", "matrix_semantics")?, "matrix_semantics.color_limits")?;
    if limits.len() != 2 {
        return Err(SceneBuildError::new("input.scene.invalid_semantics", "color_limits must contain exactly two values", "matrix_semantics.color_limits"));
    }
    let lower = as_number(&limits[0], "matrix_semantics.color_limits[0]")?;
    let upper = as_number(&limits[1], "matrix_semantics.color_limits[1]")?;
    if lower > upper {
        return Err(SceneBuildError::new("input.scene.invalid_semantics", "color_limits lower bound must not exceed upper bound", "matrix_semantics.color_limits"));
    }
    let mapping = as_string(required(object, "color_mapping", "matrix_semantics")?, "matrix_semantics.color_mapping")?;
    if !matches!(mapping, "sequential" | "diverging") {
        return Err(SceneBuildError::new("input.scene.invalid_semantics", "color_mapping must be sequential or diverging", "matrix_semantics.color_mapping"));
    }
    let normalization = as_object(required(object, "normalization", "matrix_semantics")?, "matrix_semantics.normalization")?;
    reject_unknown_fields(normalization, &["intercept", "zoom"], "matrix_semantics.normalization")?;
    as_number(required(normalization, "intercept", "matrix_semantics.normalization")?, "matrix_semantics.normalization.intercept")?;
    if as_number(required(normalization, "zoom", "matrix_semantics.normalization")?, "matrix_semantics.normalization.zoom")? <= 0.0 {
        return Err(SceneBuildError::new("input.scene.invalid_semantics", "normalization.zoom must be positive", "matrix_semantics.normalization.zoom"));
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
    if input.schema_version != 1 {
        return Err(SceneBuildError::new("input.scene.unsupported_version", "only normalized_matrix schema_version 1 is supported", "schema_version"));
    }
    if input.kind != "normalized_matrix" {
        return Err(SceneBuildError::new("input.matrix.invalid_kind", "expected normalized_matrix input kind", "kind"));
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
                CellMethod::Ellipse => CellGlyph::Ellipse {
                    rx: CELL_SIZE * 0.43,
                    ry: (CELL_SIZE * 0.43 * (1.0 - magnitude)).max(CELL_SIZE * 0.015),
                    rotation_degrees: if raw > 0.0 { -45.0 } else if raw < 0.0 { 45.0 } else { 0.0 },
                    color,
                },
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
            if method == CellMethod::Ellipse {
                assert!(matches!(&scene.cells[0].glyph, Some(CellGlyph::Ellipse { rotation_degrees, .. }) if *rotation_degrees == -45.0));
                assert!(matches!(&scene.cells[1].glyph, Some(CellGlyph::Ellipse { rotation_degrees, .. }) if *rotation_degrees == 45.0));
            }
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
    fn validates_optional_scene_v1_typography_legend_and_display_metadata() {
        let base = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let valid = base.replace("\"title\":\"\"", "\"title\":\"\",\"typography\":{\"label_font_family\":\"sans\",\"label_font_size\":12,\"label_font_weight\":\"bold\",\"column_label_orientation\":\"vertical\",\"label_collision\":\"wrap\"},\"legend\":[{\"kind\":\"missing\",\"label\":\"NA\",\"color\":\"#f2f2f2\"}],\"glyph_method\":\"circle\",\"display_type\":\"full\",\"show_diagonal\":true");
        assert!(parse_scene_v1_matrix(&valid).is_ok());
        let invalid = valid.replace("\"label_font_size\":12", "\"label_font_size\":60");
        let error = parse_scene_v1_matrix(&invalid).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_value");
        assert_eq!(error.path, "typography.label_font_size");
        let invalid_legend = valid.replace("\"#f2f2f2\"}],\"glyph_method\"", "\"red\"}],\"glyph_method\"");
        let error = parse_scene_v1_matrix(&invalid_legend).unwrap_err();
        assert_eq!(error.code, "input.scene.invalid_color");
        assert_eq!(error.path, "legend[0].color");
    }

    #[test]
    fn validates_scene_v1_cluster_regions_figure_text_global_significance_and_annotations() {
        let base = r##"{
          "schema_version":1,"kind":"correlation_plot","width":100,"height":100,
          "title":"","description":"","annotation":"",
          "style":{"positive_color":"#2166ac","negative_color":"#b2182b","zero_color":"#bdbdbd","missing_fill":"#f2f2f2","grid_color":"#e0e0e0"},
          "labels":[{"text":"a","row_x":1,"row_y":2,"column_x":3,"column_y":4}],
          "cells":[{"row":0,"column":0,"value":1,"x":1,"y":1,"center_x":2,"center_y":2,"size":1}]
        }"##;
        let valid = base.replace(
            "\"title\":\"\"",
            "\\"title\\":\\"\\",\\"cluster_regions\\":[{\\"id\\":\\"c1\\",\\"variables\\":[\\"a\\",\\"b\\"],\\"source_indices\\":[0,1],\\"row_start\\":0,\\"row_end\\":1,\\"column_start\\":0,\\"column_end\\":1,\\"x\\":1,\\"y\\":2,\\"width\\":10,\\"height\\":10,\\"color\\":\\"#2166ac\\",\\"line_width\\":1}],\\"figure_text\\":[{\\"role\\":\\"caption\\",\\"text\\":\\"caption\\",\\"lines\\":[\\"caption\\"],\\"alignment\\":\\"center\\",\\"font_family\\":\\"sans\\",\\"font_size\\":10,\\"font_weight\\":\\"normal\\",\\"font_style\\":\\"italic\\",\\"line_height\\":1.2,\\"spacing_after\\":0,\\"x\\":0,\\"y\\":90}],\\"significance\\":{\\"mode\\":\\"stars\\",\\"thresholds\\":[0.05,0.01],\\"symbol\\":\\"*\\"},\\"text_annotations\\":[{\\"row\\":0,\\"column\\":0,\\"role\\":\\"coefficient\\",\\"text\\":\\"1.0\\",\\"color\\":\\"black\\",\\"font_size\\":8,\\"font_weight\\":\\"bold\\",\\"font_style\\":\\"normal\\"}]"
        );
        assert!(parse_scene_v1_matrix(&valid).is_ok());
