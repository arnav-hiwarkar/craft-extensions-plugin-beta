//! This table must mirror the contracts supplied by the application adapters.
use craft_ext_core::contract::Provided;
/// Mirrors the `Contracts provided` tables in docs/plugins/contracts/*.md.
pub fn provided(app: &str) -> Option<Provided> {
    let names: &[&str] = match app {
        "effectcraft" => &["raster.filter", "effectcraft.session", "resource.preset", "resource.lut", "craft-theme"],
        "photocraft" => &["raster.filter", "photocraft.session", "resource.preset", "resource.lut", "craft-theme"],
        "figcraft" => &["figcraft.session", "resource.font", "craft-theme"],
        _ => return None,
    };
    Some(Provided::new(&names.iter().map(|n| (*n, 1, if matches!(*n, "effectcraft.session" | "figcraft.session" | "photocraft.session") { 1 } else { 0 })).collect::<Vec<_>>()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matches_contract_pages() {
        for app in ["effectcraft", "photocraft", "figcraft"] {
            let page = std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../docs/plugins/contracts/{app}.md"))).unwrap();
            let table = page.split("## Contracts provided").nth(1).unwrap().split("\n## ").next().unwrap();
            let actual = table.lines().filter_map(|line| {
                let cols: Vec<_> = line.split('|').map(str::trim).collect();
                if cols.len() == 4 && cols[1].starts_with('`') {
                    Some((cols[1].trim_matches('`').to_string(), cols[2].to_string()))
                } else { None }
            }).collect::<std::collections::BTreeSet<_>>();
            let expected = provided(app).unwrap().contracts.iter().map(|(n,l)| (n.to_string(),l.to_string())).collect();
            assert_eq!(actual, expected, "{app}");
        }
    }
}
