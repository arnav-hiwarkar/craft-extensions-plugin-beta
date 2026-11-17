use crate::catalogue::{Index, PackageSummary, Release};
use crate::text::{Lines, validate_text};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

pub const SCHEMA_MINOR: u32 = 1;
pub const MAX_SUMMARY_CHARS: usize = 140;
pub const MAX_DESCRIPTION_BYTES: usize = 8 * 1024;
pub const MAX_CHANGELOG_BYTES: usize = 4 * 1024;
pub const MAX_TAGS: usize = 10;
pub const MAX_TAG_CHARS: usize = 32;
pub const MAX_CATEGORIES: usize = 2;
pub const MAX_SCREENSHOTS: usize = 6;
pub const MAX_ICON_BYTES: u64 = 256 * 1024;
pub const MAX_SCREENSHOT_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_IMAGE_SIDE: u32 = 4096;
pub const MAX_FEATURED: usize = 24;
pub const MAX_URL_CHARS: usize = 2048;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Category {
    #[serde(rename = "effects-filters")]
    EffectsFilters,
    #[serde(rename = "generators")]
    Generators,
    #[serde(rename = "color-looks")]
    ColorLooks,
    #[serde(rename = "text-titles")]
    TextTitles,
    #[serde(rename = "motion-animation")]
    MotionAnimation,
    #[serde(rename = "3d-particles")]
    ThreeDParticles,
    #[serde(rename = "utilities")]
    Utilities,
    #[serde(rename = "panels-workflow")]
    PanelsWorkflow,
    #[serde(rename = "import-export")]
    ImportExport,
    #[serde(rename = "ai-analysis")]
    AiAnalysis,
    #[serde(rename = "resources")]
    Resources,
}

pub struct CategoryRow {
    pub category: Category,
    pub key: &'static str,
    pub english: &'static str,
    pub apps: &'static [&'static str],
}

const ALL_APPS: &[&str] = &["effectcraft", "photocraft", "figcraft"];

pub const CATEGORIES: [CategoryRow; 11] = [
    CategoryRow { category: Category::EffectsFilters, key: "effects-filters", english: "Effects & filters", apps: &["effectcraft", "photocraft"] },
    CategoryRow { category: Category::Generators, key: "generators", english: "Generators", apps: ALL_APPS },
    CategoryRow { category: Category::ColorLooks, key: "color-looks", english: "Color & looks", apps: ALL_APPS },
    CategoryRow { category: Category::TextTitles, key: "text-titles", english: "Text & titles", apps: ALL_APPS },
    CategoryRow { category: Category::MotionAnimation, key: "motion-animation", english: "Motion & animation", apps: &["effectcraft", "figcraft"] },
    CategoryRow { category: Category::ThreeDParticles, key: "3d-particles", english: "3D & particles", apps: &["effectcraft"] },
    CategoryRow { category: Category::Utilities, key: "utilities", english: "Utilities", apps: ALL_APPS },
    CategoryRow { category: Category::PanelsWorkflow, key: "panels-workflow", english: "Panels & workflow", apps: ALL_APPS },
    CategoryRow { category: Category::ImportExport, key: "import-export", english: "Import & export", apps: ALL_APPS },
    CategoryRow { category: Category::AiAnalysis, key: "ai-analysis", english: "AI & analysis", apps: ALL_APPS },
    CategoryRow { category: Category::Resources, key: "resources", english: "Resources", apps: ALL_APPS },
];

impl Category {
    pub fn row(self) -> &'static CategoryRow {
        &CATEGORIES[self as usize]
    }

    pub fn key(self) -> &'static str {
        self.row().key
    }

    pub fn from_key(key: &str) -> Option<Self> {
        CATEGORIES.iter().find(|r| r.key == key).map(|r| r.category)
    }

    pub fn applies_to(self, app: &str) -> bool {
        self.row().apps.contains(&app)
    }

    pub fn for_app(app: &str) -> impl Iterator<Item = Category> + '_ {
        CATEGORIES.iter().filter(move |r| r.apps.contains(&app)).map(|r| r.category)
    }
}

pub(crate) fn known_categories<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Category>, D::Error> {
    let keys = Vec::<String>::deserialize(d)?;
    Ok(keys.iter().filter_map(|k| Category::from_key(k)).collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub url: String,
    pub sha256: String,
    pub size: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageRole {
    Icon,
    Screenshot,
}

impl ImageRole {
    pub fn max_bytes(self) -> u64 {
        match self {
            ImageRole::Icon => MAX_ICON_BYTES,
            ImageRole::Screenshot => MAX_SCREENSHOT_BYTES,
        }
    }

    fn field(self) -> &'static str {
        match self {
            ImageRole::Icon => "icon",
            ImageRole::Screenshot => "screenshot",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attestation {
    pub verified: bool,
    pub signer_workflow: String,
    pub source_commit: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Downloads {
    pub count: u64,
    pub date: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListingError {
    pub field: String,
    pub problem: String,
}

impl fmt::Display for ListingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.problem)
    }
}

impl std::error::Error for ListingError {}

fn refuse(field: impl Into<String>, problem: impl Into<String>) -> ListingError {
    ListingError { field: field.into(), problem: problem.into() }
}

fn text(field: &str, value: &str, max_chars: usize, max_bytes: usize, lines: Lines) -> Result<(), ListingError> {
    if value.len() > max_bytes {
        return Err(refuse(field, format!("is {} bytes; at most {max_bytes} are allowed", value.len())));
    }
    validate_text(value, max_chars, lines).map_err(|e| refuse(field, e.to_string()))
}

fn plain_segment(s: &str) -> bool {
    !s.is_empty() && s != "." && s != ".." && s.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c))
}

fn github_slug(repo: &str) -> Option<(&str, &str)> {
    let slug = repo.strip_prefix("https://github.com/")?;
    let mut parts = slug.strip_suffix('/').unwrap_or(slug).split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    (parts.next().is_none() && plain_segment(owner) && plain_segment(name)).then_some((owner, name))
}

pub fn media_url_ok(url: &str, repo: &str) -> bool {
    if url.len() > MAX_URL_CHARS || !url.to_ascii_lowercase().ends_with(".png") {
        return false;
    }
    let Some(rest) = url.strip_prefix("https://github.com/") else {
        return !url.contains("://") && url.split('/').all(plain_segment);
    };
    let Some((owner, name)) = github_slug(repo) else { return false };
    let parts: Vec<&str> = rest.split('/').collect();
    parts.len() >= 6
        && parts[0].eq_ignore_ascii_case(owner)
        && parts[1].eq_ignore_ascii_case(name)
        && parts[2] == "releases"
        && parts[3] == "download"
        && parts[4..].iter().all(|s| plain_segment(s))
}

fn hex64(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10 && b[4] == b'-' && b[7] == b'-' && b.iter().enumerate().all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

pub fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
    if bytes.get(..8)? != SIGNATURE || bytes.get(12..16)? != b"IHDR" {
        return None;
    }
    let word = |at: usize| bytes.get(at..at + 4).and_then(|b| <[u8; 4]>::try_from(b).ok()).map(u32::from_be_bytes);
    Some((word(16)?, word(20)?))
}

impl Image {
    pub fn check(&self, role: ImageRole, repo: &str) -> Result<(), ListingError> {
        let field = role.field();
        if !media_url_ok(&self.url, repo) {
            return Err(refuse(field, "must be a .png path in the catalogue or a .png asset of the package's own GitHub release"));
        }
        if !hex64(&self.sha256) {
            return Err(refuse(field, "sha256 must be 64 lowercase hexadecimal characters"));
        }
        if self.size == 0 || self.size > role.max_bytes() {
            return Err(refuse(field, format!("is {} bytes; it must be 1 to {} bytes", self.size, role.max_bytes())));
        }
        if self.width == 0 || self.height == 0 || self.width > MAX_IMAGE_SIDE || self.height > MAX_IMAGE_SIDE {
            return Err(refuse(field, format!("is {}x{}; each side must be 1 to {MAX_IMAGE_SIDE} pixels", self.width, self.height)));
        }
        if role == ImageRole::Icon && self.width != self.height {
            return Err(refuse(field, format!("is {}x{}; an icon must be square", self.width, self.height)));
        }
        Ok(())
    }

    pub fn check_bytes(&self, role: ImageRole, bytes: &[u8], sha256: &str) -> Result<(), ListingError> {
        let field = role.field();
        if bytes.len() as u64 != self.size || sha256 != self.sha256 {
            return Err(refuse(field, "the file differs from its recorded size or sha256"));
        }
        match png_dimensions(bytes) {
            Some((w, h)) if (w, h) == (self.width, self.height) => Ok(()),
            Some((w, h)) => Err(refuse(field, format!("the PNG is {w}x{h}, not the recorded {}x{}", self.width, self.height))),
            None => Err(refuse(field, "the file is not a PNG")),
        }
    }
}

pub fn check_homepage(url: &str) -> Result<(), ListingError> {
    let rest = url.strip_prefix("https://").ok_or_else(|| refuse("homepage", "must start with https://"))?;
    let host = rest.split(['/', '?', '#']).next().unwrap_or_default();
    if url.len() > MAX_URL_CHARS || host.is_empty() || !host.contains('.') || url.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(refuse("homepage", "must be an https address with a host name and no spaces"));
    }
    Ok(())
}

pub fn check_tags(tags: &[String]) -> Result<(), ListingError> {
    if tags.len() > MAX_TAGS {
        return Err(refuse("tags", format!("{} given; at most {MAX_TAGS} are allowed", tags.len())));
    }
    for t in tags {
        text("tags", t, MAX_TAG_CHARS, MAX_TAG_CHARS * 4, Lines::One)?;
    }
    Ok(())
}

pub fn check_categories(categories: &[Category], apps: &[crate::id::AppId]) -> Result<(), ListingError> {
    if categories.len() > MAX_CATEGORIES {
        return Err(refuse("categories", format!("{} given; at most {MAX_CATEGORIES} are allowed", categories.len())));
    }
    let mut seen = std::collections::BTreeSet::new();
    for c in categories {
        if !seen.insert(*c) {
            return Err(refuse("categories", format!("{} is listed twice", c.key())));
        }
        if !apps.iter().any(|a| c.applies_to(a.as_str())) {
            return Err(refuse("categories", format!("{} does not apply to any app this plugin supports", c.key())));
        }
    }
    Ok(())
}

impl PackageSummary {
    pub fn check_listing(&self) -> Result<(), ListingError> {
        text("summary", &self.summary, MAX_SUMMARY_CHARS, MAX_SUMMARY_CHARS * 4, Lines::One)?;
        if !self.description.is_empty() {
            text("description", &self.description, MAX_DESCRIPTION_BYTES, MAX_DESCRIPTION_BYTES, Lines::Many)?;
        }
        check_tags(&self.tags)?;
        check_categories(&self.categories, &self.apps)?;
        if let Some(icon) = &self.icon {
            icon.check(ImageRole::Icon, &self.repo)?;
        }
        if self.screenshots.len() > MAX_SCREENSHOTS {
            return Err(refuse("screenshots", format!("{} given; at most {MAX_SCREENSHOTS} are allowed", self.screenshots.len())));
        }
        for s in &self.screenshots {
            s.check(ImageRole::Screenshot, &self.repo)?;
        }
        if let Some(h) = &self.homepage {
            check_homepage(h)?;
        }
        if self.license.trim().is_empty() {
            return Err(refuse("license", "is empty"));
        }
        if let Some(d) = &self.downloads {
            d.check()?;
        }
        Ok(())
    }
}

impl Downloads {
    pub fn check(&self) -> Result<(), ListingError> {
        if date(&self.date) { Ok(()) } else { Err(refuse("downloads", "date must be YYYY-MM-DD")) }
    }
}

impl Attestation {
    pub fn check(&self) -> Result<(), ListingError> {
        let commit_ok = self.source_commit.len() == 40 && self.source_commit.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
        if !commit_ok {
            return Err(refuse("attestation", "source_commit must be 40 lowercase hexadecimal characters"));
        }
        check_signer_workflow(&self.signer_workflow).map_err(|e| refuse("attestation", e.problem))
    }
}

pub fn check_signer_workflow(path: &str) -> Result<(), ListingError> {
    let file = path.strip_prefix(".github/workflows/").unwrap_or_default();
    let ok = (file.ends_with(".yml") || file.ends_with(".yaml")) && plain_segment(file);
    if ok { Ok(()) } else { Err(refuse("signer_workflow", "must be a workflow file such as .github/workflows/release.yml")) }
}

impl Release {
    pub fn check_listing(&self) -> Result<(), ListingError> {
        if !self.changelog.is_empty() {
            text("changelog", &self.changelog, MAX_CHANGELOG_BYTES, MAX_CHANGELOG_BYTES, Lines::Many)?;
        }
        if let Some(a) = &self.attestation {
            a.check()?;
            if a.source_commit != self.commit {
                return Err(refuse("attestation", "source_commit differs from the release commit"));
            }
        }
        if let Some(d) = &self.downloads {
            d.check()?;
        }
        Ok(())
    }
}

impl Index {
    pub fn check_listing(&self) -> Result<(), ListingError> {
        for p in &self.packages {
            p.check_listing().map_err(|e| refuse(format!("{} {}", p.id, e.field), e.problem))?;
        }
        if self.featured.len() > MAX_FEATURED {
            return Err(refuse("featured", format!("{} given; at most {MAX_FEATURED} are allowed", self.featured.len())));
        }
        let mut seen = std::collections::BTreeSet::new();
        for id in &self.featured {
            if !seen.insert(id) {
                return Err(refuse("featured", format!("{id} is listed twice")));
            }
            if self.package(id).is_none() {
                return Err(refuse("featured", format!("{id} is not a listed package")));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
