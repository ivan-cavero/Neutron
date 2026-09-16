//! Two-sided dungeon-box dump: ref NBT vs my full-pipeline chunk, printed as
//! per-Y layers so block-level differences read directly.
//! Usage: dungeon_pair_dump <seed> <cx> <cz> <x0> <y0> <z0> <x1> <y1> <z1>
use neutron_world::nbt::ussr_nbt::owned::{List, Tag};
use neutron_world::nbt::{compound_get, read_nbt};
use neutron_world::Region;
use neutron_worldgen::surface::vanilla_name;
use neutron_worldgen::{ChunkGenerator, NoiseCache};
use std::collections::HashMap;
use std::path::PathBuf;

fn load_vanilla(region_dir: &str, cx: i32, cz: i32) -> HashMap<(u8, i32, u8), String> {
    let (rx, rz) = (cx >> 5, cz >> 5);
    let path = PathBuf::from(format!("{region_dir}/r.{rx}.{rz}.mca"));
    let region = Region::open(&path).expect("open").with_coords(rx, rz);
    let data = region.get_chunk(cx & 31, cz & 31).expect("get").expect("present");
    let nbt = read_nbt(&data).expect("nbt");
    let sections = match compound_get(&nbt.compound, "sections") {
        Some(Tag::List(List::Compound(l))) => l,
        _ => panic!("no sections"),
    };
    let mut map = HashMap::new();
    for sec in sections {
        let y_sec = match compound_get(sec, "Y") {
            Some(Tag::Byte(y)) => *y as i8 as i32,
            _ => continue,
        };
        let Some(Tag::Compound(bs)) = compound_get(sec, "block_states") else { continue };
        let Some(Tag::List(List::Compound(palette))) = compound_get(bs, "palette") else { continue };
        let names: Vec<String> = palette
            .iter()
            .map(|pc| match compound_get(pc, "Name") {
                Some(Tag::String(s)) => s.to_string(),
                _ => "minecraft:air".into(),
            })
            .collect();
        if names.len() == 1 {
            for i in 0..4096u32 {
                map.insert(
                    ((i & 15) as u8, y_sec * 16 + (i >> 8) as i32, ((i >> 4) & 15) as u8),
                    names[0].clone(),
                );
            }
            continue;
        }
        let bits = ((names.len() - 1).ilog2() + 1).max(4) as u32;
        let Some(Tag::LongArray(d)) = compound_get(bs, "data") else { continue };
        let longs: Vec<i64> = d.to_vec();
        let epl = 64 / bits;
        let mask = (1u64 << bits) - 1;
        for i in 0..4096u32 {
            let li = (i / epl) as usize;
            let bo = (i % epl) * bits;
            let idx = ((longs[li] as u64) >> bo) & mask;
            map.insert(
                ((i & 15) as u8, y_sec * 16 + (i >> 8) as i32, ((i >> 4) & 15) as u8),
                names.get(idx as usize).cloned().unwrap_or_default(),
            );
        }
    }
    map
}

const LEGEND: &[(char, &str)] = &[
    ('.', "minecraft:air"),
    (',', "minecraft:cave_air"),
    ('#', "minecraft:cobblestone"),
    ('%', "minecraft:mossy_cobblestone"),
    ('S', "minecraft:spawner"),
    ('C', "minecraft:chest"),
    ('P', "minecraft:oak_planks"),
    ('F', "minecraft:oak_fence"),
    ('R', "minecraft:rail"),
    ('W', "minecraft:water"),
    ('L', "minecraft:lava"),
    ('c', "minecraft:clay"),
    ('m', "minecraft:moss_block"),
    ('d', "minecraft:deepslate"),
    ('s', "minecraft:stone"),
    ('v', "minecraft:cave_vines"),
    ('V', "minecraft:cave_vines_plant"),
    ('w', "minecraft:cobweb"),
    ('g', "minecraft:gravel"),
    ('t', "minecraft:tuff"),
];

fn sym(name: &str) -> char {
    for (c, n) in LEGEND {
        if *n == name {
            return *c;
        }
    }
    '?'
}

fn main() {
    let a: Vec<String> = std::env::args().collect();
    let seed: i64 = a[1].parse().unwrap();
    let (cx, cz): (i32, i32) = (a[2].parse().unwrap(), a[3].parse().unwrap());
    let (x0, y0, z0): (i32, i32, i32) =
        (a[4].parse().unwrap(), a[5].parse().unwrap(), a[6].parse().unwrap());
    let (x1, y1, z1): (i32, i32, i32) =
        (a[7].parse().unwrap(), a[8].parse().unwrap(), a[9].parse().unwrap());
    let dir = format!("tools/nbt-ref/vanilla-fresh-{seed}/world/dimensions/minecraft/overworld/region");
    let van = load_vanilla(&dir, cx, cz);
    let gen = ChunkGenerator::new(seed);
    let mut cache = NoiseCache::new();
    let chunk = gen.generate_chunk_cached(cx, cz, &mut cache);
    println!(
        "legend: {}",
        LEGEND.iter().map(|(c, n)| format!("{c}={n}")).collect::<Vec<_>>().join(" ")
    );
    let mut diff_total = 0u32;
    for y in y0..=y1 {
        let mut vline = String::new();
        let mut mline = String::new();
        let mut dline = String::new();
        for z in z0..=z1 {
            let mut vz = String::new();
            let mut mz = String::new();
            for x in x0..=x1 {
                let vb = van
                    .get(&((x & 15) as u8, y, (z & 15) as u8))
                    .cloned()
                    .unwrap_or_else(|| "?".into());
                let mb = chunk.block_at((x & 15) as u32, y, (z & 15) as u32).block_name();
                if vb != mb {
                    diff_total += 1;
                }
                vz.push(sym(&vb));
                mz.push(sym(mb));
            }
            vline.push_str(&format!("{vz} "));
            mline.push_str(&format!("{mz} "));
            dline.push_str(&format!("{} ", if vz == mz { ' ' } else { '^' }));
        }
        println!("y={y}  van");
        println!("       {vline}");
        println!("       mine");
        println!("       {mline}");
        println!("       diff");
        println!("       {dline}");
    }
    println!("diff cells in box: {diff_total}");
    let _ = vanilla_name;
}
