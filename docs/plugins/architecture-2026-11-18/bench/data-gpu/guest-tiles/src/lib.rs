#![no_std]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { core::arch::wasm32::unreachable() }
#[link(wasm_import_module = "host")]
unsafe extern "C" {
    fn read_tile(x: u32, y: u32, w: u32, h: u32, dst: *mut f32) -> i32;
    fn write_tile(x: u32, y: u32, w: u32, h: u32, src: *const f32) -> i32;
    fn tick() -> i32;
}
static mut HEAP: usize = 0;
#[unsafe(no_mangle)]
pub extern "C" fn alloc(n: u32) -> *mut u8 {
    unsafe {
        if HEAP == 0 { HEAP = (core::arch::wasm32::memory_size(0) * 65536) as usize; }
        let pages = (n as usize + 65535) / 65536;
        let base = HEAP;
        if core::arch::wasm32::memory_grow(0, pages) == usize::MAX { return core::ptr::null_mut(); }
        HEAP += pages * 65536;
        base as *mut u8
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn bright(p: *mut f32, n: u32, gain: f32) {
    let s = unsafe { core::slice::from_raw_parts_mut(p, n as usize) };
    for px in s.chunks_exact_mut(4) { px[0] *= gain; px[1] *= gain; px[2] *= gain; }
}
#[unsafe(no_mangle)]
pub extern "C" fn bright_tiles(w: u32, h: u32, t: u32, gain: f32, scratch: *mut f32) -> i32 {
    let mut y = 0;
    while y < h {
        let th = if y + t > h { h - y } else { t };
        let mut x = 0;
        while x < w {
            let tw = if x + t > w { w - x } else { t };
            unsafe { if read_tile(x, y, tw, th, scratch) != 0 { return -1; } }
            bright(scratch, tw * th * 4, gain);
            unsafe { if write_tile(x, y, tw, th, scratch) != 0 { return -1; } }
            x += t;
        }
        y += t;
    }
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn calls(n: u32) -> i32 {
    let mut a = 0;
    for _ in 0..n { a += unsafe { tick() }; }
    a
}
#[unsafe(no_mangle)]
pub extern "C" fn noop() -> i32 { 0 }
