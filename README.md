# corrplot-ng

**A next-generation, cross-language toolkit for correlation matrix visualization.**

`corrplot-ng` aims to evolve the functionality of the original R [`corrplot`](https://github.com/taiyun/corrplot) package into a reusable, extensible visualization toolkit with a modern API and language-specific wrappers.

> **Status:** Design and planning. The architecture, API, and roadmap are subject to validation through prototyping. Features described below are goals, not claims about an existing implementation.

## Goals

- **Preserve compatibility:** provide a drop-in replacement for the existing R `corrplot` API so established scripts can continue to work without modification.
- **Design a better API:** replace the legacy collection of plotting parameters with a consistent, declarative configuration model.
- **Support multiple languages:** provide wrappers for R, Python, and TypeScript.
- **Cover existing functionality:** support all visualization methods and relevant features available in the original `corrplot` package.
- **Enable new visualizations:** provide an extensible foundation for additional chart types and analytical capabilities.
- **Support migration:** develop a tool that converts existing R `corrplot` calls into the new configuration format.
- **Support reproducible research:** provide clear software citation instructions and version-specific citation metadata.

## Planned architecture

The intended architecture separates visualization semantics from language-specific APIs and rendering backends.

- **Core engine:** shared data model, configuration validation, visual encodings, layout, and visualization logic.
- **R wrapper:** modern API and legacy compatibility layer.
- **Python wrapper:** Python-native interface for scientific and data analysis workflows.
- **TypeScript wrapper:** interface for web applications and JavaScript/TypeScript environments.
- **Migration tool:** assists in converting legacy R code to the new API.
- **Rendering backends:** shared output formats and native integrations where appropriate.

Rust is a candidate for the shared core. The implementation language and rendering architecture will be confirmed through a technical prototype.

## Planned visualization capabilities

The initial compatibility target includes the seven visualization methods supported by the original `corrplot` package:

- Circles
- Squares
- Ellipses
- Numerical values
- Shading
- Color heatmaps
- Pie charts

The compatibility layer will also aim to preserve relevant matrix layouts, variable ordering, labels, significance annotations, confidence intervals, and mixed upper/lower triangle displays.

Potential future extensions include clustered heatmaps, pairwise correlation plots, correlation networks, and interactive visualizations.

These capabilities are planned features; they are not yet claimed to be implemented.

## Migration and backward compatibility

The project distinguishes between two interfaces:

1. **Legacy R API:** intended to preserve existing `corrplot` call patterns and behavior.
2. **New API:** designed around a smaller set of composable configuration objects with sensible defaults.

The migration tool will help users translate legacy calls into the new API and identify arguments that cannot be converted automatically.

Compatibility will be validated with regression tests against the original package. Full behavioral compatibility is a goal to verify, not an assumption.

## Local development

The repository currently contains an R compatibility-test package and a small Python prototype. These are development scaffolding, not a complete implementation.

### Prerequisites

- Git
- Python 3 (the shared prototype and fixture tests use only the Python standard library)
- R (version 4.x recommended)

On Debian or Ubuntu, install the system tools with:

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

To run the SVG prototype manually and write its output to standard output:

```sh
python3 prototypes/shared-svg/renderer.py prototypes/shared-svg/example.json
```

The R visual-regression tests compare output with committed PNG baselines. If a test fails, investigate the cause before regenerating or replacing those baselines. See the test scripts and fixtures for the exact workflow.

## Scientific citation

Software citation is an important project goal.

The project intends to provide standard citation metadata for each supported language ecosystem, including an R `CITATION` entry, a `CITATION.cff` file, and versioned archival releases with DOIs where feasible.

Users should be able to identify and cite both the relevant wrapper and the shared core.

## License

This project is licensed under the [MIT License](LICENSE).

## Roadmap

1. Inventory the original package's functionality.
2. Define the shared data model and new API.
3. Prototype the core and rendering architecture.
4. Establish R backward-compatibility tests.
5. Implement the language wrappers and migration tooling incrementally.

## Relationship to the original project

`corrplot-ng` is intended to build on the experience and established functionality of the original [`taiyun/corrplot`](https://github.com/taiyun/corrplot) project while developing a more general cross-language architecture.

The original package remains the reference for legacy R behavior and compatibility testing.
