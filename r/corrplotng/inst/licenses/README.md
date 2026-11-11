# Third-party license notices

The R package itself is licensed under MIT. The `LICENSE` file contains the
DCF metadata required by `License: MIT + file LICENSE`; the complete MIT
license text is preserved in `corrplotng-MIT.txt`.

`extendr-MIT.txt` is the MIT license notice from the upstream extendr repository,
pinned to the `extendr-api-v0.9.0` release:

https://github.com/extendr/extendr/blob/main/LICENSE

It applies to the `extendr-api`, `extendr-ffi`, and `extendr-macros`
0.9.0 crates bundled by the production R package. The same notice is shared
by those three crates; it is copied into the generated R package under
`inst/licenses/` so it remains in the package source/install tree.

The copyright holders for the complete bundled Rust dependency set are listed
in `inst/COPYRIGHTS` and in the package `Authors@R` field. The remaining
third-party license texts are retained in the Cargo vendor archive by
`cargo vendor`.
