use crate::common::errors::ProxyError;
use image::{DynamicImage, Rgb, RgbImage};

/// Background used to flatten transparency when a format cannot store it.
pub const DEFAULT_BACKGROUND: [u8; 3] = [255, 255, 255];

/// Returns true when at least one pixel is not fully opaque.
///
/// A color type with an alpha channel is not enough: most RGBA PNGs are fully
/// opaque and can safely go to formats without alpha.
pub fn has_transparency(img: &DynamicImage) -> bool {
  if !img.color().has_alpha() {
    return false;
  }
  img.to_rgba8().pixels().any(|p| p[3] < 255)
}

/// Composites the image over a solid background and drops the alpha channel.
///
/// Encoders for formats without alpha (JPEG) discard the channel, which exposes
/// the color stored under transparent pixels - usually black. Blending first
/// makes the result match what a viewer shows.
pub fn flatten(img: &DynamicImage, bg: [u8; 3]) -> DynamicImage {
  let rgba = img.to_rgba8();
  let (w, h) = rgba.dimensions();
  let mut out = RgbImage::new(w, h);
  for (x, y, p) in rgba.enumerate_pixels() {
    let a = p[3] as u32;
    let blend = |fg: u8, bg: u8| ((fg as u32 * a + bg as u32 * (255 - a) + 127) / 255) as u8;
    out.put_pixel(
      x,
      y,
      Rgb([blend(p[0], bg[0]), blend(p[1], bg[1]), blend(p[2], bg[2])]),
    );
  }
  DynamicImage::ImageRgb8(out)
}

/// Parses a `RRGGBB` hex color, with or without a leading `#`.
pub fn parse_background(hex: &str) -> Result<[u8; 3], ProxyError> {
  let invalid = || ProxyError::InvalidParams(format!("invalid bg: {hex}"));
  let digits = hex.trim_start_matches('#');
  if digits.len() != 6 || !digits.is_ascii() {
    return Err(invalid());
  }
  let channel =
    |range: std::ops::Range<usize>| u8::from_str_radix(&digits[range], 16).map_err(|_| invalid());
  Ok([channel(0..2)?, channel(2..4)?, channel(4..6)?])
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::{ImageBuffer, Rgba};

  fn rgba(pixels: &[[u8; 4]]) -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(pixels.len() as u32, 1, |x, _| {
      Rgba(pixels[x as usize])
    }))
  }

  #[test]
  fn test_opaque_rgba_has_no_transparency() {
    assert!(!has_transparency(&rgba(&[[1, 2, 3, 255], [4, 5, 6, 255]])));
  }

  #[test]
  fn test_rgb_has_no_transparency() {
    assert!(!has_transparency(&DynamicImage::new_rgb8(2, 2)));
  }

  #[test]
  fn test_any_translucent_pixel_counts() {
    assert!(has_transparency(&rgba(&[[1, 2, 3, 255], [0, 0, 0, 0]])));
    assert!(has_transparency(&rgba(&[[1, 2, 3, 254]])));
  }

  #[test]
  fn test_flatten_transparent_pixel_takes_background() {
    let out = flatten(&rgba(&[[0, 0, 0, 0]]), [10, 20, 30]).to_rgb8();
    assert_eq!(out.get_pixel(0, 0).0, [10, 20, 30]);
  }

  #[test]
  fn test_flatten_opaque_pixel_is_unchanged() {
    let out = flatten(&rgba(&[[200, 100, 50, 255]]), [255, 255, 255]).to_rgb8();
    assert_eq!(out.get_pixel(0, 0).0, [200, 100, 50]);
  }

  #[test]
  fn test_flatten_blends_partial_alpha() {
    // Half black over white lands mid-way.
    let out = flatten(&rgba(&[[0, 0, 0, 128]]), [255, 255, 255]).to_rgb8();
    let v = out.get_pixel(0, 0)[0];
    assert!((126..=128).contains(&v), "got {v}");
  }

  #[test]
  fn test_parse_background() {
    assert_eq!(parse_background("ff8000").unwrap(), [255, 128, 0]);
    assert_eq!(parse_background("#FF8000").unwrap(), [255, 128, 0]);
  }

  #[test]
  fn test_parse_background_rejects_bad_input() {
    for bad in ["", "fff", "gggggg", "ff80001", "ééé"] {
      assert!(parse_background(bad).is_err(), "{bad}");
    }
  }
}
