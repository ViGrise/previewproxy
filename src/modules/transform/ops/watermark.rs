use crate::common::errors::ProxyError;
use ab_glyph::{Font, FontArc, PxScale, point};
use image::{DynamicImage, RgbaImage, imageops};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct WatermarkPlacement {
  pub opacity: f32,
  pub pos: WmPosition,
  pub x: i32,
  pub y: i32,
}

#[derive(Debug, Clone)]
pub enum WmPosition {
  Ce,
  No,
  So,
  Ea,
  We,
  NoEa,
  NoWe,
  SoEa,
  SoWe,
  Re,
}

impl WmPosition {
  pub fn from_str(s: &str) -> WmPosition {
    match s {
      "ce" => WmPosition::Ce,
      "no" => WmPosition::No,
      "so" => WmPosition::So,
      "ea" => WmPosition::Ea,
      "we" => WmPosition::We,
      "noea" => WmPosition::NoEa,
      "nowe" => WmPosition::NoWe,
      "soea" => WmPosition::SoEa,
      "sowe" => WmPosition::SoWe,
      "re" => WmPosition::Re,
      _ => {
        tracing::warn!(pos = s, "unknown wm_pos value, defaulting to noea");
        WmPosition::NoEa
      }
    }
  }
}

#[derive(Debug, Clone)]
pub enum WatermarkSpec {
  Image {
    bytes: Vec<u8>,
    scale: f32,
    placement: WatermarkPlacement,
  },
  Text {
    text: String,
    color: [u8; 4],
    size: f32,
    font: String,
    placement: WatermarkPlacement,
  },
}

// ---------------------------------------------------------------------------
// Positioning
// ---------------------------------------------------------------------------

/// Compute the (x, y) top-left offset for placing a watermark on a base image.
/// `Re` position returns (0, 0) - tiling is handled separately.
pub fn compute_single_position(
  base_w: u32,
  base_h: u32,
  wm_w: u32,
  wm_h: u32,
  pos: &WmPosition,
  x_off: i32,
  y_off: i32,
) -> (i32, i32) {
  let (bw, bh, ww, wh) = (base_w as i32, base_h as i32, wm_w as i32, wm_h as i32);
  let (x, y) = match pos {
    WmPosition::Ce => ((bw - ww) / 2, (bh - wh) / 2),
    WmPosition::No => ((bw - ww) / 2, 0),
    WmPosition::So => ((bw - ww) / 2, bh - wh),
    WmPosition::Ea => (bw - ww, (bh - wh) / 2),
    WmPosition::We => (0, (bh - wh) / 2),
    WmPosition::NoEa => (bw - ww, 0),
    WmPosition::NoWe => (0, 0),
    WmPosition::SoEa => (bw - ww, bh - wh),
    WmPosition::SoWe => (0, bh - wh),
    WmPosition::Re => (0, 0),
  };
  (x + x_off, y + y_off)
}

// ---------------------------------------------------------------------------
// Opacity
// ---------------------------------------------------------------------------

/// Pre-multiply each pixel's alpha by `opacity` (0.0 = fully transparent, 1.0 = unchanged).
pub fn apply_opacity(img: &mut RgbaImage, opacity: f32) {
  if (opacity - 1.0).abs() < f32::EPSILON {
    return;
  }
  let opacity = opacity.clamp(0.0, 1.0);
  for pixel in img.pixels_mut() {
    pixel[3] = (pixel[3] as f32 * opacity).round() as u8;
  }
}

// ---------------------------------------------------------------------------
// Image watermark helpers
// ---------------------------------------------------------------------------

fn apply_image_watermark(
  base: DynamicImage,
  bytes: &[u8],
  scale: f32,
  placement: &WatermarkPlacement,
) -> Result<DynamicImage, ProxyError> {
  let base_w = base.width();
  let base_h = base.height();

  tracing::trace!("Decoding watermark image data");
  let wm_src = image::ImageReader::new(std::io::Cursor::new(bytes))
    .with_guessed_format()
    .map_err(|e| ProxyError::InternalError(e.to_string()))?
    .decode()
    .map_err(|e| ProxyError::InternalError(e.to_string()))?;

  // Scale watermark
  let (wm_w, wm_h) = if scale > 0.0 {
    let target_w = ((base_w as f32) * scale).max(1.0) as u32;
    let ratio = target_w as f32 / wm_src.width() as f32;
    let target_h = ((wm_src.height() as f32) * ratio).max(1.0) as u32;
    (target_w, target_h)
  } else {
    (wm_src.width(), wm_src.height())
  };
  let wm_resized = wm_src.resize(wm_w, wm_h, imageops::FilterType::Lanczos3);
  let mut wm_rgba = wm_resized.to_rgba8();
  apply_opacity(&mut wm_rgba, placement.opacity);

  let mut base_rgba = base.to_rgba8();

  match placement.pos {
    WmPosition::Re => {
      tile_watermark(&mut base_rgba, &wm_rgba, placement.x, placement.y);
    }
    _ => {
      let (x, y) = compute_single_position(
        base_w,
        base_h,
        wm_rgba.width(),
        wm_rgba.height(),
        &placement.pos,
        placement.x,
        placement.y,
      );
      imageops::overlay(&mut base_rgba, &wm_rgba, x as i64, y as i64);
    }
  }

  Ok(DynamicImage::ImageRgba8(base_rgba))
}

fn tile_watermark(base: &mut RgbaImage, wm: &RgbaImage, x_spacing: i32, y_spacing: i32) {
  let tile_w = (wm.width() as i32 + x_spacing.max(0)).max(1);
  let tile_h = (wm.height() as i32 + y_spacing.max(0)).max(1);
  let base_w = base.width() as i32;
  let base_h = base.height() as i32;
  let mut ty = 0i32;
  while ty < base_h {
    let mut tx = 0i32;
    while tx < base_w {
      imageops::overlay(base, wm, tx as i64, ty as i64);
      tx += tile_w;
    }
    ty += tile_h;
  }
}

// ---------------------------------------------------------------------------
// Hex color
// ---------------------------------------------------------------------------

pub fn parse_hex_color(hex: &str) -> Result<[u8; 4], ProxyError> {
  let hex = hex.trim_start_matches('#');
  if hex.len() != 6 {
    return Err(ProxyError::InvalidParams(format!(
      "invalid wmt_color: {hex}"
    )));
  }
  let r = u8::from_str_radix(&hex[0..2], 16)
    .map_err(|_| ProxyError::InvalidParams("invalid wmt_color".to_string()))?;
  let g = u8::from_str_radix(&hex[2..4], 16)
    .map_err(|_| ProxyError::InvalidParams("invalid wmt_color".to_string()))?;
  let b = u8::from_str_radix(&hex[4..6], 16)
    .map_err(|_| ProxyError::InvalidParams("invalid wmt_color".to_string()))?;
  Ok([r, g, b, 255])
}

// ---------------------------------------------------------------------------
// Font loading
// ---------------------------------------------------------------------------

const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../../../../assets/fonts/default.ttf");

fn load_font(font_name: &str) -> Result<FontArc, ProxyError> {
  // Sanitize: reject path traversal attempts
  if font_name.contains('/') || font_name.contains('\\') || font_name.contains("..") {
    tracing::warn!(
      font = font_name,
      "wmt_font contains invalid characters, using default"
    );
    return FontArc::try_from_slice(DEFAULT_FONT_BYTES)
      .map_err(|e| ProxyError::InternalError(format!("default font load failed: {e}")));
  }
  if font_name.is_empty() || font_name.eq_ignore_ascii_case("sans") {
    tracing::trace!("Using default sans font since font name is empty or 'sans'");
    return FontArc::try_from_slice(DEFAULT_FONT_BYTES)
      .map_err(|e| ProxyError::InternalError(format!("default font load failed: {e}")));
  }
  let candidates = [
    format!("/usr/share/fonts/truetype/{font_name}.ttf"),
    format!("/usr/share/fonts/{font_name}.ttf"),
    format!("/usr/local/share/fonts/{font_name}.ttf"),
    format!("/usr/share/fonts/truetype/{font_name}/{font_name}.ttf"),
  ];
  for path in &candidates {
    if let Ok(data) = std::fs::read(path) {
      if let Ok(font) = FontArc::try_from_vec(data) {
        return Ok(font);
      }
    }
  }
  tracing::warn!(
    font = font_name,
    "wmt_font not found, falling back to bundled default"
  );
  FontArc::try_from_slice(DEFAULT_FONT_BYTES)
    .map_err(|e| ProxyError::InternalError(format!("default font load failed: {e}")))
}

// ---------------------------------------------------------------------------
// Text rendering
// ---------------------------------------------------------------------------

fn render_text_image(text: &str, font: &FontArc, scale: PxScale, color: [u8; 4]) -> RgbaImage {
  use ab_glyph::ScaleFont;
  let scaled = font.as_scaled(scale);
  let ascent = scaled.ascent();
  let descent = scaled.descent(); // negative
  let line_h = (ascent - descent).ceil() as u32;

  let mut width = 0.0f32;
  let mut prev_gid = None;
  for c in text.chars() {
    let gid = font.glyph_id(c);
    if let Some(prev) = prev_gid {
      width += scaled.kern(prev, gid);
    }
    width += scaled.h_advance(gid);
    prev_gid = Some(gid);
  }

  let img_w = (width.ceil() as u32 + 2).max(1);
  let img_h = (line_h + 2).max(1);
  let mut img = RgbaImage::new(img_w, img_h);

  let baseline_y = ascent + 1.0;
  let mut caret_x = 1.0f32;
  let mut prev_gid = None;
  for c in text.chars() {
    let gid = font.glyph_id(c);
    if let Some(prev) = prev_gid {
      caret_x += scaled.kern(prev, gid);
    }
    let glyph = gid.with_scale_and_position(scale, point(caret_x, baseline_y));
    caret_x += scaled.h_advance(gid);
    prev_gid = Some(gid);

    if let Some(outlined) = font.outline_glyph(glyph) {
      let bounds = outlined.px_bounds();
      outlined.draw(|px, py, coverage| {
        let ix = bounds.min.x as i32 + px as i32;
        let iy = bounds.min.y as i32 + py as i32;
        if ix >= 0 && iy >= 0 {
          let ix = ix as u32;
          let iy = iy as u32;
          if ix < img.width() && iy < img.height() {
            let a = (coverage * color[3] as f32).round() as u8;
            img.put_pixel(ix, iy, image::Rgba([color[0], color[1], color[2], a]));
          }
        }
      });
    }
  }

  img
}

fn apply_text_watermark(
  base: DynamicImage,
  text: &str,
  color: [u8; 4],
  size: f32,
  font_name: &str,
  placement: &WatermarkPlacement,
) -> Result<DynamicImage, ProxyError> {
  let base_w = base.width();
  let base_h = base.height();

  let font = load_font(font_name)?;
  let scale = PxScale::from(size.max(1.0));
  let mut text_img = render_text_image(text, &font, scale, color);
  apply_opacity(&mut text_img, placement.opacity);

  let mut base_rgba = base.to_rgba8();

  match placement.pos {
    WmPosition::Re => {
      tile_watermark(&mut base_rgba, &text_img, placement.x, placement.y);
    }
    _ => {
      let (x, y) = compute_single_position(
        base_w,
        base_h,
        text_img.width(),
        text_img.height(),
        &placement.pos,
        placement.x,
        placement.y,
      );
      imageops::overlay(&mut base_rgba, &text_img, x as i64, y as i64);
    }
  }

  Ok(DynamicImage::ImageRgba8(base_rgba))
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tracing::instrument(skip(base, spec), fields(base_w = base.width(), base_h = base.height()))]
pub fn apply_watermark_sync(
  base: DynamicImage,
  spec: WatermarkSpec,
) -> Result<DynamicImage, ProxyError> {
  match spec {
    WatermarkSpec::Image {
      bytes,
      scale,
      placement,
    } => apply_image_watermark(base, &bytes, scale, &placement),
    WatermarkSpec::Text {
      text,
      color,
      size,
      font,
      placement,
    } => apply_text_watermark(base, &text, color, size, &font, &placement),
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod type_check {
  use super::*;
  #[test]
  fn watermark_spec_variants_exist() {
    let _p = WatermarkPlacement {
      opacity: 1.0,
      pos: WmPosition::NoEa,
      x: 0,
      y: 0,
    };
    let _spec_img = WatermarkSpec::Image {
      bytes: vec![],
      scale: 0.15,
      placement: WatermarkPlacement {
        opacity: 1.0,
        pos: WmPosition::Ce,
        x: 0,
        y: 0,
      },
    };
    let _spec_txt = WatermarkSpec::Text {
      text: "hello".to_string(),
      color: [0, 0, 0, 255],
      size: 24.0,
      font: "sans".to_string(),
      placement: WatermarkPlacement {
        opacity: 1.0,
        pos: WmPosition::SoWe,
        x: 5,
        y: 5,
      },
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::modules::transform::test_helpers::tiny_png_bytes;

  #[test]
  fn image_watermark_preserves_base_dimensions() {
    let base = DynamicImage::new_rgba8(100, 100);
    let spec = WatermarkSpec::Image {
      bytes: tiny_png_bytes(),
      scale: 0.15,
      placement: WatermarkPlacement {
        opacity: 1.0,
        pos: WmPosition::NoEa,
        x: 0,
        y: 0,
      },
    };
    let result = apply_watermark_sync(base, spec).unwrap();
    assert_eq!(result.width(), 100);
    assert_eq!(result.height(), 100);
  }

  #[test]
  fn image_watermark_ce_position() {
    let base = DynamicImage::new_rgba8(200, 200);
    let spec = WatermarkSpec::Image {
      bytes: tiny_png_bytes(),
      scale: 0.1,
      placement: WatermarkPlacement {
        opacity: 1.0,
        pos: WmPosition::Ce,
        x: 0,
        y: 0,
      },
    };
    let result = apply_watermark_sync(base, spec).unwrap();
    assert_eq!(result.width(), 200);
    assert_eq!(result.height(), 200);
  }

  #[test]
  fn image_watermark_tiling_re_position() {
    let base = DynamicImage::new_rgba8(50, 50);
    let spec = WatermarkSpec::Image {
      bytes: tiny_png_bytes(),
      scale: 0.2,
      placement: WatermarkPlacement {
        opacity: 0.5,
        pos: WmPosition::Re,
        x: 2,
        y: 2,
      },
    };
    let result = apply_watermark_sync(base, spec).unwrap();
    assert_eq!(result.width(), 50);
    assert_eq!(result.height(), 50);
  }

  #[test]
  fn position_ce_centers_watermark() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::Ce, 0, 0);
    assert_eq!(x, 40); // (100-20)/2
    assert_eq!(y, 35); // (80-10)/2
  }

  #[test]
  fn position_noea_top_right() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::NoEa, 0, 0);
    assert_eq!(x, 80); // 100-20
    assert_eq!(y, 0);
  }

  #[test]
  fn position_sowe_bottom_left() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::SoWe, 0, 0);
    assert_eq!(x, 0);
    assert_eq!(y, 70); // 80-10
  }

  #[test]
  fn position_offsets_applied() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::NoEa, -5, 10);
    assert_eq!(x, 75); // 80 + (-5)
    assert_eq!(y, 10);
  }

  #[test]
  fn position_no_top_center() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::No, 0, 0);
    assert_eq!(x, 40);
    assert_eq!(y, 0);
  }

  #[test]
  fn position_so_bottom_center() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::So, 0, 0);
    assert_eq!(x, 40);
    assert_eq!(y, 70);
  }

  #[test]
  fn position_ea_right_center() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::Ea, 0, 0);
    assert_eq!(x, 80);
    assert_eq!(y, 35);
  }

  #[test]
  fn position_we_left_center() {
    let (x, y) = compute_single_position(100, 80, 20, 10, &WmPosition::We, 0, 0);
    assert_eq!(x, 0);
    assert_eq!(y, 35);
  }

  #[test]
  fn opacity_reduces_alpha() {
    let mut img = RgbaImage::new(2, 2);
    for p in img.pixels_mut() {
      *p = image::Rgba([255, 0, 0, 200]);
    }
    apply_opacity(&mut img, 0.5);
    assert_eq!(img.get_pixel(0, 0)[3], 100); // 200 * 0.5 = 100
  }

  #[test]
  fn opacity_full_unchanged() {
    let mut img = RgbaImage::new(1, 1);
    img.put_pixel(0, 0, image::Rgba([255, 255, 255, 128]));
    apply_opacity(&mut img, 1.0);
    assert_eq!(img.get_pixel(0, 0)[3], 128);
  }

  #[test]
  fn opacity_zero_transparent() {
    let mut img = RgbaImage::new(1, 1);
    img.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
    apply_opacity(&mut img, 0.0);
    assert_eq!(img.get_pixel(0, 0)[3], 0);
  }

  #[test]
  fn text_watermark_preserves_base_dimensions() {
    let base = DynamicImage::new_rgba8(100, 100);
    let spec = WatermarkSpec::Text {
      text: "Hello".to_string(),
      color: [255, 255, 255, 255],
      size: 16.0,
      font: "sans".to_string(),
      placement: WatermarkPlacement {
        opacity: 1.0,
        pos: WmPosition::Ce,
        x: 0,
        y: 0,
      },
    };
    let result = apply_watermark_sync(base, spec).unwrap();
    assert_eq!(result.width(), 100);
    assert_eq!(result.height(), 100);
  }

  #[test]
  fn text_watermark_renders_non_transparent_pixels() {
    // Text should leave at least one non-transparent pixel on a transparent base
    let base = DynamicImage::new_rgba8(200, 100);
    let spec = WatermarkSpec::Text {
      text: "X".to_string(),
      color: [255, 0, 0, 255],
      size: 32.0,
      font: "sans".to_string(),
      placement: WatermarkPlacement {
        opacity: 1.0,
        pos: WmPosition::Ce,
        x: 0,
        y: 0,
      },
    };
    let result = apply_watermark_sync(base, spec).unwrap().to_rgba8();
    let has_colored = result.pixels().any(|p| p[0] > 0 && p[3] > 0);
    assert!(has_colored, "expected some red pixels from text rendering");
  }

  #[test]
  fn parse_hex_color_valid() {
    assert_eq!(parse_hex_color("ff0000").unwrap(), [255, 0, 0, 255]);
    assert_eq!(parse_hex_color("000000").unwrap(), [0, 0, 0, 255]);
    assert_eq!(parse_hex_color("FFFFFF").unwrap(), [255, 255, 255, 255]);
  }

  #[test]
  fn parse_hex_color_invalid_returns_err() {
    assert!(parse_hex_color("gg0000").is_err());
    assert!(parse_hex_color("fff").is_err());
  }
}
