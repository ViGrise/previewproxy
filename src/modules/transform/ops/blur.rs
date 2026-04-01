use crate::common::errors::ProxyError;
use image::DynamicImage;

#[tracing::instrument(skip(img))]
pub fn gaussian_blur(img: DynamicImage, sigma: f32) -> Result<DynamicImage, ProxyError> {
  if sigma <= 0.0 {
    return Ok(img);
  }
  let result = img.fast_blur(sigma);
  tracing::debug!(sigma, "blur: op applied");
  Ok(result)
}

#[cfg(test)]
mod tests {
  use super::*;
  use image::DynamicImage;

  #[test]
  fn test_blur_sigma_zero_unchanged() {
    let img = DynamicImage::new_rgb8(4, 4);
    let result = gaussian_blur(img, 0.0).unwrap();
    assert_eq!(result.width(), 4);
  }

  #[test]
  fn test_blur_applies() {
    let img = DynamicImage::new_rgb8(4, 4);
    gaussian_blur(img, 2.0).unwrap();
  }
}
