//! Underground ore, disk and magma features (generation step 6).
//!
//! Ports `OreFeature`, `DiskFeature`, `UnderwaterMagmaFeature` and the
//! placement chain (`count`, `rarity_filter`, `in_square`, `height_range`,
//! `OCEAN_FLOOR_WG`, water `matching_fluids`).
//!
//! FeatureSorter step-6 indices are explicit (`feature_index`). Biome filter
//! is sampled at the placement origin.
//!
//! Copyright (c) 2026 Neutron Contributors -- MIT License

use crate::biome_manager::biome_id_at_block;
use crate::biome_source::biome_id;
use crate::feature_rng::FeatureRandom;
use crate::generator::{WORLD_BOTTOM, WORLD_TOP};
use crate::region_buf::RegionBuf;
use crate::surface::BlockId;
use crate::worldgen::WorldgenState;
use serde_json::Value;

const STEP_UNDERGROUND_ORES: i32 = 6;
const PI: f32 = 3.1415927;

/// Apply underground ores for every chunk origin inside `region`.
///
/// Origin-major, center first (vanilla FEATURES order) with masking of the
/// not-yet-decorated origins — deterministic and matches a full WorldGenRegion
/// decoration pass over the area.
pub fn apply_underground_ores_region(region: &mut RegionBuf, level_seed: i64) {
    let order = crate::sculk::decoration_origin_order(region.chunks, region.origin_x, region.origin_z);
    for (pos, &(cxl, czl)) in order.iter().enumerate() {
        let origin_min_x = region.origin_x + cxl * 16;
        let origin_min_z = region.origin_z + czl * 16;
        apply_underground_ores_origin(
            region,
            level_seed,
            origin_min_x,
            origin_min_z,
            &order[pos + 1..],
        );
    }
}

/// Run the step-6 ore/disk pass for ONE chunk origin `(origin_min_x,
/// origin_min_z)`. `undecorated` are the origins after this one in the
/// decoration order: their feature output is masked to the terrain base for
/// the duration of the pass and restored afterwards (vanilla decorates each
/// origin while the not-yet-decorated neighbours are still at CARVERS).
pub(crate) fn apply_underground_ores_origin(
    region: &mut RegionBuf,
    level_seed: i64,
    origin_min_x: i32,
    origin_min_z: i32,
    undecorated: &[(i32, i32)],
) {
    let state = WorldgenState::overworld(level_seed);
    region.current_writer = crate::writers::MASK;
    let saved = crate::sculk::mask_undecorated_output(region, undecorated, crate::sculk::FAMILY_ALL);
    static ORES: std::sync::OnceLock<Vec<FeatureDef>> = std::sync::OnceLock::new();
    let ores = ORES.get_or_init(load_overworld_ores);
    let mut rng = FeatureRandom::new(level_seed);
    let decoration_seed = rng.set_decoration_seed(level_seed, origin_min_x, origin_min_z);
    for def in ores {
        rng.set_feature_seed(decoration_seed, def.feature_index, STEP_UNDERGROUND_ORES);
        if matches!(def.gate, BiomeGate::Off) {
            continue;
        }
        region.current_writer = match &def.kind {
            FeatureKind::Ore(_) => crate::writers::ORE,
            FeatureKind::Disk(_) => crate::writers::DISK,
            FeatureKind::UnderwaterMagma { .. } => crate::writers::UNDERWATER_MAGMA,
        };
        place_feature(&mut rng, region, &state, origin_min_x, origin_min_z, def);
    }
    region.current_writer = crate::writers::MASK;
    crate::sculk::restore_masked(region, saved);
    region.current_writer = crate::writers::TERRAIN;
}

// ---------------------------------------------------------------------------
// Feature definitions (from datapack placed_feature + configured_feature)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
enum CountSpec {
    Fixed(i32),
    Uniform { min: i32, max: i32 },
    Rarity(i32),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum HeightSpec {
    Uniform {
        min: HeightAnchor,
        max: HeightAnchor,
    },
    Trapezoid {
        min: HeightAnchor,
        max: HeightAnchor,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum HeightAnchor {
    Absolute(i32),
    AboveBottom(i32),
    BelowTop(i32),
}

impl HeightAnchor {
    fn resolve(self) -> i32 {
        match self {
            HeightAnchor::Absolute(y) => y,
            HeightAnchor::AboveBottom(n) => WORLD_BOTTOM + n,
            HeightAnchor::BelowTop(n) => (WORLD_TOP - 1) - n,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct FeatureDef {
    /// FeatureSorter global index for step 6.
    feature_index: i32,
    gate: BiomeGate,
    count: CountSpec,
    y: YSpec,
    kind: FeatureKind,
}

/// Vanilla `minecraft:biome` placement filter, sampled at the origin.
#[derive(Clone, Copy, Debug, PartialEq)]
enum BiomeGate {
    Any,
    /// Index reserved; no place (missing BlockId or colliding biome id).
    Off,
    Ids(&'static [u8]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum YSpec {
    Range {
        height: HeightSpec,
        /// `surface_relative_threshold_filter` vs OCEAN_FLOOR_WG first-available.
        max_relative_to_ocean_floor_wg: Option<i32>,
    },
    /// `heightmap` OCEAN_FLOOR_WG (`getHeight` = first available = solid+1).
    OceanFloorWg {
        require_water: bool,
        require_mud: bool,
        y_offset: i32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum FeatureKind {
    Ore(OreDef),
    Disk(DiskDef),
    UnderwaterMagma {
        floor_search_range: i32,
        probability: f32,
        radius: i32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct OreDef {
    size: i32,
    discard_chance: f32,
    stone_block: BlockId,
    deepslate_block: Option<BlockId>,
    target: TargetKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DiskDef {
    half_height: i32,
    radius_min: i32,
    radius_max: i32,
    target: DiskTarget,
    provider: DiskProvider,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DiskTarget {
    DirtOrClay,
    DirtOrGrass,
    DirtOrMud,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DiskProvider {
    Simple(BlockId),
    /// Sand; sandstone if the block below is air.
    SandOrSandstone,
    /// Grass if the block above is neither solid nor water; else dirt.
    GrassOrDirt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TargetKind {
    /// stone, granite, diorite, andesite
    StoneOre,
    /// stone + granite + diorite + andesite + tuff + deepslate
    BaseStone,
    /// deepslate + tuff only (unused for most; deepslate ores use dual target)
    DeepslateOre,
}



// ---------------------------------------------------------------------------
// Data-driven step-6 definitions (datapack placed_feature + configured_feature)
// ---------------------------------------------------------------------------

/// Overworld step-6 feature names in FeatureSorter order (= `feature_index`).
const OVERWORLD_ORE_NAMES: [&str; 34] = [
    "ore_dirt",
    "ore_gravel",
    "ore_granite_upper",
    "ore_granite_lower",
    "ore_diorite_upper",
    "ore_diorite_lower",
    "ore_andesite_upper",
    "ore_andesite_lower",
    "ore_tuff",
    "ore_coal_upper",
    "ore_coal_lower",
    "ore_iron_upper",
    "ore_iron_middle",
    "ore_iron_small",
    "ore_gold",
    "ore_gold_lower",
    "ore_redstone",
    "ore_redstone_lower",
    "ore_diamond",
    "ore_diamond_medium",
    "ore_diamond_large",
    "ore_diamond_buried",
    "ore_lapis",
    "ore_lapis_buried",
    "ore_copper_large",
    "ore_copper",
    "underwater_magma",
    "ore_clay",
    "ore_gold_extra",
    "disk_grass",
    "disk_sand",
    "disk_clay",
    "disk_gravel",
    "ore_emerald",
];

/// Biome membership that vanilla encodes in each biome's step-6 feature list.
/// These five are the only overworld ores/disks whose gate is not "any biome".
static GATE_OVERRIDES: &[(&str, BiomeGate)] = &[
    ("ore_copper_large", BiomeGate::Ids(&[biome_id::DRIPSTONE_CAVES])),
    ("ore_clay", BiomeGate::Ids(&[biome_id::LUSH_CAVES])),
    (
        "ore_gold_extra",
        BiomeGate::Ids(&[
            biome_id::BADLANDS,
            biome_id::ERODED_BADLANDS,
            biome_id::WOODED_BADLANDS,
        ]),
    ),
    ("disk_grass", BiomeGate::Ids(&[biome_id::MANGROVE_SWAMP])),
    // peaks/grove/meadow/…; BlockId lacks emerald ore → slot reserved
    ("ore_emerald", BiomeGate::Off),
];

fn parse_anchor(v: &Value) -> HeightAnchor {
    if let Some(n) = v.get("absolute").and_then(|x| x.as_i64()) {
        return HeightAnchor::Absolute(n as i32);
    }
    if let Some(n) = v.get("above_bottom").and_then(|x| x.as_i64()) {
        return HeightAnchor::AboveBottom(n as i32);
    }
    if let Some(n) = v.get("below_top").and_then(|x| x.as_i64()) {
        return HeightAnchor::BelowTop(n as i32);
    }
    panic!("unrecognized height anchor: {v}");
}

fn parse_height_spec(h: &Value) -> HeightSpec {
    let min = parse_anchor(&h["min_inclusive"]);
    let max = parse_anchor(&h["max_inclusive"]);
    match h["type"].as_str().unwrap_or("") {
        "minecraft:uniform" => HeightSpec::Uniform { min, max },
        "minecraft:trapezoid" => HeightSpec::Trapezoid { min, max },
        other => panic!("unrecognized height distribution: {other}"),
    }
}

fn parse_count(placements: &[Value]) -> CountSpec {
    let mut spec = CountSpec::Fixed(1); // vanilla default when no count modifier
    for m in placements {
        match m["type"].as_str().unwrap_or("") {
            "minecraft:count" => {
                let c = &m["count"];
                if let Some(n) = c.as_i64() {
                    spec = CountSpec::Fixed(n as i32);
                } else if c["type"].as_str() == Some("minecraft:uniform") {
                    spec = CountSpec::Uniform {
                        min: c["min_inclusive"].as_i64().unwrap() as i32,
                        max: c["max_inclusive"].as_i64().unwrap() as i32,
                    };
                }
            }
            "minecraft:rarity_filter" => {
                spec = CountSpec::Rarity(m["chance"].as_i64().unwrap() as i32);
            }
            _ => {}
        }
    }
    spec
}

fn parse_yspec(placements: &[Value]) -> YSpec {
    let mut yspec = YSpec::Range {
        height: HeightSpec::Uniform {
            min: HeightAnchor::Absolute(0),
            max: HeightAnchor::Absolute(0),
        },
        max_relative_to_ocean_floor_wg: None,
    };
    for m in placements {
        match m["type"].as_str().unwrap_or("") {
            "minecraft:height_range" => {
                yspec = YSpec::Range {
                    height: parse_height_spec(&m["height"]),
                    max_relative_to_ocean_floor_wg: None,
                };
            }
            "minecraft:surface_relative_threshold_filter" => {
                if let YSpec::Range { height, .. } = yspec {
                    yspec = YSpec::Range {
                        height,
                        max_relative_to_ocean_floor_wg: m["max_inclusive"]
                            .as_i64()
                            .map(|n| n as i32),
                    };
                }
            }
            "minecraft:heightmap" => {
                debug_assert_eq!(m["heightmap"].as_str(), Some("OCEAN_FLOOR_WG"));
                yspec = YSpec::OceanFloorWg {
                    require_water: false,
                    require_mud: false,
                    y_offset: 0,
                };
            }
            "minecraft:random_offset" => {
                if let YSpec::OceanFloorWg {
                    require_water,
                    require_mud,
                    ..
                } = yspec
                {
                    yspec = YSpec::OceanFloorWg {
                        require_water,
                        require_mud,
                        y_offset: m["y_spread"].as_i64().unwrap_or(0) as i32,
                    };
                }
            }
            "minecraft:block_predicate_filter" => {
                let p = &m["predicate"];
                if let YSpec::OceanFloorWg { require_water, require_mud, y_offset } = yspec {
                    if p["type"] == "minecraft:matching_fluids"
                        && p["fluids"] == "minecraft:water"
                    {
                        yspec = YSpec::OceanFloorWg { require_water: true, require_mud, y_offset };
                    }
                    if p["type"] == "minecraft:matching_blocks" {
                        let blocks = &p["blocks"];
                        let is_mud = |b: &Value| b.as_str() == Some("minecraft:mud");
                        let has_mud = blocks.as_str().is_some_and(|s| is_mud(&Value::String(s.into())))
                            || blocks
                                .as_array()
                                .is_some_and(|a| a.iter().any(is_mud));
                        if has_mud {
                            yspec = YSpec::OceanFloorWg { require_water, require_mud: true, y_offset };
                        }
                    }
                }
            }
            _ => {}
        }
    }
    yspec
}

fn block_from_state(v: &Value) -> Option<BlockId> {
    v.pointer("/state/Name")
        .and_then(|n| n.as_str())
        .and_then(BlockId::from_name)
}

fn parse_kind(cfg: &Value) -> FeatureKind {
    match cfg["type"].as_str().unwrap_or("") {
        "minecraft:ore" => {
            let targets = cfg["config"]["targets"].as_array().expect("ore targets");
            let stone_block = block_from_state(&targets[0])
                // ore_emerald: BlockId lacks emerald_ore; slot stays reserved
                // behind BiomeGate::Off with the same copper placeholder as
                // the old hand-written table.
                .unwrap_or_else(|| {
                    if targets[0]["state"]["Name"] == "minecraft:emerald_ore" {
                        BlockId::CopperOre
                    } else {
                        panic!("no BlockId for {}", targets[0]["state"]);
                    }
                });
            let deepslate_block = targets.get(1).and_then(block_from_state);
            let tag = targets[0]["target"]["tag"].as_str().unwrap_or("");
            let target = if tag == "minecraft:base_stone_overworld" {
                TargetKind::BaseStone
            } else {
                TargetKind::StoneOre
            };
            FeatureKind::Ore(OreDef {
                size: cfg["config"]["size"].as_i64().unwrap() as i32,
                discard_chance: cfg["config"]["discard_chance_on_air_exposure"]
                    .as_f64()
                    .unwrap() as f32,
                stone_block,
                deepslate_block,
                target,
            })
        }
        "minecraft:disk" => {
            let cfg = &cfg["config"];
            let provider = &cfg["state_provider"];
            let fallback = block_from_state(&provider["fallback"]);
            let simple = block_from_state(provider);
            // rule shape decides the two special providers; else Simple(fallback)
            let rules = provider["rules"].as_array();
            let disk_provider = if let Some(rules) = rules {
                let then_block = block_from_state(&rules[0]["then"]);
                match (fallback, then_block) {
                    (Some(BlockId::Sand), Some(BlockId::Sandstone)) => {
                        DiskProvider::SandOrSandstone
                    }
                    (Some(BlockId::Dirt), Some(BlockId::GrassBlock)) => DiskProvider::GrassOrDirt,
                    _ => panic!(
                        "unrecognized disk rules: fallback={fallback:?} then={then_block:?}"
                    ),
                }
            } else {
                DiskProvider::Simple(
                    block_from_state(provider).expect("disk simple provider"),
                )
            };
            let mut target_blocks: Vec<&str> = Vec::new();
            if let Some(arr) = cfg["target"]["blocks"].as_array() {
                for b in arr {
                    if let Some(s) = b.as_str() {
                        target_blocks.push(s);
                    }
                }
            } else if let Some(s) = cfg["target"]["blocks"].as_str() {
                target_blocks.push(s);
            }
            let has = |n: &str| target_blocks.iter().any(|b| b.ends_with(n));
            let target = if has("clay") {
                DiskTarget::DirtOrClay
            } else if has("grass_block") {
                DiskTarget::DirtOrGrass
            } else if has("mud") {
                DiskTarget::DirtOrMud
            } else {
                DiskTarget::DirtOrGrass
            };
            let radius = &cfg["radius"];
            FeatureKind::Disk(DiskDef {
                half_height: cfg["half_height"].as_i64().unwrap() as i32,
                radius_min: radius["min_inclusive"].as_i64().unwrap() as i32,
                radius_max: radius["max_inclusive"].as_i64().unwrap() as i32,
                target,
                provider: disk_provider,
            })
        }
        "minecraft:underwater_magma" => {
            let c = &cfg["config"];
            FeatureKind::UnderwaterMagma {
                floor_search_range: c["floor_search_range"].as_i64().unwrap() as i32,
                probability: c["placement_probability_per_valid_position"]
                    .as_f64()
                    .unwrap() as f32,
                radius: c["placement_radius_around_floor"].as_i64().unwrap() as i32,
            }
        }
        other => panic!("unsupported step-6 configured type: {other}"),
    }
}

/// Build the step-6 definitions from the embedded datapack JSONs.
pub fn load_overworld_ores() -> Vec<FeatureDef> {
    OVERWORLD_ORE_NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let placed = crate::feature_catalog::load_placed_feature(name)
                .unwrap_or_else(|| panic!("placed_feature/{name}.json"));
            let configured_name = placed["feature"]
                .as_str()
                .and_then(|s| s.strip_prefix("minecraft:"))
                .unwrap_or(name);
            let configured = crate::feature_catalog::load_configured_feature(configured_name)
                .unwrap_or_else(|| panic!("configured_feature/{configured_name}.json"));
            let placements = placed["placement"].as_array().expect("placement list");
            let gate = GATE_OVERRIDES
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, g)| *g)
                .unwrap_or(BiomeGate::Any);
            FeatureDef {
                feature_index: i as i32,
                gate,
                count: parse_count(placements),
                y: parse_yspec(placements),
                kind: parse_kind(configured),
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Placement + OreFeature
// ---------------------------------------------------------------------------

fn place_feature(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    state: &WorldgenState,
    origin_min_x: i32,
    origin_min_z: i32,
    def: &FeatureDef,
) {
    let attempts = match def.count {
        CountSpec::Fixed(n) => n,
        CountSpec::Uniform { min, max } => {
            if max < min {
                0
            } else {
                min + rng.next_int(max - min + 1)
            }
        }
        CountSpec::Rarity(chance) => {
            // 26.2 RarityFilter.shouldPlace: nextFloat() < 1.0f / chance
            // (not nextInt(chance) == 0 — that desyncs the rest of the chain).
            if chance > 0 && rng.next_f32() < 1.0 / chance as f32 {
                1
            } else {
                0
            }
        }
    };

    for _ in 0..attempts {
        let lx = rng.next_int(16);
        let lz = rng.next_int(16);
        let x = origin_min_x + lx;
        let z = origin_min_z + lz;
        let y = match def.y {
            YSpec::Range {
                height,
                max_relative_to_ocean_floor_wg,
            } => {
                let y = sample_height(rng, height);
                // NOTE: no early-out on out-of-world Y here — vanilla runs
                // OreFeature.place regardless and its per-cell
                // isOutsideBuildHeight skips writes; the blob draws MUST be
                // consumed either way or every later attempt of this feature
                // desyncs (deep diamond/redstone_lower samples go below -64).
                if let Some(max_rel) = max_relative_to_ocean_floor_wg {
                    let Some(surf) = ocean_floor_wg_first_available(region, x, z) else {
                        continue;
                    };
                    if y > surf + max_rel {
                        continue;
                    }
                }
                y
            }
            YSpec::OceanFloorWg {
                require_water,
                require_mud,
                y_offset,
            } => {
                let Some(base) = ocean_floor_wg_first_available(region, x, z) else {
                    continue;
                };
                let y = base + y_offset;
                if y < WORLD_BOTTOM || y >= WORLD_TOP {
                    continue;
                }
                if require_water && region.get(x, y, z) != BlockId::Water {
                    continue;
                }
                if require_mud && region.get(x, y, z) != BlockId::Mud {
                    continue;
                }
                y
            }
        };
        let pre = std::env::var_os("NEUTRON_ORE_TRACE").is_some();
        let _ = pre; // trace prints sampled position before gates below
        if pre {
            eprintln!(
                "{} ({},{},{}) @({},{}) draws={}",
                def.feature_index,
                x,
                y,
                z,
                origin_min_x,
                origin_min_z,
                rng.draw_count()
            );
        }
        if !biome_gate_ok(state, def.gate, def.feature_index, x, y, z) {
            continue;
        }
        let who = if std::env::var_os("NEUTRON_ORE_CELL").is_some() {
            format!(
                "{}@({},{}).{}",
                def.feature_index, origin_min_x, origin_min_z,
                std::env::var("NEUTRON_SCULK_ORIGIN_ORDER").unwrap_or_default()
            )
        } else {
            String::new()
        };
        match def.kind {
            FeatureKind::Ore(ore) => place_ore_blob(rng, region, x, y, z, &ore, &who),
            FeatureKind::Disk(disk) => place_disk(rng, region, x, y, z, &disk),
            FeatureKind::UnderwaterMagma {
                floor_search_range,
                probability,
                radius,
            } => place_underwater_magma(
                rng,
                region,
                x,
                y,
                z,
                floor_search_range,
                probability,
                radius,
            ),
        };
    }
}

/// `BiomeFilter.shouldPlace` + optional id allow-list.
fn biome_gate_ok(
    state: &WorldgenState,
    gate: BiomeGate,
    feature_index: i32,
    x: i32,
    y: i32,
    z: i32,
) -> bool {
    match gate {
        BiomeGate::Off => false,
        BiomeGate::Any | BiomeGate::Ids(_) => {
            if let BiomeGate::Ids(ids) = gate {
                if !ids.contains(&biome_id_at_block(state, x, y, z)) {
                    return false;
                }
            }
            let id = biome_id_at_block(state, x, y, z);
            let bname = crate::feature_dispatch::biome_id_to_name(id);
            let Some(placed) = crate::feature_catalog::features_per_step_at(STEP_UNDERGROUND_ORES)
                .get(feature_index as usize)
            else {
                return true;
            };
            crate::feature_catalog::features_at_step(bname, STEP_UNDERGROUND_ORES)
                .iter()
                .any(|f| {
                    f.strip_prefix("minecraft:").unwrap_or(f.as_str()) == placed.as_str()
                })
        }
    }
}

/// `Chunk.getHeight(OCEAN_FLOOR_WG, x, z)` = first available y = the topmost
/// `blocksMotion` block + 1 (`Heightmap.Types.OCEAN_FLOOR_WG` predicate is
/// `BlockState.blocksMotion`; `getFirstAvailable` returns `stored + minY`).
///
/// Reads the frozen post-carver snapshot (`RegionBuf::ocean_floor_frozen`),
/// taken at the CARVERS→FEATURES transition before decoration writes.
/// A live-read variant was MEASURED and REVERTED: the s73 experiment showed a
/// live scan beating the frozen snapshot for the tree/lush windows, but a
/// full-scan A/B of a live OCEAN_FLOOR map regressed 531,943 → 638,330
/// (+106k). The live map's interaction with origin-order writes is not
/// vanilla's: vanilla's heightmaps are per-chunk (each chunk's own map is
/// updated by writes to that chunk), while a shared live region map lets
/// earlier origins' spillover move later origins' gates.
/// Witness: seed 424242 column (-194,-163) — ref sand y58-61, was dirt (the
/// heightmaps-plane predicate counts powder_snow as the column top).
fn ocean_floor_wg_first_available(region: &RegionBuf, x: i32, z: i32) -> Option<i32> {
    let top = region.ocean_floor_frozen_at(x, z)?;
    if top <= WORLD_BOTTOM as i16 {
        return None;
    }
    Some(top as i32 + 1)
}

/// `DiskFeature.place` + `placeColumn`. Radius via UniformInt.
fn place_disk(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    ox: i32,
    oy: i32,
    oz: i32,
    def: &DiskDef,
) {
    let span = def.radius_max - def.radius_min + 1;
    let r = if span <= 0 {
        def.radius_min
    } else {
        def.radius_min + rng.next_int(span)
    };
    let top = oy + def.half_height;
    let bottom = oy - def.half_height - 1;
    for z in (oz - r)..=(oz + r) {
        for x in (ox - r)..=(ox + r) {
            let xd = x - ox;
            let zd = z - oz;
            if xd * xd + zd * zd > r * r {
                continue;
            }
            place_disk_column(region, def, top, bottom, x, z);
        }
    }
}

/// Generic config-driven disk placement (`DiskFeature` at any step, e.g.
/// `ice_patch` at step 4). Reads `half_height`, `radius` (uniform), the
/// `state_provider` and the `target` block predicate from the configured-feature
/// JSON. RNG: one `radius` sample, then per-column state draws (simple
/// providers consume none) — matches `DiskFeature.place` + `placeColumn`.
pub(crate) fn place_disk_from_config(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    ox: i32,
    oy: i32,
    oz: i32,
    cfg: &serde_json::Value,
) {
    let c = &cfg["config"];
    let half_height = c["half_height"].as_i64().unwrap_or(1) as i32;
    let radius = c["radius"].as_i64().map(|n| (n as i32, n as i32)).unwrap_or_else(|| {
        let min = c["radius"]["min_inclusive"].as_i64().unwrap_or(2) as i32;
        let max = c["radius"]["max_inclusive"].as_i64().unwrap_or(3) as i32;
        (min, max)
    });
    let r = if radius.1 <= radius.0 {
        radius.0
    } else {
        radius.0 + rng.next_int(radius.1 - radius.0 + 1)
    };
    let Some(state) =
        crate::feature_dispatch::block_from_to_place(rng, &c["state_provider"])
    else {
        return;
    };
    let top = oy + half_height;
    let bottom = oy - half_height - 1;
    for z in (oz - r)..=(oz + r) {
        for x in (ox - r)..=(ox + r) {
            let xd = x - ox;
            let zd = z - oz;
            if xd * xd + zd * zd > r * r {
                continue;
            }
            for y in (bottom + 1..=top).rev() {
                if region.index(x, y, z).is_none() {
                    continue;
                }
                if !crate::feature_dispatch::eval_block_predicate(region, x, y, z, &c["target"]) {
                    continue;
                }
                region.set(x, y, z, state);
            }
        }
    }
}

fn place_disk_column(region: &mut RegionBuf, def: &DiskDef, top: i32, bottom: i32, x: i32, z: i32) {
    for y in (bottom + 1..=top).rev() {
        if y < WORLD_BOTTOM || y >= WORLD_TOP {
            continue;
        }
        if region.index(x, y, z).is_none() {
            continue;
        }
        if !disk_target_match(region.get(x, y, z), def.target) {
            continue;
        }
        let Some(state) = disk_state(region, def.provider, x, y, z) else {
            continue;
        };
        region.set(x, y, z, state);
    }
}

fn disk_target_match(existing: BlockId, target: DiskTarget) -> bool {
    match target {
        DiskTarget::DirtOrClay => matches!(existing, BlockId::Dirt | BlockId::Clay),
        DiskTarget::DirtOrGrass => matches!(existing, BlockId::Dirt | BlockId::GrassBlock),
        DiskTarget::DirtOrMud => matches!(existing, BlockId::Dirt | BlockId::Mud),
    }
}

fn disk_state(
    region: &RegionBuf,
    provider: DiskProvider,
    x: i32,
    y: i32,
    z: i32,
) -> Option<BlockId> {
    match provider {
        DiskProvider::Simple(b) => Some(b),
        DiskProvider::SandOrSandstone => {
            if region.get(x, y - 1, z).is_air() {
                Some(BlockId::Sandstone)
            } else {
                Some(BlockId::Sand)
            }
        }
        DiskProvider::GrassOrDirt => {
            let above = region.get(x, y + 1, z);
            if !is_solid_predicate(above) && above != BlockId::Water {
                Some(BlockId::GrassBlock)
            } else {
                Some(BlockId::Dirt)
            }
        }
    }
}

fn is_solid_predicate(b: BlockId) -> bool {
    !matches!(
        b,
        BlockId::Air
            | BlockId::Water
            | BlockId::Lava
            | BlockId::ShortGrass
            | BlockId::LeafLitter
            | BlockId::SnowLayer
    )
}

/// `UnderwaterMagmaFeature.place`. Magma has no `BlockId` in surface.rs, so
/// valid positions are found and RNG is consumed, but the block is not written.
fn place_underwater_magma(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    ox: i32,
    oy: i32,
    oz: i32,
    floor_search_range: i32,
    probability: f32,
    radius: i32,
) {
    let Some(floor_y) = magma_floor_y(region, ox, oy, oz, floor_search_range) else {
        return;
    };
    for z in (oz - radius)..=(oz + radius) {
        for y in (floor_y - radius)..=(floor_y + radius) {
            for x in (ox - radius)..=(ox + radius) {
                if rng.next_f32() >= probability {
                    continue;
                }
                if !magma_valid_placement(region, x, y, z) {
                    continue;
                }
                // No Magma BlockId — leave the solid block in place.
            }
        }
    }
}

/// `Column.scan` with inside=water, edge=!water, then `Column.getFloor`.
fn magma_floor_y(region: &RegionBuf, x: i32, y: i32, z: i32, search_range: i32) -> Option<i32> {
    if region.get(x, y, z) != BlockId::Water {
        return None;
    }
    magma_scan_direction(region, x, y, z, search_range, -1)
}

fn magma_scan_direction(
    region: &RegionBuf,
    x: i32,
    start_y: i32,
    z: i32,
    search_range: i32,
    dy: i32,
) -> Option<i32> {
    let mut cy = start_y;
    let mut i = 1;
    while i < search_range && region.get(x, cy, z) == BlockId::Water {
        cy += dy;
        i += 1;
    }
    if region.get(x, cy, z) != BlockId::Water {
        Some(cy)
    } else {
        None
    }
}

fn magma_valid_placement(region: &RegionBuf, x: i32, y: i32, z: i32) -> bool {
    let here = region.get(x, y, z);
    if matches!(here, BlockId::Water | BlockId::Air) {
        return false;
    }
    if magma_visible_from_outside(region.get(x, y - 1, z)) {
        return false;
    }
    for (dx, dz) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
        if magma_visible_from_outside(region.get(x + dx, y, z + dz)) {
            return false;
        }
    }
    true
}

fn magma_visible_from_outside(b: BlockId) -> bool {
    matches!(
        b,
        BlockId::Air
            | BlockId::Water
            | BlockId::Lava
            | BlockId::ShortGrass
            | BlockId::LeafLitter
            | BlockId::OakLeaves
            | BlockId::DarkOakLeaves
            | BlockId::SnowLayer
            | BlockId::SculkVein
    )
}

fn sample_height(rng: &mut FeatureRandom, h: HeightSpec) -> i32 {
    match h {
        HeightSpec::Uniform { min, max } => {
            let lo = min.resolve();
            let hi = max.resolve();
            if hi <= lo {
                return lo;
            }
            // Mth.randomBetweenInclusive(random, lo, hi)
            lo + rng.next_int(hi - lo + 1)
        }
        HeightSpec::Trapezoid { min, max } => {
            // Vanilla TrapezoidHeight with plateau=0:
            //   range = max - min
            //   bottom = range / 2; top = range - bottom
            //   return min + randomBetweenInclusive(0, top) + randomBetweenInclusive(0, bottom)
            let lo = min.resolve();
            let hi = max.resolve();
            if hi <= lo {
                return lo;
            }
            let range = hi - lo;
            let bottom = range / 2;
            let top = range - bottom;
            lo + rng.next_int(top + 1) + rng.next_int(bottom + 1)
        }
    }
}

/// `OreFeature.place` + `doPlace` into a multi-chunk region.
fn place_ore_blob(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    ox: i32,
    oy: i32,
    oz: i32,
    def: &OreDef,
    who: &str,
) {
    let size = def.size;
    if size <= 0 {
        return;
    }
    let target_fn = |existing: BlockId| target_match(existing, def);
    place_ore_blob_inner(rng, region, ox, oy, oz, size, def.discard_chance, target_fn, who);
}

/// Generic config-driven ore placement (`OreFeature` at any step, e.g.
/// `ore_infested` at step 7). Reads `size`, `discard_chance_on_air_exposure`
/// and the ordered `targets` array (state + tag_match predicate) from the
/// configured-feature JSON, then runs the same blob algorithm as the batch.
pub(crate) fn place_ore_from_config(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    ox: i32,
    oy: i32,
    oz: i32,
    cfg: &serde_json::Value,
) {
    let c = &cfg["config"];
    let size = c["size"].as_i64().unwrap_or(0) as i32;
    let discard = c["discard_chance_on_air_exposure"].as_f64().unwrap_or(0.0) as f32;
    if size <= 0 {
        return;
    }
    // Ordered first-match targets (vanilla iterates targetStates in order).
    let mut targets: Vec<(BlockId, TargetKind)> = Vec::new();
    if let Some(arr) = c["targets"].as_array() {
        for t in arr {
            let state = t["state"]["Name"]
                .as_str()
                .and_then(crate::surface::BlockId::from_name);
            let Some(state) = state else { continue };
            let tag = t["target"]["tag"].as_str().unwrap_or("");
            let kind = match tag {
                "minecraft:deepslate_ore_replaceables" => TargetKind::DeepslateOre,
                "minecraft:base_stone_overworld" => TargetKind::BaseStone,
                _ => TargetKind::StoneOre,
            };
            targets.push((state, kind));
        }
    }
    if targets.is_empty() {
        return;
    }
    let target_fn = move |existing: BlockId| -> Option<BlockId> {
        for (state, kind) in &targets {
            let hits = match kind {
                TargetKind::StoneOre => is_stone_ore_replaceable(existing),
                TargetKind::BaseStone => is_base_stone(existing),
                TargetKind::DeepslateOre => is_deepslate_ore_replaceable(existing),
            };
            if hits {
                return Some(*state);
            }
        }
        None
    };
    place_ore_blob_inner(rng, region, ox, oy, oz, size, discard, target_fn, "");
}

/// Shared blob math: `OreFeature.place` + `doPlace` with a target resolver.
fn place_ore_blob_inner(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    ox: i32,
    oy: i32,
    oz: i32,
    size: i32,
    discard_chance: f32,
    target_fn: impl Fn(BlockId) -> Option<BlockId>,
    who: &str,
) {
    if size <= 0 {
        return;
    }
    let angle = rng.next_f32() * PI;
    let f = size as f32 / 8.0;
    // Match Java: Math.sin((double)angle) after float angle
    let start_x = ox as f64 + (angle as f64).sin() * f as f64;
    let end_x = ox as f64 - (angle as f64).sin() * f as f64;
    let start_z = oz as f64 + (angle as f64).cos() * f as f64;
    let end_z = oz as f64 - (angle as f64).cos() * f as f64;
    let start_y = (oy + rng.next_int(3) - 2) as f64;
    let end_y = (oy + rng.next_int(3) - 2) as f64;

    // Bounding box is computed in `place` *before* doPlace. If no column in the
    // box has OCEAN_FLOOR_WG >= minY, vanilla returns false without consuming
    // the per-sphere nextDouble stream.
    let cell = ((size as f32 / 16.0 * 2.0 + 1.0) / 2.0).ceil() as i32;
    let f_ceil = f.ceil() as i32;
    let start_block_x = ox - f_ceil - cell;
    let start_block_y = oy - 2 - cell;
    let start_block_z = oz - f_ceil - cell;
    let size_xz = 2 * (f_ceil + cell);
    let size_y = 2 * (2 + cell);
    if !ocean_floor_wg_allows_ore(region, start_block_x, start_block_y, start_block_z, size_xz) {
        return;
    }

    // Sphere path samples (`doPlace`)
    let mut spheres = vec![0f64; (size as usize) * 4];
    for i in 0..size {
        // Vanilla: float t = (float)i / (float)size; then (double)t into lerp.
        let t = i as f32 / size as f32;
        let td = t as f64;
        let sx = lerp(td, start_x, end_x);
        let sy = lerp(td, start_y, end_y);
        let sz = lerp(td, start_z, end_z);
        let blip = rng.next_f64() * size as f64 / 16.0;
        // Java: ((Mth.sin((double)(PI * t)) + 1.0f) * blip + 1.0) / 2.0
        let sin_part = (crate::carvers::mth_sin_d((PI * t) as f64) + 1.0) as f64;
        let radius = (sin_part * blip + 1.0) / 2.0;
        let base = (i as usize) * 4;
        spheres[base] = sx;
        spheres[base + 1] = sy;
        spheres[base + 2] = sz;
        spheres[base + 3] = radius;
    }

    // Remove overlapping spheres (bytecode cull)
    for i in 0..size - 1 {
        let bi = (i as usize) * 4;
        if spheres[bi + 3] <= 0.0 {
            continue;
        }
        for j in (i + 1)..size {
            let bj = (j as usize) * 4;
            if spheres[bj + 3] <= 0.0 {
                continue;
            }
            let dx = spheres[bi] - spheres[bj];
            let dy = spheres[bi + 1] - spheres[bj + 1];
            let dz = spheres[bi + 2] - spheres[bj + 2];
            let dr = spheres[bi + 3] - spheres[bj + 3];
            if dr * dr > dx * dx + dy * dy + dz * dz {
                if dr > 0.0 {
                    spheres[bj + 3] = -1.0;
                } else {
                    spheres[bi + 3] = -1.0;
                }
            }
        }
    }

    let mut bitset = vec![false; (size_xz * size_y * size_xz).max(1) as usize];

    for i in 0..size {
        let bi = (i as usize) * 4;
        let radius = spheres[bi + 3];
        if radius < 0.0 {
            continue;
        }
        let cx = spheres[bi];
        let cy = spheres[bi + 1];
        let cz = spheres[bi + 2];
        let min_x = floor(cx - radius).max(start_block_x);
        let min_y = floor(cy - radius).max(start_block_y);
        let min_z = floor(cz - radius).max(start_block_z);
        let max_x = floor(cx + radius).max(min_x);
        let max_y = floor(cy + radius).max(min_y);
        let max_z = floor(cz + radius).max(min_z);

        for x in min_x..=max_x {
            let dx = ((x as f64 + 0.5) - cx) / radius;
            if dx * dx >= 1.0 {
                continue;
            }
            for y in min_y..=max_y {
                if y < WORLD_BOTTOM || y >= WORLD_TOP {
                    continue;
                }
                let dy = ((y as f64 + 0.5) - cy) / radius;
                if dx * dx + dy * dy >= 1.0 {
                    continue;
                }
                for z in min_z..=max_z {
                    let dz = ((z as f64 + 0.5) - cz) / radius;
                    if dx * dx + dy * dy + dz * dz >= 1.0 {
                        continue;
                    }
                    let bit = ((x - start_block_x)
                        + (y - start_block_y) * size_xz
                        + (z - start_block_z) * size_xz * size_y)
                        as usize;
                    if bit >= bitset.len() || bitset[bit] {
                        continue;
                    }
                    bitset[bit] = true;

                    if region.index(x, y, z).is_none() {
                        continue;
                    }
                    let existing = region.get(x, y, z);
                    let Some(replacement) = target_fn(existing) else {
                        continue;
                    };
                    // OreFeature.canPlaceOre: shouldSkipAirCheck *then* isAdjacentToAir.
                    if !can_place_ore(region, x, y, z, discard_chance, rng) {
                        continue;
                    }
                    if let Some(cell) = std::env::var_os("NEUTRON_ORE_CELL") {
                        let want: Vec<i32> = cell
                            .to_string_lossy()
                            .split(',')
                            .filter_map(|s| s.parse().ok())
                            .collect();
                        if want.len() == 3 && (x, y, z) == (want[0], want[1], want[2]) {
                            eprintln!(
                                "[cell-write] ({x},{y},{z}) {existing:?} -> {} by [{who}]",
                                replacement.block_name()
                            );
                        }
                    }
                    region.set(x, y, z, replacement);
                }
            }
        }
    }
}

fn target_match(existing: BlockId, def: &OreDef) -> Option<BlockId> {
    match def.target {
        TargetKind::StoneOre => {
            if is_stone_ore_replaceable(existing) {
                Some(def.stone_block)
            } else if is_deepslate_ore_replaceable(existing) {
                Some(def.deepslate_block.unwrap_or(def.stone_block))
            } else {
                None
            }
        }
        TargetKind::BaseStone => {
            if is_base_stone(existing) {
                Some(def.stone_block)
            } else {
                None
            }
        }
        TargetKind::DeepslateOre => {
            if is_deepslate_ore_replaceable(existing) {
                Some(def.deepslate_block.unwrap_or(def.stone_block))
            } else {
                None
            }
        }
    }
}

fn is_stone_ore_replaceable(b: BlockId) -> bool {
    matches!(
        b,
        BlockId::Stone | BlockId::Granite | BlockId::Diorite | BlockId::Andesite
    )
}

fn is_deepslate_ore_replaceable(b: BlockId) -> bool {
    matches!(b, BlockId::Deepslate | BlockId::Tuff)
}

fn is_base_stone(b: BlockId) -> bool {
    matches!(
        b,
        BlockId::Stone
            | BlockId::Granite
            | BlockId::Diorite
            | BlockId::Andesite
            | BlockId::Tuff
            | BlockId::Deepslate
    )
}

/// `OreFeature.place` heightmap gate: first column with
/// `minY <= getHeight(OCEAN_FLOOR_WG, x, z)` → allow `doPlace`.
///
/// The map read is [`RegionBuf::ocean_floor_frozen_at`] — the post-carver
/// snapshot, NOT the `heightmaps` plane (which is a "not air/fluid" scan: it
/// counts `powder_snow` and other non-motion blocks as the column top).
/// Witness: seed 424242 column (-194,-163) — ref sand y58-61,
/// `disk_sand`'s placement read a y one block too high and its
/// `matching_fluids water` filter then failed.
fn ocean_floor_wg_allows_ore(
    region: &RegionBuf,
    start_x: i32,
    start_y: i32,
    start_z: i32,
    size_xz: i32,
) -> bool {
    for x in start_x..=start_x + size_xz {
        for z in start_z..=start_z + size_xz {
            if let Some(h) = ocean_floor_wg_first_available(region, x, z) {
                if start_y <= h {
                    return true;
                }
            }
        }
    }
    false
}

/// `OreFeature.canPlaceOre` after the RuleTest already matched.
///
/// `shouldSkipAirCheck` runs (and may consume `nextFloat`) *before*
/// `isAdjacentToAir`. Fluids are not air (`BlockState.isAir()`).
fn can_place_ore(
    region: &RegionBuf,
    x: i32,
    y: i32,
    z: i32,
    chance: f32,
    rng: &mut FeatureRandom,
) -> bool {
    if should_skip_air_check(rng, chance) {
        return true;
    }
    !is_adjacent_to_air(region, x, y, z)
}

/// `OreFeature.shouldSkipAirCheck`.
fn should_skip_air_check(rng: &mut FeatureRandom, chance: f32) -> bool {
    if chance <= 0.0 {
        return true;
    }
    if chance >= 1.0 {
        return false;
    }
    rng.next_f32() >= chance
}

/// `Feature.isAdjacentToAir` — `BlockState.isAir()` on all 6 faces.
fn is_adjacent_to_air(region: &RegionBuf, x: i32, y: i32, z: i32) -> bool {
    neighbor_is_air(region, x + 1, y, z)
        || neighbor_is_air(region, x - 1, y, z)
        || neighbor_is_air(region, x, y + 1, z)
        || neighbor_is_air(region, x, y - 1, z)
        || neighbor_is_air(region, x, y, z + 1)
        || neighbor_is_air(region, x, y, z - 1)
}

fn neighbor_is_air(region: &RegionBuf, x: i32, y: i32, z: i32) -> bool {
    if y < WORLD_BOTTOM || y >= WORLD_TOP {
        return true;
    }
    if region.index(x, y, z).is_none() {
        return false;
    }
    region.get(x, y, z).is_air()
}

#[inline]
fn lerp(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

#[inline]
fn floor(v: f64) -> i32 {
    v.floor() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step6_feature_indices_are_explicit_0_to_33() {
        let ores = load_overworld_ores();
        let idxs: Vec<i32> = ores.iter().map(|d| d.feature_index).collect();
        assert_eq!(idxs, (0..=33).collect::<Vec<i32>>());
        assert!(matches!(ores[24].gate, BiomeGate::Ids(_))); // copper_large
        assert!(matches!(ores[25].gate, BiomeGate::Any)); // copper
        assert!(matches!(ores[26].gate, BiomeGate::Any)); // underwater_magma
        assert!(matches!(ores[27].gate, BiomeGate::Ids(_))); // clay
        assert!(matches!(ores[28].gate, BiomeGate::Ids(_))); // gold_extra
        assert!(matches!(ores[29].gate, BiomeGate::Ids(_))); // disk_grass
        assert!(matches!(ores[30].gate, BiomeGate::Any)); // disk_sand
        assert!(matches!(ores[31].gate, BiomeGate::Any)); // disk_clay
        assert!(matches!(ores[32].gate, BiomeGate::Any)); // disk_gravel
        assert!(matches!(ores[33].gate, BiomeGate::Off)); // emerald
        assert!(matches!(ores[30].kind, FeatureKind::Disk(_)));
        assert!(matches!(ores[31].kind, FeatureKind::Disk(_)));
        assert!(matches!(ores[32].kind, FeatureKind::Disk(_)));
        assert!(matches!(
            ores[26].kind,
            FeatureKind::UnderwaterMagma { .. }
        ));
    }

    #[test]
    fn should_skip_air_check_matches_vanilla_consumption() {
        // chance <= 0 → true, no draw; chance >= 1 → false, no draw.
        let mut rng = FeatureRandom::new(1);
        let before = rng.next_int(16);
        let mut rng = FeatureRandom::new(1);
        assert!(should_skip_air_check(&mut rng, 0.0));
        assert!(!should_skip_air_check(&mut rng, 1.0));
        assert_eq!(rng.next_int(16), before);

        // 0 < chance < 1 consumes nextFloat; skip iff nextFloat >= chance.
        let mut a = FeatureRandom::new(99);
        let mut b = FeatureRandom::new(99);
        let skip = should_skip_air_check(&mut a, 0.5);
        let drawn = b.next_f32();
        assert_eq!(skip, drawn >= 0.5);
        assert_eq!(a.next_int(8), b.next_int(8));
    }

    #[test]
    fn rarity_filter_uses_next_float_not_next_int() {
        // ProbeRarity vs 26.2: setFeatureSeed(dec(12345,96,-32), 6, 6)
        // nextFloat = 0.42050785 >= 1/6 → do not place.
        let mut rng = FeatureRandom::new(12345);
        let dec = rng.set_decoration_seed(12345, 96, -32);
        rng.set_feature_seed(dec, 6, 6);
        let f = rng.next_f32();
        assert!((f - 0.42050785).abs() < 1e-6, "got {f}");
        assert!(f >= 1.0 / 6.0);
    }
}

