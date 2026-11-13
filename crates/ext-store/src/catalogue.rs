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
        Self::download_release_from(asset, max, "https://catalogue.invalid")
    }
    #[cfg(feature = "net")]
    pub fn download_release_from(asset: &Asset, max: u64, base: &str) -> Result<Vec<u8>> {
        if asset.url.starts_with("file:") && !file_allowed(base) {
            return Err(error("file assets require a file catalogue or CRAFT_CATALOGUE_ALLOW_FILE=1"));
        }
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
    let mut b = vec![];
    if let Some(path) = url.strip_prefix("file://") {
        if !path.starts_with('/') || path.contains(['?', '#']) {
            return Err(error("file URL must be an absolute local path"));
        }
        std::fs::File::open(path).map_err(error)?.take(max.saturating_add(1)).read_to_end(&mut b).map_err(error)?;
        if b.len() as u64 > max {
            return Err(error("the download is too large"));
        }
        return Ok(b);
    }
    let mut response = ureq::get(url).call().map_err(error)?;
    response.body_mut().as_reader().take(max.saturating_add(1)).read_to_end(&mut b).map_err(error)?;
    if b.len() as u64 > max {
        return Err(error("the download is too large"));
    }
    Ok(b)
}

#[cfg(feature = "net")]
fn file_allowed(base: &str) -> bool {
    base.starts_with("file://") || std::env::var("CRAFT_CATALOGUE_ALLOW_FILE").as_deref() == Ok("1")
}

#[cfg(all(test, feature = "net"))]
mod local_tests {
    use super::*;
    #[test]
    fn local_asset_policy_size_and_digest() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("archive.craftplugin");
        std::fs::write(&path, b"archive").unwrap();
        let mut asset = Asset { platform: None, url: format!("file://{}", path.display()), sha256: digest(b"archive"), size: 7 };
        assert_eq!(Catalogue::download_release_from(&asset, 100, "file:///catalogue").unwrap(), b"archive");
        if std::env::var("CRAFT_CATALOGUE_ALLOW_FILE").as_deref() == Ok("1") {
            assert_eq!(Catalogue::download_release_from(&asset, 100, "https://example.com").unwrap(), b"archive");
        } else {
            assert!(Catalogue::download_release_from(&asset, 100, "https://example.com").is_err());
        }
        assert!(Catalogue::download_release_from(&asset, 3, "file:///catalogue").is_err());
        asset.sha256 = "a".repeat(64);
        assert!(Catalogue::download_release_from(&asset, 100, "file:///catalogue").is_err());
        assert!(!verify_index(b"{}", &"0".repeat(128), TRUSTED_KEYS));
    }
}
