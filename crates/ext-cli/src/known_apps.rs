//! This table must mirror the contracts supplied by the application adapters.
use craft_ext_core::contract::Provided;
pub fn provided(app: &str) -> Option<Provided> {
    let names: &[&str] = match app {
        "effectcraft" => &["raster.filter", "effectcraft.session", "resource.preset", "resource.lut", "resource.font", "craft-theme"],
        "photocraft" => &["raster.filter", "photocraft.session", "resource.preset", "resource.lut", "craft-theme"],
        "figcraft" => &["raster.filter", "figcraft.session", "resource.font", "craft-theme"],
        _ => return None,
    };
    Some(Provided::new(&names.iter().map(|n| (*n, 1, 0)).collect::<Vec<_>>()))
}
