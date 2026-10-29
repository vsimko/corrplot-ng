# Tests

## Current scope

The initial automated tests check that shared JSON fixtures are valid and internally consistent. They do not test the corrplot implementation or prove compatibility with the original R package.

Run the current checks with Python 3.10 or later:

```sh
python -m unittest discover -s tests -v
```

The tests use only the Python standard library.

## Shared fixtures

`fixtures/correlation-cases.json` contains small, language-neutral example matrices and a list of legacy plot methods that need compatibility coverage.

Use JSON `null` for missing values. Keep matrix labels and dimensions explicit. Add a stable case ID and document the expected properties for each new case.

## Next steps

- Add pinned reference outputs from the original R package.
- Add `testthat` tests when the R package structure is available.
- Test each legacy plot method and key argument combinations.
- Reuse the same fixtures in future language interfaces.
