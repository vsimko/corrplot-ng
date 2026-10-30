//! Opt-in interactive configuration wizard.
//!
//! This module has no external dependencies. It is called only by the explicit
//! `init` command; ordinary CLI commands never construct or execute wizard state.

use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

fn prompt(label: &str, default: Option<&str>) -> Result<String, String> {
    let suffix = default.map(|value| format!(" [{value}]")).unwrap_or_default();
    print!("{label}{suffix}: ");
    io::stdout().flush().map_err(|e| format!("cannot write prompt: {e}"))?;
    let mut value = String::new();
    io::stdin().read_line(&mut value).map_err(|e| format!("cannot read response: {e}"))?;
    let value = value.trim();
    if value.is_empty() {
        return default.map(str::to_owned).ok_or_else(|| format!("{label} is required"));
    }
    Ok(value.to_owned())
}

fn choose(label: &str, choices: &[&str], default: &str) -> Result<String, String> {
    loop {
        let value = prompt(&format!("{label} ({})", choices.join("/")), Some(default))?;
        if choices.contains(&value.as_str()) { return Ok(value); }
        eprintln!("Please choose one of: {}", choices.join(", "));
    }
}

fn yes_no(label: &str, default: bool) -> Result<bool, String> {
    let value = prompt(label, Some(if default { "y" } else { "n" }))?;
    match value.to_ascii_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => Err(format!("{label}: enter y or n")),
    }
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\u{20}' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn build_config(
    input: &str, delimiter: &str, kind: &str, method: &str, glyph: &str, missing: &str,
    display: &str, ordering: &str, palette: &str, font_family: &str, show_values: bool, output: &str,
) -> String {
    let matrix = if kind == "correlation" {
        format!("{{ \"kind\": \"correlation\", \"method\": {}, \"missing\": {} }}",
            json_string(method), json_string(missing))
    } else {
        format!("{{ \"kind\": \"general_numeric\", \"missing\": {} }}", json_string(missing))
    };
    let (negative, neutral, positive, missing_color) = match palette {
        "blue-red" => ("#b2182b", "#f7f7f7", "#2166ac", "#d9d9d9"),
        "purple-green" => ("#762a83", "#f7f7f7", "#1b7837", "#d9d9d9"),
        "orange-teal" => ("#d95f02", "#f7f7f7", "#1b9e77", "#d9d9d9"),
        _ => unreachable!("palette is selected from the wizard's fixed choices"),
    };
    format!(r##"{{
  "config_version": 1,
  "input": {{
    "kind": "matrix_csv",
    "path": {},
    "delimiter": {},
    "header": true,
    "row_labels": true
  }},
  "matrix": {},
  "plot": {{
    "method": {},
    "display": {},
    "ordering": {},
    "show_values": {},
    "significance": {{ "enabled": false, "p_values": null, "thresholds": [0.001, 0.01, 0.05] }}
  }},
  "colors": {{ "negative": {}, "neutral": {}, "positive": {}, "missing": {} }},
  "dimensions": {{ "width": 160, "height": 145, "unit": "mm" }},
  "typography": {{ "family": {}, "base_size": 9 }},
  "output": {{ "format": "svg", "path": {} }}
}}
"##,
        json_string(input), json_string(delimiter), matrix, json_string(glyph),
        json_string(display), json_string(ordering), json_string(negative), json_string(neutral),
        json_string(positive), json_string(missing_color), json_string(font_family), show_values, json_string(output))
}

fn temporary_path(config_path: &Path) -> PathBuf {
    let parent = config_path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    parent.join(format!(".corrplot-ng-init-{}.tmp", std::process::id()))
}

pub(crate) fn run(args: &[String]) -> Result<(), String> {
    let mut output_config = "corrplot.json".to_owned();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                output_config = args.get(i).ok_or("missing value for --output")?.clone();
            }
            "--help" | "-h" => {
                println!("USAGE: corrplot-ng init [--output PATH]\nCreate a publication-config v1 file interactively.");
                return Ok(());
            }
            other => return Err(format!("unknown init option '{other}' (supported: --output PATH, --help)")),
        }
        i += 1;
    }

    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("init requires an interactive terminal (TTY); it cannot run in scripts or CI".into());
    }
    println!("corrplot-ng configuration wizard");
    println!("Enter paths relative to the configuration file's directory when using relative paths.");
    let input = prompt("Input labelled CSV/TSV path", None)?;
    if input == "-" { return Err("init requires a file path; stdin is not supported for reusable configurations".into()); }
    let delimiter = choose("Delimiter", &[",", "tab"], ",")?;
    let delimiter = if delimiter == "tab" { "\t" } else { "," };
    let kind = choose("Matrix semantics", &["correlation", "general_numeric"], "correlation")?;
    let method = if kind == "correlation" {
        choose("Correlation method", &["pearson", "spearman", "kendall"], "pearson")?
    } else { "pearson".to_owned() };
    let missing = choose("Missing-value policy", &["preserve", "error"], "preserve")?;
    let plot_method = choose("Cell encoding", &["circle", "number"], "circle")?;
    let display = choose("Displayed matrix region", &["full", "upper", "lower"], "full")?;
    let ordering = if kind == "correlation" {
        choose("Variable ordering", &["original", "alphabetical", "aoe", "hclust"], "original")?
    } else {
        choose("Variable ordering", &["original", "alphabetical"], "original")?
    };
    let palette = choose("Color palette", &["blue-red", "purple-green", "orange-teal"], "blue-red")?;
    let font_family = choose("Label font family", &["sans-serif", "serif"], "sans-serif")?;
    let show_values = yes_no("Show numeric values in cells?", false)?;
    let output = prompt("SVG output path", Some("corrplot.svg"))?;

    let config = build_config(&input, delimiter, &kind, &method, &plot_method, &missing, &display, &ordering, &palette, &font_family, show_values, &output);
    let destination = PathBuf::from(&output_config);
    let parent = destination.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() { return Err(format!("configuration directory '{}' does not exist", parent.display())); }
    let temp = temporary_path(&destination);
    std::fs::write(&temp, &config).map_err(|e| format!("cannot stage configuration for validation: {e}"))?;
    let validation = crate::publication::Publication::load(temp.to_str().ok_or("configuration path is not valid UTF-8")?);
    let _ = std::fs::remove_file(&temp);
    validation.map_err(|e| format!("generated configuration failed validation: {e}"))?;

    println!("\nConfiguration preview ({})", output_config);
    println!("{config}");
    if !yes_no("Write this configuration file?", true)? { println!("Cancelled; no file written."); return Ok(()); }
    if destination.exists() && !yes_no(&format!("'{}' exists. Overwrite?", destination.display()), false)? {
        println!("Cancelled; existing file left unchanged.");
        return Ok(());
    }
    std::fs::write(&destination, config).map_err(|e| format!("cannot write '{}': {e}", destination.display()))?;
    println!("Wrote valid publication-config v1 to {}", destination.display());
    println!("Next: corrplot-ng validate --config {}", destination.display());
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_configuration_uses_selected_glyph_and_validates() {
        let directory = std::env::temp_dir().join(format!("corrplot-ng-wizard-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let matrix = directory.join("matrix.csv");
        let config = directory.join("figure.json");
        std::fs::write(&matrix, ",alpha,beta\nalpha,1,0.4\nbeta,0.4,1\n").unwrap();
        let contents = build_config(
            "matrix.csv", ",", "correlation", "spearman", "number",
            "preserve", "upper", "alphabetical", "purple-green", "serif", true, "figure.svg",
        );
        std::fs::write(&config, &contents).unwrap();
        let parsed = crate::publication::Publication::load(config.to_str().unwrap()).unwrap();
        assert_eq!(parsed.method, "number");
        assert_eq!(parsed.ordering, "alphabetical");
        assert_eq!(parsed.positive, "#1b7837");
        assert_eq!(parsed.family, "serif");
        assert!(parsed.show_values);
        std::fs::remove_dir_all(directory).ok();
    }

    #[test]
    fn tab_delimiter_is_encoded_as_a_json_tab_escape() {
        let config = build_config(
            "matrix.tsv", "\t", "correlation", "pearson", "circle",
            "preserve", "full", "original", "blue-red", "sans-serif", false, "figure.svg",
        );
        assert!(config.contains(r#""delimiter": "\t""#));
    }
}
