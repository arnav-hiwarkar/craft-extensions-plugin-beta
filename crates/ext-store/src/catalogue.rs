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
/// Environment variable naming an extra ed25519 public key (hex) for a local TEST catalogue.
pub const TEST_KEY_ENV: &str = "CRAFT_CATALOGUE_TEST_KEY";

/// Keys accepted for a catalogue at `base`: the compiled [`TRUSTED_KEYS`], plus a local test key
/// only when every development condition holds. The override is compiled out of release builds,
/// and in debug builds it needs `CRAFT_CATALOGUE_TEST_KEY`, `CRAFT_CATALOGUE_ALLOW_FILE=1` and a
/// `file://` catalogue base. It never applies to HTTPS catalogues and never skips any other check.
pub fn trusted_keys_for(base: &str) -> Vec<String> {
    let mut keys: Vec<String> = TRUSTED_KEYS.iter().map(|k| (*k).to_string()).collect();
    if let Some(key) = test_key(base, std::env::var(TEST_KEY_ENV).ok(), std::env::var("CRAFT_CATALOGUE_ALLOW_FILE").ok()) {
        keys.push(key);
    }
    keys
}
fn test_key(base: &str, key: Option<String>, allow_file: Option<String>) -> Option<String> {
    if !cfg!(debug_assertions) || allow_file.as_deref() != Some("1") || !base.starts_with("file://") {
        return None;
    }
    let key = key?.trim().to_ascii_lowercase();
    (key.len() == 64 && hex::decode(&key).is_ok()).then_some(key)
}
#[cfg(feature = "net")]
fn verify_for(bytes: &[u8], sig: &[u8], base: &str) -> Result<bool> {
    let keys = trusted_keys_for(base);
    let keys: Vec<&str> = keys.iter().map(String::as_str).collect();
    Ok(verify_index(bytes, std::str::from_utf8(sig).map_err(error)?, &keys))
}
/// The configured catalogue base, as the manager reads it.
fn configured_base() -> String {
    std::env::var("CRAFT_CATALOGUE_URL").unwrap_or_else(|_| DEFAULT_URL.into())
}
/// Resolve a release asset URL. Relative URLs (plain path segments, no `..`) are resolved against
/// the catalogue base, so a local catalogue directory can be moved or copied as a unit.
pub fn resolve_asset_url(url: &str, base: &str) -> Result<String> {
    if url.contains("://") {
        return Ok(url.to_string());
    }
    let valid = !url.is_empty()
        && !url.contains(['\\', '?', '#', ':'])
        && url.split('/').all(|s| !s.is_empty() && s != "." && s != "..");
    if !valid {
        return Err(error("relative asset URL must be plain path segments"));
    }
    Ok(format!("{}/{}", base.trim_end_matches('/'), url))
}
/// A segment of a release URL: ASCII letters, digits, `.`, `_` and `-`, never `.` or `..`.
fn plain_segment(segment: &str) -> bool {
    !segment.is_empty() && segment != "." && segment != ".." && segment.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}
/// `owner/name` of a package `repo`, `https://github.com/<owner>/<name>` with an optional `/`.
fn github_slug(repo: &str) -> Option<(&str, &str)> {
    let slug = repo.strip_prefix("https://github.com/")?;
    let mut parts = slug.strip_suffix('/').unwrap_or(slug).split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    (parts.next().is_none() && plain_segment(owner) && plain_segment(name)).then_some((owner, name))
}
/// Check a release asset URL exactly as the signed package record names it, before anything is
/// fetched. Admission (`craft-ext-cli`) checks listings too; the client keeps its own check so a
/// mistaken or hand-built catalogue cannot send a download elsewhere. EffectCraft's retired
/// Plug-in Library enforced the same rules, and decision D4 says assets come from the author's
/// GitHub Releases:
/// - a relative URL is plain path segments under the catalogue base ([`resolve_asset_url`]);
/// - a remote URL is `https://github.com/<owner>/<name>/releases/download/<tag>/<file>` of the
///   package's own `repo` (owner and name compared case-insensitively; a tag may hold `/`), with
///   plain segments only, so no query, fragment, backslash, white space, percent escape or dot
///   segment; plain `http://` and every other scheme are refused;
/// - a `file://` URL names an absolute local path, never a network share (`file:////host/…`),
///   and holds no backslash, query, fragment, percent-encoded separator or dot segment. Whether
///   file assets are allowed at all is decided by [`Catalogue::download_release_from`].
pub fn check_asset_url(url: &str, repo: &str) -> Result<()> {
    if let Some(path) = url.strip_prefix("file://") {
        let lower = path.to_ascii_lowercase();
        let local = path.starts_with('/')
            && !path.starts_with("//")
            && !path.contains(['\\', '?', '#'])
            && !["%2f", "%5c", "%2e"].iter().any(|e| lower.contains(e))
            && path.split('/').skip(1).all(|s| !s.is_empty() && s != "." && s != "..");
        return if local { Ok(()) } else { Err(error("a file asset must name an absolute local path")) };
    }
    if !url.contains("://") {
        return resolve_asset_url(url, "file:///").map(|_| ());
    }
    let Some((owner, name)) = github_slug(repo) else {
        return Err(error("the package repository is not a GitHub repository, so its remote assets are refused"));
    };
    let rest = url.strip_prefix("https://github.com/").ok_or_else(|| error("a remote asset must be a GitHub release asset over https"))?;
    let mut parts = rest.split('/');
    let same_repo = parts.next().is_some_and(|o| o.eq_ignore_ascii_case(owner)) && parts.next().is_some_and(|n| n.eq_ignore_ascii_case(name));
    if !same_repo {
        return Err(error("a remote asset must be a release asset of the package's own repository"));
    }
    let tail: Vec<&str> = parts.collect();
    if tail.len() < 4 || tail[0] != "releases" || tail[1] != "download" || !tail[2..].iter().all(|s| plain_segment(s)) {
        return Err(error("a remote asset URL must name one release file: releases/download/<tag>/<file>"));
    }
    Ok(())
}
/// The cached catalogue: the signature line, a newline, then the exact signed index bytes, in one
/// file replaced atomically, so a reader never pairs one index with another's signature.
pub const SNAPSHOT: &str = "index.signed";
/// Write the cache for a verified index. Callers hold the store lock and have checked the serial.
pub fn write_cache(cache: &Path, index: &[u8], sig: &[u8]) -> Result<()> {
    std::fs::create_dir_all(cache).map_err(error)?;
    let mut b = sig.trim_ascii().to_vec();
    b.push(b'\n');
    b.extend_from_slice(index);
    craft_atomic::atomic_write(&cache.join(SNAPSHOT), &b).map_err(error)?;
    // The older two-file layout is no longer read once a snapshot exists.
    let _ = std::fs::remove_file(cache.join("index.json"));
    let _ = std::fs::remove_file(cache.join("index.json.sig"));
    Ok(())
}
impl Catalogue {
    /// The cached index, verified with the keys trusted for the configured catalogue.
    pub fn cached(dir: &Path) -> Result<Index> {
        Self::cached_for(dir, &configured_base())
    }
    /// The cached index, verified with the keys trusted for `base` (the manager's catalogue).
    pub fn cached_for(dir: &Path, base: &str) -> Result<Index> {
        let keys = trusted_keys_for(base);
        Self::cached_with_keys(dir, &keys.iter().map(String::as_str).collect::<Vec<_>>())
    }
    pub fn cached_with_keys(dir: &Path, keys: &[&str]) -> Result<Index> {
        let (b, sig) = match crate::state_read(dir.join(SNAPSHOT), MAX_INDEX_BYTES + 1024) {
            Ok(snapshot) => {
                let split = snapshot.iter().position(|&c| c == b'\n').ok_or_else(|| error("the cached catalogue is damaged"))?;
                let (sig, rest) = snapshot.split_at(split);
                (rest.get(1..).unwrap_or_default().to_vec(), sig.to_vec())
            }
            Err(_) => (crate::state_read(dir.join("index.json"), MAX_INDEX_BYTES)?, crate::state_read(dir.join("index.json.sig"), 1024)?),
        };
        if !verify_index(&b, std::str::from_utf8(&sig).map_err(error)?, keys) {
            return Err(error("the cached catalogue signature is invalid"));
        }
        Index::parse_cached(&b).map_err(error)
    }
    #[cfg(feature = "net")]
    pub fn media(base: &str, repo: &str, image: &craft_ext_core::listing::Image, role: craft_ext_core::listing::ImageRole) -> Result<Vec<u8>> {
        image.check(role, repo).map_err(error)?;
        let url = resolve_asset_url(&image.url, base)?;
        if url.starts_with("file:") && !file_allowed(base) {
            return Err(error("file media require a file catalogue or CRAFT_CATALOGUE_ALLOW_FILE=1"));
        }
        let b = get(&url, image.size)?;
        image.check_bytes(role, &b, &digest(&b)).map_err(error)?;
        Ok(b)
    }
    /// Download and verify the index (signature, schema, expiry, serial, clock). Nothing is written.
    #[cfg(feature = "net")]
    pub fn fetch_verified(base: &str, now: u64, high_water: u64) -> Result<(Index, Vec<u8>, Vec<u8>)> {
        let b = get(&format!("{}/index.json", base.trim_end_matches('/')), MAX_INDEX_BYTES as u64)?;
        let sig = get(&format!("{}/index.json.sig", base.trim_end_matches('/')), 1024)?;
        if !verify_for(&b, &sig, base)? {
            return Err(error("the catalogue signature is invalid"));
        }
        let i = Index::parse(&b, now, high_water).map_err(error)?;
        if now < i.issued {
            return Err(error("the clock is earlier than the catalogue"));
        }
        Ok((i, b, sig))
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
    pub fn download_release(asset: &Asset, repo: &str, max: u64) -> Result<Vec<u8>> {
        Self::download_release_from(asset, repo, max, "https://catalogue.invalid")
    }
    /// Download one asset of a release of the package whose `repo` the signed index names. The
    /// URL is checked with [`check_asset_url`] before anything is fetched.
    #[cfg(feature = "net")]
    pub fn download_release_from(asset: &Asset, repo: &str, max: u64, base: &str) -> Result<Vec<u8>> {
        check_asset_url(&asset.url, repo)?;
        let url = resolve_asset_url(&asset.url, base)?;
        if url.starts_with("file:") && !file_allowed(base) {
            return Err(error("file assets require a file catalogue or CRAFT_CATALOGUE_ALLOW_FILE=1"));
        }
        let b = get(&url, max.min(asset.size))?;
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
        // `file:////host/share/…` would reach a network share on Windows: local paths only.
        if !path.starts_with('/') || path.starts_with("//") || path.contains(['?', '#', '\\']) {
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
    const REPO: &str = "https://github.com/alice/test";
    #[test]
    fn local_asset_policy_size_and_digest() {
        let t = tempfile::tempdir().unwrap();
        let path = t.path().join("archive.craftplugin");
        std::fs::write(&path, b"archive").unwrap();
        let mut asset = Asset { platform: None, url: format!("file://{}", path.display()), sha256: digest(b"archive"), size: 7 };
        assert_eq!(Catalogue::download_release_from(&asset, REPO, 100, "file:///catalogue").unwrap(), b"archive");
        if std::env::var("CRAFT_CATALOGUE_ALLOW_FILE").as_deref() == Ok("1") {
            assert_eq!(Catalogue::download_release_from(&asset, REPO, 100, "https://example.com").unwrap(), b"archive");
        } else {
            assert!(Catalogue::download_release_from(&asset, REPO, 100, "https://example.com").is_err());
        }
        assert!(Catalogue::download_release_from(&asset, REPO, 3, "file:///catalogue").is_err());
        asset.sha256 = "a".repeat(64);
        assert!(Catalogue::download_release_from(&asset, REPO, 100, "file:///catalogue").is_err());
        assert!(!verify_index(b"{}", &"0".repeat(128), TRUSTED_KEYS));
    }
}

#[cfg(test)]
mod trust_tests {
    use super::*;
    #[test]
    fn cache_snapshot_round_trips_and_rejects_a_mismatched_signature() {
        let t = tempfile::tempdir().unwrap();
        write_cache(t.path(), b"{}", b"00").unwrap();
        // A bad signature over the stored bytes is refused, never trusted.
        assert!(Catalogue::cached_for(t.path(), "https://example.com/").is_err());
        let raw = std::fs::read(t.path().join(SNAPSHOT)).unwrap();
        assert_eq!(raw, b"00\n{}");
        assert!(!t.path().join("index.json").exists());
    }
    const KEY: &str = "d49ebfdf63910eafaaba1cf72ee2dd43f1e82b4cf8cea8c5a0ce93021c99323b";
    #[test]
    fn test_key_needs_debug_file_base_and_allow_flag() {
        let one = Some("1".to_string());
        let key = Some(KEY.to_string());
        assert_eq!(test_key("file:///c/", key.clone(), one.clone()).is_some(), cfg!(debug_assertions));
        assert!(test_key("https://example.com/", key.clone(), one.clone()).is_none());
        assert!(test_key("file:///c/", key.clone(), None).is_none());
        assert!(test_key("file:///c/", key.clone(), Some("true".into())).is_none());
        assert!(test_key("file:///c/", None, one.clone()).is_none());
        assert!(test_key("file:///c/", Some("zz".repeat(32)), one.clone()).is_none());
        assert!(test_key("file:///c/", Some("ab".into()), one).is_none());
    }
    #[test]
    fn asset_urls_stay_in_the_package_repository_or_on_this_computer() {
        let repo = "https://github.com/Alice/test";
        for ok in [
            "https://github.com/alice/test/releases/download/v1/a.craftplugin",
            "https://github.com/ALICE/Test/releases/download/v1.2.0/a-b_c.craftplugin",
            "https://github.com/alice/test/releases/download/group/v1/a.craftplugin",
            "assets/a.craftplugin",
            "file:///home/me/my plugins/a.craftplugin",
        ] {
            assert!(check_asset_url(ok, repo).is_ok(), "{ok}");
        }
        assert!(check_asset_url("https://github.com/alice/test/releases/download/v1/a.zip", "https://github.com/alice/test/").is_ok());
        for bad in [
            "http://github.com/alice/test/releases/download/v1/a.zip",
            "https://github.com/bob/test/releases/download/v1/a.zip",
            "https://github.com/alice/test2/releases/download/v1/a.zip",
            "https://github.com/alice/test/archive/v1.zip",
            "https://github.com/alice/test/releases/download/v1",
            "https://github.com/alice/test/releases/download/v1/../a.zip",
            "https://github.com/alice/test/releases/download/v1/%2e%2e/a.zip",
            "https://github.com/alice/test/releases/download/v1/a.zip?x=1",
            "https://github.com/alice/test/releases/download/v1/a.zip#x",
            "https://github.com/alice/test/releases/download/v1/a b.zip",
            "https://github.com/alice/test/releases/download//a.zip",
            "https://github.com.evil.example/alice/test/releases/download/v1/a.zip",
            "https://objects.githubusercontent.com/alice/test/a.zip",
            "ftp://github.com/alice/test/releases/download/v1/a.zip",
            "file:////server/share/a.craftplugin",
            "file://server/share/a.craftplugin",
            "file:///C:\\a.craftplugin",
            "file:///a/../b.craftplugin",
            "file:///a/%2F/b.craftplugin",
            "../a.craftplugin",
        ] {
            assert!(check_asset_url(bad, repo).is_err(), "{bad}");
        }
        for bad_repo in ["https://gitlab.com/alice/test", "https://github.com/alice", "https://github.com/alice/test/tree/main", "http://github.com/alice/test", ""] {
            assert!(check_asset_url("https://github.com/alice/test/releases/download/v1/a.zip", bad_repo).is_err(), "{bad_repo}");
        }
        // Relative and local assets don't depend on the repository.
        assert!(check_asset_url("assets/a.craftplugin", "").is_ok());
    }
    #[test]
    fn relative_assets_resolve_inside_the_catalogue() {
        assert_eq!(resolve_asset_url("assets/a.craftplugin", "file:///c/").unwrap(), "file:///c/assets/a.craftplugin");
        assert_eq!(resolve_asset_url("https://x/a", "file:///c").unwrap(), "https://x/a");
        for bad in ["", "../a", "assets/../a", "/abs", "a//b", "a?b", "a#b", "c:/a", "a\\b", "./a"] {
            assert!(resolve_asset_url(bad, "file:///c/").is_err(), "{bad}");
        }
    }
}
