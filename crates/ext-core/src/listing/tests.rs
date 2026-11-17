#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
use super::*;
use crate::catalogue::{Index, PackageFile, SCHEMA};
use crate::id::ExtensionId;

const V1_0_INDEX: &str = r#"{"schema":1,"serial":7,"issued":0,"expires":99,"vendors":[{"login":"alice","github_id":1,"name":"Alice","tier":"community"}],
"packages":[{"id":"io.github.alice.blur","name":"Soft Blur","summary":"Blurs gently","vendor":"alice","repo":"https://github.com/alice/blur",
"kinds":["effect"],"apps":["effectcraft"],"native":false,"license":"MIT","latest":"1.0.0","updated":"2026-11-01","file_sha256":"00","file_size":1}]}"#;

fn v1_1_index() -> String {
    V1_0_INDEX
        .replace(r#""schema":1,"#, r#""schema":1,"schema_minor":1,"featured":["io.github.alice.blur"],"#)
        .replace(
            r#""file_size":1}"#,
            r#""file_size":1,"description":"Long text","categories":["effects-filters","sparkles-from-the-future"],
"icon":{"url":"media/io.github.alice.blur/icon.png","sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","size":10,"width":64,"height":64},
"homepage":"https://alice.example","downloads":{"count":42,"date":"2026-11-16"},"a_field_from_schema_1_2":true}"#,
        )
}

#[derive(serde::Deserialize)]
struct OldIndex {
    schema: u32,
    packages: Vec<OldSummary>,
}

#[derive(serde::Deserialize)]
struct OldSummary {
    id: String,
    summary: String,
    tags: Option<Vec<String>>,
}

#[test]
fn a_1_0_reader_reads_a_1_1_index() {
    let old: OldIndex = serde_json::from_str(&v1_1_index()).unwrap();
    assert_eq!(old.schema, SCHEMA);
    assert_eq!(old.packages[0].id, "io.github.alice.blur");
    assert_eq!(old.packages[0].summary, "Blurs gently");
    assert_eq!(old.packages[0].tags, None);
}

#[test]
fn a_1_1_reader_reads_1_0_and_tolerates_unknown_categories() {
    let old = Index::parse(V1_0_INDEX.as_bytes(), 0, 0).unwrap();
    assert_eq!((old.schema_minor, old.featured.len()), (0, 0));
    assert_eq!(old.packages[0].categories, vec![]);
    assert_eq!(old.packages[0].description, "");
    let new = Index::parse(v1_1_index().as_bytes(), 0, 0).unwrap();
    assert_eq!(new.schema_minor, 1);
    assert_eq!(new.featured, vec![ExtensionId::new("io.github.alice.blur").unwrap()]);
    let row = &new.packages[0];
    assert_eq!(row.categories, vec![Category::EffectsFilters]);
    assert_eq!(row.downloads, Some(Downloads { count: 42, date: "2026-11-16".into() }));
    assert_eq!(row.icon.as_ref().map(|i| i.width), Some(64));
}

#[test]
fn a_listing_without_new_fields_serialises_exactly_as_1_0() {
    let index = Index::parse(V1_0_INDEX.as_bytes(), 0, 0).unwrap();
    let text = serde_json::to_string(&index.packages[0]).unwrap();
    for key in ["description", "categories", "icon", "screenshots", "homepage", "downloads"] {
        assert!(!text.contains(&format!("\"{key}\"")), "{key} in {text}");
    }
    assert!(!serde_json::to_string(&index).unwrap().contains("schema_minor"));
}

#[test]
fn release_fields_are_optional_and_old_package_files_parse() {
    let release = r#"{"version":"1.0.0","tag":"v1.0.0","commit":"COMMIT","published":"2026-11-01","assets":[],"manifest":MANIFEST}"#;
    let manifest = serde_json::to_string(&crate::manifest::Manifest::parse(crate::manifest::tests::SAMPLE.as_bytes()).unwrap()).unwrap();
    let id = crate::manifest::Manifest::parse(crate::manifest::tests::SAMPLE.as_bytes()).unwrap().id;
    let file = format!(r#"{{"schema":1,"id":"{id}","vendor":"alice","releases":[{}]}}"#, release.replace("MANIFEST", &manifest).replace("1.0.0", "1.2.0"));
    let p = PackageFile::parse(file.as_bytes(), &id).unwrap();
    assert_eq!((p.releases[0].changelog.as_str(), p.releases[0].attestation.is_none()), ("", true));
    let with = file.replace(r#""published""#, r#""changelog":"Faster","attestation":{"verified":true,"signer_workflow":".github/workflows/release.yml","source_commit":"COMMIT"},"published""#);
    let p = PackageFile::parse(with.as_bytes(), &id).unwrap();
    assert_eq!(p.releases[0].changelog, "Faster");
    assert_eq!(p.releases[0].attestation.as_ref().map(|a| a.verified), Some(true));
}

fn summary() -> crate::catalogue::PackageSummary {
    Index::parse(v1_1_index().as_bytes(), 0, 0).unwrap().packages.remove(0)
}

fn problem(p: &crate::catalogue::PackageSummary) -> String {
    p.check_listing().unwrap_err().to_string()
}

#[test]
fn valid_listing_passes_and_bad_values_are_refused_not_clamped() {
    let ok = summary();
    assert_eq!(ok.check_listing(), Ok(()));
    let mut p = ok.clone();
    p.summary = "x".repeat(141);
    assert_eq!(problem(&p), "summary: the text is too long");
    let mut p = ok.clone();
    p.description = "y".repeat(8 * 1024 + 1);
    assert_eq!(problem(&p), "description: is 8193 bytes; at most 8192 are allowed");
    let mut p = ok.clone();
    p.tags = (0..11).map(|i| format!("t{i}")).collect();
    assert_eq!(problem(&p), "tags: 11 given; at most 10 are allowed");
    let mut p = ok.clone();
    p.categories = vec![Category::ThreeDParticles];
    p.apps = vec![crate::id::AppId::new("figcraft").unwrap()];
    assert_eq!(problem(&p), "categories: 3d-particles does not apply to any app this plugin supports");
    let mut p = ok.clone();
    p.categories = vec![Category::Utilities, Category::Resources, Category::Generators];
    assert_eq!(problem(&p), "categories: 3 given; at most 2 are allowed");
    let mut p = ok.clone();
    p.icon.as_mut().unwrap().width = 65;
    assert_eq!(problem(&p), "icon: is 65x64; an icon must be square");
    let mut p = ok.clone();
    p.icon.as_mut().unwrap().size = 256 * 1024 + 1;
    assert_eq!(problem(&p), "icon: is 262145 bytes; it must be 1 to 262144 bytes");
    let mut p = ok.clone();
    p.icon.as_mut().unwrap().url = "https://example.com/icon.png".into();
    assert!(problem(&p).starts_with("icon: must be a .png path"));
    let mut p = ok.clone();
    let shot = Image { url: "https://github.com/alice/blur/releases/download/v1.0.0/shot.png".into(), width: 1280, height: 720, size: 1000, ..ok.icon.clone().unwrap() };
    p.screenshots = vec![shot; 7];
    assert_eq!(problem(&p), "screenshots: 7 given; at most 6 are allowed");
    p.screenshots.truncate(6);
    assert_eq!(p.check_listing(), Ok(()));
    p.homepage = Some("http://alice.example".into());
    assert_eq!(problem(&p), "homepage: must start with https://");
}

#[test]
fn release_listing_limits() {
    let manifest = crate::manifest::Manifest::parse(crate::manifest::tests::SAMPLE.as_bytes()).unwrap();
    let mut r = crate::catalogue::Release {
        version: manifest.version,
        tag: "v1".into(),
        commit: "c".repeat(40),
        published: "2026-11-01".into(),
        assets: vec![],
        manifest,
        changelog: "z".repeat(4097),
        attestation: None,
        downloads: None,
    };
    assert_eq!(r.check_listing().unwrap_err().to_string(), "changelog: is 4097 bytes; at most 4096 are allowed");
    r.changelog = "Fixed the edges".into();
    r.attestation = Some(Attestation { verified: true, signer_workflow: ".github/workflows/release.yml".into(), source_commit: "d".repeat(40) });
    assert_eq!(r.check_listing().unwrap_err().to_string(), "attestation: source_commit differs from the release commit");
    r.attestation = Some(Attestation { verified: true, signer_workflow: "release.yml".into(), source_commit: "c".repeat(40) });
    assert_eq!(r.check_listing().unwrap_err().to_string(), "attestation: must be a workflow file such as .github/workflows/release.yml");
    r.attestation.as_mut().unwrap().signer_workflow = ".github/workflows/release.yml".into();
    r.downloads = Some(Downloads { count: 3, date: "16 Nov".into() });
    assert_eq!(r.check_listing().unwrap_err().to_string(), "downloads: date must be YYYY-MM-DD");
    r.downloads = Some(Downloads { count: 3, date: "2026-11-16".into() });
    assert_eq!(r.check_listing(), Ok(()));
}

#[test]
fn featured_must_name_listed_packages_once() {
    let mut i = Index::parse(v1_1_index().as_bytes(), 0, 0).unwrap();
    assert_eq!(i.check_listing(), Ok(()));
    i.featured.push(i.featured[0].clone());
    assert_eq!(i.check_listing().unwrap_err().to_string(), "featured: io.github.alice.blur is listed twice");
    i.featured = vec![ExtensionId::new("io.github.alice.gone").unwrap()];
    assert_eq!(i.check_listing().unwrap_err().to_string(), "featured: io.github.alice.gone is not a listed package");
}

#[test]
fn taxonomy_is_one_table_in_enum_order() {
    for (i, row) in CATEGORIES.iter().enumerate() {
        assert_eq!(row.category as usize, i);
        assert_eq!(serde_json::to_string(&row.category).unwrap(), format!("\"{}\"", row.key));
        assert_eq!(Category::from_key(row.key), Some(row.category));
    }
    assert_eq!(Category::for_app("figcraft").count(), 9);
    assert_eq!(Category::for_app("effectcraft").count(), 11);
    assert_eq!(Category::ThreeDParticles.row().english, "3D & particles");
    assert!(serde_json::from_str::<Category>("\"unknown\"").is_err());
}

#[test]
fn png_header_and_bytes_are_checked() {
    let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\x0dIHDR".to_vec();
    png.extend_from_slice(&64u32.to_be_bytes());
    png.extend_from_slice(&48u32.to_be_bytes());
    assert_eq!(png_dimensions(&png), Some((64, 48)));
    assert_eq!(png_dimensions(b"GIF89a"), None);
    let image = Image { url: "media/a.png".into(), sha256: "a".repeat(64), size: png.len() as u64, width: 64, height: 48 };
    assert_eq!(image.check_bytes(ImageRole::Screenshot, &png, &"a".repeat(64)), Ok(()));
    assert_eq!(image.check_bytes(ImageRole::Screenshot, &png, &"b".repeat(64)).unwrap_err().to_string(), "screenshot: the file differs from its recorded size or sha256");
    let tall = Image { height: 49, ..image };
    assert_eq!(tall.check_bytes(ImageRole::Screenshot, &png, &"a".repeat(64)).unwrap_err().to_string(), "screenshot: the PNG is 64x48, not the recorded 64x49");
}

#[test]
fn media_urls_stay_in_the_catalogue_or_the_package_release() {
    let repo = "https://github.com/alice/blur";
    for ok in ["media/a/icon.png", "https://github.com/Alice/blur/releases/download/v1/icon.PNG"] {
        assert!(media_url_ok(ok, repo), "{ok}");
    }
    for bad in ["../icon.png", "media/icon.gif", "https://github.com/bob/blur/releases/download/v1/icon.png", "file:///icon.png", "/abs/icon.png", "a//b.png"] {
        assert!(!media_url_ok(bad, repo), "{bad}");
    }
}

#[test]
fn a_later_catalogue_field_inside_an_image_does_not_break_this_app() {
    let image: Image = serde_json::from_value(serde_json::json!({
        "url": "media/x/icon.png", "sha256": "ab", "size": 10, "width": 64, "height": 64, "alt": "added in 1.2"
    }))
    .unwrap();
    assert_eq!((image.width, image.height), (64, 64));
}
