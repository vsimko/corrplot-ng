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
