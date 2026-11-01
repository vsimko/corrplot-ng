//! Direct raster rendering from a renderer-neutral scene.
//!
//! The renderer writes owned sRGB RGBA8 pixels. It does not serialize or
//! reparse SVG. Coordinates are logical scene units and are uniformly scaled
//! into the requested viewport with letterboxing rather than cropping.

use crate::output::{
    AlphaMode, ColorSpace, OutputOptions, OutputValidationError, PixelFormat, RasterOptions,
    RenderOutput, Renderer, RgbaBuffer, RgbaColor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneText {
    pub text: String,
    pub x: f64,
    pub y: f64,
    /// Logical scene units for the full seven-row bitmap font.
    pub font_size: f64,
    pub color: RgbaColor,
    pub anchor: TextAnchor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellGlyph {
    Circle { radius: f64, color: RgbaColor },
    Square { side: f64, color: RgbaColor },
    Ellipse { rx: f64, ry: f64, rotation_degrees: f64, color: RgbaColor },
    Pie { radius: f64, start_radians: f64, end_radians: f64, color: RgbaColor },
    Number { text: String, font_size: f64, color: RgbaColor },
    Shade { color: RgbaColor },
    /// The cell's semantic fill is the complete glyph for the color method.
    Color,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneLegendEntry {
    pub kind: String,
    pub label: String,
    pub color: RgbaColor,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneClusterRegion {
    pub id: String,
    pub variables: Vec<String>,
    pub source_indices: Vec<usize>,
    pub row_start: usize,
    pub row_end: usize,
    pub column_start: usize,
    pub column_end: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: RgbaColor,
    pub line_width: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SceneCell {
    pub x: f64,
    pub y: f64,
    pub size: f64,
    pub visible: bool,
    pub fill: RgbaColor,
    pub stroke: RgbaColor,
    pub glyph: Option<CellGlyph>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub width: f64,
    pub height: f64,
    pub cells: Vec<SceneCell>,
    pub texts: Vec<SceneText>,
    /// Renderer-neutral legend semantics, retained independently of text labels.
    pub legend: Vec<SceneLegendEntry>,
    /// Source and displayed matrix ranges for cluster overlays.
    pub cluster_regions: Vec<SceneClusterRegion>,
}

impl Scene {
    pub fn validate(&self) -> Result<(), OutputValidationError> {
        if !self.width.is_finite() || self.width <= 0.0
            || !self.height.is_finite() || self.height <= 0.0
        {
            return Err(OutputValidationError::invalid_options(
                "scene width and height must be finite and positive",
            ));
        }
        for (index, cell) in self.cells.iter().enumerate() {
            if !cell.x.is_finite() || !cell.y.is_finite()
                || !cell.size.is_finite() || cell.size <= 0.0
            {
                return Err(OutputValidationError::invalid_options(format!(
                    "scene cell {index} has invalid geometry",
                )));
            }
            match &cell.glyph {
                Some(CellGlyph::Circle { radius, .. }) if !radius.is_finite() || *radius < 0.0 =>
                    return Err(OutputValidationError::invalid_options(format!("scene cell {index} has an invalid circle radius"))),
                Some(CellGlyph::Square { side, .. }) if !side.is_finite() || *side < 0.0 =>
                    return Err(OutputValidationError::invalid_options(format!("scene cell {index} has an invalid square side"))),
                Some(CellGlyph::Ellipse { rx, ry, rotation_degrees, .. }) if !rx.is_finite() || !ry.is_finite() || !rotation_degrees.is_finite() || *rx < 0.0 || *ry < 0.0 =>
                    return Err(OutputValidationError::invalid_options(format!("scene cell {index} has invalid ellipse radii"))),
                Some(CellGlyph::Pie { radius, start_radians, end_radians, .. })
                    if !radius.is_finite() || *radius < 0.0 || !start_radians.is_finite() || !end_radians.is_finite() || !(end_radians - start_radians).is_finite() =>
                    return Err(OutputValidationError::invalid_options(format!("scene cell {index} has invalid pie geometry"))),
                Some(CellGlyph::Number { font_size, .. }) if !font_size.is_finite() || *font_size <= 0.0 =>
                    return Err(OutputValidationError::invalid_options(format!("scene cell {index} has invalid number font size"))),
                _ => {}
            }
        }
        for (index, region) in self.cluster_regions.iter().enumerate() {
            if region.id.is_empty() || region.variables.is_empty()
                || region.variables.len() != region.source_indices.len()
                || region.row_start > region.row_end
                || region.column_start > region.column_end
                || !region.x.is_finite() || !region.y.is_finite()
                || !region.width.is_finite() || region.width <= 0.0
                || !region.height.is_finite() || region.height <= 0.0
                || !region.line_width.is_finite() || region.line_width <= 0.0
            {
                return Err(OutputValidationError::invalid_options(format!(
                    "scene cluster region {index} has invalid metadata or geometry",
                )));
            }
        }
        for (index, text) in self.texts.iter().enumerate() {
            if !text.x.is_finite() || !text.y.is_finite()
                || !text.font_size.is_finite() || text.font_size <= 0.0
            {
                return Err(OutputValidationError::invalid_options(format!(
                    "scene text {index} has invalid position or font size",
                )));
            }
        }
        Ok(())
    }
}

/// A dependency-free renderer for the shared logical scene.
pub struct RgbaRenderer;

impl Renderer<Scene> for RgbaRenderer {
    type Error = OutputValidationError;

    fn render(
        &self,
        scene: &Scene,
        options: &OutputOptions,
    ) -> Result<RenderOutput, Self::Error> {
        options.validate()?;
        scene.validate()?;
        let OutputOptions::Rgba(raster) = options else {
            return Err(OutputValidationError::invalid_options(
                "RGBA renderer supports only raw RGBA output",
            ));
        };
        let buffer = render_rgba(scene, *raster)?;
        let output = RenderOutput::Rgba(buffer);
        output.validate_for(options)?;
        Ok(output)
    }
}

const MAX_RASTER_BYTES: usize = 128 * 1024 * 1024;

pub fn render_rgba(
    scene: &Scene,
    options: RasterOptions,
) -> Result<RgbaBuffer, OutputValidationError> {
    options.validate()?;
    scene.validate()?;
    let row_bytes = (options.width_px as usize).checked_mul(4).ok_or_else(|| {
        OutputValidationError::invalid_options("raster row size exceeds addressable memory")
    })?;
    let byte_len = row_bytes.checked_mul(options.height_px as usize).ok_or_else(|| {
        OutputValidationError::invalid_options("raster buffer size exceeds addressable memory")
    })?;
    if byte_len > MAX_RASTER_BYTES {
        return Err(OutputValidationError::invalid_options(format!(
            "RGBA output exceeds the {} MiB per-render allocation limit",
            MAX_RASTER_BYTES / (1024 * 1024),
        )));
    }
    let mut canvas = Canvas {
        width: options.width_px as usize,
        height: options.height_px as usize,
        pixels: vec![0; byte_len],
        scale: (options.width_px as f64 / scene.width)
            .min(options.height_px as f64 / scene.height),
        offset_x: 0.0,
        offset_y: 0.0,
    };
    canvas.offset_x = (options.width_px as f64 - scene.width * canvas.scale) / 2.0;
    canvas.offset_y = (options.height_px as f64 - scene.height * canvas.scale) / 2.0;
    canvas.fill(options.background);

    for cell in &scene.cells {
        if !cell.visible {
            continue;
        }
        let x = cell.x;
        let y = cell.y;
        let size = cell.size;
        canvas.fill_rect(x, y, x + size, y + size, cell.fill);
        canvas.stroke_rect(x, y, x + size, y + size, cell.stroke);
        let cx = x + size / 2.0;
        let cy = y + size / 2.0;
        match &cell.glyph {
            Some(CellGlyph::Circle { radius, color }) => canvas.fill_circle(cx, cy, *radius, *color),
            Some(CellGlyph::Square { side, color }) => {
                let half = side / 2.0;
                canvas.fill_rect(cx - half, cy - half, cx + half, cy + half, *color);
            }
            Some(CellGlyph::Ellipse { rx, ry, rotation_degrees, color }) => canvas.fill_ellipse(cx, cy, *rx, *ry, *rotation_degrees, *color),
            Some(CellGlyph::Pie { radius, start_radians, end_radians, color }) =>
                canvas.fill_pie(cx, cy, *radius, *start_radians, *end_radians, *color),
            Some(CellGlyph::Number { text, font_size, color }) =>
                canvas.draw_text(text, cx, cy - font_size / 2.0, *font_size, *color, TextAnchor::Middle),
            Some(CellGlyph::Shade { color }) =>
                canvas.draw_line(x, y + size, x + size, y, *color),
            Some(CellGlyph::Color) | None => {}
        }
    }
    for text in &scene.texts {
        canvas.draw_text(&text.text, text.x, text.y, text.font_size, text.color, text.anchor);
    }

    Ok(RgbaBuffer {
        width_px: options.width_px,
        height_px: options.height_px,
        stride_bytes: row_bytes,
        pixel_format: PixelFormat::Rgba8,
        alpha_mode: AlphaMode::Straight,
        color_space: ColorSpace::Srgb,
        dpi: options.dpi,
        pixels: canvas.pixels,
    })
}

struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
}

impl Canvas {
    fn point(&self, x: f64, y: f64) -> (f64, f64) {
        let bound = (self.width.max(self.height) as f64 * 4.0 + 10.0).max(10.0);
        (
            (self.offset_x + x * self.scale).clamp(-bound, bound),
            (self.offset_y + y * self.scale).clamp(-bound, bound),
        )
    }

    fn fill(&mut self, color: RgbaColor) {
        for pixel in self.pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[color.red, color.green, color.blue, color.alpha]);
        }
    }

    fn blend_pixel(&mut self, x: isize, y: isize, source: RgbaColor) {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return;
        }
        let index = (y as usize * self.width + x as usize) * 4;
        let destination = RgbaColor {
            red: self.pixels[index],
            green: self.pixels[index + 1],
            blue: self.pixels[index + 2],
            alpha: self.pixels[index + 3],
        };
        let sa = source.alpha as f64 / 255.0;
        let da = destination.alpha as f64 / 255.0;
        let out_a = sa + da * (1.0 - sa);
        if out_a <= 0.0 {
            self.pixels[index..index + 4].copy_from_slice(&[0, 0, 0, 0]);
            return;
        }
        let blend = |src: u8, dst: u8| {
            ((src as f64 * sa + dst as f64 * da * (1.0 - sa)) / out_a)
                .round().clamp(0.0, 255.0) as u8
        };
        self.pixels[index] = blend(source.red, destination.red);
        self.pixels[index + 1] = blend(source.green, destination.green);
        self.pixels[index + 2] = blend(source.blue, destination.blue);
        self.pixels[index + 3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    }

    fn fill_rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: RgbaColor) {
        let (x0, y0) = self.point(x0, y0);
        let (x1, y1) = self.point(x1, y1);
        let left = x0.floor().max(0.0) as isize;
        let top = y0.floor().max(0.0) as isize;
        let right = x1.ceil().min(self.width as f64) as isize;
        let bottom = y1.ceil().min(self.height as f64) as isize;
        for y in top..bottom {
            for x in left..right {
                self.blend_pixel(x, y, color);
            }
        }
    }

    fn stroke_rect(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: RgbaColor) {
        self.draw_line(x0, y0, x1, y0, color);
        self.draw_line(x1, y0, x1, y1, color);
        self.draw_line(x1, y1, x0, y1, color);
        self.draw_line(x0, y1, x0, y0, color);
    }

    fn draw_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, color: RgbaColor) {
        let (x0, y0) = self.point(x0, y0);
        let (x1, y1) = self.point(x1, y1);
        let mut x = x0.round() as isize;
        let mut y = y0.round() as isize;
        let end_x = x1.round() as isize;
        let end_y = y1.round() as isize;
        let dx = (end_x - x).abs();
        let sx = if x < end_x { 1 } else { -1 };
        let dy = -(end_y - y).abs();
        let sy = if y < end_y { 1 } else { -1 };
        let mut error = dx + dy;
        loop {
            self.blend_pixel(x, y, color);
            if x == end_x && y == end_y { break; }
            let twice = 2 * error;
            if twice >= dy { error += dy; x += sx; }
            if twice <= dx { error += dx; y += sy; }
        }
    }

    fn fill_circle(&mut self, cx: f64, cy: f64, radius: f64, color: RgbaColor) {
        let (cx, cy) = self.point(cx, cy);
        let radius = radius * self.scale;
        let left = (cx - radius).floor().max(0.0) as isize;
        let right = (cx + radius).ceil().min(self.width as f64) as isize;
        let top = (cy - radius).floor().max(0.0) as isize;
        let bottom = (cy + radius).ceil().min(self.height as f64) as isize;
        let r2 = radius * radius;
        for y in top..bottom {
            for x in left..right {
                let dx = x as f64 + 0.5 - cx;
                let dy = y as f64 + 0.5 - cy;
                if dx * dx + dy * dy <= r2 { self.blend_pixel(x, y, color); }
            }
        }
    }

    fn fill_ellipse(&mut self, cx: f64, cy: f64, rx: f64, ry: f64, rotation_degrees: f64, color: RgbaColor) {
        if rx <= 0.0 || ry <= 0.0 { return; }
        let (cx, cy) = self.point(cx, cy);
        let rx = rx * self.scale;
        let ry = ry * self.scale;
        let angle = rotation_degrees.to_radians();
        let cos = angle.cos();
        let sin = angle.sin();
        let extent_x = ((rx * cos).powi(2) + (ry * sin).powi(2)).sqrt();
        let extent_y = ((rx * sin).powi(2) + (ry * cos).powi(2)).sqrt();
        let left = (cx - extent_x).floor().max(0.0) as isize;
        let right = (cx + extent_x).ceil().min(self.width as f64) as isize;
        let top = (cy - extent_y).floor().max(0.0) as isize;
        let bottom = (cy + extent_y).ceil().min(self.height as f64) as isize;
        for y in top..bottom {
            for x in left..right {
                let dx = x as f64 + 0.5 - cx;
                let dy = y as f64 + 0.5 - cy;
                let local_x = (dx * cos + dy * sin) / rx;
                let local_y = (-dx * sin + dy * cos) / ry;
                if local_x * local_x + local_y * local_y <= 1.0 { self.blend_pixel(x, y, color); }
            }
        }
    }

    fn fill_pie(&mut self, cx: f64, cy: f64, radius: f64, start: f64, end: f64, color: RgbaColor) {
        let (cx, cy) = self.point(cx, cy);
        let radius = radius * self.scale;
        let left = (cx - radius).floor().max(0.0) as isize;
        let right = (cx + radius).ceil().min(self.width as f64) as isize;
        let top = (cy - radius).floor().max(0.0) as isize;
        let bottom = (cy + radius).ceil().min(self.height as f64) as isize;
        let full = (end - start).abs() >= std::f64::consts::TAU;
        for y in top..bottom {
            for x in left..right {
                let dx = x as f64 + 0.5 - cx;
                let dy = y as f64 + 0.5 - cy;
                if dx * dx + dy * dy > radius * radius { continue; }
                let angle = dy.atan2(dx);
                let within = if full { true } else {
                    let delta = end - start;
                    if delta >= 0.0 {
                        (angle - start).rem_euclid(std::f64::consts::TAU) <= delta.rem_euclid(std::f64::consts::TAU)
                    } else {
                        (start - angle).rem_euclid(std::f64::consts::TAU) <= (-delta).rem_euclid(std::f64::consts::TAU)
                    }
                };
                if within { self.blend_pixel(x, y, color); }
            }
        }
    }

    fn draw_text(&mut self, text: &str, x: f64, y: f64, font_size: f64, color: RgbaColor, anchor: TextAnchor) {
        let cell = font_size * self.scale / 7.0;
        if cell <= 0.0 || !cell.is_finite() { return; }
        let width = text.chars().count() as f64 * 6.0 * cell - cell;
        let (x, y) = self.point(x, y);
        let start_x = match anchor {
            TextAnchor::Start => x,
            TextAnchor::Middle => x - width / 2.0,
            TextAnchor::End => x - width,
        };
        for (index, character) in text.chars().enumerate() {
            let rows = bitmap_glyph(character);
            for (row, bits) in rows.iter().enumerate() {
                for column in 0..5 {
                    if bits & (1 << (4 - column)) != 0 {
                        let left = start_x + (index as f64 * 6.0 + column as f64) * cell;
                        let top = y + row as f64 * cell;
                        let right = left + cell;
                        let bottom = top + cell;
                        let x0 = left.floor().max(0.0) as isize;
                        let y0 = top.floor().max(0.0) as isize;
                        let x1 = right.ceil().min(self.width as f64) as isize;
                        let y1 = bottom.ceil().min(self.height as f64) as isize;
                        for py in y0..y1 {
                            for px in x0..x1 { self.blend_pixel(px, py, color); }
                        }
                    }
                }
            }
        }
    }
}

fn bitmap_glyph(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [14,17,17,31,17,17,17], 'B' => [30,17,17,30,17,17,30],
        'C' => [14,17,16,16,16,17,14], 'D' => [30,17,17,17,17,17,30],
        'E' => [31,16,16,30,16,16,31], 'F' => [31,16,16,30,16,16,16],
        'G' => [14,17,16,23,17,17,15], 'H' => [17,17,17,31,17,17,17],
        'I' => [14,4,4,4,4,4,14], 'J' => [7,2,2,2,18,18,12],
        'K' => [17,18,20,24,20,18,17], 'L' => [16,16,16,16,16,16,31],
        'M' => [17,27,21,21,17,17,17], 'N' => [17,25,21,19,17,17,17],
        'O' => [14,17,17,17,17,17,14], 'P' => [30,17,17,30,16,16,16],
        'Q' => [14,17,17,17,21,18,13], 'R' => [30,17,17,30,20,18,17],
        'S' => [15,16,16,14,1,1,30], 'T' => [31,4,4,4,4,4,4],
        'U' => [17,17,17,17,17,17,14], 'V' => [17,17,17,17,17,10,4],
        'W' => [17,17,17,21,21,21,10], 'X' => [17,17,10,4,10,17,17],
        'Y' => [17,17,10,4,4,4,4], 'Z' => [31,1,2,4,8,16,31],
        '0' => [14,17,19,21,25,17,14], '1' => [4,12,4,4,4,4,14],
        '2' => [14,17,1,2,4,8,31], '3' => [30,1,1,14,1,1,30],
        '4' => [2,6,10,18,31,2,2], '5' => [31,16,16,30,1,1,30],
        '6' => [14,16,16,30,17,17,14], '7' => [31,1,2,4,8,8,8],
        '8' => [14,17,17,14,17,17,14], '9' => [14,17,17,15,1,1,14],
        '-' => [0,0,0,31,0,0,0], '+' => [0,4,4,31,4,4,0],
        '.' => [0,0,0,0,0,12,12], ',' => [0,0,0,0,4,4,8],
        ':' => [0,12,12,0,12,12,0], '/' => [1,1,2,4,8,16,16],
        '(' => [2,4,8,8,8,4,2], ')' => [8,4,2,2,2,4,8],
        '%' => [17,2,4,8,16,17,0], '_' => [0,0,0,0,0,0,31],
        ' ' => [0; 7], _ => [31,17,5,4,4,0,4],
    }
}

/// SVG renderer for the same typed scene consumed by `RgbaRenderer`.
pub struct SceneSvgRenderer;

impl Renderer<Scene> for SceneSvgRenderer {
    type Error = OutputValidationError;

    fn render(
        &self,
        scene: &Scene,
        options: &OutputOptions,
    ) -> Result<RenderOutput, Self::Error> {
        options.validate()?;
        scene.validate()?;
        let OutputOptions::Svg(svg) = options else {
            return Err(OutputValidationError::invalid_options(
                "scene SVG renderer supports only SVG output",
            ));
        };
        let unit = svg.unit.as_str();
        let mut out = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}{}\" height=\"{}{}\" viewBox=\"0 0 {} {}\" preserveAspectRatio=\"xMidYMid meet\">",
            svg.width, unit, svg.height, unit, scene.width, scene.height
        );
        out.push_str(&format!(
            "<rect width=\"100%\" height=\"100%\" fill=\"{}\" fill-opacity=\"{}\"/>",
            color_hex(svg.background),
            alpha(svg.background),
        ));
        for cell in &scene.cells {
            if !cell.visible { continue; }
            out.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" fill-opacity=\"{}\" stroke=\"{}\" stroke-opacity=\"{}\"/>",
                cell.x, cell.y, cell.size, cell.size, color_hex(cell.fill), alpha(cell.fill), color_hex(cell.stroke), alpha(cell.stroke)
            ));
            let cx = cell.x + cell.size / 2.0;
            let cy = cell.y + cell.size / 2.0;
            match &cell.glyph {
                Some(CellGlyph::Circle { radius, color }) => out.push_str(&format!(
                    "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{radius}\" fill=\"{}\" fill-opacity=\"{}\"/>",
                    color_hex(*color), alpha(*color)
                )),
                Some(CellGlyph::Square { side, color }) => out.push_str(&format!(
                    "<rect x=\"{}\" y=\"{}\" width=\"{side}\" height=\"{side}\" fill=\"{}\" fill-opacity=\"{}\"/>",
                    cx - side / 2.0, cy - side / 2.0, color_hex(*color), alpha(*color)
                )),
                Some(CellGlyph::Ellipse { rx, ry, rotation_degrees, color }) => out.push_str(&format!(
                    "<ellipse cx=\"{cx}\" cy=\"{cy}\" rx=\"{rx}\" ry=\"{ry}\" transform=\"rotate({rotation_degrees} {cx} {cy})\" fill=\"{}\" fill-opacity=\"{}\"/>",
                    color_hex(*color), alpha(*color)
                )),
                Some(CellGlyph::Pie { radius, start_radians, end_radians, color }) => {
                    let start_x = cx + radius * start_radians.cos();
                    let start_y = cy + radius * start_radians.sin();
                    let end_x = cx + radius * end_radians.cos();
                    let end_y = cy + radius * end_radians.sin();
                    let span = (end_radians - start_radians).abs();
                    if span >= std::f64::consts::TAU {
                        out.push_str(&format!(
                            "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{radius}\" fill=\"{}\" fill-opacity=\"{}\"/>",
                            color_hex(*color), alpha(*color)
                        ));
                    } else {
                        let large_arc = if span > std::f64::consts::PI { 1 } else { 0 };
                        let sweep = if end_radians >= start_radians { 1 } else { 0 };
                        out.push_str(&format!(
                            "<path d=\"M {cx} {cy} L {start_x} {start_y} A {radius} {radius} 0 {large_arc} {sweep} {end_x} {end_y} Z\" fill=\"{}\" fill-opacity=\"{}\"/>",
                            color_hex(*color), alpha(*color)
                        ));
                    }
                }
                Some(CellGlyph::Number { text, font_size, color }) => out.push_str(&format!(
                    "<text x=\"{cx}\" y=\"{cy}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" fill-opacity=\"{}\">{}</text>",
                    escape_xml(&svg.font_family), font_size, color_hex(*color), alpha(*color), escape_xml(text)
                )),
                Some(CellGlyph::Shade { color }) => out.push_str(&format!(
                    "<path d=\"M {} {} L {} {}\" stroke=\"{}\" stroke-opacity=\"{}\" fill=\"none\"/>",
                    cell.x, cell.y + cell.size, cell.x + cell.size, cell.y, color_hex(*color), alpha(*color)
                )),
                Some(CellGlyph::Color) | None => {}
            }
        }
        for text in &scene.texts {
            let anchor = match text.anchor {
                TextAnchor::Start => "start",
                TextAnchor::Middle => "middle",
                TextAnchor::End => "end",
            };
            out.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" text-anchor=\"{}\" dominant-baseline=\"hanging\" font-family=\"{}\" font-size=\"{}\" fill=\"{}\" fill-opacity=\"{}\">{}</text>",
                text.x, text.y, anchor, escape_xml(&svg.font_family), text.font_size,
                color_hex(text.color), alpha(text.color), escape_xml(&text.text)
            ));
        }
        out.push_str("</svg>");
        let output = RenderOutput::Svg(out);
        output.validate_for(options)?;
        Ok(output)
    }
}

fn color_hex(color: RgbaColor) -> String {
    format!("#{:02x}{:02x}{:02x}", color.red, color.green, color.blue)
}

fn alpha(color: RgbaColor) -> String {
    format!("{:.6}", color.alpha as f64 / 255.0)
}

fn escape_xml(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
        .replace('"', "&quot;").replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scene() -> Scene {
        Scene {
            width: 100.0,
            height: 100.0,
            cells: vec![
                SceneCell {
                    x: 20.0, y: 20.0, size: 30.0, visible: true,
                    fill: RgbaColor::WHITE,
                    stroke: RgbaColor { red: 0, green: 0, blue: 0, alpha: 255 },
                    glyph: Some(CellGlyph::Circle {
                        radius: 10.0,
                        color: RgbaColor { red: 20, green: 80, blue: 160, alpha: 255 },
                    }),
                },
                SceneCell {
                    x: 60.0, y: 20.0, size: 30.0, visible: true,
                    fill: RgbaColor::WHITE,
                    stroke: RgbaColor { red: 0, green: 0, blue: 0, alpha: 255 },
                    glyph: Some(CellGlyph::Number {
                        text: "-0.5".into(), font_size: 12.0,
                        color: RgbaColor { red: 0, green: 0, blue: 0, alpha: 255 },
                    }),
                },
            ],
            legend: vec![],
            cluster_regions: vec![],
            texts: vec![SceneText {
                text: "A".into(), x: 10.0, y: 10.0, font_size: 10.0,
                color: RgbaColor { red: 0, green: 0, blue: 0, alpha: 255 },
                anchor: TextAnchor::Start,
            }],
        }
    }

    #[test]
    fn renders_owned_rgba_with_requested_dimensions_stride_and_dpi() {
        let scene = sample_scene();
        let options = OutputOptions::Rgba(RasterOptions {
            width_px: 200, height_px: 100, dpi: Some(144.0),
            background: RgbaColor::WHITE,
        });
        let output = RgbaRenderer.render(&scene, &options).unwrap();
        let RenderOutput::Rgba(buffer) = output else { panic!("expected RGBA output") };
        assert_eq!((buffer.width_px, buffer.height_px), (200, 100));
        assert_eq!(buffer.stride_bytes, 800);
        assert_eq!(buffer.pixels.len(), 80_000);
        assert_eq!(buffer.dpi, Some(144.0));
        assert_eq!(buffer.alpha_mode, AlphaMode::Straight);
        assert_eq!(buffer.color_space, ColorSpace::Srgb);
    }

    #[test]
    fn preserves_transparent_background_and_composites_visible_glyphs() {
        let scene = sample_scene();
        let options = RasterOptions {
            width_px: 100, height_px: 100, dpi: None,
            background: RgbaColor::TRANSPARENT,
        };
        let buffer = render_rgba(&scene, options).unwrap();
        let background = (0 * buffer.stride_bytes) + 0;
        assert_eq!(&buffer.pixels[background..background + 4], &[0, 0, 0, 0]);
        let center = ((35 * buffer.stride_bytes) + 35 * 4);
        assert_eq!(&buffer.pixels[center..center + 4], &[20, 80, 160, 255]);
    }

    #[test]
    fn clips_geometry_to_the_requested_viewport_and_skips_hidden_cells() {
        let mut scene = sample_scene();
        scene.cells[0].x = -40.0;
        scene.cells[1].visible = false;
        let buffer = render_rgba(&scene, RasterOptions::new(100, 100)).unwrap();
        assert_eq!(buffer.pixels.len(), 40_000);
        assert_eq!(&buffer.pixels[(35 * buffer.stride_bytes) + 35 * 4..][..4], &[255, 255, 255, 255]);
    }

    #[test]
    fn supports_all_declared_glyph_primitives_without_svg_round_trip() {
        let mut scene = sample_scene();
        let color = RgbaColor { red: 220, green: 20, blue: 60, alpha: 255 };
        scene.cells.extend([
            SceneCell { x: 10.0, y: 60.0, size: 15.0, visible: true, fill: RgbaColor::WHITE, stroke: color,
                glyph: Some(CellGlyph::Square { side: 10.0, color }) },
            SceneCell { x: 30.0, y: 60.0, size: 15.0, visible: true, fill: RgbaColor::WHITE, stroke: color,
                glyph: Some(CellGlyph::Ellipse { rx: 6.0, ry: 3.0, rotation_degrees: 25.0, color }) },
            SceneCell { x: 50.0, y: 60.0, size: 15.0, visible: true, fill: RgbaColor::WHITE, stroke: color,
                glyph: Some(CellGlyph::Pie { radius: 6.0, start_radians: 0.0, end_radians: 2.0, color }) },
            SceneCell { x: 70.0, y: 60.0, size: 15.0, visible: true, fill: RgbaColor::WHITE, stroke: color,
                glyph: Some(CellGlyph::Shade { color }) },
            SceneCell { x: 85.0, y: 60.0, size: 10.0, visible: true, fill: color, stroke: color,
                glyph: Some(CellGlyph::Color) },
        ]);
        let buffer = render_rgba(&scene, RasterOptions::new(100, 100)).unwrap();
        assert!(buffer.pixels.chunks_exact(4).any(|pixel| pixel == [220, 20, 60, 255]));

        let svg_options = OutputOptions::Svg(crate::output::SvgOptions::new(100.0, 100.0, crate::output::SvgUnit::Pixels));
        let svg = SceneSvgRenderer.render(&scene, &svg_options).unwrap();
        let RenderOutput::Svg(svg) = svg else { panic!("expected SVG output") };
        assert!(svg.contains("rotate(25"));
    }

    #[test]
    fn svg_and_rgba_renderers_consume_the_same_scene() {
        let scene = sample_scene();
        let svg_options = OutputOptions::Svg(crate::output::SvgOptions::new(
            100.0, 100.0, crate::output::SvgUnit::Pixels,
        ));
        let svg = SceneSvgRenderer.render(&scene, &svg_options).unwrap();
        let RenderOutput::Svg(svg) = svg else { panic!("expected SVG output") };
        assert!(svg.contains("<circle"));
        assert!(svg.contains("-0.5"));
        assert!(svg.contains(">A</text>"));

        let rgba_options = OutputOptions::Rgba(RasterOptions::new(100, 100));
        let rgba = RgbaRenderer.render(&scene, &rgba_options).unwrap();
        let RenderOutput::Rgba(rgba) = rgba else { panic!("expected RGBA output") };
        assert_eq!((rgba.width_px, rgba.height_px), (100, 100));
        assert!(rgba.pixels.chunks_exact(4).any(|pixel| pixel == [20, 80, 160, 255]));
    }

    #[test]
    fn rejects_raster_allocations_above_the_documented_limit() {
        let scene = sample_scene();
        let options = RasterOptions::new(20_000, 20_000);
        assert_eq!(
            render_rgba(&scene, options).unwrap_err().code,
            "renderer.output.invalid_options",
        );
    }


    #[test]
    fn renderer_capability_declaration_matches_the_core_backends() {
        let capabilities = crate::RENDERER_CAPABILITIES_V1;
        assert!(capabilities.contains("\"id\": \"rust-core-scene\""));
        assert!(capabilities.contains("\"output_formats\": [\"svg\", \"rgba\"]"));
        assert!(capabilities.contains("\"plotmath_labels\": false"));
    }

    #[test]
    fn rejects_svg_options_and_invalid_scene_geometry() {
        let scene = sample_scene();
        let svg = OutputOptions::Svg(crate::output::SvgOptions::new(10.0, 10.0, crate::output::SvgUnit::Pixels));
        assert_eq!(RgbaRenderer.render(&scene, &svg).unwrap_err().code, "renderer.output.invalid_options");
        let mut invalid = scene;
        invalid.width = f64::NAN;
        assert_eq!(render_rgba(&invalid, RasterOptions::new(10, 10)).unwrap_err().code, "renderer.output.invalid_options");
    }
}
