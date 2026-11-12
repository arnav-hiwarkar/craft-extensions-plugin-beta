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
fn check_online(l: &Listing, r: &Release, warnings: &mut Vec<String>) -> Result<()> {
    let repo = github(&l.repo, "")?;
    if repo["private"].as_bool() != Some(false) {
        return Err("repository must be public".into());
    }
    if repo["id"].as_u64() != Some(l.repo_id) {
        return Err("repository numeric id differs".into());
    }
    let license = repo["license"]["spdx_id"].as_str().unwrap_or("NOASSERTION");
    if license == "NOASSERTION" {
        warnings.push(format!("{}: GitHub did not detect a licence", l.id));
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
                if !a.url.starts_with("https://") || a.size == 0 || a.sha256.len() != 64 || hex::decode(&a.sha256).is_err() {
                    return Err("invalid release asset".into());
                }
            }
            for word in craft_ext_core::licence::monetisation_flags(&format!("{} {} {}", r.manifest.name, r.manifest.summary, r.manifest.description))
            {
                warnings.push(format!("{}: review monetisation wording {word}", l.id));
            }
            if !offline {
                check_online(&l, &r, &mut warnings)?
            }
            releases.push(r);
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
pub fn run(args: &[String]) -> Result<Value> {
    let a = Args::parse(args, &["repo", "only", "key", "out"], &["offline"])?;
    if a.pos.is_empty() || a.has("help") {
        return Ok(json!(
            "catalogue check --repo <dir> [--offline] [--only id] | build --repo <dir> [--offline] | sign --repo <dir> --key <32-byte-hex-seed-file> | keygen --repo <dir> --out <file>. All accept --json. Review source/build provenance, free behaviour and permissions manually."
        ));
    }
    a.arity(1)?;
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
                json!({"public_key":hex::encode(key.verifying_key().as_bytes()),"instruction":"Paste public_key into craft_ext_core::catalogue::TRUSTED_KEYS"}),
            )
        }
        "sign" => {
            let key = a.opt("key").ok_or_else(|| Usage("--key is required".into()))?;
            let bytes = hex::decode(fs::read_to_string(key)?.trim()).map_err(|_| "key file must contain a hex seed")?;
            let seed: [u8; 32] = bytes.try_into().map_err(|_| "key seed must be 32 bytes")?;
            let index = fs::read(repo.join("dist/index.json"))?;
            Index::parse(&index, craft_ext_store::store::now(), 0)?;
            let sig = SigningKey::from_bytes(&seed).sign(&index);
            fs::write(repo.join("dist/index.json.sig"), hex::encode(sig.to_bytes()))?;
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
    }
}
