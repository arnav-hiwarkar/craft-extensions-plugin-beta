//! The only module with `unsafe` code in craft-guest. It owns the scratch arena that `craft_alloc`
//! hands out, and the raw `craft` host imports. Everything else in the crate is safe Rust.
//!
//! Invariants that keep the unsafe blocks sound:
//! - Each scratch buffer is a `Vec<u64>`: its start is 8-byte aligned, so it can be viewed as bytes or
//!   as `f32`, and it is zero-initialised when allocated. It is never resized, so its address is stable
//!   until `reset` drops it.
//! - A host pointer becomes a slice only if it is exactly the start of a live buffer and the requested
//!   length fits inside that buffer. Anything else yields `None`.
//! - Slices are created while the arena's `RefCell` borrow is held, so `alloc` and `reset` cannot run
//!   (and drop the buffer) while a slice exists. Every bit pattern is a valid `u8` and `f32`.

use std::cell::RefCell;

struct Buf {
    words: Vec<u64>,
    /// Bytes the host may use. The allocation can be up to 7 bytes larger.
    bytes: usize,
}

thread_local! {
    static SCRATCH: RefCell<Vec<Buf>> = const { RefCell::new(Vec::new()) };
}

/// Allocates `len` zeroed bytes and returns their start address, or `None` when out of memory.
pub(crate) fn alloc(len: usize) -> Option<usize> {
    let words = len.div_ceil(8).max(1);
    let mut data: Vec<u64> = Vec::new();
    data.try_reserve_exact(words).ok()?;
    data.resize(words, 0);
    let addr = data.as_ptr() as usize;
    SCRATCH.with(|s| {
        let mut s = s.try_borrow_mut().ok()?;
        s.push(Buf { words: data, bytes: len });
        Some(addr)
    })
}

/// Frees all scratch. The host calls `craft_reset` before every invocation.
pub(crate) fn reset() {
    SCRATCH.with(|s| {
        if let Ok(mut s) = s.try_borrow_mut() {
            s.clear();
        }
    });
}

/// Runs `f` on the `len` bytes at `addr`, if `addr` is the start of a live scratch buffer.
pub(crate) fn with_bytes<R>(addr: usize, len: usize, f: impl FnOnce(&[u8]) -> R) -> Option<R> {
    SCRATCH.with(|s| {
        let s = s.try_borrow().ok()?;
        let buf = s.iter().find(|b| b.words.as_ptr() as usize == addr && len <= b.bytes)?;
        // SAFETY: `buf.words` is a live, zero-initialised allocation of `words.len() * 8 >= len` bytes,
        // and `u8` has no alignment or validity requirement beyond that. The shared `RefCell` borrow
        // `s` lives until after `f` returns, so the buffer is neither dropped nor resized meanwhile.
        let bytes = unsafe { std::slice::from_raw_parts(buf.words.as_ptr().cast::<u8>(), len) };
        Some(f(bytes))
    })
}

/// Runs `f` on `count` mutable `f32` values at `addr`, if `addr` is the start of a live scratch
/// buffer with room for them. The host writes these as little-endian bytes; wasm is little-endian.
pub(crate) fn with_f32_mut<R>(addr: usize, count: usize, f: impl FnOnce(&mut [f32]) -> R) -> Option<R> {
    let need = count.checked_mul(4)?;
    SCRATCH.with(|s| {
        let mut s = s.try_borrow_mut().ok()?;
        let buf = s.iter_mut().find(|b| b.words.as_ptr() as usize == addr && need <= b.bytes)?;
        // SAFETY: as in `with_bytes`. The allocation starts 8-byte aligned, which satisfies `f32`
        // alignment, and `count * 4 <= bytes` keeps the slice inside it. Every bit pattern is a valid
        // `f32`. The mutable `RefCell` borrow `s` is held for the whole closure, so this slice is the
        // only live reference to the memory.
        let px = unsafe { std::slice::from_raw_parts_mut(buf.words.as_mut_ptr().cast::<f32>(), count) };
        Some(f(px))
    })
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "craft")]
unsafe extern "C" {
    #[link_name = "log"]
    fn craft_log(level: i32, ptr: i32, len: i32);
    #[link_name = "call"]
    fn craft_call(op: i32, op_len: i32, args: i32, args_len: i32) -> i64;
    #[link_name = "result"]
    fn craft_result(ptr: i32, len: i32) -> i32;
    #[link_name = "take"]
    fn craft_take(dst: i32, len: i32) -> i32;
}

#[cfg(target_arch = "wasm32")]
fn ptr_of(bytes: &[u8]) -> i32 {
    // wasm32: a pointer is an i32 offset into linear memory.
    bytes.as_ptr() as usize as i32
}

#[cfg(target_arch = "wasm32")]
fn len_of(bytes: &[u8]) -> i32 {
    // A negative length is refused by the host, so an oversized slice fails safely.
    i32::try_from(bytes.len()).unwrap_or(-1)
}

/// Sends a line to the plugin console. The host reads at most 4 KiB.
#[cfg(target_arch = "wasm32")]
pub(crate) fn log(level: i32, text: &[u8]) {
    // SAFETY: the host bounds-checks `[ptr, ptr + len)` in guest memory and copies it out. `text` is
    // a live slice for the duration of the call.
    unsafe { craft_log(level, ptr_of(text), len_of(text)) }
}

/// Runs one session op. Returns the reply length (>= 0) or a negative error code.
#[cfg(target_arch = "wasm32")]
pub(crate) fn call(op: &[u8], args: &[u8]) -> i64 {
    // SAFETY: the host bounds-checks both ranges in guest memory and reads them. Both slices stay
    // alive across the call. The host never writes into them.
    unsafe { craft_call(ptr_of(op), len_of(op), ptr_of(args), len_of(args)) }
}

/// Copies the pending reply (or error JSON) into `dst`. Returns the bytes written or a negative code.
#[cfg(target_arch = "wasm32")]
pub(crate) fn take(dst: &mut [u8]) -> i32 {
    // SAFETY: the host bounds-checks `[dst, dst + len)` and writes only into that range. `dst` is an
    // exclusive live slice, so nothing else reads it during the call.
    unsafe { craft_take(ptr_of(dst), len_of(dst)) }
}

// Off wasm there is no host: the SDK compiles and its pure parts can be unit-tested.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn log(_level: i32, _text: &[u8]) {}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn call(_op: &[u8], _args: &[u8]) -> i64 {
    -7
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn take(_dst: &mut [u8]) -> i32 {
    -3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_is_aligned_readable_and_reset() {
        let addr = alloc(10).expect("allocates");
        assert_eq!(addr % 8, 0, "scratch starts 8-byte aligned");
        assert_eq!(with_bytes(addr, 10, <[u8]>::to_vec), Some(vec![0u8; 10]), "fresh scratch is zeroed");
        // The host would write these bytes; the guest then reads them back.
        let written = [1u8, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        SCRATCH.with(|s| {
            let mut s = s.borrow_mut();
            let buf = s.iter_mut().find(|b| b.words.as_ptr() as usize == addr).expect("buffer exists");
            // Write through the same typed view the host's bytes would land in.
            let view = unsafe { std::slice::from_raw_parts_mut(buf.words.as_mut_ptr().cast::<u8>(), 10) };
            view.copy_from_slice(&written);
        });
        assert_eq!(with_bytes(addr, 10, <[u8]>::to_vec), Some(written.to_vec()));
        assert!(with_bytes(addr, 11, |_| ()).is_none(), "a read past the allocation's requested length is refused");
        assert!(with_bytes(addr + 8, 1, |_| ()).is_none(), "a pointer inside a buffer is refused");
        reset();
        assert!(with_bytes(addr, 10, |_| ()).is_none(), "reset frees scratch");
    }

    #[test]
    fn f32_view_sees_host_bytes_in_little_endian_order() {
        let addr = alloc(8).expect("allocates");
        let values = [0.5f32, -2.0];
        let got = with_f32_mut(addr, 2, |px| {
            px.copy_from_slice(&values);
            px.to_vec()
        });
        assert_eq!(got, Some(values.to_vec()));
        let bytes = with_bytes(addr, 8, <[u8]>::to_vec).expect("same buffer");
        assert_eq!(&bytes[..4], &values[0].to_ne_bytes());
        assert!(with_f32_mut(addr, 3, |_| ()).is_none(), "too many values for the buffer");
        reset();
    }
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn result(bytes: &[u8]) -> i32 {
    // SAFETY: live immutable bytes; host copies a bounded range.
    unsafe { craft_result(ptr_of(bytes), len_of(bytes)) }
}
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn result(_bytes: &[u8]) -> i32 {
    -7
}
