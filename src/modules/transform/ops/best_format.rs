use crate::common::config::BestFormatConfig;
use crate::common::config::DisallowedOutput;
use crate::common::errors::ProxyError;
use crate::modules::transform::ops::encode;
use image::DynamicImage;
use std::collections::HashSet;

/// Computes the percentage of edge pixels using a Sobel scan on a downsampled luma image.
/// Returns a value in [0, 100].
pub fn edge_density(img: &DynamicImage) -> f64 {
  let thumb = img.thumbnail(200, 200);
  let luma = thumb.to_luma8();
  let (w, h) = luma.dimensions();
  if w < 3 || h < 3 {
    return 0.0;
  }
  let total = (w - 2) * (h - 2);
  let mut edge_count = 0u32;
  for y in 1..(h - 1) {
    for x in 1..(w - 1) {
      let p = |dx: i32, dy: i32| {
        luma.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32)[0] as i32
      };
      let gx = -p(-1, -1) + p(1, -1) - 2 * p(-1, 0) + 2 * p(1, 0) - p(-1, 1) + p(1, 1);
      let gy = -p(-1, -1) - 2 * p(0, -1) - p(1, -1) + p(1, 1) + 2 * p(0, 1) + p(1, 1);
      let mag = ((gx * gx + gy * gy) as f64).sqrt();
      if mag > 32.0 {
        edge_count += 1;
      }
    }
  }
  edge_count as f64 / total as f64 * 100.0
}

fn format_to_disallow_token(fmt: &str) -> Option<DisallowedOutput> {
  match fmt {
    "jpeg" => Some(DisallowedOutput::Jpeg),
    "webp" => Some(DisallowedOutput::Webp),
    "avif" => Some(DisallowedOutput::Avif),
    "jxl" => Some(DisallowedOutput::Jxl),
    "png" => Some(DisallowedOutput::Png),
    _ => None,
  }
}

/// Selects the best output format for `img` by:
/// 1. Measuring edge density to classify complexity.
/// 2. If resolution exceeds `cfg.max_resolution`, picking a single format (fast path).
/// 3. Otherwise encoding all candidate formats and returning the smallest.
pub fn select_best_format(
  img: &DynamicImage,
  quality: u32,
  cfg: &BestFormatConfig,
  output_disallow: &HashSet<DisallowedOutput>,
) -> Result<(Vec<u8>, String), ProxyError> {
  let mpx = img.width() as f64 * img.height() as f64 / 1_000_000.0;
  let density = edge_density(img);
  let is_complex = density >= cfg.complexity_threshold;

  // Fast path: image too large to trial-encode all formats
  if cfg.max_resolution.map_or(false, |max| mpx > max) {
    let fmt = if is_complex { "webp" } else { "png" };
    if format_to_disallow_token(fmt).map_or(false, |t| output_disallow.contains(&t)) {
      let fallback = if is_complex { "png" } else { "webp" };
      if format_to_disallow_token(fallback).map_or(false, |t| output_disallow.contains(&t)) {
        return Err(ProxyError::TransformDisabled("best".to_string()));
      }
      return encode::encode(img.clone(), fallback, quality);
    }
    return encode::encode(img.clone(), fmt, quality);
  }

  // Full path: try all candidates, pick smallest
  let mut candidates = vec!["jpeg", "webp", "avif", "jxl"];
  if !is_complex {
    candidates.push("png");
  }
  let candidates: Vec<&str> = candidates
    .into_iter()
    .filter(|fmt| {
      format_to_disallow_token(fmt).map_or(true, |t| !output_disallow.contains(&t))
    })
    .collect();

  if candidates.is_empty() {
    return Err(ProxyError::TransformDisabled("best".to_string()));
  }

  let mut best: Option<(Vec<u8>, String)> = None;
  for fmt in candidates {
    if let Ok(result) = encode::encode(img.clone(), fmt, quality) {
      let is_smaller = best.as_ref().map_or(true, |(b, _)| result.0.len() < b.len());
      if is_smaller {
        best = Some(result);
      }
    }
  }

  best.ok_or_else(|| ProxyError::InternalError("best_format: all encoders failed".to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::{DynamicImage, ImageBuffer, Rgb};

  fn solid_image(size: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(size, size, |_, _| {
      Rgb([100u8, 150, 200])
    }))
  }

  fn checkerboard_image(size: u32) -> DynamicImage {
    DynamicImage::ImageRgb8(ImageBuffer::from_fn(size, size, |x, y| {
      if (x + y) % 2 == 0 {
        Rgb([255u8, 255, 255])
      } else {
        Rgb([0u8, 0, 0])
      }
    }))
  }

  #[test]
  fn test_solid_image_has_low_edge_density() {
    let img = solid_image(20);
    let density = edge_density(&img);
    assert!(
      density < 5.5,
      "solid image edge density {density} should be below 5.5"
    );
  }

  #[test]
  fn test_checkerboard_has_high_edge_density() {
    let img = checkerboard_image(20);
    let density = edge_density(&img);
    assert!(
      density >= 5.5,
      "checkerboard edge density {density} should be >= 5.5"
    );
  }

  #[test]
  fn test_select_best_format_returns_bytes() {
    let img = solid_image(10);
    let cfg = BestFormatConfig::default();
    let (bytes, ct) = select_best_format(&img, 85, &cfg, &HashSet::new()).unwrap();
    assert!(!bytes.is_empty());
    assert!(ct.starts_with("image/"));
  }

  #[test]
  fn test_select_best_format_low_complexity_may_choose_png() {
    let img = solid_image(50);
    let cfg = BestFormatConfig::default();
    let (_, ct) = select_best_format(&img, 85, &cfg, &HashSet::new()).unwrap();
    assert!(ct.starts_with("image/"));
  }

  #[test]
  fn test_all_candidates_disallowed_returns_error() {
    let img = solid_image(10);
    let cfg = BestFormatConfig::default();
    let mut disallow = HashSet::new();
    disallow.insert(DisallowedOutput::Jpeg);
    disallow.insert(DisallowedOutput::Webp);
    disallow.insert(DisallowedOutput::Avif);
    disallow.insert(DisallowedOutput::Jxl);
    disallow.insert(DisallowedOutput::Png);
    let result = select_best_format(&img, 85, &cfg, &disallow);
    assert!(
      matches!(result, Err(ProxyError::TransformDisabled(_))),
      "all candidates disallowed must return TransformDisabled"
    );
  }

  #[test]
  fn test_fast_path_skips_multi_encode() {
    let img = checkerboard_image(10);
    let cfg = BestFormatConfig {
      max_resolution: Some(0.0),
      ..BestFormatConfig::default()
    };
    let (bytes, ct) = select_best_format(&img, 85, &cfg, &HashSet::new()).unwrap();
    assert!(!bytes.is_empty());
    assert!(ct.starts_with("image/"));
  }
}
