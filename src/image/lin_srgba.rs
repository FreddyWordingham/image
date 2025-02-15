use ndarray::{s, Array2};
use palette::LinSrgba;
use png::{ColorType, Decoder, Encoder};
use std::{
    fs::{create_dir_all, File},
    io::BufWriter,
    path::Path,
};

use crate::{Image, ImageError};

impl Image<LinSrgba> {
    /// Creates a new ImageG from the provided data.
    pub fn new(data: Array2<LinSrgba>) -> Self {
        debug_assert!(data.ncols() > 0);
        debug_assert!(data.nrows() > 0);
        Self { data }
    }

    /// Creates an empty (all zeros) image with the given dimensions.
    pub fn empty(width: usize, height: usize) -> Self {
        debug_assert!(width > 0);
        debug_assert!(height > 0);
        let data = Array2::from_elem((height, width), LinSrgba::new(0.0, 0.0, 0.0, 0.0));
        Self { data }
    }

    /// Creates an image filled with a constant value.
    pub fn filled(width: usize, height: usize, value: LinSrgba) -> Self {
        debug_assert!(width > 0);
        debug_assert!(height > 0);
        let data = Array2::from_elem((height, width), value);
        Self { data }
    }

    /// Get the value of a component at the specified position.
    pub fn get_component(&self, coords: [usize; 2], component: usize) -> f32 {
        debug_assert!(component < 4);
        let colour = self.data[[coords[1], coords[0]]];
        match component {
            0 => colour.red,
            1 => colour.green,
            2 => colour.blue,
            3 => colour.alpha,
            _ => unreachable!(),
        }
    }

    /// Set the value of a component at the specified position.
    pub fn set_component(&mut self, coords: [usize; 2], component: usize, value: f32) {
        debug_assert!(component < 4);
        let mut colour = self.data[[coords[1], coords[0]]];
        match component {
            0 => colour.red = value,
            1 => colour.green = value,
            2 => colour.blue = value,
            3 => colour.alpha = value,
            _ => unreachable!(),
        }
    }

    /// Get the value of a pixel at the specified position.
    pub fn get_pixel(&self, coords: [usize; 2]) -> LinSrgba {
        self.data[[coords[1], coords[0]]]
    }

    /// Set the value of a pixel at the specified position.
    pub fn set_pixel(&mut self, coords: [usize; 2], pixel: LinSrgba) {
        self.data[[coords[1], coords[0]]] = pixel;
    }

    /// Saves the LinSrgba image to the specified path as a PNG RGBA image.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ImageError> {
        let height = self.data.nrows();
        let width = self.data.ncols();
        debug_assert!(width > 0 && height > 0);

        if let Some(parent) = path.as_ref().parent() {
            create_dir_all(parent).map_err(|err| {
                ImageError::from_message(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    err
                ))
            })?;
        }

        let file = File::create(&path).map_err(|err| {
            ImageError::from_message(format!(
                "Failed to create file {}: {}",
                path.as_ref().display(),
                err
            ))
        })?;
        let writer = BufWriter::new(file);
        let mut encoder = Encoder::new(writer, width as u32, height as u32);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut png_writer = encoder.write_header().map_err(|err| {
            ImageError::from_message(format!("Failed to write PNG header: {}", err))
        })?;

        // Flip vertically and convert each LinSrgba to u8.
        let mut bytes = Vec::with_capacity(width * height * 4);
        // Iterate rows in reverse order.
        for row in self.data.slice(s![..;-1, ..]).outer_iter() {
            for color in row.iter() {
                let r = (color.red.clamp(0.0, 1.0) * 255.0).round() as u8;
                let g = (color.green.clamp(0.0, 1.0) * 255.0).round() as u8;
                let b = (color.blue.clamp(0.0, 1.0) * 255.0).round() as u8;
                let a = (color.alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
                bytes.extend_from_slice(&[r, g, b, a]);
            }
        }

        png_writer.write_image_data(&bytes).map_err(|err| {
            ImageError::from_message(format!("Failed to write PNG data: {}", err))
        })?;
        Ok(())
    }

    /// Loads a PNG RGBA image from the specified path and converts it to an ImageLinSrgba.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ImageError> {
        let file = File::open(&path).map_err(|err| {
            ImageError::from_message(format!(
                "Failed to open file {}: {}",
                path.as_ref().display(),
                err
            ))
        })?;
        let decoder = Decoder::new(file);
        let mut reader = decoder
            .read_info()
            .map_err(|err| ImageError::from_message(format!("Failed to read PNG info: {}", err)))?;
        let mut buffer = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buffer).map_err(|err| {
            ImageError::from_message(format!("Failed to decode PNG frame: {}", err))
        })?;

        if info.color_type != ColorType::Rgba || info.bit_depth != png::BitDepth::Eight {
            return Err(ImageError::UnsupportedColorType);
        }

        let width = info.width as usize;
        let height = info.height as usize;
        let total_bytes = width * height * 4;
        let data_vec = buffer[..total_bytes].to_vec();

        // Iterate in chunks of 4 bytes (RGBA) and convert to LinSrgba.
        let data = Array2::from_shape_fn((height, width), |(y, x)| {
            let i = (y * width + x) * 4;
            let r = data_vec[i] as f32 / 255.0;
            let g = data_vec[i + 1] as f32 / 255.0;
            let b = data_vec[i + 2] as f32 / 255.0;
            let a = data_vec[i + 3] as f32 / 255.0;
            LinSrgba::new(r, g, b, a)
        });

        // Flip vertically.
        let data = data.slice(s![..;-1, ..]).to_owned();

        Ok(Self { data })
    }
}

impl std::fmt::Display for Image<LinSrgba> {
    /// Displays the image in the terminal.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in self.data.outer_iter().rev() {
            for pixel in row.iter() {
                let r = (pixel.red.clamp(0.0, 1.0) * 255.0) as u8;
                let g = (pixel.green.clamp(0.0, 1.0) * 255.0) as u8;
                let b = (pixel.blue.clamp(0.0, 1.0) * 255.0) as u8;
                let a = (pixel.alpha.clamp(0.0, 1.0) * 255.0) as u8;
                write!(f, "\x1b[48;2;{r};{g};{b};{a}m  \x1b[0m")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
