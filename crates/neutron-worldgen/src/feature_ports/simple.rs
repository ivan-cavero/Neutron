//! Feature ports: desert_well / freeze_top_layer / spike / bamboo / monster_room.
use super::*;
use crate::feature_catalog;
use crate::feature_dispatch;
use crate::feature_dispatch::*;
use crate::feature_rng::FeatureRandom;
use crate::generator::{WORLD_BOTTOM, WORLD_TOP};
use crate::legacy_rng::LegacyRandom;
use crate::region_buf::RegionBuf;
use crate::surface::BlockId;
use crate::worldgen::WorldgenState;
use serde_json::Value;


// ---------------------------------------------------------------------------
// desert_well
// ---------------------------------------------------------------------------

/// `DesertWellFeature.place` (26.2). No RNG except the two suspicious-sand
/// picks at the end.
pub(crate) fn place_desert_well(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    y: i32,
    z: i32,
) {
    let mut oy = y + 1;
    while region.get(x, oy, z).is_air() && oy > WORLD_BOTTOM + 2 {
        oy -= 1;
    }
    if region.get(x, oy, z) != BlockId::Sand {
        return;
    }
    for dx in -2..=2 {
        for dz in -2..=2 {
            if region.get(x + dx, oy - 1, z + dz).is_air()
                && region.get(x + dx, oy - 2, z + dz).is_air()
            {
                return;
            }
        }
    }
    for dy in -2..=0 {
        for dx in -2..=2 {
            for dz in -2..=2 {
                region.set(x + dx, oy + dy, z + dz, BlockId::Sandstone);
            }
        }
    }
    region.set(x, oy, z, BlockId::Water);
    for &(dx, dz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
        region.set(x + dx, oy, z + dz, BlockId::Water);
    }
    region.set(x, oy - 1, z, BlockId::Sand);
    for &(dx, dz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
        region.set(x + dx, oy - 1, z + dz, BlockId::Sand);
    }
    for dx in -2..=2 {
        for dz in -2..=2 {
            if dx == -2 || dx == 2 || dz == -2 || dz == 2 {
                region.set(x + dx, oy + 1, z + dz, BlockId::Sandstone);
            }
        }
    }
    region.set(x + 2, oy + 1, z, BlockId::SandstoneSlab);
    region.set(x - 2, oy + 1, z, BlockId::SandstoneSlab);
    region.set(x, oy + 1, z + 2, BlockId::SandstoneSlab);
    region.set(x, oy + 1, z - 2, BlockId::SandstoneSlab);
    for dx in -1..=1 {
        for dz in -1..=1 {
            if dx == 0 && dz == 0 {
                region.set(x + dx, oy + 4, z + dz, BlockId::Sandstone);
            } else {
                region.set(x + dx, oy + 4, z + dz, BlockId::SandstoneSlab);
            }
        }
    }
    for dy in 1..=3 {
        region.set(x - 1, oy + dy, z - 1, BlockId::Sandstone);
        region.set(x - 1, oy + dy, z + 1, BlockId::Sandstone);
        region.set(x + 1, oy + dy, z - 1, BlockId::Sandstone);
        region.set(x + 1, oy + dy, z + 1, BlockId::Sandstone);
    }
    let waters = [(x, oy, z), (x + 1, oy, z), (x, oy, z + 1), (x, oy, z - 1), (x - 1, oy, z)];
    let p1 = waters[rng.next_int(5) as usize];
    region.set(p1.0, p1.1 - 1, p1.2, BlockId::SuspiciousSand);
    let p2 = waters[rng.next_int(5) as usize];
    region.set(p2.0, p2.1 - 2, p2.2, BlockId::SuspiciousSand);
}

// ---------------------------------------------------------------------------
// freeze_top_layer
// ---------------------------------------------------------------------------

/// `SnowAndFreezeFeature.place` (26.2): 16×16 MOTION_BLOCKING columns.
///
/// Vanilla reads `level.getHeight(MOTION_BLOCKING, x, z)` = the FIRST
/// AVAILABLE y (the air cell above the top motion-blocking block) and probes
/// `topPos` / `topPos.below()`. [`heightmap_top`] returns the topmost opaque
/// y instead, so the column position is `top + 1`.
///
/// Both predicates route through `Biome.warmEnoughToRain` →
/// `getHeightAdjustedTemperature` (Biome.java:112-177), which applies the
/// biome's `temperature_modifier` and, ABOVE `seaLevel + 17` (the snow line),
/// subtracts a TEMPERATURE_NOISE term. Skipping that adjustment placed no
/// snow at all on the high cold biomes (jagged_peaks at y≈98-108 adjusts to
/// −0.7, well below the 0.15 rain threshold).
pub(crate) fn place_freeze_top_layer(
    region: &mut RegionBuf,
    state: &WorldgenState,
    x: i32,
    y: i32,
    z: i32,
) {
    let _ = y;
    for dx in 0..16 {
        for dz in 0..16 {
            let bx = x + dx;
            let bz = z + dz;
            let Some(top_y) = heightmap_top(region, bx, bz, HeightmapKind::MotionBlocking) else {
                continue;
            };
            let sy = top_y + 1;
            // Vanilla samples the biome AT topPos (level.getBiome(topPos)).
            let bid = crate::biome_manager::biome_id_at_block(state, bx, sy, bz);
            let name = crate::feature_dispatch::biome_id_to_name(bid);

            let below = sy - 1;
            // Biome.shouldFreeze(level, belowPos, false): water below and not
            // warm enough to rain. Block light < 10 is trivially true here.
            if !biome_warm_enough(state, bx, below, bz)
                && region.get(bx, below, bz) == BlockId::Water
            {
                region.set(bx, below, bz, BlockId::Ice);
            }

            // Biome.shouldSnow(topPos): precipitation==SNOW (has precipitation
            // && !warmEnoughToRain), (air | snow) on top, and
            // SnowLayerBlock.canSurvive (below must support it).
            let top = region.get(bx, sy, bz);
            if !biome_warm_enough(state, bx, sy, bz)
                && biome_has_precipitation(name)
                && (top.is_air() || top == BlockId::SnowLayer)
                && snow_layer_can_survive(region.get(bx, below, bz))
            {
                region.set(bx, sy, bz, BlockId::SnowLayer);
            }
        }
    }
}

/// `SnowLayerBlock.canSurvive` (SnowLayerBlock.java:77-86) for a fresh
/// (layers=1) snow layer: `cannot_support_snow_layer` → false,
/// `support_override_snow_layer` → true, else the block below must have a
/// full collision shape on its UP face.
///
/// Both tags verified against 26.2 (`ProbeSnowMotion`): ice/packed_ice/
/// barrier cannot support; honey_block/soul_sand/mud always can. Every block
/// in the tags has a distinct `BlockId` except `barrier` and `honey_block`,
/// which no overworld worldgen path produces (the lake config's "barrier" is
/// stone), so they need no ids. A snow layer never supports another layer
/// (its collision shape is EMPTY at layers=1) — `is_face_sturdy_full` already
/// excludes it.
fn snow_layer_can_survive(below: BlockId) -> bool {
    if matches!(below, BlockId::Ice | BlockId::PackedIce) {
        return false;
    }
    if matches!(below, BlockId::SoulSand | BlockId::Mud) {
        return true;
    }
    crate::multiface_spreader::is_face_sturdy_full(below)
}

fn biome_has_precipitation(name: &str) -> bool {
    crate::feature_catalog::biome_climate(name).1
}

/// `TEMPERATURE_NOISE` (Biome.java:62): seed 1234, octave set `[0]`.
fn temperature_noise() -> &'static crate::perlin_simplex::PerlinSimplexNoise {
    static N: std::sync::LazyLock<crate::perlin_simplex::PerlinSimplexNoise> =
        std::sync::LazyLock::new(|| {
            crate::perlin_simplex::PerlinSimplexNoise::new(1234, &[0])
        });
    &N
}

/// `Biome.getHeightAdjustedTemperature(pos, seaLevel)` (Biome.java:112-121).
fn height_adjusted_temperature(
    state: &WorldgenState,
    name: &str,
    x: i32,
    y: i32,
    z: i32,
) -> f32 {
    height_adjusted_temperature_at(name, state.sea_level, x, y, z)
}

/// Test seam: the snow-line math without a `WorldgenState`.
#[cfg(test)]
pub(crate) fn height_adjusted_temperature_for_test(
    name: &str,
    sea_level: i32,
    x: i32,
    y: i32,
    z: i32,
) -> f32 {
    height_adjusted_temperature_at(name, sea_level, x, y, z)
}

fn height_adjusted_temperature_at(
    name: &str,
    sea_level: i32,
    x: i32,
    y: i32,
    z: i32,
) -> f32 {
    let (base, _, frozen) = crate::feature_catalog::biome_climate(name);
    let adjusted = if frozen {
        // TemperatureModifier.FROZEN (Biome.java:395-409).
        let large = crate::surface_rules::frozen_temperature_noise()
            .get_value(x as f64 * 0.05, z as f64 * 0.05)
            * 7.0;
        let edge = crate::surface_rules::biome_info_noise()
            .get_value(x as f64 * 0.2, z as f64 * 0.2);
        if large + edge < 0.3 {
            let small = crate::surface_rules::biome_info_noise()
                .get_value(x as f64 * 0.09, z as f64 * 0.09);
            if small < 0.8 {
                0.2
            } else {
                base
            }
        } else {
            base
        }
    } else {
        base
    };
    let snow_level = sea_level + 17;
    if y > snow_level {
        // Java: float v = (float)(TEMPERATURE_NOISE.getValue(x/8.0F, z/8.0F,
        // false) * 8.0); then adjustedTemperature - (v + y - snowLevel) * 0.05F
        // / 40.0F — every step in f32.
        let v = (temperature_noise().get_value(x as f64 / 8.0, z as f64 / 8.0) * 8.0) as f32;
        adjusted - (v + (y - snow_level) as f32) * 0.05f32 / 40.0f32
    } else {
        adjusted
    }
}

/// `Biome.warmEnoughToRain(pos, seaLevel)`: adjusted temperature >= 0.15.
pub(crate) fn biome_warm_enough(state: &WorldgenState, x: i32, y: i32, z: i32) -> bool {
    let bid = crate::biome_manager::biome_id_at_block(state, x, y, z);
    let name = crate::feature_dispatch::biome_id_to_name(bid);
    height_adjusted_temperature(state, name, x, y, z) >= 0.15
}

// ---------------------------------------------------------------------------
// spike (ice_spike)
// ---------------------------------------------------------------------------

/// `SpikeFeature.place` (26.2).
pub(crate) fn place_spike(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    y: i32,
    z: i32,
    cfg: &Value,
) {
    let c = &cfg["config"];
    let state = c["state"]["Name"]
        .as_str()
        .and_then(BlockId::from_name)
        .unwrap_or(BlockId::PackedIce);
    let mut oy = y;
    while region.get(x, oy, z).is_air() && oy > WORLD_BOTTOM + 2 {
        oy -= 1;
    }
    if !eval_block_predicate(region, x, oy, z, &c["can_place_on"]) {
        return;
    }
    oy += rng.next_int(4);
    let height = rng.next_int(4) + 7;
    let mut width = height / 4 + rng.next_int(2);
    if width > 1 && rng.next_int(60) == 0 {
        oy += 10 + rng.next_int(30);
    }
    for y_off in 0..height {
        let scale = (1.0 - y_off as f32 / height as f32) * width as f32;
        let new_width = scale.ceil() as i32;
        for xo in -new_width..=new_width {
            let dx = xo.abs() as f32 - 0.25;
            for zo in -new_width..=new_width {
                let dz = zo.abs() as f32 - 0.25;
                let in_circle = (xo == 0 && zo == 0) || (dx * dx + dz * dz <= scale * scale);
                let edge = xo == -new_width
                    || xo == new_width
                    || zo == -new_width
                    || zo == new_width;
                if in_circle && (!edge || !(rng.next_f32() > 0.75)) {
                    let b = region.get(x + xo, oy + y_off, z + zo);
                    if b.is_air()
                        || eval_block_predicate(region, x + xo, oy + y_off, z + zo, &c["can_replace"])
                    {
                        region.set(x + xo, oy + y_off, z + zo, state);
                    }
                    if y_off != 0 && new_width > 1 {
                        let b2 = region.get(x + xo, oy - y_off, z + zo);
                        if b2.is_air()
                            || eval_block_predicate(region, x + xo, oy - y_off, z + zo, &c["can_replace"])
                        {
                            region.set(x + xo, oy - y_off, z + zo, state);
                        }
                    }
                }
            }
        }
    }
    let mut pillar_width = width - 1;
    if pillar_width < 0 {
        pillar_width = 0;
    } else if pillar_width > 1 {
        pillar_width = 1;
    }
    for xo in -pillar_width..=pillar_width {
        for zo in -pillar_width..=pillar_width {
            let mut cy = oy - 1;
            let mut run_length = 50;
            if xo.abs() == 1 && zo.abs() == 1 {
                run_length = rng.next_int(5);
            }
            while cy > 50 {
                let b = region.get(x + xo, cy, z + zo);
                if !(b.is_air()
                    || eval_block_predicate(region, x + xo, cy, z + zo, &c["can_replace"]))
                    && b != state
                {
                    break;
                }
                region.set(x + xo, cy, z + zo, state);
                cy -= 1;
                run_length -= 1;
                if run_length <= 0 {
                    cy -= rng.next_int(5) + 1;
                    run_length = rng.next_int(5);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// bamboo
// ---------------------------------------------------------------------------

/// `BambooFeature.place` (26.2). Block states collapse to `BlockId::Bamboo`
/// (age/leaves/stage are palette properties only).
pub(crate) fn place_bamboo(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    y: i32,
    z: i32,
    cfg: &Value,
) {
    let prob = cfg["config"]["probability"].as_f64().unwrap_or(0.0);
    // BambooStalkBlock.canSurvive: the block below must support bamboo
    // (approx: not air — matches vanilla behavior on the ground).
    if !region.get(x, y, z).is_air() || region.get(x, y - 1, z).is_air() {
        return;
    }
    let height = rng.next_int(12) + 5;
    if rng.next_f32() < prob as f32 {
        let r = rng.next_int(4) + 1;
        for xx in (x - r)..=(x + r) {
            for zz in (z - r)..=(z + r) {
                let xd = xx - x;
                let zd = zz - z;
                if xd * xd + zd * zd <= r * r {
                    if let Some(sy) = heightmap_top(region, xx, zz, HeightmapKind::WorldSurface) {
                        let py = sy - 1;
                        if is_in_tag(region.get(xx, py, zz), "#minecraft:beneath_bamboo_podzol_replaceable")
                        {
                            region.set(xx, py, zz, BlockId::Podzol);
                        }
                    }
                }
            }
        }
    }
    let mut by = y;
    for _ in 0..height {
        if !region.get(x, by, z).is_air() {
            break;
        }
        region.set(x, by, z, BlockId::Bamboo);
        by += 1;
    }
    if by - y >= 3 {
        region.set(x, by, z, BlockId::Bamboo);
        region.set(x, by - 1, z, BlockId::Bamboo);
        region.set(x, by - 2, z, BlockId::Bamboo);
    }
}

// ---------------------------------------------------------------------------
// monster_room
// ---------------------------------------------------------------------------

/// `MonsterRoomFeature.place` (26.2). Chest/spawner loot entities are not
/// modelled (block parity only).
pub(crate) fn place_monster_room(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    y: i32,
    z: i32,
) {
    // NEUTRON_MONSTER_TRACE=1 — attempt/verdict trace (diagnostic).
    static TRACE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let trace = *TRACE.get_or_init(|| std::env::var_os("NEUTRON_MONSTER_TRACE").is_some());
    let xr = rng.next_int(2) + 2;
    let zr = rng.next_int(2) + 2;
    let min_x = -xr - 1;
    let max_x = xr + 1;
    let min_z = -zr - 1;
    let max_z = zr + 1;
    let mut hole_count = 0;
    for dx in min_x..=max_x {
        for dy in -1..=4 {
            for dz in min_z..=max_z {
                let solid_b = blocks_motion(region.get(x + dx, y + dy, z + dz));
                if dy == -1 && !solid_b {
                    if trace {
                        eprintln!("MONSTER {x} {y} {z} reject floor_air");
                    }
                    return;
                }
                if dy == 4 && !solid_b {
                    if trace {
                        eprintln!("MONSTER {x} {y} {z} reject roof_air");
                    }
                    return;
                }
                if (dx == min_x || dx == max_x || dz == min_z || dz == max_z)
                    && dy == 0
                    && region.get(x + dx, y + dy, z + dz).is_air()
                    && region.get(x + dx, y + dy + 1, z + dz).is_air()
                {
                    hole_count += 1;
                }
            }
        }
    }
    if !(1..=5).contains(&hole_count) {
        if trace {
            eprintln!("MONSTER {x} {y} {z} reject holes={hole_count}");
        }
        return;
    }
    if trace {
        eprintln!("MONSTER {x} {y} {z} ACCEPT holes={hole_count} xr={xr} zr={zr}");
    }
    for dx in min_x..=max_x {
        // Vanilla: `for (int dy = 3; dy >= -1; dy--)`. NOTE: `(3..=-1)` is an
        // EMPTY RangeInclusive in Rust, so the descending form must be written
        // as the ascending range reversed.
        for dy in (-1..=3).rev() {
            for dz in min_z..=max_z {
                let is_wall = dx == min_x
                    || dy == -1
                    || dz == min_z
                    || dx == max_x
                    || dy == 4
                    || dz == max_z;
                if is_wall {
                    if y + dy >= WORLD_BOTTOM && !blocks_motion(region.get(x + dx, y + dy - 1, z + dz))
                    {
                        region.set(x + dx, y + dy, z + dz, BlockId::CaveAir);
                    } else {
                        let ws = region.get(x + dx, y + dy, z + dz);
                        if blocks_motion(ws) && ws != BlockId::Chest {
                            if dy == -1 && rng.next_int(4) != 0 {
                                region.set(x + dx, y + dy, z + dz, BlockId::MossyCobblestone);
                            } else {
                                region.set(x + dx, y + dy, z + dz, BlockId::Cobblestone);
                            }
                        }
                    }
                } else {
                    let ws = region.get(x + dx, y + dy, z + dz);
                    if ws != BlockId::Chest && ws != BlockId::Spawner {
                        region.set(x + dx, y + dy, z + dz, BlockId::CaveAir);
                    }
                }
            }
        }
    }
    'chest: for _ in 0..2 {
        for _ in 0..3 {
            let xc = x + rng.next_int(xr * 2 + 1) - xr;
            let zc = z + rng.next_int(zr * 2 + 1) - zr;
            if region.get(xc, y, zc).is_air() {
                let mut wall_count = 0;
                for &(dx, dz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if blocks_motion(region.get(xc + dx, y, zc + dz)) {
                        wall_count += 1;
                    }
                }
                if wall_count == 1 {
                    region.set(xc, y, zc, BlockId::Chest);
                    break 'chest;
                }
            }
        }
    }
    region.set(x, y, z, BlockId::Spawner);
}

#[cfg(test)]
mod snow_tests {
    use super::*;

    /// Two-sided check vs `Biome.getHeightAdjustedTemperature` (real 26.2 jar,
    /// `ProbeSnowTemp`): the snow-line term above `seaLevel + 17` is what makes
    /// `warmEnoughToRain` false on the high cold biomes. Regression guard for
    /// the bug where `biome_climate` returned its `(0.5, true)` fallback for
    /// every biome (no `biome/*` entry in `datapack_data`), so `warm_enough`
    /// was true everywhere and `freeze_top_layer` placed no snow at all.
    #[test]
    fn height_adjusted_temperature_matches_vanilla() {
        // (biome, x, y, z, vanilla float)
        let cases: [(&str, i32, i32, i32, f32); 8] = [
            ("jagged_peaks", 144, 98, 64, -0.7220023),
            ("jagged_peaks", 150, 100, 64, -0.71830904),
            ("jagged_peaks", 150, 106, 70, -0.73637086),
            ("jagged_peaks", 144, 80, 64, -0.7),
            ("snowy_slopes", 0, 120, 0, -0.35000002),
            ("grove", 100, 140, 100, -0.27933648),
            ("frozen_peaks", -200, 130, -200, -0.762238),
            ("plains", 0, 70, 0, 0.8),
        ];
        for (biome, x, y, z, want) in cases {
            let got = height_adjusted_temperature_for_test(biome, 63, x, y, z);
            assert!(
                (got - want).abs() < 1e-6,
                "{biome} ({x},{y},{z}): adjusted temp = {got}, want {want}"
            );
        }
    }

    /// The raw biome temperature must come from the biome JSON, not a default.
    #[test]
    fn biome_climate_reads_real_temperatures() {
        for (name, want) in [("jagged_peaks", -0.7f32), ("frozen_peaks", -0.7), ("plains", 0.8)] {
            let (t, precip, _) = crate::feature_catalog::biome_climate(name);
            assert_eq!(t, want, "{name} temperature");
            assert!(precip, "{name} has_precipitation");
        }
        // frozen_ocean carries the FROZEN temperature modifier.
        assert!(crate::feature_catalog::biome_climate("frozen_ocean").2);
        assert!(!crate::feature_catalog::biome_climate("jagged_peaks").2);
    }

    /// A snow layer is NOT `blocksMotion` and has no face-full shape, so it can
    /// never support another layer — while snow_block is a full opaque cube.
    #[test]
    fn snow_layer_and_snow_block_differ() {
        use crate::feature_dispatch::predicates::blocks_motion;
        use crate::surface::BlockId;
        assert!(!blocks_motion(BlockId::SnowLayer));
        assert!(blocks_motion(BlockId::Snow));
        assert!(!snow_layer_can_survive(BlockId::SnowLayer));
        assert!(snow_layer_can_survive(BlockId::Snow));
        // cannot_support_snow_layer / support_override_snow_layer.
        assert!(!snow_layer_can_survive(BlockId::Ice));
        assert!(!snow_layer_can_survive(BlockId::PackedIce));
        assert!(snow_layer_can_survive(BlockId::Mud));
        assert!(snow_layer_can_survive(BlockId::SoulSand));
        assert!(!snow_layer_can_survive(BlockId::Water));
        assert!(!snow_layer_can_survive(BlockId::Air));
    }
}

#[cfg(test)]
mod monster_room_tests {
    use super::*;

    /// `MonsterRoomFeature.place` must build the full room: cobblestone/
    /// mossy_cobblestone walls on the `xr`/`zr` ring, a solid floor at
    /// `dy == -1`, a solid roof at `dy == 4`, cave_air interior, and a
    /// spawner at the origin. The wall loop is
    /// `for (int dy = 3; dy >= -1; dy--)` — a `(3..=-1).rev()` port is an
    /// EMPTY `RangeInclusive` in Rust and silently built nothing (no walls, no
    /// interior, no spawner), which is what let the ref's dungeons diverge.
    #[test]
    fn builds_walls_interior_and_spawner() {
        let mut region = RegionBuf::new(0, 0, 0);
        let seed = 12345i64;
        // Learn the room half-extents the way the feature will draw them, so
        // the single carved pocket lands exactly on the wall ring (dx=min_x).
        let mut probe = FeatureRandom::new(seed);
        let xr = probe.next_int(2) + 2;
        let min_x = -xr - 1;
        let oy = 64;
        // Solid box covering dy -1..=4 (y oy-1 ..= oy+4) with margin.
        for y in oy - 1..=oy + 4 {
            for z in 1..=13 {
                for x in 1..=13 {
                    region.set(x, y, z, BlockId::Deepslate);
                }
            }
        }
        // One 2-tall pocket on the ring at dy=0 → hole count 1 (vanilla's
        // accepted window is 1..=5).
        region.set(7 + min_x, oy, 7, BlockId::Air);
        region.set(7 + min_x, oy + 1, 7, BlockId::Air);

        let mut rng = FeatureRandom::new(seed);
        place_monster_room(&mut rng, &mut region, 7, oy, 7);

        assert_eq!(region.get(7, oy, 7), BlockId::Spawner, "spawner at origin");
        let mut walls = 0u32;
        let mut interior = 0u32;
        for dy in -1..=4i32 {
            for dz in 1..=13i32 {
                for dx in 1..=13i32 {
                    let b = region.get(dx, oy + dy, dz);
                    if matches!(b, BlockId::Cobblestone | BlockId::MossyCobblestone) {
                        walls += 1;
                    }
                    if dy == 0 && b == BlockId::CaveAir {
                        interior += 1;
                    }
                }
            }
        }
        assert!(walls > 20, "room walls placed (got {walls})");
        assert!(interior > 10, "room interior carved (got {interior})");
    }
}
