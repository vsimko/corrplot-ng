use std::collections::HashSet;

/// Matrix semantics supported by the shared validation boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixKind {
    Correlation,
    General,
}

/// Validate matrix shape and common numeric semantics independently of any
/// input format or runtime adapter.
///
/// Missing values are represented as `None`. Correlation matrices must be
/// finite (when present), bounded by [-1, 1], symmetric, and have a unit
/// diagonal. General numeric matrices only require finite observed values.
pub fn validate_matrix(
    labels: &[String],
    values: &[Vec<Option<f64>>],
    kind: MatrixKind,
) -> Result<(), String> {
    let n = labels.len();
    if n == 0 {
        return Err("matrix labels must not be empty".into());
    }
    if values.len() != n {
        return Err(format!("matrix must have {n} rows, found {}", values.len()));
    }

    let mut seen = HashSet::with_capacity(n);
    for label in labels {
        if label.trim().is_empty() {
            return Err("matrix labels must not be empty".into());
        }
        if !seen.insert(label) {
            return Err(format!("duplicate matrix label '{label}'"));
        }
    }

    for (row_index, row) in values.iter().enumerate() {
        if row.len() != n {
            return Err(format!(
                "matrix row {} has {} values; expected {n}",
                row_index + 1,
                row.len()
            ));
        }
        for (column_index, value) in row.iter().enumerate() {
            if let Some(value) = value {
                if !value.is_finite() {
                    return Err(format!(
                        "non-finite matrix value at row {}, column '{}'",
                        row_index + 1,
                        labels[column_index]
                    ));
                }
                if kind == MatrixKind::Correlation && !(-1.0..=1.0).contains(value) {
                    return Err(format!(
                        "correlation at row {}, column '{}' is outside [-1, 1]",
                        row_index + 1,
                        labels[column_index]
                    ));
                }
            }
        }
    }

    if kind == MatrixKind::Correlation {
        for row in 0..n {
            for column in 0..n {
                let symmetric = match (values[row][column], values[column][row]) {
                    (Some(left), Some(right)) => (left - right).abs() <= 1e-8,
                    (None, None) => true,
                    _ => false,
                };
                if !symmetric {
                    return Err(format!(
                        "correlation matrix is not symmetric at ('{}', '{}')",
                        labels[row], labels[column]
                    ));
                }
            }
        }
        for index in 0..n {
            if values[index][index].map_or(true, |value| (value - 1.0).abs() > 1e-8) {
                return Err(format!(
                    "correlation diagonal for '{}' must equal 1",
                    labels[index]
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
        vec!["alpha".into(), "beta".into()]
    }

    #[test]
    fn accepts_valid_correlation_with_missing_pairs() {
        let values = vec![vec![Some(1.0), None], vec![None, Some(1.0)]];
        assert_eq!(validate_matrix(&labels(), &values, MatrixKind::Correlation), Ok(()));
    }

    #[test]
    fn rejects_asymmetric_correlation() {
        let values = vec![vec![Some(1.0), Some(0.2)], vec![Some(0.3), Some(1.0)]];
        assert!(validate_matrix(&labels(), &values, MatrixKind::Correlation)
            .unwrap_err().contains("not symmetric"));
    }

    #[test]
    fn rejects_non_finite_and_out_of_range_values() {
        let non_finite = vec![vec![Some(1.0), Some(f64::NAN)], vec![Some(f64::NAN), Some(1.0)]];
        assert!(validate_matrix(&labels(), &non_finite, MatrixKind::General)
            .unwrap_err().contains("non-finite"));
        let out_of_range = vec![vec![Some(1.0), Some(1.2)], vec![Some(1.2), Some(1.0)]];
        assert!(validate_matrix(&labels(), &out_of_range, MatrixKind::Correlation)
            .unwrap_err().contains("outside [-1, 1]"));
    }

    #[test]
    fn general_matrix_allows_values_outside_unit_interval() {
        let values = vec![vec![Some(10.0), Some(-4.0)], vec![Some(2.0), Some(8.0)]];
        assert_eq!(validate_matrix(&labels(), &values, MatrixKind::General), Ok(()));
    }

    #[test]
    fn rejects_invalid_shape_and_duplicate_labels() {
        let short_row = vec![vec![Some(1.0)], vec![Some(0.0), Some(1.0)]];
        assert!(validate_matrix(&labels(), &short_row, MatrixKind::General).is_err());
        let duplicate = vec!["same".into(), "same".into()];
        let values = vec![vec![Some(1.0); 2]; 2];
        assert!(validate_matrix(&duplicate, &values, MatrixKind::General)
            .unwrap_err().contains("duplicate"));
    }
}
