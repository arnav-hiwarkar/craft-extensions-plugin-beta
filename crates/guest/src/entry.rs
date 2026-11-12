//! Glue behind the `filter!` and `session!` macros. Safe code only; the unsafe reads live in `abi`.

use crate::abi;
use crate::host::{LogLevel, log};
use crate::{Event, FilterCtx, Image};

/// The ABI version this SDK implements (beta-design B4). The host refuses other values.
pub const ABI_VERSION: i32 = 1;

/// `craft_alloc`: returns a scratch buffer of `len` bytes, or 0 when out of memory.
pub fn alloc(len: i32) -> i32 {
    let Ok(len) = usize::try_from(len) else { return 0 };
    abi::alloc(len).and_then(|addr| i32::try_from(addr).ok()).filter(|&p| p > 0).unwrap_or(0)
}

/// `craft_reset`: frees all scratch.
pub fn reset() {
    abi::reset();
}

fn addr(ptr: i32) -> Result<usize, String> {
    usize::try_from(ptr).map_err(|_| format!("the host passed a bad pointer ({ptr})"))
}

/// `craft_filter`: runs `run` on the host's context and pixels. Returns 0, or -1 after logging.
pub fn filter_entry<F>(ctx: i32, ctx_len: i32, px: i32, w: i32, h: i32, run: F) -> i32
where
    F: FnOnce(&FilterCtx, &mut Image<'_>) -> Result<(), String>,
{
    match run_filter(ctx, ctx_len, px, w, h, run) {
        Ok(()) => 0,
        Err(message) => {
            log(LogLevel::Error, &message);
            -1
        }
    }
}

fn run_filter<F>(ctx: i32, ctx_len: i32, px: i32, w: i32, h: i32, run: F) -> Result<(), String>
where
    F: FnOnce(&FilterCtx, &mut Image<'_>) -> Result<(), String>,
{
    let width = u32::try_from(w).map_err(|_| "bad image width".to_owned())?;
    let height = u32::try_from(h).map_err(|_| "bad image height".to_owned())?;
    let count = (width as usize).checked_mul(height as usize).and_then(|n| n.checked_mul(4)).ok_or_else(|| "image too large".to_owned())?;
    let ctx_len = usize::try_from(ctx_len).map_err(|_| "bad context length".to_owned())?;
    let parsed =
        abi::with_bytes(addr(ctx)?, ctx_len, |bytes| serde_json::from_slice::<FilterCtx>(bytes).map_err(|e| format!("bad filter context: {e}")))
            .ok_or_else(|| "the host's filter context is not in scratch memory".to_owned())??;
    abi::with_f32_mut(addr(px)?, count, |pixels| {
        let mut image = Image { width, height, pixels };
        run(&parsed, &mut image)
    })
    .ok_or_else(|| "the host's pixel buffer is not in scratch memory".to_owned())?
}

/// `craft_event`: decodes the event and runs `run` on it. Returns 0, or -1 after logging.
pub fn session_entry<F>(ev: i32, ev_len: i32, run: F) -> i32
where
    F: FnOnce(Event) -> Result<(), String>,
{
    match read_event(ev, ev_len).and_then(run) {
        Ok(()) => 0,
        Err(message) => {
            log(LogLevel::Error, &message);
            -1
        }
    }
}

fn read_event(ev: i32, ev_len: i32) -> Result<Event, String> {
    let len = usize::try_from(ev_len).map_err(|_| "bad event length".to_owned())?;
    abi::with_bytes(addr(ev)?, len, |bytes| serde_json::from_slice::<Event>(bytes).map_err(|e| format!("bad event: {e}")))
        .ok_or_else(|| "the host's event is not in scratch memory".to_owned())?
}
