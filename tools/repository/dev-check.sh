#!/usr/bin/env bash
set -Eeuo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

PYTHON_BIN="${PYTHON:-python3}"

usage() {
  cat <<'USAGE'
Usage: ./tools/repository/dev-check.sh <command>

Quick and focused checks:
  quick                 Python foundation checks used by the foundation CI job
  test-python           Alias for quick
  test-r                R scene consumer and R CMD check
  test-r-legacy         Pinned corrplot 0.95 semantic-oracle bridge
  test-rust-cli         Shared-core and native CLI tests plus release build
  test-node             Build the native Node addon and run its tests
  test-wasm             Test and build the browser WASM binding (requires wasm-pack)
  test-all              Run quick, R, Rust CLI, Node, browser-WASM, and legacy-oracle checks

Builds:
  build-cli             Build the native Rust CLI in release mode
  build-node            Build the native Node-API addon

Benchmarks (explicit opt-in; never part of quick/test-all):
  benchmarks            Run timing-harness tests and regular shared-core/scalability runs
  benchmarks-stress     Run scalability benchmarks including B03/B04 stress workloads

Other:
  help                  Show this help

Optional environment:
  PYTHON                Python executable (default: python3)
USAGE
}

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "Required command '$1' was not found. Install it and retry."
}

need_python() {
  need "$PYTHON_BIN"
}

need_r() {
  need R
  need Rscript
}

need_rust() {
  need cargo
  need rustc
}

need_node() {
  need node
  need npm
}

check_python_dependencies() {
  need_python
  "$PYTHON_BIN" -c 'import jsonschema' >/dev/null 2>&1 ||
    fail "Python test dependencies are missing. Install them with: $PYTHON_BIN -m pip install -r python/reference/requirements-dev.txt"
}

quick_checks() {
  check_python_dependencies
  # Check the canonical shared corpus before running language-specific contract tests.
  "$PYTHON_BIN" -m json.tool fixtures/matrix-input-v1/correlation-cases.json > /dev/null
  "$PYTHON_BIN" -m unittest discover -s tests -v
  "$PYTHON_BIN" - <<'PY'
import json
import jsonschema
from pathlib import Path

schema = json.loads(Path("contracts/renderer-capability-v1/schema.json").read_text())
declaration = json.loads(Path("rust/crates/corrplot-core/renderer-capabilities.json").read_text())
jsonschema.validate(declaration, schema)
PY
  "$PYTHON_BIN" -m unittest discover -s python/reference/prototypes/normalized-matrix-v1 -p 'test_*.py' -v
  (
    cd "$ROOT_DIR/python/reference"
    "$PYTHON_BIN" -m unittest discover -s . -p 'test_*.py' -v
  )
  "$PYTHON_BIN" -m unittest discover -s python/reference/prototypes/publication-config-v1 -p 'test_*.py' -v
  "$PYTHON_BIN" -m unittest discover -s python/reference/prototypes/publication-svg-v1 -p 'test_*.py' -v
  "$PYTHON_BIN" python/reference/renderer.py python/reference/example.json > "${TMPDIR:-/tmp}/corrplot-ng-shared-svg.svg"
  "$PYTHON_BIN" tools/repository/validate_svg.py "${TMPDIR:-/tmp}/corrplot-ng-shared-svg.svg"
}

test_r() {
  need_r
  Rscript -e 'stopifnot(as.character(utils::packageVersion("corrplot")) == "0.95", requireNamespace("jsonlite", quietly = TRUE))'
  Rscript python/reference/test_r_scene_consumer.R
  R CMD check --as-cran r/corrplotng
}

test_r_legacy() {
  need_r
  check_python_dependencies
  Rscript -e 'stopifnot(as.character(utils::packageVersion("corrplot")) == "0.95", requireNamespace("jsonlite", quietly = TRUE))'
  local output="${TMPDIR:-/tmp}/corrplot-legacy-scene-input.json"
  Rscript python/reference/test_legacy_corrplot_oracle.R
  "$PYTHON_BIN" python/reference/test_legacy_scene_bridge.py "$output"
}

test_rust_cli() {
  need_rust
  cargo test --locked --offline --manifest-path rust/crates/corrplot-core/Cargo.toml
  cargo test --locked --offline --manifest-path rust/tools/corrplot-cli/Cargo.toml
  cargo build --locked --offline --profile release-cli --manifest-path rust/tools/corrplot-cli/Cargo.toml
  need_python
  rust/target/release-cli/corrplot-ng render \
    --config python/reference/prototypes/publication-config-v1/examples/basic.json \
    --output "${TMPDIR:-/tmp}/corrplot-ng-native-cli.svg"
  "$PYTHON_BIN" scripts/validate_svg.py "${TMPDIR:-/tmp}/corrplot-ng-native-cli.svg"
}

test_node() {
  need_node
  need_rust
  (cd npm/packages/native-node && npm test)
}

test_wasm() {
  need_node
  need_rust
  need wasm-pack
  (cd npm/packages/wasm && wasm-pack test --node ../../../rust/bindings/wasm)
  (cd npm/packages/wasm && wasm-pack build --target web --release --out-dir ../../../npm/packages/wasm/pkg ../../../rust/bindings/wasm)
}

build_cli() {
  need_rust
  cargo build --locked --profile release-cli --manifest-path rust/tools/corrplot-cli/Cargo.toml
}

build_node() {
  need_node
  need_rust
  (cd npm/packages/native-node && npm run build)
}

benchmarks() {
  check_python_dependencies
  "$PYTHON_BIN" -m unittest discover -s benchmarks -p 'test_timing_method.py' -v
  "$PYTHON_BIN" benchmarks/run_shared_core.py --iterations 20 --output "${TMPDIR:-/tmp}/corrplot-ng-shared-core-benchmarks.json"
  "$PYTHON_BIN" benchmarks/run_scalability.py --iterations 5 --output "${TMPDIR:-/tmp}/corrplot-ng-scalability-benchmarks.json"
}

benchmarks_stress() {
  check_python_dependencies
  "$PYTHON_BIN" benchmarks/run_scalability.py --iterations 3 --include-stress --output "${TMPDIR:-/tmp}/corrplot-ng-scalability-stress.json"
}

command_name="${1:-help}"
case "$command_name" in
  help|-h|--help) usage ;;
  quick|test-python) quick_checks ;;
  test-r) test_r ;;
  test-r-legacy) test_r_legacy ;;
  test-rust-cli) test_rust_cli ;;
  test-node) test_node ;;
  test-wasm) test_wasm ;;
  test-all)
    quick_checks
    test_r
    test_rust_cli
    test_node
    test_wasm
    test_r_legacy
    ;;
  build-cli) build_cli ;;
  build-node) build_node ;;
  benchmarks) benchmarks ;;
  benchmarks-stress) benchmarks_stress ;;
  *) usage >&2; fail "Unknown command '$command_name'." ;;
esac
