//! Feature step / index resolution from vanilla biome JSON.
//!
//! `WorldgenRandom.setFeatureSeed(decorationSeed, index, step)` uses the
//! **global FeatureSorter index** (`ChunkGenerator.applyBiomeDecoration`),
//! not the index inside one biome's `features[step]` list.
//! On a Mojang drop: refresh `src/data/` (see docs/PARITY.md runbook) and
//! re-check `OVERWORLD_BIOME_ORDER` — it shifts every global index.
//!
//! Copyright (c) 2026 Neutron Contributors -- MIT License

use crate::datapack_fs;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Generation step indices (vanilla GenerationStep.Decoration ordinal).
pub mod step {
    pub const RAW_GENERATION: i32 = 0;
    pub const LAKES: i32 = 1;
    pub const LOCAL_MODIFICATIONS: i32 = 2;
    pub const UNDERGROUND_STRUCTURES: i32 = 3;
    pub const SURFACE_STRUCTURES: i32 = 4;
    pub const STRONGHOLDS: i32 = 5;
    pub const UNDERGROUND_ORES: i32 = 6;
    pub const UNDERGROUND_DECORATION: i32 = 7;
    pub const FLUID_SPRINGS: i32 = 8;
    pub const VEGETAL_DECORATION: i32 = 9;
    pub const TOP_LAYER_MODIFICATION: i32 = 10;
}

/// Overworld `BiomeSource.possibleBiomes()` first-seen order (26.2 probe).
/// FeatureSorter iterates this set; order is part of the global index.
const OVERWORLD_BIOME_ORDER: &[&str] = &[
    "mushroom_fields",
    "deep_frozen_ocean",
    "frozen_ocean",
    "deep_cold_ocean",
    "cold_ocean",
    "deep_ocean",
    "ocean",
    "deep_lukewarm_ocean",
    "lukewarm_ocean",
    "warm_ocean",
    "stony_shore",
    "swamp",
    "mangrove_swamp",
    "snowy_slopes",
    "snowy_plains",
    "snowy_beach",
    "windswept_gravelly_hills",
    "grove",
    "windswept_hills",
    "snowy_taiga",
    "windswept_forest",
    "taiga",
    "plains",
    "meadow",
    "beach",
    "forest",
    "old_growth_spruce_taiga",
    "flower_forest",
    "birch_forest",
    "dark_forest",
    "pale_garden",
    "savanna_plateau",
    "savanna",
    "jungle",
    "badlands",
    "desert",
    "wooded_badlands",
    "jagged_peaks",
    "stony_peaks",
    "frozen_river",
    "river",
    "ice_spikes",
    "old_growth_pine_taiga",
    "sunflower_plains",
    "old_growth_birch_forest",
    "sparse_jungle",
    "bamboo_jungle",
    "eroded_badlands",
    "windswept_savanna",
    "cherry_grove",
    "frozen_peaks",
    "dripstone_caves",
    "lush_caves",
    "sulfur_caves",
    "deep_dark",
];

/// Global FeatureSorter index of `placed` in `step`, or None if absent.
pub fn global_feature_index(step: i32, placed: &str) -> Option<i32> {
    let want = strip_mc(placed);
    features_per_step()
        .get(step as usize)?
        .iter()
        .position(|f| f == want)
        .map(|i| i as i32)
}

/// Global FeatureSorter list for a generation step.
pub fn features_per_step_at(step: i32) -> &'static [String] {
    features_per_step()
        .get(step as usize)
        .map(|v| v.as_slice())
        .unwrap_or(&[])
}

fn features_per_step() -> &'static Vec<Vec<String>> {
    static CACHE: OnceLock<Vec<Vec<String>>> = OnceLock::new();
    CACHE.get_or_init(build_features_per_step)
}

/// Port of `FeatureSorter.buildFeaturesPerStep` over overworld biomes.
fn build_features_per_step() -> Vec<Vec<String>> {
    use std::collections::{BTreeMap, BTreeSet};

    #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct FeatureData {
        step: i32,
        feature_index: i32,
    }

    let mut name_of: HashMap<i32, String> = HashMap::new();
    let mut index_of: HashMap<String, i32> = HashMap::new();
    let mut next_index = 0i32;
    let mut edges: BTreeMap<FeatureData, BTreeSet<FeatureData>> = BTreeMap::new();
    let mut max_step = 0i32;

    for biome in OVERWORLD_BIOME_ORDER {
        let Some(steps) = biome_feature_steps(biome) else {
            continue;
        };
        max_step = max_step.max(steps.len() as i32);
        let mut flat: Vec<FeatureData> = Vec::new();
        for (step_i, list) in steps.iter().enumerate() {
            for feat in list {
                let key = strip_mc(feat).to_string();
                let idx = *index_of.entry(key.clone()).or_insert_with(|| {
                    let i = next_index;
                    next_index += 1;
                    name_of.insert(i, key);
                    i
                });
                flat.push(FeatureData {
                    step: step_i as i32,
                    feature_index: idx,
                });
            }
        }
        for i in 0..flat.len() {
            edges.entry(flat[i]).or_default();
            if i + 1 < flat.len() {
                edges.entry(flat[i]).or_default().insert(flat[i + 1]);
            }
        }
    }

    // Graph.depthFirstSearch → post-order, then reverse.
    let mut discovered: BTreeSet<FeatureData> = BTreeSet::new();
    let mut visiting: BTreeSet<FeatureData> = BTreeSet::new();
    let mut postorder: Vec<FeatureData> = Vec::new();

    fn dfs(
        node: FeatureData,
        edges: &BTreeMap<FeatureData, BTreeSet<FeatureData>>,
        discovered: &mut BTreeSet<FeatureData>,
        visiting: &mut BTreeSet<FeatureData>,
        postorder: &mut Vec<FeatureData>,
    ) -> bool {
        if discovered.contains(&node) {
            return false;
        }
        if visiting.contains(&node) {
            return true;
        }
        visiting.insert(node);
        if let Some(nexts) = edges.get(&node) {
            for &next in nexts {
                if dfs(next, edges, discovered, visiting, postorder) {
                    return true;
                }
            }
        }
        visiting.remove(&node);
        discovered.insert(node);
        postorder.push(node);
        false
    }

    for &node in edges.keys() {
        if discovered.contains(&node) {
            continue;
        }
        let _cycle = dfs(node, &edges, &mut discovered, &mut visiting, &mut postorder);
    }
    postorder.reverse();

    let mut out = vec![Vec::new(); max_step.max(0) as usize];
    for fd in postorder {
        if let Some(name) = name_of.get(&fd.feature_index) {
            let bucket = &mut out[fd.step as usize];
            if !bucket.iter().any(|s| s == name) {
                bucket.push(name.clone());
            }
        }
    }
    out
}

/// Feature index of `placed` inside biome `biome`'s step list, or None.
pub fn feature_index_in_biome(biome: &str, step: i32, placed: &str) -> Option<i32> {
    let lists = biome_feature_steps(biome)?;
    let step = step as usize;
    if step >= lists.len() {
        return None;
    }
    let want = strip_mc(placed);
    lists[step]
        .iter()
        .position(|f| strip_mc(f) == want)
        .map(|i| i as i32)
}

/// All placed feature ids for a biome at a step (empty if missing).
pub fn features_at_step(biome: &str, step: i32) -> Vec<String> {
    biome_feature_steps(biome)
        .and_then(|lists| lists.get(step as usize).cloned())
        .unwrap_or_default()
}

fn strip_mc(s: &str) -> &str {
    s.strip_prefix("minecraft:").unwrap_or(s)
}

fn biome_feature_steps(biome: &str) -> Option<&'static Vec<Vec<String>>> {
    let map = biome_features_cache();
    let key = strip_mc(biome);
    map.get(key)
}

fn biome_features_cache() -> &'static HashMap<String, Vec<Vec<String>>> {
    static CACHE: OnceLock<HashMap<String, Vec<Vec<String>>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut out = HashMap::new();
        // Prefer crate data; fall back to extract tree.
        let biome_dir = datapack_fs::worldgen_path("biome");
        let paths: Vec<std::path::PathBuf> = if biome_dir.is_dir() {
            std::fs::read_dir(&biome_dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("json"))
                .collect()
        } else {
            Vec::new()
        };
        for path in paths {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Some(steps) = parse_biome_features(&text) {
                out.insert(stem, steps);
            }
        }
        out
    })
}

fn parse_biome_features(text: &str) -> Option<Vec<Vec<String>>> {
    let v: Value = serde_json::from_str(text).ok()?;
    let arr = v.get("features")?.as_array()?;
    let mut steps = Vec::with_capacity(arr.len());
    for step in arr {
        let list = step
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(|s| s.to_string()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        steps.push(list);
    }
    Some(steps)
}

/// Biome climate attributes from the biome JSON
/// (`Biome.climateSettings`): `(temperature, has_precipitation, frozen)`.
///
/// `frozen` is `temperature_modifier == "frozen"` — the modifier itself is
/// applied per-position by the caller (`Biome.getHeightAdjustedTemperature`,
/// see `feature_ports::simple`), because it depends on the block coords.
///
/// Source: the biome JSONs under `src/data/worldgen/biome/` (the same
/// directory `biome_features_cache` reads). The old embedded-path lookup
/// never matched — no `biome/*` entry exists in `datapack_data`, so every
/// biome silently took the `(0.5, true)` default. That made
/// `warmEnoughToRain` true everywhere and suppressed every
/// `SnowAndFreezeFeature` snow placement on cold biomes (2,182 cells on
/// seed 424242; jagged_peaks/frozen_peaks/grove all read 0.5 instead of
/// their real negative temperatures).
pub fn biome_climate(biome: &str) -> (f32, bool, bool) {
    static CACHE: std::sync::LazyLock<HashMap<String, (f32, bool, bool)>> =
        std::sync::LazyLock::new(|| {
            let mut m = HashMap::new();
            let dir = datapack_fs::worldgen_path("biome");
            for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                if path.extension().and_then(|x| x.to_str()) != Some("json") {
                    continue;
                }
                let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                    continue;
                };
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let Ok(v) = serde_json::from_str::<Value>(&text) else {
                    continue;
                };
                m.insert(
                    name.to_string(),
                    (
                        v["temperature"].as_f64().unwrap_or(0.5) as f32,
                        v["has_precipitation"].as_bool().unwrap_or(true),
                        v["temperature_modifier"].as_str() == Some("frozen"),
                    ),
                );
            }
            m
        });
    CACHE
        .get(strip_mc(biome))
        .copied()
        .unwrap_or((0.5, true, false))
}


struct FeatureJsonCache {
    placed: HashMap<String, Value>,
    configured: HashMap<String, Value>,
}

fn feature_json_cache() -> &'static FeatureJsonCache {
    static CACHE: OnceLock<FeatureJsonCache> = OnceLock::new();
    CACHE.get_or_init(|| FeatureJsonCache {
        placed: load_feature_directory("placed_feature"),
        configured: load_feature_directory("configured_feature"),
    })
}

fn load_feature_directory(kind: &str) -> HashMap<String, Value> {
    let mut out = HashMap::new();
    let dir = datapack_fs::worldgen_path(kind);
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str(&text) else {
            continue;
        };
        out.insert(name, value);
    }
    out
}

/// Load a placed_feature JSON object.
///
/// Returns a `&'static Value`: cache hits borrow from the global JSON cache;
/// disk fallbacks are leaked once (bounded by the finite datapack file count)
/// to keep the return type uniform and clone-free.
pub fn load_placed_feature(id: &str) -> Option<&'static Value> {
    let name = strip_mc(id);
    if let Some(value) = feature_json_cache().placed.get(name) {
        return Some(value);
    }
    let text = datapack_fs::worldgen_json_with_fallback(&format!("placed_feature/{name}.json"))?;
    let value: Value = serde_json::from_str(&text).ok()?;
    Some(Box::leak(value.into()))
}

/// Load a configured_feature JSON object (see [`load_placed_feature`]).
pub fn load_configured_feature(id: &str) -> Option<&'static Value> {
    let name = strip_mc(id);
    if let Some(value) = feature_json_cache().configured.get(name) {
        return Some(value);
    }
    let text =
        datapack_fs::worldgen_json_with_fallback(&format!("configured_feature/{name}.json"))?;
    let value: Value = serde_json::from_str(&text).ok()?;
    Some(Box::leak(value.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_dark_sculk_indices() {
        // deep_dark features[7] = [sculk_vein, sculk_patch_deep_dark]
        let vein = feature_index_in_biome("deep_dark", step::UNDERGROUND_DECORATION, "sculk_vein");
        let patch = feature_index_in_biome(
            "deep_dark",
            step::UNDERGROUND_DECORATION,
            "sculk_patch_deep_dark",
        );
        assert_eq!(vein, Some(0), "sculk_vein should be index 0 in step 7");
        assert_eq!(patch, Some(1), "sculk_patch_deep_dark should be index 1");
    }

    #[test]
    fn feature_sorter_matches_vanilla_probe() {
        // Ground truth: tools/java-probe ProbeFeatureOrder (26.2 overworld).
        assert_eq!(global_feature_index(7, "sculk_vein"), Some(0));
        assert_eq!(global_feature_index(7, "sculk_patch_deep_dark"), Some(1));
        assert_eq!(global_feature_index(7, "sulfur_spike_cluster"), Some(2));
        assert_eq!(global_feature_index(7, "dripstone_cluster"), Some(4));
        assert_eq!(global_feature_index(7, "ore_infested"), Some(6));
        assert_eq!(global_feature_index(6, "ore_dirt"), Some(0));
        assert_eq!(global_feature_index(6, "ore_emerald"), Some(33));
        assert_eq!(global_feature_index(6, "ore_copper"), Some(25));
        assert_eq!(global_feature_index(9, "glow_lichen"), Some(0));
        assert_eq!(global_feature_index(9, "trees_plains"), Some(52));
        assert_eq!(global_feature_index(9, "patch_leaf_litter"), Some(77));
        // dark_forest_vegetation must keep the FeatureSorter slot (setFeatureSeed).
        assert!(
            global_feature_index(9, "dark_forest_vegetation").is_some(),
            "dark_forest_vegetation missing from step 9 sorter"
        );
        assert_eq!(global_feature_index(10, "freeze_top_layer"), Some(0));
        assert_eq!(features_per_step_at(7).len(), 7);
        assert_eq!(features_per_step_at(6).len(), 34);
        assert_eq!(features_per_step_at(9).len(), 106);
    }

    #[test]
    fn sculk_patch_config_loads() {
        let cfg = load_configured_feature("sculk_patch_deep_dark").expect("config");
        assert_eq!(cfg["type"], "minecraft:sculk_patch");
        assert_eq!(cfg["config"]["charge_count"], 10);
        assert_eq!(cfg["config"]["amount_per_charge"], 32);
        assert_eq!(cfg["config"]["spread_attempts"], 64);
    }
}
