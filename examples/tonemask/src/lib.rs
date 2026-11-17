use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use craft_guest::kernel::{self, Buffer, Curves};
use craft_guest::ui::NodeExt;
use craft_guest::{Event, call, json, session, ui};

const TILE_BYTES: usize = 4 * 1024 * 1024;
const MASK_BYTES: usize = 700 * 1024;
const LUMA: [f32; 20] = [0.2126, 0.7152, 0.0722, 0.0, 0.0, 0.2126, 0.7152, 0.0722, 0.0, 0.0, 0.2126, 0.7152, 0.0722, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Lights(u32),
    Darks(u32),
    Midtones(u32),
}

impl Tone {
    pub const ALL: [Tone; 10] = [
        Tone::Lights(1),
        Tone::Lights(2),
        Tone::Lights(3),
        Tone::Lights(4),
        Tone::Midtones(1),
        Tone::Midtones(2),
        Tone::Darks(1),
        Tone::Darks(2),
        Tone::Darks(3),
        Tone::Darks(4),
    ];

    pub fn id(self) -> String {
        match self {
            Tone::Lights(n) => format!("lights-{n}"),
            Tone::Darks(n) => format!("darks-{n}"),
            Tone::Midtones(n) => format!("mids-{n}"),
        }
    }

    pub fn label(self) -> String {
        match self {
            Tone::Lights(n) => format!("Lights {n}"),
            Tone::Darks(n) => format!("Darks {n}"),
            Tone::Midtones(n) => format!("Midtones {n}"),
        }
    }

    pub fn weight(self, luminance: f32) -> f32 {
        let l = luminance.clamp(0.0, 1.0);
        match self {
            Tone::Lights(n) => l.powi(n as i32),
            Tone::Darks(n) => (1.0 - l).powi(n as i32),
            Tone::Midtones(n) => (1.0 - l.powi(n as i32 + 1) - (1.0 - l).powi(n as i32 + 1)).clamp(0.0, 1.0),
        }
    }

    pub fn curve(self) -> Vec<f32> {
        (0..256).map(|i| self.weight(i as f32 / 255.0)).collect()
    }

    fn parse(id: &str) -> Option<Tone> {
        Tone::ALL.into_iter().find(|t| t.id() == id)
    }
}

pub fn mask_from_rgba8(rgba: &[u8], width: u32, height: u32, tone: Tone) -> Result<Vec<u8>, String> {
    let pixels: Vec<f32> = rgba.chunks_exact(4).flat_map(|p| [f32::from(p[0]) / 255.0, f32::from(p[1]) / 255.0, f32::from(p[2]) / 255.0, 1.0]).collect();
    let mut buf = Buffer { width, height, pixels };
    kernel::color_matrix(&mut buf.image(), LUMA).map_err(|e| e.to_string())?;
    kernel::curves(&mut buf.image(), &Curves { rgb: Some(tone.curve()), ..Curves::default() }).map_err(|e| e.to_string())?;
    Ok(buf.pixels.chunks_exact(4).map(|p| (p[0].clamp(0.0, 1.0) * 255.0).round() as u8).collect())
}

fn select(tone: Tone) -> Result<(), String> {
    let info = call("document.info", json!({})).map_err(|e| e.to_string())?;
    let (w, h) = (info["width"].as_u64().unwrap_or(0) as u32, info["height"].as_u64().unwrap_or(0) as u32);
    if w == 0 || h == 0 || info["mode"] != "rgb" {
        return Err("Tonemask works on RGB images".into());
    }
    let rows = ((TILE_BYTES / (w as usize * 4)).min(MASK_BYTES / w as usize)).max(1) as u32;
    let mut y = 0;
    while y < h {
        let n = rows.min(h - y);
        let tile = call("pixels.get", json!({"composite": true, "rect": [0, y, w, n], "format": "rgba8"})).map_err(|e| e.to_string())?;
        let rgba = STANDARD.decode(tile["data"].as_str().unwrap_or_default()).map_err(|e| e.to_string())?;
        if rgba.len() != (w * n * 4) as usize {
            return Err("the app returned a tile of the wrong size".into());
        }
        let mask = mask_from_rgba8(&rgba, w, n, tone)?;
        let mode = if y == 0 { "replace" } else { "add" };
        call("selection.set", json!({"mask": {"x": 0, "y": y, "width": w, "height": n, "data": STANDARD.encode(mask)}, "mode": mode})).map_err(|e| e.to_string())?;
        y += n;
    }
    Ok(())
}

fn panel(note: &str) -> Result<(), String> {
    let buttons = |filter: fn(&Tone) -> bool| ui::row(Tone::ALL.into_iter().filter(filter).map(|t| ui::button(&t.id(), &t.label())).collect());
    let tree = ui::column(vec![
        ui::text("Select by tone", ui::TextStyle::Strong),
        buttons(|t| matches!(t, Tone::Lights(_))),
        buttons(|t| matches!(t, Tone::Midtones(_))),
        buttons(|t| matches!(t, Tone::Darks(_))),
        ui::separator(),
        ui::row(vec![ui::button("save", "Save as channel"), ui::button("mask", "Add as layer mask")]),
        ui::label(note).style(ui::TextStyle::Small),
    ]);
    ui::set_panel("tonemask", tree).map(drop).map_err(|e| e.to_string())
}

fn handle(event: Event) -> Result<(), String> {
    match event {
        Event::Init => panel("Pick a tone range. Each press replaces the selection and is one undo step."),
        Event::Ui { widget, .. } => {
            if let Some(tone) = Tone::parse(&widget) {
                select(tone)?;
                return panel(&format!("{} selected.", tone.label()));
            }
            match widget.as_str() {
                "save" => call("selection.save", json!({"name": "Tonemask"})).map(drop).map_err(|e| e.to_string())?,
                "mask" => call("mask.add", json!({"reveal": "selection"})).map(drop).map_err(|e| e.to_string())?,
                _ => {}
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

session!(handle);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_weights_follow_the_luminosity_mask_ladder() {
        assert_eq!(Tone::Lights(2).weight(0.5), 0.25);
        assert_eq!(Tone::Darks(1).weight(0.25), 0.75);
        assert_eq!(Tone::Midtones(1).weight(0.5), 0.5);
        assert_eq!(Tone::Midtones(1).weight(0.0), 0.0);
        assert_eq!(Tone::parse("darks-3"), Some(Tone::Darks(3)));
    }

    #[test]
    fn a_lights_mask_is_white_on_white_and_black_on_black() {
        let rgba = [255, 255, 255, 255, 0, 0, 0, 255, 128, 128, 128, 255];
        assert_eq!(mask_from_rgba8(&rgba, 3, 1, Tone::Lights(1)).unwrap(), vec![255, 0, 128]);
    }
}
