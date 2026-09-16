//! Classic overworld carvers ported from Minecraft 26.2.
//!
//! - `NoiseBasedChunkGenerator.applyCarvers` (range ±8, LegacyRandom + `setLargeFeatureSeed`)
//! - `CaveWorldCarver` (`isStartChunk`, carve, createRoom, createTunnel)
//! - `WorldCarver.carveEllipsoid` (target-chunk local write)
//! - `Mth.sin` / `Mth.cos` use the vanilla 65536-entry lookup table
//!
//! Runs after noise+surface, before ore features.
//!
//! Copyright (c) 2026 Neutron Contributors -- MIT License

use crate::aquifer::NoiseBasedAquifer;
use crate::generator::WORLD_BOTTOM;
use crate::legacy_rng::LegacyRandom;
use crate::region_buf::RegionBuf;
use crate::surface::BlockId;
use crate::worldgen::WorldgenState;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::OnceLock;

/// Debug: blocks set to air/lava by carvers (process-wide).
pub static CARVE_WRITES: AtomicU32 = AtomicU32::new(0);
pub static CARVE_STARTS: AtomicU32 = AtomicU32::new(0);
pub static CARVE_TARGET_WRITES: AtomicU32 = AtomicU32::new(0);
pub static CARVE_ELLIPSOIDS: AtomicU32 = AtomicU32::new(0);
pub static CARVE_ELLIPSOID_HIT: AtomicU32 = AtomicU32::new(0);
pub static CARVE_CAN_REACH_FAIL: AtomicU32 = AtomicU32::new(0);
pub static CARVE_ROOM_CALLS: AtomicU32 = AtomicU32::new(0);
pub static CARVE_TUNNEL_STEPS: AtomicU32 = AtomicU32::new(0);
pub static CARVE_EARLY_OUT: AtomicU32 = AtomicU32::new(0);
pub static CARVE_EMPTY_RANGE: AtomicU32 = AtomicU32::new(0);
/// Ellipsoid cells in chunks (0,0)/(0,1) with y in [-16, 16).
pub static CARVE_BAND_CELL: AtomicU32 = AtomicU32::new(0);
pub static CARVE_BAND_NONE: AtomicU32 = AtomicU32::new(0);
pub static CARVE_BAND_WRITE: AtomicU32 = AtomicU32::new(0);
/// Set true to bypass canReach (diagnostic only).
pub static DIAG_SKIP_CAN_REACH: AtomicU32 = AtomicU32::new(0);
/// Diagnostic: disable the canyon carver entirely.
pub static DIAG_DISABLE_CANYON: AtomicU32 = AtomicU32::new(0);

/// Lava carver fill: `above_bottom: 8` → Y = -56.
const LAVA_Y: i32 = WORLD_BOTTOM + 8;

/// Hardcoded in `NoiseBasedChunkGenerator.applyCarvers` (not `getRange()`).
const APPLY_RANGE: i32 = 8;

/// Enable after parity verification.
pub const CARVERS_ENABLED: bool = true;

/// NEUTRON_CARVE_TRACE="<cx>,<cz>" — print SRC/EL trace lines for that target
/// chunk matching ProbeCarveTrace.java's format (cave carvers only).
fn carve_trace_target() -> Option<(i32, i32)> {
    static T: OnceLock<Option<(i32, i32)>> = OnceLock::new();
    *T.get_or_init(|| {
        let v = std::env::var("NEUTRON_CARVE_TRACE").ok()?;
        let mut it = v.split(',');
        let cx: i32 = it.next()?.trim().parse().ok()?;
        let cz: i32 = it.next()?.trim().parse().ok()?;
        Some((cx, cz))
    })
}

// ---------------------------------------------------------------------------
// Mth.sin / Mth.cos — vanilla 65536-entry table
// ---------------------------------------------------------------------------
// SIN[i] = (float) Math.sin(i / 10430.378350470453)
// sin(d) = SIN[(int)((long)(d * 10430.378350470453) & 0xFFFF)]
// cos(d) = SIN[(int)((long)(d * 10430.378350470453 + 16384.0) & 0xFFFF)]

const SIN_SCALE: f64 = 10430.378350470453;
const COS_OFFSET: f64 = 16384.0;

fn sin_table() -> &'static [f32; 65536] {
    static TABLE: OnceLock<[f32; 65536]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut t = [0f32; 65536];
        for i in 0..65536 {
            t[i] = (i as f64 / SIN_SCALE).sin() as f32;
        }
        t
    })
}

/// `Mth.sin(double)` → float via lookup table.
#[inline]
pub(crate) fn mth_sin_d(v: f64) -> f32 {
    let idx = ((v * SIN_SCALE) as i64 as u64 & 0xFFFF) as usize;
    sin_table()[idx]
}

/// `Mth.cos(double)` → float via lookup table.
#[inline]
fn mth_cos_d(v: f64) -> f32 {
    let idx = ((v * SIN_SCALE + COS_OFFSET) as i64 as u64 & 0xFFFF) as usize;
    sin_table()[idx]
}

/// `Mth.sin(float)` — promotes to double then table.
#[inline]
fn mth_sin_f(v: f32) -> f32 {
    mth_sin_d(v as f64)
}

/// `Mth.cos(float)` — promotes to double then table.
#[inline]
fn mth_cos_f(v: f32) -> f32 {
    mth_cos_d(v as f64)
}

// ---------------------------------------------------------------------------

struct CaveCfg {
    probability: f32,
    y_min: i32,
    y_max: i32,
}

/// Apply cave carvers for every chunk held in `region`.
pub fn apply_carvers_region(region: &mut RegionBuf, st: &WorldgenState) {
    if !CARVERS_ENABLED {
        return;
    }

    let level_seed = st.seed;
    if std::env::var_os("NEUTRON_DISABLE_CANYON").is_some() {
        DIAG_DISABLE_CANYON.store(1, Ordering::Relaxed);
    }

    // Biome carvers order (typical overworld biomes):
    // 0: cave, 1: cave_extra_underground, 2: canyon
    let cave_cfgs = [
        CaveCfg {
            probability: 0.15,
            y_min: WORLD_BOTTOM + 8, // above_bottom: 8 → -56
            y_max: 180,
        },
        CaveCfg {
            probability: 0.07,
            y_min: WORLD_BOTTOM + 8,
            y_max: 47,
        },
    ];

    let chunks = region.chunks;
    for tzl in 0..chunks {
        for txl in 0..chunks {
            let target_cx = region.origin_x.div_euclid(16) + txl;
            let target_cz = region.origin_z.div_euclid(16) + tzl;
            let mut aquifer = NoiseBasedAquifer::for_chunk(st, target_cx, target_cz);
            apply_carvers_for_target(
                region,
                &mut aquifer,
                level_seed,
                target_cx,
                target_cz,
                &cave_cfgs,
            );
        }
    }
}

fn apply_carvers_for_target(
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    level_seed: i64,
    target_cx: i32,
    target_cz: i32,
    cave_cfgs: &[CaveCfg],
) {
    let trace = carve_trace_target() == Some((target_cx, target_cz));
    for dz in -APPLY_RANGE..=APPLY_RANGE {
        for dx in -APPLY_RANGE..=APPLY_RANGE {
            let source_cx = target_cx + dx;
            let source_cz = target_cz + dz;
            // Indices 0,1: caves
            for (index, cfg) in cave_cfgs.iter().enumerate() {
                let mut rng = LegacyRandom::new(0);
                rng.set_large_feature_seed(
                    level_seed.wrapping_add(index as i64),
                    source_cx,
                    source_cz,
                );
                if rng.next_f32() > cfg.probability {
                    continue;
                }
                CARVE_STARTS.fetch_add(1, Ordering::Relaxed);
                if trace {
                    eprintln!("SRC {source_cx} {source_cz} {index}");
                }
                carve_from_chunk(
                    &mut rng, region, aquifer, source_cx, source_cz, target_cx, target_cz, cfg,
                );
            }
            // Index 2: canyon (probability 0.01)
            {
                let mut rng = LegacyRandom::new(0);
                rng.set_large_feature_seed(level_seed.wrapping_add(2), source_cx, source_cz);
                if rng.next_f32() <= 0.01 {
                    CARVE_STARTS.fetch_add(1, Ordering::Relaxed);
                    if trace {
                        eprintln!("SRC {source_cx} {source_cz} 2");
                    }
                    if DIAG_DISABLE_CANYON.load(Ordering::Relaxed) == 0 {
                        canyon_from_chunk(
                            &mut rng, region, aquifer, source_cx, source_cz, target_cx, target_cz,
                        );
                    }
                }
            }
        }
    }
}

fn carve_from_chunk(
    rng: &mut LegacyRandom,
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    source_cx: i32,
    source_cz: i32,
    target_cx: i32,
    target_cz: i32,
    cfg: &CaveCfg,
) {
    // rangeBlocks = SectionPos.sectionToBlockCoord(getRange()*2 - 1) with getRange=4
    // = sectionToBlockCoord(7) = 7*16 = 112
    let range_blocks = 7 * 16;
    // caveCount = nextInt(nextInt(nextInt(15)+1)+1)
    let a = rng.next_int(15) + 1;
    let b = rng.next_int(a) + 1;
    let cave_count = rng.next_int(b);

    for _ in 0..cave_count {
        let x = (source_cx * 16 + rng.next_int(16)) as f64;
        let y = sample_y(rng, cfg.y_min, cfg.y_max) as f64;
        let z = (source_cz * 16 + rng.next_int(16)) as f64;

        // FloatProvider.uniform: min + nextFloat() * (maxExclusive - min)
        let horiz_mult = 0.7 + rng.next_f32() * (1.4 - 0.7);
        let vert_mult = 0.8 + rng.next_f32() * (1.3 - 0.8);
        let floor_level = -1.0 + rng.next_f32() * (-0.4 - -1.0); // [-1, -0.4)

        let mut tunnel_count = 1;
        if rng.next_int(4) == 0 {
            // room
            let y_scale = 0.1 + rng.next_f32() * (0.9 - 0.1);
            let thickness = 1.0 + rng.next_f32() * 6.0;
            CARVE_ROOM_CALLS.fetch_add(1, Ordering::Relaxed);
            create_room(
                region,
                aquifer,
                target_cx,
                target_cz,
                x,
                y,
                z,
                thickness,
                y_scale as f64,
                floor_level as f64,
            );
            tunnel_count += rng.next_int(4);
        }

        for _ in 0..tunnel_count {
            // yaw = nextFloat() * TWO_PI  (6.2831855f)
            let yaw = rng.next_f32() * 6.2831855;
            let pitch = (rng.next_f32() - 0.5) / 4.0;
            let thickness = get_thickness(rng);
            let branch_count = range_blocks - rng.next_int(range_blocks / 4);
            let seed = rng.next_long();
            create_tunnel(
                region,
                aquifer,
                target_cx,
                target_cz,
                seed,
                x,
                y,
                z,
                horiz_mult as f64,
                vert_mult as f64,
                thickness,
                yaw,
                pitch,
                0,
                branch_count,
                1.0,
                floor_level as f64,
            );
        }
    }
}

fn sample_y(rng: &mut LegacyRandom, y_min: i32, y_max: i32) -> i32 {
    if y_max <= y_min {
        return y_min;
    }
    // UniformHeight inclusive both ends
    y_min + rng.next_int(y_max - y_min + 1)
}

fn get_thickness(rng: &mut LegacyRandom) -> f32 {
    let mut t = rng.next_f32() * 2.0 + rng.next_f32();
    if rng.next_int(10) == 0 {
        t *= rng.next_f32() * rng.next_f32() * 3.0 + 1.0;
    }
    t
}

fn create_room(
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    target_cx: i32,
    target_cz: i32,
    x: f64,
    y: f64,
    z: f64,
    thickness: f32,
    y_scale: f64,
    floor_level: f64,
) {
    // 1.5 + Mth.sin(1.5707963705062866d) * thickness
    let sin_half_pi = mth_sin_d(1.5707963705062866);
    let horiz = 1.5 + (sin_half_pi * thickness) as f64;
    let vert = horiz * y_scale;
    carve_ellipsoid(
        region,
        aquifer,
        target_cx,
        target_cz,
        x + 1.0,
        y,
        z,
        horiz,
        vert,
        floor_level,
    );
}

fn create_tunnel(
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    target_cx: i32,
    target_cz: i32,
    seed: i64,
    mut x: f64,
    mut y: f64,
    mut z: f64,
    horiz_mult: f64,
    vert_mult: f64,
    thickness: f32,
    mut yaw: f32,
    mut pitch: f32,
    branch_index: i32,
    branch_count: i32,
    y_scale: f64,
    floor_level: f64,
) {
    // RandomSource.createThreadLocalInstance(seed) → SingleThreadedRandomSource
    // (same LCG as LegacyRandomSource)
    let mut rng = LegacyRandom::new(seed);
    // steeperAt = nextInt(branchCount/2) + branchCount/4
    let steeper_at = rng.next_int(branch_count / 2) + branch_count / 4;
    let rare = rng.next_int(6) == 0;
    let mut yaw_vel = 0.0f32;
    let mut pitch_vel = 0.0f32;

    let mut i = branch_index;
    while i < branch_count {
        // horiz = 1.5 + Mth.sin((double)(PI * i / branchCount)) * thickness
        // bytecode: f2d of (3.1415927f * i / branchCount), then Mth.sin(D)F
        let angle = (3.1415927f32 * i as f32 / branch_count as f32) as f64;
        let sin_v = mth_sin_d(angle);
        let horiz_base = 1.5 + (sin_v * thickness) as f64;
        let vert_base = horiz_base * y_scale;
        // radius for this step (multipliers applied only at carveEllipsoid call site)
        let horiz = horiz_base * horiz_mult;
        let vert = vert_base * vert_mult;

        // Move along direction BEFORE velocity update:
        // cosPitch = Mth.cos(pitch)
        // x += Mth.cos(yaw) * cosPitch
        // y += Mth.sin(pitch)
        // z += Mth.sin(yaw) * cosPitch
        let cos_pitch = mth_cos_f(pitch);
        x += (mth_cos_f(yaw) * cos_pitch) as f64;
        y += mth_sin_f(pitch) as f64;
        z += (mth_sin_f(yaw) * cos_pitch) as f64;

        // pitch *= rare ? 0.92 : 0.7
        pitch *= if rare { 0.92 } else { 0.7 };
        pitch += pitch_vel * 0.1;
        yaw += yaw_vel * 0.1;
        pitch_vel *= 0.9;
        yaw_vel *= 0.75;
        pitch_vel += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 2.0;
        yaw_vel += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 4.0;

        if i == steeper_at && thickness > 1.0 {
            // fork two tunnels and return (no carve at fork step)
            create_tunnel(
                region,
                aquifer,
                target_cx,
                target_cz,
                rng.next_long(),
                x,
                y,
                z,
                horiz_mult,
                vert_mult,
                rng.next_f32() * 0.5 + 0.5,
                yaw - 1.5707964,
                pitch / 3.0,
                i,
                branch_count,
                1.0,
                floor_level,
            );
            create_tunnel(
                region,
                aquifer,
                target_cx,
                target_cz,
                rng.next_long(),
                x,
                y,
                z,
                horiz_mult,
                vert_mult,
                rng.next_f32() * 0.5 + 0.5,
                yaw + 1.5707964,
                pitch / 3.0,
                i,
                branch_count,
                1.0,
                floor_level,
            );
            return;
        }

        // skip carve 1/4 of the time (still advances)
        if rng.next_int(4) == 0 {
            i += 1;
            continue;
        }

        // canReach — abort entire tunnel if too far from target
        if DIAG_SKIP_CAN_REACH.load(Ordering::Relaxed) == 0
            && !can_reach(target_cx, target_cz, x, z, i, branch_count, thickness)
        {
            CARVE_CAN_REACH_FAIL.fetch_add(1, Ordering::Relaxed);
            return;
        }

        CARVE_TUNNEL_STEPS.fetch_add(1, Ordering::Relaxed);
        carve_ellipsoid(
            region,
            aquifer,
            target_cx,
            target_cz,
            x,
            y,
            z,
            horiz,
            vert,
            floor_level,
        );

        i += 1;
    }
}

/// `WorldCarver.canReach(chunkPos, x, z, branchIndex, branchCount, thickness)`.
///
/// Returns true when `dx²+dz² - remaining² <= (thickness+2+16)²`.
fn can_reach(
    target_cx: i32,
    target_cz: i32,
    x: f64,
    z: f64,
    branch_index: i32,
    branch_count: i32,
    thickness: f32,
) -> bool {
    let mid_x = (target_cx * 16 + 8) as f64;
    let mid_z = (target_cz * 16 + 8) as f64;
    let dx = x - mid_x;
    let dz = z - mid_z;
    let remaining = (branch_count - branch_index) as f64;
    // thickness + 2.0f + 16.0f, then f2d
    let max_r = (thickness + 2.0 + 16.0) as f64;
    // dcmpg: lhs <= rhs → true
    dx * dx + dz * dz - remaining * remaining <= max_r * max_r
}

/// `WorldCarver.carveEllipsoid` restricted to `target` chunk local blocks.
fn carve_ellipsoid(
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    target_cx: i32,
    target_cz: i32,
    cx: f64,
    cy: f64,
    cz: f64,
    horiz: f64,
    vert: f64,
    floor_level: f64,
) {
    CARVE_ELLIPSOIDS.fetch_add(1, Ordering::Relaxed);
    if let Some((tcx, tcz)) = carve_trace_target() {
        if tcx == target_cx && tcz == target_cz {
            eprintln!("EL {cx:.6} {cy:.6} {cz:.6} {horiz:.6} {vert:.6}");
        }
    }
    if horiz <= 0.0 || vert <= 0.0 {
        return;
    }
    let mid_x = (target_cx * 16 + 8) as f64;
    let mid_z = (target_cz * 16 + 8) as f64;
    // Early out if too far from target chunk middle
    let reach = 16.0 + horiz * 2.0;
    if (cx - mid_x).abs() > reach || (cz - mid_z).abs() > reach {
        CARVE_EARLY_OUT.fetch_add(1, Ordering::Relaxed);
        return;
    }
    let writes_before = CARVE_WRITES.load(Ordering::Relaxed);

    let min_bx = target_cx * 16;
    let min_bz = target_cz * 16;

    // Local x range in [0,15] — Mth.floor
    let min_lx = (mth_floor(cx - horiz) - min_bx - 1).max(0);
    let max_lx = (mth_floor(cx + horiz) - min_bx).min(15);
    // minY = max(floor(cy-vert)-1, minGenY+1)
    let min_y = (mth_floor(cy - vert) - 1).max(WORLD_BOTTOM + 1);
    // maxY = min(floor(cy+vert)+1, minY+height-1-7)
    // minY+height-1-7 = -64+384-1-7 = 312
    let max_y = (mth_floor(cy + vert) + 1)
        .min(WORLD_BOTTOM + 384 - 1 - 7)
        .max(min_y);
    let min_lz = (mth_floor(cz - horiz) - min_bz - 1).max(0);
    let max_lz = (mth_floor(cz + horiz) - min_bz).min(15);

    if min_lx > max_lx || min_lz > max_lz {
        CARVE_EMPTY_RANGE.fetch_add(1, Ordering::Relaxed);
        return;
    }

    for lx in min_lx..=max_lx {
        let wx = min_bx + lx;
        // xd = (wx + 0.5 - cx) / horiz
        let xd = (wx as f64 + 0.5 - cx) / horiz;
        if xd * xd >= 1.0 {
            continue;
        }
        for lz in min_lz..=max_lz {
            let wz = min_bz + lz;
            let zd = (wz as f64 + 0.5 - cz) / horiz;
            if xd * xd + zd * zd >= 1.0 {
                continue;
            }
            // Vanilla WorldCarver.carveEllipsoid: `for (worldY = maxY; worldY > minY; --)`.
            let mut y = max_y;
            while y > min_y {
                // yd = (y - 0.5 - cy) / vert
                // bytecode: i2d; ldc 0.5; dsub; dload cy; dsub; dload vert; ddiv
                let yd = (y as f64 - 0.5 - cy) / vert;
                if should_skip(xd, yd, zd, floor_level) {
                    y -= 1;
                    continue;
                }
                let existing = region.get(wx, y, wz);
                if !can_replace(existing) {
                    y -= 1;
                    continue;
                }
                let in_band = (target_cx == 0 && (target_cz == 0 || target_cz == 1))
                    && (-16..16).contains(&y);
                if in_band {
                    CARVE_BAND_CELL.fetch_add(1, Ordering::Relaxed);
                }
                // Vanilla WorldCarver.carveEllipsoid: `for (worldY = maxY; worldY > minY; --)`
                // so minY itself is not carved. Match that bound below via `y > min_y`.
                // getCarveState: y <= lavaLevel → lava; else aquifer.computeSubstance(ctx, 0.0).
                if y <= LAVA_Y {
                    region.set(wx, y, wz, BlockId::Lava);
                    if in_band {
                        CARVE_BAND_WRITE.fetch_add(1, Ordering::Relaxed);
                    }
                } else if let Some(b) = aquifer.compute_substance(wx, y, wz, 0.0) {
                    region.set(wx, y, wz, b);
                    CARVE_WRITES.fetch_add(1, Ordering::Relaxed);
                    CARVE_TARGET_WRITES.fetch_add(1, Ordering::Relaxed);
                    if in_band {
                        CARVE_BAND_WRITE.fetch_add(1, Ordering::Relaxed);
                    }
                } else if in_band {
                    CARVE_BAND_NONE.fetch_add(1, Ordering::Relaxed);
                }
                y -= 1;
            }
        }
    }
    if CARVE_WRITES.load(Ordering::Relaxed) > writes_before {
        CARVE_ELLIPSOID_HIT.fetch_add(1, Ordering::Relaxed);
    }
}

/// `Mth.floor(double)`.
#[inline]
fn mth_floor(v: f64) -> i32 {
    v.floor() as i32
}

/// CaveWorldCarver.shouldSkip(xd, yd, zd, floorLevel)
fn should_skip(xd: f64, yd: f64, zd: f64, floor_level: f64) -> bool {
    // yd <= floorLevel → skip
    if yd <= floor_level {
        return true;
    }
    // xd²+yd²+zd² >= 1 → skip
    xd * xd + yd * yd + zd * zd >= 1.0
}

// ---------------------------------------------------------------------------
// CanyonWorldCarver
// ---------------------------------------------------------------------------

/// `CanyonWorldCarver.carve` for one start chunk → target.
fn canyon_from_chunk(
    rng: &mut LegacyRandom,
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    source_cx: i32,
    source_cz: i32,
    target_cx: i32,
    target_cz: i32,
) {
    // rangeBlocks = getRange()*2-1 * 16 = 7*16 = 112 (same as cave getRange=4)
    let range_blocks = 7 * 16;
    let x = (source_cx * 16 + rng.next_int(16)) as f64;
    // y uniform absolute 10..67
    let y = (10 + rng.next_int(67 - 10 + 1)) as f64;
    let z = (source_cz * 16 + rng.next_int(16)) as f64;
    let yaw = rng.next_f32() * 6.2831855;
    // verticalRotation uniform [-0.125, 0.125)
    let pitch = -0.125 + rng.next_f32() * 0.25;
    // yScale constant 3.0
    let y_scale = 3.0f64;
    // thickness trapezoid min=0 max=6 plateau=2 — approximate as nextFloat*6
    let thickness = sample_trapezoid_thickness(rng);
    // distanceFactor uniform [0.75, 1.0)
    let distance_factor = 0.75 + rng.next_f32() * 0.25;
    let branch_count = ((range_blocks as f32) * distance_factor) as i32;
    let seed = rng.next_long();
    do_canyon(
        region,
        aquifer,
        target_cx,
        target_cz,
        seed,
        x,
        y,
        z,
        thickness,
        yaw,
        pitch,
        0,
        branch_count,
        y_scale,
    );
}

/// `TrapezoidFloat(min=0, max=6, plateau=2)` sample.
/// `TrapezoidFloat.sample` (TrapezoidFloat.java:35-40):
/// range=max-min; plateauStart=(range-plateau)/2; plateauEnd=range-plateauStart;
/// return min + nextFloat()*plateauEnd + nextFloat()*plateauStart.
fn sample_trapezoid_thickness(rng: &mut LegacyRandom) -> f32 {
    let min = 0.0f32;
    let max = 6.0f32;
    let plateau = 2.0f32;
    let range = max - min;
    let plateau_start = (range - plateau) / 2.0;
    let plateau_end = range - plateau_start;
    min + rng.next_f32() * plateau_end + rng.next_f32() * plateau_start
}

fn do_canyon(
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    target_cx: i32,
    target_cz: i32,
    seed: i64,
    mut x: f64,
    mut y: f64,
    mut z: f64,
    thickness: f32,
    mut yaw: f32,
    mut pitch: f32,
    branch_index: i32,
    branch_count: i32,
    y_scale: f64,
) {
    let mut rng = LegacyRandom::new(seed);
    // width factors for genDepth=384 Y levels
    let width_factors = init_width_factors(&mut rng, 384, 3); // widthSmoothness=3
    let mut yaw_vel = 0.0f32;
    let mut pitch_vel = 0.0f32;

    let mut i = branch_index;
    while i < branch_count {
        let sin_v = mth_sin_d((3.1415927f32 * i as f32 / branch_count as f32) as f64);
        let mut horiz = 1.5 + (sin_v * thickness) as f64;
        let mut vert = horiz * y_scale;
        // horizontalRadiusFactor uniform [0.75, 1.0) each step
        let hrf = 0.75 + rng.next_f32() * 0.25;
        horiz *= hrf as f64;
        vert = update_vertical_radius(&mut rng, vert, branch_count as f32, i as f32);

        let cos_pitch = mth_cos_f(pitch);
        x += (mth_cos_f(yaw) * cos_pitch) as f64;
        y += mth_sin_f(pitch) as f64;
        z += (mth_sin_f(yaw) * cos_pitch) as f64;

        pitch *= 0.7;
        pitch += pitch_vel * 0.05;
        yaw += yaw_vel * 0.05;
        pitch_vel *= 0.8;
        yaw_vel *= 0.5;
        pitch_vel += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 2.0;
        yaw_vel += (rng.next_f32() - rng.next_f32()) * rng.next_f32() * 4.0;

        if rng.next_int(4) == 0 {
            i += 1;
            continue;
        }
        if !can_reach(target_cx, target_cz, x, z, i, branch_count, thickness) {
            return;
        }

        // canyon shouldSkip uses widthFactors[y - minY - 1]
        carve_ellipsoid_canyon(
            region,
            aquifer,
            target_cx,
            target_cz,
            x,
            y,
            z,
            horiz,
            vert,
            &width_factors,
        );
        i += 1;
    }
}

fn init_width_factors(rng: &mut LegacyRandom, gen_depth: i32, width_smoothness: i32) -> Vec<f32> {
    let mut factors = vec![0.0f32; gen_depth as usize];
    let mut w = 1.0f32;
    for y in 0..gen_depth {
        if y == 0 || rng.next_int(width_smoothness) == 0 {
            w = 1.0 + rng.next_f32() * rng.next_f32();
        }
        factors[y as usize] = w * w;
    }
    factors
}

/// verticalRadiusDefaultFactor=1.0, verticalRadiusCenterFactor=0.0
fn update_vertical_radius(
    rng: &mut LegacyRandom,
    base_vert: f64,
    branch_count: f32,
    step: f32,
) -> f64 {
    // 1 - 2 * abs(0.5 - step/branchCount)
    let t = 1.0 - 2.0 * (0.5 - step / branch_count).abs();
    // defaultFactor + centerFactor * t = 1.0 + 0.0 * t
    let factor = 1.0f32 + 0.0 * t;
    // * randomBetween(0.75, 1.0)
    let r = 0.75 + rng.next_f32() * 0.25;
    base_vert * factor as f64 * r as f64
}

fn carve_ellipsoid_canyon(
    region: &mut RegionBuf,
    aquifer: &mut NoiseBasedAquifer<'_>,
    target_cx: i32,
    target_cz: i32,
    cx: f64,
    cy: f64,
    cz: f64,
    horiz: f64,
    vert: f64,
    width_factors: &[f32],
) {
    if let Some((tcx, tcz)) = carve_trace_target() {
        if tcx == target_cx && tcz == target_cz {
            eprintln!("CEL {cx:.6} {cy:.6} {cz:.6} {horiz:.6} {vert:.6}");
        }
    }
    if horiz <= 0.0 || vert <= 0.0 {
        return;
    }
    let mid_x = (target_cx * 16 + 8) as f64;
    let mid_z = (target_cz * 16 + 8) as f64;
    let reach = 16.0 + horiz * 2.0;
    if (cx - mid_x).abs() > reach || (cz - mid_z).abs() > reach {
        return;
    }
    let min_bx = target_cx * 16;
    let min_bz = target_cz * 16;
    let min_lx = (mth_floor(cx - horiz) - min_bx - 1).max(0);
    let max_lx = (mth_floor(cx + horiz) - min_bx).min(15);
    let min_y = (mth_floor(cy - vert) - 1).max(WORLD_BOTTOM + 1);
    let max_y = (mth_floor(cy + vert) + 1)
        .min(WORLD_BOTTOM + 384 - 1 - 7)
        .max(min_y);
    let min_lz = (mth_floor(cz - horiz) - min_bz - 1).max(0);
    let max_lz = (mth_floor(cz + horiz) - min_bz).min(15);
    if min_lx > max_lx || min_lz > max_lz {
        return;
    }
    for lx in min_lx..=max_lx {
        let wx = min_bx + lx;
        let xd = (wx as f64 + 0.5 - cx) / horiz;
        if xd * xd >= 1.0 {
            continue;
        }
        for lz in min_lz..=max_lz {
            let wz = min_bz + lz;
            let zd = (wz as f64 + 0.5 - cz) / horiz;
            if xd * xd + zd * zd >= 1.0 {
                continue;
            }
            // Vanilla: `worldY > minY` (minY itself is not carved).
            let mut y = max_y;
            while y > min_y {
                let yd = (y as f64 - 0.5 - cy) / vert;
                // canyon shouldSkip: xd²+zd² * widthFactors[y-minY-1] + yd²/6 >= 1
                let wi = (y - WORLD_BOTTOM - 1).clamp(0, width_factors.len() as i32 - 1) as usize;
                let wf = width_factors[wi] as f64;
                if xd * xd * wf + yd * yd / 6.0 + zd * zd * wf >= 1.0 {
                    y -= 1;
                    continue;
                }
                let existing = region.get(wx, y, wz);
                if !can_replace(existing) {
                    y -= 1;
                    continue;
                }
                if y <= LAVA_Y {
                    region.set(wx, y, wz, BlockId::Lava);
                } else if let Some(b) = aquifer.compute_substance(wx, y, wz, 0.0) {
                    // getCarveState: density-0 lets the aquifer carve water/air.
                    region.set(wx, y, wz, b);
                    CARVE_WRITES.fetch_add(1, Ordering::Relaxed);
                }
                y -= 1;
            }
        }
    }
}

fn can_replace(b: BlockId) -> bool {
    // #minecraft:overworld_carver_replaceables
    matches!(
        b,
        BlockId::Stone
            | BlockId::Granite
            | BlockId::Diorite
            | BlockId::Andesite
            | BlockId::Dirt
            | BlockId::GrassBlock
            | BlockId::Podzol
            | BlockId::CoarseDirt
            | BlockId::Mycelium
            | BlockId::Deepslate
            | BlockId::Tuff
            | BlockId::Gravel
            | BlockId::Sand
            | BlockId::RedSand
            | BlockId::Sandstone
            | BlockId::RedSandstone
            | BlockId::Calcite
            | BlockId::PackedIce
            | BlockId::Snow
            | BlockId::SnowLayer
            | BlockId::PowderSnow
            | BlockId::Clay
            | BlockId::Terracotta
            | BlockId::WhiteTerracotta
            | BlockId::OrangeTerracotta
            | BlockId::BrownTerracotta
            | BlockId::BlackTerracotta
            | BlockId::YellowTerracotta
            | BlockId::RedTerracotta
            | BlockId::LightGrayTerracotta
            | BlockId::Mud
            | BlockId::CoalOre
            | BlockId::IronOre
            | BlockId::CopperOre
            | BlockId::GoldOre
            | BlockId::RedstoneOre
            | BlockId::LapisOre
            | BlockId::DiamondOre
            | BlockId::DeepslateCoalOre
            | BlockId::DeepslateIronOre
            | BlockId::DeepslateCopperOre
            | BlockId::DeepslateGoldOre
            | BlockId::DeepslateRedstoneOre
            | BlockId::DeepslateLapisOre
            | BlockId::DeepslateDiamondOre
            | BlockId::RawIronBlock
            | BlockId::RawCopperBlock
            | BlockId::Water
            | BlockId::Sulfur
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mth_sin_half_pi_is_one() {
        // Vanilla constant used in createRoom
        let s = mth_sin_d(1.5707963705062866);
        assert!((s - 1.0).abs() < 1e-5, "sin(half_pi)={s}");
    }

    #[test]
    fn mth_sin_zero() {
        assert!((mth_sin_d(0.0) - 0.0).abs() < 1e-6);
        assert!((mth_cos_d(0.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn can_reach_near_center() {
        // At chunk (0,0) middle, remaining long → reachable
        assert!(can_reach(0, 0, 8.0, 8.0, 0, 100, 2.0));
    }

    #[test]
    fn can_reach_far_short_remaining() {
        // Far away with little remaining → not reachable
        assert!(!can_reach(0, 0, 500.0, 500.0, 99, 100, 2.0));
    }
}
