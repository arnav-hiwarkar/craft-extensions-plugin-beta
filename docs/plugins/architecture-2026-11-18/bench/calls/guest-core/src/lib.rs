use serde_json::{json, Value};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GetArgs { layer: u32 }

#[derive(Deserialize)]
struct LayerReply { id: u64, name: String, visible: bool, opacity: f32 }

fn call_raw(op: &str, a: &[u8]) -> Vec<u8> {
    let n = unsafe { craft_call(op.as_ptr() as i32, op.len() as i32, a.as_ptr() as i32, a.len() as i32) };
    assert!(n >= 0);
    let mut reply = vec![0u8; n as usize];
    unsafe { craft_take(reply.as_mut_ptr() as i32, reply.len() as i32) };
    reply
}


#[link(wasm_import_module = "craft")]
unsafe extern "C" {
    #[link_name = "call"]
    fn craft_call(op: i32, op_len: i32, args: i32, args_len: i32) -> i64;
    #[link_name = "take"]
    fn craft_take(dst: i32, len: i32) -> i32;
}

#[link(wasm_import_module = "bench")]
unsafe extern "C" {
    fn noop();
    fn set_opacity(layer: i32, v: f32) -> i32;
    fn get_layer(layer: i32, out: i32) -> i32;
    fn batch(ops: i32, n: i32, out: i32) -> i32;
}

fn call(op: &str, args: &Value) -> Value {
    let a = serde_json::to_vec(args).unwrap();
    let n = unsafe { craft_call(op.as_ptr() as i32, op.len() as i32, a.as_ptr() as i32, a.len() as i32) };
    assert!(n >= 0);
    let mut reply = vec![0u8; n as usize];
    let got = unsafe { craft_take(reply.as_mut_ptr() as i32, reply.len() as i32) };
    assert_eq!(got as i64, n);
    serde_json::from_slice(&reply).unwrap()
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LayerOut {
    id: u64,
    opacity: f32,
    visible: u32,
    name_len: u32,
    name: [u8; 52],
}

impl Default for LayerOut { fn default() -> Self { LayerOut { id: 0, opacity: 0.0, visible: 0, name_len: 0, name: [0; 52] } } }

#[repr(C)]
#[derive(Clone, Copy)]
struct OpIn {
    kind: u32,
    layer: u32,
    value: f32,
}

#[unsafe(no_mangle)]
pub extern "C" fn run(mode: u32, n: u32) -> u64 {
    let mut acc: u64 = 0;
    match mode {
        0 => {
            for i in 0..n {
                let v = call("layer.get", &json!({"layer": i % 1000, "name": "Layer"}));
                acc = acc.wrapping_add(v["id"].as_u64().unwrap() + v["name"].as_str().unwrap().len() as u64);
            }
        }
        1 => {
            for i in 0..n {
                let v = call("layer.set_opacity", &json!({"layer": i % 1000, "opacity": 0.5}));
                if v.is_null() { acc += 1; }
            }
        }
        2 => {
            let ops: Vec<Value> = (0..n).map(|i| json!({"op": "layer.get", "args": {"layer": i % 1000}})).collect();
            let v = call("ops.batch", &json!({"ops": ops}));
            for r in v.as_array().unwrap() {
                acc = acc.wrapping_add(r["id"].as_u64().unwrap());
            }
        }
        3 => {
            for _ in 0..n {
                unsafe { noop() };
                acc += 1;
            }
        }
        4 => {
            for i in 0..n {
                let r = unsafe { set_opacity((i % 1000) as i32, 0.5) };
                if r == 0 { acc += 1; }
            }
        }
        5 => {
            let mut out = LayerOut::default();
            for i in 0..n {
                let r = unsafe { get_layer((i % 1000) as i32, &mut out as *mut LayerOut as i32) };
                if r == 0 {
                    let name = core::str::from_utf8(&out.name[..out.name_len as usize]).unwrap();
                    acc = acc.wrapping_add(out.id + name.len() as u64);
                }
            }
        }
        6 => {
            let ops: Vec<OpIn> = (0..n).map(|i| OpIn { kind: 0, layer: i % 1000, value: 0.0 }).collect();
            let mut out = vec![LayerOut::default(); n as usize];
            let r = unsafe { batch(ops.as_ptr() as i32, n as i32, out.as_mut_ptr() as i32) };
            assert_eq!(r, 0);
            for o in &out { acc = acc.wrapping_add(o.id + o.name_len as u64); }
        }
        7 => {
            let ops: Vec<Value> = (0..n).map(|i| json!({"op": "layer.set_opacity", "args": {"layer": i % 1000, "opacity": 0.5}})).collect();
            let v = call("ops.batch", &json!({"ops": ops}));
            acc = v.as_array().unwrap().len() as u64;
        }
        8 => {
            for i in 0..n {
                let a = serde_json::to_vec(&GetArgs { layer: i % 1000 }).unwrap();
                let r: LayerReply = serde_json::from_slice(&call_raw("layer.get.typed", &a)).unwrap();
                acc = acc.wrapping_add(r.id + r.name.len() as u64 + r.visible as u64 + r.opacity as u64);
            }
        }
        _ => {}
    }
    acc
}
