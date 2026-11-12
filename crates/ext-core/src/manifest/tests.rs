use super::*;
use crate::contract::Provided;

pub(crate) const SAMPLE: &str = r#"
schema = 2
id = "io.github.ink.grain"
name = "Ink & Grain"
version = "1.2.0"
license = "MIT OR Apache-2.0"
source = "https://github.com/ink/grain"
summary = "Halftone and grain"

apps = ["effectcraft", "photocraft"]

[why]
"network:api.example.com" = "Downloads screens"

[[contribution]]
id = "halftone"
kind = "effect"
contract = "raster.filter@1.0"
optional_permissions = ["network:api.example.com"]
module = "halftone.wasm"
label = "Halftone"
category = "Stylize"
[[contribution.param]]
id = "size"
label = "Dot size"
type = "slider"
default = 8
min = 1
max = 64
"#;

fn ec() -> AppId {
    AppId::new("effectcraft").unwrap()
}

#[test]
fn parses_sample() {
    let m = Manifest::parse(SAMPLE.as_bytes()).unwrap();
    assert_eq!(m.contributions[0].params.len(), 1);
    assert!(m.has_code() && !m.is_native());
    let again = Manifest::parse(m.to_toml().as_bytes()).unwrap();
    assert_eq!(again, m);
}

#[test]
fn refuses_unknown_fields_bad_schema_and_permissions() {
    assert!(Manifest::parse(SAMPLE.replace("summary =", "tier = \"official\"\nsummary =").as_bytes()).is_err());
    assert!(Manifest::parse(SAMPLE.replace("schema = 2", "schema = 3").as_bytes()).unwrap_err().0.contains("format 3"));
    assert!(Manifest::parse(SAMPLE.replace("network:api.example.com", "process").as_bytes()).is_err());
    assert!(Manifest::parse(SAMPLE.replace("default = 8", "default = 99").as_bytes()).is_err());
    assert!(Manifest::parse(SAMPLE.replace("module = \"halftone.wasm\"", "module = \"../x.wasm\"").as_bytes()).is_err());
}

#[test]
fn compat_by_contract() {
    let m = Manifest::parse(SAMPLE.as_bytes()).unwrap();
    assert_eq!(m.compat(&ec(), &Provided::new(&[("raster.filter", 1, 0)])), Compat::Compatible);
    assert!(!m.compat(&ec(), &Provided::new(&[("raster.filter", 2, 0)])).usable());
    let (req, opt) = m.permissions_for(&ec(), |_| true);
    assert!(req.is_empty() && opt.len() == 1);
    assert!(!m.compat(&AppId::new("figcraft").unwrap(), &Provided::new(&[("raster.filter", 1, 0)])).usable());
}
