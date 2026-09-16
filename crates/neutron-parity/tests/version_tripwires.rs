//! Version-drift tripwires: fail LOUDLY when a new vanilla version changes
//! content instead of silently manufacturing phantom diffs.
//!
//! These tests read whatever reference worlds exist on disk under
//! tools/nbt-ref/ (gitignored); they skip cleanly when none are present so
//! `cargo test --workspace` stays usable on machines without refs.

use neutron_parity::refdata::{ParityError, RegionSet};
use neutron_parity::vanilla_resolves;
use neutron_worldgen::feature_dispatch::biome_id_to_name;
use neutron_worldgen::surface::{vanilla_name, BlockId};
use std::collections::BTreeSet;

const REF_DIRS: &[&str] = &[
    "tools/nbt-ref/vanilla-fresh-424242/world/dimensions/minecraft/overworld/region",
    "tools/nbt-ref/vanilla-fresh-12345/dimensions/minecraft/overworld/region",
    "tools/nbt-ref/vanilla-fresh-777/dimensions/minecraft/overworld/region",
];

/// Known-unmapped vanilla names in CURRENT refs: porting debt, tracked here
/// so any NEW name fails this test immediately (upgrade-day tripwire).
/// Fixing a name means removing it from this list — stale entries also fail,
/// keeping the list honest.
const UNMAPPED_ALLOWLIST: &[&str] = &[
    "minecraft:bubble_column",
    "minecraft:iron_chain",
    "minecraft:suspicious_gravel",
];

fn workspace_root() -> std::path::PathBuf {
    // Tests must not depend on CWD: resolve <repo>/ from our manifest dir.
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("manifest under <repo>/crates/neutron-parity")
        .to_path_buf()
}

fn existing_refs() -> Vec<std::path::PathBuf> {
    REF_DIRS
        .iter()
        .map(|d| workspace_root().join(d))
        .filter(|p| p.is_dir())
        .collect()
}

fn collect_ref_names(
    dir: &std::path::Path,
) -> Result<(BTreeSet<String>, BTreeSet<String>), ParityError> {
    let mut regions = RegionSet::open(dir)?;
    let d = regions.discover()?;
    let mut blocks = BTreeSet::new();
    let mut biomes = BTreeSet::new();
    for &(cx, cz) in &d.full {
        let Some(chunk) = regions.load_chunk(cx, cz, neutron_parity::DimSpec::OVERWORLD)? else {
            continue;
        };
        // Only palette members matter: walk sections' distinct names via the
        // grid is O(cells); cheaper to re-walk palettes through the grid we
        // already have — but grids are flat strings, so just insert them all.
        blocks.extend(chunk.blocks.names.iter().cloned());
        if let Some(b) = &chunk.biomes {
            biomes.extend(b.names.iter().cloned());
        }
    }
    Ok((blocks, biomes))
}

/// Every vanilla block name present in any on-disk reference must be
/// representable by our palette. New version added/renamed a block?
/// This fails with its exact name instead of degrading parity numbers.
#[test]
fn ref_block_palettes_fully_mapped() {
    let refs = existing_refs();
    if refs.is_empty() {
        eprintln!("skip: no reference worlds on disk");
        return;
    }
    let mut unknown: BTreeSet<String> = BTreeSet::new();
    for dir in refs {
        let (blocks, _) = collect_ref_names(&dir).expect("decode ref");
        for n in blocks {
            if !vanilla_resolves(&n) {
                unknown.insert(n);
            }
        }
    }
    let new: Vec<_> = unknown
        .iter()
        .filter(|n| !UNMAPPED_ALLOWLIST.contains(&n.as_str()))
        .collect();
    assert!(
        new.is_empty(),
        "UNMAPPED vanilla block names in reference (version drift!). \
         Add mappings in surface.rs BlockId/from_name/vanilla_name (+ \
         server protocol_data), then move them out of UNMAPPED_ALLOWLIST: {new:?}"
    );
    let stale: Vec<_> = UNMAPPED_ALLOWLIST
        .iter()
        .filter(|n| !unknown.contains(&(**n).to_string()))
        .collect();
    assert!(
        stale.is_empty(),
        "allowlist entries no longer appear in refs (fix landed?): remove {stale:?}"
    );
}

/// Every biome stored in references must resolve through our id->name table.
/// A new biome in 26.x flips feature dispatch everywhere — catch it by name.
#[test]
fn ref_biome_palettes_fully_mapped() {
    let refs = existing_refs();
    if refs.is_empty() {
        eprintln!("skip: no reference worlds on disk");
        return;
    }
    let mut known: BTreeSet<&'static str> = BTreeSet::new();
    for id in 0..=255u8 {
        known.insert(biome_id_to_name(id));
    }
    let mut missing: BTreeSet<String> = BTreeSet::new();
    for dir in refs {
        let (_, biomes) = collect_ref_names(&dir).expect("decode ref");
        for n in biomes {
            // Refs store namespaced names; our table uses bare ids' names.
            let bare = n.trim_start_matches("minecraft:").to_string();
            if !bare.is_empty() && !known.contains(bare.as_str()) {
                missing.insert(bare);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "biome names present in references but absent from biome_id_to_name \
         (add to biome/source.rs + predicates.rs + OVERWORLD_BIOME_ORDER + \
         multi_noise param list): {missing:?}"
    );
}

/// The two hand-maintained name tables must agree for every variant, and
/// every emitted name must re-parse through from_name (tolerating the two
/// intentional internal-split pairs that share one vanilla name).
#[test]
fn block_id_tables_consistent() {
    let mut checked = 0usize;
    for v in 0..=u16::MAX {
        let Some(b) = BlockId::from_u16(v) else {
            continue;
        };
        checked += 1;
        let via_method = b.block_name();
        let via_fn = vanilla_name(b);
        assert_eq!(
            via_method,
            via_fn,
            "{via_fn}: block_name()/vanilla_name() disagree for variant {v} — \
             fix the drifted table"
        );
        assert!(
            BlockId::from_name(via_fn).is_some(),
            "{via_fn}: emitted by vanilla_name() but unparseable by from_name()"
        );
    }
    assert!(checked > 100, "BlockId iteration found only {checked} variants — from_u16 broken?");
}
