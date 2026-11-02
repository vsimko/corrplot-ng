#![forbid(unsafe_code)]

//! Minimal WASI command host for testing one shared-core operation from R and Node.
//! The host protocol is one JSON request on stdin and one JSON response on stdout.

use corrplot_core::{validate_matrix, MatrixKind};
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

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

fn validate(request: MatrixRequest) -> Response {
    let kind = match request.kind.as_str() {
        "correlation" => MatrixKind::Correlation,
        "general_numeric" => MatrixKind::GeneralNumeric,
        other => return error(
            "configuration.invalid_field",
            "configuration",
            format!("unsupported matrix kind '{other}'"),
        ),
    };

    match validate_matrix(
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
    }
}

fn main() {
    let mut input = String::new();
    let response = match io::stdin().read_to_string(&mut input) {
        Ok(_) => match serde_json::from_str::<MatrixRequest>(&input) {
            Ok(request) => validate(request),
            Err(problem) => error(
                "configuration.invalid_input",
                "configuration",
                format!("invalid JSON request: {problem}"),
            ),
        },
        Err(problem) => error(
            "runtime.input_error",
            "runtime",
            format!("failed to read request from stdin: {problem}"),
        ),
    };

    // Keep stdout machine-readable; runtime diagnostics belong on stderr.
    match serde_json::to_string(&response) {
        Ok(json) => println!("{json}"),
        Err(problem) => {
            eprintln!("failed to serialize response: {problem}");
            std::process::exit(2);
        }
    }
}
