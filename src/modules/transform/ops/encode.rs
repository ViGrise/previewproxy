use crate::common::errors::ProxyError;
use bytemuck::cast_slice;
use image::{DynamicImage, ImageFormat};
use std::io::Cursor;

#[tracing::instrument(skip(img))]
pub fn encode(
  img: DynamicImage,
  format: &str,
  quality: u32,
) -> Result<(Vec<u8>, String), ProxyError> {
  let (fmt, content_type) = match format {
    "webp" => (ImageFormat::WebP, "image/webp"),
    "png" => (ImageFormat::Png, "image/png"),
    "gif" => (ImageFormat::Gif, "image/gif"),
    "bmp" => (ImageFormat::Bmp, "image/bmp"),
    "tiff" => (ImageFormat::Tiff, "image/tiff"),
    "ico" => (ImageFormat::Ico, "image/x-icon"),
    "avif" => {
      let rgba = img.to_rgba8();
      let (width, height) = rgba.dimensions();
      let px: &[ravif::RGBA8] = cast_slice(rgba.as_raw());
      let buf = ravif::Img::new(px, width as usize, height as usize);
      let encoded = ravif::Encoder::new()
        .with_quality(quality as f32)
        .with_speed(6)
        .encode_rgba(buf)
        .map_err(|e| ProxyError::InternalError(e.to_string()))?;
      let avif_bytes = encoded.avif_file.to_vec();
      tracing::debug!(
        format = "avif",
        output_bytes = avif_bytes.len(),
        "encode: op applied"
      );
      return Ok((avif_bytes, "image/avif".to_string()));
    }
    "jxl" => {
      use jpegxl_rs::{encode::EncoderFrame, encoder_builder};
      let rgba = img.to_rgba8();
      let (width, height) = rgba.dimensions();
      let mut encoder = encoder_builder()
        .has_alpha(true)
        .build()
        .map_err(|e| ProxyError::InternalError(e.to_string()))?;
      let frame = EncoderFrame::new(rgba.as_raw()).num_channels(4);
      let result = encoder
        .encode_frame::<u8, u8>(&frame, width, height)
        .map_err(|e| ProxyError::InternalError(e.to_string()))?;
      let jxl_bytes = result.to_vec();
      tracing::debug!(
        format = "jxl",
        output_bytes = jxl_bytes.len(),
        "encode: op applied"
      );
      return Ok((jxl_bytes, "image/jxl".to_string()));
    }
    _ => (ImageFormat::Jpeg, "image/jpeg"),
  };

  let mut buf = Cursor::new(Vec::new());

  if fmt == ImageFormat::Jpeg {
    let encoder =
      image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality.clamp(1, 100) as u8);
    img
      .write_with_encoder(encoder)
      .map_err(|e| ProxyError::InternalError(e.to_string()))?;
  } else {
    img
      .write_to(&mut buf, fmt)
      .map_err(|e| ProxyError::InternalError(e.to_string()))?;
  }

  let out = buf.into_inner();
  tracing::debug!(format, output_bytes = out.len(), "encode: op applied");
  Ok((out, content_type.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::DynamicImage;

  #[test]
  fn test_encode_png() {
    let img = DynamicImage::new_rgb8(2, 2);
    let (bytes, ct) = encode(img, "png", 85).unwrap();
    assert_eq!(ct, "image/png");
    assert_eq!(&bytes[1..4], b"PNG");
  }

  #[test]
  fn test_encode_jpeg() {
    let img = DynamicImage::new_rgb8(2, 2);
    let (bytes, ct) = encode(img, "jpeg", 85).unwrap();
    assert_eq!(ct, "image/jpeg");
    assert_eq!(&bytes[0..2], &[0xFF, 0xD8]);
  }

  #[test]
  fn test_encode_gif() {
    let img = DynamicImage::new_rgb8(2, 2);
    let (bytes, ct) = encode(img, "gif", 85).unwrap();
    assert_eq!(ct, "image/gif");
    assert_eq!(&bytes[..3], b"GIF");
  }

  #[test]
  fn test_encode_bmp() {
    let img = DynamicImage::new_rgb8(2, 2);
    let (bytes, ct) = encode(img, "bmp", 85).unwrap();
    assert_eq!(ct, "image/bmp");
    assert_eq!(&bytes[..2], b"BM");
  }

  #[test]
  fn test_encode_tiff() {
    let img = DynamicImage::new_rgb8(2, 2);
    let (bytes, ct) = encode(img, "tiff", 85).unwrap();
    assert_eq!(ct, "image/tiff");
    assert!(bytes.starts_with(b"II") || bytes.starts_with(b"MM"));
  }

  #[test]
  fn test_encode_ico() {
    let img = DynamicImage::new_rgb8(16, 16);
    let (bytes, ct) = encode(img, "ico", 85).unwrap();
    assert_eq!(ct, "image/x-icon");
    assert_eq!(&bytes[..4], &[0x00, 0x00, 0x01, 0x00]);
  }

  #[test]
  fn test_encode_avif() {
    let img = DynamicImage::new_rgb8(2, 2);
    let (bytes, ct) = encode(img, "avif", 85).unwrap();
    assert_eq!(ct, "image/avif");
    assert!(!bytes.is_empty());
  }

  #[test]
  fn test_encode_jxl() {
    let img = DynamicImage::new_rgb8(4, 4);
    let (bytes, ct) = encode(img, "jxl", 85).unwrap();
    assert_eq!(ct, "image/jxl");
    assert!(!bytes.is_empty());
  }
}
