//! This crate is only for demonstration purposes.
//! You might want to remove this crate in production.
//! Mandelbrot is copied and modified from
//! https://github.com/ProgrammingRust/mandelbrot/blob/task-queue/src/main.rs and
//! https://github.com/Ducolnd/rust-mandelbrot/blob/master/src/main.rs

use anyhow::*;
use image::codecs::png::PngEncoder;
use image::*;
use num::Complex;
use std::sync::Mutex;

pub fn add_seven(before: i32) -> i32 {
    before + 7
}

#[derive(Debug, Clone)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Try to determine if `c` is in the Mandelbrot set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius two centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex
/// plane designating the area our image covers.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
        // Why subtraction here? pixel.1 increases as we go down,
        // but the imaginary component increases as we go up.
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 200),
            (25, 175),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 },
        ),
        Complex {
            re: -0.5,
            im: -0.75,
        }
    );
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. The `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper-
/// left and lower-right corners of the pixel buffer.
fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert_eq!(pixels.len(), bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            };
        }
    }
}

fn colorize(grey_pixels: &[u8]) -> Vec<u8> {
    let mut ans = vec![0u8; grey_pixels.len() * 3];
    for i in 0..grey_pixels.len() {
        let (r, g, b) = colorize_pixel(grey_pixels[i]);
        ans[i * 3] = r;
        ans[i * 3 + 1] = g;
        ans[i * 3 + 2] = b;
    }
    ans
}

const A: f64 = 1.0 * (1.0 / std::f64::consts::LOG2_10);
const B: f64 = (1.0 / (3.0 * std::f64::consts::SQRT_2)) * (1.0 / std::f64::consts::LOG2_10);

pub fn colorize_pixel(it: u8) -> (u8, u8, u8) {
    if it == 0 {
        return (0, 0, 0);
    }
    let it = it as f64;

    let c: f64 = (1.0_f64 / ((7.0 * 3.0_f64).powf(1.0 / 8.0))) * (1.0 / std::f64::consts::LOG2_10);

    let r = 255.0 * ((1.0 - (A * it).cos()) / 2.0);
    let g = 255.0 * ((1.0 - (B * it).cos()) / 2.0);
    let b = 255.0 * ((1.0 - (c * it).cos()) / 2.0);

    // print!(" {:?} ", [r, g, b]);

    (r as u8, b as u8, g as u8)
}

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_image(pixels: &[u8], bounds: (usize, usize)) -> Result<Vec<u8>> {
    let mut buf = Vec::new();

    let encoder = PngEncoder::new(&mut buf);
    #[allow(deprecated)]
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Rgb8)?;

    Ok(buf)
}

pub fn mandelbrot(
    image_size: Size,
    zoom_point: Point,
    scale: f64,
    num_threads: i32,
) -> Result<Vec<u8>> {
    let bounds = (image_size.width as usize, image_size.height as usize);
    let upper_left = Complex::new(zoom_point.x - scale, zoom_point.y - scale);
    let lower_right = Complex::new(zoom_point.x + scale, zoom_point.y + scale);

    let mut pixels = vec![0; bounds.0 * bounds.1];

    let band_rows = bounds.1 / (num_threads as usize) + 1;

    {
        let bands = Mutex::new(pixels.chunks_mut(band_rows * bounds.0).enumerate());
        let worker = || loop {
            match {
                let mut guard = bands.lock().unwrap();
                guard.next()
            } {
                None => {
                    return;
                }
                Some((i, band)) => {
                    let top = band_rows * i;
                    let height = band.len() / bounds.0;
                    let band_bounds = (bounds.0, height);
                    let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
                    let band_lower_right =
                        pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);
                    render(band, band_bounds, band_upper_left, band_lower_right);
                }
            }
        };

        #[cfg(not(target_family = "wasm"))]
        crossbeam::scope(|scope| {
            for _ in 0..num_threads {
                scope.spawn(|_| worker());
            }
        })
        .unwrap();

        #[cfg(target_family = "wasm")]
        worker();
    }

    write_image(&colorize(&pixels), bounds)
}

// Some crates only support desktop platforms.
// That's why we are doing the compilation test
// only on desktop platforms.
#[allow(unused_imports)]
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub mod compilation_test {
    use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
    use slint;
    pub fn perform() {
        let mut builder = IdBuilder::new(Encryption::MD5);
        builder
            .add_component(HWIDComponent::SystemID)
            .add_component(HWIDComponent::CPUCores);
        let hwid = builder.build("mykey").unwrap();
        println!("Machine ID: {}", hwid);
    }
}
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub mod compilation_test {
    pub fn perform() {
        println!("Skipping RIF compilation test...");
    }
}
