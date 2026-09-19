//! Turning a generated picture into a sprite.
//!
//! This is the half of the pipeline that does not involve a provider, and it is
//! the half that decides whether the result looks like a 1993 shooter or like a
//! photograph someone shrank.
//!
//! The order matters and is not arbitrary.
//!
//! 1. **Trim to the alpha bounding box.** A generator centres its subject in
//!    whatever canvas it likes. Trimming first means the downscale ratio is
//!    decided by the sprite, not by the empty space around it.
//! 2. **Downscale by area averaging.** Every source pixel contributes to
//!    exactly one output pixel. Nearest-neighbour at this step would throw away
//!    most of the render and alias the detail that was paid for.
//! 3. **Quantise to the palette**, matched in CIE L\*a\*b\* rather than in RGB,
//!    because RGB distance does not match what an eye calls "the nearest
//!    colour" and picks visibly wrong swatches in dark and saturated regions.
//! 4. **Harden the alpha.** A sprite has no half-transparent edge. Anything
//!    ambiguous is pushed to fully on or fully off, which is what gives the
//!    silhouette its bite.
//!
//! Steps two and three are why the contract asks for generation at four times
//! the target: an exact integer ratio keeps the pixel grid square and aligned.

use std::path::Path;

use image::{imageops, GenericImageView, ImageBuffer, Rgba, RgbaImage};

use crate::Error;

/// Alpha at or below this is treated as empty when trimming.
const TRIM_ALPHA: u8 = 8;

/// Alpha above this becomes solid, at or below becomes empty.
const ALPHA_CUT: u8 = 128;

/// A colour in CIE L*a*b*.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lab {
    pub l: f32,
    pub a: f32,
    pub b: f32,
}

/// sRGB to linear light. The 0.04045 knee is the sRGB transfer function.
fn srgb_to_linear(channel: u8) -> f32 {
    let c = f32::from(channel) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// sRGB to CIE L*a*b* through XYZ, D65 white point.
pub fn to_lab(r: u8, g: u8, b: u8) -> Lab {
    let (rl, gl, bl) = (srgb_to_linear(r), srgb_to_linear(g), srgb_to_linear(b));

    let x = rl * 0.412_456_4 + gl * 0.357_576_1 + bl * 0.180_437_5;
    let y = rl * 0.212_672_9 + gl * 0.715_152_2 + bl * 0.072_175;
    let z = rl * 0.019_333_9 + gl * 0.119_192 + bl * 0.950_304_1;

    // D65 reference white.
    let (xn, yn, zn) = (0.950_49, 1.0, 1.088_84);
    let f = |t: f32| -> f32 {
        const DELTA: f32 = 6.0 / 29.0;
        if t > DELTA * DELTA * DELTA {
            t.cbrt()
        } else {
            t / (3.0 * DELTA * DELTA) + 4.0 / 29.0
        }
    };
    let (fx, fy, fz) = (f(x / xn), f(y / yn), f(z / zn));

    Lab {
        l: 116.0 * fy - 16.0,
        a: 500.0 * (fx - fy),
        b: 200.0 * (fy - fz),
    }
}

/// Squared distance in Lab. Squared because only the ordering is used.
pub fn lab_distance_squared(x: Lab, y: Lab) -> f32 {
    let (dl, da, db) = (x.l - y.l, x.a - y.a, x.b - y.b);
    dl * dl + da * da + db * db
}

/// A palette, held as RGB and as the Lab used to match against it.
#[derive(Debug, Clone)]
pub struct Palette {
    pub colours: Vec<[u8; 3]>,
    lab: Vec<Lab>,
}

impl Palette {
    pub fn new(colours: Vec<[u8; 3]>) -> Result<Self, Error> {
        if colours.is_empty() {
            return Err(Error::Spec("palette has no colours".into()));
        }
        let lab = colours.iter().map(|c| to_lab(c[0], c[1], c[2])).collect();
        Ok(Palette { colours, lab })
    }

    /// Read `["#rrggbb", ...]`, or an object with a `colours` array of the same.
    pub fn from_json(text: &str) -> Result<Self, Error> {
        let value: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| Error::Spec(format!("palette is not valid json: {e}")))?;
        let array = value
            .as_array()
            .or_else(|| value.get("colours").and_then(|v| v.as_array()))
            .or_else(|| value.get("colors").and_then(|v| v.as_array()))
            .ok_or_else(|| Error::Spec("palette needs an array of hex strings".into()))?;
        let mut colours = Vec::with_capacity(array.len());
        for entry in array {
            let hex = entry
                .as_str()
                .ok_or_else(|| Error::Spec(format!("palette entry {entry} is not a string")))?;
            colours.push(parse_hex(hex)?);
        }
        Palette::new(colours)
    }

    /// The nearest palette colour, by Lab distance.
    pub fn nearest(&self, r: u8, g: u8, b: u8) -> [u8; 3] {
        let target = to_lab(r, g, b);
        let mut best = 0;
        let mut best_distance = f32::MAX;
        for (index, candidate) in self.lab.iter().enumerate() {
            let distance = lab_distance_squared(target, *candidate);
            if distance < best_distance {
                best_distance = distance;
                best = index;
            }
        }
        self.colours[best]
    }
}

/// `#rrggbb` or `rrggbb`.
pub fn parse_hex(hex: &str) -> Result<[u8; 3], Error> {
    let trimmed = hex.trim().trim_start_matches('#');
    if trimmed.len() != 6 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::Spec(format!("\"{hex}\" is not a #rrggbb colour")));
    }
    let byte = |at: usize| u8::from_str_radix(&trimmed[at..at + 2], 16).unwrap_or(0);
    Ok([byte(0), byte(2), byte(4)])
}

/// The bounding box of everything not transparent, as `(x, y, w, h)`.
///
/// Returns `None` for a fully transparent image, which is a real outcome when a
/// generator returns an empty frame and is worth reporting rather than cropping
/// to nothing.
pub fn alpha_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (mut min_x, mut min_y) = (u32::MAX, u32::MAX);
    let (mut max_x, mut max_y) = (0u32, 0u32);
    let mut any = false;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0[3] > TRIM_ALPHA {
            any = true;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x);
            max_y = max_y.max(y);
        }
    }
    if !any {
        return None;
    }
    Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
}

/// How a sprite is reduced.
#[derive(Debug, Clone)]
pub struct Reduction {
    /// Target height in pixels. Width follows the trimmed aspect ratio.
    pub height: u32,
    /// Trim to the alpha bounding box before scaling.
    pub trim: bool,
    /// Palette to quantise to, if any.
    pub palette: Option<Palette>,
    /// Force every pixel fully opaque or fully transparent.
    pub harden_alpha: bool,
}

impl Default for Reduction {
    fn default() -> Self {
        Reduction {
            height: 128,
            trim: true,
            palette: None,
            harden_alpha: true,
        }
    }
}

/// Run the reduction.
pub fn reduce(source: &RgbaImage, settings: &Reduction) -> Result<RgbaImage, Error> {
    if settings.height == 0 {
        return Err(Error::Spec("target height must be above zero".into()));
    }

    let working = if settings.trim {
        match alpha_bounds(source) {
            Some((x, y, w, h)) => source.view(x, y, w, h).to_image(),
            None => return Err(Error::Spec("image is entirely transparent".into())),
        }
    } else {
        source.clone()
    };

    let (sw, sh) = working.dimensions();
    // Keep the aspect ratio of what is actually drawn, and never round a
    // dimension away to nothing.
    let target_h = settings.height;
    let target_w = (((u64::from(sw) * u64::from(target_h)) as f64) / f64::from(sh)).round() as u32;
    let target_w = target_w.max(1);

    // Area averaging. Every source pixel lands in exactly one output pixel,
    // which is the whole reason for generating large.
    let mut out = imageops::resize(&working, target_w, target_h, imageops::FilterType::Triangle);

    if let Some(palette) = &settings.palette {
        for pixel in out.pixels_mut() {
            let [r, g, b, a] = pixel.0;
            // Nothing is gained by matching a colour nobody can see, and a
            // transparent pixel's colour is usually meaningless.
            if a == 0 {
                continue;
            }
            let [nr, ng, nb] = palette.nearest(r, g, b);
            *pixel = Rgba([nr, ng, nb, a]);
        }
    }

    if settings.harden_alpha {
        for pixel in out.pixels_mut() {
            pixel.0[3] = if pixel.0[3] > ALPHA_CUT { 255 } else { 0 };
        }
    }

    Ok(out)
}

/// Load, reduce, save.
pub fn reduce_file(input: &Path, output: &Path, settings: &Reduction) -> Result<(u32, u32), Error> {
    let image = image::open(input)
        .map_err(|e| Error::Io(format!("could not read {}: {e}", input.display())))?
        .to_rgba8();
    let reduced = reduce(&image, settings)?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent).map_err(|e| Error::Io(e.to_string()))?;
    }
    reduced
        .save(output)
        .map_err(|e| Error::Io(format!("could not write {}: {e}", output.display())))?;
    Ok(reduced.dimensions())
}

/// An upscale by an exact integer with no interpolation, for looking at a small
/// sprite without a viewer smoothing it into mush.
pub fn nearest_upscale(source: &RgbaImage, factor: u32) -> RgbaImage {
    let (w, h) = source.dimensions();
    let factor = factor.max(1);
    ImageBuffer::from_fn(w * factor, h * factor, |x, y| {
        *source.get_pixel(x / factor, y / factor)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, colour: [u8; 4]) -> RgbaImage {
        ImageBuffer::from_pixel(w, h, Rgba(colour))
    }

    #[test]
    fn hex_parses_with_and_without_a_hash() {
        assert_eq!(parse_hex("#ff8000").unwrap(), [255, 128, 0]);
        assert_eq!(parse_hex("ff8000").unwrap(), [255, 128, 0]);
        assert!(parse_hex("#ff80").is_err());
        assert!(parse_hex("#gggggg").is_err());
    }

    #[test]
    fn lab_puts_black_at_zero_and_white_at_one_hundred() {
        let black = to_lab(0, 0, 0);
        let white = to_lab(255, 255, 255);
        assert!(black.l.abs() < 0.01, "{black:?}");
        assert!((white.l - 100.0).abs() < 0.01, "{white:?}");
        assert!(white.a.abs() < 0.01 && white.b.abs() < 0.01);
    }

    #[test]
    fn lab_distance_is_zero_for_a_colour_against_itself() {
        let c = to_lab(180, 90, 40);
        assert!(lab_distance_squared(c, c).abs() < 1e-6);
    }

    #[test]
    fn nearest_picks_the_obvious_swatch() {
        let palette = Palette::new(vec![[0, 0, 0], [255, 255, 255], [200, 80, 30]]).unwrap();
        assert_eq!(palette.nearest(250, 250, 250), [255, 255, 255]);
        assert_eq!(palette.nearest(5, 5, 5), [0, 0, 0]);
        assert_eq!(palette.nearest(190, 85, 35), [200, 80, 30]);
    }

    #[test]
    fn nearest_in_lab_beats_nearest_in_rgb_on_a_dark_colour() {
        // Naive RGB distance sends a very dark blue to black; Lab keeps the
        // hue, which is the failure this whole conversion exists to avoid.
        let palette = Palette::new(vec![[0, 0, 0], [40, 40, 120]]).unwrap();
        assert_eq!(palette.nearest(20, 20, 90), [40, 40, 120]);
    }

    #[test]
    fn palette_reads_a_bare_array_or_a_wrapped_one() {
        // Doubled hashes: a `"#` inside a single-hash raw string ends it.
        let bare = Palette::from_json(r##"["#000000","#ffffff"]"##).unwrap();
        assert_eq!(bare.colours.len(), 2);
        let wrapped = Palette::from_json(r##"{"colours":["#112233"]}"##).unwrap();
        assert_eq!(wrapped.colours, vec![[0x11, 0x22, 0x33]]);
        let american = Palette::from_json(r##"{"colors":["#112233"]}"##).unwrap();
        assert_eq!(american.colours.len(), 1);
    }

    #[test]
    fn an_empty_palette_is_refused() {
        assert!(Palette::new(Vec::new()).is_err());
        assert!(Palette::from_json("[]").is_err());
    }

    #[test]
    fn alpha_bounds_finds_the_drawn_region() {
        let mut image = solid(10, 10, [0, 0, 0, 0]);
        image.put_pixel(3, 4, Rgba([255, 0, 0, 255]));
        image.put_pixel(6, 8, Rgba([255, 0, 0, 255]));
        assert_eq!(alpha_bounds(&image), Some((3, 4, 4, 5)));
    }

    #[test]
    fn a_fully_transparent_image_has_no_bounds() {
        assert_eq!(alpha_bounds(&solid(4, 4, [9, 9, 9, 0])), None);
    }

    #[test]
    fn reduce_hits_the_target_height_and_keeps_the_aspect_ratio() {
        let source = solid(400, 200, [10, 20, 30, 255]);
        let out = reduce(
            &source,
            &Reduction {
                height: 64,
                trim: false,
                ..Reduction::default()
            },
        )
        .unwrap();
        assert_eq!(out.dimensions(), (128, 64));
    }

    #[test]
    fn reduce_measures_the_ratio_after_trimming_not_before() {
        // A tall canvas holding a wide sprite. Without trimming first the
        // output would be shaped like the canvas instead of like the sprite.
        let mut source = solid(200, 400, [0, 0, 0, 0]);
        for x in 0..100 {
            for y in 0..50 {
                source.put_pixel(x, y + 100, Rgba([200, 80, 30, 255]));
            }
        }
        let out = reduce(
            &source,
            &Reduction {
                height: 50,
                trim: true,
                palette: None,
                harden_alpha: true,
            },
        )
        .unwrap();
        assert_eq!(out.dimensions(), (100, 50));
    }

    #[test]
    fn reduce_refuses_an_empty_image_rather_than_producing_nothing() {
        let err = reduce(&solid(8, 8, [0, 0, 0, 0]), &Reduction::default()).unwrap_err();
        assert!(format!("{err}").contains("transparent"), "{err}");
    }

    #[test]
    fn reduce_refuses_a_zero_height() {
        let settings = Reduction {
            height: 0,
            ..Reduction::default()
        };
        assert!(reduce(&solid(8, 8, [1, 2, 3, 255]), &settings).is_err());
    }

    #[test]
    fn quantisation_leaves_only_palette_colours_behind() {
        let mut source = solid(64, 64, [0, 0, 0, 255]);
        for (i, pixel) in source.pixels_mut().enumerate() {
            let v = (i % 256) as u8;
            *pixel = Rgba([v, v / 2, 255 - v, 255]);
        }
        let palette = Palette::new(vec![[0, 0, 0], [255, 255, 255], [200, 80, 30]]).unwrap();
        let out = reduce(
            &source,
            &Reduction {
                height: 16,
                trim: false,
                palette: Some(palette.clone()),
                harden_alpha: true,
            },
        )
        .unwrap();
        for pixel in out.pixels() {
            let rgb = [pixel.0[0], pixel.0[1], pixel.0[2]];
            assert!(palette.colours.contains(&rgb), "stray colour {rgb:?}");
        }
    }

    #[test]
    fn hardening_leaves_no_half_transparent_edge() {
        let mut source = solid(32, 32, [200, 80, 30, 255]);
        source.put_pixel(0, 0, Rgba([200, 80, 30, 100]));
        source.put_pixel(1, 1, Rgba([200, 80, 30, 200]));
        let out = reduce(
            &source,
            &Reduction {
                height: 32,
                trim: false,
                palette: None,
                harden_alpha: true,
            },
        )
        .unwrap();
        assert!(out.pixels().all(|p| p.0[3] == 0 || p.0[3] == 255));
    }

    #[test]
    fn hardening_can_be_turned_off() {
        let mut source = solid(8, 8, [200, 80, 30, 255]);
        source.put_pixel(4, 4, Rgba([200, 80, 30, 100]));
        let out = reduce(
            &source,
            &Reduction {
                height: 8,
                trim: false,
                palette: None,
                harden_alpha: false,
            },
        )
        .unwrap();
        assert!(out.pixels().any(|p| p.0[3] != 0 && p.0[3] != 255));
    }

    #[test]
    fn nearest_upscale_repeats_pixels_exactly() {
        let mut source = solid(2, 2, [0, 0, 0, 255]);
        source.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let out = nearest_upscale(&source, 4);
        assert_eq!(out.dimensions(), (8, 8));
        for y in 0..4 {
            for x in 0..4 {
                assert_eq!(out.get_pixel(x, y).0, [255, 0, 0, 255]);
            }
        }
        assert_eq!(out.get_pixel(5, 5).0, [0, 0, 0, 255]);
    }

    #[test]
    fn a_transparent_pixel_keeps_its_alpha_through_quantisation() {
        let mut source = solid(16, 16, [200, 80, 30, 255]);
        for x in 0..16 {
            source.put_pixel(x, 0, Rgba([0, 0, 0, 0]));
        }
        let palette = Palette::new(vec![[255, 255, 255]]).unwrap();
        let out = reduce(
            &source,
            &Reduction {
                height: 16,
                trim: false,
                palette: Some(palette),
                harden_alpha: true,
            },
        )
        .unwrap();
        assert!(out.pixels().any(|p| p.0[3] == 0), "transparency survived");
    }
}
