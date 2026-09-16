//! `MultifaceSpreader` port (`net.minecraft.world.level.block.MultifaceSpreader`).
//!
//! Face state is a `u8` mask (bit i = `Direction.values()[i]`), not a full
//! `BlockState`. Used by sculk veins and `MultifaceGrowthFeature`.
//!
//! Copyright (c) 2026 Neutron Contributors -- MIT License

use crate::feature_rng::FeatureRandom;
use crate::region_buf::RegionBuf;
use crate::surface::BlockId;
use std::collections::HashMap;

/// Direction.values() order: DOWN, UP, NORTH, SOUTH, WEST, EAST.
pub const DIRS: [(i32, i32, i32); 6] = [
    (0, -1, 0),
    (0, 1, 0),
    (0, 0, -1),
    (0, 0, 1),
    (-1, 0, 0),
    (1, 0, 0),
];

pub type FaceMap = HashMap<(i32, i32, i32), u8>;

/// MultifaceSpreader.SpreadType
#[derive(Clone, Copy, Debug)]
pub enum SpreadType {
    SamePosition,
    SamePlane,
    WrapAround,
}

pub const DEFAULT_SPREAD_ORDER: [SpreadType; 3] = [
    SpreadType::SamePosition,
    SpreadType::SamePlane,
    SpreadType::WrapAround,
];

/// MultifaceSpreader.SpreadPos
#[derive(Clone, Copy, Debug)]
pub struct SpreadPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// Face index 0..5 (direction the multiface attaches).
    pub face: usize,
}

impl SpreadType {
    /// SpreadType.getSpreadPos(pos, spreadDirection, fromFace)
    pub fn get_spread_pos(
        self,
        x: i32,
        y: i32,
        z: i32,
        spread_dir: usize,
        from_face: usize,
    ) -> SpreadPos {
        let (sdx, sdy, sdz) = DIRS[spread_dir];
        let (fdx, fdy, fdz) = DIRS[from_face];
        match self {
            SpreadType::SamePosition => SpreadPos {
                x,
                y,
                z,
                face: spread_dir,
            },
            SpreadType::SamePlane => SpreadPos {
                x: x + sdx,
                y: y + sdy,
                z: z + sdz,
                face: from_face,
            },
            SpreadType::WrapAround => {
                let opp = opposite(spread_dir);
                SpreadPos {
                    x: x + sdx + fdx,
                    y: y + sdy + fdy,
                    z: z + sdz + fdz,
                    face: opp,
                }
            }
        }
    }
}

fn opposite(dir: usize) -> usize {
    crate::deco_util::opposite(dir)
}

/// Config for sculk vein spreaders.
#[derive(Clone, Copy)]
pub struct VeinSpreadConfig {
    pub spread_types: &'static [SpreadType],
    /// SculkVeinSpreaderConfig.isOtherBlockValidAsSource: !is(SCULK_VEIN)
    pub other_block_valid_as_source: bool,
    /// Use SCULK_REPLACEABLE_WORLD_GEN attach rules for placement validity.
    pub worldgen: bool,
}

impl VeinSpreadConfig {
    /// sameSpaceSpreader: ONLY SAME_POSITION
    pub fn same_space() -> Self {
        static T: [SpreadType; 1] = [SpreadType::SamePosition];
        Self {
            spread_types: &T,
            other_block_valid_as_source: true, // !is(vein) for air/sculk/etc.
            worldgen: true,
        }
    }

    /// veinSpreader: DEFAULT_SPREAD_ORDER
    pub fn vein() -> Self {
        Self {
            spread_types: &DEFAULT_SPREAD_ORDER,
            other_block_valid_as_source: true,
            worldgen: true,
        }
    }
}

/// MultifaceSpreader bound to sculk_vein face map.
pub struct MultifaceSpreader {
    config: VeinSpreadConfig,
}

impl MultifaceSpreader {
    pub fn new(config: VeinSpreadConfig) -> Self {
        Self { config }
    }

    pub fn same_space() -> Self {
        Self::new(VeinSpreadConfig::same_space())
    }

    pub fn vein() -> Self {
        Self::new(VeinSpreadConfig::vein())
    }

    fn has_face_mask(mask: u8, face: usize) -> bool {
        mask & (1u8 << face) != 0
    }

    fn can_spread_from(
        &self,
        region: &RegionBuf,
        faces: &FaceMap,
        x: i32,
        y: i32,
        z: i32,
        face: usize,
    ) -> bool {
        let b = region.get(x, y, z);
        let mask = faces.get(&(x, y, z)).copied().unwrap_or(0);
        self.can_spread_from_snap(b, mask, face)
    }

    fn can_spread_from_snap(&self, b: BlockId, mask: u8, face: usize) -> bool {
        if self.config.other_block_valid_as_source && b != BlockId::SculkVein {
            return true;
        }
        Self::has_face_mask(mask, face)
    }

    /// isValidStateForPlacement: neighbour in face direction is sturdy solid.
    fn is_valid_placement_face(region: &RegionBuf, x: i32, y: i32, z: i32, face: usize) -> bool {
        let (dx, dy, dz) = DIRS[face];
        let n = region.get(x + dx, y + dy, z + dz);
        is_sturdy_attach(n)
    }

    /// SculkVeinSpreaderConfig.stateCanBeReplaced + DefaultSpreaderConfig.canSpreadInto.
    fn can_spread_into(
        &self,
        region: &RegionBuf,
        faces: &FaceMap,
        sx: i32,
        sy: i32,
        sz: i32,
        sp: SpreadPos,
    ) -> bool {
        if !self.state_can_be_replaced(region, sx, sy, sz, sp) {
            return false;
        }
        // MultifaceBlock.isValidStateForPlacement: already-has-face is checked here so
        // getSpreadFromFaceTowardDirection can fall through to the next SpreadType.
        let existing = region.get(sp.x, sp.y, sp.z);
        let bit = 1u8 << sp.face;
        if existing == BlockId::SculkVein
            && faces.get(&(sp.x, sp.y, sp.z)).copied().unwrap_or(0) & bit != 0
        {
            return false;
        }
        Self::is_valid_placement_face(region, sp.x, sp.y, sp.z, sp.face)
    }

    /// SculkVeinSpreaderConfig.stateCanBeReplaced (CFR).
    fn state_can_be_replaced(
        &self,
        region: &RegionBuf,
        sx: i32,
        sy: i32,
        sz: i32,
        sp: SpreadPos,
    ) -> bool {
        let (fdx, fdy, fdz) = DIRS[sp.face];
        let against = region.get(sp.x + fdx, sp.y + fdy, sp.z + fdz);
        if matches!(against, BlockId::Sculk | BlockId::SculkCatalyst) {
            return false;
        }
        // wrap-around: manhattan==2 + opposite-face sturdy at source.relative(face.opposite)
        let manh = (sp.x - sx).abs() + (sp.y - sy).abs() + (sp.z - sz).abs();
        // wrap-around: manhattan==2 + opposite-face sturdy at source.relative(face.opposite)
        let manh = (sp.x - sx).abs() + (sp.y - sy).abs() + (sp.z - sz).abs();
        if manh == 2 {
            let opp = opposite(sp.face);
            let (odx, ody, odz) = DIRS[opp];
            if is_face_sturdy_full(region.get(sx + odx, sy + ody, sz + odz)) {
                return false;
            }
        }
        let existing = region.get(sp.x, sp.y, sp.z);
        if existing == BlockId::Lava {
            return false;
        }
        // canBeReplaced() || super (air / this / water source)
        matches!(
            existing,
            BlockId::Air
                | BlockId::Water
                | BlockId::SculkVein
                | BlockId::ShortGrass
                | BlockId::SnowLayer
                | BlockId::LeafLitter
        )
    }

    fn place_block(
        &self,
        region: &mut RegionBuf,
        faces: &mut FaceMap,
        sx: i32,
        sy: i32,
        sz: i32,
        sp: SpreadPos,
    ) -> bool {
        if let Some(c) = trace_coord() {
            if (sp.x, sp.y, sp.z) == c || (sx, sy, sz) == c {
                eprintln!(
                    "TRACE place ({},{},{}) face={} from ({},{},{}) ok={}",
                    sp.x,
                    sp.y,
                    sp.z,
                    sp.face,
                    sx,
                    sy,
                    sz,
                    self.can_spread_into(region, faces, sx, sy, sz, sp)
                );
            }
        }
        if !self.can_spread_into(region, faces, sx, sy, sz, sp) {
            return false;
        }
        let key = (sp.x, sp.y, sp.z);
        let prev = faces.get(&key).copied().unwrap_or(0);
        let bit = 1u8 << sp.face;
        if std::env::var_os("NEUTRON_SCULK_STEPS").is_some()
            && sp.x == 97
            && sp.y == -44
            && sp.z == -21
        {
            eprintln!(
                "place_vein (97,-44,-21) face={} from ({sx},{sy},{sz}) prev={prev}",
                sp.face
            );
        }
        faces.insert(key, prev | bit);
        let b = region.get(sp.x, sp.y, sp.z);
        if matches!(
            b,
            BlockId::Air
                | BlockId::Water
                | BlockId::ShortGrass
                | BlockId::SnowLayer
                | BlockId::LeafLitter
        ) {
            region.set(sp.x, sp.y, sp.z, BlockId::SculkVein);
        }
        true
    }

    fn get_spread_from_face_toward_direction(
        &self,
        region: &RegionBuf,
        faces: &FaceMap,
        x: i32,
        y: i32,
        z: i32,
        starting_face: usize,
        spread_dir: usize,
        src_block: BlockId,
        src_mask: u8,
    ) -> Option<SpreadPos> {
        // same axis → empty
        if axis(starting_face) == axis(spread_dir) {
            return None;
        }
        let other_ok = self.config.other_block_valid_as_source && src_block != BlockId::SculkVein;
        let has_start = Self::has_face_mask(src_mask, starting_face);
        let has_spread = Self::has_face_mask(src_mask, spread_dir);
        if !(other_ok || (has_start && !has_spread)) {
            return None;
        }
        for &ty in self.config.spread_types {
            let sp = ty.get_spread_pos(x, y, z, spread_dir, starting_face);
            if self.can_spread_into(region, faces, x, y, z, sp) {
                return Some(sp);
            }
        }
        None
    }

    /// spreadAll — returns number of successful face placements.
    ///
    /// Vanilla passes the source BlockState into canSpreadFrom / hasFace; faces
    /// added mid-call must not unlock extra start/spread directions.
    pub fn spread_all(
        &self,
        region: &mut RegionBuf,
        faces: &mut FaceMap,
        x: i32,
        y: i32,
        z: i32,
    ) -> u64 {
        let src_block = region.get(x, y, z);
        let src_mask = faces.get(&(x, y, z)).copied().unwrap_or(0);
        let mut count = 0u64;
        for start_face in 0..6 {
            if !self.can_spread_from_snap(src_block, src_mask, start_face) {
                continue;
            }
            for spread_dir in 0..6 {
                if let Some(sp) = self.get_spread_from_face_toward_direction(
                    region, faces, x, y, z, start_face, spread_dir, src_block, src_mask,
                ) {
                    if self.place_block(region, faces, x, y, z, sp) {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    /// spreadFromFaceTowardRandomDirection
    pub fn spread_from_face_toward_random_direction(
        &self,
        rng: &mut FeatureRandom,
        region: &mut RegionBuf,
        faces: &mut FaceMap,
        x: i32,
        y: i32,
        z: i32,
        starting_face: usize,
    ) -> bool {
        let mut order: Vec<usize> = (0..6).collect();
        let mut i = order.len();
        while i > 1 {
            let j = rng.next_int(i as i32) as usize;
            order.swap(i - 1, j);
            i -= 1;
        }
        let src_block = region.get(x, y, z);
        let src_mask = faces.get(&(x, y, z)).copied().unwrap_or(0);
        for spread_dir in order {
            if let Some(sp) = self.get_spread_from_face_toward_direction(
                region,
                faces,
                x,
                y,
                z,
                starting_face,
                spread_dir,
                src_block,
                src_mask,
            ) {
                if self.place_block(region, faces, x, y, z, sp) {
                    return true;
                }
            }
        }
        false
    }

    /// SculkVeinBlock.regrow — set faces from collection if can attach.
    pub fn regrow(
        region: &mut RegionBuf,
        faces: &mut FaceMap,
        x: i32,
        y: i32,
        z: i32,
        face_bits: u8,
    ) -> bool {
        let mut mask = 0u8;
        for i in 0..6 {
            if face_bits & (1u8 << i) == 0 {
                continue;
            }
            if Self::is_valid_placement_face(region, x, y, z, i) {
                mask |= 1u8 << i;
            }
        }
        if mask == 0 {
            return false;
        }
        faces.insert((x, y, z), mask);
        let b = region.get(x, y, z);
        if matches!(b, BlockId::Air | BlockId::CaveAir | BlockId::Water) {
            region.set(x, y, z, BlockId::SculkVein);
        }
        true
    }
}

/// NEUTRON_TRACE_COORD="x,y,z" — log every spread touching that cell.
pub fn trace_coord() -> Option<(i32, i32, i32)> {
    static CACHED: std::sync::OnceLock<Option<(i32, i32, i32)>> = std::sync::OnceLock::new();
    *CACHED.get_or_init(|| {
        std::env::var("NEUTRON_TRACE_COORD")
            .ok()
            .and_then(|s| {
                let p: Vec<i32> = s.split(',').filter_map(|v| v.parse().ok()).collect();
                if p.len() == 3 { Some((p[0], p[1], p[2])) } else { None }
            })
    })
}

fn axis(dir: usize) -> u8 {
    match dir {
        0 | 1 => 1, // Y
        2 | 3 => 2, // Z
        _ => 0,     // X
    }
}

/// MultifaceBlock.canAttachTo: full support or collision face.
/// SCULK / catalyst are full cubes — regrow uses this (not stateCanBeReplaced,
/// which separately rejects attaching *toward* sculk in spreadAll).
fn is_sturdy_attach(b: BlockId) -> bool {
    !matches!(
        b,
        BlockId::Air
            | BlockId::Water
            | BlockId::Lava
            | BlockId::SculkVein
            | BlockId::SculkSensor
            | BlockId::SculkShrieker
            | BlockId::OakLeaves
            | BlockId::DarkOakLeaves
            | BlockId::ShortGrass
            | BlockId::LeafLitter
            | BlockId::SnowLayer
            | BlockId::PowderSnow
    )
}

/// isFaceSturdy for a full cube (wrap-around reject). Sculk/catalyst are sturdy.

/// isFaceSturdy for a full cube (wrap-around reject). Sculk/catalyst are sturdy.
/// NOTE: this is the BlockState.isFaceSturdy == collision-shape-full-block
/// shortcut, so vein/sensor/shrieker are NOT sturdy on ANY face. Verified by
/// the Java flat-floor probe: the full-cube check matches 166 sculk; a
/// face-aware variant (is_face_sturdy_at) regresses to 162.
pub(crate) fn is_face_sturdy_full(b: BlockId) -> bool {
    !matches!(
        b,
        BlockId::Air
            | BlockId::Water
            | BlockId::Lava
            | BlockId::SculkVein
            | BlockId::SculkSensor
            | BlockId::SculkShrieker
            | BlockId::OakLeaves
            | BlockId::DarkOakLeaves
            | BlockId::ShortGrass
            | BlockId::LeafLitter
            | BlockId::SnowLayer
            | BlockId::PowderSnow
    )
}

/// Shuffle Direction.allShuffled
pub fn all_shuffled(rng: &mut FeatureRandom) -> Vec<usize> {
    let mut order: Vec<usize> = (0..6).collect();
    let mut i = order.len();
    while i > 1 {
        let j = rng.next_int(i as i32) as usize;
        order.swap(i - 1, j);
        i -= 1;
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_space_places_faces_on_open_with_solid() {
        let mut region = RegionBuf::new(0, 0, 0);
        // y=10 air over deepslate floor at y=9 (within world range)
        region.set(0, 10, 0, BlockId::Air);
        region.set(0, 9, 0, BlockId::Deepslate);
        let mut faces = FaceMap::new();
        let n = MultifaceSpreader::same_space().spread_all(&mut region, &mut faces, 0, 10, 0);
        assert!(n > 0, "should place at least one face, n={n}");
        assert_eq!(region.get(0, 10, 0), BlockId::SculkVein);
        let m = faces.get(&(0, 10, 0)).copied().unwrap_or(0);
        assert!(m & (1 << 0) != 0, "should have DOWN face, mask={m}");
    }

    #[test]
    fn wrap_around_rejected_when_intermediate_is_sturdy() {
        // source (0,10,0), WRAP_AROUND toward EAST (5) from DOWN (0):
        // placement = (1, 9, 0), face = WEST (4). manhattan==2.
        // neighbour = source.relative(face.opposite=EAST) = (1,10,0).
        // If (1,10,0) is sturdy, stateCanBeReplaced is false.
        let mut region = RegionBuf::new(0, 0, 0);
        region.set(0, 10, 0, BlockId::SculkVein);
        region.set(1, 10, 0, BlockId::Deepslate);
        region.set(1, 9, 0, BlockId::Air);
        region.set(0, 9, 0, BlockId::Deepslate);
        let mut faces = FaceMap::new();
        faces.insert((0, 10, 0), 1u8 << 0); // DOWN
        let spreader = MultifaceSpreader::vein();
        let sp = SpreadType::WrapAround.get_spread_pos(0, 10, 0, 5, 0);
        assert_eq!((sp.x, sp.y, sp.z, sp.face), (1, 9, 0, 4));
        assert!(
            !spreader.can_spread_into(&region, &faces, 0, 10, 0, sp),
            "wrap around a sturdy corner must be rejected"
        );
    }

}





