wit_bindgen::generate!({ path: "../wit", world: "plugin" });

use craft::bench::doc;

struct G;

impl Guest for G {
    fn run(mode: u32, n: u32) -> u64 {
        let mut acc: u64 = 0;
        match mode {
            0 => {
                for i in 0..n {
                    let l = doc::get_layer(i % 1000).unwrap();
                    acc = acc.wrapping_add(l.id + l.name.len() as u64 + (l.opacity * 100.0) as u64);
                }
            }
            1 => {
                for i in 0..n {
                    doc::set_opacity(i % 1000, 0.5).unwrap();
                    acc += 1;
                }
            }
            2 => {
                let h = doc::open_layer(7).unwrap();
                for _ in 0..n {
                    h.set_opacity(0.25).unwrap();
                    acc += 1;
                }
            }
            3 => {
                let ops: Vec<doc::Op> = (0..n).map(|i| doc::Op { kind: doc::OpKind::GetLayer, layer: i % 1000, value: 0.0 }).collect();
                for r in doc::batch(&ops) {
                    if let doc::Reply::Layer(l) = r {
                        acc = acc.wrapping_add(l.id + l.name.len() as u64 + (l.opacity * 100.0) as u64);
                    }
                }
            }
            4 => {
                for i in 0..n {
                    let args = serde_json::to_string(&serde_json::json!({"layer": i % 1000, "name": "Layer"})).unwrap();
                    let r = doc::call("layer.get", &args).unwrap();
                    let v: serde_json::Value = serde_json::from_str(&r).unwrap();
                    acc = acc.wrapping_add(v["id"].as_u64().unwrap());
                }
            }
            5 => {
                for _ in 0..n {
                    doc::noop();
                    acc += 1;
                }
            }
            6 => {
                for i in 0..n {
                    let h = doc::open_layer(i % 1000).unwrap();
                    acc = acc.wrapping_add((h.opacity() * 100.0) as u64);
                }
            }
            7 => {
                let ops: Vec<doc::Op> = (0..n).map(|i| doc::Op { kind: doc::OpKind::SetOpacity, layer: i % 1000, value: 0.5 }).collect();
                acc = doc::batch(&ops).len() as u64;
            }
            _ => {}
        }
        acc
    }
}

export!(G);
