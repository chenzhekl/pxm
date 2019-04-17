use crate::common::Endian;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;
use std::str;

/// PFM struct contains all the information about a PFM file.
/// Note that
#[derive(Debug)]
pub struct PFM {
    /// Width of image.
    pub width: usize,
    /// Hight of image.
    pub height: usize,
    /// True for RGB image, False for monochrome image.
    pub color: bool,
    /// Scaling factor of pixel values.
    pub scale_factor: f32,
    /// Endianness of pixel value in the PFM file.
    pub endian: Endian,
    /// Raw pixel values which are stored in the top to bottom, left
    /// to right order **without** dividing the scale factor.
    pub data: Vec<f32>,
}

impl PFM {
    /// Create PFM struct from disk file.
    pub fn from_file(path: impl AsRef<Path>) -> Result<PFM, &'static str> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(_) => return Err("Unable to open file"),
        };

        let mut buffer = Vec::new();
        match file.read_to_end(&mut buffer) {
            Ok(bytes) => {
                if bytes == 0 {
                    return Err("Empty file");
                }
            }
            Err(_) => return Err("Unable to read from file"),
        };

        parse(&buffer)
    }
}

/// Provides the tool to create PFM struct, and fill in all needed information by hand.
#[derive(Debug)]
pub struct PFMBuilder(PFM);

impl PFMBuilder {
    /// Creates an empty PFM struct.
    pub fn new() -> PFMBuilder {
        let pfm = PFM {
            width: 0,
            height: 0,
            color: true,
            scale_factor: 1.0,
            endian: Endian::Little,
            data: Vec::new(),
        };

        PFMBuilder(pfm)
    }

    /// Set width and height of the PFM file.
    pub fn size(mut self, width: usize, height: usize) -> PFMBuilder {
        assert!(width > 0 && height > 0);

        self.0.width = width;
        self.0.height = height;

        self
    }

    /// Set if it's a RGB or monochrome image.
    pub fn color(mut self, color: bool) -> PFMBuilder {
        self.0.color = color;

        self
    }

    /// Set the scaling factor and endianness.
    pub fn scale(mut self, scale: f32) -> PFMBuilder {
        assert!(scale != 0.0);

        if scale > 0.0 {
            self.0.endian = Endian::Big;
            self.0.scale_factor = scale;
        } else if scale < 0.0 {
            self.0.endian = Endian::Little;
            self.0.scale_factor = -scale;
        }

        self
    }

    /// Set the pixel data.
    pub fn data(mut self, data: Vec<f32>) -> PFMBuilder {
        self.0.data = data;

        self
    }

    /// Build to get the final PFM struct.
    pub fn build(self) -> Result<PFM, &'static str> {
        let num_channels = if self.0.color { 3 } else { 1 };
        let num_pixels = self.0.width * self.0.height;
        if self.0.data.len() != num_channels * num_pixels {
            return Err("The length of data is not equal to width * height * channels");
        }

        Ok(self.0)
    }
}

fn parse(buffer: &[u8]) -> Result<PFM, &'static str> {
    let (mut builder, buffer) = parse_header(buffer)?;

    let endian = builder.0.endian;
    let num_channels = if builder.0.color { 3 } else { 1 };
    let height = builder.0.height;
    let width = builder.0.width;
    let num_pixels = width * height;

    if num_pixels * num_channels != buffer.len() / 4 {
        return Err("Broken file. The length of image data is not equal to width * height * channels specified in the header");
    }

    let mut data = vec![0.0f32; num_pixels * num_channels];
    let mut buffer = Cursor::new(buffer);

    match endian {
        Endian::Little => match buffer.read_f32_into::<LittleEndian>(&mut data) {
            Err(_) => return Err("File data is broken"),
            _ => (),
        },
        Endian::Big => match buffer.read_f32_into::<BigEndian>(&mut data) {
            Err(_) => return Err("File data is broken"),
            _ => (),
        },
    };

    for row in 0..height {
        if row >= height - 1 - row {
            break;
        }
        for col in 0..(width * num_channels) {
            let a = row * width + col;
            let b = (height - 1 - row) * width * num_channels + col;
            data.swap(a, b);
        }
    }

    builder = builder.data(data);

    builder.build()
}

fn parse_header(buffer: &[u8]) -> Result<(PFMBuilder, &[u8]), &'static str> {
    let mut builder = PFMBuilder::new();

    // Parse PF | Pf

    let (header_pf, buffer) = read_until_space(buffer)?;

    if header_pf[0] != ('P' as u8) {
        return Err("Tht first character must be 'P'");
    }

    if header_pf[1] == ('F' as u8) {
        builder = builder.color(true);
    } else if header_pf[1] == ('f' as u8) {
        builder = builder.color(false);
    } else {
        return Err("Tht second character must be 'F' or 'f'");
    }

    // Parse width and height

    let (header_width, buffer) = read_until_space(buffer)?;
    let width: usize = parse_token(header_width, "Invalid width")?;
    if width == 0 {
        return Err("Invalid width");
    }

    let (header_height, buffer) = read_until_space(buffer)?;
    let height: usize = parse_token(header_height, "Invalid height")?;
    if height == 0 {
        return Err("Invalid height");
    }

    builder = builder.size(width, height);

    // Parse scale and endian

    let (header_scale, buffer) = read_until_space(buffer)?;
    let scale: f32 = parse_token(header_scale, "Invalid scale")?;;
    if scale == 0.0 {
        return Err("Invalid scale");
    }

    builder = builder.scale(scale);

    Ok((builder, &buffer[1..]))
}

fn parse_token<T>(buffer: &[u8], err_msg: &'static str) -> Result<T, &'static str>
where
    T: str::FromStr,
{
    match str::from_utf8(buffer) {
        Ok(s) => match s.parse() {
            Ok(w) => Ok(w),
            Err(_) => return Err(err_msg),
        },
        Err(_) => return Err(err_msg),
    }
}

fn read_until_space(buffer: &[u8]) -> Result<(&[u8], &[u8]), &'static str> {
    let mut start = 0;

    while start < buffer.len() && (buffer[start] as char).is_ascii_whitespace() {
        start += 1;
    }

    if start >= buffer.len() {
        return Err("Reached EOF before finishing parsing");
    }

    let mut end = start;

    while end < buffer.len() && !(buffer[end] as char).is_ascii_whitespace() {
        end += 1;
    }

    if end > buffer.len() {
        return Err("Reached EOF before finishing parsing");
    }

    Ok((&buffer[start..end], &buffer[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let buffer = vec![
            0x50, 0x46, 0x0A, // PF
            0x31, 0x20, 0x32, 0x0A, // 1 2
            0x2D, 0x31, 0x2E, 0x30, 0x30, 0x30, 0x30, 0x30, 0x30, 0x0A, // -1.0
            0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x80, 0x3f, 0x00, 0x00, 0x80,
            0x3f, // 1.0 1.0 1.0
            0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x00,
            0x3f, // 0.5 0.5 0.5
        ];

        let pfm = parse(&buffer).unwrap();

        assert_eq!(pfm.color, true);
        assert_eq!(pfm.endian, Endian::Little);
        assert_eq!(pfm.scale_factor, 1.0);
        assert_eq!(pfm.height, 2);
        assert_eq!(pfm.width, 1);
        assert_eq!(pfm.data, vec![0.5, 0.5, 0.5, 1.0, 1.0, 1.0])
    }

    #[test]
    fn test_read_until_space() {
        let buffer = " token1   token2 token3".as_bytes();

        let (s, buffer) = read_until_space(buffer).unwrap();
        assert_eq!(s, "token1".as_bytes());
        assert_eq!(buffer, "   token2 token3".as_bytes());

        let (s, buffer) = read_until_space(buffer).unwrap();
        assert_eq!(s, "token2".as_bytes());
        assert_eq!(buffer, " token3".as_bytes());

        let (s, buffer) = read_until_space(buffer).unwrap();
        assert_eq!(s, "token3".as_bytes());
        assert_eq!(buffer, "".as_bytes());
    }
}
