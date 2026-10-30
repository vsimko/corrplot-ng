use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::process;

mod publication;
mod wizard;

const VERSION: &str = env!("CARGO_PKG_VERSION");
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
enum Format { Svg, SceneJson }

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
  version    Print version and build information
  help       Show this help

RENDER OPTIONS:
  -i, --input PATH       Input CSV/TSV or scene-v1 JSON file; use - for stdin\n      --config PATH      Publication configuration v1 JSON file (render/validate)
  -o, --output PATH      Output file; default is stdout
      --format FORMAT    svg (default) or scene-json (output format)
      --input-format FMT  auto (default), csv, tsv, or scene-json
      --kind KIND        correlation (default) or general
      --method METHOD    circle (default) or number
      --title TEXT       Figure title

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
    let mut i = 2;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "-h" || arg == "--help" { return Err(usage().to_string()); }
        if i + 1 >= args.len() { return Err(format!("missing value for {arg}\n\n{}", usage())); }
        let value = args[i + 1].clone();
        match arg {
            "-i" | "--input" => input = Some(value),
            "--config" => config = Some(value),
            "-o" | "--output" => output = Some(value),
            "--format" => format = match value.as_str() {
                "svg" => Format::Svg,
                "scene-json" => Format::SceneJson,
                _ => return Err(format!("unsupported format '{value}' (expected svg or scene-json)")),
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
        output, format, input_format, kind, method, title,
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
    let labels: Vec<String> = header[1..].iter().map(|s| s.trim().to_string()).collect();
    let n = labels.len();
    if labels.iter().any(|s| s.is_empty()) { return Err("column labels must not be empty".into()); }
    let mut seen = HashSet::new();
    for label in &labels {
        if !seen.insert(label) { return Err(format!("duplicate column label '{label}'")); }
    }
    if records.len() != n + 1 {
        return Err(format!("matrix must be square: {} column labels but {} data rows", n, records.len() - 1));
    }
    let mut values = Vec::with_capacity(n);
    let mut row_seen = HashSet::new();
    for (r, record) in records.iter().skip(1).enumerate() {
        if record.len() != n + 1 {
            return Err(format!("row {} has {} values; expected {n}", r + 1, record.len().saturating_sub(1)));
        }
        let label = record[0].trim();
        if label != labels[r] { return Err(format!("row label '{}' at row {} does not match column label '{}'", label, r + 1, labels[r])); }
        if !row_seen.insert(label) { return Err(format!("duplicate row label '{label}'")); }
        let mut row = Vec::with_capacity(n);
        for (c, raw) in record.iter().skip(1).enumerate() {
            let value = match raw.trim() {
                "" | "NA" | "NaN" | "null" => None,
                text => {
                    let parsed: f64 = text.parse().map_err(|_| format!("invalid number at row {}, column '{}': '{}'", r + 1, labels[c], text))?;
                    if !parsed.is_finite() { return Err(format!("non-finite value at row {}, column '{}'", r + 1, labels[c])); }
                    Some(parsed)
                }
            };
            if kind == Kind::Correlation {
                if let Some(v) = value {
                    if !(-1.0..=1.0).contains(&v) { return Err(format!("correlation at row {}, column '{}' is outside [-1, 1]", r + 1, labels[c])); }
                }
            }
            row.push(value);
        }
        values.push(row);
    }
    if kind == Kind::Correlation {
        for r in 0..n {
            for c in 0..n {
                if match (values[r][c], values[c][r]) { (Some(a), Some(b)) => (a - b).abs() > 1e-8, (None, None) => false, _ => true } {
                    return Err(format!("correlation matrix is not symmetric at ('{}', '{}')", labels[r], labels[c]));
                }
            }
        }
        for i in 0..n {
            if values[i][i].map_or(true, |v| (v - 1.0).abs() > 1e-8) {
                return Err(format!("correlation diagonal for '{}' must equal 1", labels[i]));
            }
        }
    }
    if values.iter().flatten().all(Option::is_none) { return Err("matrix contains no numeric values".into()); }
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
        Some(path) => fs::write(path, content).map_err(|e| format!("cannot write '{path}': {e}")),
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "help" || args[1] == "--help" || args[1] == "-h" {
        print!("{}", usage());
        return Ok(());
    }
    match args[1].as_str() {
        "init" => wizard::run(&args[2..])?,
        "version" => println!("corrplot-ng {VERSION} (Rust {})", env!("CARGO_PKG_VERSION")),
        "presets" => println!("default\tCircle glyphs with sign-based colors\nnumber\tNumeric values in cells"),
        "render" | "validate" | "inspect" => {
            let opts = parse_args(&args)?;
            if let Some(config_path) = opts.config.as_deref() {
                let publication = publication::Publication::load(config_path)?;
                match args[1].as_str() {
                    "validate" => println!("valid: publication-config v1; {} x {} {} matrix; ordering={}; display={}",
                        publication.labels.len(), publication.labels.len(), publication.kind, publication.ordering, publication.display),
                    "inspect" => println!("input format: publication-config v1\nkind: {}\nrows: {}\ncolumns: {}\nordering: {}\ndisplay: {}",
                        publication.kind, publication.labels.len(), publication.labels.len(), publication.ordering, publication.display),
                    "render" => {
                        let content = match opts.format {
                            Format::Svg => publication.svg(&opts.title),
                            Format::SceneJson => publication.scene_json(&opts.title),
                        };
                        let destination = match opts.format {
                            Format::Svg => opts.output.as_deref().or(Some(publication.output.as_str())),
                            Format::SceneJson => opts.output.as_deref(),
                        };
                        write_output(destination, &content)?;
                    }
                    _ => unreachable!(),
                }
                return Ok(());
            }
            if opts.input.is_empty() { return Err(format!("{} requires --input PATH or --config PATH", args[1])); }
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
                    };
                    write_output(opts.output.as_deref(), &output)?;
                }
                _ => unreachable!(),
            }
        }
        other => return Err(format!("unknown command '{other}'\n\n{}", usage())),
    }
    Ok(())
}

fn main() {
    if let Err(message) = run() {
        let usage_error = message.contains("requires --input") || message.contains("requires --input PATH or --config PATH") || message.contains("unknown option") ||
            message.contains("unknown command") || message.contains("unsupported format") ||
            message.contains("unsupported matrix kind") || message.contains("unsupported input format") || message.contains("unsupported method") ||
            message.contains("missing value for") || message.contains("unknown init option") || message.contains("requires an interactive terminal");
        fail(message, if usage_error { 2 } else { 1 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = ",alpha,beta\nalpha,1,0.5\nbeta,0.5,1\n";

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
