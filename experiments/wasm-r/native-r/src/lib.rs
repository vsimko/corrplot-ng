//! Native R binding experiment. Validation is delegated to the same
//! corrplot-core function used by both WebAssembly builds.

use corrplot_core::{validate_matrix, MatrixKind};
use extendr_api::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct MatrixRequest {
    labels: Vec<String>,
    rows: usize,
    columns: usize,
    values: Vec<Option<f64>>,
    kind: String,
}

#[derive(Debug, Serialize)]
struct Diagnostic {
    schema_version: u32,
    code: String,
    severity: String,
    stage: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    row: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column: Option<usize>,
}

#[derive(Debug, Serialize)]
struct Response {
    valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostic: Option<Diagnostic>,
}

fn error(code: &str, stage: &str, message: impl Into<String>) -> Response {
    Response {
        valid: false,
        diagnostic: Some(Diagnostic {
            schema_version: 1,
            code: code.to_owned(),
            severity: "error".to_owned(),
            stage: stage.to_owned(),
            message: message.into(),
            path: Some("matrix".to_owned()),
            row: None,
            column: None,
        }),
    }
}

fn validate_json(input: &str) -> String {
    let response = match serde_json::from_str::<MatrixRequest>(input) {
        Err(problem) => error(
            "configuration.invalid_input",
            "configuration",
            format!("invalid JSON request: {problem}"),
        ),
        Ok(request) => {
            let kind = match request.kind.as_str() {
                "correlation" => Some(MatrixKind::Correlation),
                "general_numeric" => Some(MatrixKind::GeneralNumeric),
                _ => None,
            };
            match kind {
                None => error(
                    "configuration.invalid_field",
                    "configuration",
                    format!("unsupported matrix kind '{}'", request.kind),
                ),
                Some(kind) => match validate_matrix(
                    &request.labels,
                    request.rows,
                    request.columns,
                    &request.values,
                    kind,
                ) {
                    Ok(()) => Response { valid: true, diagnostic: None },
                    Err(problem) => Response {
                        valid: false,
                        diagnostic: Some(Diagnostic {
                            schema_version: 1,
                            code: problem.code.to_owned(),
                            severity: "error".to_owned(),
                            stage: "input".to_owned(),
                            message: problem.message,
                            path: Some("matrix".to_owned()),
                            row: problem.row,
                            column: problem.column,
                        }),
                    },
                },
            }
        }
    };
    serde_json::to_string(&response).unwrap_or_else(|_| {
        r#"{"valid":false,"diagnostic":{"schema_version":1,"code":"runtime.serialize_error","severity":"error","stage":"runtime","message":"failed to serialize response","path":"matrix"}}"#.to_owned()
    })
}

/// Accept and return JSON strings to keep the R/native boundary independent
/// from R's vector representation and identical to the WASM experiment ABI.
#[extendr]
fn corrplot_validate_json(request: &str) -> String {
    validate_json(request)
}

extendr_module! {
    mod corrplot_r_native;
}
