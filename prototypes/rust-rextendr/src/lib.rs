use extendr_api::prelude::*;

/// Return the number of cells in an n-by-n plot scene.
///
/// This function exists only to test the Rust-to-R boundary.
/// @export
#[extendr]
fn scene_cell_count(n_labels: i32) -> i32 {
    if n_labels <= 0 {
        return 0;
    }

    n_labels.saturating_mul(n_labels)
}

extendr_module! {
    mod corrplotngrustspike;
    fn scene_cell_count;
}
