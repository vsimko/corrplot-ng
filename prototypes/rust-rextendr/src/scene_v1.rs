use extendr_api::prelude::*;

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\u{20}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn json_number(value: f64) -> String {
    let mut out = format!("{:.15}", value);
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    if out == "-0" { "0".to_string() } else { out }
}

/// Build the renderer-neutral scene-v1 JSON from an R character vector and matrix.
/// @export
#[extendr]
fn build_scene_json_rust(labels: Strings, matrix: RMatrix<f64>, annotation: &str) -> extendr_api::Result<String> {
    let n = labels.len();
    if n == 0 || matrix.nrows() != n || matrix.ncols() != n {
        return Err("labels and matrix dimensions must agree and be non-empty".into());
    }

    let cell = 84.0_f64;
    let left = 104.0_f64;
    let top = 76.0_f64;
    let width = left + n as f64 * cell + 24.0;
    let height = top + n as f64 * cell + 64.0;

    let mut out = String::with_capacity(n * n * 160 + n * 100 + 256);
    out.push_str("{\"schema_version\":1,\"kind\":\"correlation_plot\",\"width\":");
    out.push_str(&json_number(width));
    out.push_str(",\"height\":");
    out.push_str(&json_number(height));
    out.push_str(",\"labels\":[");
    for i in 0..n {
        if i > 0 { out.push(','); }
        let label = labels.elt(i);
        out.push_str("{\"text\":\"");
        out.push_str(&json_escape(label.as_ref()));
        out.push_str("\",\"row_x\":92,\"row_y\":");
        out.push_str(&json_number(top + i as f64 * cell + cell / 2.0));
        out.push_str(",\"column_x\":");
        out.push_str(&json_number(left + i as f64 * cell + cell / 2.0));
        out.push_str(",\"column_y\":60}");
    }

    out.push_str("],\"cells\":[");
    let mut first = true;
    for row in 0..n {
        for column in 0..n {
            let value = matrix[[row, column]];
            let is_missing = value.to_bits() == NA_REAL.to_bits();
            if !is_missing && (!value.is_finite() || value < -1.0 || value > 1.0) {
                return Err("matrix values must be finite or between -1 and 1".into());
            }
            if !first { out.push(','); }
            first = false;
            out.push_str("{\"row\":");
            out.push_str(&row.to_string());
            out.push_str(",\"column\":");
            out.push_str(&column.to_string());
            out.push_str(",\"value\":");
            if is_missing { out.push_str("null"); }
            else { out.push_str(&json_number(value)); }
            out.push_str(",\"x\":");
            out.push_str(&json_number(left + column as f64 * cell));
            out.push_str(",\"y\":");
            out.push_str(&json_number(top + row as f64 * cell));
            out.push_str(",\"center_x\":");
            out.push_str(&json_number(left + column as f64 * cell + cell / 2.0));
            out.push_str(",\"center_y\":");
            out.push_str(&json_number(top + row as f64 * cell + cell / 2.0));
            out.push_str(",\"size\":84}");
        }
    }
    out.push_str("],\"annotation\":\"");
    out.push_str(&json_escape(annotation));
    out.push_str("\",\"style\":{\"positive_color\":\"#2166ac\",\"negative_color\":\"#b2182b\",\"zero_color\":\"#bdbdbd\",\"missing_fill\":\"#f2f2f2\",\"grid_color\":\"#e0e0e0\"}}");
    Ok(out)
}

#[extendr]
fn scene_cell_count(n_labels: i32) -> i32 {
    if n_labels <= 0 { return 0; }
    n_labels.saturating_mul(n_labels)
}

extendr_module! {
    mod corrplotngrustspike;
    fn build_scene_json_rust;
    fn scene_cell_count;
}
