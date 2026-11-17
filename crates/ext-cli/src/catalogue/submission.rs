use super::{Assembled, Listing, VendorRecord, assemble, check_online, read};
use craft_ext_core::{
    catalogue::{Index, PackageFile, Release},
    listing::Attestation,
    review::{id_tail, too_similar, widenings},
    tier::VendorTier,
    version::Version,
};
use serde::Serialize;
use std::{fs, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Pass,
    NeedsReview,
    Fail,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub check: &'static str,
    pub status: Status,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct Recorded {
    pub version: Version,
    pub attestation: Attestation,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub id: String,
    pub verdict: Status,
    pub checks: Vec<Check>,
    pub attestations: Vec<Recorded>,
}

pub const CHECKS: [&str; 6] = ["schema", "names", "permissions", "source", "immutable-release", "attestation"];

fn check(check: &'static str, status: Status, detail: impl Into<String>) -> Check {
    Check { check, status, detail: detail.into() }
}

fn not_run(why: &str) -> Vec<Check> {
    CHECKS.iter().skip(1).map(|c| check(c, Status::Fail, format!("not run: {why}"))).collect()
}

fn published(repo: &Path, id: &str) -> Option<PackageFile> {
    let id = craft_ext_core::id::ExtensionId::new(id).ok()?;
    PackageFile::parse(&fs::read(repo.join(format!("dist/packages/{id}.json"))).ok()?, &id).ok()
}

fn names(repo: &Path, id: &str, name: &str) -> Check {
    let index = match fs::read(repo.join("dist/index.json")) {
        Ok(b) => match Index::parse_cached(&b) {
            Ok(i) => i,
            Err(e) => return check("names", Status::Fail, format!("the published index cannot be read: {e}")),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return check("names", Status::Pass, "no published listings to compare"),
        Err(e) => return check("names", Status::Fail, format!("the published index cannot be read: {e}")),
    };
    let clashes: Vec<String> = index
        .packages
        .iter()
        .filter(|p| p.id.as_str() != id)
        .filter(|p| too_similar(&p.name, name) || too_similar(id_tail(p.id.as_str()), id_tail(id)))
        .map(|p| format!("{} ({})", p.name, p.id))
        .collect();
    if clashes.is_empty() {
        check("names", Status::Pass, format!("{name} is distinct from {} listed plugins", index.packages.len()))
    } else {
        check("names", Status::NeedsReview, format!("looks like {}", clashes.join(", ")))
    }
}

fn permissions(all: &[Release], new: &[&Release]) -> Check {
    let mut wider = vec![];
    for r in new {
        let previous = all.iter().filter(|p| p.version < r.version).max_by_key(|p| p.version);
        wider.extend(widenings(previous.map(|p| &p.manifest), &r.manifest).into_iter().map(|w| format!("{}: {w}", r.version)));
    }
    if wider.is_empty() {
        check("permissions", Status::Pass, "no new release asks for more than the one before it")
    } else {
        check("permissions", Status::NeedsReview, wider.join("; "))
    }
}

pub fn run(repo: &Path, id: &str, offline: bool) -> Report {
    let mut checks = vec![];
    let mut attestations = vec![];
    match assemble(repo, true, Some(id)) {
        Err(e) => {
            checks.push(check("schema", Status::Fail, e.to_string()));
            checks.extend(not_run("the listing did not validate"));
        }
        Ok(Assembled { index, packages, .. }) => {
            checks.push(check("schema", Status::Pass, "listing, releases, sizes and media are valid"));
            let (Some(row), Some(package)) = (index.packages.first(), packages.first()) else {
                checks.extend(not_run("the listing has no package"));
                return report(id, checks, attestations);
            };
            checks.push(names(repo, id, &row.name));
            let old = published(repo, id).map(|p| p.releases.into_iter().map(|r| r.version).collect::<Vec<_>>()).unwrap_or_default();
            let new: Vec<&Release> = package.releases.iter().filter(|r| !old.contains(&r.version)).collect();
            checks.push(permissions(&package.releases, &new));
            let vendor_tier = index.vendor(&row.vendor).map(|v| v.tier);
            online(repo, id, &new, vendor_tier, offline, &mut checks, &mut attestations);
        }
    }
    report(id, checks, attestations)
}

fn online(repo: &Path, id: &str, new: &[&Release], tier: Option<VendorTier>, offline: bool, checks: &mut Vec<Check>, attestations: &mut Vec<Recorded>) {
    if offline {
        checks.extend(["source", "immutable-release", "attestation"].map(|c| check(c, Status::Fail, "not run: --offline")));
        return;
    }
    if new.is_empty() {
        checks.extend(["source", "immutable-release", "attestation"].map(|c| check(c, Status::Pass, "no new release")));
        return;
    }
    let dir = repo.join("plugins").join(id);
    let records = read::<Listing>(&dir.join("listing.toml")).and_then(|l| {
        let v = read::<VendorRecord>(&repo.join(format!("vendors/{}.toml", l.vendor)))?;
        Ok((l, v))
    });
    let (listing, vendor) = match records {
        Ok(r) => r,
        Err(e) => {
            checks.extend(["source", "immutable-release", "attestation"].map(|c| check(c, Status::Fail, format!("not run: {e}"))));
            return;
        }
    };
    let mut source = vec![];
    let mut mutable = vec![];
    let mut attest = vec![];
    for r in new {
        let facts = match check_online(&listing, &vendor, r) {
            Ok(f) => f,
            Err(e) => {
                source.push(format!("{}: {e}", r.version));
                continue;
            }
        };
        if !facts.immutable {
            mutable.push(r.version.to_string());
        }
        let required = r.manifest.is_native() || tier.is_some_and(|t| t != VendorTier::Community);
        match facts.attestation {
            Ok(Some(a)) => attestations.push(Recorded { version: r.version, attestation: a }),
            Ok(None) if required => attest.push(format!("{}: native and verified or official listings need signer_workflow and a build attestation", r.version)),
            Ok(None) => {}
            Err(e) => attest.push(format!("{}: {e}", r.version)),
        }
    }
    let ran = source.is_empty();
    checks.push(if ran {
        check("source", Status::Pass, "public repository, owner, licence, tag, digests and unpacked manifests match")
    } else {
        check("source", Status::Fail, source.join("; "))
    });
    checks.push(match (ran, mutable.is_empty()) {
        (false, _) => check("immutable-release", Status::Fail, "not run: the source check failed"),
        (true, true) => check("immutable-release", Status::Pass, "every new GitHub release is immutable"),
        (true, false) => check("immutable-release", Status::Fail, format!("not immutable: {}; enable release immutability and publish again", mutable.join(", "))),
    });
    checks.push(match (ran, attest.is_empty(), attestations.is_empty()) {
        (false, _, _) => check("attestation", Status::Fail, "not run: the source check failed"),
        (true, false, _) => check("attestation", Status::Fail, attest.join("; ")),
        (true, true, false) => check("attestation", Status::Pass, "verified with gh attestation verify; recorded as Built from source"),
        (true, true, true) => check("attestation", Status::Pass, "no signer_workflow; listed without the Built from source badge"),
    });
}

fn report(id: &str, checks: Vec<Check>, attestations: Vec<Recorded>) -> Report {
    let verdict = checks.iter().map(|c| c.status).max().unwrap_or(Status::Fail);
    Report { id: id.into(), verdict, checks, attestations }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::super::listing_tests::repository;
    use super::*;

    fn statuses(r: &Report) -> Vec<(&'static str, Status)> {
        r.checks.iter().map(|c| (c.check, c.status)).collect()
    }

    #[test]
    fn offline_run_fails_closed_on_the_checks_it_cannot_run() {
        let d = repository("categories=['color-looks']\n");
        let r = run(d.path(), "io.github.alice.test", true);
        assert_eq!(
            statuses(&r),
            vec![
                ("schema", Status::Pass),
                ("names", Status::Pass),
                ("permissions", Status::Pass),
                ("source", Status::Fail),
                ("immutable-release", Status::Fail),
                ("attestation", Status::Fail)
            ]
        );
        assert_eq!(r.verdict, Status::Fail);
        assert_eq!(r.checks[3].detail, "not run: --offline");
    }

    #[test]
    fn a_bad_listing_fails_every_check() {
        let d = repository("homepage='http://alice.example'\n");
        let r = run(d.path(), "io.github.alice.test", true);
        assert_eq!(r.checks[0].detail, "io.github.alice.test: homepage: must start with https://");
        assert!(r.checks.iter().all(|c| c.status == Status::Fail));
        assert_eq!(r.checks.len(), CHECKS.len());
    }

    #[test]
    fn lookalike_names_and_wider_permissions_need_review() {
        let d = repository("");
        super::super::build(d.path(), true).unwrap();
        let index_path = d.path().join("dist/index.json");
        let mut index = Index::parse_cached(&fs::read(&index_path).unwrap()).unwrap();
        let mut twin = index.packages[0].clone();
        twin.id = craft_ext_core::id::ExtensionId::new("io.github.mallory.test").unwrap();
        twin.name = "Te5t".into();
        index.packages.push(twin);
        fs::write(&index_path, serde_json::to_vec(&index).unwrap()).unwrap();
        let mut next = super::super::tests::fixture();
        next.version = Version::new(1, 1, 0);
        next.manifest.version = next.version;
        next.tag = "v1.1.0".into();
        next.assets[0].url = "https://github.com/alice/test/releases/download/v1.1.0/test.craftplugin".into();
        next.manifest.contributions[0].permissions = vec![craft_ext_core::permission::Permission::parse("clipboard.read").unwrap()];
        fs::write(d.path().join("plugins/io.github.alice.test/releases/1.1.0.toml"), toml::to_string_pretty(&next).unwrap()).unwrap();
        let r = run(d.path(), "io.github.alice.test", true);
        assert_eq!(r.checks[1].status, Status::NeedsReview);
        assert_eq!(r.checks[1].detail, "looks like Te5t (io.github.mallory.test)");
        assert_eq!(r.checks[2].detail, "1.1.0: adds permission clipboard.read (Read the clipboard when you use it)");
        assert_eq!(r.checks[2].status, Status::NeedsReview);
    }
}
