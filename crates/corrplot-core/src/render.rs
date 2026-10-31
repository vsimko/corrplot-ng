//! Renderer-neutral output contracts shared by native and future WASM adapters.
//!
//! A renderer consumes a validated scene document and an explicit output
//! request. Scene JSON, publication configuration, and renderer output
//! formats are separate contracts: scene-json is interchange, not a format.

use std::fmt;

pub const SCENE_SCHEMA_V1: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OutputFormat { Svg, Png, Rgba }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneKind { CorrelationPlot, GeneralMatrixPlot }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneFeature {
    RowLabels, ColumnLabels, PlotmathLabels, MissingCells,
    CellTextAnnotations, Significance, ConfidenceIntervals, Legend,
    FigureText, AccessibleMetadata, ClusterRegions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NegotiationMode { Strict, Permissive }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalUnit { Millimeter, Inch }

impl PhysicalUnit {
    pub const fn suffix(self) -> &'static str {
        match self { Self::Millimeter => "mm", Self::Inch => "in" }
    }
}

/// SVG may use scene-logical, pixel, or physical dimensions. Raster output
/// requires explicit pixels or physical dimensions with a DPI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputSize {
    SceneUnits,
    Physical { width: f64, height: f64, unit: PhysicalUnit, dpi: Option<f64> },
    Pixels { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor { pub red: u8, pub green: u8, pub blue: u8, pub alpha: u8 }

impl RgbaColor {
    pub const WHITE: Self = Self { red: 255, green: 255, blue: 255, alpha: 255 };
    pub const BLACK: Self = Self { red: 0, green: 0, blue: 0, alpha: 255 };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Background { Transparent, Color(RgbaColor) }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat { Rgba8 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode { Opaque, Straight, Premultiplied }

#[derive(Debug, Clone, PartialEq)]
pub struct RenderRequest {
    pub format: OutputFormat,
    pub size: OutputSize,
    pub background: Background,
    pub pixel_format: PixelFormat,
    pub alpha_mode: AlphaMode,
    pub negotiation: NegotiationMode,
    /// Features that must be preserved; unsupported required features fail.
    pub required_features: Vec<SceneFeature>,
    /// Features the caller explicitly permits a permissive renderer to omit.
    pub optional_features: Vec<SceneFeature>,
}

impl Default for RenderRequest {
    fn default() -> Self {
        Self {
            format: OutputFormat::Svg,
            size: OutputSize::SceneUnits,
            background: Background::Color(RgbaColor::WHITE),
            pixel_format: PixelFormat::Rgba8,
            alpha_mode: AlphaMode::Straight,
            negotiation: NegotiationMode::Strict,
            required_features: Vec::new(),
            optional_features: Vec::new(),
        }
    }
}

/// Opaque scene-v1 JSON plus metadata needed for output negotiation.
///
/// The producer must validate the JSON payload against the scene-v1 schema
/// before constructing this envelope. The core deliberately has no JSON
/// library dependency.
#[derive(Debug, Clone, Copy)]
pub struct SceneDocument<'a> {
    json: &'a str,
    schema_version: u32,
    kind: SceneKind,
    logical_width: f64,
    logical_height: f64,
    cell_methods: &'a [&'a str],
    features: &'a [SceneFeature],
}

impl<'a> SceneDocument<'a> {
    pub fn new(
        json: &'a str,
        schema_version: u32,
        kind: SceneKind,
        logical_width: f64,
        logical_height: f64,
        cell_methods: &'a [&'a str],
        features: &'a [SceneFeature],
    ) -> Result<Self, RenderError> {
        if !logical_width.is_finite() || logical_width <= 0.0
            || !logical_height.is_finite() || logical_height <= 0.0
        {
            return Err(RenderError::new(
                "renderer.invalid_scene_dimensions",
                "scene logical width and height must be positive finite numbers",
            ));
        }
        Ok(Self { json, schema_version, kind, logical_width, logical_height, cell_methods, features })
    }

    pub fn json(&self) -> &'a str { self.json }
    pub fn schema_version(&self) -> u32 { self.schema_version }
    pub fn kind(&self) -> SceneKind { self.kind }
    pub fn logical_size(&self) -> (f64, f64) { (self.logical_width, self.logical_height) }
    pub fn cell_methods(&self) -> &'a [&'a str] { self.cell_methods }
    pub fn features(&self) -> &'a [SceneFeature] { self.features }
}

#[derive(Debug, Clone, Copy)]
pub struct RendererCapabilities {
    pub renderer_id: &'static str,
    pub renderer_version: &'static str,
    pub scene_schema_versions: &'static [u32],
    pub output_formats: &'static [OutputFormat],
    pub matrix_kinds: &'static [SceneKind],
    pub cell_methods: &'static [&'static str],
    pub features: &'static [SceneFeature],
    pub negotiation_modes: &'static [NegotiationMode],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError { pub code: &'static str, pub message: String }

impl RenderError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for RenderError {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NegotiationReport {
    /// Features omitted only because they were explicitly optional and
    /// permissive negotiation was selected. Emit one warning per item.
    pub omitted_features: Vec<SceneFeature>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderedArtifact<T> {
    pub format: OutputFormat,
    pub size: OutputSize,
    pub renderer_id: &'static str,
    pub renderer_version: &'static str,
    pub omitted_features: Vec<SceneFeature>,
    pub content: T,
}

/// Implement this interface for each renderer/scene adapter pair.
///
/// Implementations must call validate_render_request before writing output.
/// Associated output types keep SVG strings, PNG bytes, and raw pixel buffers
/// distinct without importing a platform-specific graphics API.
pub trait Renderer<S> {
    type Output;
    fn capabilities(&self) -> &'static RendererCapabilities;
    fn render(
        &self,
        scene: &S,
        request: &RenderRequest,
    ) -> Result<RenderedArtifact<Self::Output>, RenderError>;
}

pub fn validate_render_request(
    capabilities: &RendererCapabilities,
    scene: &SceneDocument<'_>,
    request: &RenderRequest,
) -> Result<NegotiationReport, RenderError> {
    if !capabilities.scene_schema_versions.contains(&scene.schema_version) {
        return Err(RenderError::new(
            "renderer.schema_version_unsupported",
            format!("renderer '{}' does not support scene schema version {}", capabilities.renderer_id, scene.schema_version),
        ));
    }
    if !capabilities.matrix_kinds.contains(&scene.kind) {
        return Err(RenderError::new(
            "renderer.unsupported_matrix_kind",
            format!("renderer '{}' does not support matrix kind {:?}", capabilities.renderer_id, scene.kind),
        ));
    }
    if !capabilities.output_formats.contains(&request.format) {
        return Err(RenderError::new(
            "renderer.unsupported_format",
            format!("renderer '{}' does not support output format {:?}", capabilities.renderer_id, request.format),
        ));
    }
    if !capabilities.negotiation_modes.contains(&request.negotiation) {
        return Err(RenderError::new(
            "renderer.unsupported_negotiation_mode",
            format!("renderer '{}' does not support {:?} negotiation", capabilities.renderer_id, request.negotiation),
        ));
    }
    for method in scene.cell_methods {
        if !capabilities.cell_methods.contains(method) {
            return Err(RenderError::new(
                "renderer.unsupported_feature",
                format!("renderer '{}' does not support cell method '{method}'", capabilities.renderer_id),
            ));
        }
    }

    validate_size(request.format, request.size)?;
    validate_alpha(request)?;

    for feature in request.required_features.iter().chain(request.optional_features.iter()) {
        if !scene.features.contains(feature) {
            return Err(RenderError::new(
                "renderer.scene_feature_missing",
                format!("scene does not contain requested feature {:?}", feature),
            ));
        }
    }

    for feature in &request.required_features {
        if !capabilities.features.contains(feature) {
            return Err(RenderError::new(
                "renderer.unsupported_feature",
                format!("renderer '{}' cannot preserve required feature {:?}", capabilities.renderer_id, feature),
            ));
        }
    }

    let mut report = NegotiationReport::default();
    for feature in &request.optional_features {
        if request.required_features.contains(feature) {
            return Err(RenderError::new(
                "renderer.invalid_request",
                format!("feature {:?} cannot be both required and optional", feature),
            ));
        }
        if !capabilities.features.contains(feature) {
            if request.negotiation == NegotiationMode::Strict {
                return Err(RenderError::new(
                    "renderer.unsupported_feature",
                    format!("strict negotiation does not permit omitting {:?}", feature),
                ));
            }
            report.omitted_features.push(*feature);
        }
    }
    Ok(report)
}

fn validate_size(format: OutputFormat, size: OutputSize) -> Result<(), RenderError> {
    match size {
        OutputSize::SceneUnits if format != OutputFormat::Svg => Err(RenderError::new(
            "renderer.invalid_output_size",
            "raster output requires explicit pixel dimensions or physical dimensions with DPI",
        )),
        OutputSize::SceneUnits => Ok(()),
        OutputSize::Pixels { width, height } if width == 0 || height == 0 => Err(RenderError::new(
            "renderer.invalid_output_size",
            "pixel width and height must be greater than zero",
        )),
        OutputSize::Pixels { .. } => Ok(()),
        OutputSize::Physical { width, height, dpi, .. } => {
            if !width.is_finite() || width <= 0.0 || !height.is_finite() || height <= 0.0 {
                return Err(RenderError::new(
                    "renderer.invalid_output_size",
                    "physical width and height must be positive finite numbers",
                ));
            }
            if let Some(dpi) = dpi {
                if !dpi.is_finite() || dpi <= 0.0 {
                    return Err(RenderError::new(
                        "renderer.invalid_output_size",
                        "DPI must be a positive finite number",
                    ));
                }
            } else if format != OutputFormat::Svg {
                return Err(RenderError::new(
                    "renderer.invalid_output_size",
                    "raster output from physical dimensions requires an explicit DPI",
                ));
            }
            Ok(())
        }
    }
}

fn validate_alpha(request: &RenderRequest) -> Result<(), RenderError> {
    if request.format == OutputFormat::Svg { return Ok(()); }
    if request.background == Background::Transparent && request.alpha_mode == AlphaMode::Opaque {
        return Err(RenderError::new(
            "renderer.invalid_raster_options",
            "a transparent background requires an alpha-capable output mode",
        ));
    }
    if request.format == OutputFormat::Png && request.alpha_mode == AlphaMode::Premultiplied {
        return Err(RenderError::new(
            "renderer.invalid_raster_options",
            "PNG output uses straight alpha; premultiplied alpha is valid only for raw RGBA buffers",
        ));
    }
    if request.format == OutputFormat::Rgba && request.pixel_format != PixelFormat::Rgba8 {
        return Err(RenderError::new(
            "renderer.invalid_raster_options",
            "raw RGBA output requires the RGBA8 pixel format",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORMATS: &[OutputFormat] = &[OutputFormat::Svg, OutputFormat::Png, OutputFormat::Rgba8];
    const KINDS: &[SceneKind] = &[SceneKind::CorrelationPlot, SceneKind::GeneralMatrixPlot];
    const METHODS: &[&str] = &["circle", "number"];
    const FEATURES: &[SceneFeature] = &[
        SceneFeature::RowLabels, SceneFeature::ColumnLabels,
        SceneFeature::MissingCells, SceneFeature::AccessibleMetadata,
    ];
    const MODES: &[NegotiationMode] = &[NegotiationMode::Strict, NegotiationMode::Permissive];
    const CAPS: RendererCapabilities = RendererCapabilities {
        renderer_id: "test-renderer", renderer_version: "1",
        scene_schema_versions: &[SCENE_SCHEMA_V1], output_formats: FORMATS,
        matrix_kinds: KINDS, cell_methods: METHODS, features: FEATURES,
        negotiation_modes: MODES,
    };

    fn scene() -> SceneDocument<'static> {
        SceneDocument::new(
            r#"{"schema_version":1,"kind":"correlation_plot"}"#,
            SCENE_SCHEMA_V1, SceneKind::CorrelationPlot, 640.0, 480.0, &["circle"],
            &[SceneFeature::RowLabels, SceneFeature::ColumnLabels, SceneFeature::Legend],
        ).unwrap()
    }

    #[test]
    fn svg_defaults_to_scene_units_and_white_background() {
        let request = RenderRequest::default();
        assert_eq!(request.format, OutputFormat::Svg);
        assert_eq!(request.size, OutputSize::SceneUnits);
        assert_eq!(request.background, Background::Color(RgbaColor::WHITE));
        assert_eq!(request.negotiation, NegotiationMode::Strict);
        assert!(validate_render_request(&CAPS, &scene(), &request).is_ok());
    }

    #[test]
    fn strict_negotiation_rejects_unsupported_required_features() {
        let mut request = RenderRequest::default();
        request.required_features.push(SceneFeature::Legend);
        let error = validate_render_request(&CAPS, &scene(), &request).unwrap_err();
        assert_eq!(error.code, "renderer.unsupported_feature");
    }

    #[test]
    fn permissive_negotiation_reports_only_explicitly_optional_omissions() {
        let mut request = RenderRequest::default();
        request.negotiation = NegotiationMode::Permissive;
        request.optional_features.push(SceneFeature::Legend);
        let report = validate_render_request(&CAPS, &scene(), &request).unwrap();
        assert_eq!(report.omitted_features, vec![SceneFeature::Legend]);
        request.negotiation = NegotiationMode::Strict;
        assert_eq!(
            validate_render_request(&CAPS, &scene(), &request).unwrap_err().code,
            "renderer.unsupported_feature"
        );
    }

    #[test]
    fn rejects_unknown_scene_versions_and_unsupported_cell_methods() {
        let future = SceneDocument::new(
            "{}", 2, SceneKind::CorrelationPlot, 1.0, 1.0, &["circle"], &[],
        ).unwrap();
        assert_eq!(
            validate_render_request(&CAPS, &future, &RenderRequest::default()).unwrap_err().code,
            "renderer.schema_version_unsupported"
        );
        let unknown_method = SceneDocument::new(
            "{}", 1, SceneKind::CorrelationPlot, 1.0, 1.0, &["pie"], &[],
        ).unwrap();
        assert_eq!(
            validate_render_request(&CAPS, &unknown_method, &RenderRequest::default()).unwrap_err().code,
            "renderer.unsupported_feature"
        );
    }

    #[test]
    fn raster_output_requires_explicit_dimensions_and_valid_alpha() {
        let mut request = RenderRequest::default();
        request.format = OutputFormat::Png;
        assert_eq!(
            validate_render_request(&CAPS, &scene(), &request).unwrap_err().code,
            "renderer.invalid_output_size"
        );
        request.size = OutputSize::Pixels { width: 640, height: 480 };
        request.background = Background::Transparent;
        request.alpha_mode = AlphaMode::Opaque;
        assert_eq!(
            validate_render_request(&CAPS, &scene(), &request).unwrap_err().code,
            "renderer.invalid_raster_options"
        );
        request.alpha_mode = AlphaMode::Straight;
        assert!(validate_render_request(&CAPS, &scene(), &request).is_ok());
    }

    #[test]
    fn physical_raster_dimensions_require_positive_dpi() {
        let mut request = RenderRequest::default();
        request.format = OutputFormat::Rgba8;
        request.size = OutputSize::Physical {
            width: 120.0, height: 80.0, unit: PhysicalUnit::Millimeter, dpi: Some(300.0),
        };
        assert!(validate_render_request(&CAPS, &scene(), &request).is_ok());
        request.size = OutputSize::Physical {
            width: 120.0, height: 80.0, unit: PhysicalUnit::Millimeter, dpi: None,
        };
        assert_eq!(
            validate_render_request(&CAPS, &scene(), &request).unwrap_err().code,
            "renderer.invalid_output_size"
        );
    }
}
