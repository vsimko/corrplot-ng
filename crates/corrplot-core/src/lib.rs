#![forbid(unsafe_code)]

//! Shared matrix validation and renderer output contracts used by native and WebAssembly consumers.
//!
//! This crate is deliberately runtime-neutral: it contains no filesystem, process,
//! environment, dynamic-library, browser-DOM, or host-device integration. Native
//! and `wasm32-unknown-unknown` builds compile this same implementation.

pub mod output;
pub mod raster;
pub mod scene;

/// Renderer capability declaration for the typed SVG/RGBA scene backend.
pub const RENDERER_CAPABILITIES_V1: &str = include_str!("../renderer-capabilities.json");

use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixKind {
    Correlation,
    GeneralNumeric,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixValidationError {
    pub code: &'static str,
    pub message: String,
    pub row: Option<usize>,
    pub column: Option<usize>,
}

impl MatrixValidationError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into(), row: None, column: None }
    }

    fn at_cell(code: &'static str, message: impl Into<String>, row: usize, column: usize) -> Self {
        Self { code, message: message.into(), row: Some(row), column: Some(column) }
    }
}

impl fmt::Display for MatrixValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for MatrixValidationError {}

/// Validate labels, shape, missing values, finite values, and correlation semantics.
///
/// Values are row-major. None represents a missing cell; Some(NaN) and
/// infinities are invalid. Coordinates in errors are zero-based.
pub fn validate_matrix(
    labels: &[String],
    rows: usize,
    columns: usize,
    values: &[Option<f64>],
    kind: MatrixKind,
) -> Result<(), MatrixValidationError> {
    if labels.is_empty() || rows == 0 || columns == 0 {
        return Err(MatrixValidationError::new(
            "input.matrix.invalid_shape",
            "matrix and labels must be non-empty",
        ));
    }
    if rows != columns || rows != labels.len() {
        return Err(MatrixValidationError::new(
            "input.matrix.invalid_shape",
            format!(
                "matrix must be square and match labels: {} labels, {rows} rows, {columns} columns",
                labels.len()
            ),
        ));
    }
    let expected = rows.checked_mul(columns).ok_or_else(|| {
        MatrixValidationError::new("input.matrix.invalid_shape", "matrix dimensions overflow")
    })?;
    if values.len() != expected {
        return Err(MatrixValidationError::new(
            "input.matrix.invalid_shape",
            format!("matrix has {} values; expected {expected}", values.len()),
        ));
    }

    let mut seen = HashSet::with_capacity(labels.len());
    for (index, label) in labels.iter().enumerate() {
        if label.trim().is_empty() {
            return Err(MatrixValidationError::new(
                "input.matrix.invalid_label",
                format!("label at index {index} must not be empty"),
            ));
        }
        if !seen.insert(label) {
            return Err(MatrixValidationError::new(
                "input.matrix.invalid_label",
                format!("duplicate label '{label}'"),
            ));
        }
    }

    for row in 0..rows {
        for column in 0..columns {
            if let Some(value) = values[row * columns + column] {
                if !value.is_finite() {
                    return Err(MatrixValidationError::at_cell(
                        "input.matrix.non_finite_value",
                        format!("matrix value at row {row}, column {column} must be finite or missing"),
                        row,
                        column,
                    ));
                }
                if kind == MatrixKind::Correlation && !(-1.0..=1.0).contains(&value) {
                    return Err(MatrixValidationError::at_cell(
                        "input.matrix.out_of_range",
                        format!("correlation at row {row}, column {column} is outside [-1, 1]"),
                        row,
                        column,
                    ));
                }
            }
        }
    }

    if kind == MatrixKind::Correlation {
        for row in 0..rows {
            for column in 0..rows {
                let left = values[row * columns + column];
                let right = values[column * columns + row];
                let symmetric = match (left, right) {
                    (Some(a), Some(b)) => (a - b).abs() <= 1e-8,
                    (None, None) => true,
                    _ => false,
                };
                if !symmetric {
                    return Err(MatrixValidationError::at_cell(
                        "input.matrix.not_symmetric",
                        format!("correlation matrix is not symmetric at row {row}, column {column}"),
                        row,
                        column,
                    ));
                }
            }
        }
        for index in 0..rows {
            if values[index * columns + index].map_or(true, |value| (value - 1.0).abs() > 1e-8) {
                return Err(MatrixValidationError::at_cell(
                    "input.matrix.invalid_diagonal",
                    format!("correlation diagonal at index {index} must equal 1"),
                    index,
                    index,
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn labels() -> Vec<String> {
        vec!["a".into(), "b".into()]
    }

    #[test]
    fn accepts_symmetric_correlation_with_missing_pairs() {
        let values = [Some(1.0), None, None, Some(1.0)];
        assert_eq!(
            validate_matrix(&labels(), 2, 2, &values, MatrixKind::Correlation),
            Ok(())
        );
    }

    #[test]
    fn rejects_shape_and_duplicate_labels() {
        assert_eq!(
            validate_matrix(&labels(), 2, 1, &[Some(1.0), Some(1.0)], MatrixKind::GeneralNumeric)
                .unwrap_err().code,
            "input.matrix.invalid_shape"
        );
        let duplicate = vec!["a".into(), "a".into()];
        assert_eq!(
            validate_matrix(&duplicate, 2, 2, &[Some(1.0); 4], MatrixKind::GeneralNumeric)
                .unwrap_err().code,
            "input.matrix.invalid_label"
        );
    }

    #[test]
    fn rejects_non_finite_values_with_zero_based_coordinates() {
        let values = [Some(1.0), Some(f64::INFINITY), Some(f64::INFINITY), Some(1.0)];
        let error = validate_matrix(&labels(), 2, 2, &values, MatrixKind::GeneralNumeric).unwrap_err();
        assert_eq!(error.code, "input.matrix.non_finite_value");
        assert_eq!((error.row, error.column), (Some(0), Some(1)));
    }

    #[test]
    fn rejects_asymmetric_values_and_missing_pairs() {
        let values = [Some(1.0), Some(0.2), Some(0.3), Some(1.0)];
        assert_eq!(
            validate_matrix(&labels(), 2, 2, &values, MatrixKind::Correlation).unwrap_err().code,
            "input.matrix.not_symmetric"
        );
        let missing = [Some(1.0), None, Some(0.3), Some(1.0)];
        assert_eq!(
            validate_matrix(&labels(), 2, 2, &missing, MatrixKind::Correlation).unwrap_err().code,
            "input.matrix.not_symmetric"
        );
    }

    #[test]
    fn rejects_nan_and_invalid_correlation_diagonal() {
        let nan = [Some(1.0), Some(f64::NAN), Some(f64::NAN), Some(1.0)];
        assert_eq!(
            validate_matrix(&labels(), 2, 2, &nan, MatrixKind::Correlation).unwrap_err().code,
            "input.matrix.non_finite_value"
        );
        let diagonal = [Some(0.9), Some(0.2), Some(0.2), Some(1.0)];
        assert_eq!(
            validate_matrix(&labels(), 2, 2, &diagonal, MatrixKind::Correlation).unwrap_err().code,
            "input.matrix.invalid_diagonal"
        );
    }

    #[test]
    fn correlation_rules_are_shared_but_general_numeric_is_not_overconstrained() {
        let values = [Some(1.0), Some(1.5), Some(0.2), Some(1.0)];
        assert_eq!(
            validate_matrix(&labels(), 2, 2, &values, MatrixKind::Correlation).unwrap_err().code,
            "input.matrix.out_of_range"
        );
        assert!(validate_matrix(&labels(), 2, 2, &values, MatrixKind::GeneralNumeric).is_ok());
    }

    #[test]
    fn rejects_nan_and_infinity_but_preserves_explicit_missing_values() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let values = [Some(1.0), Some(value), Some(value), Some(1.0)];
            let error = validate_matrix(&labels(), 2, 2, &values, MatrixKind::GeneralNumeric)
                .unwrap_err();
            assert_eq!(error.code, "input.matrix.non_finite_value");
        }
        let values = [Some(1.0), None, None, Some(1.0)];
        assert!(validate_matrix(&labels(), 2, 2, &values, MatrixKind::Correlation).is_ok());
    }

    #[test]
    fn rejects_asymmetry_and_missing_mismatch_with_coordinates() {
        let asymmetric = [Some(1.0), Some(0.2), Some(0.3), Some(1.0)];
        let error = validate_matrix(&labels(), 2, 2, &asymmetric, MatrixKind::Correlation)
            .unwrap_err();
        assert_eq!(error.code, "input.matrix.not_symmetric");
        assert_eq!((error.row, error.column), (Some(0), Some(1)));

        let missing_mismatch = [Some(1.0), None, Some(0.2), Some(1.0)];
        let error = validate_matrix(&labels(), 2, 2, &missing_mismatch, MatrixKind::Correlation)
            .unwrap_err();
        assert_eq!(error.code, "input.matrix.not_symmetric");
    }

    #[test]
    fn rejects_invalid_correlation_diagonal() {
        let values = [Some(0.9), Some(0.2), Some(0.2), Some(1.0)];
        let error = validate_matrix(&labels(), 2, 2, &values, MatrixKind::Correlation)
            .unwrap_err();
        assert_eq!(error.code, "input.matrix.invalid_diagonal");
        assert_eq!((error.row, error.column), (Some(0), Some(0)));
    }

    #[test]
    fn reports_wrong_flat_value_count() {
        let error = validate_matrix(&labels(), 2, 2, &[Some(1.0)], MatrixKind::GeneralNumeric)
            .unwrap_err();
        assert_eq!(error.code, "input.matrix.invalid_shape");
    }
}
