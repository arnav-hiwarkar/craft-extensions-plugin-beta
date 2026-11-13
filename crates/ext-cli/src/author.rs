//! Author tools. Package contents come only from validated manifest references.
use crate::{Args, Result, Usage};
use craft_ext_core::{
    catalogue::{Asset, Release},
    manifest::{ContributionKind, MANIFEST_NAME, Manifest},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeSet,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
pub fn manifest(dir: &Path) -> Result<Manifest> {
    Ok(Manifest::parse(&fs::read(dir.join(MANIFEST_NAME))?)?)
}
pub fn files(m: &Manifest) -> BTreeSet<String> {
    std::iter::once(MANIFEST_NAME.to_string())
        .chain(m.contributions.iter().flat_map(|c| c.module.iter().chain(&c.files)).cloned())
        .chain(m.native.iter().flat_map(|n| n.executables.values()).cloned())
        .collect()
}
pub fn date(seconds: u64) -> String {
    let z = (seconds / 86400) as i64 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    format!("{:04}-{month:02}-{d:02}", y + i64::from(month <= 2))
}
pub fn pack(dir: &Path, out: Option<&str>) -> Result<(Manifest, PathBuf, Vec<u8>)> {
    pack_described(dir, out, false)
}
pub fn pack_described(dir: &Path, out: Option<&str>, describe: bool) -> Result<(Manifest, PathBuf, Vec<u8>)> {
    let mut m = manifest(dir)?;
    if describe {
        for c in &mut m.contributions {
            if let Some(module) = &c.module {
                let path = dir.join(module);
                if fs::symlink_metadata(&path)?.file_type().is_symlink() || !path.canonicalize()?.starts_with(dir.canonicalize()?) {
                    return Err("module path escapes package".into());
                }
                let plugin = craft_ext_host::wasm::Plugin::load(m.id.as_str(), &fs::read(path)?)?;
                if let Some(form) = plugin.describe(std::sync::Arc::new(|_, _, _| {}), &|| Ok(()))? {
                    c.form = Some(form);
                }
            }
        }
        m.validate()?;
    }
    let target = out.map(PathBuf::from).unwrap_or_else(|| dir.join(format!("{}-{}.craftplugin", m.id, m.version)));
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    for f in files(&m) {
        let path = dir.join(&f);
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            return Err("package references a symbolic link".into());
        }
        let canonical = path.canonicalize()?;
        if !canonical.starts_with(dir.canonicalize()?) {
            return Err("package path escapes its directory".into());
        }
        let mut options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        if m.native.as_ref().is_some_and(|n| n.executables.values().any(|p| p == &f)) {
            options = options.unix_permissions(0o755);
        }
        let bytes = if describe && f == MANIFEST_NAME { m.to_toml().into_bytes() } else { fs::read(path)? };
        zip.start_file(f, options)?;
        zip.write_all(&bytes)?;
    }
    let bytes = zip.finish()?.into_inner();
    fs::write(&target, &bytes)?;
    Ok((m, target, bytes))
}
fn copy_template(src: &Path, dst: &Path, id: &str) -> Result<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let target = dst.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_template(&e.path(), &target, id)?
        } else if e.file_type()?.is_file() {
            let b = fs::read(e.path())?;
            match String::from_utf8(b) {
                Ok(s) => fs::write(target, s.replace("io.github.YOUR-NAME.PLUGIN", id))?,
                Err(e) => fs::write(target, e.into_bytes())?,
            }
        } else {
            return Err("templates cannot contain symbolic links".into());
        }
    }
    Ok(())
}
fn embedded_template(kind: &str, dst: &Path, id: &str) -> Result<()> {
    let files: &[(&str, &str)] = match kind {
        "effect" => &[
            ("Cargo.toml", include_str!("../../../templates/effect/Cargo.toml")),
            ("README.md", include_str!("../../../templates/effect/README.md")),
            ("craft-plugin.toml", include_str!("../../../templates/effect/craft-plugin.toml")),
            ("src/lib.rs", include_str!("../../../templates/effect/src/lib.rs")),
        ],
        "panel" => &[
            ("Cargo.toml", include_str!("../../../templates/panel/Cargo.toml")),
            ("README.md", include_str!("../../../templates/panel/README.md")),
            ("craft-plugin.toml", include_str!("../../../templates/panel/craft-plugin.toml")),
            ("src/lib.rs", include_str!("../../../templates/panel/src/lib.rs")),
        ],
        "data" => &[
            ("README.md", include_str!("../../../templates/data/README.md")),
            ("craft-plugin.toml", include_str!("../../../templates/data/craft-plugin.toml")),
            ("luts/warm.cube", include_str!("../../../templates/data/luts/warm.cube")),
            ("presets/looks.json", include_str!("../../../templates/data/presets/looks.json")),
        ],
        _ => return Err("unknown template".into()),
    };
    for (path, text) in files {
        let path = dst.join(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, text.replace("io.github.YOUR-NAME.PLUGIN", id))?;
    }
    Ok(())
}
pub fn run(args: &[String]) -> Result<Value> {
    let a = Args::parse(args, &["id", "out", "tag", "commit", "url", "app", "size"], &["store", "release-record", "describe"])?;
    if a.pos.is_empty() || a.has("help") {
        return Ok(json!(
            "new <effect|panel|data> <dir> --id <id> | build [dir] | check [dir] [--store] | test [dir] [--size WxH] | pack [dir] [--describe] [--out file] [--release-record --tag vX --commit SHA --url URL] | dev <dir> --app <app>. All accept --json."
        ));
    }
    let cmd = a.at(0)?;
    if cmd == "new" {
        a.arity(3)?;
        let kind = a.at(1)?;
        if !["effect", "panel", "data"].contains(&kind) {
            return Err(Usage("choose effect, panel or data".into()).into());
        }
        let id = a.opt("id").ok_or_else(|| Usage("--id is required".into()))?;
        craft_ext_core::id::ExtensionId::new(id)?;
        let dst = Path::new(a.at(2)?);
        if dst.exists() {
            return Err("destination already exists".into());
        }
        let root = std::env::var_os("CRAFT_TEMPLATES")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../templates"));
        if std::env::var_os("CRAFT_TEMPLATES").is_some() {
            copy_template(&root.join(kind), dst, id)?;
        } else {
            embedded_template(kind, dst, id)?;
        }
        let guest = std::env::var_os("CRAFT_GUEST_PATH")
            .map(PathBuf::from)
            .or_else(|| root.canonicalize().ok()?.ancestors().find(|p| p.join("crates/guest/Cargo.toml").is_file()).map(|p| p.join("crates/guest")));
        if let Some(guest) = guest {
            let guest = guest.canonicalize()?;
            let cargo = dst.join("Cargo.toml");
            if cargo.exists() {
                let mut doc: toml::Table = toml::from_str(&fs::read_to_string(&cargo)?)?;
                doc["dependencies"]["craft-guest"] =
                    toml::Value::Table(toml::Table::from_iter([("path".into(), toml::Value::String(guest.to_string_lossy().into_owned()))]));
                fs::write(cargo, toml::to_string_pretty(&doc)?)?;
            }
        }
        return Ok(json!("Plugin created"));
    }
    if a.pos.len() > 2 {
        return Err(Usage("too many arguments".into()).into());
    }
    let dir = Path::new(a.pos.get(1).map(String::as_str).unwrap_or("."));
    let m = manifest(dir)?;
    match cmd {
        "build" => {
            let output =
                std::process::Command::new("cargo").args(["build", "--release", "--target", "wasm32-unknown-unknown"]).current_dir(dir).output()?;
            if !output.status.success() {
                return Err(format!("Author build failed: {}", String::from_utf8_lossy(&output.stderr)).into());
            }
            let wasm = fs::read_dir(dir.join("target/wasm32-unknown-unknown/release"))?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().is_some_and(|e| e == "wasm"))
                .collect::<Vec<_>>();
            for c in &m.contributions {
                if let Some(module) = &c.module {
                    let stem = Path::new(module).file_stem();
                    let src = wasm
                        .iter()
                        .find(|p| p.file_stem() == stem)
                        .or_else(|| if wasm.len() == 1 { wasm.first() } else { None })
                        .ok_or("cannot identify wasm artifact for contribution")?;
                    let dst = dir.join(module);
                    if let Some(p) = dst.parent() {
                        fs::create_dir_all(p)?;
                    }
                    fs::copy(src, dst)?;
                }
            }
            Ok(json!("Modules built and copied"))
        }
        "check" => {
            let mut warnings = vec![];
            for f in files(&m) {
                if !dir.join(&f).is_file() {
                    return Err(format!("missing file {f}").into());
                }
            }
            for c in &m.contributions {
                if let Some(module) = &c.module {
                    check_wasm(&fs::read(dir.join(module))?, c.kind == ContributionKind::Effect)?;
                }
            }
            if let Err(term) = craft_ext_core::licence::check_osi(&m.license) {
                if a.has("store") {
                    return Err(format!("licence term {term} is not OSI-approved").into());
                }
                warnings.push(format!("Local use only: licence term {term} is not OSI-approved"));
            }
            for w in craft_ext_core::licence::monetisation_flags(&format!("{} {} {}", m.name, m.summary, m.description)) {
                warnings.push(format!("Review monetisation wording: {w}"));
            }
            warnings.push("Permission declarations need human review; module inspection cannot prove all permission use".into());
            Ok(json!({"checked":m.id,"warnings":warnings}))
        }
        "test" => {
            if !m.contributions.iter().any(|c| c.kind == ContributionKind::Effect) {
                return Ok(json!({"tested": [], "message": "No automated tests for panels/commands yet; run in the app with `craft-ext dev`"}));
            }
            let (w, h) = a.opt("size").unwrap_or("64x64").split_once('x').ok_or_else(|| Usage("size must be WxH".into()))?;
            let (w, h) = (w.parse::<u32>()?, h.parse::<u32>()?);
            if w == 0 || h == 0 || u64::from(w) * u64::from(h) > 16777216 {
                return Err(Usage("image size must be positive and at most 16777216 pixels".into()).into());
            }
            let input = (0..w * h).flat_map(|i| [(i % w) as f32 / w as f32, (i / w) as f32 / h as f32, 0.5, 1.0]).collect::<Vec<_>>();
            let mut results = vec![];
            for c in m.contributions.iter().filter(|c| c.kind == ContributionKind::Effect) {
                let module = c.module.as_ref().ok_or("effect has no module")?;
                let p = craft_ext_host::wasm::Plugin::load(&m.id.to_string(), &fs::read(dir.join(module))?)?;
                let mut filter = craft_ext_host::wasm::Filter::new(&p, std::sync::Arc::new(|_, _, _| {}))?;
                let params = c.form.as_ref().map(|f| f.defaults()).unwrap_or_default();
                let ctx = json!({"params":params,"time":0,"scale":1,"origin":[0,0],"full":[w,h],"quality":"final","seed":0,"color":"linear-srgb"});
                let start = std::time::Instant::now();
                let mut first = input.clone();
                let mut second = input.clone();
                filter.run(&ctx, &mut first, w, h, &|| Ok(()))?;
                filter.run(&ctx, &mut second, w, h, &|| Ok(()))?;
                if first.iter().zip(&second).any(|(a, b)| a.to_bits() != b.to_bits()) {
                    return Err(format!("{} is not deterministic", c.id).into());
                }
                if first.iter().any(|p| !p.is_finite()) || first.chunks_exact(4).any(|p| !(0.0..=1.0).contains(&p[3])) {
                    return Err("invalid output pixels".into());
                }
                results.push(json!({"contribution":c.id,"milliseconds":start.elapsed().as_secs_f64()*1000.0,"deterministic":true}));
            }
            Ok(json!(results))
        }
        "pack" | "dev" => {
            if a.has("release-record") {
                for k in ["tag", "commit", "url"] {
                    if a.opt(k).is_none() {
                        return Err(Usage(format!("--release-record requires --{k}")).into());
                    }
                }
            }
            let (m, path, bytes) = pack_described(dir, a.opt("out"), a.has("describe"))?;
            let hash = craft_ext_store::store::digest(&bytes);
            if a.has("release-record") {
                let r = Release {
                    version: m.version,
                    tag: a.opt("tag").unwrap_or_default().into(),
                    commit: a.opt("commit").unwrap_or_default().into(),
                    published: date(craft_ext_store::store::now()),
                    assets: vec![Asset {
                        platform: None,
                        url: a.opt("url").unwrap_or_default().into(),
                        sha256: hash.clone(),
                        size: bytes.len() as u64,
                    }],
                    manifest: m.clone(),
                };
                fs::write(dir.join(format!("{}.toml", m.version)), toml::to_string_pretty(&r)?)?;
            }
            let mut needs = vec![];
            if cmd == "dev" {
                let mut manager = crate::manager(a.opt("app").ok_or_else(|| Usage("--app is required".into()))?)?;
                manager.store = craft_ext_store::Store::at(manager.store.root().join("development"));
                manager.store.install_bytes(&bytes, craft_ext_core::tier::Tier::Dev, craft_ext_store::state::Origin::Dev)?;
                let app = manager.app.as_ref().ok_or("choose an app")?;
                manager.store.set_active(app, &m.id, m.version)?;
                for c in m.contributions_for(app) {
                    if c.permissions.is_empty() && !m.is_native() {
                        manager.enable(&m.id, &c.id)?
                    } else {
                        for p in &c.permissions {
                            needs.push(format!("plugin grant {} {}", m.id, p));
                        }
                        needs.push(format!("plugin setup {} --enable {}{}", m.id, c.id, if m.is_native() { " --full-access" } else { "" }));
                    }
                }
            }
            Ok(
                json!({"file":path,"sha256":hash,"size":bytes.len(),"needs_grants":needs,"development_store":if cmd=="dev"{Some(crate::manager(a.opt("app").unwrap_or_default())?.store.root().join("development"))}else{None}}),
            )
        }
        _ => Err(Usage(format!("unknown author command {cmd}")).into()),
    }
}

fn check_wasm(bytes: &[u8], effect: bool) -> Result<()> {
    use ValType::{I32, I64};
    use wasmi::{Engine, Module, ValType};
    let engine = Engine::default();
    let module = Module::new(&engine, bytes)?;
    let signature =
        |ty: &wasmi::ExternType, params: &[ValType], results: &[ValType]| ty.func().is_some_and(|f| f.params() == params && f.results() == results);
    for (name, params, results) in [
        ("craft_abi", vec![], vec![I32]),
        ("craft_alloc", vec![I32], vec![I32]),
        ("craft_reset", vec![], vec![]),
        ("craft_filter", vec![I32; 5], vec![I32]),
        ("craft_event", vec![I32; 2], vec![I32]),
        ("craft_describe", vec![], vec![I32]),
        ("craft_update", vec![I32; 2], vec![I32]),
    ] {
        let export = match module.exports().find(|e| e.name() == name) {
            Some(e) => e,
            None if name == "craft_describe"
                || name == "craft_update"
                || (name == "craft_filter" && !effect)
                || (name == "craft_event" && effect) =>
            {
                continue;
            }
            None => return Err(format!("missing export {name}").into()),
        };
        if !signature(export.ty(), &params, &results) {
            return Err(format!("invalid signature for {name}").into());
        }
    }
    for import in module.imports() {
        let (params, results) = match (import.module(), import.name()) {
            ("craft", "log") => (vec![I32; 3], vec![]),
            ("craft", "call") => (vec![I32; 4], vec![I64]),
            ("craft", "result") | ("craft", "take") => (vec![I32; 2], vec![I32]),
            _ => return Err("unsupported Wasm import".into()),
        };
        if !signature(import.ty(), &params, &results) {
            return Err(format!("invalid signature for import {}", import.name()).into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod beta_tests {
    use super::*;
    #[test]
    fn pack_describe_embeds_form_without_changing_source() {
        let t = tempfile::tempdir().unwrap();
        embedded_template("effect", t.path(), "io.github.alice.test").unwrap();
        let source = fs::read(t.path().join(MANIFEST_NAME)).unwrap();
        let form = r#"{"schema":1,"key":"described","title":"Described","params":[]}"#;
        let escaped = form.bytes().map(|b| format!("\\{b:02x}")).collect::<String>();
        let wat = format!(
            r#"(module
            (import "craft" "result" (func $result (param i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "craft_abi") (result i32) (i32.const 1))
            (func (export "craft_alloc") (param i32) (result i32) (i32.const 4096))
            (func (export "craft_reset"))
            (data (i32.const 0) "{escaped}")
            (func (export "craft_describe") (result i32) (call $result (i32.const 0) (i32.const {}))))"#,
            form.len()
        );
        fs::write(t.path().join("plugin.wasm"), wat::parse_str(wat).unwrap()).unwrap();
        let (plain, _, _) = pack(t.path(), Some(t.path().join("plain.craftplugin").to_str().unwrap())).unwrap();
        assert_eq!(plain.contributions[0].form.as_ref().unwrap().key, "effect");
        let (m, _, bytes) = pack_described(t.path(), None, true).unwrap();
        assert_eq!(m.contributions[0].form.as_ref().unwrap().key, "described");
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut embedded = Vec::new();
        std::io::Read::read_to_end(&mut zip.by_name(MANIFEST_NAME).unwrap(), &mut embedded).unwrap();
        assert_eq!(manifest(t.path()).unwrap().contributions[0].form.as_ref().unwrap().key, "effect");
        assert_eq!(Manifest::parse(&embedded).unwrap(), m);
        assert_eq!(fs::read(t.path().join(MANIFEST_NAME)).unwrap(), source);
    }
    #[test]
    fn embedded_templates_and_non_effect_test_message() {
        let t = tempfile::tempdir().unwrap();
        for kind in ["data", "panel", "effect"] {
            let dst = t.path().join(kind);
            embedded_template(kind, &dst, "io.github.alice.test").unwrap();
            assert_eq!(manifest(&dst).unwrap().id.as_str(), "io.github.alice.test");
        }
        let value = run(&["test".into(), t.path().join("panel").to_str().unwrap().into()]).unwrap();
        assert_eq!(value["tested"], json!([]));
        assert_eq!(value["message"], "No automated tests for panels/commands yet; run in the app with `craft-ext dev`");
    }
}
