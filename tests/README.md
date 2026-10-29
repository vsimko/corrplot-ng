# Tests

## Current scope

The R package is a provisional scaffold. Its exported `corrplot_ng()` function delegates to `corrplot::corrplot()`. The `testthat` tests exercise all seven legacy plot methods through this adapter. This verifies that the adapter and test harness work with the installed legacy package; it does not test a new implementation or prove independent compatibility.

The Python tests check that shared JSON fixtures are valid and internally consistent.

Run the fixture checks with Python 3.10 or later:

```sh
python -m unittest discover -s tests -v
```

The tests use only the Python standard library.

## Shared fixtures

`fixtures/correlation-cases.json` contains small, language-neutral example matrices and a list of legacy plot methods that need compatibility coverage.

Use JSON `null` for missing values. Keep matrix labels and dimensions explicit. Add a stable case ID and document the expected properties for each new case.

## R tests

Install the package dependencies, then run `R CMD check implementations/r` from the repository root, or change to `implementations/r` and run `R CMD check .`. To run tests interactively, use `testthat::test_local("implementations/r")`. GitHub Actions runs an R package check on pull requests and pushes to `main`.

## Next steps

- Pin the original `corrplot` package version used as the reference.
- Record expected outputs and behavior for key arguments.
- Replace the adapter implementation when the new R engine exists, without discarding the compatibility cases.
- Reuse the same fixtures in future language interfaces.
