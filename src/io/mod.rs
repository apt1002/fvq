use image::{DynamicImage, ImageBuffer, Primitive};
use multidimension::{View, Array};

use super::{Grid};

pub mod cli;

mod pixels;
pub use pixels::{PixelArray, Pixels, Channels, L, LA, RGB, RGBA};

// ----------------------------------------------------------------------------

fn to_f32<T: Primitive>(x: T) -> f32 {
    let mut x = x.to_f32().unwrap();
    x /= T::DEFAULT_MAX_VALUE.to_f32().unwrap();
    x.clamp(0.0, 1.0)
}

/// The part of `load_image()` which is generic in the pixel format.
fn to_pixels<
    C: pixels::Channels,
    P: image::Pixel,
>(img: ImageBuffer<P, Vec<P::Subpixel>>) -> PixelArray<C> {
    assert_eq!(C::NUM_CHANNELS, P::CHANNEL_COUNT as usize);
    let size = (img.height() as usize, img.width() as usize);
    let pixels: Array<(Grid, C), P::Subpixel> = Array::new((size, ()), img.into_raw());
    let pixels = pixels.enumerate().map(|((_, c), x)| {
        if c.is_alpha() { to_f32(x) } else { colcon::expand_gamma(to_f32(x)) }
    }).collect();
    PixelArray(pixels)
}

/// Load the specified file into a `Pixels`.
pub fn load_image(name: &str) -> crate::Result<Pixels> {
    let img = image::io::Reader::open(name)?.decode()?;
    Ok(match img {
        DynamicImage::ImageLuma8(img) => Pixels::L(to_pixels(img)),
        DynamicImage::ImageLumaA8(img) => Pixels::LA(to_pixels(img)),
        DynamicImage::ImageRgb8(img) => Pixels::RGB(to_pixels(img)),
        DynamicImage::ImageRgba8(img) => Pixels::RGBA(to_pixels(img)),
        DynamicImage::ImageLuma16(img) => Pixels::L(to_pixels(img)),
        DynamicImage::ImageLumaA16(img) => Pixels::LA(to_pixels(img)),
        DynamicImage::ImageRgb16(img) => Pixels::RGB(to_pixels(img)),
        DynamicImage::ImageRgba16(img) => Pixels::RGBA(to_pixels(img)),
        DynamicImage::ImageRgb32F(img) => Pixels::RGB(to_pixels(img)),
        DynamicImage::ImageRgba32F(img) => Pixels::RGBA(to_pixels(img)),
        _ => Err(super::Error("Unknown image format"))?,
    })
}

// ----------------------------------------------------------------------------

fn from_f32<T: Primitive>(mut x: f32) -> T {
    x = x.clamp(0.0, 1.0);
    x *= T::DEFAULT_MAX_VALUE.to_f32().unwrap();
    T::from(x).unwrap()
}

/// The part of `save_image()` which is generic in the pixel format.
fn from_pixels<
    C: pixels::Channels,
    P: image::Pixel,
>(pixels: &PixelArray<C>) -> ImageBuffer<P, Vec<P::Subpixel>> {
    assert_eq!(C::NUM_CHANNELS, P::CHANNEL_COUNT as usize);
    let ((height, width), ()) = pixels.0.size();
    let pixels: Array<(Grid, C), P::Subpixel> = (&pixels.0).enumerate().map(|((_, c), x)| {
        if c.is_alpha() { from_f32(x) } else { from_f32(colcon::correct_gamma(x)) }
    }).collect();
    ImageBuffer::from_raw(width as u32, height as u32, pixels.to_raw().into()).unwrap()
}

/// Save `pixels` to the specified file.
pub fn save_image(pixels: &Pixels, name: &str) -> crate::Result<()> {
    Ok(match pixels {
        Pixels::L(pixels) => DynamicImage::ImageLuma8(from_pixels(pixels)),
        Pixels::LA(pixels) => DynamicImage::ImageLumaA8(from_pixels(pixels)),
        Pixels::RGB(pixels) => DynamicImage::ImageRgb8(from_pixels(pixels)),
        Pixels::RGBA(pixels) => DynamicImage::ImageRgba8(from_pixels(pixels)),
    }.save(name)?)
}

// ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_save() {
        let pixels = load_image("standard/lenna.png").unwrap();
        save_image(&pixels, "/tmp/lenna.pgm").unwrap();
        let pixels2 = load_image("/tmp/lenna.pgm").unwrap();
        let diff = match (pixels, pixels2) {
            (Pixels::L(pa), Pixels::L(pa2)) => pa2.0 - pa.0,
            _ => panic!("Not a luma image"),
        };
        diff.each(|x| {
            assert!(x.abs() < 0.01);
        });
    }
}
