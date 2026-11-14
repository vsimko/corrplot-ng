//! Executable validation for the renderer-neutral visualization-model-v1 contract.
//!
//! This module deliberately does not compile geometry or render plots. It validates
//! semantic document invariants that JSON Schema cannot express (identity,
//! dimensions, and cross-references) before a future compiler consumes the model.

use crate::json::{self, Value};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisualizationModelError {
    pub code: &'static str,
    pub message: String,
    pub path: String,
}

impl VisualizationModelError {
    fn new(code: &'static str, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self { code, message: message.into(), path: path.into() }
    }
}

impl fmt::Display for VisualizationModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.path, self.message)
    }
}
impl std::error::Error for VisualizationModelError {}

type Result<T> = std::result::Result<T, VisualizationModelError>;
type Object = BTreeMap<String, Value>;

fn object<'a>(v: &'a Value, path: &str) -> Result<&'a Object> {
    match v {
        Value::Object(value) => Ok(value),
        _ => Err(VisualizationModelError::new("model.invalid_type", "expected object", path)),
    }
}
fn array<'a>(v: &'a Value, path: &str) -> Result<&'a [Value]> {
    match v {
        Value::Array(value) => Ok(value),
        _ => Err(VisualizationModelError::new("model.invalid_type", "expected array", path)),
    }
}
fn required<'a>(o: &'a Object, key: &str, path: &str) -> Result<&'a Value> {
    o.get(key).ok_or_else(|| VisualizationModelError::new(
        "model.missing_field", format!("required field '{key}' is missing"), format!("{path}.{key}")
    ))
}
fn string<'a>(v: &'a Value, path: &str) -> Result<&'a str> {
    match v {
        Value::String(value) => Ok(value),
        _ => Err(VisualizationModelError::new("model.invalid_type", "expected string", path)),
    }
}
fn number(v: &Value, path: &str) -> Result<f64> {
    match v {
        Value::Number(value) if value.is_finite() => Ok(*value),
        _ => Err(VisualizationModelError::new("model.invalid_number", "expected finite number", path)),
    }
}
fn unique_id(id: &str, ids: &mut HashSet<String>, path: &str) -> Result<()> {
    if id.trim().is_empty() {
        return Err(VisualizationModelError::new("model.invalid_id", "ID must not be empty", path));
    }
    if !ids.insert(id.to_owned()) {
        return Err(VisualizationModelError::new("model.duplicate_id", format!("duplicate ID '{id}'"), path));
    }
    Ok(())
}
fn id_field(o: &Object, path: &str, ids: &mut HashSet<String>) -> Result<String> {
    let id = string(required(o, "id", path)?, &format!("{path}.id"))?.to_owned();
    unique_id(&id, ids, &format!("{path}.id"))?;
    Ok(id)
}
fn optional_id_ref(o: &Object, key: &str, path: &str, known: &HashSet<String>) -> Result<()> {
    if let Some(value) = o.get(key) {
        let id = string(value, &format!("{path}.{key}"))?;
        if !known.contains(id) {
            return Err(VisualizationModelError::new(
                "model.reference.not_found", format!("unknown reference '{id}'"), format!("{path}.{key}")
            ));
        }
    }
    Ok(())
}

/// Validate a JSON visualization document against semantic invariants in v1.
///
/// Shape and closed vocabularies are additionally checked by the canonical JSON
/// Schema. This function rejects malformed JSON, duplicate IDs, invalid matrix
/// and table dimensions, invalid correlation semantics, and unresolved IDs.
pub fn validate_visualization_model_v1(source: &str) -> Result<()> {
    let root_value = json::parse(source).map_err(|e| VisualizationModelError::new(
        "model.invalid_json", format!("invalid JSON at byte {}: {}", e.offset, e.message), "$"
    ))?;
    let root = object(&root_value, "$")?;
    if number(required(root, "schema_version", "$")?, "$.schema_version")? != 1.0 {
        return Err(VisualizationModelError::new("model.unsupported_version", "only schema_version 1 is supported", "$.schema_version"));
    }
    if string(required(root, "kind", "$")?, "$.kind")? != "visualization_document" {
        return Err(VisualizationModelError::new("model.invalid_kind", "kind must be visualization_document", "$.kind"));
    }

    let layout = object(required(root, "layout", "$")?, "$.layout")?;
    for field in ["width", "height"] {
        if number(required(layout, field, "$.layout")?, &format!("$.layout.{field}"))? <= 0.0 {
            return Err(VisualizationModelError::new("model.layout.invalid_size", "layout dimensions must be positive", format!("$.layout.{field}")));
        }
    }
    if string(required(layout, "units", "$.layout")?, "$.layout.units")? != "logical" {
        return Err(VisualizationModelError::new("model.layout.invalid_units", "layout units must be logical", "$.layout.units"));
    }

    let mut ids = HashSet::new();
    let mut data_ids = HashSet::new();
    let mut scale_ids = HashSet::new();
    let sources = array(required(root, "data_sources", "$")?, "$.data_sources")?;
    if sources.is_empty() {
        return Err(VisualizationModelError::new("model.data_sources.empty", "at least one data source is required", "$.data_sources"));
    }
    for (i, source) in sources.iter().enumerate() {
        let path = format!("$.data_sources[{i}]");
        let source = object(source, &path)?;
        let id = id_field(source, &path, &mut ids)?;
        data_ids.insert(id);
        match string(required(source, "kind", &path)?, &format!("{path}.kind"))? {
            "matrix" => {
                let rows = array(required(source, "values", &path)?, &format!("{path}.values"))?;
                if rows.is_empty() {
                    return Err(VisualizationModelError::new("model.matrix.shape_mismatch", "matrix must have at least one row", format!("{path}.values")));
                }
                let first_len = array(&rows[0], &format!("{path}.values[0]"))?.len();
                if first_len == 0 {
                    return Err(VisualizationModelError::new("model.matrix.shape_mismatch", "matrix must have at least one column", format!("{path}.values[0]")));
                }
                let mut matrix = Vec::with_capacity(rows.len());
                for (r, row) in rows.iter().enumerate() {
                    let row = array(row, &format!("{path}.values[{r}]"))?;
                    if row.len() != first_len {
                        return Err(VisualizationModelError::new("model.matrix.shape_mismatch", "matrix rows must have equal lengths", format!("{path}.values[{r}]")));
                    }
                    let mut values = Vec::with_capacity(row.len());
                    for (c, value) in row.iter().enumerate() {
                        values.push(match value {
                            Value::Null => None,
                            Value::Number(n) if n.is_finite() => Some(*n),
                            _ => return Err(VisualizationModelError::new("model.matrix.invalid_value", "matrix values must be finite numbers or null", format!("{path}.values[{r}][{c}]"))),
                        });
                    }
                    matrix.push(values);
                }
                let (row_labels, column_labels) = if let Some(labels) = source.get("labels") {
                    let labels = array(labels, &format!("{path}.labels"))?;
                    let values = labels.iter().enumerate().map(|(i,v)| string(v, &format!("{path}.labels[{i}]")).map(str::to_owned)).collect::<Result<Vec<_>>>()?;
                    (values.clone(), values)
                } else {
                    let rows = array(required(source, "row_labels", &path)?, &format!("{path}.row_labels"))?;
                    let cols = array(required(source, "column_labels", &path)?, &format!("{path}.column_labels"))?;
                    (
                        rows.iter().enumerate().map(|(i,v)| string(v, &format!("{path}.row_labels[{i}]")).map(str::to_owned)).collect::<Result<Vec<_>>>()?,
                        cols.iter().enumerate().map(|(i,v)| string(v, &format!("{path}.column_labels[{i}]")).map(str::to_owned)).collect::<Result<Vec<_>>>()?,
                    )
                };
                if row_labels.len() != matrix.len() || column_labels.len() != first_len {
                    return Err(VisualizationModelError::new("model.matrix.shape_mismatch", "row/column label counts must match matrix dimensions", path.clone()));
                }
                for (i, label) in row_labels.iter().chain(column_labels.iter()).enumerate() {
                    if label.trim().is_empty() {
                        return Err(VisualizationModelError::new("model.matrix.invalid_label", "matrix labels must not be empty", format!("{path}.labels[{i}]")));
                    }
                }
                let semantics = string(required(source, "semantics", &path)?, &format!("{path}.semantics"))?;
                if semantics == "correlation" {
                    if matrix.len() != first_len || row_labels != column_labels {
                        return Err(VisualizationModelError::new("model.matrix.correlation_shape", "correlation matrices must be square and use identical row and column labels", path.clone()));
                    }
                    for r in 0..matrix.len() {
                        for c in 0..first_len {
                            if let Some(v) = matrix[r][c] {
                                if !(-1.0..=1.0).contains(&v) {
                                    return Err(VisualizationModelError::new("model.matrix.correlation_range", "correlations must be within [-1, 1]", format!("{path}.values[{r}][{c}]")));
                                }
                            }
                            if matrix[r][c] != matrix[c][r] {
                                let symmetric = match (matrix[r][c], matrix[c][r]) {
                                    (Some(a), Some(b)) => (a-b).abs() <= 1e-8,
                                    (None, None) => true,
                                    _ => false,
                                };
                                if !symmetric {
                                    return Err(VisualizationModelError::new("model.matrix.correlation_symmetry", "correlation matrix must be symmetric", format!("{path}.values[{r}][{c}]")));
                                }
                            }
                        }
                        if matrix[r][r].map_or(true, |v| (v-1.0).abs() > 1e-8) {
                            return Err(VisualizationModelError::new("model.matrix.correlation_diagonal", "correlation diagonal must equal 1", format!("{path}.values[{r}][{r}]")));
                        }
                    }
                } else if semantics != "general_numeric" {
                    return Err(VisualizationModelError::new("model.matrix.invalid_semantics", "semantics must be correlation or general_numeric", format!("{path}.semantics")));
                }
            }
            "table" => {
                let columns = object(required(source, "columns", &path)?, &format!("{path}.columns"))?;
                if columns.is_empty() {
                    return Err(VisualizationModelError::new("model.table.empty", "table must contain at least one column", format!("{path}.columns")));
                }
                let mut expected_len = None;
                for (name, column) in columns {
                    let cpath = format!("{path}.columns.{name}");
                    let column = object(column, &cpath)?;
                    let values = array(required(column, "values", &cpath)?, &format!("{cpath}.values"))?;
                    if let Some(expected) = expected_len {
                        if values.len() != expected {
                            return Err(VisualizationModelError::new("model.table.column_length_mismatch", "all table columns must have equal lengths", format!("{cpath}.values")));
                        }
                    } else { expected_len = Some(values.len()); }
                }
            }
            _ => return Err(VisualizationModelError::new("model.data_source.invalid_kind", "data source kind must be matrix or table", format!("{path}.kind"))),
        }
    }

    let scales = array(required(root, "scales", "$")?, "$.scales")?;
    for (i, scale) in scales.iter().enumerate() {
        let path = format!("$.scales[{i}]");
        let scale = object(scale, &path)?;
        let id = id_field(scale, &path, &mut ids)?;
        scale_ids.insert(id);
        let domain = array(required(scale, "domain", &path)?, &format!("{path}.domain"))?;
        let range = array(required(scale, "range", &path)?, &format!("{path}.range"))?;
        if domain.is_empty() || range.is_empty() {
            return Err(VisualizationModelError::new("model.scale.empty_domain_or_range", "scale domain and range must not be empty", path));
        }
        if string(required(scale, "kind", &path)?, &format!("{path}.kind"))? == "log" {
            for (j, value) in domain.iter().enumerate() {
                if number(value, &format!("{path}.domain[{j}]"))? <= 0.0 {
                    return Err(VisualizationModelError::new("model.scale.invalid_log_domain", "log scale domains must be positive", format!("{path}.domain[{j}]")));
                }
            }
        }
    }

    let plots = array(required(root, "plots", "$")?, "$.plots")?;
    if plots.is_empty() {
        return Err(VisualizationModelError::new("model.plots.empty", "at least one plot is required", "$.plots"));
    }
    for (i, plot) in plots.iter().enumerate() {
        let path = format!("$.plots[{i}]");
        let plot = object(plot, &path)?;
        id_field(plot, &path, &mut ids)?;
        let layers = array(required(plot, "layers", &path)?, &format!("{path}.layers"))?;
        if layers.is_empty() {
            return Err(VisualizationModelError::new("model.layers.empty", "plot must contain at least one layer", format!("{path}.layers")));
        }
        let mut layer_ids = HashSet::new();
        for (j, layer) in layers.iter().enumerate() {
            let lpath = format!("{path}.layers[{j}]");
            let layer = object(layer, &lpath)?;
            let id = id_field(layer, &lpath, &mut ids)?;
            layer_ids.insert(id);
            let data = string(required(layer, "data", &lpath)?, &format!("{lpath}.data"))?;
            if !data_ids.contains(data) {
                return Err(VisualizationModelError::new("model.reference.not_found", format!("unknown data source '{data}'"), format!("{lpath}.data")));
            }
            if let Some(encoding) = layer.get("encoding") {
                let encoding = object(encoding, &format!("{lpath}.encoding"))?;
                optional_id_ref(encoding, "color_scale", &format!("{lpath}.encoding"), &scale_ids)?;
                for channel in ["x", "y", "color", "size"] {
                    if let Some(channel_value) = encoding.get(channel) {
                        let channel_obj = object(channel_value, &format!("{lpath}.encoding.{channel}"))?;
                        optional_id_ref(channel_obj, "scale", &format!("{lpath}.encoding.{channel}"), &scale_ids)?;
                    }
                }
            }
        }
        if let Some(panels) = plot.get("panels") {
            for (j, panel) in array(panels, &format!("{path}.panels"))?.iter().enumerate() {
                let ppath = format!("{path}.panels[{j}]");
                let panel = object(panel, &ppath)?;
                id_field(panel, &ppath, &mut ids)?;
                for (k, layer_id) in array(required(panel, "layer_ids", &ppath)?, &format!("{ppath}.layer_ids"))?.iter().enumerate() {
                    let layer_id = string(layer_id, &format!("{ppath}.layer_ids[{k}]"))?;
                    if !layer_ids.contains(layer_id) {
                        return Err(VisualizationModelError::new("model.reference.not_found", format!("unknown layer '{layer_id}'"), format!("{ppath}.layer_ids[{k}]")));
                    }
                }
            }
        }
        if let Some(legends) = plot.get("legends") {
            for (j, legend) in array(legends, &format!("{path}.legends"))?.iter().enumerate() {
                let lpath = format!("{path}.legends[{j}]");
                let legend = object(legend, &lpath)?;
                id_field(legend, &lpath, &mut ids)?;
                optional_id_ref(legend, "scale", &lpath, &scale_ids)?;
            }
        }
        if let Some(annotations) = plot.get("annotations") {
            for (j, annotation) in array(annotations, &format!("{path}.annotations"))?.iter().enumerate() {
                let apath = format!("{path}.annotations[{j}]");
                let annotation = object(annotation, &apath)?;
                id_field(annotation, &apath, &mut ids)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_canonical_visualization_model_fixtures() {
        for fixture in [
            include_str!("../../../../fixtures/visualization-model-v1/legacy-corrplot.json"),
            include_str!("../../../../fixtures/visualization-model-v1/general-matrix.json"),
            include_str!("../../../../fixtures/visualization-model-v1/modern-bar-chart.json"),
        ] {
            validate_visualization_model_v1(fixture).unwrap();
        }
    }

    #[test]
    fn accepts_rectangular_general_numeric_matrix() {
        let fixture = include_str!("../../../../fixtures/visualization-model-v1/general-matrix.json");
        assert!(validate_visualization_model_v1(fixture).is_ok());
    }

    #[test]
    fn rejects_unresolved_data_reference() {
        let fixture = include_str!("../../../../fixtures/visualization-model-v1/modern-bar-chart.json")
            .replace("\"data\": \"", "\"data\": \"missing-");
        let error = validate_visualization_model_v1(&fixture).unwrap_err();
        assert_eq!(error.code, "model.reference.not_found");
    }

    #[test]
    fn rejects_unequal_matrix_rows() {
        let fixture = r#"{"schema_version":1,"kind":"visualization_document","layout":{"width":100,"height":80,"units":"logical"},"data_sources":[{"id":"m","kind":"matrix","values":[[1,2],[3]],"semantics":"general_numeric","row_labels":["a","b"],"column_labels":["x","y"]}],"scales":[],"plots":[{"id":"p","coordinate_system":{"kind":"matrix"},"layers":[{"id":"l","kind":"matrix","data":"m","encoding":{}}]}]}"#;
        assert_eq!(validate_visualization_model_v1(fixture).unwrap_err().code, "model.matrix.shape_mismatch");
    }
}
