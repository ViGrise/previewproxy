use crate::common::config::BestFormatConfig;
use crate::common::config::DisallowedOutput;
use crate::common::errors::ProxyError;
use crate::modules::transform::ops::encode;
use image::DynamicImage;
use std::collections::HashSet;
use std::time::Instant;

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
      let p =
        |dx: i32, dy: i32| luma.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32)[0] as i32;
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
    "gif" => Some(DisallowedOutput::Gif),
    "bmp" => Some(DisallowedOutput::Bmp),
    "tiff" => Some(DisallowedOutput::Tiff),
    "ico" => Some(DisallowedOutput::Ico),
    _ => None,
  }
}

/// Returns true for lossless formats that should be skipped for complex (high-edge) images.
fn is_lossless(fmt: &str) -> bool {
  matches!(fmt, "png" | "bmp" | "tiff")
}

/// Selects the best output format for `img` by:
/// 1. Measuring edge density to classify complexity.
/// 2. If resolution exceeds `cfg.max_resolution`, picking the first allowed format (fast path).
/// 3. Otherwise encoding all preferred candidate formats and returning the smallest.
pub fn select_best_format(
  img: &DynamicImage,
  quality: u32,
  cfg: &BestFormatConfig,
  output_disallow: &HashSet<DisallowedOutput>,
  src_content_type: &str,
) -> Result<(Vec<u8>, String), ProxyError> {
  let t0 = Instant::now();
  let mpx = img.width() as f64 * img.height() as f64 / 1_000_000.0;
  let src_is_gif = src_content_type == "image/gif";

  let t_density = Instant::now();
  let density = edge_density(img);
  let is_complex = density >= cfg.complexity_threshold;
  tracing::debug!(
    density,
    is_complex,
    mpx,
    elapsed_ms = t_density.elapsed().as_millis(),
    "best_format: edge density"
  );

  let build_candidates = |fmts: &[String]| -> Vec<String> {
    fmts
      .iter()
      .filter(|fmt| !(is_complex && is_lossless(fmt)))
      .filter(|fmt| fmt.as_str() != "gif" || src_is_gif)
      .filter(|fmt| format_to_disallow_token(fmt).is_none_or(|t| !output_disallow.contains(&t)))
      .cloned()
      .collect()
  };

  // Fast path: image too large to trial-encode - pick the first allowed preferred format
  if cfg.max_resolution.is_some_and(|max| mpx > max) {
    let allowed = build_candidates(&cfg.preferred_formats);
    if allowed.is_empty() {
      return Err(ProxyError::TransformDisabled("best".to_string()));
    }
    tracing::debug!(
      fmt = allowed[0].as_str(),
      elapsed_ms = t0.elapsed().as_millis(),
      "best_format: fast path selected"
    );
    return encode::encode(img.clone(), &allowed[0], quality);
  }

  // Full path: encode all preferred candidates, pick smallest
  let candidates = build_candidates(&cfg.preferred_formats);
  if candidates.is_empty() {
    return Err(ProxyError::TransformDisabled("best".to_string()));
  }

  tracing::debug!(candidates = ?candidates, "best_format: trialing formats");

  let mut best: Option<(Vec<u8>, String)> = None;
  for fmt in &candidates {
    let t_enc = Instant::now();
    match encode::encode(img.clone(), fmt, quality) {
      Ok(result) => {
        let is_smaller = best.as_ref().is_none_or(|(b, _)| result.0.len() < b.len());
        tracing::debug!(
          fmt,
          bytes = result.0.len(),
          elapsed_ms = t_enc.elapsed().as_millis(),
          winning = is_smaller,
          "best_format: encoded"
        );
        if is_smaller {
          best = Some(result);
        }
      }
      Err(e) => {
        tracing::debug!(fmt, error = %e, "best_format: encoder failed, skipping");
      }
    }
  }

  let result = best
    .ok_or_else(|| ProxyError::InternalError("best_format: all encoders failed".to_string()))?;
  tracing::debug!(
    winner = result.1.as_str(),
    bytes = result.0.len(),
    total_ms = t0.elapsed().as_millis(),
    "best_format: done"
  );
  Ok(result)
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
    let (bytes, ct) = select_best_format(&img, 85, &cfg, &HashSet::new(), "image/png").unwrap();
    assert!(!bytes.is_empty());
    assert!(ct.starts_with("image/"));
  }

  #[test]
  fn test_select_best_format_low_complexity_may_choose_png() {
    let img = solid_image(50);
    let cfg = BestFormatConfig::default();
    let (_, ct) = select_best_format(&img, 85, &cfg, &HashSet::new(), "image/png").unwrap();
    assert!(ct.starts_with("image/"));
  }

  #[test]
  fn test_all_candidates_disallowed_returns_error() {
    let img = solid_image(10);
    let cfg = BestFormatConfig::default(); // default preferred: jpeg, webp, png
    let mut disallow = HashSet::new();
    disallow.insert(DisallowedOutput::Jpeg);
    disallow.insert(DisallowedOutput::Webp);
    disallow.insert(DisallowedOutput::Png);
    let result = select_best_format(&img, 85, &cfg, &disallow, "image/png");
    assert!(
      matches!(result, Err(ProxyError::TransformDisabled(_))),
      "all candidates disallowed must return TransformDisabled"
    );
  }

  #[test]
  fn test_gif_excluded_for_non_gif_source() {
    let img = solid_image(10);
    let cfg = BestFormatConfig {
      preferred_formats: vec!["jpeg".to_string(), "gif".to_string()],
      ..BestFormatConfig::default()
    };
    // Non-GIF source: gif must be excluded, jpeg wins
    let (_, ct) = select_best_format(&img, 85, &cfg, &HashSet::new(), "image/png").unwrap();
    assert_eq!(ct, "image/jpeg");
  }

  #[test]
  fn test_gif_included_for_gif_source() {
    let img = solid_image(10);
    let cfg = BestFormatConfig {
      preferred_formats: vec!["jpeg".to_string(), "gif".to_string()],
      ..BestFormatConfig::default()
    };
    // GIF source: gif is allowed as a candidate
    let (bytes, ct) = select_best_format(&img, 85, &cfg, &HashSet::new(), "image/gif").unwrap();
    assert!(!bytes.is_empty());
    assert!(ct == "image/jpeg" || ct == "image/gif");
  }

  #[test]
  fn test_preferred_formats_avif_jxl() {
    let img = solid_image(10);
    let cfg = BestFormatConfig {
      preferred_formats: vec!["avif".to_string(), "jxl".to_string()],
      ..BestFormatConfig::default()
    };
    let (bytes, ct) = select_best_format(&img, 85, &cfg, &HashSet::new(), "image/png").unwrap();
    assert!(!bytes.is_empty());
    assert!(ct == "image/avif" || ct == "image/jxl");
  }

  #[test]
  fn test_fast_path_skips_multi_encode() {
    let img = checkerboard_image(10);
    let cfg = BestFormatConfig {
      max_resolution: Some(0.0),
      ..BestFormatConfig::default()
    };
    let (bytes, ct) = select_best_format(&img, 85, &cfg, &HashSet::new(), "image/png").unwrap();
    assert!(!bytes.is_empty());
    assert!(ct.starts_with("image/"));
  }
}
