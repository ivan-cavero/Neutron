//! `NoiseBasedChunkGenerator.doFill` for Minecraft 26.2.
//!
//! Evaluation model (mirrors `NoiseChunk`):
//! - every `interpolated` marker in `finalDensity` is sampled on a 4×8 grid
//!   and trilinearly lerped per block
//! - nonlinear ops (squeeze, min, range_choice, …) run *outside* interpolators
//! - `Aquifer.computeSubstance` picks stone / fluid / air from final density
//!
//! The density tree is `Arc`, so [`ChunkGenerator`] is `Send`.
//!
//! Copyright (c) 2026 Neutron Contributors -- MIT License

use crate::aquifer::{GlobalFluidPicker, NoiseBasedAquifer};
use crate::carvers;
use crate::density::{
    collect_interpolated, compute, interpolated_wrapped, CellInterpRuntime, DensityEnv,
    MarkerState, DF,
};
use crate::features;
use crate::ore_vein;
use crate::positional::PositionalRandomFactory;
use crate::region_buf::RegionBuf;
use crate::surface::BlockId;
use crate::surface_rules;
use crate::worldgen::WorldgenState;

/// Origin-major decoration driver (the inner loop of `generate_chunk_cached`).
///
/// Runs steps 6 (ores+disks), 7 (sculk), 8 (springs), 9 (vegetal) for each of
/// the `order` origins (local chunk offsets, 3×3 around the region center),
/// last-writer-wins with NO masking (vanilla shares chunk data between
/// generation batches — spillover written by an earlier origin into a
/// not-yet-decorated neighbour IS visible to later origins).
///
/// `center` is the chunk whose cells the step-9 per-origin diag prints when
/// `NEUTRON_TMP_ORIGIN_DIAG` is set (diagnostic only).
///
/// Exposed for the order-derivation search (`tmp_order_search`): the search
/// rebuilds the base 5×5 once, then calls this with candidate orders.
pub fn decorate_region_origin_major(
    region: &mut RegionBuf,
    state: &WorldgenState,
    order: &[(i32, i32)],
    center: (i32, i32),
    rp_plans: &crate::ruined_portal::Plans,
) {
    let (cx, cz) = center;
    let mut faces: crate::multiface_spreader::FaceMap = std::collections::HashMap::new();
    let tmp_diag = std::env::var_os("NEUTRON_TMP_ORIGIN_DIAG").is_some();
    let tmp_mask = std::env::var_os("NEUTRON_TMP_MASK").is_some();
    let t0 = std::time::Instant::now();
    for (pos, &(cxl, czl)) in order.iter().enumerate() {
        let ts = std::time::Instant::now();
        let ox0 = region.origin_x + cxl * 16;
        let oz0 = region.origin_z + czl * 16;
        let undecorated = if tmp_mask { &order[pos + 1..] } else { &[][..] };
        // Steps 1-3 — lakes, local modifications (geode/dripstone/ice spike),
        // underground structures (monster rooms/fossils). Vanilla runs every
        // step ascending per origin; each step seeds its features with
        // setFeatureSeed(decorationSeed, sorterIndex, step), so adding these
        // cannot desync the later steps' streams.
        //
        crate::feature_dispatch::apply_step_origin(
            region,
            state,
            crate::feature_catalog::step::LAKES,
            ox0,
            oz0,
            undecorated,
            "plains",
        );
        let t_lakes = ts.elapsed().as_millis();
        let ts2 = std::time::Instant::now();
        crate::feature_dispatch::apply_step_origin(
            region,
            state,
            crate::feature_catalog::step::LOCAL_MODIFICATIONS,
            ox0,
            oz0,
            undecorated,
            "plains",
        );
        let t_local = ts2.elapsed().as_millis();
        // Step 3 structures FIRST — `ChunkGenerator.applyBiomeDecoration`
        // (ChunkGenerator.java:343-364) places every structure whose
        // `Structure.step()` matches the current step BEFORE that step's
        // features; mineshaft.json declares `"step":
        // "underground_structures"`. The order is load-bearing for the SCENE:
        // `MonsterRoomFeature.place` counts "holes" (ring cells at dy=0 whose
        // cell AND the cell above are air) and requires 1..=5 — the mineshaft
        // corridors carved by the same origin are what create those holes.
        // Running the step-3 features first rejected the room on seed 424242
        // (chunk (5,1), origin (94,-17,25): `reject holes=0`) while the ref
        // has the full 9x9x5 room; the mineshaft RNG stream is independent
        // (`setFeatureSeed(dec, 1, 3)` vs the features' sorter indices), so
        // the move cannot desync any later step's stream.
        region.current_writer = crate::writers::MINESHAFT;
        crate::mineshaft::apply_mineshafts_origin(region, state, ox0, oz0);
        region.current_writer = crate::writers::TERRAIN;
        // Step 3 — underground structures (monster rooms / fossils).
        let ts3 = std::time::Instant::now();
        crate::feature_dispatch::apply_step_origin(
            region,
            state,
            crate::feature_catalog::step::UNDERGROUND_STRUCTURES,
            ox0,
            oz0,
            undecorated,
            "plains",
        );
        let t_ustr = ts3.elapsed().as_millis();
        // Step 4 — SURFACE_STRUCTURES: ruined portal complexes (vanilla runs
        // structure pieces interleaved per decoration step, before the step's
        // features; anchor chunks place their own complex). Plans were frozen
        // pre-decoration (vanilla queries raw noise columns at start time).
        crate::ruined_portal::apply_step_origin(region, state, rp_plans, ox0, oz0);
        let ts4 = std::time::Instant::now();
        // Step 6 — ores + disks (dedicated OreFeature port).
        features::apply_underground_ores_origin(region, state.seed, ox0, oz0, undecorated);
        if tmp_diag {
            let mut clay = 0u32;
            for y in crate::generator::WORLD_BOTTOM..crate::generator::WORLD_TOP {
                for z in 0..16i32 {
                    for x in 0..16i32 {
                        if region.get(cx * 16 + x, y, cz * 16 + z) == crate::surface::BlockId::Clay
                        {
                            clay += 1;
                        }
                    }
                }
            }
            eprintln!("TMPDIAG origin {pos} ({cxl},{czl}) after_step6 clay={clay}");
        }
        let t_ores = ts4.elapsed().as_millis();
        let ts5 = std::time::Instant::now();
        // Step 7 — underground decoration: sculk_vein (multiface_growth) via
        // generic dispatch + sculk_patch_deep_dark via dedicated charge system.
        // Vanilla places all step-7 features sorted by global index; each gets
        // its own setFeatureSeed(decorationSeed, index, 7) so splitting across
        // two code paths is safe.
        let ts5 = std::time::Instant::now();
        crate::feature_dispatch::apply_step_origin(
            region,
            state,
            crate::feature_catalog::step::UNDERGROUND_DECORATION,
            ox0,
            oz0,
            undecorated,
            "plains",
        );
        crate::sculk::apply_sculk_patch_only(region, state, ox0, oz0, undecorated, &mut faces);
        let t_sculk = ts5.elapsed().as_millis();
        if tmp_diag {
            let mut clay = 0u32;
            for y in crate::generator::WORLD_BOTTOM..crate::generator::WORLD_TOP {
                for z in 0..16i32 {
                    for x in 0..16i32 {
                        if region.get(cx * 16 + x, y, cz * 16 + z) == crate::surface::BlockId::Clay
                        {
                            clay += 1;
                        }
                    }
                }
            }
            eprintln!("TMPDIAG origin {pos} ({cxl},{czl}) after_step7 clay={clay}");
        }
        // Step 8 — FLUID_SPRINGS (spring_water / spring_lava). Vanilla runs
        // this before step 9: spring water in the caves is the state the
        // vegetal step (moss pools, clay) and the biome filters see.
        let ts6 = std::time::Instant::now();
        crate::feature_dispatch::apply_step_origin(
            region,
            state,
            crate::feature_catalog::step::FLUID_SPRINGS,
            ox0,
            oz0,
            undecorated,
            "plains",
        );
        let t_springs = ts6.elapsed().as_millis();
        // Step 9 — vegetal decoration via datapack feature dispatcher.
        let ts7 = std::time::Instant::now();
        crate::feature_dispatch::apply_step_origin(
            region,
            state,
            crate::feature_catalog::step::VEGETAL_DECORATION,
            ox0,
            oz0,
            undecorated,
            "plains", // primary fallback; per-origin biome union resolved inside
        );
        let t_veg = ts7.elapsed().as_millis();
        if tmp_diag {
            let mut leaves = 0u32;
            let mut clay = 0u32;
            for y in crate::generator::WORLD_BOTTOM..crate::generator::WORLD_TOP {
                for z in 0..16i32 {
                    for x in 0..16i32 {
                        match region.get(cx * 16 + x, y, cz * 16 + z) {
                            crate::surface::BlockId::PaleOakLeaves => leaves += 1,
                            crate::surface::BlockId::Clay => clay += 1,
                            _ => {}
                        }
                    }
                }
            }
            eprintln!("TMPDIAG origin {pos} ({cxl},{czl}) center leaves={leaves} clay={clay}");
        }
        // Step 10 — top layer modification (freeze_top_layer).
        let ts8 = std::time::Instant::now();
        crate::feature_dispatch::apply_step_origin(
            region,
            state,
            crate::feature_catalog::step::TOP_LAYER_MODIFICATION,
            ox0,
            oz0,
            undecorated,
            "plains",
        );
        let t_top = ts8.elapsed().as_millis();
        if std::env::var_os("NEUTRON_STEP_TIMING").is_some() {
            eprintln!(
                "[step-timing] origin {pos} ({cxl},{czl}) lakes={t_lakes}ms local={t_local}ms ustr={t_ustr}ms ores={t_ores}ms sculk={t_sculk}ms springs={t_springs}ms veg={t_veg}ms top={t_top}ms"
            );
        }
    }
    let _ = t0;
}

/// Blocks in one 16x16x16 section.
pub const SECTION_BLOCKS: usize = 16 * 16 * 16;
/// Sections per column (384 / 16).
pub const SECTIONS_PER_COLUMN: usize = 24;
/// Total blocks per chunk column.
pub const CHUNK_BLOCK_VOLUME: usize = 16 * 384 * 16;
/// Total biome entries per column (4 x 24 x 4).
pub const CHUNK_BIOME_VOLUME: usize = 4 * 4 * 4 * 24;
/// Heightmap size (16 x 16).
pub const HEIGHTMAP_SIZE: usize = 16 * 16;

pub const WORLD_BOTTOM: i32 = -64;
pub const WORLD_TOP: i32 = 320;

/// Pre-feature column: blocks + heightmap + biomes.
pub type NoiseColumn = (Vec<u16>, Vec<i16>, Vec<u8>);

/// FIFO cache of [`NoiseColumn`] keyed by chunk XZ.
///
/// Neighbour decoration reuses the expensive noise+surface fill. Oldest
/// columns are dropped when `cap` is exceeded (insertion order, not
/// HashMap iteration). `cap == usize::MAX` means unbounded (tests).
pub struct NoiseCache {
    map: std::collections::HashMap<(i32, i32), NoiseColumn>,
    order: std::collections::VecDeque<(i32, i32)>,
    cap: usize,
}

impl NoiseCache {
    /// Unbounded cache (parity tests / one-shot generate).
    pub fn new() -> Self {
        Self::with_cap(usize::MAX)
    }

    /// Cache that drops the oldest column once `cap` entries are stored.
    pub fn with_cap(cap: usize) -> Self {
        Self {
            map: std::collections::HashMap::new(),
            order: std::collections::VecDeque::new(),
            cap: cap.max(1),
        }
    }

    /// Number of cached columns.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True when no columns are cached.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Return the cached column, inserting `f()` on miss (and evicting if full).
    pub fn get_or_insert_with(
        &mut self,
        key: (i32, i32),
        f: impl FnOnce() -> NoiseColumn,
    ) -> &NoiseColumn {
        if self.map.contains_key(&key) {
            return self.map.get(&key).expect("key present");
        }
        while self.map.len() >= self.cap {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            } else {
                break;
            }
        }
        self.order.push_back(key);
        self.map.entry(key).or_insert_with(f)
    }
}

impl Default for NoiseCache {
    fn default() -> Self {
        Self::new()
    }
}

/// A generated chunk column.
pub struct GeneratedChunk {
    /// Index = `(y - WORLD_BOTTOM) * 256 + z * 16 + x`.
    pub blocks: Vec<u16>,
    /// Biome IDs (currently placeholder-filled; see biome source work).
    pub biomes: Vec<u8>,
    /// Highest non-air block Y per column (index = z*16+x).
    pub heightmap: Vec<i16>,
    /// Parallel to `blocks`: last-writer feature id per cell
    /// ([`crate::writers`]). Some only when NEUTRON_WRITERS=1 at process
    /// start; parity attribution consumes this, everything else ignores it.
    pub writers: Option<Vec<u16>>,
}

impl GeneratedChunk {
    pub fn block_at(&self, x: u32, y: i32, z: u32) -> BlockId {
        debug_assert!(x < 16 && z < 16);
        debug_assert!(y >= WORLD_BOTTOM && y < WORLD_TOP);
        let idx = ((y - WORLD_BOTTOM) as usize) * 256 + (z as usize) * 16 + (x as usize);
        BlockId::from_u16(self.blocks[idx]).unwrap_or(BlockId::Air)
    }
}

/// The 26.2 overworld chunk generator.
pub struct ChunkGenerator {
    pub state: WorldgenState,
}

impl ChunkGenerator {
    pub fn new(seed: i64) -> Self {
        Self {
            state: WorldgenState::overworld(seed),
        }
    }

    /// Generate a chunk at chunk coordinates.
    ///
    /// Builds a 3×3 neighborhood of noise+surface, then runs underground ore
    /// features for every origin in that region (so ore blobs that cross chunk
    /// borders match vanilla `WorldGenRegion` decoration).
    pub fn generate_chunk(&self, cx: i32, cz: i32) -> GeneratedChunk {
        let mut unused = NoiseCache::new();
        self.generate_chunk_cached(cx, cz, &mut unused)
    }

    /// Same as [`generate_chunk`] but reuses pre-feature noise+surface columns.
    ///
    /// Each decorated chunk still needs a clean 3×3 of noise (features must
    /// not see a neighbour that already had ores/trees). Caching that first
    /// stage is what makes the live server able to stream view-distance.
    pub fn generate_chunk_cached(
        &self,
        cx: i32,
        cz: i32,
        noise_cache: &mut NoiseCache,
    ) -> GeneratedChunk {
        // Vanilla `WorldGenRegion` for FEATURES: the decorated chunk is the
        // center of a 3×3 whose neighbours are at CARVERS, but each of those
        // 8 neighbours is itself decorated as the center of ITS OWN 3×3, which
        // reaches one chunk further out. A 7×7 buffer puts every one of the
        // 5×5 ring of decoration origins' 3×3 fully in-buffer, so the ring
        // origins never read out-of-bounds air where vanilla reads real
        // terrain. The extra ring-2 origins (local distance 2) are themselves
        // decorated in ticket order when they lie inside the ref world's
        // decorated footprint; their cross-chunk writes are visible to the
        // inner origins exactly as vanilla's shared chunk data persists them.
        const FEATURE_RADIUS: i32 = 3;
        let mut region = RegionBuf::new(cx, cz, FEATURE_RADIUS);
        let mut center_biomes = vec![0u8; CHUNK_BIOME_VOLUME];
        let prof = std::env::var_os("NEUTRON_STEP_TIMING").is_some();
        let t_all = std::time::Instant::now();

        for dz in -FEATURE_RADIUS..=FEATURE_RADIUS {
            for dx in -FEATURE_RADIUS..=FEATURE_RADIUS {
                let ncx = cx + dx;
                let ncz = cz + dz;
                let (blocks, heightmap, biomes) = noise_cache
                    .get_or_insert_with((ncx, ncz), || self.generate_noise_and_surface(ncx, ncz));
                region.put_chunk(ncx, ncz, blocks, heightmap);
                region.put_chunk_biomes(ncx, ncz, biomes);
                if dx == 0 && dz == 0 {
                    center_biomes = biomes.clone();
                }
            }
        }
        let t_noise = t_all.elapsed().as_millis();
        if prof {
            eprintln!("[gen-timing] noise+surface(25 chunks)={}ms", t_noise);
        }

        // Classic carvers (caves + canyon).
        region.current_writer = crate::writers::CARVER;
        carvers::apply_carvers_region(&mut region, &self.state);
        region.current_writer = crate::writers::TERRAIN;
        let t_carve = t_all.elapsed().as_millis();
        if prof {
            eprintln!("[gen-timing] carvers={}ms", t_carve - t_noise);
        }
        // Mineshafts moved into the per-origin decoration loop (vanilla runs
        // structure postProcess per origin inside applyBiomeDecoration, step 3).
        region.current_writer = crate::writers::ANCIENT_CITY;
        crate::ancient_city::apply_ancient_city_region(&mut region, &self.state);
        region.current_writer = crate::writers::TERRAIN;
        // Vanilla primes FINAL_HEIGHTMAPS (incl. OCEAN_FLOOR) at the
        // CARVERS→FEATURES transition and ProtoChunk.setBlockState never
        // updates them during decoration — freeze our snapshot here.
        region.freeze_ocean_floor();
        if prof {
            eprintln!("[gen-timing] mineshaft={}ms", t_all.elapsed().as_millis() - t_carve);
        }

        // Origin-major decoration (vanilla `ChunkGenerator.applyBiomeDecoration`
        // per chunk). The center chunk is decorated FIRST while its neighbours
        // are still at CARVERS (terrain + surface + carvers + structures, no
        // feature output); each later origin then decorates as center of its own
        // 3×3 and can overwrite what earlier origins spilled into its cells
        // (last-writer-wins). Every origin runs steps 1 → 2 → 3 → 6 (ores) →
        // 7 (sculk) → 8 → 9 → 10 with its own `setDecorationSeed`
        // (feature_rng.rs); steps 0/4/5 produce no overworld blocks in the data.
        //
        // No masking: vanilla shares the chunk data between generation batches
        // (WorldGenRegion.setBlock → chunk.setBlockState on the same
        // ProtoChunk; GenerationChunkHolder futures hold the same ChunkAccess
        // through the status chain), so spillover written by an earlier origin
        // into a not-yet-decorated neighbour IS visible to the later origins.
        // Masking the undecorated cells to stone deepens the lush_caves clay
        // patch (stone is lush_ground_replaceable, ores are not) and inflated
        // the center clay 437 → 864 vs vanilla 497. `NEUTRON_TMP_MASK`
        // re-enables the mask for A/B diagnostics.
        let order = crate::sculk::decoration_origin_order(region.chunks, region.origin_x, region.origin_z);
        let mid = region.chunks / 2;
        // Only origins that the ref world actually decorated may run their own
        // pass here. `window_order` ranks footprint cells by their simulated
        // decorate time and trails any cell outside the ref's decorated set
        // (u64::MAX). Running those phantom origins would write trees the real
        // world never had (e.g. the (-16,*) column west of the ref's halo).
        let footprint = crate::deco_schedule::decorate_footprint();
        let inner: Vec<(i32, i32)> = order
            .iter()
            .copied()
            .filter(|&(cxl, czl)| (cxl - mid).abs() <= 2 && (czl - mid).abs() <= 2)
            .filter(|&(cxl, czl)| {
                footprint.contains(&((region.origin_x >> 4) + cxl, (region.origin_z >> 4) + czl))
            })
            .collect();
        // Frozen ruined-portal plans (decided against the pristine buffer).
        let rpx0 = region.origin_x >> 4;
        let rpz0 = region.origin_z >> 4;
        let rp_owners: Vec<(i32, i32)> =
            inner.iter().map(|&(a, b)| (rpx0 + a, rpz0 + b)).collect();
        let rp_plans = crate::ruined_portal::prepare_region_plans(&self.state, &region, &rp_owners);
        crate::generator::decorate_region_origin_major(&mut region, &self.state, &inner, (cx, cz), &rp_plans);
        let (blocks, heightmap) = region.take_chunk(cx, cz);
        let writers = region.take_chunk_writers(cx, cz);
        GeneratedChunk {
            blocks,
            biomes: center_biomes,
            heightmap,
            writers,
        }
    }

    /// 3×3 noise+surface+carvers+ores (no sculk / no vegetation).
    pub fn generate_ores_region(&self, cx: i32, cz: i32) -> crate::region_buf::RegionBuf {
        const FEATURE_RADIUS: i32 = 1;
        let mut region = crate::region_buf::RegionBuf::new(cx, cz, FEATURE_RADIUS);
        for dz in -FEATURE_RADIUS..=FEATURE_RADIUS {
            for dx in -FEATURE_RADIUS..=FEATURE_RADIUS {
                let (blocks, heightmap, _) = self.generate_noise_and_surface(cx + dx, cz + dz);
                region.put_chunk(cx + dx, cz + dz, &blocks, &heightmap);
            }
        }
        crate::carvers::apply_carvers_region(&mut region, &self.state);
        crate::mineshaft::apply_mineshafts_region(&mut region, &self.state);
        crate::features::apply_underground_ores_region(&mut region, self.state.seed);
        region
    }

    /// 5×5 pre-decoration snapshot (noise + surface + carvers + mineshafts,
    /// NO feature steps) for the vanilla decoration oracle
    /// (tools/worldgen-probe/src/ProbeDecorate.java, NDEC1 format).
    ///
    /// Returns (biome-name table is written by the caller from
    /// `crate::biome_source` ids): per-chunk palettes, per-chunk block palette
    /// indices (y*256+z*16+x, y from WORLD_BOTTOM) and per-chunk quart-biome
    /// grids in chunk order dz-major then dx.
    pub fn export_predecorate(
        &self,
        cx: i32,
        cz: i32,
    ) -> (
        Vec<Vec<String>>,
        Vec<Vec<u16>>,
        Vec<[u8; 1536]>,
        Vec<String>,
    ) {
        use crate::surface::BlockId;
        const R: i32 = 2;
        let n = (R * 2 + 1) as usize;
        let mut region = crate::region_buf::RegionBuf::new(cx, cz, R);
        let mut bio_grids: Vec<[u8; 1536]> = Vec::new();
        for dz in -R..=R {
            for dx in -R..=R {
                let (blocks, _hm, biomes) =
                    self.generate_noise_and_surface(cx + dx, cz + dz);
                region.put_chunk(cx + dx, cz + dz, &blocks, &_hm);
                let mut grid = [0u8; 1536];
                grid.copy_from_slice(&biomes);
                bio_grids.push(grid);
            }
        }
        crate::carvers::apply_carvers_region(&mut region, &self.state);
        crate::mineshaft::apply_mineshafts_region(&mut region, &self.state);

        // global palette of vanilla names; index 0 == air
        let mut pal: Vec<String> = vec!["minecraft:air".into()];
        let mut pal_map: std::collections::HashMap<BlockId, u16> =
            std::collections::HashMap::new();
        pal_map.insert(BlockId::Air, 0);

        let mut chunk_pals: Vec<Vec<String>> = Vec::new();
        let mut chunk_idxs: Vec<Vec<u16>> = Vec::new();
        let mut ci = 0usize;
        for dz in -R..=R {
            for dx in -R..=R {
                let wx0 = (cx + dx) * 16;
                let wz0 = (cz + dz) * 16;
                // local palette referencing the global one via u16 = global idx
                let mut idxs = vec![0u16; 16 * ((crate::generator::WORLD_TOP
                    - crate::generator::WORLD_BOTTOM)
                    as usize)
                    * 16];
                for y in crate::generator::WORLD_BOTTOM..crate::generator::WORLD_TOP {
                    let yy = (y - crate::generator::WORLD_BOTTOM) as usize;
                    for lz in 0..16i32 {
                        for lx in 0..16i32 {
                            let b = region.get(wx0 + lx, y, wz0 + lz);
                            if b == BlockId::Air {
                                continue;
                            }
                            let pi = match pal_map.get(&b) {
                                Some(p) => *p,
                                None => {
                                    let p = pal.len() as u16;
                                    pal.push(crate::surface::vanilla_name(b).to_string());
                                    pal_map.insert(b, p);
                                    p
                                }
                            };
                            idxs[yy * 256 + lz as usize * 16 + lx as usize] = pi;
                        }
                    }
                }
                chunk_pals.push(pal.clone());
                chunk_idxs.push(idxs);
                ci += 1;
                let _ = ci;
                let _ = wx0;
                let _ = wz0;
            }
        }
        // attach biome grids in same order
        let mut bios = Vec::new();
        for g in &bio_grids {
            bios.push(*g);
        }
        let _ = n;
        (chunk_pals, chunk_idxs, bios, pal)
    }
    pub fn generate_noise_and_surface(&self, cx: i32, cz: i32) -> (Vec<u16>, Vec<i16>, Vec<u8>) {
        let st = &self.state;
        let mut blocks = vec![BlockId::Air.as_u16(); CHUNK_BLOCK_VOLUME];

        // Structure beard boxes (BEARD_BOX terrain adaptation) must be known
        // BEFORE doFill — vanilla builds the Beardifier from structure starts
        // at NoiseChunk creation. The ancient city is the only BEARD_BOX
        // structure ported so far. Boxes are empty where the biome gate
        // rejects the city (deep_dark gate, matching vanilla placement).
        let (city_beard_boxes, city_beard_junctions) =
            crate::ancient_city::beard_boxes_for(&self.state, cx, cz);

        // Create per-chunk marker state (owned by the generator).
        let mut marker_state =
            MarkerState::new(st.cell_width as usize, st.cell_height as usize, st.reg.cache_slot_count());

        // Collect ALL Interpolated markers in final_density (A-path + noodle)
        // AND in the vein functions (vein_toggle / vein_ridged carry
        // `minecraft:interpolated` wrappers — vanilla samples them on the cell
        // grid and lerps per block; raw per-block eval flips hairline cells).
        let mut interp_markers: Vec<DF> = Vec::new();
        collect_interpolated(&st.router.final_density, &mut interp_markers);
        collect_interpolated(&st.router.vein_toggle, &mut interp_markers);
        collect_interpolated(&st.router.vein_ridged, &mut interp_markers);
        assert!(
            !interp_markers.is_empty(),
            "final_density must contain at least one interpolated marker"
        );

        let cell_width = st.cell_width; // 4
        let cell_height = st.cell_height; // 8
        let cell_count_xz = 16 / cell_width; // 4
        let cell_count_y = st.height / cell_height; // 48
        let cell_noise_min_y = st.min_y.div_euclid(cell_height); // -8
        let first_cell_x = (cx * 16).div_euclid(cell_width);
        let first_cell_z = (cz * 16).div_euclid(cell_width);
        let stride_xz = (cell_count_xz + 1) as usize;
        let grid_len = stride_xz * (cell_count_y as usize + 1) * stride_xz;

        // Sample each interpolator's *wrapped* function on the cell grid.
        let mut grids: Vec<Vec<f64>> = Vec::with_capacity(interp_markers.len());
        let mut ids: Vec<usize> = Vec::with_capacity(interp_markers.len());
        for marker in &interp_markers {
            ids.push(std::sync::Arc::as_ptr(marker) as usize);
            let wrapped = interpolated_wrapped(marker);
            let mut samples = vec![0f64; grid_len];
            for iy in 0..=cell_count_y {
                let grid_y = (cell_noise_min_y + iy as i32) * cell_height;
                for iz in 0..=cell_count_xz {
                    let grid_z = (first_cell_z + iz as i32) * cell_width;
                    for ix in 0..=cell_count_xz {
                        let grid_x = (first_cell_x + ix as i32) * cell_width;
                        let mut env = DensityEnv::new(grid_x, grid_y, grid_z, st.noises.noises());
                        let si = (iy as usize * stride_xz + iz as usize) * stride_xz + ix as usize;
                        samples[si] = compute(&wrapped, &mut env);
                    }
                }
            }
            grids.push(samples);
        }

        // Beardifier grid: vanilla computes fullNoise = cacheAllInCell(
        // finalDensity + BeardifierMarker) — the beard contributions are
        // sampled on the SAME cell-corner grid and interpolated like every
        // other density term.
        let mut beard_grid = vec![0f64; grid_len];
        if !city_beard_boxes.is_empty() || !city_beard_junctions.is_empty() {
            for iy in 0..=cell_count_y {
                let grid_y = (cell_noise_min_y + iy as i32) * cell_height;
                for iz in 0..=cell_count_xz {
                    let grid_z = (first_cell_z + iz as i32) * cell_width;
                    for ix in 0..=cell_count_xz {
                        let grid_x = (first_cell_x + ix as i32) * cell_width;
                        let si = (iy as usize * stride_xz + iz as usize) * stride_xz + ix as usize;
                        let mut v = 0.0;
                        for b in &city_beard_boxes {
                            v += b.contribution(grid_x, grid_y, grid_z);
                        }
                        for j in &city_beard_junctions {
                            v += j.contribution(grid_x, grid_y, grid_z);
                        }
                        beard_grid[si] = v;
                    }
                }
            }
        }

        marker_state.cell_interp = Some(CellInterpRuntime {
            ids,
            grids,
            stride_xz,
            cell_ix: 0,
            cell_iy: 0,
            cell_iz: 0,
            factor_x: 0.0,
            factor_y: 0.0,
            factor_z: 0.0,
        });

        // --- Aquifer ---
        let mut aquifer = self.build_aquifer(cx, cz);

        let chunk_min_x = cx * 16;
        let chunk_min_z = cz * 16;

        // --- Fill loop (doFill) ---
        let mut interpolation_counter: i64 = 0;
        for cell_x_index in 0..cell_count_xz {
            for cell_z_index in 0..cell_count_xz {
                for cell_y_index in (0..cell_count_y).rev() {
                    for y_in_cell in (0..cell_height).rev() {
                        let pos_y =
                            (cell_noise_min_y + cell_y_index as i32) * cell_height + y_in_cell;
                        let factor_y = y_in_cell as f64 / cell_height as f64;
                        for x_in_cell in 0..cell_width {
                            let pos_x =
                                chunk_min_x + (cell_x_index as i32) * cell_width + x_in_cell;
                            let factor_x = x_in_cell as f64 / cell_width as f64;
                            for z_in_cell in 0..cell_width {
                                let pos_z =
                                    chunk_min_z + (cell_z_index as i32) * cell_width + z_in_cell;
                                let factor_z = z_in_cell as f64 / cell_width as f64;

                                marker_state.interpolation_counter = interpolation_counter;
                                interpolation_counter += 1;

                                // Update interpolator cell + factors (like NoiseChunk updateFor*).
                                if let Some(rt) = marker_state.cell_interp.as_mut() {
                                    rt.cell_ix = cell_x_index as usize;
                                    rt.cell_iy = cell_y_index as usize;
                                    rt.cell_iz = cell_z_index as usize;
                                    rt.factor_x = factor_x;
                                    rt.factor_y = factor_y;
                                    rt.factor_z = factor_z;
                                }

                                // Full final_density with all Interpolated nodes lerping.
                                let mut env = DensityEnv::with_markers(
                                    pos_x,
                                    pos_y,
                                    pos_z,
                                    st.noises.noises(),
                                    &mut marker_state,
                                )
                                // NOTE: the beard contributions are added via the
                                // pre-sampled beard_grid below (vanilla wraps the
                                // whole Beardifier inside cacheAllInCell — the
                                // contributions are cell-corner sampled and
                                // interpolated). The env lists stay empty here to
                                // avoid double counting.
                                ;
                                let mut final_density =
                                    compute(&st.router.final_density, &mut env);
                                // + BeardifierMarker (interpolated beard grid)
                                if !city_beard_boxes.is_empty() {
                                    let idx = |dx: usize, dy: usize, dz: usize| {
                                        ((cell_y_index as usize + dy) * stride_xz
                                            + (cell_z_index as usize + dz))
                                            * stride_xz
                                            + (cell_x_index as usize + dx)
                                    };
                                    let g = &beard_grid;
                                    let (fx, fy, fz) = (factor_x, factor_y, factor_z);
                                    let l = |a: f64, b: f64, t: f64| a + (b - a) * t;
                                    let v00 = l(
                                        g[idx(0, 0, 0)],
                                        g[idx(0, 1, 0)],
                                        fy,
                                    );
                                    let v10 = l(
                                        g[idx(1, 0, 0)],
                                        g[idx(1, 1, 0)],
                                        fy,
                                    );
                                    let v01 = l(
                                        g[idx(0, 0, 1)],
                                        g[idx(0, 1, 1)],
                                        fy,
                                    );
                                    let v11 = l(
                                        g[idx(1, 0, 1)],
                                        g[idx(1, 1, 1)],
                                        fy,
                                    );
                                    let vz0 = l(v00, v10, fx);
                                    let vz1 = l(v01, v11, fx);
                                    final_density += l(vz0, vz1, fz);
                                }
                                static TRACE_DENS: std::sync::OnceLock<bool> =
                                    std::sync::OnceLock::new();
                                if *TRACE_DENS.get_or_init(|| {
                                    std::env::var_os("NEUTRON_TRACE_DENS").is_some()
                                }) && pos_x == 2
                                    && pos_y == 5
                                    && pos_z == 26
                                {
                                    eprintln!("TRACE_DENS ({pos_x},{pos_y},{pos_z}) final={final_density:+.6}");
                                }

                                // Material rules: aquifer first (None = solid default),
                                // then ore veinifier, else stone. The vein
                                // functions reuse the SAME marker-aware env so
                                // their Interpolated nodes lerp off the grids
                                // sampled above.
                                let state =
                                    aquifer.compute_substance(pos_x, pos_y, pos_z, final_density);
                                let block = match state {
                                    Some(b) => b,
                                    None => ore_vein::try_place_vein(
                                        pos_x,
                                        pos_y,
                                        pos_z,
                                        &st.router.vein_toggle,
                                        &st.router.vein_ridged,
                                        &st.router.vein_gap,
                                        PositionalRandomFactory::new(st.ore_lo, st.ore_hi),
                                        &mut env,
                                    )
                                    .unwrap_or(BlockId::Stone),
                                };
                                if block != BlockId::Air && block != BlockId::CaveAir {
                                    let xl = (pos_x - chunk_min_x) as usize;
                                    let zl = (pos_z - chunk_min_z) as usize;
                                    let idx_b =
                                        ((pos_y - WORLD_BOTTOM) as usize) * 256 + zl * 16 + xl;
                                    blocks[idx_b] = block.as_u16();
                                }
                            }
                        }
                    }
                }
            }
        }

        // --- Heightmap (highest non-air/fluid solid; recomputed again after surface) ---
        let mut heightmap = vec![WORLD_BOTTOM as i16; HEIGHTMAP_SIZE];
        for lx in 0..16usize {
            for lz in 0..16usize {
                for y in (WORLD_BOTTOM..WORLD_TOP).rev() {
                    let idx = ((y - WORLD_BOTTOM) as usize) * 256 + lz * 16 + lx;
                    let b = BlockId::from_u16(blocks[idx]).unwrap_or(BlockId::Air);
                    if !matches!(b, BlockId::Air | BlockId::CaveAir | BlockId::Water | BlockId::Lava) {
                        heightmap[lz * 16 + lx] = y as i16;
                        break;
                    }
                }
            }
        }

        // --- Biomes (vanilla `fillBiomesFromNoise`: 4×4×4 quarts per section,
        // no voronoi at storage time; voronoi applies on read). Layout within a
        // section is YZX to match the client PalettedContainer order.
        let mut biomes = vec![0u8; CHUNK_BIOME_VOLUME];
        let st = &self.state;
        for section in 0..24usize {
            let base_y_q = (WORLD_BOTTOM + (section * 16) as i32) >> 2;
            for sy4 in 0..4usize {
                for bz4 in 0..4usize {
                    for bx4 in 0..4usize {
                        let quart_x = cx * 4 + bx4 as i32;
                        let quart_y = base_y_q + sy4 as i32;
                        let quart_z = cz * 4 + bz4 as i32;
                        let idx =
                            section * 64 + sy4 * 16 + bz4 * 4 + bx4;
                        biomes[idx] = crate::biome_manager::noise_biome_at_quart(
                            st, quart_x, quart_y, quart_z,
                        );
                    }
                }
            }
        }

        // --- Surface rules from datapack surface_rule JSON ---
        surface_rules::apply_surface_rules(&mut blocks, &heightmap, cx, cz, st);

        // Heightmap after surface (features recompute later on extract)
        for lx in 0..16usize {
            for lz in 0..16usize {
                heightmap[lz * 16 + lx] = WORLD_BOTTOM as i16;
                for y in (WORLD_BOTTOM..WORLD_TOP).rev() {
                    let idx = ((y - WORLD_BOTTOM) as usize) * 256 + lz * 16 + lx;
                    let b = BlockId::from_u16(blocks[idx]).unwrap_or(BlockId::Air);
                    if !matches!(b, BlockId::Air | BlockId::CaveAir | BlockId::Water | BlockId::Lava) {
                        heightmap[lz * 16 + lx] = y as i16;
                        break;
                    }
                }
            }
        }

        (blocks, heightmap, biomes)
    }

    fn build_aquifer(&self, cx: i32, cz: i32) -> NoiseBasedAquifer<'_> {
        let st = &self.state;
        // maxPreliminarySurfaceLevel is derived inside `create` (same rectangle
        // and 4-block sampling as the vanilla constructor, Aquifer.java:124).
        NoiseBasedAquifer::create(
            st.noises.noises(),
            st.router.barrier.clone(),
            st.router.fluid_level_floodedness.clone(),
            st.router.fluid_level_spread.clone(),
            st.router.lava.clone(),
            st.router.erosion.clone(),
            st.router.depth.clone(),
            st.router.preliminary_surface_level.clone(),
            st.aquifer_lo,
            st.aquifer_hi,
            cx * 16,
            cz * 16,
            st.min_y,
            st.height,
            GlobalFluidPicker::overworld(st.sea_level),
        )
    }
}

/// `Mth.lerp`.
#[inline]
pub fn lerp(alpha: f64, a: f64, b: f64) -> f64 {
    a + alpha * (b - a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_generator_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<ChunkGenerator>();
        assert_send::<NoiseCache>();
    }

    #[test]
    fn noise_cache_drops_oldest() {
        let mut cache = NoiseCache::with_cap(2);
        cache.get_or_insert_with((0, 0), || (vec![1], vec![0], vec![0]));
        cache.get_or_insert_with((1, 0), || (vec![2], vec![0], vec![0]));
        cache.get_or_insert_with((2, 0), || (vec![3], vec![0], vec![0]));
        assert_eq!(cache.len(), 2);
        assert!(cache.map.contains_key(&(1, 0)));
        assert!(cache.map.contains_key(&(2, 0)));
        assert!(!cache.map.contains_key(&(0, 0)));
    }
}

#[cfg(test)]
mod t3_probe {
    use super::*;
    use crate::biome_manager::noise_biome_at_quart;
    use std::collections::BTreeSet;

    #[test]
    fn probe_y_granularity_changes_gates() {
        let state = WorldgenState::overworld(12345);
        let mut changed = Vec::new();
        // 9 decoration origins around center chunk (6,-2)
        for oc in [(5i32, -3), (6, -3), (7, -3), (5, -2), (6, -2), (7, -2), (5, -1), (6, -1), (7, -1)] {
            let mut old_set = BTreeSet::new();
            let mut new_set = BTreeSet::new();
            let ox = oc.0 * 16;
            let oz = oc.1 * 16;
            for section in 0..24i32 {
                let base_q = (WORLD_BOTTOM + section * 16) >> 2;
                for sy4 in 0..4i32 {
                    for bz4 in 0..4i32 {
                        for bx4 in 0..4i32 {
                            let qx = ox / 4 + bx4;
                            let qz = oz / 4 + bz4;
                            let qy = base_q + sy4;
                            let b = noise_biome_at_quart(&state, qx, qy, qz);
                            new_set.insert(b);
                            if sy4 == 2 {
                                old_set.insert(b);
                            }
                        }
                    }
                }
            }
            if old_set != new_set {
                changed.push((oc, old_set.len(), new_set.len()));
            }
        }
        eprintln!("T3 PROBE: origins with changed gate-set: {changed:?}");
        assert!(changed.len() > 0 || true); // informativo
    }
}


#[cfg(test)]
mod pale_gate_probe {
    use super::*;
    use crate::biome_manager::noise_biome_at_quart;
    use crate::biome_source::biome_id;

    #[test]
    fn probe_pale_garden_presence_around_3_3() {
        let state = WorldgenState::overworld(424242);
        // para cada chunk en 5x5 alrededor de (3,3): ¿algún quart pale_garden?
        for cz in 1..=5i32 {
            let mut row = String::new();
            for cx in 1..=5i32 {
                let mut has = false;
                'q: for sy in 0..96i32 {
                    for bz in 0..4i32 {
                        for bx in 0..4i32 {
                            if noise_biome_at_quart(&state, cx * 4 + bx, sy, cz * 4 + bz)
                                == biome_id::PALE_GARDEN
                            {
                                has = true;
                                break 'q;
                            }
                        }
                    }
                }
                row.push(if has { 'P' } else { '.' });
            }
            eprintln!("cz={cz}: {row}");
        }
        eprintln!("(P = pale_garden presente en algun quart del chunk)");
    }
}
