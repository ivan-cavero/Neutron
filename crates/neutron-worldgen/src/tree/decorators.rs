//! TreeDecorator ports — beehive, pale_moss, creaking_heart, place_on_ground.
use super::*;
use crate::feature_rng::FeatureRandom;
use crate::region_buf::RegionBuf;
use crate::surface::BlockId;
use serde_json::Value;
use crate::feature_catalog;

pub(super) fn apply_decorators(ctx: &mut TreeCtx<'_>, decorators: &[Value]) {
    for dec in decorators {
        let ty = dec["type"].as_str().unwrap_or("");
        match ty {
            "minecraft:beehive" => place_beehive(ctx, dec),
            "minecraft:place_on_ground" => place_on_ground(ctx, dec),
            "minecraft:pale_moss" => place_pale_moss(ctx, dec),
            "minecraft:creaking_heart" => place_creaking_heart(ctx, dec),
            "minecraft:trunk_vine" => place_trunk_vine(ctx),
            "minecraft:attached_to_logs" => place_attached_to_logs(ctx, dec),
            // leave_vine / attached_to_leaves: not used by placed features in
            // this region set yet — skip.
            _ => {}
        }
    }
}

/// `TrunkVineDecorator.place` (26.2): per log, west/east/north/south in that
/// order — each rolls `nextInt(3) > 0` ALWAYS (draw unconditional), then
/// vine placed iff the neighbor is air.
fn place_trunk_vine(ctx: &mut TreeCtx<'_>) {
    for i in 0..ctx.trunks.len() {
        let (x, y, z) = ctx.trunks[i];
        for &(dx, dz) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
            if ctx.rng.next_int(3) > 0 && ctx.region.get(x + dx, y, z + dz).is_air() {
                ctx.region.set(x + dx, y, z + dz, BlockId::Vine);
            }
        }
    }
}

/// `AttachedToLogsDecorator.place` (26.2): `Util.shuffledCopy(logs, random)`
/// (Fisher-Yates, draws n-1), then PER LOG: `Util.getRandom(directions)`
/// draw ALWAYS, `nextFloat() <= probability` ALWAYS, and only then
/// isAir + weighted block_provider draw + set.
fn place_attached_to_logs(ctx: &mut TreeCtx<'_>, dec: &Value) {
    let probability = dec["probability"].as_f64().unwrap_or(0.0) as f32;
    let mut shuffled: Vec<(i32, i32, i32)> = ctx.trunks.clone();
    crate::deco_util::shuffle(&mut shuffled, ctx.rng);
    let dirs: Vec<(i32, i32, i32)> = dec["directions"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|d| match d.as_str()? {
                    "up" => Some((0, 1, 0)),
                    "down" => Some((0, -1, 0)),
                    "north" => Some((0, 0, -1)),
                    "south" => Some((0, 0, 1)),
                    "west" => Some((-1, 0, 0)),
                    "east" => Some((1, 0, 0)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    if dirs.is_empty() {
        return;
    }
    for &(lx, ly, lz) in &shuffled {
        let di = ctx.rng.next_int(dirs.len() as i32);
        let (ddx, ddy, ddz) = dirs[di as usize];
        let (px, py, pz) = (lx + ddx, ly + ddy, lz + ddz);
        if ctx.rng.next_f32() <= probability && ctx.region.get(px, py, pz).is_air() {
            if let Some(st) =
                crate::feature_dispatch::sampling::block_from_to_place(ctx.rng, &dec["block_provider"])
            {
                ctx.region.set(px, py, pz, st);
            }
        }
    }
}

/// Port of `PaleMossDecorator.place` (26.2 CFR), RNG order exact:
///   1. `Util.shuffledCopy(context.logs(), random)` — Fisher-Yates over the
///      Y-sorted trunk list (consumes nextInt(size..2)).
///   2. origin = min-Y trunk; `nextFloat() < ground_probability` → place the
///      `pale_moss_patch` configured feature at origin.above().
///   3. per trunk (Y-sorted): nextFloat < trunk_probability && air below →
///      hanging-moss hanger.
///   4. per leaf (Y-sorted): nextFloat < leaves_probability && air below →
///      hanging-moss hanger.
fn place_pale_moss(ctx: &mut TreeCtx<'_>, dec: &Value) {
    let ground_prob = dec["ground_probability"].as_f64().unwrap_or(0.0) as f32;
    let trunk_prob = dec["trunk_probability"].as_f64().unwrap_or(0.0) as f32;
    let leaves_prob = dec["leaves_probability"].as_f64().unwrap_or(0.0) as f32;
    let trace = std::env::var_os("NEUTRON_RNG_TRACE").is_some();
    // Util.shuffledCopy(context.logs(), random) — consumes RNG even when the
    // ground check then fails.
    let mut shuffled: Vec<(i32, i32, i32)> = ctx.trunks.clone();
    crate::deco_util::shuffle(&mut shuffled, ctx.rng);
    if shuffled.is_empty() {
        return;
    }
    // Collections.min(logs, comparingInt(Y)) — FIRST minimal element on ties
    // (Java only replaces the winner when compare > 0), over the shuffled list.
    let origin = *shuffled
        .iter()
        .fold(&shuffled[0], |best, p| if p.1 < best.1 { p } else { best });
    let groll = ctx.rng.next_f32();
    if trace {
        eprintln!("[palemoss] ground-roll={groll} (<{ground_prob} entra)");
    }
    if groll < ground_prob {
        // PALE_MOSS_PATCH configured feature at origin.above().
        if let Some(cfg) = feature_catalog::load_configured_feature("pale_moss_patch") {
            if trace {
                eprintln!("[palemoss] patch start trunks={} leaves={}", ctx.trunks.len(), ctx.foliage.len());
            }
            crate::feature_dispatch::place_vegetation_patch(
                ctx.rng,
                ctx.region,
                ctx.state,
                origin.0,
                origin.1 + 1,
                origin.2,
                &cfg,
                crate::feature_catalog::step::VEGETAL_DECORATION,
            );
            if trace {
                eprintln!("[palemoss] patch end");
            }
        }
    }
    if trace {
        eprintln!("[palemoss] trunk loop n={}", ctx.trunks.len());
    }
    for i in 0..ctx.trunks.len() {
        let (tx, ty, tz) = ctx.trunks[i];
        if ctx.rng.next_f32() < trunk_prob && ctx.region.get(tx, ty - 1, tz).is_air() {
            add_pale_moss_hanger(ctx, tx, ty - 1, tz);
        }
    }
    if trace {
        eprintln!("[palemoss] leaves loop n={}", ctx.foliage.len());
    }
    for i in 0..ctx.foliage.len() {
        let (tx, ty, tz) = ctx.foliage[i];
        if ctx.rng.next_f32() < leaves_prob && ctx.region.get(tx, ty - 1, tz).is_air() {
            add_pale_moss_hanger(ctx, tx, ty - 1, tz);
        }
    }
    if trace {
        eprintln!("[palemoss] done");
    }
}

/// Port of `CreakingHeartDecorator.place` (26.2 CFR). RNG: one nextFloat for
/// the probability gate, then a Fisher-Yates shuffle of the trunk list.
/// The first trunk whose six cardinal neighbours are all logs becomes a
/// dormant natural creaking heart.
fn place_creaking_heart(ctx: &mut TreeCtx<'_>, dec: &Value) {
    if ctx.trunks.is_empty() {
        return;
    }
    let probability = dec["probability"].as_f64().unwrap_or(0.0) as f32;
    if ctx.rng.next_f32() >= probability {
        return;
    }
    let mut placements: Vec<(i32, i32, i32)> = ctx.trunks.clone();
    crate::deco_util::shuffle(&mut placements, ctx.rng);
    // Direction.values() = DOWN, UP, NORTH, SOUTH, WEST, EAST.
    const DIRS: [(i32, i32, i32); 6] = [
        (0, -1, 0),
        (0, 1, 0),
        (0, 0, -1),
        (0, 0, 1),
        (-1, 0, 0),
        (1, 0, 0),
    ];
    let target = placements.into_iter().find(|&(x, y, z)| {
        DIRS.iter()
            .all(|&(dx, dy, dz)| is_log(ctx.region.get(x + dx, y + dy, z + dz)))
    });
    if let Some((hx, hy, hz)) = target {
        ctx.region.set(hx, hy, hz, BlockId::CreakingHeart);
    }
}

fn add_pale_moss_hanger(ctx: &mut TreeCtx<'_>, x: i32, y: i32, z: i32) {
    let mut px = x;
    let mut py = y;
    let mut pz = z;
    while ctx.region.get(px, py - 1, pz).is_air() && ctx.rng.next_f32() >= 0.5 {
        ctx.region.set(px, py, pz, BlockId::PaleHangingMoss);
        py -= 1;
    }
    ctx.region.set(px, py, pz, BlockId::PaleHangingMoss);
}

fn place_beehive(ctx: &mut TreeCtx<'_>, dec: &Value) {
    if ctx.trunks.is_empty() {
        return;
    }
    let probability = dec["probability"].as_f64().unwrap_or(0.0) as f32;
    if ctx.rng.next_f32() >= probability {
        return;
    }
    let min_log_y = ctx.trunks.iter().map(|p| p.1).min().unwrap();
    let max_log_y = ctx.trunks.iter().map(|p| p.1).max().unwrap();
    let hive_y = if !ctx.foliage.is_empty() {
        let min_leaf_y = ctx.foliage.iter().map(|p| p.1).min().unwrap();
        (min_leaf_y - 1).max(min_log_y + 1)
    } else {
        (min_log_y + 1 + ctx.rng.next_int(3)).min(max_log_y)
    };

    // SPAWN_DIRECTIONS = HORIZONTAL except NORTH (WORLDGEN_FACING=SOUTH).
    const DIRS: [(i32, i32); 3] = [(1, 0), (0, 1), (-1, 0)]; // EAST, SOUTH, WEST
    let mut placements = Vec::new();
    for &(lx, ly, lz) in &ctx.trunks {
        if ly != hive_y {
            continue;
        }
        for (dx, dz) in DIRS {
            placements.push((lx + dx, ly, lz + dz));
        }
    }
    if placements.is_empty() {
        return;
    }
    crate::deco_util::shuffle(&mut placements, ctx.rng);
    let hive = placements.into_iter().find(|&(hx, hy, hz)| {
        ctx.region.get(hx, hy, hz).is_air() && ctx.region.get(hx, hy, hz + 1).is_air()
    });
    if hive.is_some() {
        // Bee nest BlockId is not in the palette. Consume occupant RNG as vanilla.
        let num_bees = 2 + ctx.rng.next_int(2);
        for _ in 0..num_bees {
            let _ = ctx.rng.next_int(599);
        }
    }
}

fn place_on_ground(ctx: &mut TreeCtx<'_>, dec: &Value) {
    if ctx.trunks.is_empty() {
        return;
    }
    let min_y = ctx.trunks.iter().map(|p| p.1).min().unwrap();
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;
    for &(tx, ty, tz) in &ctx.trunks {
        if ty != min_y {
            continue;
        }
        min_x = min_x.min(tx);
        max_x = max_x.max(tx);
        min_z = min_z.min(tz);
        max_z = max_z.max(tz);
    }
    let tries = dec["tries"].as_i64().unwrap_or(128) as i32;
    let radius = dec["radius"].as_i64().unwrap_or(2) as i32;
    let height = dec["height"].as_i64().unwrap_or(1) as i32;
    let bb_min_x = min_x - radius;
    let bb_max_x = max_x + radius;
    let bb_min_y = min_y - height;
    let bb_max_y = min_y + height;
    let bb_min_z = min_z - radius;
    let bb_max_z = max_z + radius;
    let provider = &dec["block_state_provider"];
    for _ in 0..tries {
        let px = crate::deco_util::next_int_inclusive(ctx.rng, bb_min_x, bb_max_x);
        let py = crate::deco_util::next_int_inclusive(ctx.rng, bb_min_y, bb_max_y);
        let pz = crate::deco_util::next_int_inclusive(ctx.rng, bb_min_z, bb_max_z);
        let above = (px, py + 1, pz);
        let here = ctx.region.get(px, py, pz);
        let above_b = ctx.region.get(above.0, above.1, above.2);
        // Vanilla attemptToPlaceBlockAbove (PlaceOnGroundDecorator.java:80)
        // accepts above ∈ {air, VINE}. Accepting vine here was TESTED and
        // REJECTED: 424242 regressed 568,109 → 568,965 (+856). Neutron's vine
        // positions diverge from vanilla's (origin-order cascade), so the
        // extra accepts write leaf_litter where vanilla has air. Keep the
        // strict air gate until the origin-order mechanism is resolved.
        if !above_b.is_air() {
            continue;
        }
        if !is_solid_render(here) {
            continue;
        }
        if heightmap_motion_blocking_no_leaves(ctx.region, px, pz) > above.1 {
            continue;
        }
        let block = sample_state_provider(ctx.rng, provider);
        ctx.region.set(above.0, above.1, above.2, block);
    }
}



/// `BlockBehaviour.BlockStateBase.isSolidRender` = canOcclude (full opaque
/// cube). Every non-full-cube plant/leaf/fluid in the worldgen palette reads
/// as NOT solid render (26.2 block classes). Each success of a place_on_ground
/// try draws from the weighted_state_provider, so any wrong accept/reject here
/// desyncs the rest of the chunk's decoration stream.
fn is_solid_render(b: BlockId) -> bool {
    !matches!(
        b,
        BlockId::Air
            | BlockId::CaveAir
            | BlockId::Water
            | BlockId::Lava
            | BlockId::SnowLayer // layer: canOcclude false (snow_block IS solid render)
            | BlockId::OakLeaves
            | BlockId::DarkOakLeaves
            | BlockId::PaleOakLeaves
            | BlockId::BirchLeaves
            | BlockId::SpruceLeaves
            | BlockId::JungleLeaves
            | BlockId::AcaciaLeaves
            | BlockId::MangroveLeaves
            | BlockId::CherryLeaves
            | BlockId::ShortGrass
            | BlockId::LeafLitter
            | BlockId::SculkVein
            | BlockId::MossCarpet
            | BlockId::PaleMossCarpet
            | BlockId::PaleHangingMoss
            | BlockId::Vine
            | BlockId::GlowLichen
            | BlockId::CaveVines
            | BlockId::CaveVinesPlant
            | BlockId::HangingRoots
            | BlockId::Azalea
            | BlockId::FloweringAzalea
    )
}

/// Heightmap.Types.MOTION_BLOCKING_NO_LEAVES: blocksMotion() || !fluid.isEmpty(),
/// minus LeavesBlock instances. All #minecraft:leaves are excluded regardless of
/// species (the old list only excluded oak/dark oak, so birch canopies in dark
/// forest wrongly raised the heightmap and rejected ground tries).
fn is_motion_blocking_no_leaves(b: BlockId) -> bool {
    !matches!(
        b,
        BlockId::Air
            | BlockId::CaveAir
            | BlockId::OakLeaves
            | BlockId::DarkOakLeaves
            | BlockId::PaleOakLeaves
            | BlockId::BirchLeaves
            | BlockId::SpruceLeaves
            | BlockId::JungleLeaves
            | BlockId::AcaciaLeaves
            | BlockId::MangroveLeaves
            | BlockId::CherryLeaves
            | BlockId::ShortGrass
            | BlockId::LeafLitter
            | BlockId::SculkVein
            | BlockId::MossCarpet
            | BlockId::PaleMossCarpet
            | BlockId::PaleHangingMoss
            | BlockId::Vine
            | BlockId::GlowLichen
            | BlockId::CaveVines
            | BlockId::CaveVinesPlant
            | BlockId::HangingRoots
            | BlockId::SnowLayer // layer overrides blocksMotion() to false
    )
}

fn heightmap_motion_blocking_no_leaves(region: &RegionBuf, x: i32, z: i32) -> i32 {
    for y in (WORLD_BOTTOM..WORLD_TOP).rev() {
        if is_motion_blocking_no_leaves(region.get(x, y, z)) {
            return y + 1;
        }
    }
    WORLD_BOTTOM
}

fn sample_state_provider(rng: &mut FeatureRandom, v: &Value) -> BlockId {
    let ty = v["type"].as_str().unwrap_or("");
    if ty.ends_with("weighted_state_provider") {
        let Some(entries) = v["entries"].as_array() else {
            return BlockId::LeafLitter;
        };
        let total: i32 = entries
            .iter()
            .map(|e| e["weight"].as_i64().unwrap_or(1) as i32)
            .sum();
        if total > 0 {
            let _ = rng.next_int(total);
        }
        return entries
            .iter()
            .find_map(|e| {
                e.pointer("/data/Name")
                    .and_then(|n| n.as_str())
                    .and_then(BlockId::from_name)
            })
            .unwrap_or(BlockId::LeafLitter);
    }
    block_from_provider(v).unwrap_or(BlockId::LeafLitter)
}


