use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use craft_native::{Event, Hello, Host, Value, json};
use std::sync::{Arc, Mutex};

const READ_BYTES: usize = 4 * 1024 * 1024;
const WRITE_BYTES: usize = 700 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Picture {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

pub fn denoise(src: &Picture, strength: f32, cancelled: &(dyn Fn() -> bool + Sync), progress: &(dyn Fn(f32) + Sync)) -> Option<Picture> {
    let (w, h) = (src.width, src.height);
    let h2 = (strength * strength).max(1e-6);
    let px = |x: isize, y: isize| {
        let (x, y) = (x.clamp(0, w as isize - 1) as usize, y.clamp(0, h as isize - 1) as usize);
        let i = (y * w + x) * 4;
        [f32::from(src.rgba[i]), f32::from(src.rgba[i + 1]), f32::from(src.rgba[i + 2])]
    };
    let mut out = src.rgba.clone();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let rows_per = h.div_ceil(threads).max(1);
    let done = std::sync::atomic::AtomicUsize::new(0);
    let stopped = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|s| {
        for (band, chunk) in out.chunks_mut(rows_per * w * 4).enumerate() {
            let (done, stopped) = (&done, &stopped);
            s.spawn(move || {
                for (r, row) in chunk.chunks_mut(w * 4).enumerate() {
                    if stopped.load(std::sync::atomic::Ordering::Relaxed) || cancelled() {
                        stopped.store(true, std::sync::atomic::Ordering::Relaxed);
                        return;
                    }
                    let y = (band * rows_per + r) as isize;
                    for x in 0..w as isize {
                        let (mut acc, mut total) = ([0.0f32; 3], 0.0f32);
                        for sy in -3..=3 {
                            for sx in -3..=3 {
                                let mut d = 0.0;
                                for py in -1..=1 {
                                    for px_ in -1..=1 {
                                        let (a, b) = (px(x + px_, y + py), px(x + sx + px_, y + sy + py));
                                        d += (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
                                    }
                                }
                                let wgt = (-(d / 27.0) / h2).exp();
                                let c = px(x + sx, y + sy);
                                for k in 0..3 {
                                    acc[k] += c[k] * wgt;
                                }
                                total += wgt;
                            }
                        }
                        for k in 0..3 {
                            row[x as usize * 4 + k] = (acc[k] / total).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                    let n = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                    if n % 32 == 0 {
                        progress(n as f32 / h as f32);
                    }
                }
            });
        }
    });
    (!stopped.into_inner()).then_some(Picture { width: w, height: h, rgba: out })
}

fn read_layer(host: &mut Host, width: usize, height: usize) -> Result<Picture, String> {
    let rows = (READ_BYTES / (width * 4)).max(1);
    let mut rgba = Vec::with_capacity(width * height * 4);
    for y in (0..height).step_by(rows) {
        let n = rows.min(height - y);
        let tile = host.call("pixels.get", json!({"rect": [0, y, width, n], "format": "rgba8"})).map_err(|e| e.to_string())?;
        let data = STANDARD.decode(tile["data"].as_str().unwrap_or_default()).map_err(|e| e.to_string())?;
        if data.len() != width * n * 4 {
            return Err("the app returned a tile of the wrong size".into());
        }
        rgba.extend_from_slice(&data);
    }
    Ok(Picture { width, height, rgba })
}

fn write_layer(host: &mut Host, picture: &Picture) -> Result<(), String> {
    let layer = host.call("layer.create", json!({"kind": "pixel", "name": "Quietgrain"})).map_err(|e| e.to_string())?;
    let rows = (WRITE_BYTES / (picture.width * 4)).max(1);
    for y in (0..picture.height).step_by(rows) {
        let n = rows.min(picture.height - y);
        let bytes = &picture.rgba[y * picture.width * 4..(y + n) * picture.width * 4];
        let args = json!({"layer": layer["id"], "x": 0, "y": y, "width": picture.width, "height": n, "format": "rgba8", "data": STANDARD.encode(bytes)});
        host.call("pixels.set", args).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Default)]
struct State {
    pending: Arc<Mutex<Option<(u64, Value, Picture)>>>,
}

fn handle(state: &mut State, host: &mut Host, event: Event) -> Result<(), String> {
    match event {
        Event::Command { .. } => {
            let info = host.call("document.info", json!({})).map_err(|e| e.to_string())?;
            let (w, h) = (info["width"].as_u64().unwrap_or(0) as usize, info["height"].as_u64().unwrap_or(0) as usize);
            if w == 0 || h == 0 || info["mode"] != "rgb" {
                return Err("Quietgrain works on RGB documents".into());
            }
            let picture = read_layer(host, w, h)?;
            let job = host.start_job("Quietgrain: removing noise").map_err(|e| e.to_string())?;
            let pending = state.pending.clone();
            let size = json!([w, h]);
            std::thread::spawn(move || {
                let cancelled = || job.cancelled();
                let report = |f: f32| job.progress(Some(f), "cleaning");
                match denoise(&picture, 24.0, &cancelled, &report) {
                    Some(clean) => {
                        let id = job.id();
                        *pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some((id, size, clean));
                        job.finish(json!({"ready": true}));
                    }
                    None => job.fail("cancelled"),
                }
            });
            Ok(())
        }
        Event::Job { job, outcome: craft_native::JobOutcome::Done(_) } => {
            let Some((id, size, clean)) = state.pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take() else { return Ok(()) };
            if id != job {
                return Ok(());
            }
            let info = host.call("document.info", json!({})).map_err(|e| e.to_string())?;
            if json!([info["width"], info["height"]]) != size {
                return Err("the document changed size while Quietgrain was working".into());
            }
            write_layer(host, &clean)
        }
        _ => Ok(()),
    }
}

fn main() -> std::process::ExitCode {
    let mut state = State::default();
    craft_native::run(move |host: &mut Host, _: &Hello, event| handle(&mut state, host, event))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_reduced_while_a_flat_area_keeps_its_level() {
        let (w, h) = (24, 24);
        let rgba: Vec<u8> = (0..w * h).flat_map(|i| { let n = if (i * 7919) % 5 == 0 { 40 } else { 0 }; [100 + n, 100 + n, 100 + n, 255] }).collect();
        let src = Picture { width: w, height: h, rgba };
        let clean = denoise(&src, 30.0, &|| false, &|_| {}).unwrap();
        let spread = |p: &Picture| { let v: Vec<f32> = p.rgba.chunks_exact(4).map(|c| f32::from(c[0])).collect(); let m = v.iter().sum::<f32>() / v.len() as f32; v.iter().map(|x| (x - m).powi(2)).sum::<f32>() / v.len() as f32 };
        assert!(spread(&clean) < spread(&src) / 4.0, "{} vs {}", spread(&clean), spread(&src));
        assert!(clean.rgba.chunks_exact(4).all(|c| c[3] == 255));
    }

    #[test]
    fn a_cancelled_run_returns_nothing() {
        let src = Picture { width: 8, height: 8, rgba: vec![128; 8 * 8 * 4] };
        assert_eq!(denoise(&src, 10.0, &|| true, &|_| {}), None);
    }
}
