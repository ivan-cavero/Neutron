//! Block predicates + heightmap helpers + biome id/name mapping.
use super::*;
use crate::feature_catalog;
use crate::feature_rng::FeatureRandom;
use crate::generator::{WORLD_BOTTOM, WORLD_TOP};
use crate::legacy_rng::LegacyRandom;
use crate::region_buf::RegionBuf;
use crate::sculk;
use crate::surface::BlockId;
use crate::worldgen::WorldgenState;
use serde_json::Value;


pub(crate) fn eval_block_predicate(
    region: &RegionBuf,
    x: i32,
    y: i32,
    z: i32,
    pred: &Value,
) -> bool {
    let ty = pred["type"].as_str().unwrap_or("");
    match ty {
        "minecraft:matching_block_tag" => {
            let tag = pred["tag"].as_str().unwrap_or("");
            let b = region.get(x, y, z);
            if tag.ends_with("air") {
                // `#minecraft:air` = {air, cave_air, void_air}. void_air never
                // enters the region buffer (carvers write air/cave_air), but a
                // cave-air cell must pass an "is air" filter like vanilla.
                return b == BlockId::Air || b == BlockId::CaveAir;
            }
            is_in_tag(b, tag)
        }
        "minecraft:solid" => is_solid_block(region.get(x, y, z)),
        "minecraft:has_sturdy_face" => {
            // HasSturdyFacePredicate: the block at (origin + predicate offset)
            // must have a sturdy face in `direction`. We approximate "sturdy"
            // with is_solid_block (full solid blocks are sturdy on all faces).
            let off = pred["offset"].as_array();
            let ox = off
                .and_then(|a| a.first())
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oy = off
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oz = off
                .and_then(|a| a.get(2))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            is_solid_block(region.get(x + ox, y + oy, z + oz))
        }
        "minecraft:not" => !pred
            .get("predicate")
            .map(|p| eval_block_predicate(region, x, y, z, p))
            .unwrap_or(false),
        "minecraft:matching_blocks" => {
            let ox = pred["offset"]
                .as_array()
                .and_then(|a| a.first())
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oy = pred["offset"]
                .as_array()
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oz = pred["offset"]
                .as_array()
                .and_then(|a| a.get(2))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let b = region.get(x + ox, y + oy, z + oz);
            if let Some(name) = pred["blocks"].as_str() {
                return BlockId::from_name(name) == Some(b);
            }
            if let Some(arr) = pred["blocks"].as_array() {
                return arr.iter().any(|n| {
                    n.as_str()
                        .and_then(BlockId::from_name)
                        .map(|id| id == b)
                        .unwrap_or(false)
                });
            }
            true
        }
        "minecraft:all_of" => {
            let Some(arr) = pred["predicates"].as_array() else {
                return true;
            };
            arr.iter().all(|p| eval_block_predicate(region, x, y, z, p))
        }
        "minecraft:any_of" => {
            let Some(arr) = pred["predicates"].as_array() else {
                return true;
            };
            arr.iter().any(|p| eval_block_predicate(region, x, y, z, p))
        }
        "minecraft:would_survive" => {
            // WouldSurvivePredicate: `state.canSurvive(level, pos)` =
            // SaplingBlock.canSurvive → mayPlaceOn(below) only (no check on the
            // position cell itself). mayPlaceOn = BlockTags.SUPPORTS_VEGETATION
            // (server-classes/data/minecraft/tags/block/supports_vegetation.json:
            //  #substrate_overworld + farmland;
            //  substrate_overworld = #dirt + #mud + #moss_blocks + #grass_blocks;
            //  dirt = dirt, coarse_dirt, rooted_dirt; mud = mud,
            //  muddy_mangrove_roots; moss_blocks = moss_block, pale_moss_block;
            //  grass_blocks = grass_block, podzol, mycelium). Farmland has no
            //  BlockId in surface.rs (surface trees never sit on farmland).
            // Use get_buffered to see pending writes from same decoration step.
            let below = region.get_buffered(x, y - 1, z);
            supports_vegetation(below)
        }
        "minecraft:matching_fluids" => {
            // MatchingFluidsPredicate: state.getFluidState().is(fluids) at
            // (origin + offset). A block has a fluid state only for
            // water/lava; every other block's fluid state is `empty`, so
            // `"fluids": "minecraft:empty"` passes exactly when the cell
            // holds no liquid (patch_melon). Missing `fluids` keeps the
            // legacy water-only behavior. The region buffer stores a single
            // Water id (no flow level), so flowing_water matches Water.
            let off = pred["offset"].as_array();
            let ox = off
                .and_then(|a| a.first())
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oy = off
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oz = off
                .and_then(|a| a.get(2))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let b = region.get(x + ox, y + oy, z + oz);
            let mut want: Vec<&str> = Vec::new();
            if let Some(s) = pred["fluids"].as_str() {
                want.push(s);
            } else if let Some(arr) = pred["fluids"].as_array() {
                want.extend(arr.iter().filter_map(|v| v.as_str()));
            }
            if want.is_empty() {
                return b == BlockId::Water;
            }
            if want.iter().any(|f| *f == "minecraft:empty") {
                return b != BlockId::Water && b != BlockId::Lava;
            }
            if b == BlockId::Water {
                want.iter().any(|f| f.ends_with("water"))
            } else if b == BlockId::Lava {
                want.iter().any(|f| f.ends_with("lava"))
            } else {
                false
            }
        }
        "minecraft:replaceable" => {
            // ReplaceablePredicate: state.canBeReplaced() at (origin+offset).
            let off = pred["offset"].as_array();
            let ox = off
                .and_then(|a| a.first())
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oy = off
                .and_then(|a| a.get(1))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            let oz = off
                .and_then(|a| a.get(2))
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;
            super::can_be_replaced(region.get(x + ox, y + oy, z + oz))
        }
        "minecraft:true" => true,
        _ => true,
    }
}

/// `BlockTags.SUPPORTS_VEGETATION` — what a sapling may place on.
pub(super) fn supports_vegetation(b: BlockId) -> bool {
    matches!(
        b,
        BlockId::Dirt
            | BlockId::CoarseDirt
            | BlockId::RootedDirt
            | BlockId::GrassBlock
            | BlockId::Podzol
            | BlockId::Mycelium
            | BlockId::Mud
            | BlockId::MossBlock
            | BlockId::PaleMossBlock
    )
}

/// `BlockState.isSolid()` approximation (mirrors `blocks_motion`).
pub(crate) fn is_solid_block(b: BlockId) -> bool {
    blocks_motion(b)
}

/// `SmallDripleafBlock.mayPlaceOn`: `#supports_small_dripleaf` (clay, moss)
/// or water source at the plant cell + `VegetationBlock.mayPlaceOn` (dirt).
pub(super) fn small_dripleaf_may_place_on(below: BlockId, at: BlockId) -> bool {
    matches!(below, BlockId::Clay | BlockId::MossBlock)
        || (at == BlockId::Water && supports_vegetation(below))
}

/// Membership in a block tag (subset used by lush/pale placement predicates).
pub(crate) fn is_in_tag(b: BlockId, tag: &str) -> bool {
    let t = tag.strip_prefix("#minecraft:").unwrap_or(tag);
    match t {
        // `#minecraft:air` = {air, cave_air} here (void_air is unreachable in
        // the decoration buffer; see matching_block_tag shortcut above).
        "air" => b == BlockId::Air || b == BlockId::CaveAir,
        "cave_vines" => matches!(b, BlockId::CaveVines | BlockId::CaveVinesPlant),
        "dirt" => matches!(
            b,
            BlockId::Dirt
                | BlockId::GrassBlock
                | BlockId::Podzol
                | BlockId::CoarseDirt
                | BlockId::Mycelium
                | BlockId::RootedDirt
        ),
        "moss_blocks" => matches!(b, BlockId::MossBlock | BlockId::PaleMossBlock),
        "grass_blocks" => b == BlockId::GrassBlock,
        "mud" => b == BlockId::Mud,
        "base_stone_overworld" => matches!(
            b,
            BlockId::Stone
                | BlockId::Granite
                | BlockId::Diorite
                | BlockId::Andesite
                | BlockId::Tuff
                | BlockId::Deepslate
        ),
        "moss_replaceable" => {
            is_in_tag(b, "#minecraft:base_stone_overworld")
                || is_in_tag(b, "#minecraft:cave_vines")
                || is_in_tag(b, "#minecraft:dirt")
                || is_in_tag(b, "#minecraft:mud")
                || is_in_tag(b, "#minecraft:moss_blocks")
                || is_in_tag(b, "#minecraft:grass_blocks")
        }
        "lush_ground_replaceable" => {
            is_in_tag(b, "#minecraft:moss_replaceable")
                || b == BlockId::Clay
                || b == BlockId::Gravel
                || b == BlockId::Sand
        }
        "sand" => matches!(b, BlockId::Sand | BlockId::RedSand),
        "terracotta" => matches!(
            b,
            BlockId::Terracotta
                | BlockId::WhiteTerracotta
                | BlockId::OrangeTerracotta
                | BlockId::BrownTerracotta
                | BlockId::BlackTerracotta
                | BlockId::YellowTerracotta
                | BlockId::RedTerracotta
                | BlockId::LightGrayTerracotta
        ),
        // substrate_overworld = dirt/grass/podzol/coarse_dirt/mycelium/
        // rooted_dirt/moss_block/pale_moss_block/mud (26.2).
        "substrate_overworld" => {
            is_in_tag(b, "#minecraft:dirt")
                || is_in_tag(b, "#minecraft:moss_blocks")
                || is_in_tag(b, "#minecraft:grass_blocks")
                || is_in_tag(b, "#minecraft:mud")
        }
        // PlantBlock.mayPlaceOn: below ∈ #supports_vegetation (26.2) =
        // #substrate_overworld ∪ farmland (farmland never occurs in worldgen).
        "supports_vegetation" => is_in_tag(b, "#minecraft:substrate_overworld"),
        // AzaleaBlock.mayPlaceOn = #supports_azalea = supports_vegetation ∪ clay.
        "supports_azalea" => {
            is_in_tag(b, "#minecraft:supports_vegetation") || b == BlockId::Clay
        }
        "azalea_grows_on" => {
            is_in_tag(b, "#minecraft:substrate_overworld")
                || is_in_tag(b, "#minecraft:sand")
                || is_in_tag(b, "#minecraft:terracotta")
                || b == BlockId::Snow
                || b == BlockId::PowderSnow
        }
        "azalea_root_replaceable" => {
            is_in_tag(b, "#minecraft:base_stone_overworld")
                || is_in_tag(b, "#minecraft:substrate_overworld")
                || is_in_tag(b, "#minecraft:terracotta")
                || b == BlockId::RedSand
                || b == BlockId::Clay
                || b == BlockId::Gravel
                || b == BlockId::Sand
                || b == BlockId::Snow
                || b == BlockId::PowderSnow
        }
        _ => false,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HeightmapKind {
    /// `WORLD_SURFACE` / `WORLD_SURFACE_WG`: Heightmap.NOT_AIR.
    WorldSurface,
    /// `OCEAN_FLOOR` / `OCEAN_FLOOR_WG`: `BlockState.blocksMotion()`.
    OceanFloor,
    /// `MOTION_BLOCKING`: blocksMotion || !fluid.isEmpty.
    MotionBlocking,
    /// `MOTION_BLOCKING_NO_LEAVES`: (blocksMotion || fluid) && !LeavesBlock.
    MotionBlockingNoLeaves,
}

pub(crate) fn parse_heightmap_kind(name: &str) -> HeightmapKind {
    match name.strip_prefix("minecraft:").unwrap_or(name).to_ascii_lowercase().as_str() {
        "world_surface" | "world_surface_wg" => HeightmapKind::WorldSurface,
        "ocean_floor" | "ocean_floor_wg" => HeightmapKind::OceanFloor,
        "motion_blocking" => HeightmapKind::MotionBlocking,
        "motion_blocking_no_leaves" => HeightmapKind::MotionBlockingNoLeaves,
        _ => HeightmapKind::OceanFloor,
    }
}

/// `BlockState.blocksMotion` = `isSolid` (except cobweb / bamboo_sapling,
/// not in palette). Plants (grass, carpet, vines, hanging moss, azalea),
/// fluids, snow and veins are NOT solid.
pub(crate) fn blocks_motion(b: BlockId) -> bool {
    !matches!(
        b,
        BlockId::Air
            | BlockId::CaveAir
            | BlockId::Water
            | BlockId::Lava
            | BlockId::ShortGrass
            | BlockId::TallGrass
            | BlockId::LeafLitter
            | BlockId::SnowLayer
            | BlockId::PowderSnow
            | BlockId::PaleMossCarpet
            | BlockId::PaleMossCarpetTopper
            | BlockId::MossCarpet
            | BlockId::CaveVines
            | BlockId::CaveVinesPlant
            | BlockId::PaleHangingMoss
            | BlockId::HangingRoots
            | BlockId::Azalea
            | BlockId::FloweringAzalea
            | BlockId::SmallDripleaf
            | BlockId::BigDripleaf
            | BlockId::BigDripleafStem
            | BlockId::Vine
            | BlockId::GlowLichen
            // Palette-completeness sweep: plants and non-collidable decor.
            | BlockId::Dandelion
            | BlockId::Poppy
            | BlockId::AzureBluet
            | BlockId::Allium
            | BlockId::Cornflower
            | BlockId::OxeyeDaisy
            | BlockId::OrangeTulip
            | BlockId::PinkTulip
            | BlockId::RedTulip
            | BlockId::WhiteTulip
            | BlockId::LilyOfTheValley
            | BlockId::RoseBush
            | BlockId::Lilac
            | BlockId::Peony
            | BlockId::Sunflower
            | BlockId::PinkPetals
            | BlockId::Wildflowers
            | BlockId::Bush
            | BlockId::FireflyBush
            | BlockId::ShortDryGrass
            | BlockId::TallDryGrass
            | BlockId::ClosedEyeblossom
            | BlockId::Fern
            | BlockId::LargeFern
            | BlockId::DeadBush
            | BlockId::SugarCane
            | BlockId::LilyPad
            | BlockId::RedMushroom
            | BlockId::BrownMushroom
            | BlockId::SporeBlossom
            | BlockId::CactusFlower
            | BlockId::CrimsonRoots
            | BlockId::WarpedRoots
            | BlockId::NetherSprouts
            | BlockId::Fire
            | BlockId::SoulFire
            | BlockId::MangrovePropagule
            | BlockId::OakSapling
            | BlockId::BirchSapling
            | BlockId::SpruceSapling
            | BlockId::JungleSapling
            | BlockId::AcaciaSapling
            | BlockId::DarkOakSapling
            | BlockId::CherrySapling
            | BlockId::PaleOakSapling
    )
}

fn is_leaves(b: BlockId) -> bool {
    matches!(
        b,
        BlockId::OakLeaves | BlockId::DarkOakLeaves | BlockId::PaleOakLeaves
    )
}

pub(super) fn heightmap_opaque(b: BlockId, kind: HeightmapKind) -> bool {
    match kind {
        HeightmapKind::WorldSurface => !b.is_air(),
        HeightmapKind::OceanFloor => blocks_motion(b),
        HeightmapKind::MotionBlocking => blocks_motion(b) || b.is_fluid(),
        HeightmapKind::MotionBlockingNoLeaves => {
            (blocks_motion(b) || b.is_fluid()) && !is_leaves(b)
        }
    }
}

/// Highest Y whose block is opaque for `kind`, or None if the column is empty.
pub(crate) fn heightmap_top(
    region: &RegionBuf,
    x: i32,
    z: i32,
    kind: HeightmapKind,
) -> Option<i32> {
    // WORLD_SURFACE_WG semantics: primed post-noise, RE-PRIMED when the
    // chunk completes SURFACE (ChunkStatus.SURFACE heightmapsAfter =
    // WORLDGEN_HEIGHTMAPS), then FROZEN — ProtoChunk.setBlockState only
    // tracks FINAL_HEIGHTMAPS during CARVERS/FEATURES. So placement
    // modifiers reading WORLD_SURFACE_WG (patch_grass_jungle ferns etc.)
    // see the post-surface/pre-carver snapshot, NOT live blocks. The
    // RegionBuf per-chunk heightmaps are exactly that snapshot (written by
    // generate_noise_and_surface after apply_surface_rules).
    if kind == HeightmapKind::WorldSurface {
        if let Some(y) = region.frozen_heightmap(x, z) {
            if y > WORLD_BOTTOM as i16 {
                return Some(y as i32);
            }
            // WORLD_BOTTOM sentinel = never primed (e.g. bare test grid):
            // fall through to the live scan.
        }
    }
    // s73 REVERTED: routing OceanFloor through the frozen post-carver snapshot
    // REGRESSED the (-12,-6) window 9,459 → 21,491 mismatches. The static
    // reading of ProtoChunk/ChunkStatus says vanilla primes FINAL_HEIGHTMAPS
    // at CARVERS→FEATURES and never updates them during decoration — but the
    // frozen snapshot measured WORSE than the live scan, and the live scan
    // matches vanilla's tree ys at all three witness columns (s70). The real
    // WorldGenRegion path may re-prime (WorldGenRegion ctor clones/creates
    // heightmaps per status) or the update happens through a path this
    // analysis missed. Live scan restored; the freeze infra remains for the
    // next hypothesis.
    for y in (WORLD_BOTTOM..WORLD_TOP).rev() {
        if heightmap_opaque(region.get(x, y, z), kind) {
            return Some(y);
        }
    }
    None
}

/// `SurfaceWaterDepthFilter`: `WORLD_SURFACE` first-available minus
/// `OCEAN_FLOOR` first-available, both scanned live from the region buffer
/// top-down (vanilla Heightmap.Types, Heightmap.java):
///   - `WORLD_SURFACE(_WG)` predicate = `!state.isAir()` (air/cave_air),
///   - `OCEAN_FLOOR(_WG)` predicate = `BlockState::blocksMotion()` ONLY —
///     no fluid term, and plants (short_grass, leaf_litter, moss carpets,
///     vines, hanging moss) do NOT count as floor.
/// A leaf_litter carpet therefore raises WORLD_SURFACE one above OCEAN_FLOOR
/// even on dry land, making depth = 1 > max(0) and rejecting tree attempts
/// exactly like vanilla (pale_garden/dark_forest step-9 desync root cause).
pub(crate) fn column_water_depth(region: &RegionBuf, x: i32, z: i32) -> i32 {
    let mut surface = None;
    let mut floor = None;
    for y in (WORLD_BOTTOM..WORLD_TOP).rev() {
        let b = region.get(x, y, z);
        if !b.is_air() && surface.is_none() {
            surface = Some(y + 1);
        }
        if blocks_motion(b) {
            floor = Some(y + 1);
            break;
        }
    }
    match (surface, floor) {
        (Some(s), Some(f)) => (s - f).max(0),
        _ => 0,
    }
}

pub(crate) fn biome_name_at(state: &WorldgenState, x: i32, y: i32, z: i32) -> String {
    biome_id_to_name(biome_id_at_block(state, x, y, z)).to_string()
}

pub fn biome_id_to_name(id: u8) -> &'static str {
    // Subset — extend as biome_source ids expand
    match id {
        x if x == biome_id::DEEP_DARK => "deep_dark",
        x if x == biome_id::DARK_FOREST => "dark_forest",
        x if x == biome_id::PALE_GARDEN => "pale_garden",
        x if x == biome_id::PLAINS => "plains",
        x if x == biome_id::FOREST => "forest",
        x if x == biome_id::BIRCH_FOREST => "birch_forest",
        x if x == biome_id::TAIGA => "taiga",
        x if x == biome_id::SWAMP => "swamp",
        x if x == biome_id::DESERT => "desert",
        x if x == biome_id::JUNGLE => "jungle",
        x if x == biome_id::SAVANNA => "savanna",
        x if x == biome_id::MEADOW => "meadow",
        x if x == biome_id::OLD_GROWTH_PINE_FOREST => "old_growth_pine_taiga",
        x if x == biome_id::OLD_GROWTH_BIRCH_FOREST => "old_growth_birch_forest",
        x if x == biome_id::DRIPSTONE_CAVES => "dripstone_caves",
        x if x == biome_id::LUSH_CAVES => "lush_caves",
        x if x == biome_id::SULFUR_CAVES => "sulfur_caves",
        x if x == biome_id::MANGROVE_SWAMP => "mangrove_swamp",
        x if x == biome_id::CHERRY_GROVE => "cherry_grove",
        x if x == biome_id::BADLANDS => "badlands",
        x if x == biome_id::ERODED_BADLANDS => "eroded_badlands",
        x if x == biome_id::WOODED_BADLANDS => "wooded_badlands",
        x if x == biome_id::GROVE => "grove",
        x if x == biome_id::JAGGED_PEAKS => "jagged_peaks",
        x if x == biome_id::STONY_PEAKS => "stony_peaks",
        x if x == biome_id::FROZEN_PEAKS => "frozen_peaks",
        x if x == biome_id::SNOWY_SLOPES => "snowy_slopes",
        x if x == biome_id::WINDSWEPT_HILLS => "windswept_hills",
        // Complete the vanilla biome set: every collision here collapses
        // distinct biomes into "plains" downstream (export_predecorate
        // remap), which changes the region biome SET, reshuffles
        // FeatureSorter indices and re-seeds EVERY per-feature RNG stream.
        x if x == biome_id::MUSHROOM_FIELDS => "mushroom_fields",
        x if x == biome_id::OCEAN => "ocean",
        x if x == biome_id::DEEP_OCEAN => "deep_ocean",
        x if x == biome_id::FROZEN_OCEAN => "frozen_ocean",
        x if x == biome_id::RIVER => "river",
        x if x == biome_id::FROZEN_RIVER => "frozen_river",
        x if x == biome_id::BEACH => "beach",
        x if x == biome_id::STONY_SHORE => "stony_shore",
        x if x == biome_id::SNOWY_PLAINS => "snowy_plains",
        x if x == biome_id::ICE_SPIKES => "ice_spikes",
        x if x == biome_id::DEEP_FROZEN_OCEAN => "deep_frozen_ocean",
        x if x == biome_id::DEEP_COLD_OCEAN => "deep_cold_ocean",
        x if x == biome_id::COLD_OCEAN => "cold_ocean",
        x if x == biome_id::DEEP_LUKEWARM_OCEAN => "deep_lukewarm_ocean",
        x if x == biome_id::LUKEWARM_OCEAN => "lukewarm_ocean",
        x if x == biome_id::WARM_OCEAN => "warm_ocean",
        x if x == biome_id::SNOWY_BEACH => "snowy_beach",
        x if x == biome_id::WINDSWEPT_FOREST => "windswept_forest",
        x if x == biome_id::WINDSWEPT_GRAVELLY_HILLS => "windswept_gravelly_hills",
        x if x == biome_id::WINDSWEPT_SAVANNA => "windswept_savanna",
        x if x == biome_id::SAVANNA_PLATEAU => "savanna_plateau",
        x if x == biome_id::SPARSE_JUNGLE => "sparse_jungle",
        x if x == biome_id::BAMBOO_JUNGLE => "bamboo_jungle",
        x if x == biome_id::SUNFLOWER_PLAINS => "sunflower_plains",
        x if x == biome_id::FLOWER_FOREST => "flower_forest",
        x if x == biome_id::OLD_GROWTH_SPRUCE_TAIGA => "old_growth_spruce_taiga",
        x if x == biome_id::SNOWY_TAIGA => "snowy_taiga",
        _ => "plains",
    }
}

pub(super) fn strip(s: &str) -> &str {
    s.strip_prefix("minecraft:").unwrap_or(s)
}



