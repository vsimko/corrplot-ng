# corrplot-ng

**A next-generation, cross-language toolkit for correlation matrix visualization.**

`corrplot-ng` aims to build on the functionality of the original R [`corrplot`](https://github.com/taiyun/corrplot) package with a reusable visualization core, modern APIs, and language-specific interfaces.

> **Status:** Early prototype. The repository currently contains a shared scene contract, an R compatibility adapter, a Python prototype, and experimental C++/Rcpp and Rust/rextendr implementations. It is not a production-ready replacement. The architecture and APIs are still being validated.

## Goals

- **Compatibility:** preserve the established R `corrplot` workflows through a legacy compatibility layer.
- **A modern API:** provide a consistent, declarative configuration model with sensible defaults.
- **Multiple languages:** support R, Python, and TypeScript through language-specific interfaces.
- **Visualization coverage:** implement the visualization methods and relevant features of the original package.
- **Migration:** help convert existing R `corrplot` calls to the new configuration model.
- **Extensibility:** provide a foundation for additional visualization types and analytical capabilities.

## Architecture

The project separates visualization semantics from language-specific APIs and rendering backends. The planned components are:

- **Core:** shared data model, configuration validation, visual encodings, layout, and visualization logic.
- **Language interfaces:** R (including legacy compatibility), Python, and TypeScript.
- **Migration tooling:** assists with translating legacy R calls and identifying arguments that need manual attention.
- **Rendering:** shared output formats and native integrations where appropriate.

Rust is being evaluated as a candidate for the shared core. The implementation language and rendering architecture will be confirmed through technical prototyping.

## Visualization capabilities

The initial compatibility target covers the seven visualization methods available in the original R package:

- Circles
- Squares
- Ellipses
- Numerical values
- Shading
- Color heatmaps
- Pie charts

The compatibility layer also aims to support matrix layouts, variable ordering, labels, significance annotations, confidence intervals, and mixed upper/lower triangle displays.

Potential future extensions include clustered heatmaps, pairwise correlation plots, correlation networks, and interactive visualizations.

These are project goals, not claims that all listed capabilities are already implemented.

## Compatibility and migration

The project has two intended interfaces:

1. **Legacy R API:** preserve established `corrplot` call patterns and behavior where practical.
2. **New API:** offer a smaller set of composable configuration objects.

A migration tool is planned to translate legacy calls and report arguments that cannot be converted automatically. Compatibility will be checked with regression tests against the original package; full behavioral compatibility remains a goal to validate.

The original [`taiyun/corrplot`](https://github.com/taiyun/corrplot) package is the reference for legacy R behavior.

## Local development

### Prerequisites

- Git
- Python 3.10 or later
- R (version 4.x recommended)

On Debian or Ubuntu, install the system tools:

```sh
sudo apt-get update
sudo apt-get install -y git python3 r-base
```

Install the R packages used by the compatibility wrapper and tests:

```sh
R -q -e 'install.packages(c("corrplot", "testthat", "jsonlite", "png"), repos = "https://cloud.r-project.org")'
```

### Run tests

Run the Python fixture-integrity tests from the repository root:

```sh
python3 -m unittest discover -s tests -v
```

Run the shared SVG/scene prototype tests:

```sh
python3 -m unittest discover -s prototypes/shared-svg -p 'test_*.py' -v
```

Run the R package tests:

```sh
R -q -e 'testthat::test_local("implementations/r")'
```

Run the benchmark timing-harness unit tests (these do not execute performance benchmarks):

```sh
python3 -m unittest discover -s benchmarks -p 'test_timing_method.py' -v
```

For performance benchmarks, see the [benchmark guide](benchmarks/README.md). Benchmarks are manual and do not run on ordinary pull requests.

To run the SVG prototype manually and write its output to standard output:

```sh
python3 prototypes/shared-svg/renderer.py prototypes/shared-svg/example.json
```

The R visual-regression tests compare output with committed PNG baselines. Investigate failures before regenerating or replacing those baselines; the test scripts and fixtures document the exact workflow.

## Scientific citation

The project intends to provide standard citation metadata for supported language ecosystems, including versioned software citation information.

## License

This project is licensed under the [MIT License](LICENSE).
