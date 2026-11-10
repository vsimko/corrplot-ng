# Contributing

corrplot-ng is a multi-ecosystem monorepo. Keep changes inside the project that
owns the behavior, and do not install every toolchain unless the change spans
those ecosystems.

## Before changing code

1. Read the [developer setup guide](docs/development.md) and identify the
   owning project root.
2. Read the relevant architecture/ADR and issue acceptance criteria.
3. Check the current implementation and tests before treating documentation
   or an issue description as proof of behavior.
4. Keep shared correlation and visualization semantics in
   `rust/crates/corrplot-core`. Keep R legacy API and graphics-device behavior
   in `implementations/r/`. Keep framework lifecycle code in its npm adapter.

## Changes and validation

- Prefer one focused issue and one reviewable pull request per change.
- Run the targeted tests for the changed ecosystem. Use
  `bash scripts/dev-check.sh test-all` only when the change or integration
  risk spans all ecosystems.
- Update the owning README and relevant architecture/developer documentation
  when commands, public behavior, dependencies, or project boundaries change.
- For consumer-visible npm package changes, add a Changeset from `npm/`
  using `npm run changeset`. Repository-only documentation and maintenance
  do not need a Changeset.
- Do not commit generated dependency directories, Cargo targets, Python
  environments, R check output, generated WASM/native binaries, or benchmark
  scratch files.
- Do not publish npm packages, migrate the production R package, or publish
  Linux distribution packages without their explicit maintainer approval and
  release gates.

## Pull request description

Include:

- The issue or reason for the change.
- The owning project and files changed.
- The exact commands run and their results.
- Any checks intentionally skipped because the changed paths do not require
  them.
- Remaining limitations or release gates. Do not describe skipped checks as
  passed.

Keep documentation in English and follow the writing and status rules in
[the documentation index](docs/index.md).
