#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum DisallowedInput {
  Jpeg,
  Png,
  Gif,
  Webp,
  Avif,
  Jxl,
  Bmp,
  Tiff,
  Pdf,
  Psd,
  Video,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum DisallowedOutput {
  Jpeg,
  Png,
  Gif,
  Webp,
  Avif,
  Jxl,
  Bmp,
  Tiff,
  Ico,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum DisallowedTransform {
  Resize,
  Rotate,
  Flip,
  Grayscale,
  Brightness,
  Contrast,
  Blur,
  Watermark,
  GifAnim,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
  Development,
  Production,
}

impl std::str::FromStr for Environment {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "development" => Ok(Environment::Development),
      "production" => Ok(Environment::Production),
      _ => Err(format!("Invalid environment: {s}")),
    }
  }
}

#[derive(Debug, Clone)]
pub struct BestFormatConfig {
  pub complexity_threshold: f64,
  pub max_resolution: Option<f64>,
  pub by_default: bool,
  pub allow_skips: bool,
}

impl Default for BestFormatConfig {
  fn default() -> Self {
    Self {
      complexity_threshold: 5.5,
      max_resolution: None,
      by_default: false,
      allow_skips: false,
    }
  }
}
