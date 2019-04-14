//! # PXM
//!
//! `pxm` is a simple loader and saver for PxM (PFM, PBM, etc) formats.
//! Currently only `PFM` format is supported.
use std::fs;

pub struct PFM {}

pub enum PXM {
    PFM(PFM),
}

pub fn load(filename: &str) -> Result<PXM, &'static str> {
    Ok(PXM::PFM(PFM {}))
}

pub fn save(filename: &str, pxm: &PXM) {}

fn load_pfm(filename: &str) -> Result<PFM, &'static str> {
    Err("NotImplemented")
}

fn save_pfm(filename: &str, pfm: &PFM) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pfm() {}
}
