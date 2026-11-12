//! Embeddable plugin management and the standalone author/maintainer CLI.
use craft_ext_core::{
    id::ExtensionId,
    manifest::Manifest,
    permission::Permission,
    version::{Version, VersionReq},
};
use craft_ext_store::Manager;
use serde_json::{Value, json};
use std::{io::Write, path::Path};
#[cfg(feature = "tools")]
pub mod author;
#[cfg(feature = "tools")]
pub mod catalogue;
pub mod known_apps;
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Debug)]
pub struct Usage(pub String);
impl std::fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for Usage {}
#[derive(Default, Debug)]
pub struct Args {
    pub pos: Vec<String>,
    pub opts: std::collections::BTreeMap<String, Vec<String>>,
}
impl Args {
    pub fn parse(args: &[String], values: &[&str], flags: &[&str]) -> Result<Self> {
        let mut a = Self::default();
        let mut it = args.iter();
        while let Some(s) = it.next() {
            if let Some(k) = s.strip_prefix("--") {
                if values.contains(&k) {
                    let v = it.next().filter(|v| !v.starts_with("--")).ok_or_else(|| Usage(format!("--{k} needs a value")))?;
                    a.opts.entry(k.into()).or_default().push(v.clone());
                } else if flags.contains(&k) || ["json", "help"].contains(&k) {
                    a.opts.entry(k.into()).or_default();
                } else {
                    return Err(Usage(format!("unknown option --{k}")).into());
                }
            } else {
                a.pos.push(s.clone());
            }
        }
        Ok(a)
    }
    pub fn has(&self, k: &str) -> bool {
        self.opts.contains_key(k)
    }
    pub fn opt(&self, k: &str) -> Option<&str> {
        self.opts.get(k)?.last().map(String::as_str)
    }
    pub fn at(&self, n: usize) -> Result<&str> {
        self.pos.get(n).map(String::as_str).ok_or_else(|| Usage("missing argument; use --help".into()).into())
    }
    pub fn arity(&self, n: usize) -> Result<()> {
        if self.pos.len() == n { Ok(()) } else { Err(Usage("wrong number of arguments; use --help".into()).into()) }
    }
}
const HELP: &str = "plugin search <q> [--kind k]\nplugin info|versions|permissions|report <id>\nplugin list|updates|refresh\nplugin install <id> [--version req]\nplugin install-file <path>\nplugin setup <id> [--enable c1,c2|--all] [--grant permission] [--full-access]\nplugin enable|disable <id> <contribution>\nplugin grant|revoke <id> <permission>\nplugin update <id>|--all\nplugin rollback <id> <version>\nplugin pin <id> <req>\nplugin unpin|remove <id>\nplugin project <file.json>\nAll commands accept --json and --help. Install never enables; setup grants only named permissions.";
pub fn emit(out: &mut dyn Write, v: &Value, json: bool) -> std::io::Result<()> {
    if json {
        writeln!(out, "{}", serde_json::to_string(v)?)
    } else if let Some(s) = v.as_str() {
        writeln!(out, "{s}")
    } else {
        writeln!(out, "{}", serde_json::to_string_pretty(v)?)
    }
}
pub fn run_plugin(manager: &Manager, args: &[String], out: &mut dyn Write, err: &mut dyn Write) -> i32 {
    let json = args.iter().any(|s| s == "--json");
    match plugin(manager, args) {
        Ok(v) => {
            if emit(out, &v, json).is_ok() {
                0
            } else {
                1
            }
        }
        Err(e) => {
            let code = if e.is::<Usage>() {
                2
            } else if e.to_string().contains("Fix: plugin") {
                4
            } else {
                1
            };
            let _ = emit(err, &json!({"error":e.to_string(),"code":code}), json);
            code
        }
    }
}
fn installed(m: &Manager, id: &ExtensionId) -> Result<Manifest> {
    let s = m.store.state()?;
    let p = s.packages.get(id).ok_or_else(|| format!("Plugin is missing. Fix: plugin install {id}"))?;
    let v = m
        .app
        .as_ref()
        .and_then(|a| s.apps.get(a))
        .and_then(|a| a.get(id))
        .and_then(|a| a.pending.or(a.active))
        .or_else(|| p.releases.keys().next_back().copied())
        .ok_or("no installed releases")?;
    Ok(m.store.manifest(&s, id, v)?)
}
fn enable(m: &Manager, id: &ExtensionId, c: &str) -> Result<()> {
    let manifest = installed(m, id)?;
    let contribution = manifest.contribution(c).ok_or_else(|| Usage(format!("unknown contribution {c}")))?;
    if contribution.contract.check(&m.provided).is_some()
        || !manifest.apps.contains(m.app.as_ref().ok_or("choose an app")?)
        || (!contribution.apps.is_empty() && !contribution.apps.contains(m.app.as_ref().ok_or("choose an app")?))
    {
        return Err(format!("Contribution is unavailable in this app. Fix: plugin versions {id}").into());
    }
    let s = m.store.state()?;
    let setup = m.app.as_ref().and_then(|a| s.apps.get(a)).and_then(|a| a.get(id));
    let missing: Vec<_> = contribution.permissions.iter().filter(|p| !setup.is_some_and(|s| s.grants.iter().any(|g| g.covers(p)))).collect();
    if !missing.is_empty() {
        return Err(missing.iter().map(|p| format!("Permission needed. Fix: plugin grant {id} {p}")).collect::<Vec<_>>().join("\n").into());
    }
    if manifest.is_native() && !setup.is_some_and(|s| s.full_access) {
        return Err(format!("Native consent needed. Fix: plugin setup {id} --full-access").into());
    }
    m.enable(id, c)?;
    Ok(())
}
fn sheet(manifest: &Manifest) -> Value {
    json!({"required":manifest.contributions.iter().flat_map(|c|&c.permissions).map(|p|json!({"permission":p,"sentence":p.sentence(),"why":manifest.why.get(&p.to_string())})).collect::<Vec<_>>(),"optional":manifest.contributions.iter().flat_map(|c|&c.optional_permissions).map(|p|json!({"permission":p,"sentence":p.sentence(),"why":manifest.why.get(&p.to_string())})).collect::<Vec<_>>(),"warnings":craft_ext_core::permission::warnings(&manifest.contributions.iter().flat_map(|c|c.permissions.iter().chain(&c.optional_permissions)).cloned().collect::<Vec<_>>()),"native":if manifest.is_native(){Some("Native: full access to your computer")}else{None}})
}
fn plugin(m: &Manager, args: &[String]) -> Result<Value> {
    let a = Args::parse(args, &["kind", "version", "enable", "grant"], &["all", "full-access"])?;
    if a.pos.is_empty() || a.has("help") || a.pos[0] == "help" {
        return Ok(json!(HELP));
    }
    let cmd = a.at(0)?;
    let allowed: &[&str] = match cmd {
        "search" => &["kind"],
        "install" => &["version"],
        "setup" => &["enable", "all", "grant", "full-access"],
        "update" => &["all"],
        _ => &[],
    };
    for key in a.opts.keys() {
        if !["json", "help"].contains(&key.as_str()) && !allowed.contains(&key.as_str()) {
            return Err(Usage(format!("--{key} is not valid for plugin {cmd}")).into());
        }
    }

    match cmd {
        "list" => {
            a.arity(1)?;
            return Ok(serde_json::to_value(m.store.state()?)?);
        }
        "refresh" => {
            a.arity(1)?;
            m.refresh(craft_ext_store::store::now())?;
            return Ok(json!("Catalogue refreshed"));
        }
        "updates" => {
            a.arity(1)?;
            return Ok(json!(m.update_check()?));
        }
        "search" => {
            a.arity(2)?;
            let mut rows = m.search(a.at(1)?)?;
            if let Some(k) = a.opt("kind") {
                let kind = serde_json::from_value(json!(k)).map_err(|_| Usage("unknown kind".into()))?;
                rows.retain(|r| r.kinds.contains(&kind));
            }
            return Ok(json!(rows));
        }
        "install-file" => {
            a.arity(2)?;
            let (i, c, _) = m.install_file(Path::new(a.at(1)?))?;
            return Ok(json!({"id":i.id,"compatibility":c,"permissions":sheet(&i.manifest)}));
        }
        "project" => {
            a.arity(2)?;
            let refs = serde_json::from_slice::<Vec<craft_ext_core::project::PluginRef>>(&std::fs::read(a.at(1)?)?)?;
            let rows = m.check_project(&refs)?;
            return Ok(json!(rows.into_iter().map(|(r,s)|json!({"status":s,"install":format!("plugin install {} --version ={}",r.id,r.version),"setup":format!("plugin setup {} --all",r.id)})).collect::<Vec<_>>()));
        }
        "update" if a.has("all") => {
            a.arity(1)?;
            let mut rows = vec![];
            let (updates, problems) = m.update_check()?;
            rows.extend(problems.into_iter().map(|(id, reason)| json!({"id":id,"error":reason})));
            for (id, _, _, _) in updates {
                let (i, _, _) = m.update(&id)?;
                rows.push(json!({"id":id,"permissions":sheet(&i.manifest),"pending":m.store.state()?.apps.get(m.app.as_ref().ok_or("choose an app")?).and_then(|s|s.get(&id)).and_then(|s|s.pending)}));
            }
            return Ok(json!(rows));
        }
        _ => {}
    }
    if ![
        "info",
        "versions",
        "install",
        "update",
        "permissions",
        "setup",
        "enable",
        "disable",
        "grant",
        "revoke",
        "rollback",
        "pin",
        "unpin",
        "remove",
        "report",
    ]
    .contains(&cmd)
    {
        return Err(Usage(format!("unknown plugin command {cmd}\n{HELP}")).into());
    }
    let id = ExtensionId::new(a.at(1)?).map_err(|e| Usage(e.to_string()))?;
    match cmd {
        "info" => {
            a.arity(2)?;
            Ok(json!(m.info(&id)?))
        }
        "versions" => {
            a.arity(2)?;
            Ok(json!(m.versions(&id)?))
        }
        "install" | "update" => {
            a.arity(2)?;
            let (i, c, _) = if cmd == "update" {
                m.update(&id)?
            } else {
                m.install(&id, a.opt("version").map(VersionReq::parse).transpose().map_err(|e| Usage(e.to_string()))?)?
            };
            Ok(
                json!({"id":id,"compatibility":c,"permissions":sheet(&i.manifest),"message":"New permissions remain pending until explicitly granted and setup accepted"}),
            )
        }
        "permissions" => {
            a.arity(2)?;
            Ok(sheet(&installed(m, &id)?))
        }
        "setup" => {
            a.arity(2)?;
            if a.has("all") && a.has("enable") {
                return Err(Usage("choose --all or --enable".into()).into());
            }
            let mf = installed(m, &id)?;
            let app = m.app.as_ref().ok_or("choose an app")?;
            m.store.set_active(app, &id, mf.version)?;
            for p in a.opts.get("grant").into_iter().flatten() {
                m.grant(&id, Permission::parse(p).map_err(|e| Usage(e.to_string()))?)?;
            }
            if a.has("full-access") {
                m.store.set_full_access(app, &id, true)?;
            }
            let selected: Vec<String> = if a.has("all") {
                mf.contributions_for(app).map(|c| c.id.clone()).collect()
            } else {
                a.opt("enable").map(|s| s.split(',').map(String::from).collect()).unwrap_or_default()
            };
            let state = m.store.state()?;
            let setup = state.apps.get(app).and_then(|s| s.get(&id)).ok_or("setup missing")?;
            let required = mf.permissions_for(app, |c| selected.iter().any(|s| s == c) || setup.enabled.contains(c)).0;
            let missing = required
                .iter()
                .filter(|p| !setup.grants.iter().any(|g| g.covers(p)))
                .map(|p| format!("Permission needed. Fix: plugin grant {id} {p}"))
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                return Err(missing.join("\n").into());
            }
            if mf.is_native() && !setup.full_access {
                return Err(format!("Native consent needed. Fix: plugin setup {id} --full-access").into());
            }
            if setup.pending.is_some() {
                m.store.accept_pending(app, &id)?;
            }
            for c in selected {
                enable(m, &id, &c)?;
            }
            if m.store.state()?.apps.get(app).and_then(|s| s.get(&id)).is_some_and(|s| s.pending.is_some()) {
                m.store.accept_pending(app, &id)?;
            }
            Ok(json!("Plugin setup saved"))
        }
        "enable" | "disable" => {
            a.arity(3)?;
            if cmd == "enable" {
                enable(m, &id, a.at(2)?)?
            } else {
                m.disable(&id, a.at(2)?)?
            }
            Ok(json!("Contribution changed"))
        }
        "grant" | "revoke" => {
            a.arity(3)?;
            let _ = installed(m, &id)?;
            let p = Permission::parse(a.at(2)?).map_err(|e| Usage(e.to_string()))?;
            if cmd == "grant" {
                m.grant(&id, p)?
            } else {
                m.revoke(&id, &p)?
            }
            Ok(json!("Permission changed"))
        }
        "rollback" => {
            a.arity(3)?;
            Ok(json!(m.rollback(&id, Version::parse(a.at(2)?).map_err(|e| Usage(e.to_string()))?)?))
        }
        "pin" => {
            a.arity(3)?;
            m.pin(&id, VersionReq::parse(a.at(2)?).map_err(|e| Usage(e.to_string()))?)?;
            Ok(json!("Version pinned"))
        }
        "unpin" => {
            a.arity(2)?;
            m.unpin(&id)?;
            Ok(json!("Version unpinned"))
        }
        "remove" => {
            a.arity(2)?;
            m.remove(&id)?;
            Ok(json!("Plugin removed"))
        }
        "report" => {
            a.arity(2)?;
            Ok(json!(m.report_url(&id, installed(m, &id)?.version)))
        }
        _ => Err(Usage(format!("unknown plugin command {cmd}\n{HELP}")).into()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn args() {
        assert!(Args::parse(&["--version".into()], &["version"], &[]).is_err());
        assert!(Args::parse(&["--typo".into()], &[], &[]).is_err());
        let a = Args::parse(&["setup".into(), "--grant".into(), "clipboard.read".into(), "--json".into()], &["grant"], &[]).unwrap();
        assert!(a.has("json"));
        assert_eq!(a.at(0).unwrap(), "setup");
    }
}
pub fn manager(app: &str) -> Result<Manager> {
    let provided = known_apps::provided(app).ok_or_else(|| Usage(format!("unknown app {app}; choose effectcraft, photocraft or figcraft")))?;
    Ok(Manager::new(
        craft_ext_store::Store::at(craft_ext_store::location::default_root()?),
        Some(craft_ext_core::id::AppId::new(app)?),
        provided,
        format!("{}-{}", if cfg!(target_os = "macos") { "macos" } else { std::env::consts::OS }, std::env::consts::ARCH),
    ))
}
#[cfg(test)]
mod consent_tests {
    use super::*;
    #[test]
    fn setup_never_grants_required_permissions() {
        let d = tempfile::tempdir().unwrap();
        let source = d.path().join("source");
        std::fs::create_dir(&source).unwrap();
        std::fs::write(
            source.join("craft-plugin.toml"),
            r#"schema=2
id="io.github.alice.test"
name="Test"
version="1.0.0"
license="MIT"
source="https://github.com/alice/test"
summary="Test"
apps=["effectcraft"]
[[contribution]]
id="data"
kind="preset"
contract="resource.preset@1.0"
label="Data"
files=["data.json"]
permissions=["clipboard.read"]
"#,
        )
        .unwrap();
        std::fs::write(source.join("data.json"), "{}").unwrap();
        let (_, archive, _) = author::pack(&source, None).unwrap();
        let app = craft_ext_core::id::AppId::new("effectcraft").unwrap();
        let m = Manager::new(
            craft_ext_store::Store::at(d.path().join("store")),
            Some(app.clone()),
            known_apps::provided("effectcraft").unwrap(),
            "linux-x86_64".into(),
        );
        m.install_file(&archive).unwrap();
        let mut out = vec![];
        let mut err = vec![];
        let args = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        assert_eq!(run_plugin(&m, &args("setup io.github.alice.test --all"), &mut out, &mut err), 4);
        assert!(String::from_utf8(err).unwrap().contains("plugin grant io.github.alice.test clipboard.read"));
        let id = ExtensionId::new("io.github.alice.test").unwrap();
        let state = m.store.state().unwrap();
        assert!(state.apps[&app][&id].grants.is_empty());
        assert!(state.apps[&app][&id].enabled.is_empty());
        assert_eq!(run_plugin(&m, &args("setup io.github.alice.test --all --grant clipboard.read"), &mut out, &mut vec![]), 0);
        assert!(m.store.state().unwrap().apps[&app][&id].enabled.contains("data"));
    }
}
