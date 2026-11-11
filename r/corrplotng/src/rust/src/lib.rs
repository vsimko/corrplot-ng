use corrplot_core::output::{OutputOptions, RenderOutput, Renderer, SvgOptions, SvgUnit};
use corrplot_core::raster::SceneSvgRenderer;
use corrplot_core::scene::{scene_v1_to_scene, CellMethod};
use extendr_api::prelude::*;

/// Render a scene-v1 JSON document to SVG with the shared Rust renderer.
/// @param scene_json A single scene-v1 JSON document.
/// @return SVG markup as a character string.
/// @noRd
#[extendr]
fn render_scene_svg_rust(scene_json: &str) -> std::result::Result<String, String> {
    let scene = scene_v1_to_scene(scene_json, CellMethod::Circle)
        .map_err(|error| format!("{} ({})", error.message, error.path))?;
    let options = OutputOptions::Svg(SvgOptions::new(
        scene.width,
        scene.height,
        SvgUnit::Pixels,
    ));
    match SceneSvgRenderer
        .render(&scene, &options)
        .map_err(|error| format!("{error:?}"))?
    {
        RenderOutput::Svg(svg) => Ok(svg),
        _ => Err("shared scene renderer returned a non-SVG output".to_string()),
    }
}

extendr_module! {
    mod corrplotng;
    fn render_scene_svg_rust;
}
