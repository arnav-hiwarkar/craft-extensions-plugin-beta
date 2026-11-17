use craft_guest::kernel::{self, Blend, Buffer, Splat};
use craft_guest::{FilterCtx, Image, filter};

pub const MAX_ALIVE: usize = 200_000;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Emitter {
    pub origin: [f32; 2],
    pub rate: f32,
    pub lifetime: f32,
    pub speed: f32,
    pub direction: f32,
    pub spread: f32,
    pub gravity: f32,
    pub turbulence: f32,
    pub size: f32,
    pub birth: [f32; 4],
    pub death: [f32; 4],
    pub seed: u64,
}

fn unit(seed: u64, index: u64, salt: u64) -> f32 {
    let mut h = seed ^ index.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ salt.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
    h ^= h >> 31;
    (h >> 40) as f32 / (1u64 << 24) as f32
}

fn lerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t, a[3] + (b[3] - a[3]) * t]
}

impl Emitter {
    pub fn particles(&self, time: f32) -> Vec<Splat> {
        if self.rate <= 0.0 || time < 0.0 || !time.is_finite() {
            return Vec::new();
        }
        let first = ((time - self.lifetime) * self.rate).floor().max(0.0) as u64;
        let last = (time * self.rate).floor() as u64;
        let mut out = Vec::with_capacity(((last - first) as usize).min(MAX_ALIVE));
        for i in first..=last {
            if out.len() >= MAX_ALIVE {
                break;
            }
            let born = i as f32 / self.rate;
            let age = time - born;
            if !(0.0..self.lifetime).contains(&age) {
                continue;
            }
            let life = age / self.lifetime;
            let angle = (self.direction + (unit(self.seed, i, 1) - 0.5) * self.spread).to_radians();
            let speed = self.speed * (0.6 + 0.8 * unit(self.seed, i, 2));
            let wobble = self.turbulence * life;
            let x = self.origin[0] + angle.cos() * speed * age + (unit(self.seed, i, 3) - 0.5) * wobble * (age * 3.0 + i as f32).sin().abs();
            let y = self.origin[1] + angle.sin() * speed * age + 0.5 * self.gravity * age * age + (unit(self.seed, i, 4) - 0.5) * wobble;
            let fade = (1.0 - life) * (life * 8.0).min(1.0);
            let mut color = lerp(self.birth, self.death, life);
            color[3] = fade;
            out.push(Splat { x, y, radius: self.size * (0.5 + unit(self.seed, i, 5)) * (1.0 - 0.5 * life), color, hardness: 0.3 });
        }
        out
    }
}

fn fail(e: craft_guest::CallError) -> String {
    format!("Emberfield needs host kernels: {e}")
}

fn emberfield(ctx: &FilterCtx, img: &mut Image<'_>) -> Result<(), String> {
    let scale = ctx.scale as f32;
    if !(scale.is_finite() && scale > 0.0) {
        return Err("the render scale must be positive".into());
    }
    let [px, py] = ctx.point("emitter")?;
    let full = [ctx.full[0] as f32, ctx.full[1] as f32];
    let emitter = Emitter {
        origin: [px as f32 / 100.0 * full[0] - ctx.origin[0] as f32, py as f32 / 100.0 * full[1] - ctx.origin[1] as f32],
        rate: ctx.f("rate")? as f32,
        lifetime: ctx.f("lifetime")? as f32,
        speed: ctx.f("speed")? as f32 * scale,
        direction: ctx.f("direction")? as f32,
        spread: ctx.f("spread")? as f32,
        gravity: ctx.f("gravity")? as f32 * scale,
        turbulence: ctx.f("turbulence")? as f32 * scale,
        size: (ctx.f("size")? as f32 * scale).min(kernel::MAX_RADIUS),
        birth: ctx.color("birth_color")?,
        death: ctx.color("death_color")?,
        seed: ctx.seed,
    };
    let splats = emitter.particles(ctx.time as f32);
    if splats.is_empty() {
        return Ok(());
    }
    let mut layer = Buffer::new(img.width, img.height);
    kernel::splat(&mut layer.image(), &splats).map_err(fail)?;
    let glow = ctx.f("glow")? as f32;
    if glow > 0.0 {
        let mut halo = layer.clone();
        kernel::blur(&mut halo.image(), (emitter.size * 4.0).min(kernel::MAX_RADIUS)).map_err(fail)?;
        kernel::blend(&mut layer.image(), &halo.image(), Blend::Add, glow.min(1.0)).map_err(fail)?;
    }
    kernel::blend(img, &layer.image(), Blend::Add, 1.0).map_err(fail)
}

filter!(emberfield);

#[cfg(test)]
mod tests {
    use super::*;

    fn emitter() -> Emitter {
        Emitter {
            origin: [100.0, 100.0],
            rate: 10.0,
            lifetime: 1.0,
            speed: 50.0,
            direction: -90.0,
            spread: 0.0,
            gravity: 0.0,
            turbulence: 0.0,
            size: 2.0,
            birth: [1.0, 1.0, 1.0, 1.0],
            death: [0.0, 0.0, 0.0, 1.0],
            seed: 7,
        }
    }

    #[test]
    fn only_particles_younger_than_their_lifetime_are_alive_and_the_frame_is_repeatable() {
        let e = emitter();
        let a = e.particles(2.0);
        assert_eq!(a.len(), 10);
        assert_eq!(a, e.particles(2.0));
        assert!(e.particles(-1.0).is_empty());
    }

    #[test]
    fn a_particle_moves_along_its_direction_at_its_speed() {
        let e = emitter();
        let p = e.particles(0.5)[0];
        assert!((p.x - 100.0).abs() < 1e-3);
        assert!(p.y < 100.0 - 50.0 * 0.6 * 0.5 + 1e-3 && p.y > 100.0 - 50.0 * 1.4 * 0.5 - 1e-3, "{}", p.y);
    }
}
