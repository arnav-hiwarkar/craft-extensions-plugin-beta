//! Signed snapshot verification is available offline; network reads are bounded.
#[cfg(feature = "net")]
use crate::store::digest;
use crate::store::{Result, error};
use craft_ext_core::catalogue::*;
use std::path::Path;
pub struct Catalogue;
pub fn verify_index(bytes: &[u8], sig_hex: &str, keys: &[&str]) -> bool {
    let Ok(sig) = hex::decode(sig_hex.trim()).and_then(|b| <[u8; 64]>::try_from(b).map_err(|_| hex::FromHexError::InvalidStringLength)) else {
        return false;
    };
    keys.iter().any(|key| {
        let Ok(b) = hex::decode(key) else { return false };
        let Ok(b) = <[u8; 32]>::try_from(b) else { return false };
        ed25519_dalek::VerifyingKey::from_bytes(&b).is_ok_and(|k| k.verify_strict(bytes, &ed25519_dalek::Signature::from_bytes(&sig)).is_ok())
    })
}
impl Catalogue {
    pub fn cached(dir: &Path) -> Result<Index> {
        let b = crate::state_read(dir.join("index.json"), MAX_INDEX_BYTES)?;
        let sig = crate::state_read(dir.join("index.json.sig"), 1024)?;
        if !verify_index(&b, std::str::from_utf8(&sig).map_err(error)?, TRUSTED_KEYS) {
            return Err(error("the cached catalogue signature is invalid"));
        }
        Index::parse_cached(&b).map_err(error)
    }
    #[cfg(feature = "net")]
    pub fn fetch(base: &str, cache: &Path, now: u64, high_water: u64) -> Result<Index> {
        let b = get(&format!("{}/index.json", base.trim_end_matches('/')), MAX_INDEX_BYTES as u64)?;
        let sig = get(&format!("{}/index.json.sig", base.trim_end_matches('/')), 1024)?;
        if !verify_index(&b, std::str::from_utf8(&sig).map_err(error)?, TRUSTED_KEYS) {
            return Err(error("the catalogue signature is invalid"));
        }
        let i = Index::parse(&b, now, high_water).map_err(error)?;
        if now < i.issued {
            return Err(error("the clock is earlier than the catalogue"));
        }
        std::fs::create_dir_all(cache).map_err(error)?;
        craft_atomic::atomic_write(&cache.join("index.json.sig"), &sig).map_err(error)?;
        craft_atomic::atomic_write(&cache.join("index.json"), &b).map_err(error)?;
        Ok(i)
    }
    #[cfg(feature = "net")]
    pub fn package_file(base: &str, row: &PackageSummary) -> Result<PackageFile> {
        let b = get(&format!("{}/packages/{}.json", base.trim_end_matches('/'), row.id), row.file_size.min(MAX_PACKAGE_FILE_BYTES as u64))?;
        if b.len() as u64 != row.file_size || digest(&b) != row.file_sha256 {
            return Err(error("the package metadata does not match the signed catalogue"));
        }
        PackageFile::parse(&b, &row.id).map_err(error)
    }
    #[cfg(feature = "net")]
    pub fn download_release(asset: &Asset, max: u64) -> Result<Vec<u8>> {
        let b = get(&asset.url, max.min(asset.size))?;
        if b.len() as u64 != asset.size || digest(&b) != asset.sha256 {
            return Err(error("the downloaded release is damaged or changed"));
        }
        Ok(b)
    }
}
#[cfg(feature = "net")]
fn get(url: &str, max: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut response = ureq::get(url).call().map_err(error)?;
    let mut b = vec![];
    response.body_mut().as_reader().take(max.saturating_add(1)).read_to_end(&mut b).map_err(error)?;
    if b.len() as u64 > max {
        return Err(error("the download is too large"));
    }
    Ok(b)
}
