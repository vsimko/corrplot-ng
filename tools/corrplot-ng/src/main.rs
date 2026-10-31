use corrplot_core::output::{RenderOutput, Renderer};
use corrplot_core::{validate_matrix, MatrixKind};
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;
use std::path::{Path, PathBuf};

mod cache;
mod publication;
mod wizard;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const RENDERER_CAPABILITIES: &str = include_str!("../renderer-capabilities.json");
/// Bump when rendering semantics change without a package-version change.
const RENDERER_CACHE_VERSION: &str = "1";
const CELL: f64 = 84.0;
const LEFT: f64 = 104.0;
const TOP: f64 = 76.0;

#[derive(Debug, Clone)]
pub(crate) struct Matrix {
    pub(crate) labels: Vec<String>,
    pub(crate) values: Vec<Vec<Option<f64>>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Kind { Correlation, General }

#[derive(Debug, Clone, Copy, PartialEq)]
enum Format { Svg, SceneJson, NormalizedJson }

#[derive(Debug, Clone, Copy, PartialEq)]
enum InputFormat { Auto, Csv, Tsv, SceneJson }

#[derive(Debug)]
struct Options {
    input: String,
    config: Option<String>,
    output: Option<String>,
    format: Format,
    input_format: InputFormat,
    kind: Kind,
    method: String,
    title: String,
    incremental: bool,
}

fn usage() -> &'static str {
    "corrplot-ng — native correlation-matrix renderer

USAGE:
  corrplot-ng <COMMAND> [OPTIONS]

COMMANDS:
  init       Interactively create a publication-config v1 file
  render     Render a labelled matrix to SVG or scene-v1 JSON
  validate   Validate a labelled matrix without rendering
  inspect    Print detected matrix metadata
  presets    List built-in rendering presets
  capabilities Print machine-readable renderer capabilities
  render-batch Render a manifest of independent publication configs (supports --incremental)
  version    Print version and build information
  help       Show this help

RENDER OPTIONS:
  -i, --input PATH       Input CSV/TSV or scene-v1 JSON file; use - for stdin\n      --config PATH      Publication configuration v1 JSON file (render/validate)
  -o, --output PATH      Output file; default is stdout
      --format FORMAT    svg (default), scene-json, or normalized-json
      --input-format FMT  auto (default), csv, tsv, or scene-json
      --kind KIND        correlation (default) or general
      --method METHOD    circle (default) or number
      --title TEXT       Figure title\n      --incremental      Skip unchanged file outputs using a local cache

GLOBAL OPTIONS:
      --diagnostics json Emit one diagnostic-v1 JSON record on failure

INPUT FORMAT:
  A square labelled matrix. The first row contains an empty corner cell
  followed by column labels; each following row starts with its row label.
  Empty cells, NA, NaN, and null represent missing values.
  JSON input uses renderer-neutral scene-v1 documents (schema_version: 1).
  Auto detection uses the file extension, then content; use --input-format to override.

EXIT STATUS:
  0 success; 2 usage error; 1 input, validation, or rendering error.
"
}

fn fail(message: impl AsRef<str>, code: i32) -> ! {
    eprintln!("error: {}", message.as_ref());
    process::exit(code);
}

fn diagnostic_json(message: &str) -> String {
    let lower = message.to_lowercase();
    let (code, stage, path) = if lower.starts_with("renderer.output.invalid_options:") {
        ("renderer.output.invalid_options", "renderer", Some("output"))
    } else if lower.starts_with("renderer.output.invalid_buffer:") {
        ("renderer.output.invalid_buffer", "export", Some("output"))
    } else if lower.starts_with("renderer.output.format_mismatch:") {
        ("renderer.output.format_mismatch", "export", Some("output"))
    } else if lower.contains("non-finite") || lower.contains("must be finite") || lower.contains("nan") {
        ("input.matrix.non_finite_value", "input", Some("matrix"))
    } else if lower.contains("square") || lower.contains("dimensions") || lower.contains("matrix size") {
        ("input.matrix.invalid_shape", "input", Some("matrix"))
    } else if lower.contains("non-numeric") || lower.contains("must be numeric") || lower.contains("numeric value") || lower.contains("invalid number") {
        ("input.matrix.non_numeric", "input", Some("matrix"))
    } else if lower.contains("label") {
        ("input.matrix.invalid_label", "input", Some("labels"))
    } else if lower.contains("unsupported method") || lower.contains("unsupported feature") {
        ("renderer.unsupported_feature", "renderer", Some("method"))
    } else if lower.contains("write output") || lower.contains("cannot write output") {
        ("export.output.write_failed", "export", Some("output"))
    } else if lower.contains("config") || lower.contains("publication") {
        ("configuration.invalid_field", "configuration", None)
    } else if lower.contains("scene") {
        ("scene.invalid_content", "scene", None)
    } else {
        ("cli.operation.failed", "input", None)
    };
    let path_field = path.map(|value| format!(",\"path\":\"{}\"", value)).unwrap_or_default();
    let coordinates = message.find("matrix value at row ").and_then(|start| {
        let rest = &message[start + "matrix value at row ".len()..];
        let (row, rest) = rest.split_once(", column ")?;
        let column = rest.split(|ch: char| !ch.is_ascii_digit()).next()?;
        Some((row.parse::<usize>().ok()?, column.parse::<usize>().ok()?))
    });
    let coordinate_fields = coordinates.map(|(row, column)| {
        format!(",\"row\":{row},\"column\":{column}")
    }).unwrap_or_default();
    format!(
        "{{\"schema_version\":1,\"code\":\"{}\",\"severity\":\"error\",\"stage\":\"{}\",\"message\":\"{}\"{}{}}}",
        code, stage, json_escape(message), path_field, coordinate_fields
    )
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    if args.len() < 2 { return Err(usage().to_string()); }
    let mut input = None;
    let mut config = None;
    let mut output = None;
    let mut format = Format::Svg;
    let mut input_format = InputFormat::Auto;
    let mut kind = Kind::Correlation;
    let mut method = "circle".to_string();
    let mut title = "corrplot-ng".to_string();
    let mut incremental = false;
    let mut i = 2;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "-h" || arg == "--help" { return Err(usage().to_string()); }
        if arg == "--incremental" {
            incremental = true;
            i += 1;
            continue;
        }
        if i + 1 >= args.len() { return Err(format!("missing value for {arg}\n\n{}", usage())); }
        let value = args[i + 1].clone();
        match arg {
            "-i" | "--input" => input = Some(value),
            "--config" => config = Some(value),
            "-o" | "--output" => output = Some(value),
            "--format" => format = match value.as_str() {
                "svg" => Format::Svg,
                "scene-json" => Format::SceneJson,
                "normalized-json" => Format::NormalizedJson,
                _ => return Err(format!("unsupported format '{value}' (expected svg, scene-json, or normalized-json)")),
            },
            "--input-format" => input_format = match value.as_str() {
                "auto" => InputFormat::Auto,
                "csv" => InputFormat::Csv,
                "tsv" => InputFormat::Tsv,
                "scene-json" => InputFormat::SceneJson,
                _ => return Err(format!("unsupported input format '{value}' (expected auto, csv, tsv, or scene-json)")),
            },
            "--kind" => kind = match value.as_str() {
                "correlation" => Kind::Correlation,
                "general" => Kind::General,
                _ => return Err(format!("unsupported matrix kind '{value}' (expected correlation or general)")),
            },
            "--method" => {
                if value != "circle" && value != "number" {
                    return Err(format!("unsupported method '{value}' (supported: circle, number)"));
                }
                method = value;
            }
            "--title" => title = value,
            _ => return Err(format!("unknown option '{arg}'\n\n{}", usage())),
        }
        i += 2;
    }
    Ok(Options {
        input: input.unwrap_or_default(), config,
        output, format, input_format, kind, method, title, incremental,
    })
}

fn parse_csv_records(source: &str, delimiter: char) -> Result<Vec<Vec<String>>, String> {
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut chars = source.chars().peekable();
    let mut quoted = false;
    let mut after_quote = false;
    while let Some(ch) = chars.next() {
        if quoted {
            match ch {
                '"' => {
                    if chars.peek() == Some(&'"') { chars.next(); field.push('"'); }
                    else { quoted = false; after_quote = true; }
                }
                _ => field.push(ch),
            }
        } else if after_quote {
            match ch {
                c if c == delimiter => { row.push(std::mem::take(&mut field)); after_quote = false; }
                '\r' => { if chars.peek() == Some(&'\n') { chars.next(); } row.push(std::mem::take(&mut field)); records.push(std::mem::take(&mut row)); after_quote = false; }
                '\n' => { row.push(std::mem::take(&mut field)); records.push(std::mem::take(&mut row)); after_quote = false; }
                ' ' => {}
                _ => return Err("unexpected character after closing CSV quote".into()),
            }
        } else {
            match ch {
                '"' if field.is_empty() => quoted = true,
                c if c == delimiter => row.push(std::mem::take(&mut field)),
                '\r' => { if chars.peek() == Some(&'\n') { chars.next(); } row.push(std::mem::take(&mut field)); records.push(std::mem::take(&mut row)); }
                '\n' => { row.push(std::mem::take(&mut field)); records.push(std::mem::take(&mut row)); }
                _ => field.push(ch),
            }
        }
    }
    if quoted { return Err("unterminated quoted CSV field".into()); }
    if !field.is_empty() || !row.is_empty() || after_quote {
        row.push(field);
        records.push(row);
    }
    while records.last().map_or(false, |r| r.len() == 1 && r[0].trim().is_empty()) { records.pop(); }
    if records.is_empty() { return Err("input is empty".into()); }
    Ok(records)
}

fn infer_delimiter(source: &str) -> char {
    let first = source.lines().find(|line| !line.trim().is_empty()).unwrap_or("");
    if first.matches('\t').count() > first.matches(',').count() { '\t' } else { ',' }
}

pub(crate) fn parse_matrix(source: &str, kind: Kind) -> Result<Matrix, String> {
    parse_matrix_with_delimiter(source, kind, infer_delimiter(source))
}

pub(crate) fn parse_matrix_with_delimiter(source: &str, kind: Kind, delimiter: char) -> Result<Matrix, String> {
    if delimiter != ',' && delimiter != '\t' { return Err("matrix delimiter must be comma or tab".into()); }
    let records = parse_csv_records(source, delimiter)?;
    if records.len() < 2 { return Err("expected a header and at least one matrix row".into()); }
    let header = &records[0];
    if header.len() < 2 { return Err("header must contain an empty corner cell and column labels".into()); }
    if !header[0].trim().is_empty() && header[0].trim() != "label" {
        return Err("first header cell must be empty or 'label'".into());
    }
    let labels: Vec<String> = header[1..].to_vec();
    let n = labels.len();
    if records.len() != n + 1 {
        return Err(format!("matrix must be square: {} column labels but {} data rows", n, records.len() - 1));
    }
    let mut values = Vec::with_capacity(n);
    for (r, record) in records.iter().skip(1).enumerate() {
        if record.len() != n + 1 {
            return Err(format!("row {} has {} values; expected {n}", r + 1, record.len().saturating_sub(1)));
        }
        let label = record[0].as_str();
        if label != labels[r] { return Err(format!("row label '{}' at row {} does not match column label '{}'", label, r + 1, labels[r])); }
        let mut row = Vec::with_capacity(n);
        for (c, raw) in record.iter().skip(1).enumerate() {
            let value = match raw.trim() {
                "" | "NA" | "NaN" | "null" => None,
                text => {
                    let parsed: f64 = text.parse().map_err(|_| format!("invalid number at row {}, column '{}': '{}'", r + 1, labels[c], text))?;
                    Some(parsed)
                }
            };
            row.push(value);
        }
        values.push(row);
    }
    let flat_values: Vec<Option<f64>> = values.iter().flat_map(|row| row.iter().copied()).collect();
    let shared_kind = if kind == Kind::Correlation {
        MatrixKind::Correlation
    } else {
        MatrixKind::GeneralNumeric
    };
    validate_matrix(&labels, n, n, &flat_values, shared_kind)
        .map_err(|error| error.to_string())?;
    Ok(Matrix { labels, values })
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""), '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"), '\r' => out.push_str("\\r"), '\t' => out.push_str("\\t"),
            c if c < '\u{20}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn num(v: f64) -> String { if v == 0.0 { "0".into() } else { format!("{v}") } }

fn normalized_matrix_json(matrix: &Matrix, source_format: &str) -> String {
    let mut out = String::from("{\"schema_version\":1,\"kind\":\"normalized_matrix\",\"labels\":[");
    for (index, label) in matrix.labels.iter().enumerate() {
        if index > 0 { out.push(','); }
        out.push('"');
        out.push_str(&json_escape(label));
        out.push('"');
    }
    out.push_str("],\"values\":[");
    for (row_index, row) in matrix.values.iter().enumerate() {
        if row_index > 0 { out.push(','); }
        out.push('[');
        for (column_index, value) in row.iter().enumerate() {
            if column_index > 0 { out.push(','); }
            match value {
                Some(number) => out.push_str(&num(*number)),
                None => out.push_str("null"),
            }
        }
        out.push(']');
    }
    out.push_str(&format!(
        "],\"source\":{{\"format\":\"{}\",\"row_labels_present\":true,\"column_labels_present\":true}},\"source_indices\":[",
        source_format
    ));
    for index in 0..matrix.labels.len() {
        if index > 0 { out.push(','); }
        out.push_str(&index.to_string());
    }
    out.push_str("]}");
    out
}

fn normalized_source_format(format: InputFormat) -> &'static str {
    match format {
        InputFormat::Csv => "csv",
        InputFormat::Tsv => "tsv",
        InputFormat::SceneJson => "json",
        InputFormat::Auto => unreachable!("auto input format is resolved before serialization"),
    }
}

fn scene_json(matrix: &Matrix, title: &str, method: &str, kind: Kind) -> String {
    let n = matrix.labels.len();
    let width = LEFT + n as f64 * CELL + 24.0;
    let height = TOP + n as f64 * CELL + 64.0;
    let scene_kind = if kind == Kind::Correlation { "correlation_plot" } else { "general_matrix_plot" };
    let mut out = format!("{{\"schema_version\":1,\"kind\":\"{}\",\"width\":{},\"height\":{},\"labels\":[", scene_kind, num(width), num(height));
    for (i, label) in matrix.labels.iter().enumerate() {
        if i > 0 { out.push(','); }
        let center_y = TOP + i as f64 * CELL + CELL / 2.0;
        let center_x = LEFT + i as f64 * CELL + CELL / 2.0;
        out.push_str(&format!("{{\"text\":\"{}\",\"row_x\":92,\"row_y\":{},\"column_x\":{},\"column_y\":60}}",
            json_escape(label), num(center_y), num(center_x)));
    }
    out.push_str("],\"cells\":[");
    let mut first = true;
    for r in 0..n {
        for c in 0..n {
            if !first { out.push(','); } first = false;
            let x = LEFT + c as f64 * CELL;
            let y = TOP + r as f64 * CELL;
            let value = matrix.values[r][c].map(num).unwrap_or_else(|| "null".into());
            out.push_str(&format!("{{\"row\":{r},\"column\":{c},\"value\":{value},\"x\":{},\"y\":{},\"center_x\":{},\"center_y\":{},\"size\":84}}",
                num(x), num(y), num(x + CELL / 2.0), num(y + CELL / 2.0)));
        }
    }
    out.push_str(&format!("],\"annotation\":\"\",\"style\":{{\"positive_color\":\"#2166ac\",\"negative_color\":\"#b2182b\",\"zero_color\":\"#bdbdbd\",\"missing_fill\":\"#f2f2f2\",\"grid_color\":\"#e0e0e0\"}},\"title\":\"{}\",\"description\":\"Generated by corrplot-ng CLI\",\"glyph_method\":\"{}\",\"text_annotations\":[]}}", json_escape(title), method));
    out
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&apos;")
}

fn svg(matrix: &Matrix, title: &str, method: &str, kind: Kind) -> String {
    let n = matrix.labels.len();
    let width = LEFT + n as f64 * CELL + 24.0;
    let height = TOP + n as f64 * CELL + 64.0;
    let mut out = format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\">\n<title>{}</title>\n",
        num(width), num(height), num(width), num(height), xml_escape(title));
    out.push_str(&format!("<rect width=\"100%\" height=\"100%\" fill=\"white\"/>\n"));
    out.push_str(&format!("<text x=\"{}\" y=\"28\" font-family=\"sans-serif\" font-size=\"18\" font-weight=\"bold\">{}</text>\n",
        num(LEFT), xml_escape(title)));
    for (i, label) in matrix.labels.iter().enumerate() {
        let cy = TOP + i as f64 * CELL + CELL / 2.0;
        let cx = LEFT + i as f64 * CELL + CELL / 2.0;
        out.push_str(&format!("<text x=\"92\" y=\"{}\" text-anchor=\"end\" dominant-baseline=\"middle\" font-family=\"sans-serif\" font-size=\"12\">{}</text>\n",
            num(cy), xml_escape(label)));
        out.push_str(&format!("<text x=\"{}\" y=\"60\" text-anchor=\"middle\" font-family=\"sans-serif\" font-size=\"12\">{}</text>\n",
            num(cx), xml_escape(label)));
    }
    for r in 0..n {
        for c in 0..n {
            let x = LEFT + c as f64 * CELL;
            let y = TOP + r as f64 * CELL;
            let cx = x + CELL / 2.0;
            let cy = y + CELL / 2.0;
            let value = matrix.values[r][c];
            let fill = match value {
                None => "#f2f2f2",
                Some(v) if v > 0.0 => "#2166ac",
                Some(v) if v < 0.0 => "#b2182b",
                _ => "#bdbdbd",
            };
            out.push_str(&format!("<rect x=\"{}\" y=\"{}\" width=\"84\" height=\"84\" fill=\"{}\" stroke=\"#e0e0e0\"/>\n", num(x), num(y), fill));
            match value {
                None => out.push_str(&format!("<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"sans-serif\" font-size=\"11\" fill=\"#555\">NA</text>\n", num(cx), num(cy))),
                Some(v) if method == "number" => {
                    let shown = if kind == Kind::Correlation { format!("{v:.2}") } else { format!("{v:.3}") };
                    out.push_str(&format!("<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"sans-serif\" font-size=\"12\" fill=\"white\">{}</text>\n", num(cx), num(cy), shown));
                }
                Some(v) => {
                    let max_abs = matrix.values.iter().flatten().filter_map(|x| *x).map(f64::abs).fold(0.0_f64, f64::max).max(1.0);
                    let radius = if kind == Kind::Correlation { v.abs() * 34.0 + 2.0 } else { (v.abs() / max_abs) * 36.0 + 2.0 };
                    out.push_str(&format!("<circle cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>\n", num(cx), num(cy), num(radius), fill));
                }
            }
        }
    }
    out.push_str("</svg>\n");
    out
}

fn detect_input_format(path: &str, source: &str, requested: InputFormat) -> InputFormat {
    if requested != InputFormat::Auto { return requested; }
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".json") || source.trim_start().starts_with('{') { return InputFormat::SceneJson; }
    if lower.ends_with(".tsv") { return InputFormat::Tsv; }
    if lower.ends_with(".csv") { return InputFormat::Csv; }
    let first = source.lines().find(|line| !line.trim().is_empty()).unwrap_or("");
    if first.matches('\t').count() > first.matches(',').count() { InputFormat::Tsv } else { InputFormat::Csv }
}

fn read_input(path: &str) -> Result<String, String> {
    if path == "-" {
        let mut s = String::new();
        io::stdin().read_to_string(&mut s).map_err(|e| format!("cannot read stdin as UTF-8 text: {e}"))?;
        Ok(s)
    } else {
        fs::read_to_string(path).map_err(|e| format!("cannot read '{path}' as UTF-8 text: {e}"))
    }
}

fn write_output(path: Option<&str>, content: &str) -> Result<(), String> {
    match path {
        Some("-") | None => {
            let mut stdout = io::stdout().lock();
            stdout.write_all(content.as_bytes()).map_err(|e| format!("cannot write stdout: {e}"))
        }
        Some(path) => cache::write_atomic(Path::new(path), content.as_bytes())
            .map(|_| ())
            .map_err(|error| format!("cannot write '{path}': {error}")),
    }
}

fn run() -> Result<(), String> {
    let mut args: Vec<String> = env::args().collect();
    let diagnostics_json = args.iter().any(|arg| arg == "--diagnostics" || arg == "--diagnostics=json");
    let mut index = 1;
    while index < args.len() {
        if args[index] == "--diagnostics" {
            if index + 1 >= args.len() || args[index + 1] != "json" {
                return Err("--diagnostics currently accepts only 'json'".into());
            }
            args.drain(index..=index + 1);
        } else if args[index] == "--diagnostics=json" {
            args.remove(index);
        } else {
            index += 1;
        }
    }
    if args.len() < 2 || args[1] == "help" || args[1] == "--help" || args[1] == "-h" {
        print!("{}", usage());
        return Ok(());
    }
    match args[1].as_str() {
        "init" => wizard::run(&args[2..])?,
        "version" => println!("corrplot-ng {VERSION} (Rust {})", env!("CARGO_PKG_VERSION")),
        "presets" => println!("default\tCircle glyphs with sign-based colors\nnumber\tNumeric values in cells"),
        "capabilities" => print!("{RENDERER_CAPABILITIES}"),
        "render-batch" => run_batch(&args[2..], diagnostics_json)?,
        "render" | "validate" | "inspect" => {
            let opts = parse_args(&args)?;
            if opts.incremental && args[1] != "render" {
                return Err("--incremental is available only for render".into());
            }
            if let Some(config_path) = opts.config.as_deref() {
                let destination = match opts.format {
                    Format::Svg => opts.output.clone().or(Some(publication::Publication::configured_output_path(config_path)?.to_string_lossy().into_owned())),
                    Format::SceneJson | Format::NormalizedJson => opts.output.clone(),
                };
                let incremental_key = if opts.incremental {
                    let destination = destination.as_deref().filter(|path| *path != "-")
                        .ok_or("--incremental requires a file output path via --output or config output.path")?;
                    let mut dependencies = vec![PathBuf::from(config_path)];
                    dependencies.extend(publication::Publication::input_files(config_path)?);
                    let key = cache::fingerprint(
                        &dependencies,
                        &["publication-config-v1", VERSION, RENDERER_CACHE_VERSION,
                          match opts.format { Format::Svg => "svg", Format::SceneJson => "scene-json", Format::NormalizedJson => "normalized-json" },
                          &opts.title],
                    )?;
                    let output_path = PathBuf::from(destination);
                    if cache::is_fresh(&output_path, &key) {
                        eprintln!("unchanged: {}", output_path.display());
                        return Ok(());
                    }
                    Some((output_path, key))
                } else {
                    None
                };
                let publication = publication::Publication::load(config_path)?;
                match args[1].as_str() {
                    "validate" => println!("valid: publication-config v1; {} x {} {} matrix; ordering={}; display={}",
                        publication.labels.len(), publication.labels.len(), publication.kind, publication.ordering, publication.display),
                    "inspect" => println!("input format: publication-config v1\nkind: {}\nrows: {}\ncolumns: {}\nordering: {}\ndisplay: {}",
                        publication.kind, publication.labels.len(), publication.labels.len(), publication.ordering, publication.display),
                    "render" => {
                        let content = match opts.format {
                            Format::Svg => match publication::PublicationSvgRenderer.render(
                                &publication::PublicationRenderInput {
                                    publication: &publication,
                                    title: &opts.title,
                                },
                                &publication.svg_output_options()?,
                            ).map_err(|error| error.to_string())? {
                                RenderOutput::Svg(svg) => svg,
                                _ => return Err("publication SVG renderer returned a non-SVG output".into()),
                            },
                            Format::SceneJson => publication.scene_json(&opts.title),
                            Format::NormalizedJson => return Err("normalized-json output is available for direct matrix inputs, not publication configs".into()),
                        };
                        let destination = match opts.format {
                            Format::Svg => opts.output.as_deref().or(Some(publication.output.as_str())),
                            Format::SceneJson | Format::NormalizedJson => opts.output.as_deref(),
                        };
                        if opts.incremental && destination.is_none() {
                            return Err("--incremental requires a file output path via --output or config output.path".into());
                        }
                        write_output(destination, &content)?;
                        if let Some((output_path, key)) = incremental_key.as_ref() {
                            cache::store_fingerprint(output_path, key)?;
                        }
                    }
                    _ => unreachable!(),
                }
                return Ok(());
            }
            if opts.input.is_empty() { return Err(format!("{} requires --input PATH or --config PATH", args[1])); }
            if opts.incremental && args[1] != "render" {
                return Err("--incremental is available only for render".into());
            }
            let incremental_key = if opts.incremental {
                if opts.input == "-" {
                    return Err("--incremental requires a file input; stdin cannot be cached".into());
                }
                let destination = opts.output.as_deref().filter(|path| *path != "-")
                    .ok_or("--incremental requires a file output path via --output")?;
                let dependencies = vec![PathBuf::from(&opts.input)];
                let key = cache::fingerprint(
                    &dependencies,
                    &["matrix-input", VERSION, RENDERER_CACHE_VERSION,
                      match opts.format { Format::Svg => "svg", Format::SceneJson => "scene-json", Format::NormalizedJson => "normalized-json" },
                      match opts.input_format { InputFormat::Auto => "auto", InputFormat::Csv => "csv", InputFormat::Tsv => "tsv", InputFormat::SceneJson => "scene-json" },
                      match opts.kind { Kind::Correlation => "correlation", Kind::General => "general" },
                      &opts.method, &opts.title],
                )?;
                let output_path = PathBuf::from(destination);
                if cache::is_fresh(&output_path, &key) {
                    eprintln!("unchanged: {}", output_path.display());
                    return Ok(());
                }
                Some((output_path, key))
            } else {
                None
            };
            let source = read_input(&opts.input)?;
            let input_format = detect_input_format(&opts.input, &source, opts.input_format);
            let (matrix, kind) = match input_format {
                InputFormat::Csv => (parse_matrix_with_delimiter(&source, opts.kind, ',')?, opts.kind),
                InputFormat::Tsv => (parse_matrix_with_delimiter(&source, opts.kind, '\t')?, opts.kind),
                InputFormat::SceneJson => publication::parse_scene_matrix(&source)?,
                InputFormat::Auto => unreachable!("auto input format is resolved before parsing"),
            };
            let input_format_label = match input_format {
                InputFormat::Csv => "csv",
                InputFormat::Tsv => "tsv",
                InputFormat::SceneJson => "scene-v1 JSON",
                InputFormat::Auto => unreachable!("auto input format is resolved before reporting"),
            };
            match args[1].as_str() {
                "validate" => println!("valid: {} x {} matrix ({} missing values)", matrix.labels.len(), matrix.labels.len(), matrix.values.iter().flatten().filter(|v| v.is_none()).count()),
                "inspect" => {
                    let all: Vec<f64> = matrix.values.iter().flatten().filter_map(|v| *v).collect();
                    let min = all.iter().copied().fold(f64::INFINITY, f64::min);
                    let max = all.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                    println!("input format: {}\nkind: {}\nrows: {}\ncolumns: {}\nmissing: {}\nminimum: {}\nmaximum: {}",
                        input_format_label,
                        if kind == Kind::Correlation { "correlation" } else { "general" },
                        matrix.labels.len(), matrix.labels.len(),
                        matrix.values.iter().flatten().filter(|v| v.is_none()).count(), num(min), num(max));
                }
                "render" => {
                    let output = match opts.format {
                        Format::Svg => svg(&matrix, &opts.title, &opts.method, kind),
                        Format::SceneJson => scene_json(&matrix, &opts.title, &opts.method, kind),
                        Format::NormalizedJson => normalized_matrix_json(&matrix, normalized_source_format(input_format)),
                    };
                    write_output(opts.output.as_deref(), &output)?;
                    if let Some((output_path, key)) = incremental_key.as_ref() {
                        cache::store_fingerprint(output_path, key)?;
                    }
                }
                _ => unreachable!(),
            }
        }
        other => return Err(format!("unknown command '{other}'\n\n{}", usage())),
    }
    Ok(())
}


fn process_batch(manifest_path: &str, output_dir: &str, incremental: bool) -> Result<(Vec<String>, Vec<(String, String)>), String> {
    let manifest = Path::new(manifest_path);
    let base = manifest.parent().unwrap_or_else(|| Path::new("."));
    let source = fs::read_to_string(manifest_path)
        .map_err(|error| format!("cannot read batch manifest '{manifest_path}': {error}"))?;
    fs::create_dir_all(output_dir)
        .map_err(|error| format!("cannot create output directory '{output_dir}': {error}"))?;
    let mut successes = Vec::new();
    let mut diagnostics = Vec::new();
    let mut item_index = 0usize;

    for line in source.lines() {
        let entry = line.trim();
        if entry.is_empty() || entry.starts_with('#') {
            continue;
        }
        item_index += 1;
        let config_path = if Path::new(entry).is_absolute() {
            Path::new(entry).to_path_buf()
        } else {
            base.join(entry)
        };
        let stem = config_path.file_stem().and_then(|value| value.to_str()).unwrap_or("figure");
        let safe_stem: String = stem.chars().map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '-' }).collect();
        let output_path = Path::new(output_dir).join(format!("{item_index:03}-{safe_stem}.svg"));
        let result = (|| {
            let config_string = config_path.to_string_lossy();
            let key = if incremental {
                let mut dependencies = vec![config_path.clone()];
                dependencies.extend(publication::Publication::input_files(&config_string)?);
                Some(cache::fingerprint(
                    &dependencies,
                    &["publication-config-v1", VERSION, RENDERER_CACHE_VERSION, "svg", stem],
                )?)
            } else {
                None
            };
            if let Some(key) = key.as_deref() {
                if cache::is_fresh(&output_path, key) {
                    eprintln!("unchanged: {}", output_path.display());
                    return Ok::<(), String>(());
                }
            }
            let publication = publication::Publication::load(&config_string)?;
            let svg = publication.svg(stem);
            cache::write_atomic(&output_path, svg.as_bytes())
                .map_err(|error| format!("cannot write output '{}': {error}", output_path.display()))?;
            if let Some(key) = key.as_deref() {
                cache::store_fingerprint(&output_path, key)?;
            }
            Ok::<(), String>(())
        })();

        match result {
            Ok(()) => successes.push(format!(
                "{{\"input\":\"{}\",\"output\":\"{}\"}}",
                json_escape(entry),
                json_escape(&output_path.to_string_lossy())
            )),
            Err(message) => {
                let mut diagnostic = diagnostic_json(&message);
                diagnostic.pop();
                diagnostic.push_str(&format!(
                    ",\"details\":{{\"manifest_index\":{},\"input\":\"{}\"}}}}",
                    item_index,
                    json_escape(entry)
                ));
                diagnostics.push((diagnostic, message));
            }
        }
    }
    if item_index == 0 {
        return Err("batch manifest contains no input entries".into());
    }
    Ok((successes, diagnostics))
}

fn run_batch(args: &[String], diagnostics_json: bool) -> Result<(), String> {
    let mut manifest = None;
    let mut output_dir = None;
    let mut incremental = false;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--incremental" {
            incremental = true;
            index += 1;
            continue;
        }
        match args[index].as_str() {
            "--manifest" => {
                index += 1;
                manifest = args.get(index).cloned();
            }
            "--output-dir" => {
                index += 1;
                output_dir = args.get(index).cloned();
            }
            option => return Err(format!("unknown render-batch option '{option}'")),
        }
        index += 1;
    }
    let manifest = manifest.ok_or("render-batch requires --manifest PATH")?;
    let output_dir = output_dir.ok_or("render-batch requires --output-dir DIR")?;
    let (successes, diagnostics) = process_batch(&manifest, &output_dir, incremental)?;

    if diagnostics_json {
        println!(
            "{{\"schema_version\":1,\"successes\":[{}],\"diagnostics\":[{}]}}",
            successes.join(","),
            diagnostics.iter().map(|(diagnostic, _)| diagnostic.as_str()).collect::<Vec<_>>().join(",")
        );
    } else {
        for success in &successes {
            println!("rendered {success}");
        }
        for (_diagnostic, message) in &diagnostics {
            eprintln!("error: {message}");
        }
    }
    if !diagnostics.is_empty() {
        process::exit(1);
    }
    Ok(())
}

fn main() {
    let diagnostics_json = env::args().any(|arg| arg == "--diagnostics" || arg == "--diagnostics=json");
    if let Err(message) = run() {
        let usage_error = message.contains("requires --input") || message.contains("requires --input PATH or --config PATH") || message.contains("unknown option") ||
            message.contains("unknown command") || message.contains("unsupported format") ||
            message.contains("unsupported matrix kind") || message.contains("unsupported input format") || message.contains("unsupported method") ||
            message.contains("missing value for") || message.contains("unknown init option") || message.contains("requires an interactive terminal");
        let code = if usage_error { 2 } else { 1 };
        if diagnostics_json {
            println!("{}", diagnostic_json(&message));
            process::exit(code);
        }
        fail(message, code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = ",alpha,beta\nalpha,1,0.5\nbeta,0.5,1\n";

    #[test]
    fn serializes_stable_input_diagnostics_as_json() {
        let json = diagnostic_json("input must be a non-empty square matrix");
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"code\":\"input.matrix.invalid_shape\""));
        assert!(json.contains("\"stage\":\"input\""));
        let escaped = diagnostic_json("bad \"label\"\nvalue");
        assert!(escaped.contains("bad \\\"label\\\"\\nvalue"));
    }

    #[test]
    fn exposes_machine_readable_renderer_capabilities() {
        assert!(RENDERER_CAPABILITIES.contains("\"renderer\""));
        assert!(RENDERER_CAPABILITIES.contains("\"cell_methods\""));
        assert!(RENDERER_CAPABILITIES.contains("\"circle\""));
    }

    #[test]
    fn batch_render_keeps_successes_and_reports_each_independent_failure() {
        let temp = env::temp_dir().join(format!("corrplot-ng-batch-{}", process::id()));
        let _ = fs::remove_dir_all(&temp);
        fs::create_dir_all(&temp).unwrap();
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let valid_config = repo_root.join("prototypes/publication-config-v1/examples/basic.json");
        let manifest = temp.join("manifest.txt");
        fs::write(
            &manifest,
            format!("{}\nmissing-one.json\nmissing-two.json\n", valid_config.display()),
        ).unwrap();
        let output_dir = temp.join("out");

        let (successes, diagnostics) = process_batch(
            manifest.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            false,
        ).unwrap();

        assert_eq!(successes.len(), 1);
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics[0].0.contains("\"code\":\"configuration.invalid_field\""));
        assert!(diagnostics[0].0.contains("\"manifest_index\":2"));
        assert!(output_dir.join("001-basic.svg").exists());
        fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn current_matrix_svg_output_is_deterministic_and_escapes_titles() {
        let matrix = parse_matrix(SAMPLE, Kind::Correlation).unwrap();
        let rendered = svg(&matrix, "R&D <metrics>", "number", Kind::Correlation);
        assert!(rendered.starts_with("<svg xmlns="));
        assert!(rendered.contains("<title>R&amp;D &lt;metrics&gt;</title>"));
        assert_eq!(rendered, svg(&matrix, "R&D <metrics>", "number", Kind::Correlation));
    }

    #[test]
    fn parses_square_correlation_matrix() {
        let m = parse_matrix(SAMPLE, Kind::Correlation).unwrap();
        assert_eq!(m.labels, vec!["alpha", "beta"]);
        assert_eq!(m.values[0][1], Some(0.5));
    }

    #[test]
    fn preserves_missing_values() {
        let m = parse_matrix(",a,b\na,1,NA\nb,null,1\n", Kind::Correlation).unwrap();
        assert_eq!(m.values[0][1], None);
    }

    #[test]
    fn accepts_quoted_labels_and_escaped_quotes() {
        let m = parse_matrix(",\"a, one\",\"b\"\"two\"\n\"a, one\",1,0.2\n\"b\"\"two\",0.2,1\n", Kind::Correlation).unwrap();
        assert_eq!(m.labels[0], "a, one");
        assert_eq!(m.labels[1], "b\"two");
    }

    #[test]
    fn rejects_asymmetric_correlation() {
        assert!(parse_matrix(",a,b\na,1,0.2\nb,0.3,1\n", Kind::Correlation).unwrap_err().contains("not symmetric"));
    }

    #[test]
    fn general_matrix_allows_values_outside_unit_interval() {
        let m = parse_matrix(",a,b\na,10,-4\nb,2,8\n", Kind::General).unwrap();
        assert_eq!(m.values[0][0], Some(10.0));
    }

    #[test]
    fn escapes_json_and_xml_text() {
        assert_eq!(json_escape("a\"b"), "a\\\"b");
        assert_eq!(xml_escape("<&>"), "&lt;&amp;&gt;");
    }
}
