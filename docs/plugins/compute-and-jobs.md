# Compute kernels and background jobs

Sandboxed plugins run in an interpreter (wasmi). It is safe and portable, but a per-pixel loop
in Wasm is tens of times slower than native code. Two host features close that gap without
giving plugins native access:

- **Kernels** (`kernel.*`): prebuilt, multi-threaded image functions the app runs on pixels in
  the plugin's own memory.
- **Jobs** (`job.*`): long work on a background thread, with progress and cancel. The finished
  result comes back as an ordinary undoable event.

Both go through the existing `craft.call` import, so ABI v1 is unchanged. In an app that predates
them, a plugin gets error `-3` (not found) and can fall back to a plain loop.

## Kernels

Kernels need no permission and touch no document, so they work in effects (`raster.filter@1`),
sessions and jobs. Images are premultiplied RGBA `f32` in the plugin's memory, addressed as
`{"ptr", "width", "height"}`; the SDK's [`craft_guest::kernel`](../../crates/guest/src/kernel.rs)
does this for you.

| Op | Does | Arguments besides `image` |
|---|---|---|
| `kernel.blur` | Gaussian or box blur | `radius` 0..=512, `kind` `gaussian`/`box` |
| `kernel.directional_blur` | Motion blur | `angle` degrees, `length` 0..=2048 |
| `kernel.convolve` | Odd `size`×`size` convolution, size ≤ 15 | `size`, `weights`, `divisor`, `bias` |
| `kernel.morphology` | Dilate or erode | `radius`, `op` |
| `kernel.bilateral` | Edge-preserving smoothing | `radius` 0..=32, `tolerance` (straight colour distance) |
| `kernel.hsl_range` | Replace with a soft grey mask of a hue/saturation/lightness range | `hue` [lo, hi] degrees (wraps), `saturation`, `lightness` [lo, hi], `softness` |
| `kernel.mix` | Move `image` toward `src` | `src`, `amount` 0..=1 |
| `kernel.mix_masked` | `kernel.mix` weighted by `mask` luminance | `src`, `mask`, `amount` |
| `kernel.color_matrix` | 4×5 matrix on straight colour | `matrix` (20 values) |
| `kernel.curves` | Sampled curves on straight colour | `rgb`, `r`, `g`, `b`, `a` (2..=4096 values each) |
| `kernel.blend` | Blend `src` over `image` | `src`, `mode`, `opacity` |
| `kernel.noise` | Deterministic fractal value noise | `scale`, `octaves`, `seed`, `z`, `mode` `fill`/`multiply` |
| `kernel.resample` | Resize `src` into `image` | `src` |
| `kernel.warp_affine` | Affine-transform `src` into `image` | `src`, `matrix` (6 values) |
| `kernel.splat` | Add up to 10⁶ soft discs as light | `points` `{ptr, count}` of 8 floats each |
| `kernel.histogram` | 256-bin straight-colour histograms (reads only) | – |

Blend modes are `normal`, `add`, `screen`, `multiply`, `overlay`, `soft-light`, `lighten`,
`darken` and `difference`.

Kernels follow these rules:

- **Bad arguments are refused (`-1`), never silently clamped.** Every image must lie inside the
  plugin's memory. Non-finite results are written back as 0.
- **Kernels cost fuel.** The cost is pixels × taps, charged to the event, filter or job budget. A
  call larger than the remaining budget is refused with `-6` before it runs. Work beyond the
  current fuel slice is carried as debt, so many large calls still hit the per-call limit.
- **Kernels run on up to 8 threads**, inside the call. The plugin sees the result when the call
  returns.

```rust
use craft_guest::{filter, kernel, FilterCtx, Image};

fn glow(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let mut halo = kernel::Buffer::copy_of(img);
    kernel::blur(&mut halo.image(), ctx.f("radius")? as f32).map_err(|e| e.to_string())?;
    kernel::blend(img, &halo.image(), kernel::Blend::Screen, ctx.f("strength")? as f32).map_err(|e| e.to_string())
}
filter!(glow);
```

## Jobs

A session event starts a job with `job.start {label, input, cancellable}` and gets `{"job": id}`
back. The app runs the module's `craft_job` export with that input in a fresh sandbox. It has up
to 512 MiB of memory and about a thousand times an event's fuel, which is minutes of work.

A job has kernels, logging and `job.progress {value, note}`, but no app operations. It cannot
read or change a document, so a long job never holds a transaction or edits a stale document.
It returns one JSON result (at most 8 MiB) with `craft.result`.

When the job ends, the session receives `{"event":"job","job":id,"ok":true,"result":…}`. On
failure it gets `"ok":false` with `"error"` or `"cancelled":true` instead. This is a normal event
in its own transaction and undo step. Apply the result there and check that the target still
exists.

Jobs have these limits and controls:

- **Concurrency.** At most 4 jobs per plugin and 16 per app run at once; more are refused with `-6`.
- **Cancel.** The person can cancel a job from the app's jobs list. The plugin can cancel its own
  jobs with `job.cancel {job}`.
- **Stopping.** Turning the plugin off, revoking a permission or kill-listing it stops its jobs
  between fuel slices.

The app owns a [`JobBoard`](../../crates/ext-host/src/jobs.rs) and calls `Session::set_jobs` for
each session. It shows `JobBoard::list()` in its jobs list and delivers each
`JobBoard::take_finished()` entry to the owning session as `JobInfo::event()`.

Effects can report progress too. `job.progress` in a filter goes to the progress sink the app set
with `Filter::set_progress`, which is the app's render progress bar.

```rust
use craft_guest::{job, session, Event, JsonValue};

fn handle(event: Event) -> Result<(), String> {
    match event {
        Event::Command { .. } => { job::start("Analyse clip", craft_guest::json!({"frames": 240})).map_err(|e| e.to_string())?; }
        Event::Job { outcome: job::Outcome::Done(result), .. } => { /* apply with normal ops */ let _ = result; }
        _ => {}
    }
    Ok(())
}
fn analyse(input: JsonValue) -> Result<JsonValue, String> {
    let frames = input["frames"].as_u64().unwrap_or(0);
    for f in 0..frames { job::progress(Some(f as f32 / frames as f32), "analysing"); }
    Ok(craft_guest::json!({"frames": frames}))
}
session!(handle);
craft_guest::job!(analyse);
```
