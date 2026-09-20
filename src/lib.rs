//! Handwritten signature image rendering for formal correspondence.
//!
//! Pixels, not signatures: this library emulates the inking process (a
//! picture of handwriting placed on the letter). No cryptography, no
//! identity, no legal ceremony. Supported inputs are PNG and JPEG
//! (dimensions read from headers, never decoded) and SVG (passed through,
//! dimensions unknown). Header recognition does not validate compressed
//! pixels or sanitize SVG; callers must use an appropriate renderer for
//! untrusted images. Oversized rasters are reported, never silently resampled.
//!
//! Limits live as data in `tables/defaults.json`; `tests/vectors/*.json`
//! with fixtures in `tests/fixtures/` is the executable contract every
//! language port runs.
//!
//! Same input always yields the same output: no models, no I/O.

use base64::{Engine as _, engine::general_purpose::STANDARD};
use std::sync::LazyLock;

#[derive(serde::Deserialize)]
struct Defaults {
    max_pixels: u64,
    formats: Vec<String>,
}

static DEFAULTS: LazyLock<Defaults> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../tables/defaults.json"))
        .expect("tables/defaults.json is valid")
});

/// Image formats `normalize` accepts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageMime {
    Png,
    Jpeg,
    Svg,
}

impl ImageMime {
    /// Short format name: `png`, `jpeg`, or `svg`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpeg",
            Self::Svg => "svg",
        }
    }
}

/// Recognized image bytes plus intrinsic dimensions when the format carries
/// them (SVG passes through dimension-less). Pixels are not decoded or validated.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecodedImage {
    pub mime: ImageMime,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bytes: Vec<u8>,
}

/// Default oversize threshold in pixels, from `tables/defaults.json`.
#[must_use]
pub fn default_max_pixels() -> u64 {
    DEFAULTS.max_pixels
}

/// Accepted image formats, from `tables/defaults.json`.
#[must_use]
pub fn supported_formats() -> Vec<&'static str> {
    let mut formats: Vec<&'static str> = DEFAULTS.formats.iter().map(String::as_str).collect();
    formats.sort_unstable();
    formats
}

const PNG_MAGIC: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    // Signature, chunk length/type, all 13 IHDR bytes, and its CRC field.
    // The CRC and subsequent image data remain the renderer's responsibility.
    if bytes.len() < 33 || &bytes[..8] != PNG_MAGIC {
        return None;
    }
    if u32::from_be_bytes(bytes[8..12].try_into().ok()?) != 13 || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    if !(1..=0x7FFF_FFFF).contains(&width) || !(1..=0x7FFF_FFFF).contains(&height) {
        return None;
    }
    let valid_depth = match bytes[25] {
        0 => matches!(bytes[24], 1 | 2 | 4 | 8 | 16),
        2 | 4 | 6 => matches!(bytes[24], 8 | 16),
        3 => matches!(bytes[24], 1 | 2 | 4 | 8),
        _ => false,
    };
    if !valid_depth || bytes[26] != 0 || bytes[27] != 0 || bytes[28] > 1 {
        return None;
    }
    Some((width, height))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    let mut remaining = bytes.strip_prefix(&[0xFF, 0xD8])?;
    while !remaining.is_empty() {
        remaining = remaining.strip_prefix(&[0xFF])?;
        while let Some(rest) = remaining.strip_prefix(&[0xFF]) {
            remaining = rest;
        }
        let (&marker, rest) = remaining.split_first()?;
        remaining = rest;
        // Stuffed bytes, restart markers, repeated SOI and scan/end markers
        // cannot provide a frame header at this point in the stream.
        if matches!(marker, 0x00 | 0xD0..=0xDA) {
            return None;
        }
        if marker == 0x01 {
            continue;
        }
        let length = remaining.get(..2)?;
        let length = usize::from(u16::from_be_bytes([length[0], length[1]]));
        if length < 2 {
            return None;
        }
        let segment = remaining.get(..length)?;
        if matches!(
            marker,
            0xC0 | 0xC1
                | 0xC2
                | 0xC3
                | 0xC5
                | 0xC6
                | 0xC7
                | 0xC9
                | 0xCA
                | 0xCB
                | 0xCD
                | 0xCE
                | 0xCF
        ) {
            let header = segment.get(..8)?;
            let components = usize::from(header[7]);
            if components == 0 || length != 8 + 3 * components {
                return None;
            }
            let height = u32::from(u16::from_be_bytes([header[3], header[4]]));
            let width = u32::from(u16::from_be_bytes([header[5], header[6]]));
            if width == 0 || height == 0 {
                return None;
            }
            return Some((width, height));
        }
        remaining = &remaining[length..];
    }
    None
}

fn sniff(bytes: Vec<u8>) -> Option<DecodedImage> {
    if bytes.len() >= 8 && &bytes[..8] == PNG_MAGIC {
        let (width, height) = png_dimensions(&bytes)?;
        return Some(DecodedImage {
            mime: ImageMime::Png,
            width: Some(width),
            height: Some(height),
            bytes,
        });
    }
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        let (width, height) = jpeg_dimensions(&bytes)?;
        return Some(DecodedImage {
            mime: ImageMime::Jpeg,
            width: Some(width),
            height: Some(height),
            bytes,
        });
    }
    let trimmed = bytes.as_slice();
    let start = trimmed
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(trimmed.len());
    if trimmed[start..].starts_with(b"<svg") {
        return Some(DecodedImage {
            mime: ImageMime::Svg,
            width: None,
            height: None,
            bytes,
        });
    }
    None
}

/// Recognize a signature image given as a `data:` URL or raw base64.
/// Base64 must use the canonical alphabet, padding and trailing bits without
/// whitespace. Data URLs must end their metadata with the `;base64` flag.
/// Returns `None` for empty, undecodable, malformed-header, or unsupported
/// inputs (including WebP and GIF). This reads raster headers only: successful
/// recognition does not validate pixel data or sanitize SVG content.
#[must_use]
pub fn normalize(input: &str) -> Option<DecodedImage> {
    let payload = match input.split_once(',') {
        Some((meta, data)) if meta.starts_with("data:") => {
            if !meta.ends_with(";base64") {
                return None;
            }
            data
        }
        _ => input,
    };
    if payload.is_empty() {
        return None;
    }
    let bytes = STANDARD.decode(payload).ok()?;
    if bytes.is_empty() {
        return None;
    }
    sniff(bytes)
}

/// Whether the raster exceeds a pixel budget. Dimension-less formats (SVG)
/// scale without loss and never exceed.
#[must_use]
pub fn exceeds_limits(image: &DecodedImage, max_pixels: u64) -> bool {
    match (image.width, image.height) {
        (Some(width), Some(height)) => u64::from(width) * u64::from(height) > max_pixels,
        _ => false,
    }
}

/// Linear scale factor that fits the image into a pixel budget, never
/// above 1.0 (never upscales). The caller resamples and re-submits.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn scale_to_fit(image: &DecodedImage, max_pixels: u64) -> f64 {
    match (image.width, image.height) {
        (Some(width), Some(height)) => {
            let pixels = u64::from(width) * u64::from(height);
            if pixels <= max_pixels {
                1.0
            } else {
                (max_pixels as f64 / pixels as f64).sqrt()
            }
        }
        _ => 1.0,
    }
}

/// Point size for a target height, preserving aspect ratio, clamped to an
/// optional maximum width. Returns `None` when the image carries no
/// dimensions (SVG without explicit metrics), when dimensions or requested
/// sizes are nonpositive/nonfinite, or when the result cannot be represented
/// as positive finite point dimensions. The caller must handle those cases.
#[must_use]
pub fn signature_size(
    image: &DecodedImage,
    height_pt: f64,
    max_width_pt: Option<f64>,
) -> Option<(f64, f64)> {
    let (width, height) = match (image.width, image.height) {
        (Some(width), Some(height)) if width > 0 && height > 0 => {
            (f64::from(width), f64::from(height))
        }
        _ => return None,
    };
    if !height_pt.is_finite()
        || height_pt <= 0.0
        || max_width_pt.is_some_and(|width| !width.is_finite() || width <= 0.0)
    {
        return None;
    }
    let aspect_ratio = width / height;
    let natural = height_pt * aspect_ratio;
    if let Some(max_width) = max_width_pt.filter(|max_width| natural > *max_width) {
        // Derive the clamped height directly, even if the natural width
        // overflowed before the finite maximum width was applied.
        let clamped_height = max_width / aspect_ratio;
        return (clamped_height.is_finite() && clamped_height > 0.0)
            .then_some((max_width, clamped_height));
    }
    (natural.is_finite() && natural > 0.0).then_some((natural, height_pt))
}

fn escape_path(path: &str) -> String {
    let mut escaped = String::with_capacity(path.len());
    for character in path.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => escaped.extend(character.escape_unicode()),
            character => escaped.push(character),
        }
    }
    escaped
}

/// Typst `#image` call for a sized signature. Width is emitted only when a
/// max-width clamp applied (see [`signature_size`]); layout and placement
/// stay with the caller, whose line budgets they affect.
#[must_use]
pub fn image_snippet(path: &str, height_pt: f64, width_pt: Option<f64>) -> String {
    let path = escape_path(path);
    width_pt.map_or_else(
        || format!(r#"#image("{path}", height: {height_pt}pt)"#),
        |width| format!(r#"#image("{path}", height: {height_pt}pt, width: {width}pt)"#),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_schema() {
        assert!(DEFAULTS.max_pixels > 0, "pixel budget must be positive");
        for format in &DEFAULTS.formats {
            assert!(
                ["png", "jpeg", "svg"].contains(&format.as_str()),
                "unknown format: {format:?}"
            );
        }
    }

    const JPEG_FRAME: &[u8] = &[
        0xFF, 0xD8, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01, 0x01, 0x01, 0x11, 0x00,
    ];

    #[test]
    fn png_requires_a_complete_valid_ihdr_header() {
        let png = include_bytes!("../tests/fixtures/pixel-1x1.png");
        for end in 0..33 {
            assert_eq!(png_dimensions(&png[..end]), None, "prefix {end}");
        }
        assert_eq!(png_dimensions(&png[..33]), Some((1, 1)));
        for (offset, value) in [
            (11, 12),   // Declared IHDR length.
            (12, b'X'), // First chunk must be IHDR.
            (16, 0x80), // Dimensions must fit PNG's 31-bit range.
            (20, 0x80),
            (24, 3), // Invalid depth for the fixture's truecolor type.
            (25, 1), // Invalid color type.
            (26, 1), // Unknown compression method.
            (27, 1), // Unknown filter method.
            (28, 2), // Unknown interlace method.
        ] {
            let mut malformed = png.to_vec();
            malformed[offset] = value;
            assert_eq!(png_dimensions(&malformed), None, "offset {offset}");
        }
    }

    #[test]
    fn typst_paths_escape_controls_and_source_delimiters() {
        assert_eq!(
            image_snippet("a\n\r\t\0\u{1b}\u{85}\\\"#panic(\"x\")", 1.0, None),
            r##"#image("a\n\r\t\u{0}\u{1b}\u{85}\\\"#panic(\"x\")", height: 1pt)"##
        );
        assert_eq!(escape_path("署名/ä.png"), "署名/ä.png");
    }

    #[test]
    fn jpeg_frame_requires_complete_declared_segment() {
        assert_eq!(jpeg_dimensions(JPEG_FRAME), Some((1, 1)));
        for end in 0..JPEG_FRAME.len() {
            assert_eq!(jpeg_dimensions(&JPEG_FRAME[..end]), None, "prefix {end}");
        }
        for length in [0_u16, 1, 2, 7, 8, 10, 12, u16::MAX] {
            let mut malformed = JPEG_FRAME.to_vec();
            malformed[4..6].copy_from_slice(&length.to_be_bytes());
            assert_eq!(jpeg_dimensions(&malformed), None, "length {length}");
        }
        for components in [0, 2, u8::MAX] {
            let mut malformed = JPEG_FRAME.to_vec();
            malformed[11] = components;
            assert_eq!(jpeg_dimensions(&malformed), None, "components {components}");
        }
    }

    #[test]
    fn jpeg_markers_require_prefixes_and_valid_boundaries() {
        let mut missing_prefix = JPEG_FRAME.to_vec();
        missing_prefix.remove(2);
        assert_eq!(jpeg_dimensions(&missing_prefix), None);

        for marker in [0x00, 0xD0, 0xD7, 0xD8, 0xD9, 0xDA] {
            let mut malformed = vec![0xFF, 0xD8, 0xFF, marker];
            malformed.extend_from_slice(&JPEG_FRAME[2..]);
            assert_eq!(jpeg_dimensions(&malformed), None, "marker {marker}");
        }

        let mut with_fill_and_app = vec![0xFF, 0xD8, 0xFF, 0xFF, 0xE0, 0x00, 0x03, 0x00];
        with_fill_and_app.extend_from_slice(&JPEG_FRAME[2..]);
        assert_eq!(jpeg_dimensions(&with_fill_and_app), Some((1, 1)));
        with_fill_and_app[6] = 0x04; // Declared APP length swallows the next marker prefix.
        assert_eq!(jpeg_dimensions(&with_fill_and_app), None);
    }

    fn raster(width: u32, height: u32) -> DecodedImage {
        DecodedImage {
            mime: ImageMime::Png,
            width: Some(width),
            height: Some(height),
            bytes: Vec::new(),
        }
    }

    #[test]
    fn signature_size_rejects_invalid_dimensions_and_requests() {
        let square = raster(1, 1);
        for invalid in [0.0, -0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(signature_size(&square, invalid, None), None);
            assert_eq!(signature_size(&square, 1.0, Some(invalid)), None);
        }
        assert_eq!(signature_size(&raster(0, 1), 1.0, None), None);
        assert_eq!(signature_size(&raster(1, 0), 1.0, None), None);
    }

    #[test]
    fn signature_size_handles_overflow_and_underflow() {
        let wide = raster(2, 1);
        assert_eq!(signature_size(&wide, f64::MAX, None), None);
        assert_eq!(
            signature_size(&wide, f64::MAX, Some(20.0)),
            Some((20.0, 10.0))
        );
        assert_eq!(
            signature_size(&raster(u32::MAX, u32::MAX), f64::MAX, None),
            Some((f64::MAX, f64::MAX))
        );
        let subnormal = f64::from_bits(1);
        assert_eq!(signature_size(&raster(1, u32::MAX), subnormal, None), None);
        assert_eq!(signature_size(&wide, 1.0, Some(subnormal)), None);
    }

    #[test]
    #[allow(clippy::float_cmp)] // These budget extremes produce exact zero and one.
    fn pixel_budgets_cover_full_u32_dimension_domain() {
        let largest = raster(u32::MAX, u32::MAX);
        let pixels = u64::from(u32::MAX) * u64::from(u32::MAX);
        assert!(!exceeds_limits(&largest, pixels));
        assert!(exceeds_limits(&largest, pixels - 1));
        assert!(!exceeds_limits(&largest, u64::MAX));
        assert!(exceeds_limits(&largest, 0));
        assert_eq!(scale_to_fit(&largest, 0), 0.0);
        assert_eq!(scale_to_fit(&largest, u64::MAX), 1.0);
    }

    fn normalize_fixture(file: &str) -> Option<DecodedImage> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        let bytes = std::fs::read(dir.join(file)).expect("fixture is readable");
        normalize(&STANDARD.encode(bytes))
    }

    fn image_value(image: &DecodedImage) -> serde_json::Value {
        serde_json::json!({
            "mime": image.mime.name(),
            "width": image.width,
            "height": image.height,
        })
    }

    fn run_vector(file: &std::path::Path, vector: &serde_json::Value) {
        let name = vector["name"].as_str().unwrap_or("<unnamed>");
        let context = format!("{} :: {name}", file.display());
        let actual: serde_json::Value = match vector["fn"].as_str().unwrap_or("") {
            "normalize" => {
                let image = vector
                    .get("fixture")
                    .and_then(serde_json::Value::as_str)
                    .map_or_else(
                        || normalize(vector["input"].as_str().unwrap_or("")),
                        normalize_fixture,
                    );
                image.map_or(serde_json::Value::Null, |img| image_value(&img))
            }
            "default_max_pixels" => serde_json::json!(default_max_pixels()),
            "supported_formats" => serde_json::Value::Array(
                supported_formats()
                    .iter()
                    .map(|format| serde_json::Value::String((*format).to_owned()))
                    .collect(),
            ),
            "exceeds_limits" => {
                let fixture = vector["fixture"].as_str().expect("vector needs fixture");
                let max = vector["max_pixels"]
                    .as_u64()
                    .expect("vector needs max_pixels");
                let image = normalize_fixture(fixture).expect("fixture must normalize");
                serde_json::Value::Bool(exceeds_limits(&image, max))
            }
            "scale_to_fit" => {
                let fixture = vector["fixture"].as_str().expect("vector needs fixture");
                let max = vector["max_pixels"]
                    .as_u64()
                    .expect("vector needs max_pixels");
                let image = normalize_fixture(fixture).expect("fixture must normalize");
                serde_json::json!(scale_to_fit(&image, max))
            }
            "signature_size" => {
                let fixture = vector["fixture"].as_str().expect("vector needs fixture");
                let height = vector["height_pt"]
                    .as_f64()
                    .expect("vector needs height_pt");
                let max_width = vector
                    .get("max_width_pt")
                    .and_then(serde_json::Value::as_f64);
                let image = normalize_fixture(fixture);
                image.map_or(serde_json::Value::Null, |img| {
                    signature_size(&img, height, max_width)
                        .map_or(serde_json::Value::Null, |(w, h)| serde_json::json!([w, h]))
                })
            }
            "image_snippet" => {
                let path = vector["path"].as_str().expect("vector needs path");
                let height = vector["height_pt"]
                    .as_f64()
                    .expect("vector needs height_pt");
                let width = vector.get("width_pt").and_then(serde_json::Value::as_f64);
                serde_json::Value::String(image_snippet(path, height, width))
            }
            other => panic!("{context}: unknown fn {other:?}"),
        };
        let expected = vector
            .get("expected")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        assert_eq!(actual, expected, "{context}");
    }

    #[test]
    fn default_budget_and_formats_match_vectors() {
        assert_eq!(default_max_pixels(), 1600);
        assert_eq!(supported_formats(), ["jpeg", "png", "svg"]);
    }

    #[test]
    fn conformance_vectors() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/vectors");
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .expect("tests/vectors exists")
            .map(|entry| entry.expect("readable entry").path())
            .collect();
        files.sort();
        assert!(!files.is_empty(), "no vector files in tests/vectors");
        let mut count = 0;
        for file in &files {
            let raw = std::fs::read_to_string(file).expect("vector file is readable");
            let vectors: Vec<serde_json::Value> =
                serde_json::from_str(&raw).expect("vector file is valid JSON");
            for vector in &vectors {
                run_vector(file, vector);
                count += 1;
            }
        }
        assert!(count > 0, "no vectors ran");
    }
}
