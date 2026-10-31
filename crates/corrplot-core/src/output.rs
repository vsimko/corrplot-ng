//! Renderer-neutral output types shared by native and WebAssembly consumers.
//!
//! Output selection is separate from publication configuration and scene data.
//! Renderer implementations receive a scene plus validated output options.

use std::fmt;

/// A serialized output format supported by the shared renderer interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Svg,
    Png,
    Rgba,
}

impl OutputFormat {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Svg => "svg",
            Self::Png => "png",
            Self::Rgba => "rgba",
        }
    }
}

/// Length units accepted by the vector SVG output contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SvgUnit {
    Pixels,
    Millimeters,
    Inches,
}

impl SvgUnit {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pixels => "px",
            Self::Millimeters => "mm",
            Self::Inches => "in",
        }
    }
}

/// An sRGB color with straight (unassociated) alpha.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl RgbaColor {
    pub const WHITE: Self = Self {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };

    pub const TRANSPARENT: Self = Self {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0,
    };
}

/// SVG viewport dimensions and output-specific typography/background.
#[derive(Debug, Clone, PartialEq)]
pub struct SvgOptions {
    pub width: f64,
    pub height: f64,
    pub unit: SvgUnit,
    pub font_family: String,
    pub font_size_pt: f64,
    pub background: RgbaColor,
}

impl SvgOptions {
    pub fn new(width: f64, height: f64, unit: SvgUnit) -> Self {
        Self {
            width,
            height,
            unit,
            font_family: "sans-serif".to_string(),
            font_size_pt: 10.0,
            background: RgbaColor::WHITE,
        }
    }

    fn validate(&self) -> Result<(), OutputValidationError> {
        if !self.width.is_finite() || self.width <= 0.0
            || !self.height.is_finite() || self.height <= 0.0
        {
            return Err(OutputValidationError::invalid_options(
                "SVG width and height must be finite positive dimensions in the declared unit",
            ));
        }
        if self.font_family.trim().is_empty()
            || !self.font_size_pt.is_finite()
            || self.font_size_pt <= 0.0
        {
            return Err(OutputValidationError::invalid_options(
                "SVG font family must be non-empty and font size must be finite and positive",
            ));
        }
        Ok(())
    }
}

/// Pixel dimensions and optional density metadata for PNG or raw RGBA output.
///
/// Width and height are the exact output pixel dimensions. DPI is metadata; it
/// does not silently change those dimensions. A renderer preserves the scene
/// aspect ratio and centers it in the requested viewport.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RasterOptions {
    pub width_px: u32,
    pub height_px: u32,
    pub dpi: Option<f64>,
    pub background: RgbaColor,
}

impl RasterOptions {
    pub const fn new(width_px: u32, height_px: u32) -> Self {
        Self {
            width_px,
            height_px,
            dpi: None,
            background: RgbaColor::WHITE,
        }
    }

    fn validate(self) -> Result<(), OutputValidationError> {
        if self.width_px == 0 || self.height_px == 0 {
            return Err(OutputValidationError::invalid_options(
                "raster width and height must be greater than zero",
            ));
        }
        let row_bytes = (self.width_px as usize).checked_mul(4).ok_or_else(|| {
            OutputValidationError::invalid_options(
                "raster dimensions exceed the addressable buffer size",
            )
        })?;
        row_bytes.checked_mul(self.height_px as usize).ok_or_else(|| {
            OutputValidationError::invalid_options(
                "raster dimensions exceed the addressable buffer size",
            )
        })?;
        if let Some(dpi) = self.dpi {
            if !dpi.is_finite() || dpi <= 0.0 {
                return Err(OutputValidationError::invalid_options(
                    "raster DPI must be finite and greater than zero",
                ));
            }
        }
        Ok(())
    }
}

/// Format-specific options for a renderer invocation.
#[derive(Debug, Clone, PartialEq)]
pub enum OutputOptions {
    Svg(SvgOptions),
    Png(RasterOptions),
    Rgba(RasterOptions),
}

impl OutputOptions {
    pub const fn format(&self) -> OutputFormat {
        match self {
            Self::Svg(_) => OutputFormat::Svg,
            Self::Png(_) => OutputFormat::Png,
            Self::Rgba(_) => OutputFormat::Rgba,
        }
    }

    pub fn validate(&self) -> Result<(), OutputValidationError> {
        match self {
            Self::Svg(options) => options.validate(),
            Self::Png(options) | Self::Rgba(options) => options.validate(),
        }
    }
}

/// Pixel layout for an owned raw raster buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    Rgba8,
}

/// Alpha representation for raw pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
    Straight,
}

/// Color space for raw pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
}

/// An owned, top-left-origin, row-major RGBA8 pixel buffer.
///
/// Rows may include padding. stride_bytes is the distance between row starts;
/// pixels must contain exactly stride_bytes * height_px bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RgbaBuffer {
    pub width_px: u32,
    pub height_px: u32,
    pub stride_bytes: usize,
    pub pixel_format: PixelFormat,
    pub alpha_mode: AlphaMode,
    pub color_space: ColorSpace,
    pub dpi: Option<f64>,
    pub pixels: Vec<u8>,
}

impl RgbaBuffer {
    pub fn validate(&self) -> Result<(), OutputValidationError> {
        if self.width_px == 0 || self.height_px == 0 {
            return Err(OutputValidationError::invalid_buffer(
                "RGBA width and height must be greater than zero",
            ));
        }
        let row_bytes = (self.width_px as usize).checked_mul(4).ok_or_else(|| {
            OutputValidationError::invalid_buffer("RGBA row size exceeds the addressable buffer size")
        })?;
        if self.stride_bytes < row_bytes {
            return Err(OutputValidationError::invalid_buffer(
                "RGBA stride must be at least width_px * 4",
            ));
        }
        let expected = self.stride_bytes.checked_mul(self.height_px as usize).ok_or_else(|| {
            OutputValidationError::invalid_buffer("RGBA buffer size exceeds the addressable buffer size")
        })?;
        if self.pixels.len() != expected {
            return Err(OutputValidationError::invalid_buffer(format!(
                "RGBA buffer has {} bytes; expected {expected}",
                self.pixels.len()
            )));
        }
        if let Some(dpi) = self.dpi {
            if !dpi.is_finite() || dpi <= 0.0 {
                return Err(OutputValidationError::invalid_buffer(
                    "RGBA DPI metadata must be finite and greater than zero",
                ));
            }
        }
        Ok(())
    }
}

/// Renderer output. Pixel buffers and encoded bytes are owned by the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderOutput {
    Svg(String),
    Png(Vec<u8>),
    Rgba(RgbaBuffer),
}

impl RenderOutput {
    pub const fn format(&self) -> OutputFormat {
        match self {
            Self::Svg(_) => OutputFormat::Svg,
            Self::Png(_) => OutputFormat::Png,
            Self::Rgba(_) => OutputFormat::Rgba,
        }
    }

    pub fn validate(&self) -> Result<(), OutputValidationError> {
        match self {
            Self::Svg(svg) if svg.is_empty() => Err(OutputValidationError::invalid_buffer(
                "SVG output must not be empty",
            )),
            Self::Png(png) if !png.starts_with(b"\x89PNG\r\n\x1a\n") => {
                Err(OutputValidationError::invalid_buffer(
                    "PNG output must contain the PNG signature",
                ))
            }
            Self::Rgba(buffer) => buffer.validate(),
            _ => Ok(()),
        }
    }

    pub fn validate_for(&self, options: &OutputOptions) -> Result<(), OutputValidationError> {
        if self.format() != options.format() {
            return Err(OutputValidationError {
                code: "renderer.output.format_mismatch",
                message: format!(
                    "renderer returned {} for requested {} output",
                    self.format().as_str(),
                    options.format().as_str()
                ),
            });
        }
        self.validate()
    }
}

/// A renderer consumes an immutable scene input and produces one declared output.
///
/// The generic input permits transitional adapters while the core Scene type is
/// introduced. New shared renderers should accept the typed core Scene directly.
/// Concrete renderers must validate OutputOptions before allocating output,
/// return an output variant matching options.format(), and map failures to
/// the shared diagnostic-v1 contract in their runtime adapter. Capability
/// negotiation remains governed by renderer-capabilities-v1.
pub trait Renderer<Input> {
    type Error;

    fn render(
        &self,
        input: &Input,
        options: &OutputOptions,
    ) -> Result<RenderOutput, Self::Error>;
}

/// A renderer output contract violation that can be mapped to diagnostic-v1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputValidationError {
    pub code: &'static str,
    pub message: String,
}

impl OutputValidationError {
    pub fn invalid_options(message: impl Into<String>) -> Self {
        Self {
            code: "renderer.output.invalid_options",
            message: message.into(),
        }
    }

    pub fn invalid_buffer(message: impl Into<String>) -> Self {
        Self {
            code: "renderer.output.invalid_buffer",
            message: message.into(),
        }
    }
}

impl fmt::Display for OutputValidationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for OutputValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_options_require_finite_positive_physical_dimensions() {
        let valid = OutputOptions::Svg(SvgOptions::new(180.0, 120.0, SvgUnit::Millimeters));
        assert!(valid.validate().is_ok());

        let invalid = OutputOptions::Svg(SvgOptions::new(f64::INFINITY, 120.0, SvgUnit::Millimeters));
        assert_eq!(invalid.validate().unwrap_err().code, "renderer.output.invalid_options");
    }

    #[test]
    fn raster_options_validate_dimensions_and_density() {
        let valid = OutputOptions::Png(RasterOptions {
            width_px: 640,
            height_px: 480,
            dpi: Some(300.0),
            background: RgbaColor::WHITE,
        });
        assert!(valid.validate().is_ok());

        let zero_size = OutputOptions::Rgba(RasterOptions::new(0, 480));
        assert_eq!(zero_size.validate().unwrap_err().code, "renderer.output.invalid_options");

        let invalid_dpi = OutputOptions::Png(RasterOptions {
            width_px: 640,
            height_px: 480,
            dpi: Some(f64::NAN),
            background: RgbaColor::WHITE,
        });
        assert_eq!(invalid_dpi.validate().unwrap_err().code, "renderer.output.invalid_options");
    }

    #[test]
    fn rgba_buffer_checks_stride_and_exact_owned_buffer_length() {
        let valid = RgbaBuffer {
            width_px: 2,
            height_px: 2,
            stride_bytes: 8,
            pixel_format: PixelFormat::Rgba8,
            alpha_mode: AlphaMode::Straight,
            color_space: ColorSpace::Srgb,
            dpi: Some(96.0),
            pixels: vec![0; 16],
        };
        assert!(valid.validate().is_ok());

        let invalid = RgbaBuffer { pixels: vec![0; 15], ..valid };
        assert_eq!(invalid.validate().unwrap_err().code, "renderer.output.invalid_buffer");
    }

    #[test]
    fn output_variants_report_their_declared_format() {
        assert_eq!(RenderOutput::Svg("<svg/>".into()).format(), OutputFormat::Svg);
        assert_eq!(RenderOutput::Png(b"\x89PNG\r\n\x1a\n".to_vec()).format(), OutputFormat::Png);
        assert_eq!(OutputOptions::Rgba(RasterOptions::new(2, 2)).format(), OutputFormat::Rgba);
        let requested = OutputOptions::Svg(SvgOptions::new(100.0, 100.0, SvgUnit::Millimeters));
        assert_eq!(
            RenderOutput::Png(b"\x89PNG\r\n\x1a\n".to_vec()).validate_for(&requested).unwrap_err().code,
            "renderer.output.format_mismatch"
        );
    }
}
