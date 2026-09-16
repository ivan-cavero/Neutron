//! Surface rules: `SurfaceSystem.buildSurface` + datapack `surface_rule` JSON.
//!
//! Copyright (c) 2026 Neutron Contributors -- MIT License

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::Value;

use crate::biome_source::biome_id;
use crate::density::DensityEnv;
use crate::generator::{WORLD_BOTTOM, WORLD_TOP};
use crate::noise::NormalNoise;
use crate::positional::PositionalRandomFactory;
use crate::surface::BlockId;
use crate::worldgen::WorldgenState;

/// Apply the overworld surface_rule tree to a filled stone/fluid chunk.
pub fn apply_surface_rules(
    blocks: &mut [u16],
    heightmap: &[i16],
    cx: i32,
    cz: i32,
    st: &WorldgenState,
) {
    let rule = overworld_rule();
    let chunk_min_x = cx * 16;
    let chunk_min_z = cz * 16;
    let main_rng = PositionalRandomFactory::new(st.main_lo, st.main_hi);

    for lx in 0..16usize {
        for lz in 0..16usize {
            let world_x = chunk_min_x + lx as i32;
            let world_z = chunk_min_z + lz as i32;
            let surface_y = heightmap[lz * 16 + lx] as i32;
            if surface_y < WORLD_BOTTOM {
                continue;
            }

            // Vanilla `height` in buildSurface is WORLD_SURFACE_WG + 1, which
            // INCLUDES fluids (SurfaceSystem.java:112,119). The caller's
            // heightmap is fluid-exclusive, so recompute the fluid-inclusive
            // top here: without it the y-loop starts below the water column,
            // water_height stays MIN at deep floors, the Water(-6) condition
            // returns true (vanilla treats MIN as exposed), and sediment
            // (dirt/sand) overwrites the stone floor vanilla leaves intact
            // (1.2M `stone->dirt` ledger cells across the 30-seed gate).
            let mut surface_y_fluid = surface_y;
            for y in (surface_y..WORLD_TOP).rev() {
                let b = BlockId::from_u16(
                    blocks[block_index(lx, y, lz)],
                )
                .unwrap_or(BlockId::Air);
                if !b.is_air() {
                    surface_y_fluid = y;
                    break;
                }
            }
            let surface_y = surface_y_fluid;
            if surface_y < WORLD_BOTTOM {
                continue;
            }

            // surfaceDepth = (int)(surfaceNoise*2.75 + 3.0 + random.at(x,0,z).nextDouble()*0.25)
            let surface_noise = st
                .noises
                .noises()
                .get("surface")
                .map(|n| n.get_value(world_x as f64, 0.0, world_z as f64))
                .unwrap_or(0.0);
            let mut depth_rng = main_rng.at(world_x, 0, world_z);
            let surface_depth = (surface_noise * 2.75 + 3.0 + depth_rng.next_f64() * 0.25) as i32;

            let surface_secondary = st
                .noises
                .noises()
                .get("surface_secondary")
                .map(|n| n.get_value(world_x as f64, 0.0, world_z as f64))
                .unwrap_or(0.0);

            // SurfaceRules.Context.getMinSurfaceLevel: bilinear lerp of the
            // 16x16 surface-cell corner preliminary surface levels. Each corner
            // is the preliminary_surface_level density function evaluated at
            // the quart-quantized corner block (NoiseChunk.preliminarySurfaceLevel:
            // QuartPos.toBlock(fromBlock(x)) = x & !3), floored and cached.
            // lerp alphas are f32 ((x & 15) / 16.0F); result floored, then
            // + surfaceDepth - 8.
            let corner_prelim = |cx: i32, cz: i32| -> i32 {
                let mut env = DensityEnv::new(cx & !3, 0, cz & !3, st.noises.noises());
                let v = crate::density::compute(&st.router.preliminary_surface_level, &mut env);
                v.floor() as i32
            };
            let cell_x = world_x >> 4;
            let cell_z = world_z >> 4;
            let tx = (world_x & 15) as f32 / 16.0f32;
            let tz = (world_z & 15) as f32 / 16.0f32;
            let p00 = corner_prelim(cell_x << 4, cell_z << 4) as f64;
            let p10 = corner_prelim((cell_x + 1) << 4, cell_z << 4) as f64;
            let p01 = corner_prelim(cell_x << 4, (cell_z + 1) << 4) as f64;
            let p11 = corner_prelim((cell_x + 1) << 4, (cell_z + 1) << 4) as f64;
            let lerp = |a: f64, b: f64, c: f64| b + a * (c - b);
            let prelim = lerp(
                tz as f64,
                lerp(tx as f64, p00, p10),
                lerp(tx as f64, p01, p11),
            );
            let min_surface_level = (prelim.floor() as i32) + surface_depth - 8;


            // Steep: neighbour height delta >= 4 (within chunk)
            let steep = is_steep(heightmap, lx, lz);

            let mut stone_above = 0i32;
            let mut water_height = i32::MIN;
            let mut next_ceiling_stone_y = i32::MAX;
            let end_y = WORLD_BOTTOM;
            let height = surface_y + 1;
            // Cache last cave-biome sample to avoid density eval per block.

            for y in (end_y..=height.min(WORLD_TOP - 1)).rev() {
                let idx = block_index(lx, y, lz);
                let old = BlockId::from_u16(blocks[idx]).unwrap_or(BlockId::Air);

                if old.is_air() {
                    stone_above = 0;
                    water_height = i32::MIN;
                    continue;
                }
                if old.is_fluid() {
                    if water_height == i32::MIN {
                        water_height = y + 1;
                    }
                    continue;
                }

                // stone run — find how deep we are above the next non-stone below
                if next_ceiling_stone_y >= y {
                    next_ceiling_stone_y = WORLD_BOTTOM;
                    let mut look = y - 1;
                    while look >= end_y - 1 {
                        if look < WORLD_BOTTOM {
                            next_ceiling_stone_y = WORLD_BOTTOM;
                            break;
                        }
                        let b = BlockId::from_u16(blocks[block_index(lx, look, lz)])
                            .unwrap_or(BlockId::Air);
                        if !is_stone_like(b) {
                            next_ceiling_stone_y = look + 1;
                            break;
                        }
                        look -= 1;
                    }
                }
                let stone_below = y - next_ceiling_stone_y + 1;
                stone_above += 1;

                // Only replace default stone (and deepslate pre-surface for deepslate rule)
                if old != BlockId::Stone && old != BlockId::Deepslate {
                    continue;
                }

                // Vanilla SurfaceSystem evaluates context.biome (BiomeManager.getBiome)
                // PER BLOCK — no surface shortcut. (The previous 8-block cache and the
                // min_surface_level-16 surface shortcut both desynced biome-gated rules:
                // on seed 777 sulfur_caves bands at y 27-45 sat above min_surf-16, got the
                // surface biome, and the sulfur/cinnabar rule never fired — a 190k-cell
                // sulfur-family gap.)
                let biome = sample_biome(st, world_x, y, world_z);

                let mut ctx = RuleContext {
                    x: world_x,
                    y,
                    z: world_z,
                    stone_depth_above: stone_above,
                    stone_depth_below: stone_below,
                    water_height,
                    surface_depth,
                    surface_secondary,
                    min_surface_level,
                    biome,
                    steep,
                    hole: surface_depth <= 0,
                    noises: st.noises.noises(),
                    main_rng,
                    sea_level: st.sea_level,
                };
                if let Some(new_block) = rule.try_apply(&mut ctx) {
                    blocks[idx] = new_block.as_u16();
                }
            }

            // SurfaceSystem.frozenOceanExtension (26.2, SurfaceSystem.java:160,235):
            // runs AFTER the surface rule for the column, gated on the surface
            // biome being frozen_ocean / deep_frozen_ocean.
            let surface_biome = sample_biome(st, world_x, surface_y.max(WORLD_BOTTOM), world_z);
            if matches!(surface_biome, biome_id::FROZEN_OCEAN | biome_id::DEEP_FROZEN_OCEAN) {
                frozen_ocean_extension(
                    st,
                    blocks,
                    min_surface_level,
                    surface_biome,
                    world_x,
                    world_z,
                    height,
                );
            }
        }
    }
}

/// `SurfaceSystem.frozenOceanExtension` (26.2, SurfaceSystem.java:235-284):
/// paints snow_block / packed_ice berg columns over frozen-ocean water using
/// the three iceberg NormalNoises (seeded per world via NoiseSet like every
/// other registered noise). `noise_random` = per-column legacy random at
/// (x, 0, z) — vanilla `this.noiseRandom.at(blockX, 0, blockZ)` where
/// noiseRandom = WorldgenRandom(LegacyRandomSource(seed)).
fn frozen_ocean_extension(
    st: &WorldgenState,
    blocks: &mut [u16],
    min_surface_level: i32,
    surface_biome: u8,
    block_x: i32,
    block_z: i32,
    height: i32,
) {
    let sea_level = st.sea_level;
    let noises = st.noises.noises();
    let (Some(surface_n), Some(pillar_n), Some(roof_n)) = (
        noises.get("iceberg_surface"),
        noises.get("iceberg_pillar"),
        noises.get("iceberg_pillar_roof"),
    ) else {
        return;
    };

    let pillar_scale = 1.28f64;
    let iceberg = (surface_n.get_value(block_x as f64, 0.0, block_z as f64) * 8.25)
        .abs()
        .min(
            pillar_n.get_value(
                block_x as f64 * pillar_scale,
                0.0,
                block_z as f64 * pillar_scale,
            ) * 15.0,
        );
    if iceberg <= 1.8 {
        return;
    }

    let roof_scale = 1.17f64;
    let iceberg_roof = (roof_n.get_value(
        block_x as f64 * roof_scale,
        0.0,
        block_z as f64 * roof_scale,
    ) * 1.5)
        .abs();
    let mut top = (iceberg * iceberg * 1.2).min(iceberg_roof.ceil() + 14.0);
    if surface_should_melt_frozen_iceberg_slightly(st, surface_biome, block_x, block_z, sea_level) {
        top -= 2.0;
    }

    let extension_bottom;
    let extension_top;
    if top > 2.0 {
        extension_bottom = sea_level as f64 - top - 7.0;
        top += sea_level as f64;
        extension_top = top;
    } else {
        extension_top = 0.0;
        extension_bottom = 0.0;
    }

    // noiseRandom = WorldgenRandom(LegacyRandomSource(levelSeed)) shared by the
    // SurfaceSystem; `.at(x, 0, z)` per column. Neutron: PositionalRandomFactory
    // over the main seed pair (main_lo/main_hi), matching the surface RNG used
    // for surface_depth.
    let mut random = PositionalRandomFactory::new(st.main_lo, st.main_hi).at(block_x, 0, block_z);
    let max_snow_depth = 2 + random.next_int(4);
    let min_snow_height = sea_level + 18 + random.next_int(10);
    let mut snow_depth = 0;

    let y_start = (height).max(extension_top as i32 + 1);
    let mut y = y_start;
    while y >= min_surface_level {
        let b = BlockId::from_u16(blocks[block_index(
            (block_x.rem_euclid(16)) as usize,
            y,
            (block_z.rem_euclid(16)) as usize,
        )])
        .unwrap_or(BlockId::Air);
        let in_water_band = b == BlockId::Water
            && y > extension_bottom as i32
            && y < sea_level
            && extension_bottom != 0.0;
        let in_air_above_top = b.is_air() && y < extension_top as i32;
        if (in_air_above_top && random.next_f64() > 0.01)
            || (in_water_band && random.next_f64() > 0.15)
        {
            if snow_depth <= max_snow_depth && y > min_snow_height {
                blocks[block_index(
                    (block_x.rem_euclid(16)) as usize,
                    y,
                    (block_z.rem_euclid(16)) as usize,
                )] = BlockId::Snow.as_u16();
                snow_depth += 1;
            } else {
                blocks[block_index(
                    (block_x.rem_euclid(16)) as usize,
                    y,
                    (block_z.rem_euclid(16)) as usize,
                )] = BlockId::PackedIce.as_u16();
            }
        }
        y -= 1;
    }
}

/// `Biome.FROZEN_TEMPERATURE_NOISE` (Biome.java:62): seed 3456, octaves
/// `[-2, -1, 0]`.
pub(crate) fn frozen_temperature_noise() -> &'static crate::perlin_simplex::PerlinSimplexNoise {
    static N: std::sync::LazyLock<crate::perlin_simplex::PerlinSimplexNoise> =
        std::sync::LazyLock::new(|| {
            crate::perlin_simplex::PerlinSimplexNoise::new(3456, &[-2, -1, 0])
        });
    &N
}

/// `Biome.BIOME_INFO_NOISE` (Biome.java:66): seed 2345, octave `[0]`.
pub(crate) fn biome_info_noise() -> &'static crate::perlin_simplex::PerlinSimplexNoise {
    static N: std::sync::LazyLock<crate::perlin_simplex::PerlinSimplexNoise> =
        std::sync::LazyLock::new(|| crate::perlin_simplex::PerlinSimplexNoise::new(2345, &[0]));
    &N
}

/// `Biome.shouldMeltFrozenOceanIcebergSlightly` = getTemperature > 0.1.
/// getTemperature = getHeightAdjustedTemperature (cache elided — pure fn):
///   adjusted = FROZEN modifier (see below) applied to base temperature
///   if y > seaLevel + 17: adjusted -= (TEMPERATURE_NOISE(x/8, z/8)*8 + y -
///                             (seaLevel+17)) * 0.05 / 40
/// FROZEN modifier (frozen_ocean base 0.0, deep_frozen_ocean base 0.5):
///   large = FROZEN_TEMPERATURE_NOISE(x*0.05, z*0.05)*7
///   edge  = BIOME_INFO_NOISE(x*0.2, z*0.2)
///   if large + edge < 0.3 && BIOME_INFO_NOISE(x*0.09, z*0.09) < 0.8: 0.2
///   else base
/// All three PerlinSimplexNoise instances are world-independent (fixed
/// LegacyRandomSource seeds 3456 / 2345).
fn surface_should_melt_frozen_iceberg_slightly(
    st: &WorldgenState,
    surface_biome: u8,
    x: i32,
    z: i32,
    sea_level: i32,
) -> bool {
    let base = match surface_biome {
        biome_id::DEEP_FROZEN_OCEAN => 0.5f32,
        _ => 0.0f32,
    };
    let mut adjusted = base;
    // FROZEN modifier (both frozen ocean biomes use it)
    let large = frozen_temperature_noise().get_value(x as f64 * 0.05, z as f64 * 0.05) * 7.0;
    let edge = biome_info_noise().get_value(x as f64 * 0.2, z as f64 * 0.2);
    if large + edge < 0.3 {
        let small = biome_info_noise().get_value(x as f64 * 0.09, z as f64 * 0.09);
        if small < 0.8 {
            adjusted = 0.2;
        }
    }
    // getHeightAdjustedTemperature uses the BLOCK y; the iceberg check samples at
    // y = seaLevel (blockPos.set(x, seaLevel, z)), so y = seaLevel <= snowLevel —
    // the height adjustment (TEMPERATURE_NOISE, seed 1234) never applies here.
    let _ = sea_level;
    adjusted > 0.1
}

fn is_steep(heightmap: &[i16], lx: usize, lz: usize) -> bool {
    // SurfaceRules.Context.SteepMaterialCondition: WORLD_SURFACE_WG heightmap,
    // ONE-directional — south >= north + 4, else west >= east + 4. Chunk-edge
    // neighbors clamp to the block's own row (equal heights -> false).
    let hn = heightmap[lz.saturating_sub(1) * 16 + lx] as i32;
    let hs = heightmap[(lz + 1).min(15) * 16 + lx] as i32;
    if hs >= hn + 4 {
        return true;
    }
    let hw = heightmap[lz * 16 + lx.saturating_sub(1)] as i32;
    let he = heightmap[lz * 16 + (lx + 1).min(15)] as i32;
    hw >= he + 4
}

fn is_stone_like(b: BlockId) -> bool {
    !b.is_air() && !b.is_fluid()
}

fn sample_biome(st: &WorldgenState, x: i32, y: i32, z: i32) -> u8 {
    // SurfaceSystem.buildSurface uses `biomeManager::getBiome` (4-block voronoi).
    crate::biome_manager::biome_id_at_block(st, x, y, z)
}

#[inline]
fn block_index(lx: usize, y: i32, lz: usize) -> usize {
    ((y - WORLD_BOTTOM) as usize) * 256 + lz * 16 + lx
}

// ---------------------------------------------------------------------------
// Rule tree
// ---------------------------------------------------------------------------

struct RuleContext<'a> {
    x: i32,
    y: i32,
    z: i32,
    stone_depth_above: i32,
    stone_depth_below: i32,
    water_height: i32,
    surface_depth: i32,
    surface_secondary: f64,
    min_surface_level: i32,
    biome: u8,
    steep: bool,
    hole: bool,
    noises: &'a HashMap<String, NormalNoise>,
    main_rng: PositionalRandomFactory,
    sea_level: i32,
}

enum Rule {
    Sequence(Vec<Rule>),
    Condition { cond: Condition, then: Box<Rule> },
    Block(BlockId),
    Bandlands,
}

enum Condition {
    VerticalGradient {
        name: String,
        true_at: VerticalAnchor,
        false_at: VerticalAnchor,
    },
    AbovePreliminarySurface,
    StoneDepth {
        offset: i32,
        add_surface_depth: bool,
        secondary_depth_range: i32,
        ceiling: bool,
    },
    Biome(Vec<u8>),
    YAbove {
        anchor: VerticalAnchor,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    Water {
        offset: i32,
        surface_depth_multiplier: i32,
        add_stone_depth: bool,
    },
    NoiseThreshold {
        noise: String,
        min: f64,
        max: f64,
    },
    Not(Box<Condition>),
    Hole,
    Steep,
    Temperature,
}

#[derive(Clone, Copy)]
enum VerticalAnchor {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

impl VerticalAnchor {
    fn resolve(self) -> i32 {
        match self {
            VerticalAnchor::Absolute(y) => y,
            VerticalAnchor::AboveBottom(n) => WORLD_BOTTOM + n,
            VerticalAnchor::BelowTop(n) => WORLD_TOP - 1 - n,
        }
    }
}

impl Rule {
    fn try_apply(&self, ctx: &mut RuleContext<'_>) -> Option<BlockId> {
        match self {
            Rule::Sequence(rules) => {
                for r in rules {
                    if let Some(b) = r.try_apply(ctx) {
                        return Some(b);
                    }
                }
                None
            }
            Rule::Condition { cond, then } => {
                if cond.test(ctx) {
                    then.try_apply(ctx)
                } else {
                    None
                }
            }
            Rule::Block(b) => Some(*b),
            Rule::Bandlands => Some(bandlands_block(ctx)),
        }
    }
}

impl Condition {
    fn test(&self, ctx: &mut RuleContext<'_>) -> bool {
        match self {
            Condition::VerticalGradient {
                name,
                true_at,
                false_at,
            } => {
                let t = true_at.resolve();
                let f = false_at.resolve();
                let y = ctx.y;
                if y <= t {
                    return true;
                }
                if y >= f {
                    return false;
                }
                // map(y, t, f, 1.0, 0.0)
                let threshold = map(y as f64, t as f64, f as f64, 1.0, 0.0);
                let factory = ctx.main_rng.from_hash_of_positional(name);
                let mut rng = factory.at(ctx.x, y, ctx.z);
                (rng.next_f32() as f64) < threshold
            }
            Condition::AbovePreliminarySurface => ctx.y >= ctx.min_surface_level,
            Condition::StoneDepth {
                offset,
                add_surface_depth,
                secondary_depth_range,
                ceiling,
            } => {
                let depth = if *ceiling {
                    ctx.stone_depth_below
                } else {
                    ctx.stone_depth_above
                };
                let surface_addon = if *add_surface_depth {
                    ctx.surface_depth
                } else {
                    0
                };
                let secondary = if *secondary_depth_range == 0 {
                    0
                } else {
                    map(
                        ctx.surface_secondary,
                        -1.0,
                        1.0,
                        0.0,
                        *secondary_depth_range as f64,
                    ) as i32
                };
                depth <= 1 + *offset + surface_addon + secondary
            }
            Condition::Biome(list) => list.contains(&ctx.biome),
            Condition::YAbove {
                anchor,
                surface_depth_multiplier,
                add_stone_depth,
            } => {
                let mut threshold = anchor.resolve();
                threshold += ctx.surface_depth * *surface_depth_multiplier;
                if *add_stone_depth {
                    threshold += ctx.stone_depth_above;
                }
                ctx.y >= threshold
            }
            Condition::Water {
                offset,
                surface_depth_multiplier,
                add_stone_depth,
            } => {
                if ctx.water_height == i32::MIN {
                    return true;
                }
                let stone_addon = if *add_stone_depth {
                    ctx.stone_depth_above
                } else {
                    0
                };
                ctx.y + stone_addon
                    >= ctx.water_height + *offset + ctx.surface_depth * *surface_depth_multiplier
            }
            Condition::NoiseThreshold { noise, min, max } => {
                // Most surface noises are 2D (y=0); a few are 3D (calcite, gravel, …).
                let y_sample = match noise.as_str() {
                    "surface" | "surface_secondary" | "surface_swamp" | "badlands_surface" => 0.0,
                    _ => ctx.y as f64,
                };
                let v = ctx
                    .noises
                    .get(noise.as_str())
                    .map(|n| n.get_value(ctx.x as f64, y_sample, ctx.z as f64))
                    .unwrap_or(0.0);
                // SurfaceRules.NoiseThresholdCondition.test: >= min && <= max
                v >= *min && v <= *max
            }
            Condition::Not(inner) => !inner.test(ctx),
            Condition::Hole => ctx.hole,
            Condition::Steep => ctx.steep,
            Condition::Temperature => {
                // Frozen ocean ice vs water: approx cold if biome frozen
                matches!(
                    ctx.biome,
                    biome_id::FROZEN_OCEAN
                        | biome_id::SNOWY_PLAINS
                        | biome_id::ICE_SPIKES
                        | biome_id::FROZEN_PEAKS
                        | biome_id::SNOWY_SLOPES
                        | biome_id::GROVE
                        | biome_id::JAGGED_PEAKS
                )
            }
        }
    }
}

#[inline]
fn map(v: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    if (from_max - from_min).abs() < 1e-12 {
        return to_min;
    }
    let t = (v - from_min) / (from_max - from_min);
    to_min + t * (to_max - to_min)
}

/// Simplified badlands terracotta banding by Y.
fn bandlands_block(ctx: &RuleContext<'_>) -> BlockId {
    // Vanilla uses clay_bands array; approximate with Y-mod bands.
    match ctx.y.rem_euclid(16) {
        0..=1 => BlockId::YellowTerracotta,
        2..=3 => BlockId::BrownTerracotta,
        4..=5 => BlockId::OrangeTerracotta,
        6..=7 => BlockId::RedTerracotta,
        8..=9 => BlockId::WhiteTerracotta,
        10..=11 => BlockId::LightGrayTerracotta,
        12..=13 => BlockId::Terracotta,
        _ => BlockId::OrangeTerracotta,
    }
}

// ---------------------------------------------------------------------------
// JSON parsing
// ---------------------------------------------------------------------------

fn overworld_rule() -> &'static Rule {
    static RULE: OnceLock<Rule> = OnceLock::new();
    RULE.get_or_init(|| {
        let json = crate::datapack_data::datapack_json("noise_settings_overworld.json")
            .expect("noise_settings_overworld.json");
        let value: Value = serde_json::from_str(json).expect("parse noise_settings");
        parse_rule(&value["surface_rule"])
    })
}

fn parse_rule(v: &Value) -> Rule {
    let t = v
        .get("type")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .strip_prefix("minecraft:")
        .unwrap_or_else(|| v.get("type").and_then(|x| x.as_str()).unwrap_or(""));
    match t {
        "sequence" => {
            let seq = v["sequence"]
                .as_array()
                .unwrap_or(&Vec::new())
                .iter()
                .map(parse_rule)
                .collect();
            Rule::Sequence(seq)
        }
        "condition" => Rule::Condition {
            cond: parse_condition(&v["if_true"]),
            then: Box::new(parse_rule(&v["then_run"])),
        },
        "block" => {
            let name = v["result_state"]["Name"]
                .as_str()
                .unwrap_or("minecraft:stone");
            Rule::Block(BlockId::from_name(name).unwrap_or(BlockId::Stone))
        }
        "bandlands" => Rule::Bandlands,
        _ => Rule::Sequence(vec![]),
    }
}

fn parse_condition(v: &Value) -> Condition {
    let t = v
        .get("type")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .strip_prefix("minecraft:")
        .unwrap_or("");
    match t {
        "vertical_gradient" => Condition::VerticalGradient {
            name: v["random_name"]
                .as_str()
                .unwrap_or("minecraft:bedrock_floor")
                .to_string(),
            true_at: parse_anchor(&v["true_at_and_below"]),
            false_at: parse_anchor(&v["false_at_and_above"]),
        },
        "above_preliminary_surface" => Condition::AbovePreliminarySurface,
        "stone_depth" => Condition::StoneDepth {
            offset: v["offset"].as_i64().unwrap_or(0) as i32,
            add_surface_depth: v["add_surface_depth"].as_bool().unwrap_or(false),
            secondary_depth_range: v["secondary_depth_range"].as_i64().unwrap_or(0) as i32,
            ceiling: v["surface_type"].as_str() == Some("ceiling"),
        },
        "biome" => {
            let biomes = match &v["biome_is"] {
                Value::String(s) => vec![biome_name_to_id(s)],
                Value::Array(arr) => arr
                    .iter()
                    .filter_map(|x| x.as_str().map(biome_name_to_id))
                    .collect(),
                _ => vec![],
            };
            Condition::Biome(biomes)
        }
        "y_above" => Condition::YAbove {
            anchor: parse_anchor(&v["anchor"]),
            surface_depth_multiplier: v["surface_depth_multiplier"].as_i64().unwrap_or(0) as i32,
            add_stone_depth: v["add_stone_depth"].as_bool().unwrap_or(false),
        },
        "water" => Condition::Water {
            offset: v["offset"].as_i64().unwrap_or(0) as i32,
            surface_depth_multiplier: v["surface_depth_multiplier"].as_i64().unwrap_or(0) as i32,
            add_stone_depth: v["add_stone_depth"].as_bool().unwrap_or(false),
        },
        "noise_threshold" => {
            let noise = v["noise"]
                .as_str()
                .unwrap_or("minecraft:surface")
                .strip_prefix("minecraft:")
                .unwrap_or("surface")
                .to_string();
            Condition::NoiseThreshold {
                noise,
                min: v["min_threshold"].as_f64().unwrap_or(f64::NEG_INFINITY),
                max: v["max_threshold"].as_f64().unwrap_or(f64::INFINITY),
            }
        }
        "not" => Condition::Not(Box::new(parse_condition(&v["invert"]))),
        "hole" => Condition::Hole,
        "steep" => Condition::Steep,
        "temperature" => Condition::Temperature,
        _ => Condition::Hole, // never matches harmlessly? hole is surfaceDepth<=0
    }
}

fn parse_anchor(v: &Value) -> VerticalAnchor {
    if let Some(a) = v.get("absolute").and_then(|x| x.as_i64()) {
        return VerticalAnchor::Absolute(a as i32);
    }
    if let Some(a) = v.get("above_bottom").and_then(|x| x.as_i64()) {
        return VerticalAnchor::AboveBottom(a as i32);
    }
    if let Some(a) = v.get("below_top").and_then(|x| x.as_i64()) {
        return VerticalAnchor::BelowTop(a as i32);
    }
    VerticalAnchor::Absolute(0)
}

fn biome_name_to_id(name: &str) -> u8 {
    let n = name.strip_prefix("minecraft:").unwrap_or(name);
    match n {
        "ocean" => biome_id::OCEAN,
        "deep_ocean" => biome_id::DEEP_OCEAN,
        "frozen_ocean" => biome_id::FROZEN_OCEAN,
        "deep_frozen_ocean" => biome_id::DEEP_FROZEN_OCEAN,
        "deep_cold_ocean" => biome_id::DEEP_COLD_OCEAN,
        "cold_ocean" => biome_id::COLD_OCEAN,
        "deep_lukewarm_ocean" => biome_id::DEEP_LUKEWARM_OCEAN,
        "lukewarm_ocean" => biome_id::LUKEWARM_OCEAN,
        "warm_ocean" => biome_id::WARM_OCEAN,
        "desert" => biome_id::DESERT,
        "plains" => biome_id::PLAINS,
        "sunflower_plains" => biome_id::SUNFLOWER_PLAINS,
        "forest" => biome_id::FOREST,
        "flower_forest" => biome_id::FLOWER_FOREST,
        "taiga" => biome_id::TAIGA,
        "snowy_taiga" => biome_id::SNOWY_TAIGA,
        "swamp" => biome_id::SWAMP,
        "mangrove_swamp" => biome_id::MANGROVE_SWAMP,
        "river" => biome_id::RIVER,
        "frozen_river" => biome_id::FROZEN_RIVER,
        "beach" => biome_id::BEACH,
        "snowy_beach" => biome_id::SNOWY_BEACH,
        "stony_shore" => biome_id::STONY_SHORE,
        "savanna" => biome_id::SAVANNA,
        "savanna_plateau" => biome_id::SAVANNA_PLATEAU,
        "windswept_savanna" => biome_id::WINDSWEPT_SAVANNA,
        "jungle" => biome_id::JUNGLE,
        "sparse_jungle" => biome_id::SPARSE_JUNGLE,
        "bamboo_jungle" => biome_id::BAMBOO_JUNGLE,
        "snowy_plains" => biome_id::SNOWY_PLAINS,
        "snowy_slopes" => biome_id::SNOWY_SLOPES,
        "jagged_peaks" => biome_id::JAGGED_PEAKS,
        "frozen_peaks" => biome_id::FROZEN_PEAKS,
        "stony_peaks" => biome_id::STONY_PEAKS,
        "grove" => biome_id::GROVE,
        "windswept_hills" => biome_id::WINDSWEPT_HILLS,
        "windswept_gravelly_hills" => biome_id::WINDSWEPT_GRAVELLY_HILLS,
        "windswept_forest" => biome_id::WINDSWEPT_FOREST,
        "dark_forest" => biome_id::DARK_FOREST,
        "meadow" => biome_id::MEADOW,
        "ice_spikes" => biome_id::ICE_SPIKES,
        "old_growth_pine_taiga" | "old_growth_pine_forest" => biome_id::OLD_GROWTH_PINE_FOREST,
        "old_growth_spruce_taiga" => biome_id::OLD_GROWTH_SPRUCE_TAIGA,
        "old_growth_birch_forest" => biome_id::OLD_GROWTH_BIRCH_FOREST,
        "birch_forest" => biome_id::BIRCH_FOREST,
        "cherry_grove" => biome_id::CHERRY_GROVE,
        "pale_garden" => biome_id::PALE_GARDEN,
        "badlands" => biome_id::BADLANDS,
        "eroded_badlands" => biome_id::ERODED_BADLANDS,
        "wooded_badlands" => biome_id::WOODED_BADLANDS,
        "dripstone_caves" => biome_id::DRIPSTONE_CAVES,
        "lush_caves" => biome_id::LUSH_CAVES,
        "deep_dark" => biome_id::DEEP_DARK,
        "mushroom_fields" => biome_id::MUSHROOM_FIELDS,
        "sulfur_caves" => biome_id::SULFUR_CAVES,
        _ => biome_id::PLAINS,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 777 sulfur_caves chunk (-5,-6): vanilla paints sulfur/cinnabar bands
    /// via the sulfur_cave_gradient noise (SurfaceRuleData sulfurCaveBands).
    /// This check pins the noise registration + biome gate used by the
    /// surface rule: params must match the vanilla JSON, and the biome
    /// gate `biome_name_to_id("sulfur_caves")` must equal the biome-source
    /// id (36).
    #[test]
    #[ignore = "diagnostic: generates chunk (-5,-6) seed 777 through noise+surface and prints blocks at mismatch cells"]
    fn sulfur_pipeline_dump() {
        let gen = crate::ChunkGenerator::new(777);
        // (chunk -> cells) so every dump line maps to its own chunk
        let mut by_chunk: std::collections::BTreeMap<(i32, i32), Vec<String>> = Default::default();
        for line in std::fs::read_to_string("/tmp/sulfur_cells.txt").unwrap().lines() {
            let mut it = line.split_whitespace();
            let wx: i32 = it.next().unwrap().parse().unwrap();
            let _y: i32 = it.next().unwrap().parse().unwrap();
            let wz: i32 = it.next().unwrap().parse().unwrap();
            by_chunk.entry((wx.div_euclid(16), wz.div_euclid(16))).or_default().push(line.to_string());
        }
        for ((cx, cz), lines) in by_chunk {
            let (blocks, _, _) = gen.generate_noise_and_surface(cx, cz);
            for line in lines.iter().take(12) {
                let mut it = line.split_whitespace();
                let wx: i32 = it.next().unwrap().parse().unwrap();
                let y: i32 = it.next().unwrap().parse().unwrap();
                let wz: i32 = it.next().unwrap().parse().unwrap();
                let lx = (wx - cx * 16) as usize;
                let lz = (wz - cz * 16) as usize;
                let rel = y - crate::generator::WORLD_BOTTOM;
                let idx = (rel as usize) * 256 + lz * 16 + lx;
                let b = crate::surface::BlockId::from_u16(blocks[idx]);
                let pip_biome = crate::biome::manager::biome_id_at_block(&gen.state, wx, y, wz);
                println!("PIPE {wx} {y} {wz} -> {:?} biome_here={pip_biome}", b);
            }
        }
    }

    #[test]
    #[ignore = "diagnostic: walks the parsed surface rule at vanilla-mismatch cells and prints the verdict"]
    fn sulfur_rule_walk() {
        let rule = overworld_rule();
        let reg = crate::density::DensityRegistry::build();
        let ns = crate::worldgen::NoiseSet::for_seed(777, &reg);
        for line in std::fs::read_to_string("/tmp/sulfur_cells.txt").unwrap().lines().take(20) {
            let mut it = line.split_whitespace();
            let x: i32 = it.next().unwrap().parse().unwrap();
            let y: i32 = it.next().unwrap().parse().unwrap();
            let z: i32 = it.next().unwrap().parse().unwrap();
            let grad = ns.noises().get("sulfur_cave_gradient").unwrap()
                .get_value(x as f64, y as f64, z as f64);
            let ctx = RuleContext {
                x, y, z,
                stone_depth_above: 1,
                stone_depth_below: 1,
                water_height: i32::MIN,
                surface_depth: 3,
                surface_secondary: 0.0,
                min_surface_level: 60,
                biome: biome_id::SULFUR_CAVES,
                steep: false,
                hole: false,
                noises: ns.noises(),
                main_rng: crate::positional::PositionalRandomFactory::new(0, 0),
                sea_level: 63,
            };
            let mut ctx = ctx;
            let out = rule.try_apply(&mut ctx);
            println!("WALK {x} {y} {z} grad={grad:.6} -> {:?}", out);
        }
    }

    #[test]
    #[ignore = "diagnostic: for each vanilla/neutron classifier mismatch, prints the climate target and the fitness of BOTH answers under neutron's metric — equal fitness = tie-break divergence"]
    fn sulfur_biome_dump() {
        let gen = crate::ChunkGenerator::new(456);
        let mut by_chunk: std::collections::BTreeMap<(i32, i32), Vec<String>> = Default::default();
        for line in std::fs::read_to_string("/tmp/clay456.txt").unwrap().lines() {
            let mut it = line.split_whitespace();
            let wx: i32 = it.next().unwrap().parse().unwrap();
            let _y: i32 = it.next().unwrap().parse().unwrap();
            let wz: i32 = it.next().unwrap().parse().unwrap();
            by_chunk.entry((wx.div_euclid(16), wz.div_euclid(16))).or_default().push(line.to_string());
        }
        for ((cx, cz), lines) in by_chunk.iter(){
            let (blocks, _, _) = gen.generate_noise_and_surface(*cx, *cz);
            for line in lines.iter().take(10) {
                let mut it = line.split_whitespace();
                let wx: i32 = it.next().unwrap().parse().unwrap();
                let y: i32 = it.next().unwrap().parse().unwrap();
                let wz: i32 = it.next().unwrap().parse().unwrap();
                let lx = (wx - cx * 16) as usize;
                let lz = (wz - cz * 16) as usize;
                let rel = y - crate::generator::WORLD_BOTTOM;
                let idx = (rel as usize) * 256 + lz * 16 + lx;
                let b = crate::surface::BlockId::from_u16(blocks[idx]);
                let pip = crate::biome::manager::biome_id_at_block(&gen.state, wx, y, wz);
                let g = crate::biome::manager::climate_at(&gen.state, wx, y, wz);
                let grad = crate::worldgen::NoiseSet::for_seed(777, &gen.state.reg);
                let gv = grad.noises().get("sulfur_cave_gradient").unwrap()
                    .get_value(wx as f64, y as f64, wz as f64);
                let _ = (pip, g);
                println!("CLAY {wx} {y} {wz} -> {:?} biome={pip}", b);
            }
        }
    }

    /// Column (-88,-56) seed 777 — drove the per-block biome fix (55b0f5f) and the
    /// shortcut removal (0d3093d). Vanilla voronoi (ProbeBiomeAt): birch_forest at
    /// y -30..-25, sulfur_caves at -24,-23, birch at -22..-20, sulfur at -19..-16.
    /// Neutron must match block-for-block now.
    #[test]
    #[ignore = "diagnostic: compares seed-777 column (-88,-56) biome ids against vanilla voronoi"]
    fn sulfur_column_biomes() {
        let gen = crate::ChunkGenerator::new(777);
        // (y, expected vanilla biome id)
        let want: [(i32, u8); 15] = [
            (-30, 33), (-29, 33), (-28, 33), (-27, 33), (-26, 33), (-25, 33),
            (-24, 36), (-23, 36), (-22, 33), (-21, 33), (-20, 33),
            (-19, 36), (-18, 36), (-17, 36), (-16, 36),
        ];
        for (y, exp) in want {
            let b = crate::biome::manager::biome_id_at_block(&gen.state, -88, y, -56);
            assert_eq!(b, exp, "column (-88,{y},-56)");
        }
    }

    /// Seed 777 chunk (-6,-4): the sulfur-band cells (-88,-24,-56) etc. must now be
    /// painted sulfur/cinnabar by the surface rule (previously deepslate — the
    /// 8-block cache bug).
    #[test]
    #[ignore = "diagnostic: verifies surface rule paints sulfur at the previously-broken cells"]
    fn sulfur_pipeline_fixed() {
        let gen = crate::ChunkGenerator::new(777);
        let (blocks, _, _) = gen.generate_noise_and_surface(-6, -4);
        let cells: [(i32, i32, i32); 4] = [
            (-88, -24, -56), (-85, -23, -61), (-84, -22, -50), (-83, -21, -54),
        ];
        for (wx, y, wz) in cells {
            let idx = ((y - crate::generator::WORLD_BOTTOM) as usize) * 256
                + ((wz + 64) as usize) * 16
                + ((wx + 96) as usize);
            let b = crate::surface::BlockId::from_u16(blocks[idx]).unwrap();
            assert!(
                matches!(b, crate::surface::BlockId::Sulfur | crate::surface::BlockId::Cinnabar),
                "({wx},{y},{wz}) = {b:?}, want sulfur/cinnabar"
            );
        }
    }

    /// 456 clay: the union diagnostic that cleared the dispatch hypothesis —
    /// lush_caves IS present in the origin 3x3 union for all three missing-clay
    /// chunks, so the step-6 ore_clay (size-33 blob, count=46) and step-9
    /// lush_caves_clay both dispatch. The residual clay->deepslate gap is a
    /// stream-displacement symptom: vanilla-missing clay sits at y -32..0
    /// while neutron-only clay leaks up to +56, i.e. the per-attempt
    /// in_square/height draws are shifted by upstream desync — same root as
    /// the tree cascade. No local fix; documented 7 Sep s31.
    #[test]
    #[ignore = "diagnostic: prints the origin biome union at the 456 clay chunks"]
    fn clay456_union_dump() {
        let gen = crate::ChunkGenerator::new(456);
        for (cx, cz) in [(3, -8), (-8, -4), (-3, -5)] {
            let ox0 = cx * 16;
            let oz0 = cz * 16;
            // replicate the union source: section-quart noise biomes over 3x3
            let st = &gen.state;
            let mut names: Vec<String> = Vec::new();
            for dz in -1..=1i32 {
                for dx in -1..=1i32 {
                    for sy in 0..24i32 {
                        let qy = -4 + sy * 2; // section midpoint quarts (y -64..320 / 8)
                        let q = crate::biome::manager::noise_biome_at_quart(
                            st,
                            ((ox0 + dx * 16) >> 2) + 2,
                            qy,
                            ((oz0 + dz * 16) >> 2) + 2,
                        );
                        let n = match q {
                            34 => "lush_caves",
                            36 => "sulfur_caves",
                            33 => "birch_forest",
                            3 => "forest",
                            10 => "jungle",
                            48 => "sparse_jungle",
                            1 => "plains",
                            _ => "?",
                        };
                        if !names.iter().any(|x| x.contains(n)) {
                            names.push(format!("{n}({q})"));
                        }
                    }
                }
            }
            println!("UNION chunk=({cx},{cz}) origin=({ox0},{oz0}): {}", names.join(", "));
        }
    }

    #[test]
    fn sulfur_cave_gradient_noise_and_gate() {
        assert_eq!(biome_name_to_id("sulfur_caves"), biome_id::SULFUR_CAVES);
        let reg = crate::density::DensityRegistry::build();
        let (o, a) = reg.noise_params("sulfur_cave_gradient");
        assert_eq!(*o, -5);
        assert_eq!(a.as_slice(), &[1.0, 0.0, 1.0]);
    }
}

#[cfg(test)]
mod dripstone_trace {
    /// 10000 dripstone stream probe: chunk (-14,-7) — dumps neutron's
    /// dripstone_cluster/pointed_dripstone attempt draw counts for
    /// comparison against vanilla GIFDRAW (ProbeFullDecorate
    /// PROBE_DRAW_ALL=1, step 7, gif=4).
    #[test]
    #[ignore = "diagnostic: traces neutron dripstone draws for chunk (-14,-7) seed 10000"]
    fn dripstone10000_trace() {
        let gen = crate::ChunkGenerator::new(10000);
        let _ = gen.generate_chunk(-14, -7);
    }
}

#[cfg(test)]
mod dripstone_gate {
    /// Seed 10000 origin (-224,-112): vanilla's dripstone_cluster first attempt
    /// (count raw 2 → count 50) at (-219, 1, -98) PASSES the biome gate
    /// (dripstone_caves). Neutron must agree.
    #[test]
    #[ignore = "diagnostic: prints neutron biome ids at vanilla's first dripstone attempts"]
    fn dripstone10000_gate_biomes() {
        let gen = crate::ChunkGenerator::new(10000);
        // vanilla attempts: (x,z,y) triples from GIFDRAW: (5,14,1),(11,13,202),(6,5,18)...
        let cells: [(i32, i32, i32); 6] = [
            (-224 + 5, 1, -112 + 14),
            (-224 + 11, 202, -112 + 13),
            (-224 + 6, 18, -112 + 5),
            (-224 + 10, 43, -112 + 8),
            (-224 + 0, 242, -112 + 0),
            (-224 + 9, 105, -112 + 2),
        ];
        for (x, y, z) in cells {
            let b = crate::biome::manager::biome_id_at_block(&gen.state, x, y, z);
            println!("GATE ({x},{y},{z}) id={b} ({})", if b == 35 { "dripstone_caves" } else { "?" });
        }
    }
}

#[cfg(test)]
mod dripstone_noise_gate {
    /// Seed 10000 origin (-224,-112): the vanilla biome-gate reads the chunk's
    /// STORED section biomes (noise biomes at quart resolution — filled by
    /// MultiNoiseBiomeSource during noise gen, no voronoi zoom). Neutron's
    /// gate must use noise_biome_at_quart, not biome_id_at_block (voronoi).
    #[test]
    #[ignore = "diagnostic: prints noise-biome vs voronoi at vanilla's dripstone gate cells"]
    fn dripstone10000_noise_vs_voronoi() {
        let gen = crate::ChunkGenerator::new(10000);
        let cells: [(i32, i32, i32); 3] = [
            (-219, 1, -98),
            (-213, 202, -99),
            (-218, 18, -107),
        ];
        for (x, y, z) in cells {
            let noise = crate::biome::manager::noise_biome_at_quart(
                &gen.state, x >> 2, y >> 2, z >> 2,
            );
            let vor = crate::biome::manager::biome_id_at_block(&gen.state, x, y, z);
            println!(
                "CELL ({x},{y},{z}) noise_biome={noise} voronoi={vor}"
            );
        }
    }
}

#[cfg(test)]
mod dripstone_ref_biome {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    /// Read the REF world's STORED biome palette around y=1 for chunk
    /// (-14,-7) seed 10000.
    #[test]
    #[ignore = "diagnostic: dumps the ref chunk's stored biomes near the dripstone gate cell"]
    fn dripstone10000_ref_stored_biome() {
        use neutron_world::Region;
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-10000/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (-14, -7);
        let region = match Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
        {
            Ok(r) => r.with_coords(cx >> 5, cz >> 5),
            Err(e) => panic!("region open failed: {e:?}"),
        };
        let raw = region
            .get_chunk(cx & 31, cz & 31)
            .expect("chunk")
            .expect("data");
        let nbt = read_nbt(&raw).expect("nbt");
        let sections = match compound_get(&nbt.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l,
            _ => panic!("no sections"),
        };
        for sec in sections {
            let y_sec = match compound_get(sec, "Y") {
                Some(Tag::Byte(y)) => *y as i8 as i32,
                Some(Tag::Int(y)) => *y,
                _ => continue,
            };
            if y_sec != 4 {
                continue;
            }
            let Some(Tag::Compound(bs)) = compound_get(sec, "biomes") else {
                println!("SECTION Y=4 has NO biomes");
                continue;
            };
            match compound_get(bs, "palette") {
                Some(Tag::List(List::String(p))) => {
                    println!("SECTION Y=4 biome palette: {:?}", p)
                }
                _ => println!("SECTION Y=4 biome palette: (unexpected tag)"),
            }
            if let Some(Tag::List(data)) = compound_get(bs, "data") {
                let n = match data {
                    List::Long(l) => l.len(),
                    _ => 0,
                };
                println!("SECTION Y=4 biome data: {n} longs");
            } else {
                println!("SECTION Y=4 biome data: none");
            }
        }
    }
}




#[cfg(test)]
mod mineshaft_ref_10101 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// Seed 10101: the ref has 244k air cells at y -56..-16 that neutron keeps
    /// as deepslate. Find which structure owns this area: dump the ref chunk
    /// (-14,1) structures starts + the neighbors' starts.
    #[test]
    #[ignore = "diagnostic: dumps ref chunk structures for seed 10101 chunk (-14,1)"]
    fn mineshaft10101_ref_structures_orig() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-10101/world/dimensions/minecraft/overworld/region");
        let mut all = String::new();
        for rx in [-1i32, 0] {
            for rz in [0i32, 1] {
                let region = match Region::open(std::path::Path::new(&format!(
                    "{dir}/r.{rx}.{rz}.mca")))
                {
                    Ok(r) => r.with_coords(rx, rz),
                    Err(e) => {
                        all.push_str(&format!("region ({rx},{rz}) open failed: {e:?}\n"));
                        continue;
                    }
                };
                for lx in 0..32 {
                    for lz in 0..32 {
                        let cx = rx * 32 + lx;
                        let cz = rz * 32 + lz;
                        let raw = match region.get_chunk(lx, lz) {
                            Ok(Some(r)) => r,
                            _ => continue,
                        };
                        let nbt = read_nbt(&raw).expect("nbt");
                        let Some(Tag::Compound(st)) = compound_get(&nbt.compound, "structures")
                        else {
                            continue;
                        };
                        let Some(Tag::Compound(starts)) = compound_get(st, "starts") else {
                            continue;
                        };
                        for (name, tag) in &starts.tags {
                            if name.to_string() == "minecraft:mineshaft" {
                                all.push_str(&format!(
                                    "chunk ({cx},{cz}) mineshaft\n"
                                ));
                            }
                            if name.to_string() == "minecraft:ancient_city" {
                                all.push_str(&format!(
                                    "chunk ({cx},{cz}) ancient_city\n"
                                ));
                            }
                            if name.to_string() == "minecraft:trial_chambers" {
                                all.push_str(&format!(
                                    "chunk ({cx},{cz}) trial_chambers\n"
                                ));
                            }
                        }
                    }
                }
            }
        }
        eprintln!("STRUCTS-10101\n{all}");
        // Ground truth measured 8 Sep (seed 10101): mineshaft starts at
        // (-14,0) and (-2,8), ancient_city at (-14,9). Neutron's mineshaft
        // placement matches; no ancient_city port exists yet (the 244k
        // air->deepslate family at y -56..-16 is the unported city).
        assert!(all.contains("mineshaft"));
        assert!(all.contains("ancient_city"));
    }
}

#[cfg(test)]
mod city_ref_10101 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// Seed 10101 chunk (-14,9): dump the ref's ancient_city start: BB,
    /// Children piece ids + BBs — the assembly ground truth for the port.
    #[test]
    #[ignore = "diagnostic: dumps ref ancient_city pieces for seed 10101 chunk (-14,9)"]
    fn city10101_ref_pieces() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-10101/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (-14, 9);
        let region = match Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
        {
            Ok(r) => r.with_coords(cx >> 5, cz >> 5),
            Err(e) => panic!("region open failed: {e:?}"),
        };
        let raw = region
            .get_chunk(cx & 31, cz & 31)
            .expect("chunk")
            .expect("data");
        let nbt = read_nbt(&raw).expect("nbt");
        let Tag::Compound(st) = compound_get(&nbt.compound, "structures").unwrap() else {
            panic!("no structures");
        };
        let Tag::Compound(starts) = compound_get(st, "starts").unwrap() else {
            panic!("no starts");
        };
        let mut found = false;
        for (name, tag) in &starts.tags {
            if name.to_string() != "minecraft:ancient_city" {
                continue;
            }
            found = true;
            let Tag::Compound(start) = tag else { continue };
            let bb = compound_get(start, "BB");
            let Tag::List(List::Compound(children)) =
                compound_get(start, "Children").expect("children")
            else {
                panic!("children not compound list");
            };
            let mut out = format!("CITY-PIECES {}\nBB={bb:?}\n", children.len());
            for (i, ch) in children.iter().enumerate() {
                let id = compound_get(ch, "id");
                let bb = compound_get(ch, "BB");
                let pos = compound_get(ch, "Pos");
                let mut extra = String::new();
                if let Some(Tag::IntArray(b)) = bb {
                    let v = b.to_vec();
                    if v.len() == 6 && v[0] == v[3] && v[1] == v[4] && v[2] == v[5] {
                        for (k, val) in &ch.tags {
                            extra.push_str(&format!(" {k}={val:?}"));
                        }
                    }
                }
                out.push_str(&format!("[{i}] id={id:?} bb={bb:?}{extra}\n"));
                let _ = pos;
            }
            panic!("CITY-DUMP {out}");
        }
        assert!(found, "no ancient_city start in chunk");
    }
}

#[cfg(test)]
mod city_gen_10101 {
    use crate::generator::ChunkGenerator;

    /// Seed 10101 chunk (-14,9): generate through the real pipeline and
    /// check whether city blocks land.
    #[test]
    #[ignore = "diagnostic: full-pipeline generation of chunk (-14,9) seed 10101"]
    fn city10101_gen_pipeline() {
        let gen = ChunkGenerator::new(10101);
        let chunk = gen.generate_chunk(-13, 10);
        // the ref-air example cell from the ledger
        eprintln!(
            "CELL (-202,-51,144) = {:?}",
            chunk.block_at(((-202) - (-208)) as u32, -51, (144 - 144) as u32)
        );
        eprintln!(
            "CELL (-202,-50,144) = {:?}",
            chunk.block_at(((-202) - (-208)) as u32, -50, (144 - 144) as u32)
        );
        // count sculk/deepslate_tiles/chamber-ish blocks in the chunk column
        let mut counts = std::collections::HashMap::new();
        for y in crate::generator::WORLD_BOTTOM..crate::generator::WORLD_TOP {
            for lz in 0..16u32 {
                for lx in 0..16u32 {
                    let b = chunk.block_at(lx, y, lz);
                    *counts.entry(format!("{b:?}")).or_insert(0usize) += 1;
                }
            }
        }
        for k in ["DeepslateTiles", "Sculk", "PolishedBasalt", "Deepslate", "IronBars", "Lantern"] {
            eprintln!("GEN-COUNT {k}={:?}", counts.get(k).unwrap_or(&0));
        }
        panic!("GEN-DONE");
    }
}

#[cfg(test)]
mod city_ref_424242 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// Does the 424242 ref actually have the ancient_city start at chunk
    /// (-13,9)? Scan the region for its structure starts.
    #[test]
    #[ignore = "diagnostic: scan 424242 ref region r.-1.0 for city starts"]
    fn city424242_ref_check() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-424242/world/dimensions/minecraft/overworld/region");
        let mut out = String::new();
        for rx in [-1i32, 0] {
            for rz in [0i32, 1] {
                let region = match Region::open(std::path::Path::new(&format!(
                    "{dir}/r.{rx}.{rz}.mca")))
                {
                    Ok(r) => r.with_coords(rx, rz),
                    Err(e) => {
                        out.push_str(&format!("region ({rx},{rz}): {e:?}\n"));
                        continue;
                    }
                };
                for lx in 0..32 {
                    for lz in 0..32 {
                        let cx = rx * 32 + lx;
                        let cz = rz * 32 + lz;
                        let raw = match region.get_chunk(lx, lz) {
                            Ok(Some(r)) => r,
                            _ => continue,
                        };
                        let Ok(nbt) = read_nbt(&raw) else { continue };
                        let Some(Tag::Compound(st)) = compound_get(&nbt.compound, "structures")
                        else {
                            continue;
                        };
                        let Some(Tag::Compound(starts)) = compound_get(st, "starts") else {
                            continue;
                        };
                        for (name, _) in &starts.tags {
                            if name.to_string() == "minecraft:ancient_city" {
                                out.push_str(&format!("chunk ({cx},{cz}) ancient_city\n"));
                            }
                            if name.to_string() == "minecraft:trial_chambers" {
                                out.push_str(&format!("chunk ({cx},{cz}) trial_chambers\n"));
                            }
                            if name.to_string() == "minecraft:mineshaft" {
                                out.push_str(&format!("chunk ({cx},{cz}) mineshaft\n"));
                            }
                        }
                    }
                }
            }
        }
        eprintln!("REF-424242-CITIES\n{out}");
        panic!("SCAN-DONE");
    }
}

#[cfg(test)]
mod city_biome_424242 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// Read the 424242 ref chunk (-13,9) stored biomes at section Y=-3
    /// (y -48..-33) — is it deep_dark?
    #[test]
    #[ignore = "diagnostic: ref chunk (-13,9) biome palette"]
    fn city424242_ref_biome() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-424242/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (-13, 9);
        let region = Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
            .expect("region")
            .with_coords(cx >> 5, cz >> 5);
        let raw = region.get_chunk(cx & 31, cz & 31).expect("chunk").expect("data");
        let nbt = read_nbt(&raw).expect("nbt");
        let sections = match compound_get(&nbt.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l,
            _ => panic!("no sections"),
        };
        for sec in sections {
            let y_sec = match compound_get(sec, "Y") {
                Some(Tag::Byte(y)) => *y as i8 as i32,
                Some(Tag::Int(y)) => *y,
                _ => continue,
            };
            if y_sec != -3 {
                continue;
            }
            let Some(Tag::Compound(bs)) = compound_get(sec, "biomes") else {
                println!("SEC -3 NO BIOMES");
                continue;
            };
            if let Some(Tag::List(List::String(p))) = compound_get(bs, "palette") {
                println!("SEC-3 PALETTE {:?}", p);
            }
        }
        panic!("BIOME-DONE");
    }
}

#[cfg(test)]
mod carve_trace_10101 {
    use crate::generator::ChunkGenerator;

    /// Seed 10101 chunk (-14,2): trace neutron's carver starts (the ref-air
    /// cells at z 12..64 need vanilla carvers my pass may miss).
    #[test]
    #[ignore = "diagnostic: runs neutron carvers for chunk (-14,2) seed 10101 with trace"]
    fn carve10101_trace() {
        let gen = ChunkGenerator::new(10101);
        let _ = gen.generate_chunk(-14, 2);
        panic!("TRACE-DONE");
    }
}

#[cfg(test)]
mod ref_block_10101 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// FINDING (s49+s52): the predc1-based "scene divergence" was an ARTIFACT —
    /// the vanilla ProbePreDecorate dump has the deepslate transition
    /// UNAPPLIED (621k stone vs 146k deepslate below y=0, while the real ref
    /// world has deepslate below y≈0 — the parity tool confirms deepslate at
    /// y=-63 on both sides). The probe's buildSurface is not equivalent to
    /// the real server's. The displaced-tree family's cause remains the
    /// origin-order/ticket-sim residual (11-13% violations).
    ///
    /// s52 CHAIN: dark_forest_vegetation y = OCEAN_FLOOR heightmap at (x,z)
    /// (placed_feature JSON) — the heightmap depends on the SCENE. Scene
    /// terrain diffs (scattered ore/stone/carver cells from the origin-order
    /// cascade) shift the heightmap → tree attempts land at different y →
    /// acceptance flips (SurfaceWaterDepthFilter/BiomeFilter) → displaced
    /// trees. The tree displacement is DOWNSTREAM of terrain diffs; the fix
    /// is the terrain/ore parity at border origins (the origin-order
    /// cascade), not the tree feature itself.
    /// FINDING (s41): the ref air at these cells = SculkVeinBlock.onDischarged
    /// (vein with no faces left converts to AIR — SculkVeinBlock.java:81).
    /// The 194k air->deepslate family is sculk-vein discharge air; the sculk
    /// spread on seed 10101 never reaches/places veins at those positions in
    /// neutron. Root cause lives in the sculk cursor movement/placement.
    #[test]
    #[ignore = "diagnostic: MY chunk (-14,2) sculk/vein census at y=-51"]
    fn my_sculk_census_10101() {
        let gen = crate::generator::ChunkGenerator::new(10101);
        let chunk = gen.generate_chunk(-14, 2);
        let mut census: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for ly in [-64i32 + 16, -51, -44, -40, -36, -32] {
            let mut sculk_count = 0usize;
            let mut vein_count = 0usize;
            let mut air_count = 0usize;
            for lz in 0..16u32 {
                for lx in 0..16u32 {
                    let b = chunk.block_at(lx, ly, lz);
                    match b {
                        crate::surface::BlockId::Sculk => sculk_count += 1,
                        crate::surface::BlockId::SculkVein => vein_count += 1,
                        crate::surface::BlockId::Air | crate::surface::BlockId::CaveAir => {
                            air_count += 1
                        }
                        _ => {}
                    }
                }
            }
            eprintln!(
                "MY-Y {ly}: sculk={sculk_count} vein={vein_count} air={air_count}"
            );
        }
        let _ = &census;
        panic!("CENSUS-DONE");
    }

    #[test]
    #[ignore = "diagnostic: ref blocks at (-224..-222,-51,32)"]
    fn ref_block_10101_cell() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-10101/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (-14, 2);
        let region = Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
            .expect("region")
            .with_coords(cx >> 5, cz >> 5);
        let raw = region.get_chunk(cx & 31, cz & 31).expect("chunk").expect("data");
        let nbt = read_nbt(&raw).expect("nbt");
        let sections = match compound_get(&nbt.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l,
            _ => panic!("no sections"),
        };
        for sec in sections {
            let y_sec = match compound_get(sec, "Y") {
                Some(Tag::Byte(y)) => *y as i8 as i32,
                Some(Tag::Int(y)) => *y,
                _ => continue,
            };
            // y -51 → section -4 (y -64..-49), local y = -51 - (-64) = 13
            if y_sec != -4 {
                continue;
            }
            let Some(Tag::Compound(bs)) = compound_get(sec, "block_states") else {
                println!("SEC-4 no block_states");
                continue;
            };
            let palette = match compound_get(bs, "palette") {
                Some(Tag::List(List::Compound(p))) => p
                    .iter()
                    .map(|e| {
                        match compound_get(e, "Name") {
                            Some(Tag::String(n)) => n.to_string(),
                            _ => "?".to_string(),
                        }
                    })
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            println!("SEC-4 palette size {}", palette.len());
            // decode data (packed longs) for index (5, 13, 15): index = y*256 + z*16 + x
            // section Y=-4 → y -64..-49; local y = -51 + 64 = 13
            // cells (-224..-222, -51, 32): local x 0..2, local z 0
            let bits = (palette.len() as f64).log2().ceil() as usize;
            let per = 64 / bits.max(1);
            if let Some(Tag::LongArray(data)) = compound_get(bs, "data") {
                let data = data.to_vec();
                for lx in 0..16usize {
                for ly in 0..16usize {
                    let idx: usize = ly * 256 + 0 * 16 + lx;
                    let li = idx / per;
                    let po = idx % per;
                    let long = data.get(li).copied().unwrap_or(0) as u64;
                    let mut v: u64 = 0;
                    for b in 0..bits {
                        v |= (((long >> (po * bits + b)) & 1) << b);
                    }
                    let default_name = "?".to_string();
                    let name = palette.get(v as usize).unwrap_or(&default_name);
                    if name.contains("sculk") {
                        println!(
                            "REF-SCULK ({},{},32) = {}",
                            -224 + lx as i32,
                            -64 + ly as i32,
                            name
                        );
                    }
                }
                }
            } else {
                println!("SEC-4 uniform palette: {:?}", palette.first());
            }
        }
        panic!("REF-BLOCK-DONE");
    }
}

#[cfg(test)]
mod mineshaft_ref_children_10101 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// Dump the ref's mineshaft (-14,0) children BBs (piece tree ground truth).
    #[test]
    #[ignore = "diagnostic: ref mineshaft children BBs seed 10101"]
    fn mineshaft_children_10101() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-10101/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (-14, 0);
        let region = Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
            .expect("region")
            .with_coords(cx >> 5, cz >> 5);
        let raw = region.get_chunk(cx & 31, cz & 31).expect("chunk").expect("data");
        let nbt = read_nbt(&raw).expect("nbt");
        let Tag::Compound(st) = compound_get(&nbt.compound, "structures").unwrap() else {
            panic!("no structures");
        };
        let Tag::Compound(starts) = compound_get(st, "starts").unwrap() else {
            panic!("no starts");
        };
        for (name, tag) in &starts.tags {
            if name.to_string() != "minecraft:mineshaft" {
                continue;
            }
            let Tag::Compound(start) = tag else { continue };
            let Tag::List(List::Compound(children)) =
                compound_get(start, "Children").expect("children")
            else {
                panic!("children not compound list");
            };
            for (i, ch) in children.iter().enumerate() {
                let bb = compound_get(ch, "BB");
                if let Some(Tag::IntArray(b)) = bb {
                    let v = b.to_vec();
                    eprintln!(
                        "REF-MS {} {} {} {} {} {} {}",
                        i, v[0], v[1], v[2], v[3], v[4], v[5]
                    );
                }
            }
        }
        panic!("REF-MS-DONE");
    }
}

#[cfg(test)]
mod all_starts_10101 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// Dump ALL structure starts for 10101 chunks (-14,2), (-14,1), (-13,2).
    #[test]
    #[ignore = "diagnostic: all structure starts near the air family"]
    fn all_starts_10101_near() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-10101/world/dimensions/minecraft/overworld/region");
        for (cx, cz) in [(-14i32, 2), (-14, 1), (-13, 2)] {
            let region = Region::open(std::path::Path::new(&format!(
                "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
                .expect("region")
                .with_coords(cx >> 5, cz >> 5);
            let raw = region.get_chunk(cx & 31, cz & 31).expect("chunk").expect("data");
            let nbt = read_nbt(&raw).expect("nbt");
            let Tag::Compound(st) = compound_get(&nbt.compound, "structures").unwrap() else {
                continue;
            };
            let Tag::Compound(starts) = compound_get(st, "starts").unwrap() else {
                continue;
            };
            if let Some(Tag::Compound(refs)) = compound_get(st, "References") {
                for (rn, rv) in &refs.tags {
                    if let Tag::List(l) = rv {
                        let cnt = match l {
                            List::Long(v) => v.len(),
                            _ => 0,
                        };
                        eprintln!("REFS ({cx},{cz}): {rn} -> {cnt}");
                    }
                }
            }
            let names: Vec<String> = starts
                .tags
                .iter()
                .map(|(n, t)| {
                    let mut size = String::new();
                    if let Tag::Compound(ct) = t {
                        if let Some(Tag::List(l)) = compound_get(ct, "Children") {
                            if let List::Compound(c) = l {
                                size = format!(" children={}", c.len());
                            }
                        }
                    }
                    format!("{}{}", n.to_string(), size)
                })
                .collect();
            eprintln!("STARTS ({cx},{cz}): {names:?}");
        }
        panic!("STARTS-DONE");
    }
}

#[cfg(test)]
mod ref_deepslate_424242 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// 424242 ref chunk (-14,-14): section Y=-4 (y -64..-49) palette —
    /// stone or deepslate at y=-63?
    #[test]
    #[ignore = "diagnostic: ref section palette for chunk (-14,-14)"]
    fn ref424242_sec4_palette() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-424242/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (-14, -14);
        let region = Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
            .expect("region")
            .with_coords(cx >> 5, cz >> 5);
        let raw = region.get_chunk(cx & 31, cz & 31).expect("chunk").expect("data");
        let nbt = read_nbt(&raw).expect("nbt");
        let sections = match compound_get(&nbt.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l,
            _ => panic!("no sections"),
        };
        for sec in sections {
            let y_sec = match compound_get(sec, "Y") {
                Some(Tag::Byte(y)) => *y as i8 as i32,
                Some(Tag::Int(y)) => *y,
                _ => continue,
            };
            eprintln!("REF-SEC Y={y_sec}");
            if y_sec != -4 {
                continue;
            }
            let Some(Tag::Compound(bs)) = compound_get(sec, "block_states") else {
                eprintln!("SEC-4 no block_states");
                continue;
            };
            if let Some(Tag::List(l)) = compound_get(bs, "palette") {
                let names: Vec<String> = match l {
                    List::Compound(cs) => cs
                        .iter()
                        .map(|e| match compound_get(e, "Name") {
                            Some(Tag::String(n)) => n.to_string(),
                            _ => "?".into(),
                        })
                        .collect(),
                    List::String(ss) => ss.iter().map(|s| s.to_string()).collect(),
                    _ => vec!["?".into()],
                };
                eprintln!("SEC-4 palette ({}): {:?}", names.len(), names);
            } else {
                eprintln!("SEC-4 palette: absent");
            }
        }
        panic!("PALETTE-DONE");
    }
}

#[cfg(test)]
mod ref_tree_424242 {
    use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
    use neutron_world::nbt::{compound_get, read_nbt};
    use neutron_world::Region;

    /// 424242 ref chunk (-14,-14): the block at (-208,71,-218) — is the
    /// dark_oak tree there in the real world? (local x=8, y=71→sec 4 ly=7,
    /// z=-218→local 6)
    #[test]
    #[ignore = "diagnostic: ref block at (-208,71,-218)"]
    fn ref424242_tree_cell() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-424242/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (-14, -14);
        let region = Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx >> 5, cz >> 5)))
            .expect("region")
            .with_coords(cx >> 5, cz >> 5);
        let raw = region.get_chunk(cx & 31, cz & 31).expect("chunk").expect("data");
        let nbt = read_nbt(&raw).expect("nbt");
        let sections = match compound_get(&nbt.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l,
            _ => panic!("no sections"),
        };
        // y=71 → section 4 (y 64..79), local y = 71-64 = 7
        // local x = -208 - (-224) = 16?? No: chunk (-14,-14) covers x -224..-209;
        // -208 is in chunk (-13,-14)! Adjust: cx=-13.
        let (cx2, cz2) = (-13, -14);
        let region2 = Region::open(std::path::Path::new(&format!(
            "{dir}/r.{}.{}.mca", cx2 >> 5, cz2 >> 5)))
            .expect("region2")
            .with_coords(cx2 >> 5, cz2 >> 5);
        let raw2 = region2.get_chunk(cx2 & 31, cz2 & 31).expect("chunk2").expect("data2");
        let nbt2 = read_nbt(&raw2).expect("nbt2");
        let sections2 = match compound_get(&nbt2.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l,
            _ => panic!("no sections2"),
        };
        // count dark_oak logs in both chunks (all sections)
        for (label, secs) in [("chunk(-14,-14)", sections), ("chunk(-13,-14)", sections2)] {
            let mut log_count = 0usize;
            for sec in secs {
                let Some(Tag::Compound(bs)) = compound_get(sec, "block_states") else {
                    continue;
                };
                let palette = match compound_get(bs, "palette") {
                    Some(Tag::List(l)) => match l {
                        List::Compound(cs) => cs
                            .iter()
                            .map(|e| match compound_get(e, "Name") {
                                Some(Tag::String(n)) => n.to_string(),
                                _ => "?".into(),
                            })
                            .collect(),
                        _ => vec![],
                    },
                    _ => vec![],
                };
                let Some(dark_idx) = palette.iter().position(|n| n == "minecraft:dark_oak_log")
                else {
                    continue;
                };
                let bits = (palette.len() as f64).log2().ceil() as usize;
                let per = (64 / bits).max(1);
                if let Some(Tag::LongArray(data)) = compound_get(bs, "data") {
                    let data = data.to_vec();
                    for idx in 0..(16 * 16 * 16) {
                        let li = idx / per;
                        let po = idx % per;
                        let long = data.get(li).copied().unwrap_or(0) as u64;
                        let mut v: u64 = 0;
                        for b in 0..bits {
                            v |= (((long >> (po * bits + b)) & 1) << b);
                        }
                        if v as usize == dark_idx {
                            log_count += 1;
                        }
                    }
                }
            }
            eprintln!("REF-TREE-COUNT {label} dark_oak_logs = {log_count}");
        }
        for (label, secs, lx) in [("chunk(-14,-14)", sections, 16), ("chunk(-13,-14)", sections2, 8)] {
            for sec in secs {
                let y_sec = match compound_get(sec, "Y") {
                    Some(Tag::Byte(y)) => *y as i8 as i32,
                    Some(Tag::Int(y)) => *y,
                    _ => continue,
                };
                if y_sec != 4 {
                    continue;
                }
                let Some(Tag::Compound(bs)) = compound_get(sec, "block_states") else {
                    continue;
                };
                let palette = match compound_get(bs, "palette") {
                    Some(Tag::List(l)) => match l {
                        List::Compound(cs) => cs
                            .iter()
                            .map(|e| match compound_get(e, "Name") {
                                Some(Tag::String(n)) => n.to_string(),
                                _ => "?".into(),
                            })
                            .collect(),
                        _ => vec![],
                    },
                    _ => vec![],
                };
                let bits = (palette.len() as f64).log2().ceil() as usize;
                let per = (64 / bits).max(1);
                let idx = 7 * 256 + 6 * 16 + lx;
                if let Some(Tag::LongArray(data)) = compound_get(bs, "data") {
                    let data = data.to_vec();
                    let li = idx / per;
                    let po = idx % per;
                    let long = data.get(li).copied().unwrap_or(0) as u64;
                    let mut v: u64 = 0;
                    for b in 0..bits {
                        v |= (((long >> (po * bits + b)) & 1) << b);
                    }
                    eprintln!(
                        "REF-TREE {label} (-208,71,-218) = {}",
                        palette.get(v as usize).unwrap_or(&"?".into())
                    );
                } else {
                    eprintln!(
                        "REF-TREE {label} uniform = {:?}",
                        palette.first()
                    );
                }
            }
        }
        panic!("TREE-CELL-DONE");
    }
}

#[cfg(test)]
mod my_tree_census_424242 {
    use crate::generator::ChunkGenerator;

    /// MY dark_oak log count for chunks (-14,-14) and (-13,-14) on 424242.
    #[test]
    #[ignore = "diagnostic: my dark_oak log census"]
    /// Trunk-base positions (lowest dark_oak_log y per column) — ref vs mine,
    /// chunk (6,2) seed 424242. The mined-pair check for the tree family.
    #[test]
    #[ignore = "diagnostic: ref-vs-mine dark_oak trunk bases in chunk (6,2)"]
    fn trunk_bases_62_424242() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-424242/world/dimensions/minecraft/overworld/region");
        let (cx, cz) = (6, 2);
        let ref_bases = ref_trunk_bases(dir, cx, cz);
        let gen = ChunkGenerator::new(424242);
        let chunk = gen.generate_chunk(cx, cz);
        let mut my_bases = std::collections::BTreeSet::new();
        for lz in 0..16u32 {
            for lx in 0..16u32 {
                for y in crate::generator::WORLD_BOTTOM..crate::generator::WORLD_TOP {
                    if chunk.block_at(lx, y, lz) == crate::surface::BlockId::DarkOakLog {
                        my_bases.insert((cx * 16 + lx as i32, y, cz * 16 + lz as i32));
                        break;
                    }
                }
            }
        }
        let only_ref: Vec<_> = ref_bases.difference(&my_bases).collect();
        let only_my: Vec<_> = my_bases.difference(&ref_bases).collect();
        eprintln!("TRUNK ref={} my={} match={}", ref_bases.len(), my_bases.len(),
            ref_bases.len() - only_ref.len());
        eprintln!("TRUNK only_ref: {:?}", only_ref);
        eprintln!("TRUNK only_my:  {:?}", only_my);
        panic!("TRUNK-DONE");
    }

    fn ref_trunk_bases(dir: &str, cx: i32, cz: i32) -> std::collections::BTreeSet<(i32, i32, i32)> {
        use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
        use neutron_world::nbt::{compound_get, read_nbt};
        use neutron_world::region::Region;
        let mut out = std::collections::BTreeSet::new();
        let (rx, rz) = (cx >> 5, cz >> 5);
        let path = std::path::PathBuf::from(format!("{dir}/r.{rx}.{rz}.mca"));
        let Ok(region) = Region::open(&path) else { return out };
        let region = region.with_coords(rx, rz);
        let Ok(data) = region.get_chunk(cx & 31, cz & 31) else { return out };
        let Some(data) = data else { return out };
        let Ok(nbt) = read_nbt(&data) else { return out };
        let sections = match compound_get(&nbt.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l.clone(),
            _ => return out,
        };
        let wb = crate::generator::WORLD_BOTTOM;
        let mut blocks = vec![0u16; 384 * 256];
        for sec in sections.iter() {
            let y_sec = match compound_get(sec, "Y") {
                Some(Tag::Byte(y)) => *y as i8 as i32,
                Some(Tag::Int(y)) => *y,
                _ => continue,
            };
            let Some(Tag::Compound(bs)) = compound_get(sec, "block_states") else { continue };
            let Some(Tag::List(List::Compound(palette))) = compound_get(bs, "palette") else { continue };
            let names: Vec<String> = palette.iter().map(|pc| match compound_get(pc, "Name") {
                Some(Tag::String(s)) => s.to_string(),
                _ => "minecraft:air".into(),
            }).collect();
            if names.is_empty() { continue; }
            let bits = if names.len() <= 1 { 0u32 } else { ((names.len() - 1).ilog2() + 1).max(4) };
            let Some(Tag::LongArray(data)) = compound_get(bs, "data") else { continue };
            let longs: Vec<i64> = data.to_vec();
            let epl = 64 / bits.max(1);
            let mask = (1u64 << bits) - 1;
            for i in 0..4096u32 {
                if bits == 0 { break; }
                let li = (i / epl) as usize;
                if li >= longs.len() { break; }
                let bo = (i % epl) * bits;
                let idxp = ((longs[li] as u64) >> bo) & mask;
                let name = names.get(idxp as usize).cloned().unwrap_or_default();
                if name == "minecraft:dark_oak_log" {
                    let ly = (i >> 8) as i32;
                    let lz = ((i >> 4) & 15) as i32;
                    let lx = (i & 15) as i32;
                    let y = y_sec * 16 + ly;
                    blocks[((y - wb) * 256 + lz * 16 + lx) as usize] = 1;
                }
            }
        }
        for lz in 0..16i32 {
            for lx in 0..16i32 {
                for ly in 0..384i32 {
                    if blocks[(ly * 256 + lz * 16 + lx) as usize] == 1 {
                        out.insert((cx * 16 + lx, wb + ly, cz * 16 + lz));
                        break;
                    }
                }
            }
        }
        out
    }

    /// Column dumps at a missing-lush-clay position: ref vs mine, chunk (-12,-6).
    #[test]
    #[ignore = "diagnostic: column dump at (-192,13,-93)"]
    fn lush_column_dump_424242() {
        let (wx, wy, wz) = (-192, 13, -93);
        let (cx, cz) = (-12, -6);
        // ref
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/nbt-ref/vanilla-fresh-424242/world/dimensions/minecraft/overworld/region");
        let ref_bases = ref_column(dir, cx, cz, wx & 15, wz & 15, wy - 2, wy + 14);
        eprintln!("LUSH-REF: {:?}", ref_bases);
        // mine
        let gen = ChunkGenerator::new(424242);
        let chunk = gen.generate_chunk(cx, cz);
        let mut mine = Vec::new();
        for y in wy - 2..=wy + 14 {
            mine.push((y, format!("{:?}", chunk.block_at((wx & 15) as u32, y, (wz & 15) as u32))));
        }
        eprintln!("LUSH-MINE: {:?}", mine);
        panic!("LUSH-DONE");
    }

    fn ref_column(dir: &str, cx: i32, cz: i32, lx: i32, lz: i32, y0: i32, y1: i32) -> Vec<(i32, String)> {
        use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
        use neutron_world::nbt::{compound_get, read_nbt};
        use neutron_world::region::Region;
        let mut out = Vec::new();
        let (rx, rz) = (cx >> 5, cz >> 5);
        let path = std::path::PathBuf::from(format!("{dir}/r.{rx}.{rz}.mca"));
        let Ok(region) = Region::open(&path) else { return out };
        let region = region.with_coords(rx, rz);
        let Ok(data) = region.get_chunk(cx & 31, cz & 31) else { return out };
        let Some(data) = data else { return out };
        let Ok(nbt) = read_nbt(&data) else { return out };
        let sections = match compound_get(&nbt.compound, "sections") {
            Some(Tag::List(List::Compound(l))) => l.clone(),
            other => {
                eprintln!("LUSH-DBG sections tag: {:?}", other.is_some());
                return out;
            }
        };
        eprintln!("LUSH-DBG sections={}", sections.len());
        for sec in sections.iter() {
            let y_sec = match compound_get(sec, "Y") {
                Some(Tag::Byte(y)) => *y as i8 as i32,
                _ => continue,
            };
            let Some(Tag::Compound(bs)) = compound_get(sec, "block_states") else { continue };
            let Some(Tag::List(List::Compound(palette))) = compound_get(bs, "palette") else { continue };
            let names: Vec<String> = palette.iter().map(|pc| match compound_get(pc, "Name") {
                Some(Tag::String(s)) => s.to_string(),
                _ => "minecraft:air".into(),
            }).collect();
            if names.is_empty() { continue; }
            let bits = if names.len() <= 1 { 0u32 } else { ((names.len() - 1).ilog2() + 1).max(4) };
            let Some(Tag::LongArray(data)) = compound_get(bs, "data") else { continue };
            let longs: Vec<i64> = data.to_vec();
            let epl = 64 / bits.max(1);
            let mask = (1u64 << bits) - 1;
            for i in 0..4096u32 {
                if bits == 0 { break; }
                let li = (i / epl) as usize;
                if li >= longs.len() { break; }
                let bo = (i % epl) * bits;
                let idxp = ((longs[li] as u64) >> bo) & mask;
                let name = names.get(idxp as usize).cloned().unwrap_or_default();
                let ly = (i >> 8) as i32;
                let ilz = ((i >> 4) & 15) as i32;
                let ilx = (i & 15) as i32;
                if ilz == lz && ilx == lx {
                    let y = y_sec * 16 + ly;
                    if y >= y0 && y <= y1 {
                        out.push((y, name));
                    }
                }
            }
        }
        out.sort_by_key(|e| e.0);
        out
    }

    /// Climate target at the witness column's quart — my sampler vs the
    /// expected lush_caves point.
    #[test]
    #[ignore = "diagnostic: climate params at (-192,13,-93)"]
    fn climate_at_witness_424242() {
        let state = crate::worldgen::WorldgenState::overworld(424242);
        for y in [9, 13, 20, 40, 64, 90] {
            let t = crate::biome::manager::climate_at(&state, -192, y, -93);
            let _ = &t; // ClimateTarget has no Debug; print the resolved biome
            let b2 = crate::biome::manager::biome_id_at_block(&state, -192, y, -93);
            let b = crate::biome::manager::biome_id_at_block(&state, -192, y, -93);
            eprintln!("BIOME y={y}: id={b} / direct={b2}");
        }
        panic!("CLIMATE-DONE");
    }

    /// MY lush patch attempts at the 9 origins around chunk (-12,-6).
    #[test]
    #[ignore = "diagnostic: my lush attempts for the s64 oracle diff"]
    fn my_lush_attempts_424242() {
        let gen = ChunkGenerator::new(424242);
        for (cx, cz) in [
            (-13i32, -7), (-12, -7), (-11, -7),
            (-13, -6), (-12, -6), (-11, -6),
            (-13, -5), (-12, -5), (-11, -5),
        ] {
            let _ = gen.generate_chunk(cx, cz);
        }
        panic!("LUSH-ATTEMPTS-DONE");
    }

    /// MY tree trace for the 9 origins around chunk (7,2) — the tree-family
    /// window (s70 oracle diff).
    #[test]
    #[ignore = "diagnostic: my tree accepts for the tree window"]
    fn my_tree_attempts_424242() {
        let gen = ChunkGenerator::new(424242);
        for (cx, cz) in [
            (6i32, 1), (7, 1), (8, 1),
            (6, 2), (7, 2), (8, 2),
            (6, 3), (7, 3), (8, 3),
        ] {
            let _ = gen.generate_chunk(cx, cz);
        }
        panic!("TREE-ATTEMPTS-DONE");
    }

    /// Frozen OCEAN_FLOOR vs live scan at the witness columns (s72 fix check).
    #[test]
    #[ignore = "diagnostic: frozen vs live OCEAN_FLOOR at witness columns"]
    fn frozen_ocean_floor_check_424242() {
        let gen = ChunkGenerator::new(424242);
        let mut region = crate::region_buf::RegionBuf::new(7, 2, 2);
        for dz in -2..=2i32 {
            for dx in -2..=2i32 {
                let (blocks, hm, _) = gen.generate_noise_and_surface(7 + dx, 2 + dz);
                region.put_chunk(7 + dx, 2 + dz, &blocks, &hm);
            }
        }
        crate::carvers::apply_carvers_region(&mut region, &gen.state);
        crate::mineshaft::apply_mineshafts_region(&mut region, &gen.state);
        region.freeze_ocean_floor();
        for (x, z) in [(102, 18), (107, 19), (100, 24)] {
            let frozen = region.ocean_floor_frozen_at(x, z).unwrap_or(-999);
            let live = crate::feature_dispatch::predicates::heightmap_top(
                &region, x, z, crate::feature_dispatch::predicates::HeightmapKind::OceanFloor,
            ).unwrap_or(-999);
            eprintln!("OF ({x},{z}): frozen={frozen} live={live}");
        }
        panic!("OF-DONE");
    }

    fn my_tree_census_424242() {
        let gen = ChunkGenerator::new(424242);
        for (cx, cz) in [(-14i32, -14), (-13, -14)] {
            let chunk = gen.generate_chunk(cx, cz);
            let mut log_count = 0usize;
            for y in crate::generator::WORLD_BOTTOM..crate::generator::WORLD_TOP {
                for lz in 0..16u32 {
                    for lx in 0..16u32 {
                        if chunk.block_at(lx, y, lz) == crate::surface::BlockId::DarkOakLog {
                            log_count += 1;
                        }
                    }
                }
            }
            eprintln!("MY-TREE-COUNT chunk ({cx},{cz}) dark_oak_logs = {log_count}");
        }
        panic!("CENSUS-DONE");
    }
}
