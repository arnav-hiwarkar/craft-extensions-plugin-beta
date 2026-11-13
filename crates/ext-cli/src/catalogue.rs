//! Maintainer repository layout: vendors/<login>.toml (Vendor plus verified_how),
//! plugins/<id>/listing.toml (id, vendor, repo, repo_id, tags), and
//! plugins/<id>/releases/<version>.toml (core Release). revocations.toml contains
//! [[revocation]] core Revocation records. Generated metadata lives in dist/index.json,
//! dist/index.json.sig and dist/packages/<id>.json. Trust tier belongs only to vendors.
use crate::{Args, Result, Usage};
use craft_ext_core::{
    catalogue::*,
    id::ExtensionId,
    tier::{self, VendorTier},
};
use ed25519_dalek::{Signer, SigningKey};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VendorRecord {
    login: String,
    github_id: u64,
    name: String,
    tier: VendorTier,
    url: Option<String>,
    verified_how: Option<String>,
}
impl VendorRecord {
    fn vendor(&self) -> Vendor {
        Vendor { login: self.login.clone(), github_id: self.github_id, name: self.name.clone(), tier: self.tier, url: self.url.clone() }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Listing {
    id: ExtensionId,
    vendor: String,
    repo: String,
    repo_id: u64,
    #[serde(default)]
    tags: Vec<String>,
    licence_reviewed: Option<String>,
    monetisation_reviewed: Option<String>,
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Revocations {
    #[serde(default)]
    revocation: Vec<Revocation>,
}
fn read<T: serde::de::DeserializeOwned>(p: &Path) -> Result<T> {
    Ok(toml::from_str(&fs::read_to_string(p)?)?)
}
fn entries(p: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut v = fs::read_dir(p)?.map(|e| e.map(|e| e.path())).collect::<std::io::Result<Vec<_>>>()?;
    v.sort();
    Ok(v)
}
fn github(repo: &str, suffix: &str) -> Result<Value> {
    let slug = repo.strip_prefix("https://github.com/").ok_or("repository must be a GitHub https URL")?.trim_end_matches('/');
    if slug.split('/').count() != 2 {
        return Err("repository must name an owner and repository".into());
    }
    let mut request = ureq::get(&format!("https://api.github.com/repos/{slug}{suffix}")).header("User-Agent", "craft-ext-cli");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let text = request.call()?.body_mut().read_to_string()?;
    Ok(serde_json::from_str(&text)?)
}
fn validate_asset_url(repo: &str, tag: &str, url: &str) -> Result<()> {
    let slug = repo.strip_prefix("https://github.com/").ok_or("listing must use a GitHub repository")?.trim_end_matches('/');
    if slug.split('/').count() != 2 || slug.split('/').any(|s| s.is_empty() || !s.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))) {
        return Err("invalid listing repository".into());
    }
    if tag.is_empty() || !tag.chars().all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c)) {
        return Err("invalid release tag".into());
    }
    let prefix = format!("https://github.com/{slug}/releases/download/{tag}/");
    let file = url.strip_prefix(&prefix).ok_or("asset repository or tag differs from release listing")?;
    if file.is_empty() || !file.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c)) || file == "." || file == ".." {
        return Err("asset URL must name one release file".into());
    }
    Ok(())
}
fn check_online(l: &Listing, v: &VendorRecord, r: &Release, _warnings: &mut Vec<String>) -> Result<()> {
    let repo = github(&l.repo, "")?;
    if repo["private"].as_bool() != Some(false) {
        return Err("repository must be public".into());
    }
    if repo["id"].as_u64() != Some(l.repo_id) {
        return Err("repository numeric id differs".into());
    }
    let mut request = ureq::get(&format!("https://api.github.com/users/{}", v.login)).header("User-Agent", "craft-ext-cli");
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    let user: Value = serde_json::from_str(&request.call()?.body_mut().read_to_string()?)?;
    if user["id"].as_u64() != Some(v.github_id)
        || repo["owner"]["id"].as_u64() != Some(v.github_id)
        || !repo["owner"]["login"].as_str().is_some_and(|login| login.eq_ignore_ascii_case(&v.login))
    {
        return Err("repository owner or vendor numeric id differs".into());
    }
    let license = repo["license"]["spdx_id"].as_str().unwrap_or("NOASSERTION");
    if license == "NOASSERTION" {
        if !l.licence_reviewed.as_ref().is_some_and(|note| !note.trim().is_empty()) {
            return Err("GitHub did not detect a licence; licence_reviewed requires a maintainer note".into());
        }
    } else if license != r.manifest.license {
        return Err(format!("{}: GitHub licence {license} differs from manifest {}", l.id, r.manifest.license).into());
    }
    if !r.tag.chars().all(|c| c.is_ascii_alphanumeric() || "._-/".contains(c)) {
        return Err("tag contains unsupported URL characters".into());
    }
    let commit = github(&l.repo, &format!("/commits/{}", r.tag))?;
    if commit["sha"].as_str() != Some(&r.commit) {
        return Err("release tag does not resolve to recorded commit".into());
    }
    for asset in &r.assets {
        let bytes = craft_ext_store::catalogue::Catalogue::download_release(asset, 300 * 1024 * 1024)?;
        let dir = temp_dir()?;
        let result = craft_ext_store::unpack::unpack_into(&bytes, &dir, craft_ext_store::unpack::Caps::default());
        let _ = fs::remove_dir_all(&dir);
        let m = result?;
        if m != r.manifest {
            return Err("downloaded manifest differs from release record".into());
        }
    }
    Ok(())
}
fn temp_dir() -> Result<std::path::PathBuf> {
    let mut seed = [0; 16];
    getrandom::fill(&mut seed).map_err(|e| e.to_string())?;
    let p = std::env::temp_dir().join(format!("craft-ext-{}", hex::encode(seed)));
    fs::create_dir(&p)?;
    Ok(p)
}
pub fn assemble(repo: &Path, offline: bool, only: Option<&str>) -> Result<(Index, Vec<PackageFile>, Vec<String>)> {
    let old = match fs::read(repo.join("dist/index.json")) {
        Ok(b) => Some(Index::parse_cached(&b)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };
    let mut vendors = BTreeMap::new();
    for path in entries(&repo.join("vendors"))? {
        if path.extension().is_none_or(|e| e != "toml") {
            continue;
        }
        let v: VendorRecord = read(&path)?;
        if path.file_stem().and_then(|s| s.to_str()) != Some(&v.login) {
            return Err("vendor filename differs from login".into());
        }
        if vendors.insert(v.login.clone(), v).is_some() {
            return Err("duplicate vendor".into());
        }
    }
    let mut packages = vec![];
    let mut summaries = vec![];
    let mut warnings = vec![];
    for dir in entries(&repo.join("plugins"))? {
        if !dir.is_dir() {
            continue;
        }
        let l: Listing = read(&dir.join("listing.toml"))?;
        if only.is_some_and(|id| id != l.id.as_str()) {
            continue;
        }
        if dir.file_name().and_then(|s| s.to_str()) != Some(l.id.as_str()) {
            return Err("plugin directory differs from id".into());
        }
        let v = vendors.get(&l.vendor).ok_or("listing vendor does not exist")?;
        tier::check_id(v.tier.into(), &l.id)?;
        if let Some(rest) = l.id.as_str().strip_prefix("io.github.") {
            if rest.split('.').next() != Some(v.login.to_lowercase().as_str()) {
                return Err("GitHub id namespace differs from vendor login".into());
            }
        } else if !l.id.in_first_party_namespace() {
            let domain = l.id.as_str().split('.').take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join(".");
            if !v.verified_how.as_ref().is_some_and(|s| s.to_lowercase().contains(&domain)) {
                warnings.push(format!("{}: review domain ownership ({domain}) in vendor verified_how", l.id));
            }
        }
        let mut releases = vec![];
        let mut seen = BTreeSet::new();
        for p in entries(&dir.join("releases"))? {
            if p.extension().is_none_or(|e| e != "toml") {
                continue;
            }
            let raw: toml::Table = read(&p)?;
            if raw.keys().any(|k| !["version", "tag", "commit", "published", "assets", "manifest"].contains(&k.as_str())) {
                return Err("unknown field in release record (trust and payment fields are not allowed)".into());
            }
            if let Some(assets) = raw.get("assets").and_then(|v| v.as_array()) {
                for asset in assets {
                    if asset.as_table().is_some_and(|a| a.keys().any(|k| !["platform", "url", "sha256", "size"].contains(&k.as_str()))) {
                        return Err("unknown field in release asset".into());
                    }
                }
            }
            let r: Release = raw.try_into()?;
            r.manifest.validate()?;
            if r.manifest.id != l.id || r.version != r.manifest.version || r.manifest.source != l.repo {
                return Err("release identity, version or source differs from listing".into());
            }
            if p.file_stem().and_then(|s| s.to_str()) != Some(&r.version.to_string()) {
                return Err("release filename differs from version".into());
            }
            if !seen.insert(r.version) {
                return Err("duplicate release version".into());
            }
            craft_ext_core::licence::check_osi(&r.manifest.license).map_err(|t| format!("non-OSI licence: {t}"))?;
            if r.assets.is_empty() {
                return Err("release needs an asset".into());
            }
            for a in &r.assets {
                validate_asset_url(&l.repo, &r.tag, &a.url)?;
                if a.size == 0 || a.sha256.len() != 64 || hex::decode(&a.sha256).is_err() {
                    return Err("invalid release asset".into());
                }
            }
            for word in craft_ext_core::licence::monetisation_flags(&format!("{} {} {}", r.manifest.name, r.manifest.summary, r.manifest.description))
            {
                if !l.monetisation_reviewed.as_ref().is_some_and(|note| !note.trim().is_empty()) {
                    return Err(format!("{}: monetisation wording {word} requires monetisation_reviewed with a maintainer note", l.id).into());
                }
            }
            if !offline {
                check_online(&l, v, &r, &mut warnings)?
            }
            releases.push(r);
        }
        let previous_path = repo.join(format!("dist/packages/{}.json", l.id));
        match fs::read(previous_path) {
            Ok(bytes) => {
                let previous = PackageFile::parse(&bytes, &l.id)?;
                for old in previous.releases {
                    let new = releases.iter().find(|r| r.version == old.version).ok_or("published version was removed")?;
                    let digests = |r: &Release| r.assets.iter().map(|a| (a.platform.clone(), a.sha256.clone())).collect::<BTreeMap<_, _>>();
                    if digests(&old) != digests(new) {
                        return Err("published asset digests changed or were removed".into());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        releases.sort_by_key(|r| r.version);
        let newest = releases.last().ok_or("listing has no releases")?;
        let m = &newest.manifest;
        let previous = old.as_ref().and_then(|i| i.package(&l.id));
        let (stars, stars_date) = if offline {
            previous.map(|r| (r.stars, r.stars_date.clone())).unwrap_or_default()
        } else {
            let api = github(&l.repo, "")?;
            (api["stargazers_count"].as_u64().unwrap_or(0).min(u32::MAX as u64) as u32, crate::author::date(craft_ext_store::store::now()))
        };
        let package = PackageFile { schema: SCHEMA, id: l.id.clone(), vendor: l.vendor.clone(), releases: releases.clone() };
        let bytes = serde_json::to_vec_pretty(&package)?;
        PackageFile::parse(&bytes, &l.id)?;
        summaries.push(PackageSummary {
            id: l.id,
            name: m.name.clone(),
            summary: m.summary.clone(),
            vendor: l.vendor,
            repo: l.repo,
            stars,
            tags: l.tags,
            kinds: m.contributions.iter().map(|c| c.kind).collect::<BTreeSet<_>>().into_iter().collect(),
            apps: m.apps.clone(),
            native: m.is_native(),
            license: m.license.clone(),
            latest: newest.version,
            updated: newest.published.clone(),
            file_sha256: craft_ext_store::store::digest(&bytes),
            file_size: bytes.len() as u64,
            stars_date,
        });
        packages.push(package);
    }
    if only.is_some() && packages.is_empty() {
        return Err("--only plugin was not found".into());
    }
    let revocations: Revocations =
        if repo.join("revocations.toml").exists() { read(&repo.join("revocations.toml"))? } else { Revocations::default() };
    let now = craft_ext_store::store::now();
    let index = Index {
        schema: SCHEMA,
        serial: old.map(|i| i.serial).unwrap_or(0).checked_add(1).ok_or("serial overflow")?,
        issued: now,
        expires: now + 30 * 86400,
        vendors: vendors.values().map(VendorRecord::vendor).collect(),
        packages: summaries,
        revocations: revocations.revocation,
    };
    Index::parse(&serde_json::to_vec(&index)?, now, 0)?;
    Ok((index, packages, warnings))
}
pub fn build(repo: &Path, offline: bool) -> Result<Value> {
    let (i, packages, warnings) = assemble(repo, offline, None)?;
    fs::create_dir_all(repo.join("dist/packages"))?;
    for p in packages {
        fs::write(repo.join(format!("dist/packages/{}.json", p.id)), serde_json::to_vec_pretty(&p)?)?;
    }
    fs::write(repo.join("dist/index.json"), serde_json::to_vec_pretty(&i)?)?;
    let _ = fs::remove_file(repo.join("dist/index.json.sig"));
    Ok(json!({"serial":i.serial,"packages":i.packages.len(),"warnings":warnings,"next":"catalogue sign --repo <repo> --key <file>"}))
}
/// Build development metadata from real archives; signing remains a separate explicit step.
pub fn build_local(assets: &Path, out: &Path) -> Result<Value> {
    fs::create_dir_all(out)?;
    // Archives inside the catalogue directory get relative URLs, so the directory stays portable.
    let out_abs = out.canonicalize()?;
    let mut grouped: BTreeMap<ExtensionId, Vec<Release>> = BTreeMap::new();
    for path in entries(assets)? {
        if path.extension().is_none_or(|e| e != "craftplugin") {
            continue;
        }
        let bytes = fs::read(&path)?;
        let dir = temp_dir()?;
        let result = craft_ext_store::unpack::unpack_into(&bytes, &dir, craft_ext_store::unpack::Caps::default());
        let _ = fs::remove_dir_all(&dir);
        let m = result?;
        craft_ext_core::licence::check_osi(&m.license).map_err(|e| format!("non-OSI licence: {e}"))?;
        tier::check_id(VendorTier::Community.into(), &m.id)?;
        let platform = m.native.as_ref().and_then(|n| asset_platform(&n.executables));
        grouped.entry(m.id.clone()).or_default().push(Release {
            version: m.version,
            tag: format!("v{}", m.version),
            commit: "0".repeat(40),
            published: crate::author::date(craft_ext_store::store::now()),
            assets: vec![Asset {
                platform,
                url: asset_url(&path.canonicalize()?, &out_abs)?,
                sha256: craft_ext_store::store::digest(&bytes),
                size: bytes.len() as u64,
            }],
            manifest: m,
        });
    }
    fs::create_dir_all(out.join("packages"))?;
    let mut vendors = BTreeMap::new();
    let mut rows = vec![];
    for (id, mut releases) in grouped {
        releases.sort_by_key(|r| r.version);
        if releases.windows(2).any(|w| w[0].version == w[1].version) {
            return Err("duplicate local version".into());
        }
        let newest = releases.last().ok_or("no release")?;
        let m = &newest.manifest;
        let login = id.as_str().strip_prefix("io.github.").and_then(|s| s.split('.').next()).unwrap_or("local").to_string();
        vendors.insert(login.clone(), Vendor { login: login.clone(), github_id: 1, name: login.clone(), tier: VendorTier::Community, url: None });
        let package = PackageFile { schema: SCHEMA, id: id.clone(), vendor: login.clone(), releases: releases.clone() };
        let bytes = serde_json::to_vec_pretty(&package)?;
        PackageFile::parse(&bytes, &id)?;
        fs::write(out.join(format!("packages/{id}.json")), &bytes)?;
        rows.push(PackageSummary {
            id,
            vendor: login,
            name: m.name.clone(),
            summary: m.summary.clone(),
            repo: m.source.clone(),
            stars: 0,
            stars_date: crate::author::date(craft_ext_store::store::now()),
            tags: vec![],
            kinds: m.contributions.iter().map(|c| c.kind).collect::<BTreeSet<_>>().into_iter().collect(),
            apps: m.apps.clone(),
            native: m.is_native(),
            license: m.license.clone(),
            latest: m.version,
            updated: newest.published.clone(),
            file_sha256: craft_ext_store::store::digest(&bytes),
            file_size: bytes.len() as u64,
        });
    }
    let now = craft_ext_store::store::now();
    let serial = match fs::read(out.join("index.json")) {
        Ok(bytes) => Index::parse_cached(&bytes)?.serial.checked_add(1).ok_or("serial overflow")?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => 1,
        Err(e) => return Err(e.into()),
    };
    let revocations: Revocations =
        if assets.join("revocations.toml").exists() { read(&assets.join("revocations.toml"))? } else { Revocations::default() };
    let index = Index {
        schema: SCHEMA,
        serial,
        issued: now,
        expires: now + 30 * 86400,
        vendors: vendors.into_values().collect(),
        packages: rows,
        revocations: revocations.revocation,
    };
    let bytes = serde_json::to_vec_pretty(&index)?;
    Index::parse(&bytes, now, 0)?;
    fs::write(out.join("index.json"), bytes)?;
    let _ = fs::remove_file(out.join("index.json.sig"));
    Ok(json!({"packages":index.packages.len(),"next":"catalogue sign --repo <out> --key <file>"}))
}
/// Which platform a local native archive serves. Distinct paths per platform make one universal
/// archive (every helper is inside); one shared path means one archive per platform, built here.
fn asset_platform(executables: &BTreeMap<String, String>) -> Option<String> {
    let distinct = executables.values().collect::<BTreeSet<_>>().len() == executables.len();
    match (distinct, executables.len()) {
        (true, 1) => executables.keys().next().cloned(),
        (true, _) => None,
        (false, _) => Some(craft_ext_host::native::platform()),
    }
}
fn asset_url(path: &Path, out: &Path) -> Result<String> {
    if let Ok(rel) = path.strip_prefix(out) {
        let parts = rel.components().map(|c| c.as_os_str().to_str().map(str::to_string)).collect::<Option<Vec<_>>>();
        if let Some(parts) = parts {
            let url = parts.join("/");
            if craft_ext_store::catalogue::resolve_asset_url(&url, "file:///").is_ok() {
                return Ok(url);
            }
        }
    }
    Ok(format!("file://{}", path.display()))
}
pub fn run(args: &[String]) -> Result<Value> {
    let a = Args::parse(args, &["repo", "only", "key", "out", "local-assets"], &["offline"])?;
    if a.pos.is_empty() || a.has("help") {
        return Ok(json!(
            "catalogue check --repo <dir> [--offline] [--only id] | build --repo <dir> [--offline] | build --local-assets <dir> --out <dir> | sign --repo <dir> --key <32-byte-hex-seed-file> | keygen --repo <dir> --out <file>. All accept --json. Review source/build provenance, free behaviour and permissions manually."
        ));
    }
    a.arity(1)?;
    if let (true, Some(assets)) = (a.at(0)? == "build", a.opt("local-assets")) {
        return build_local(Path::new(assets), Path::new(a.opt("out").ok_or_else(|| Usage("--out is required".into()))?));
    }
    let repo = Path::new(a.opt("repo").ok_or_else(|| Usage("--repo is required".into()))?);
    match a.at(0)? {
        "check" => {
            let (i, _, warnings) = assemble(repo, a.has("offline"), a.opt("only"))?;
            Ok(
                json!({"packages":i.packages.len(),"warnings":warnings,"checklist":["Schema, vendor, namespace, OSI licence, source identity and assets checked",if a.has("offline"){"Online repository, tag and archive checks skipped"}else{"Public repository, numeric id, licence, tag, size, digest and unpacked manifest checked"},"MANUAL: confirm source-to-build provenance","MANUAL: review free behaviour, monetisation wording and permission reasons"]}),
            )
        }
        "build" => build(repo, a.has("offline")),
        "keygen" => {
            let path = a.opt("out").ok_or_else(|| Usage("--out is required".into()))?;
            let mut seed = [0; 32];
            getrandom::fill(&mut seed).map_err(|e| e.to_string())?;
            let key = SigningKey::from_bytes(&seed);
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            use std::io::Write;
            options.open(path)?.write_all(hex::encode(seed).as_bytes())?;
            Ok(
                json!({"public_key":hex::encode(key.verifying_key().as_bytes()),"instruction":"Paste public_key into craft_ext_core::catalogue::TRUSTED_KEYS","test_instruction":"For a local test catalogue only (debug builds, file:// catalogue): export CRAFT_CATALOGUE_TEST_KEY=<public_key> CRAFT_CATALOGUE_ALLOW_FILE=1"}),
            )
        }
        "sign" => {
            let key = a.opt("key").ok_or_else(|| Usage("--key is required".into()))?;
            let bytes = hex::decode(fs::read_to_string(key)?.trim()).map_err(|_| "key file must contain a hex seed")?;
            let seed: [u8; 32] = bytes.try_into().map_err(|_| "key seed must be 32 bytes")?;
            let dist = if repo.join("index.json").is_file() { repo.to_path_buf() } else { repo.join("dist") };
            let index = fs::read(dist.join("index.json"))?;
            Index::parse(&index, craft_ext_store::store::now(), 0)?;
            let sig = SigningKey::from_bytes(&seed).sign(&index);
            fs::write(dist.join("index.json.sig"), hex::encode(sig.to_bytes()))?;
            Ok(json!("Catalogue signed"))
        }
        cmd => Err(Usage(format!("unknown catalogue command {cmd}")).into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Release {
        let manifest = craft_ext_core::manifest::Manifest::parse(
            br#"schema=2
id="io.github.alice.test"
name="Test"
version="1.0.0"
license="MIT"
source="https://github.com/alice/test"
summary="A test"
apps=["effectcraft"]
[[contribution]]
id="data"
kind="preset"
contract="resource.preset@1.0"
label="Data"
files=["data.json"]
"#,
        )
        .unwrap();
        Release {
            version: manifest.version,
            tag: "v1.0.0".into(),
            commit: "a".repeat(40),
            published: "2026-11-12".into(),
            assets: vec![Asset {
                platform: None,
                url: "https://github.com/alice/test/releases/download/v1.0.0/test.craftplugin".into(),
                sha256: "0".repeat(64),
                size: 100,
            }],
            manifest,
        }
    }
    #[test]
    fn release_round_trip() {
        let r = fixture();
        let parsed: Release = toml::from_str(&toml::to_string_pretty(&r).unwrap()).unwrap();
        assert_eq!(r, parsed);
        assert_eq!(crate::author::date(0), "1970-01-01");
        assert_eq!(crate::author::date(951782400), "2000-02-29");
    }
    #[test]
    fn offline_build_and_sign() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path();
        fs::create_dir_all(p.join("vendors")).unwrap();
        fs::create_dir_all(p.join("plugins/io.github.alice.test/releases")).unwrap();
        fs::write(p.join("vendors/alice.toml"), "login='alice'\ngithub_id=1\nname='Alice'\ntier='community'\n").unwrap();
        fs::write(
            p.join("plugins/io.github.alice.test/listing.toml"),
            "id='io.github.alice.test'\nvendor='alice'\nrepo='https://github.com/alice/test'\nrepo_id=2\ntags=[]\n",
        )
        .unwrap();
        fs::write(p.join("plugins/io.github.alice.test/releases/1.0.0.toml"), toml::to_string_pretty(&fixture()).unwrap()).unwrap();
        build(p, true).unwrap();
        let bytes = fs::read(p.join("dist/index.json")).unwrap();
        let index = Index::parse(&bytes, craft_ext_store::store::now(), 0).unwrap();
        let row = &index.packages[0];
        let package = fs::read(p.join(format!("dist/packages/{}.json", row.id))).unwrap();
        assert_eq!(row.file_sha256, craft_ext_store::store::digest(&package));
        assert_eq!(row.file_size, package.len() as u64);
        let key = SigningKey::from_bytes(&[7; 32]);
        let keyfile = p.join("key");
        fs::write(&keyfile, hex::encode([7; 32])).unwrap();
        run(&["sign".into(), "--repo".into(), p.to_str().unwrap().into(), "--key".into(), keyfile.to_str().unwrap().into()]).unwrap();
        let sig = fs::read_to_string(p.join("dist/index.json.sig")).unwrap();
        let public = hex::encode(key.verifying_key().as_bytes());
        assert!(craft_ext_store::catalogue::verify_index(&bytes, &sig, &[&public]));
        assert!(!craft_ext_store::catalogue::verify_index(b"tampered", &sig, &[&public]));
        build(p, true).unwrap();
        assert_eq!(Index::parse_cached(&fs::read(p.join("dist/index.json")).unwrap()).unwrap().serial, 2);
        let release_path = p.join("plugins/io.github.alice.test/releases/1.0.0.toml");
        let mut release = fixture();
        release.manifest.summary = "Premium plugin".into();
        fs::write(&release_path, toml::to_string_pretty(&release).unwrap()).unwrap();
        assert!(assemble(p, true, None).unwrap_err().to_string().contains("monetisation_reviewed"));
        let listing = p.join("plugins/io.github.alice.test/listing.toml");
        let original = fs::read_to_string(&listing).unwrap();
        fs::write(&listing, format!("{original}monetisation_reviewed='  '\n")).unwrap();
        assert!(assemble(p, true, None).is_err());
        fs::write(&listing, format!("{original}monetisation_reviewed='Maintainer confirmed this is free'\n")).unwrap();
        assert!(assemble(p, true, None).is_ok());
    }
}

#[cfg(test)]
mod beta_tests {
    use super::*;
    #[test]
    #[ignore = "maintainer publication: needs CRAFT_CATALOGUE_KEY and CRAFT_PLUGINS_REPO"]
    fn publish_catalogue() {
        let key = std::env::var("CRAFT_CATALOGUE_KEY").expect("CRAFT_CATALOGUE_KEY");
        let repo = std::env::var("CRAFT_PLUGINS_REPO").expect("CRAFT_PLUGINS_REPO");
        let keypath = Path::new(&key);
        if !keypath.exists() {
            fs::create_dir_all(keypath.parent().unwrap()).unwrap();
            run(&["keygen".into(), "--repo".into(), repo.clone(), "--out".into(), key.clone()]).unwrap();
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(keypath, fs::Permissions::from_mode(0o600)).unwrap();
        }
        build(Path::new(&repo), true).unwrap();
        run(&["sign".into(), "--repo".into(), repo.clone(), "--key".into(), key.clone()]).unwrap();
        let seed: [u8; 32] = hex::decode(fs::read_to_string(keypath).unwrap().trim()).unwrap().try_into().unwrap();
        let public = hex::encode(SigningKey::from_bytes(&seed).verifying_key().as_bytes());
        // Only the public key is printed, never the secret seed.
        println!("public_key={public}");
    }
    #[test]
    fn repository_release_records_parse() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../craft-plugins");
        if !repo.exists() {
            return;
        }
        let mut count = 0;
        for root in [repo.join("plugins"), repo.join("examples/pending")] {
            for dir in entries(&root).unwrap().into_iter().filter(|p| p.is_dir()) {
                for file in entries(&dir.join("releases")).unwrap() {
                    let release: Release = read(&file).unwrap();
                    release.manifest.validate().unwrap();
                    count += 1;
                }
            }
        }
        // Film looks has both 1.0.0 and 1.1.0; halftone, color notes and figcraft-shapes have one each.
        assert_eq!(count, 5);
        // Test the current source records, independently of previously generated/signed dist files.
        let fresh = tempfile::tempdir().unwrap();
        fn copy_source(from: &Path, to: &Path) {
            fs::create_dir_all(to).unwrap();
            for path in entries(from).unwrap() {
                let target = to.join(path.file_name().unwrap());
                if path.is_dir() {
                    copy_source(&path, &target);
                } else {
                    fs::copy(path, target).unwrap();
                }
            }
        }
        for dir in ["plugins", "vendors"] {
            copy_source(&repo.join(dir), &fresh.path().join(dir));
        }
        let (_, packages, _) = assemble(fresh.path(), true, None).unwrap();
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].id.as_str(), "io.github.arnav-hiwarkar.film-looks");
    }
    #[test]
    fn asset_repository_tag_and_file_are_bound() {
        let repo = "https://github.com/alice/test";
        assert!(validate_asset_url(repo, "v1", "https://github.com/alice/test/releases/download/v1/a.zip").is_ok());
        for url in [
            "https://github.com/bob/test/releases/download/v1/a.zip",
            "https://github.com/alice/test/releases/download/v2/a.zip",
            "https://github.com/alice/test/releases/download/v1/a.zip?x=1",
            "https://github.com/alice/test/releases/download/v1/../a.zip",
        ] {
            assert!(validate_asset_url(repo, "v1", url).is_err());
        }
    }
}

#[cfg(test)]
mod local_tests {
    use super::*;
    #[test]
    fn native_asset_platform_follows_the_archive_layout() {
        let map = |pairs: &[(&str, &str)]| pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect::<BTreeMap<_, _>>();
        assert_eq!(asset_platform(&map(&[("macos-aarch64", "bin/a")])).as_deref(), Some("macos-aarch64"));
        assert_eq!(asset_platform(&map(&[("macos-aarch64", "bin/a"), ("linux-x86_64", "bin/b")])), None);
        assert_eq!(asset_platform(&map(&[("macos-aarch64", "bin/a"), ("linux-x86_64", "bin/a")])), Some(craft_ext_host::native::platform()));
    }
    #[test]
    fn local_build_uses_archive_identity_and_invalidates_signature() {
        let t = tempfile::tempdir().unwrap();
        let project = t.path().join("project");
        crate::author::run(&["new".into(), "data".into(), project.to_str().unwrap().into(), "--id".into(), "io.github.alice.local".into()]).unwrap();
        let assets = t.path().join("assets");
        fs::create_dir(&assets).unwrap();
        crate::author::pack(&project, Some(assets.join("one.craftplugin").to_str().unwrap())).unwrap();
        let out = t.path().join("catalogue");
        build_local(&assets, &out).unwrap();
        let i = Index::parse_cached(&fs::read(out.join("index.json")).unwrap()).unwrap();
        assert_eq!(i.packages.len(), 1);
        let p = PackageFile::parse(&fs::read(out.join("packages/io.github.alice.local.json")).unwrap(), &i.packages[0].id).unwrap();
        assert!(p.releases[0].assets[0].url.starts_with("file:///"));
        // Archives inside the catalogue directory are referenced relatively.
        let inside = out.join("assets");
        fs::create_dir(&inside).unwrap();
        fs::copy(assets.join("one.craftplugin"), inside.join("one.craftplugin")).unwrap();
        build_local(&inside, &out).unwrap();
        let p = PackageFile::parse(&fs::read(out.join("packages/io.github.alice.local.json")).unwrap(), &i.packages[0].id).unwrap();
        assert_eq!(p.releases[0].assets[0].url, "assets/one.craftplugin");
        build_local(&assets, &out).unwrap();
        assert!(validate_asset_url(&p.releases[0].manifest.source, "v0.1.0", &p.releases[0].assets[0].url).is_err());
        fs::write(out.join("index.json.sig"), "stale").unwrap();
        build_local(&assets, &out).unwrap();
        assert!(!out.join("index.json.sig").exists());
        let next = Index::parse_cached(&fs::read(out.join("index.json")).unwrap()).unwrap();
        assert_eq!(next.serial, i.serial + 3);
    }
}
