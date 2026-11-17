use craft_ext_core::contract::Provided;
pub fn provided(app: &str) -> Option<Provided> {
    let contracts: &[(&str, u32, u32)] = match app {
        "effectcraft" => &[("raster.filter", 1, 0), ("effectcraft.session", 1, 3), ("resource.preset", 1, 0), ("resource.lut", 1, 0), ("craft-theme", 1, 0)],
        "photocraft" => &[("raster.filter", 1, 0), ("photocraft.session", 1, 2), ("resource.preset", 1, 0), ("resource.lut", 1, 0), ("craft-theme", 1, 0)],
        "figcraft" => &[("figcraft.session", 1, 2), ("resource.font", 1, 0), ("craft-theme", 1, 0)],
        _ => return None,
    };
    Some(Provided::new(contracts))
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
