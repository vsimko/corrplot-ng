# Diagnostic-v1 error model

corrplot-ng uses a language-neutral diagnostic record for validation,
capability, and rendering failures. The record is defined by
[diagnostic-v1 schema](../prototypes/diagnostic-v1.schema.json).

## Record fields

Every diagnostic contains:

- \`schema_version\`: currently \`1\`;
- \`code\`: stable machine-readable identifier;
- \`severity\`: \`error\`, \`warning\`, or \`info\`;
- \`stage\`: \`configuration\`, \`input\`, \`scene\`, \`renderer\`, or \`export\`;
- \`message\`: concise human-readable explanation.

Optional fields are \`path\`, \`row\`, \`column\`, and \`details\`. Row and
column coordinates are zero-based, matching normalized-matrix source indices.
Renderer diagnostics may also include \`renderer\`, \`renderer_version\`, and
\`feature\`.

Do not use message text as a programmatic identifier. Consumers should branch
on \`code\`, \`severity\`, and \`stage\`.

## Initial diagnostic catalogue

| Code | Stage | Meaning |
|---|---|---|
| \`input.matrix.invalid_type\` | input | Input is not a supported matrix or data frame. |
| \`input.matrix.invalid_shape\` | input | Matrix is empty, non-square, or does not match its labels. |
| \`input.matrix.non_numeric\` | input | Matrix contains a non-numeric value or column. |
| \`input.matrix.invalid_label\` | input | Labels are missing, duplicated, mismatched, or invalid. |
| \`input.matrix.non_finite_value\` | input | A matrix cell contains NaN or an infinite value. Row and column are supplied when known. |
| \`configuration.invalid_field\` | configuration | A configuration field has an invalid value or type. |
| \`configuration.invalid_metadata\` | configuration | Source metadata is not a named, unique list. |
| `configuration.option.deprecated` | configuration | A deprecated compatibility option was used; the replacement is identified in the message and path. |
| \`scene.invalid_content\` | scene | Scene content does not meet the scene contract. |
| \`renderer.unsupported_feature\` | renderer | A renderer cannot preserve a required feature. |
| \`renderer.optional_feature_omitted\` | renderer | The caller explicitly allowed an unsupported optional feature to be omitted. |
| \`renderer.schema_version_unsupported\` | renderer | The renderer does not support the scene schema version. |
| `renderer.output.invalid_options` | renderer | Output dimensions, units, or renderer-specific options are invalid. |
| `renderer.output.invalid_buffer` | export | An SVG, PNG, or raw pixel output violates its declared buffer contract. |
| `renderer.output.format_mismatch` | export | A renderer returned a different output format from the requested format. |
| \`cli.operation.failed\` | input | A CLI failure does not yet have a more specific catalogue entry. |

Add codes when a stable distinction is useful to callers. Do not create a new
code only to reflect a different wording of the same failure. A code must
have the same meaning in R, Python, and the CLI.

## R conditions

R errors inherit from \`corrplot_ng_error\` and carry a diagnostic-v1 list.
Warnings inherit from \`corrplot_ng_warning\`. Use
\`corrplot_ng_diagnostic(condition)\` to read the record:

\`\`\`r
result <- tryCatch(
  corrplot_ng_normalize_matrix(x),
  corrplot_ng_error = function(error) {
    diagnostic <- corrplot_ng_diagnostic(error)
    if (identical(diagnostic$code, "input.matrix.invalid_shape")) {
      message("Please provide a square matrix.")
    }
    diagnostic
  }
)
\`\`\`

Use condition classes and codes for programmatic handling. Do not parse the
human-readable message.

## Python

The shared-SVG prototype exposes \`DiagnosticError\` and
\`build_scene_with_diagnostics()\`. The convenience \`render_svg()\` path
raises a \`DiagnosticError\` for scene-construction failures. Renderer
capability failures carry a list of diagnostic-v1 records.

\`\`\`python
from diagnostics import DiagnosticError, build_scene_with_diagnostics

try:
    scene = build_scene_with_diagnostics(config)
except DiagnosticError as error:
    print(error.diagnostic.to_json())
\`\`\`

## Command-line output

The native CLI keeps its normal human-readable stderr output by default.
Pass \`--diagnostics json\` to emit one diagnostic-v1 JSON record to stdout
when a command fails. The process still exits with a nonzero status:

\`\`\`sh
corrplot-ng render --input matrix.csv --diagnostics json
\`\`\`

The current CLI maps common matrix, scene, configuration, and renderer
failures to stable codes. Unclassified failures use
\`cli.operation.failed\` until a more specific code is added.

## Warnings and batch behavior

Warnings are distinct from errors. A warning must not turn invalid input
into accepted input. Permissive renderer negotiation can omit a feature only
when the caller explicitly marks it optional; each omission returns a
warning diagnostic. R callers can distinguish warning and error conditions
by class and read the same diagnostic-v1 fields with
`corrplot_ng_diagnostic()`.

The native CLI supports batch rendering with a UTF-8 line-based manifest.
Each non-empty line names one publication-config v1 file; relative paths are
resolved from the manifest directory. The CLI processes each item
independently, preserves successful SVG outputs, and collects one diagnostic
for each failed item. It exits with status 1 when any item fails.

With `--diagnostics json`, batch mode emits a JSON report with
`schema_version`, `successes`, and `diagnostics`. Each diagnostic includes
the manifest index and input path in `details`. The report does not hide
successful outputs when another configuration fails.

## Reporting a bug

Include:

1. corrplot-ng version and runtime (R, Python prototype, or native CLI);
2. the diagnostic \`code\`, \`stage\`, \`path\`, and matrix coordinates when
   present;
3. the smallest input or configuration that reproduces the failure;
4. the command or R call used;
5. the relevant workflow or operating system if the failure is platform
   specific.

Remove private or sensitive data from the reproduction. Do not send stack
traces as a substitute for the diagnostic record.
