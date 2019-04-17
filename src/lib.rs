//! # PxM
//!
//! `pxm` is a simple loader and saver for PxM (PFM, PBM, etc) formats.
//! Currently only `PFM` format is supported.
mod common;
mod pfm;

pub use common::Endian;
pub use pfm::PFMBuilder;
pub use pfm::PFM;
use std::path::Path;

/// Enum containing all supported formats.
pub enum PXM {
    PFM(PFM),
}

impl PXM {
    /// Load pxm file from disk.
    pub fn load(path: impl AsRef<Path>) -> Result<PXM, &'static str> {
        let path = path.as_ref();
        let ext = match path.extension() {
            Some(e) => match e.to_str() {
                Some(e) => e.to_lowercase(),
                None => return Err("Invalid file extension"),
            },
            None => return Err("Unable to extract the file extension"),
        };

        match ext.as_ref() {
            "pfm" => match PFM::from_file(path) {
                Ok(pfm) => Ok(PXM::PFM(pfm)),
                Err(e) => Err(e),
            },
            _ => Err("Invalid file extension"),
        }
    }

    /// Save pxm file to disk.
    pub fn save(path: impl AsRef<Path>, pxm: &PXM) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pfm() {}
}
