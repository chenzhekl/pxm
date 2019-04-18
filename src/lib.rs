//! # PxM
//!
//! `pxm` is a simple loader and saver for PxM (PFM, PBM, etc) formats.
//! Currently only `PFM` format is supported.
mod common;
mod pfm;

pub use common::Endian;
pub use pfm::PFMBuilder;
pub use pfm::PFM;
use std::fs::File;
use std::path::Path;

/// Enum containing all supported formats.
#[derive(Debug, PartialEq)]
pub enum PXM {
    PFM(PFM),
}

impl PXM {
    /// Load pxm file from disk file.
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
            "pfm" => {
                let mut file = match File::open(path) {
                    Ok(file) => file,
                    Err(_) => return Err("Unable to open pfm file"),
                };
                match PFM::read_from(&mut file) {
                    Ok(pfm) => Ok(PXM::PFM(pfm)),
                    Err(e) => Err(e),
                }
            }
            _ => Err("Unsupported file extension"),
        }
    }

    /// Save pxm file to disk file.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), &'static str> {
        let path = path.as_ref();
        let ext = match path.extension() {
            Some(e) => match e.to_str() {
                Some(e) => e.to_lowercase(),
                None => return Err("Invalid file extension"),
            },
            None => return Err("Unable to extract the file extension"),
        };

        match ext.as_ref() {
            "pfm" => {
                let mut file = match File::create(path) {
                    Ok(file) => file,
                    Err(_) => return Err("Unable to create pfm file"),
                };
                match self {
                    PXM::PFM(pfm) => match pfm.write_into(&mut file) {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e),
                    },
                }
            }
            _ => Err("Unsupported file extension"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_pfm_save_load() {
        let mut dir = env::temp_dir();
        dir.push("pfm_test.pfm");

        let pfm_gt = PFMBuilder::new()
            .color(true)
            .scale(1.0)
            .size(1, 3)
            .data(vec![0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 1.0, 1.0, 1.0])
            .build()
            .unwrap();
        let pxm_gt = PXM::PFM(pfm_gt);
        pxm_gt.save(&dir).unwrap();
        let pxm = PXM::load(&dir).unwrap();

        assert_eq!(pxm, pxm_gt);
    }
}
