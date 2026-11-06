# Third-party license notices

`extendr-MIT.txt` is the MIT license notice from the upstream extendr repository,
pinned to the `extendr-api-v0.9.0` release:

https://github.com/extendr/extendr/blob/main/LICENSE

It applies to the `extendr-api`, `extendr-ffi`, and `extendr-macros`
0.9.0 crates used by this isolated rextendr integration experiment. The same
notice is shared by those three crates; it is copied into the generated R
package under `inst/licenses/` so it remains in the package source/install
tree even though the individual vendored crate directories do not contain it.

The remaining dependencies' license texts are retained in their Cargo vendor
directories by `cargo vendor`.
