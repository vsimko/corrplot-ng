# corrplot-core

This internal Rust library is the first shared compiled operation used by
both the native CLI and the rextendr R-binding spike.

## API boundary

\`validate_matrix(labels, rows, columns, values, kind)\` accepts row-major
\`Option<f64>\` values. \`None\` represents an explicit missing cell; NaN and
infinity remain observed values and are rejected. It validates non-empty
square dimensions, unique labels, finite observed values, and (for
correlation matrices) range, symmetry, and unit diagonal.

Errors expose stable diagnostic codes and optional zero-based row/column
coordinates. The crate does not depend on R, extendr, CLI parsing, JSON, or
any third-party Cargo package. Adapters own parsing and serialization and
translate R NA to None at the boundary.

This is an internal integration proof, not yet the complete production
visualization engine. Keep renderer and language-specific behavior outside
this crate.
