// Texture data structure

use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Texture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub format: TextureFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8,
    Rgb8,
    R8,
}

impl Texture {
    pub fn new(name: String, width: u32, height: u32, data: Vec<u8>, format: TextureFormat) -> Self {
        Self {
            name,
            width,
            height,
            data,
            format,
        }
    }

    /// Load texture from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let img = image::open(path.as_ref())
            .with_context(|| format!("Failed to load texture: {:?}", path.as_ref()))?;

        let name = path
            .as_ref()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unnamed")
            .to_string();

        Self::from_dynamic_image(name, img)
    }

    /// Convert from DynamicImage
    pub fn from_dynamic_image(name: String, img: DynamicImage) -> Result<Self> {
        let (width, height) = img.dimensions();

        let (data, format) = match img {
            DynamicImage::ImageRgba8(img) => (img.into_raw(), TextureFormat::Rgba8),
            DynamicImage::ImageRgb8(img) => (img.into_raw(), TextureFormat::Rgb8),
            DynamicImage::ImageLuma8(img) => (img.into_raw(), TextureFormat::R8),
            _ => {
                // Convert to RGBA8
                let rgba = img.to_rgba8();
                (rgba.into_raw(), TextureFormat::Rgba8)
            }
        };

        Ok(Self::new(name, width, height, data, format))
    }

    /// Create a solid color texture
    pub fn solid_color(name: String, color: [u8; 4]) -> Self {
        Self::new(name, 1, 1, color.to_vec(), TextureFormat::Rgba8)
    }

    /// Get bytes per pixel
    pub fn bytes_per_pixel(&self) -> u32 {
        match self.format {
            TextureFormat::Rgba8 => 4,
            TextureFormat::Rgb8 => 3,
            TextureFormat::R8 => 1,
        }
    }
}
