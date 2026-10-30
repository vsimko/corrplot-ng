use std::fs;
use std::process::Command;

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
    assert!(String::from_utf8_lossy(&output.stderr).contains("not yet supported"));
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
