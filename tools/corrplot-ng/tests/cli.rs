use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn fixture(name: &str, contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("corrplot-ng-{}-{name}.csv", std::process::id()));
    fs::write(&path, contents).unwrap();
    path
}

#[test]
fn version_command_runs_as_native_binary() {
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng")).arg("version").output().unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("corrplot-ng 0.1.0"));
}

#[test]
fn validate_and_render_commands_work() {
    let path = fixture("valid", ",alpha,beta\nalpha,1,0.5\nbeta,0.5,1\n");
    let binary = env!("CARGO_BIN_EXE_corrplot-ng");
    let validated = Command::new(binary).args(["validate", "--input"]).arg(&path).output().unwrap();
    assert!(validated.status.success(), "{}", String::from_utf8_lossy(&validated.stderr));
    assert!(String::from_utf8_lossy(&validated.stdout).contains("2 x 2 matrix"));

    let rendered = Command::new(binary).args(["render", "--input"]).arg(&path).args(["--format", "scene-json"]).output().unwrap();
    assert!(rendered.status.success(), "{}", String::from_utf8_lossy(&rendered.stderr));
    let json = String::from_utf8(rendered.stdout).unwrap();
    assert!(json.contains(r#""schema_version":1"#));
    assert!(json.contains(r#""kind":"correlation_plot""#));
    fs::remove_file(path).ok();
}

#[test]
fn general_matrix_scene_keeps_general_semantics() {
    let path = fixture("general", ",x,y\nx,10,-4\ny,2,8\n");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&path)
        .args(["--kind", "general", "--format", "scene-json"]).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains(r#""kind":"general_matrix_plot""#));
    assert!(json.contains(r#""value":10"#));
    fs::remove_file(path).ok();
}

#[test]
fn invalid_matrix_returns_nonzero() {
    let path = fixture("invalid", ",a,b\na,1,0.2\nb,0.4,1\n");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng")).args(["validate", "--input"]).arg(&path).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not symmetric"));
    fs::remove_file(path).ok();
}


#[test]
fn publication_config_v1_validates_and_renders_svg() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../prototypes/publication-config-v1/examples/basic.json");
    let binary = env!("CARGO_BIN_EXE_corrplot-ng");
    let validated = Command::new(binary).args(["validate", "--config"]).arg(&root).output().unwrap();
    assert!(validated.status.success(), "{}", String::from_utf8_lossy(&validated.stderr));
    assert!(String::from_utf8_lossy(&validated.stdout).contains("publication-config v1"));

    let output_path = std::env::temp_dir().join(format!("corrplot-ng-config-{}.svg", std::process::id()));
    let rendered = Command::new(binary).args(["render", "--config"]).arg(&root)
        .args(["--output"]).arg(&output_path).output().unwrap();
    assert!(rendered.status.success(), "{}", String::from_utf8_lossy(&rendered.stderr));
    let svg = fs::read_to_string(&output_path).unwrap();
    assert!(svg.contains("width=\"160mm\""));
    assert!(svg.contains("#b2182b"));
    fs::remove_file(output_path).ok();
}

#[test]
fn publication_config_rejects_unsupported_semantics() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../prototypes/publication-config-v1/examples/significance.json");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng")).args(["validate", "--config"]).arg(&root).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("error:"), "expected an actionable validation diagnostic: {}", String::from_utf8_lossy(&output.stderr));
}


#[test]
fn publication_config_can_emit_renderer_neutral_scene_v1() {
    let config = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../prototypes/publication-config-v1/examples/basic.json");
    let output_path = std::env::temp_dir().join(format!("corrplot-ng-scene-{}.json", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&config)
        .args(["--format", "scene-json", "--output"]).arg(&output_path)
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let scene = fs::read_to_string(&output_path).unwrap();
    assert!(scene.contains("\"schema_version\":1"));
    assert!(scene.contains("\"kind\":\"correlation_plot\""));
    assert!(scene.contains("\"variable_order\""));
    assert!(scene.contains("\"typography\""));
    fs::remove_file(output_path).ok();
}


#[test]
fn publication_config_resolves_csv_paths_relative_to_config_file() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-relative-csv-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let config_path = directory.join("figure.json");
    let csv_path = directory.join("matrix.csv");
    let output_path = directory.join("rendered.svg");
    fs::write(&csv_path, ",alpha,beta\nalpha,1,0.5\nbeta,0.5,1\n").unwrap();
    let config = r##"{
      "config_version": 1,
      "input": {"kind":"matrix_csv","path":"matrix.csv","delimiter":",","header":true,"row_labels":true},
      "matrix": {"kind":"correlation","method":"pearson","missing":"preserve"},
      "plot": {"method":"circle","display":"full","ordering":"original","show_values":false,
        "significance":{"enabled":false,"p_values":null,"thresholds":[0.05]}},
      "colors":{"negative":"#b2182b","neutral":"#f7f7f7","positive":"#2166ac","missing":"#d9d9d9"},
      "dimensions":{"width":120,"height":100,"unit":"mm"},
      "typography":{"family":"sans-serif","base_size":9},
      "output":{"format":"svg","path":"ignored.svg"}
    }"##;
    fs::write(&config_path, config).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&output_path)
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let svg = fs::read_to_string(&output_path).unwrap();
    assert!(svg.contains(">alpha</text>"));
    assert!(svg.contains(">beta</text>"));

    fs::remove_dir_all(directory).ok();
}

#[test]
fn publication_svg_is_byte_for_byte_deterministic() {
    let config = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let directory = std::env::temp_dir().join(format!("corrplot-ng-deterministic-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let first_path = directory.join("first.svg");
    let second_path = directory.join("second.svg");

    for output_path in [&first_path, &second_path] {
        let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
            .args(["render", "--config"]).arg(&config)
            .args(["--output"]).arg(output_path)
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
    assert_eq!(fs::read(&first_path).unwrap(), fs::read(&second_path).unwrap());
    fs::remove_dir_all(directory).ok();
}

#[test]
fn invalid_publication_config_fails_before_creating_output() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let source = fs::read_to_string(fixture).unwrap();
    let invalid = source.replace("\"method\": \"circle\"", "\"method\": \"square\"");
    let directory = std::env::temp_dir().join(format!("corrplot-ng-invalid-config-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let config_path = directory.join("invalid.json");
    let output_path = directory.join("must-not-exist.svg");
    fs::write(&config_path, invalid).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&output_path)
        .output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not yet supported"));
    assert!(!output_path.exists(), "invalid config must not leave a partial output file");

    fs::remove_dir_all(directory).ok();
}


#[test]
fn publication_svg_respects_display_visibility_and_value_annotations() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let source = fs::read_to_string(fixture).unwrap();
    let config = source
        .replace("\"display\": \"full\"", "\"display\": \"upper\"")
        .replace("\"show_values\": false", "\"show_values\": true");
    let directory = std::env::temp_dir().join(format!("corrplot-ng-annotations-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let config_path = directory.join("figure.json");
    let scene_path = directory.join("scene.json");
    let svg_path = directory.join("figure.svg");
    fs::write(&config_path, config).unwrap();

    let scene_output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&config_path)
        .args(["--format", "scene-json", "--output"]).arg(&scene_path)
        .output().unwrap();
    assert!(scene_output.status.success(), "{}", String::from_utf8_lossy(&scene_output.stderr));
    let scene = fs::read_to_string(&scene_path).unwrap();
    assert!(scene.contains("\"visible\":false"), "lower-triangle cells should be hidden");
    assert!(scene.contains("\"role\":\"value\""), "show_values should produce value annotations");

    let svg_output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&svg_path)
        .output().unwrap();
    assert!(svg_output.status.success(), "{}", String::from_utf8_lossy(&svg_output.stderr));
    let svg = fs::read_to_string(&svg_path).unwrap();
    assert!(svg.contains(">0.71</text>"), "visible cell values should be rendered as SVG annotations");
    assert!(svg.contains("#b2182b"), "configured negative color should be used");

    fs::remove_dir_all(directory).ok();
}


#[test]
fn inline_and_relative_csv_inputs_produce_identical_scenes() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let source = fs::read_to_string(fixture).unwrap();
    let directory = std::env::temp_dir().join(format!("corrplot-ng-input-equivalence-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let inline_config = directory.join("inline.json");
    let csv_config = directory.join("csv.json");
    let csv_path = directory.join("matrix.csv");
    let inline_scene = directory.join("inline-scene.json");
    let csv_scene = directory.join("csv-scene.json");

    let input_start = source.find("\"input\": {").unwrap();
    let matrix_start = input_start + source[input_start..].find("\n  \"matrix\":").unwrap();
    let csv_source = format!(
        "{}\"input\": {{\"kind\":\"matrix_csv\",\"path\":\"matrix.csv\",\"delimiter\":\",\",\"header\":true,\"row_labels\":true}},{}",
        &source[..input_start],
        &source[matrix_start..],
    );
    fs::write(&inline_config, &source).unwrap();
    fs::write(&csv_config, csv_source).unwrap();
    fs::write(
        &csv_path,
        ",temperature,humidity,yield\ntemperature,1,-0.42,0.71\nhumidity,-0.42,1,-0.18\nyield,0.71,-0.18,1\n",
    ).unwrap();

    for (config, output_path) in [(&inline_config, &inline_scene), (&csv_config, &csv_scene)] {
        let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
            .args(["render", "--config"]).arg(config)
            .args(["--format", "scene-json", "--output"]).arg(output_path)
            .output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
    assert_eq!(fs::read(&inline_scene).unwrap(), fs::read(&csv_scene).unwrap());
    fs::remove_dir_all(directory).ok();
}

#[test]
fn invalid_publication_matrix_fails_before_creating_output() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let source = fs::read_to_string(fixture).unwrap();
    let invalid = source.replace("[1.0, -0.42, 0.71]", "[1.0, -0.41, 0.71]");
    let directory = std::env::temp_dir().join(format!("corrplot-ng-invalid-matrix-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let config_path = directory.join("invalid-matrix.json");
    let output_path = directory.join("must-not-exist.svg");
    fs::write(&config_path, invalid).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&output_path)
        .output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not symmetric"));
    assert!(!output_path.exists(), "invalid matrix must not leave a partial output file");
    fs::remove_dir_all(directory).ok();
}


#[test]
fn init_requires_a_tty_and_never_prompts_in_pipes() {
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .arg("init")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("requires an interactive terminal"));
    assert!(output.stdout.is_empty(), "non-interactive init must fail before printing prompts");
}

#[test]
fn ordinary_commands_remain_non_interactive() {
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--config"])
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../prototypes/publication-config-v1/examples/basic.json"))
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("publication-config v1"));
}


#[test]
fn init_help_is_available_without_a_tty() {
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["init", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("publication-config v1"));
}


#[test]
fn scene_v1_json_file_is_auto_detected_and_rendered() {
    let scene = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/shared-svg/scene-v1.example.json");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&scene)
        .args(["--format", "scene-json"])
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let rendered = String::from_utf8(output.stdout).unwrap();
    assert!(rendered.contains(r#""schema_version":1"#));
    assert!(rendered.contains("alpha"));
    assert!(rendered.contains("0.65"));
}

#[test]
fn scene_v1_json_can_be_read_from_stdin_with_explicit_format() {
    let scene = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/shared-svg/scene-v1.example.json");
    let contents = fs::read(scene).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--input", "-", "--input-format", "scene-json"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    child.stdin.as_mut().unwrap().write_all(&contents).unwrap();
    drop(child.stdin.take());
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("3 x 3 matrix"));
}

#[test]
fn explicit_tsv_input_format_uses_tab_delimiters() {
    let path = std::env::temp_dir().join(format!("corrplot-ng-input-{}.tsv", std::process::id()));
    fs::write(&path, "\talpha\tbeta\nalpha\t1\t0.5\nbeta\t0.5\t1\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--input"]).arg(&path)
        .args(["--input-format", "tsv"])
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    fs::remove_file(path).ok();
}

#[test]
fn invalid_scene_json_fails_before_creating_output() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-invalid-scene-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("invalid.json");
    let output_path = directory.join("must-not-exist.svg");
    fs::write(&input, r#"{ "schema_version": 1, "kind": "correlation_plot", "labels": [], "cells": [] }"#).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&input)
        .args(["--input-format", "scene-json", "--output"]).arg(&output_path)
        .output().unwrap();
    assert!(!output.status.success());
    assert!(!output_path.exists());
    fs::remove_dir_all(directory).ok();
}

#[test]
fn non_utf8_input_has_an_actionable_diagnostic() {
    let path = std::env::temp_dir().join(format!("corrplot-ng-invalid-encoding-{}.csv", std::process::id()));
    fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--input"]).arg(&path)
        .output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("UTF-8 text"));
    fs::remove_file(path).ok();
}


#[test]
fn inspect_reports_detected_input_format_and_matrix_metadata() {
    let csv = fixture("inspect", ",alpha,beta\nalpha,1,NA\nbeta,NA,1\n");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["inspect", "--input"]).arg(&csv)
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report = String::from_utf8(output.stdout).unwrap();
    assert!(report.contains("input format: csv"));
    assert!(report.contains("kind: correlation"));
    assert!(report.contains("rows: 2"));
    assert!(report.contains("missing: 2"));
    fs::remove_file(csv).ok();

    let scene = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/shared-svg/scene-v1.example.json");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["inspect", "--input"]).arg(scene)
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("input format: scene-v1 JSON"));
}

#[test]
fn missing_input_file_returns_actionable_nonzero_error() {
    let path = std::env::temp_dir().join(format!("corrplot-ng-missing-{}.csv", std::process::id()));
    let _ = fs::remove_file(&path);
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--input"]).arg(&path)
        .output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot read"));
    assert!(output.stdout.is_empty());
}

#[test]
fn malformed_arguments_use_usage_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--unsupported", "value"])
        .output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown option"));
}

#[test]
fn help_documents_all_commands_and_input_format_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .arg("--help").output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for option in ["render", "validate", "inspect", "presets", "version", "--input-format", "--format", "--incremental"] {
        assert!(help.contains(option), "help output is missing {option}");
    }
}


#[test]
fn publication_general_numeric_config_uses_automatic_limits_and_sequential_palette() {
    let basic = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let source = fs::read_to_string(basic).unwrap().replace("\r\n", "\n");
    let config = source
        .replace(
            "\"kind\": \"correlation\", \"method\": \"pearson\", \"missing\": \"preserve\"",
            "\"kind\": \"general_numeric\", \"missing\": \"preserve\"",
        )
        .replace(
            "[1.0, -0.42, 0.71],\n      [-0.42, 1.0, -0.18],\n      [0.71, -0.18, 1.0]",
            "[2.0, 4.0, 8.0],\n      [4.0, 5.0, 6.0],\n      [8.0, 6.0, 7.0]",
        );
    let config_path = std::env::temp_dir().join(format!("corrplot-ng-general-auto-{}.json", std::process::id()));
    let svg_path = std::env::temp_dir().join(format!("corrplot-ng-general-auto-{}.svg", std::process::id()));
    fs::write(&config_path, config).unwrap();

    let validated = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--config"]).arg(&config_path).output().unwrap();
    assert!(validated.status.success(), "{}", String::from_utf8_lossy(&validated.stderr));

    let rendered = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&svg_path).output().unwrap();
    assert!(rendered.status.success(), "{}", String::from_utf8_lossy(&rendered.stderr));
    let svg = fs::read_to_string(&svg_path).unwrap();
    assert!(svg.contains("fill=\"#ffffe5\""), "sequential low endpoint missing");
    assert!(svg.contains("fill=\"#662506\""), "sequential high endpoint missing");
    fs::remove_file(config_path).ok();
    fs::remove_file(svg_path).ok();
}

#[test]
fn publication_general_numeric_config_honors_explicit_limits_and_rejects_outside_values() {
    let basic = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let source = fs::read_to_string(basic).unwrap().replace("\r\n", "\n");
    let general = source
        .replace(
            "\"kind\": \"correlation\", \"method\": \"pearson\", \"missing\": \"preserve\"",
            "\"kind\": \"general_numeric\", \"missing\": \"preserve\", \"col_limits\": [0.0, 10.0]",
        )
        .replace(
            "[1.0, -0.42, 0.71],\n      [-0.42, 1.0, -0.18],\n      [0.71, -0.18, 1.0]",
            "[2.0, 4.0, 8.0],\n      [4.0, 5.0, 6.0],\n      [8.0, 6.0, 7.0]",
        );
    let valid_path = std::env::temp_dir().join(format!("corrplot-ng-general-limits-{}.json", std::process::id()));
    fs::write(&valid_path, general.clone()).unwrap();
    let valid = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--config"]).arg(&valid_path)
        .args(["--format", "scene-json"]).output().unwrap();
    assert!(valid.status.success(), "{}", String::from_utf8_lossy(&valid.stderr));
    let scene = String::from_utf8(valid.stdout).unwrap();
    assert!(scene.contains("\"color_limits\":[0,10]"));
    assert!(scene.contains("\"color_mapping\":\"sequential\""));

    let outside = general.replace("[0.0, 10.0]", "[3.0, 7.0]");
    fs::write(&valid_path, outside).unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--config"]).arg(&valid_path).output().unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("cover every observed"));
    fs::remove_file(valid_path).ok();
}

#[test]
fn publication_general_numeric_csv_requires_and_uses_explicit_limits() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-general-csv-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    fs::write(directory.join("matrix.csv"), ",a,b\na,2,4\nb,4,8\n").unwrap();
    let basic = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../prototypes/publication-config-v1/examples/basic.json");
    let source = fs::read_to_string(basic).unwrap().replace("\r\n", "\n");
    let general = source
        .replace(
            "\"kind\": \"inline_matrix\",\n    \"labels\": [\"temperature\", \"humidity\", \"yield\"],\n    \"values\": [\n      [1.0, -0.42, 0.71],\n      [-0.42, 1.0, -0.18],\n      [0.71, -0.18, 1.0]\n    ]",
            "\"kind\": \"matrix_csv\", \"path\": \"matrix.csv\", \"delimiter\": \",\" , \"header\": true, \"row_labels\": true",
        )
        .replace(
            "\"kind\": \"correlation\", \"method\": \"pearson\", \"missing\": \"preserve\"",
            "\"kind\": \"general_numeric\", \"missing\": \"preserve\", \"col_limits\": [2.0, 8.0]",
        );
    let config_path = directory.join("figure.json");
    fs::write(&config_path, general.clone()).unwrap();
    let valid = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--config"]).arg(&config_path).output().unwrap();
    assert!(valid.status.success(), "{}", String::from_utf8_lossy(&valid.stderr));

    let without_limits = general.replace(", \"col_limits\": [2.0, 8.0]", "");
    fs::write(&config_path, without_limits).unwrap();
    let invalid = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--config"]).arg(&config_path).output().unwrap();
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("required for CSV-backed"));
    fs::remove_dir_all(directory).ok();
}

#[test]
fn csv_and_tsv_inputs_can_emit_normalized_matrix_v1_json() {
    let csv = fixture("normalized-csv", ",A,B\nA,1,\nB,0,1\n");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&csv)
        .args(["--kind", "general", "--format", "normalized-json"])
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("normalized_matrix"));
    assert!(json.contains("format") && json.contains("csv"));
    assert!(json.contains("[[1,null],[0,1]]"));
    assert!(json.contains("[0,1]"));
    fs::remove_file(csv).ok();

    let tsv = fixture("normalized-tsv", "\tA\tB\nA\t1\t0\nB\t0\t1\n");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&tsv)
        .args(["--kind", "general", "--input-format", "tsv", "--format", "normalized-json"])
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("format") && json.contains("tsv"));
    assert!(json.contains("labels") && json.contains("A") && json.contains("B"));
    fs::remove_file(tsv).ok();
}

#[test]
fn all_missing_general_matrix_is_preserved_by_normalized_json_output() {
    let path = fixture("all-missing-normalized", ",A,B\nA,NA,null\nB,,NA\n");
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&path)
        .args(["--kind", "general", "--format", "normalized-json"])
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let json = String::from_utf8(output.stdout).unwrap();
    assert!(json.contains("[[null,null],[null,null]]"));
    fs::remove_file(path).ok();
}


#[test]
fn incremental_publication_render_skips_unchanged_inputs_and_invalidates_changes() {
    use std::thread;
    use std::time::Duration;

    let directory = std::env::temp_dir().join(format!("corrplot-ng-incremental-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let config_path = directory.join("figure.json");
    let matrix_path = directory.join("matrix.csv");
    let output_path = directory.join("figure.svg");
    fs::write(&matrix_path, ",alpha,beta\nalpha,1,0.5\nbeta,0.5,1\n").unwrap();
    let config = r##"{
      "config_version": 1,
      "input": {"kind":"matrix_csv","path":"matrix.csv","delimiter":",","header":true,"row_labels":true},
      "matrix": {"kind":"correlation","method":"pearson","missing":"preserve"},
      "plot": {"method":"circle","display":"full","ordering":"original","show_values":false,
        "significance":{"enabled":false,"p_values":null,"thresholds":[0.05]}},
      "colors":{"negative":"#b2182b","neutral":"#f7f7f7","positive":"#2166ac","missing":"#d9d9d9"},
      "dimensions":{"width":120,"height":100,"unit":"mm"},
      "typography":{"family":"sans-serif","base_size":9},
      "output":{"format":"svg","path":"ignored.svg"}
    }"##;
    fs::write(&config_path, config).unwrap();
    let binary = env!("CARGO_BIN_EXE_corrplot-ng");

    let first = Command::new(binary)
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&output_path)
        .arg("--incremental")
        .output().unwrap();
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let first_svg = fs::read(&output_path).unwrap();
    let first_modified = fs::metadata(&output_path).unwrap().modified().unwrap();
    assert!(output_path.with_file_name("figure.svg.corrplot-cache").exists());

    thread::sleep(Duration::from_millis(40));
    let unchanged = Command::new(binary)
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&output_path)
        .arg("--incremental")
        .output().unwrap();
    assert!(unchanged.status.success(), "{}", String::from_utf8_lossy(&unchanged.stderr));
    assert!(String::from_utf8_lossy(&unchanged.stderr).contains("unchanged:"));
    assert_eq!(fs::metadata(&output_path).unwrap().modified().unwrap(), first_modified);

    fs::write(&matrix_path, ",alpha,beta\nalpha,1,0.8\nbeta,0.8,1\n").unwrap();
    let changed_input = Command::new(binary)
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&output_path)
        .arg("--incremental")
        .output().unwrap();
    assert!(changed_input.status.success(), "{}", String::from_utf8_lossy(&changed_input.stderr));
    assert!(!String::from_utf8_lossy(&changed_input.stderr).contains("unchanged:"));
    let changed_svg = fs::read(&output_path).unwrap();
    assert_ne!(first_svg, changed_svg);

    let changed_config = config.replace("\"show_values\":false", "\"show_values\":true");
    fs::write(&config_path, changed_config).unwrap();
    let changed_options = Command::new(binary)
        .args(["render", "--config"]).arg(&config_path)
        .args(["--output"]).arg(&output_path)
        .arg("--incremental")
        .output().unwrap();
    assert!(changed_options.status.success(), "{}", String::from_utf8_lossy(&changed_options.stderr));
    assert!(!String::from_utf8_lossy(&changed_options.stderr).contains("unchanged:"));
    assert_ne!(changed_svg, fs::read(&output_path).unwrap());

    fs::remove_dir_all(directory).ok();
}

#[test]
fn incremental_batch_render_reuses_cached_outputs() {
    use std::thread;
    use std::time::Duration;

    let directory = std::env::temp_dir().join(format!("corrplot-ng-incremental-batch-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let config_path = directory.join("figure.json");
    let matrix_path = directory.join("matrix.csv");
    let manifest_path = directory.join("manifest.txt");
    let output_dir = directory.join("outputs");
    fs::write(&matrix_path, ",alpha,beta\nalpha,1,0.5\nbeta,0.5,1\n").unwrap();
    let config = r##"{
      "config_version": 1,
      "input": {"kind":"matrix_csv","path":"matrix.csv","delimiter":",","header":true,"row_labels":true},
      "matrix": {"kind":"correlation","method":"pearson","missing":"preserve"},
      "plot": {"method":"circle","display":"full","ordering":"original","show_values":false,
        "significance":{"enabled":false,"p_values":null,"thresholds":[0.05]}},
      "colors":{"negative":"#b2182b","neutral":"#f7f7f7","positive":"#2166ac","missing":"#d9d9d9"},
      "dimensions":{"width":120,"height":100,"unit":"mm"},
      "typography":{"family":"sans-serif","base_size":9},
      "output":{"format":"svg","path":"ignored.svg"}
    }"##;
    fs::write(&config_path, config).unwrap();
    fs::write(&manifest_path, "figure.json\n").unwrap();
    let binary = env!("CARGO_BIN_EXE_corrplot-ng");

    let first = Command::new(binary)
        .args(["render-batch", "--manifest"]).arg(&manifest_path)
        .args(["--output-dir"]).arg(&output_dir)
        .arg("--incremental")
        .output().unwrap();
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let output_path = output_dir.join("001-figure.svg");
    let modified = fs::metadata(&output_path).unwrap().modified().unwrap();

    thread::sleep(Duration::from_millis(40));
    let second = Command::new(binary)
        .args(["render-batch", "--manifest"]).arg(&manifest_path)
        .args(["--output-dir"]).arg(&output_dir)
        .arg("--incremental")
        .output().unwrap();
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert!(String::from_utf8_lossy(&second.stderr).contains("unchanged:"));
    assert_eq!(fs::metadata(&output_path).unwrap().modified().unwrap(), modified);

    fs::remove_dir_all(directory).ok();
}


#[test]
fn batch_diagnostics_json_preserves_partial_success_and_item_context() {
    let directory = env::temp_dir().join(format!("corrplot-ng-batch-diagnostics-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let matrix_path = directory.join("matrix.csv");
    let valid_config_path = directory.join("valid.json");
    let invalid_config_path = directory.join("invalid.json");
    let manifest_path = directory.join("manifest.txt");
    let output_dir = directory.join("outputs");
    fs::write(&matrix_path, ",alpha,beta\nalpha,1,0.5\nbeta,0.5,1\n").unwrap();
    let valid_config = r##"{
      "config_version": 1,
      "input": {"kind":"matrix_csv","path":"matrix.csv","delimiter":",","header":true,"row_labels":true},
      "matrix": {"kind":"correlation","method":"pearson","missing":"preserve"},
      "plot": {"method":"circle","display":"full","ordering":"original","show_values":false,
        "significance":{"enabled":false,"p_values":null,"thresholds":[0.05]}},
      "colors":{"negative":"#b2182b","neutral":"#f7f7f7","positive":"#2166ac","missing":"#d9d9d9"},
      "dimensions":{"width":120,"height":100,"unit":"mm"},
      "typography":{"family":"sans-serif","base_size":9},
      "output":{"format":"svg","path":"ignored.svg"}
    }"##;
    fs::write(&valid_config_path, valid_config).unwrap();
    fs::write(&invalid_config_path, "{ invalid json").unwrap();
    fs::write(&manifest_path, "valid.json\ninvalid.json\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render-batch", "--manifest"]).arg(&manifest_path)
        .args(["--output-dir"]).arg(&output_dir)
        .args(["--diagnostics", "json"])
        .output().unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\\"schema_version\\":1"));
    assert!(stdout.contains("\\"successes\\":[{\\"input\\":\\"valid.json\\""));
    assert!(stdout.contains("\\"output\\":\\""));
    assert!(stdout.contains("\\"diagnostics\\":[{"));
    assert!(stdout.contains("\\"manifest_index\\":2"));
    assert!(stdout.contains("\\"input\\":\\"invalid.json\\""));
    assert!(stdout.contains("\\"severity\\":\\"error\\""));
    assert!(output_dir.join("001-valid.svg").exists());
    assert!(!output_dir.join("002-invalid.svg").exists());

    fs::remove_dir_all(&directory).ok();
}

