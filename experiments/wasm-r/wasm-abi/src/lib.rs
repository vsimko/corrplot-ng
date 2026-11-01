//! Host-neutral JSON ABI for the R/Node Wasmtime embedding experiment.
//!
//! The host owns input/output buffers. Pointers are offsets into exported linear
//! memory. The u64 result packs output length in the high 32 bits and pointer in
//! the low 32 bits. This deliberately small ABI avoids wasm-bindgen imports.

use corrplot_core::{validate_matrix, MatrixKind};
use serde::{Deserialize, Serialize};
use std::slice;

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

fn validate_json(input: &[u8]) -> Vec<u8> {
    let response = match serde_json::from_slice::<MatrixRequest>(input) {
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
    serde_json::to_vec(&response).unwrap_or_else(|_| {
        br#"{"valid":false,"diagnostic":{"schema_version":1,"code":"runtime.serialize_error","severity":"error","stage":"runtime","message":"failed to serialize response","path":"matrix"}}"#.to_vec()
    })
}

/// Allocate a host-writable buffer in the module's linear memory.
#[no_mangle]
pub extern "C" fn corrplot_alloc(len: u32) -> u32 {
    let mut buffer = vec![0_u8; len as usize].into_boxed_slice();
    let pointer = buffer.as_mut_ptr() as u32;
    std::mem::forget(buffer);
    pointer
}

/// Release a buffer allocated by corrplot_alloc or returned by corrplot_validate_json.
#[no_mangle]
pub unsafe extern "C" fn corrplot_free(pointer: u32, len: u32) {
    if pointer != 0 && len != 0 {
        let slice = slice::from_raw_parts_mut(pointer as *mut u8, len as usize);
        drop(Box::from_raw(slice));
    }
}

/// Validate a JSON request at (pointer, len). Returns (output_len << 32) | output_ptr.
#[no_mangle]
pub unsafe extern "C" fn corrplot_validate_json(pointer: u32, len: u32) -> u64 {
    if pointer == 0 || len == 0 {
        return 0;
    }
    let input = slice::from_raw_parts(pointer as *const u8, len as usize);
    let output = validate_json(input).into_boxed_slice();
    let output_len = output.len() as u32;
    let output_pointer = Box::into_raw(output) as *mut u8 as u32;
    ((output_len as u64) << 32) | output_pointer as u64
}
