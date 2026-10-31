use std::fs;
use std::process::Command;

#[test]
fn unsupported_cell_method_uses_renderer_diagnostic_v1_fields() {
    let input = std::env::temp_dir().join(format!(
        "corrplot-ng-unsupported-method-{}.csv",
        std::process::id()
    ));
    fs::write(&input, ",A\nA,1\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_corrplot-ng"))
        .args(["render", "--input"])
        .arg(&input)
        .args(["--method", "pie", "--diagnostics", "json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let diagnostic = String::from_utf8(output.stdout).unwrap();
    assert!(diagnostic.contains(r#""code":"renderer.unsupported_feature""#), "{diagnostic}");
    assert!(diagnostic.contains(r#""severity":"error""#), "{diagnostic}");
    assert!(diagnostic.contains(r#""stage":"renderer""#), "{diagnostic}");
    assert!(diagnostic.contains(r#""path":"method""#), "{diagnostic}");

    fs::remove_file(input).ok();
}
