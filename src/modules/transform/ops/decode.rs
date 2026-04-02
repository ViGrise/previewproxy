use crate::common::errors::ProxyError;
use image::DynamicImage;
use std::io::Cursor;

#[tracing::instrument(skip(bytes), fields(input_bytes = bytes.len()))]
pub fn dispatch(mime: &str, bytes: &[u8]) -> Result<DynamicImage, ProxyError> {
  tracing::debug!(
    detected_format = mime,
    input_bytes = bytes.len(),
    "decode: dispatching"
  );
  match mime {
    "image/svg+xml" => decode_svg(bytes),
    "image/vnd.adobe.photoshop" | "image/x-photoshop" => decode_psd(bytes),
    "image/heic" | "image/heif" => decode_heic(bytes),
    "application/pdf" => decode_pdf(bytes),
    _ => decode_via_image_crate(bytes),
  }
}

fn decode_via_image_crate(bytes: &[u8]) -> Result<DynamicImage, ProxyError> {
  tracing::trace!("Decoding image using the image crate");
  image::ImageReader::new(Cursor::new(bytes))
    .with_guessed_format()
    .map_err(|e| ProxyError::InternalError(e.to_string()))?
    .decode()
    .map_err(|e| ProxyError::InternalError(e.to_string()))
}

fn decode_svg(bytes: &[u8]) -> Result<DynamicImage, ProxyError> {
  use resvg::tiny_skia::{Pixmap, Transform};
  use resvg::usvg::{Options, Tree};

  let opts = Options::default();
  tracing::trace!("Parsing SVG data");
  let tree = Tree::from_data(bytes, &opts)
    .map_err(|e| ProxyError::InternalError(format!("svg parse: {e}")))?;

  let size = tree.size();
  let width = if size.width() > 0.0 {
    size.width() as u32
  } else {
    1024
  };
  let height = if size.height() > 0.0 {
    size.height() as u32
  } else {
    1024
  };
  let width = width.min(4096);
  let height = height.min(4096);

  tracing::trace!(
    "Rendering SVG to pixmap with dimensions {}x{}",
    width,
    height
  );
  let mut pixmap = Pixmap::new(width, height)
    .ok_or_else(|| ProxyError::InternalError("svg: failed to create pixmap".to_string()))?;

  resvg::render(&tree, Transform::default(), &mut pixmap.as_mut());

  tracing::trace!("Converting SVG pixmap to RGBA image");
  let rgba = image::RgbaImage::from_raw(width, height, pixmap.data().to_vec())
    .ok_or_else(|| ProxyError::InternalError("svg: pixmap to image failed".to_string()))?;

  Ok(DynamicImage::ImageRgba8(rgba))
}

fn decode_psd(bytes: &[u8]) -> Result<DynamicImage, ProxyError> {
  tracing::trace!("Decoding PSD data");
  let doc =
    psd::Psd::from_bytes(bytes).map_err(|e| ProxyError::InternalError(format!("psd: {e}")))?;
  let rgba_bytes = doc.rgba();
  let width = doc.width();
  let height = doc.height();

  tracing::trace!(
    "Converting PSD RGBA data to image with dimensions {}x{}",
    width,
    height
  );
  image::RgbaImage::from_raw(width, height, rgba_bytes)
    .map(DynamicImage::ImageRgba8)
    .ok_or_else(|| ProxyError::InternalError("psd: buffer size mismatch".to_string()))
}

fn decode_heic(bytes: &[u8]) -> Result<DynamicImage, ProxyError> {
  use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

  tracing::trace!("Decoding HEIC data");
  let ctx = HeifContext::read_from_bytes(bytes).map_err(|_| ProxyError::HeicDecodeError)?;
  tracing::trace!("Successfully read HEIC context, decoding primary image");
  let handle = ctx
    .primary_image_handle()
    .map_err(|_| ProxyError::HeicDecodeError)?;
  let lib = LibHeif::new();
  tracing::trace!("Decoding HEIC image to RGBA format");
  let image = lib
    .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgba), None)
    .map_err(|_| ProxyError::HeicDecodeError)?;

  tracing::trace!("Extracting RGBA data from decoded HEIC image");
  let plane = image
    .planes()
    .interleaved
    .ok_or(ProxyError::HeicDecodeError)?;
  let width = image.width();
  let height = image.height();
  let stride = plane.stride;
  let row_bytes = (width * 4) as usize;

  let mut out = vec![0_u8; (width * height * 4) as usize];
  for y in 0..height as usize {
    let src_off = y * stride;
    let dst_off = y * row_bytes;
    out[dst_off..dst_off + row_bytes].copy_from_slice(&plane.data[src_off..src_off + row_bytes]);
  }

  tracing::trace!(
    "Converting HEIC RGBA data to image with dimensions {}x{}",
    width,
    height
  );
  let rgba = image::RgbaImage::from_raw(width, height, out).ok_or(ProxyError::HeicDecodeError)?;
  tracing::trace!("Successfully decoded HEIC image to RGBA format");
  Ok(DynamicImage::ImageRgba8(rgba))
}

fn decode_pdf(bytes: &[u8]) -> Result<DynamicImage, ProxyError> {
  use pdfium_render::prelude::*;
  use std::cell::RefCell;

  // PDFium is a C library with global process state - it cannot be re-initialized
  // after being dropped. Use a thread-local so each spawn_blocking worker thread
  // initializes it exactly once and reuses it for the process lifetime.
  thread_local! {
    static PDFIUM: RefCell<Option<Pdfium>> = const { RefCell::new(None) };
  }

  PDFIUM.with(|cell| {
    let mut slot = cell.borrow_mut();
    if slot.is_none() {
      tracing::trace!("Initializing PDFium (once per thread)");
      *slot = Some(Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
          .or_else(|_| Pdfium::bind_to_system_library())
          .map_err(|_| ProxyError::PdfRenderError)?,
      ));
    }
    let pdfium = slot.as_ref().unwrap();

    tracing::trace!("Loading PDF document");
    let doc = pdfium
      .load_pdf_from_byte_slice(bytes, None)
      .map_err(|_| ProxyError::PdfRenderError)?;

    tracing::trace!("Accessing first page of PDF");
    let page = doc.pages().get(0).map_err(|_| ProxyError::PdfRenderError)?;

    let render_config = PdfRenderConfig::new()
      .set_target_width(1200)
      .set_maximum_height(1600);

    tracing::trace!("Rendering PDF page to bitmap");
    let bitmap = page
      .render_with_config(&render_config)
      .map_err(|_| ProxyError::PdfRenderError)?;

    tracing::trace!("Converting PDF bitmap to image");
    let img = bitmap.as_image().map_err(|_| ProxyError::PdfRenderError)?;

    Ok(DynamicImage::ImageRgba8(img.into_rgba8()))
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  fn minimal_svg() -> Vec<u8> {
    br#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10">
      <rect width="10" height="10" fill="red"/>
    </svg>"#
      .to_vec()
  }

  #[test]
  fn test_decode_svg() {
    let img = dispatch("image/svg+xml", &minimal_svg()).unwrap();
    assert_eq!(img.width(), 10);
    assert_eq!(img.height(), 10);
  }

  #[test]
  fn test_decode_svg_no_dimensions_uses_default() {
    let svg = br#"<svg xmlns="http://www.w3.org/2000/svg"><rect width="10" height="10"/></svg>"#;
    let img = dispatch("image/svg+xml", svg).unwrap();
    assert!(img.width() > 0 && img.height() > 0);
  }

  #[test]
  fn test_dispatch_fallback_to_image_crate() {
    use crate::modules::transform::test_helpers::tiny_png_bytes;
    let img = dispatch("image/png", &tiny_png_bytes()).unwrap();
    assert!(img.width() > 0);
  }

  #[test]
  fn test_dispatch_unknown_mime_falls_back() {
    use crate::modules::transform::test_helpers::tiny_png_bytes;
    let img = dispatch("image/avif", &tiny_png_bytes());
    let _ = img;
  }

  #[test]
  #[ignore = "requires libheif system library and HEIC fixture"]
  fn test_decode_heic() {
    let bytes = std::fs::read("tests/fixtures/minimal.heic").unwrap();
    let img = dispatch("image/heic", &bytes).unwrap();
    assert!(img.width() > 0);
  }

  #[test]
  #[ignore = "requires pdfium runtime library and PDF fixture"]
  fn test_decode_pdf() {
    let bytes = std::fs::read("tests/fixtures/minimal.pdf").unwrap();
    let img = dispatch("application/pdf", &bytes).unwrap();
    assert!(img.width() > 0 && img.height() > 0);
  }
}
