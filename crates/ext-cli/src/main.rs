use craft_ext_cli::Usage;
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let json = args.iter().any(|s| s == "--json");
    let mut out = std::io::stdout();
    let mut err = std::io::stderr();
    if args.first().is_some_and(|s| s == "plugin") {
        let mut rest = args[1..].to_vec();
        let result = (|| -> craft_ext_cli::Result<_> {
            let i = rest.iter().position(|s| s == "--app").ok_or_else(|| Usage("plugin requires --app <effectcraft|photocraft|figcraft>".into()))?;
            if i + 1 >= rest.len() {
                return Err(Usage("--app needs a value".into()).into());
            }
            let app = rest.remove(i + 1);
            rest.remove(i);
            craft_ext_cli::manager(&app)
        })();
        match result {
            Ok(m) => std::process::exit(craft_ext_cli::run_plugin(&m, &rest, &mut out, &mut err)),
            Err(e) => {
                let _ = craft_ext_cli::emit(&mut err, &serde_json::json!({"error":e.to_string()}), json);
                std::process::exit(if e.is::<Usage>() { 2 } else { 1 })
            }
        }
    }
    let result = if args.first().is_some_and(|s| s == "catalogue") {
        craft_ext_cli::catalogue::run(&args[1..])
    } else if args.is_empty() || args[0] == "--help" {
        Ok(serde_json::json!(
            "craft-ext plugin --app <id> … | new | build | check | test | pack | dev | catalogue <check|build|sign|keygen>; use <command> --help"
        ))
    } else {
        craft_ext_cli::author::run(&args)
    };
    match result {
        Ok(v) => {
            let _ = craft_ext_cli::emit(&mut out, &v, json);
        }
        Err(e) => {
            let _ = craft_ext_cli::emit(&mut err, &serde_json::json!({"error":e.to_string()}), json);
            std::process::exit(if e.is::<Usage>() { 2 } else { 1 })
        }
    }
}
