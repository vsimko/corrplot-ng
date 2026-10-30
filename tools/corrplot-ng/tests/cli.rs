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
fn tsv_file_is_detected_and_validated() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-tsv-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("matrix.tsv");
    fs::write(&input, "\talpha\tbeta\nalpha\t1\t0.4\nbeta\t0.4\t1\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--input"]).arg(&input).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("2 x 2 matrix"));
    fs::remove_dir_all(directory).ok();
}

#[test]
fn json_matrix_input_renders_a_scene() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-json-matrix-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("matrix.json");
    let output_path = directory.join("scene.json");
    fs::write(&input, r#"{"labels":["alpha","beta"],"values":[[1,0.4],[0.4,1]]}"#).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&input)
        .args(["--format", "scene-json", "--output"]).arg(&output_path)
        .output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let scene = fs::read_to_string(&output_path).unwrap();
    assert!(scene.contains("\"kind\":\"correlation_plot\""));
    assert!(scene.contains("\"value\":0.4"));
    fs::remove_dir_all(directory).ok();
}

#[test]
fn scene_v1_json_can_be_reused_as_matrix_input() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-scene-input-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let csv = directory.join("matrix.csv");
    let scene = directory.join("scene.json");
    let roundtrip = directory.join("roundtrip.json");
    fs::write(&csv, ",alpha,beta\nalpha,1,0.4\nbeta,0.4,1\n").unwrap();
    let binary = env!("CARGO_BIN_EXE_corrplot-ng");
    let first = Command::new(binary).args(["render", "--input"]).arg(&csv)
        .args(["--format", "scene-json", "--output"]).arg(&scene).output().unwrap();
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    let second = Command::new(binary).args(["render", "--input"]).arg(&scene)
        .args(["--input-format", "json", "--format", "scene-json", "--output"]).arg(&roundtrip)
        .output().unwrap();
    assert!(second.status.success(), "{}", String::from_utf8_lossy(&second.stderr));
    assert_eq!(fs::read(&scene).unwrap(), fs::read(&roundtrip).unwrap());
    fs::remove_dir_all(directory).ok();
}

#[test]
fn stdin_to_stdout_keeps_scene_stream_free_of_diagnostics() {
    let binary = env!("CARGO_BIN_EXE_corrplot-ng");
    let mut child = Command::new(binary)
        .args(["render", "--input", "-", "--input-format", "tsv", "--format", "scene-json", "--output", "-"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    child.stdin.take().unwrap()
        .write_all(b"\talpha\tbeta\nalpha\t1\t0.4\nbeta\t0.4\t1\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("{\"schema_version\":1"));
    assert!(output.stderr.is_empty(), "diagnostics must not be mixed into the artifact stream");
}

#[test]
fn invalid_utf8_has_an_actionable_diagnostic() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-invalid-utf8-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("matrix.csv");
    fs::write(&input, [0xff, 0xfe, 0xfd]).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["validate", "--input"]).arg(&input).output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not valid UTF-8"));
    fs::remove_dir_all(directory).ok();
}

#[test]
fn ambiguous_stdin_format_requires_an_explicit_override() {
    let binary = env!("CARGO_BIN_EXE_corrplot-ng");
    let mut child = Command::new(binary).args(["validate", "--input", "-"])
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    child.stdin.take().unwrap().write_all(b",a\tb\n,a\t1\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ambiguous"));
}

#[test]
fn invalid_input_does_not_truncate_an_existing_output() {
    let directory = std::env::temp_dir().join(format!("corrplot-ng-no-truncate-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let input = directory.join("invalid.csv");
    let output_path = directory.join("existing.svg");
    fs::write(&input, ",alpha,beta\nalpha,1,0.2\nbeta,0.5,1\n").unwrap();
    fs::write(&output_path, "keep existing artifact").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"]).arg(&input)
        .args(["--output"]).arg(&output_path).output().unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(&output_path).unwrap(), "keep existing artifact");
    fs::remove_dir_all(directory).ok();
}
