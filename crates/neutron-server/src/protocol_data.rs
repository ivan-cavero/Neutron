//! 26.2 registries, tags, and internal `BlockId` → protocol state-id map.
//!
//! Claimed "generated" — there is no codegen pipeline yet. Edit only when
//! the vanilla reports change (dialog tags, block states). Packet *layout*
//! lives in `neutron-protocol`; numeric play IDs also live in `protocol_ids`.

/// Known pack advertised during configuration.
pub const KNOWN_PACK_NAMESPACE: &str = "minecraft";
pub const KNOWN_PACK_ID: &str = "core";
pub const KNOWN_PACK_VERSION: &str = "26.2";
pub const PROTOCOL_VERSION: i32 = 776;

/// `minecraft:water[level=0]` (source). Levels 1–7 follow, then 8–15 falling.
pub const WATER_SOURCE: i32 = 86;
/// `minecraft:water[level=8]` (falling).
pub const WATER_FALLING: i32 = 94;
/// `minecraft:water[level=1]` (almost-full flow, used at air edges).
pub const WATER_FLOW: i32 = 87;
/// `minecraft:lava[level=0]`.
pub const LAVA_SOURCE: i32 = 102;
/// `minecraft:lava[level=8]` (falling).
pub const LAVA_FALLING: i32 = 110;

/// Internal BlockId (u16) -> 26.2 block-state protocol id.
pub fn block_state_id(internal: u16) -> i32 {
    match internal {
        0 => 0,
        1 => 1,
        // minecraft:cave_air — air-equivalent on the wire (no collision, no
        // render). Exact state id not pinned yet; clients treat it as air.
        139 => 0,
        2 => 2,
        3 => 4,
        4 => 6,
        10 => 10,
        11 => 11,
        12 => 9,
        14 => 13,
        15 => 8919,
        20 => 14,
        24 => 118,
        25 => 123,
        26 => 124,
        27 => 129,
        28 => 131,
        29 => 133,
        30 => 27790,
        31 => 132,
        32 => 32070,
        33 => 85,
        34 => 32071,
        35 => 134,
        36 => 130,
        37 => 27791,
        38 => 5308,
        39 => 6884,
        40 => 137,
        41 => 279,
        42 => 564,
        43 => 5307,
        44 => 6882,
        45 => 563,
        50 => WATER_SOURCE,
        51 => LAVA_SOURCE,
        52 => 578,
        53 => 13247,
        54 => 6927,
        55 => 6928,
        56 => 6946,
        57 => 12914,
        58 => 27162,
        59 => 12912,
        60 => 11444,
        61 => 11445,
        62 => 11456,
        63 => 11459,
        64 => 30415,
        65 => 30417,
        66 => 23452,
        67 => 27160,
        68 => 15275,
        69 => 11448,
        70 => 11458,
        71 => 11452,
        72 => 25926,
        73 => 24687,
        74 => 27643,
        75 => 27773,
        76 => 27646,
        77 => 27164,
        78 => 27781,
        79 => 30355,
        80 => 2248,
        81 => 30339,
        82 => 155,
        83 => 447,
        84 => 15,
        85 => 6996,
        99 => 8517,   // glow_lichen (26.2 Block.getId default state)
        100 => 8389,  // vine
        101 => 143,   // birch_log
        102 => 335,   // birch_leaves
        103 => 140,   // spruce_log
        104 => 307,   // spruce_leaves
        105 => 146,   // jungle_log
        106 => 363,   // jungle_leaves
        107 => 149,   // acacia_log
        108 => 391,   // acacia_leaves
        109 => 161,   // mangrove_log
        110 => 503,   // mangrove_leaves
        111 => 152,   // cherry_log
        112 => 419,   // cherry_leaves
        113 => 15267, // sea_pickle
        114 => 2254,  // seagrass
        115 => 2256,  // tall_seagrass
        116 => 15062, // kelp
        117 => 15088, // kelp_plant
        118 => 3368,  // mossy_cobblestone
        // 26.2 default-state wire ids (generated/reports/blocks.json via
        // `net.minecraft.data.Main --reports`; default = the state flagged
        // "default", i.e. Block.getId(defaultState())):
        86 => 30306,  // moss_carpet
        87 => 30302,  // cave_vines_plant (default berries=false)
        88 => 30250,  // cave_vines (default berries=false, age=0)
        89 => 158,    // pale_oak_log (axis=y)
        90 => 475,    // pale_oak_leaves
        91 => 32196,  // pale_moss_block
        92 => 32197,  // pale_moss_carpet (BASE=true, sides NONE)
        93 => 30304,  // azalea
        94 => 30305,  // flowering_azalea
        95 => 30413,  // hanging_roots
        96 => 30414,  // rooted_dirt
        97 => 32359,  // pale_hanging_moss
        98 => 3896,   // creaking_heart (default state)
        // 140 tall_grass (half=lower), 141 pale_moss_carpet topper — same
        // vanilla block as 92, sent with the carpet's base state.
        140 => 12924,
        141 => 32197,
        142 => 7830,  // red_mushroom_block (default all-true faces)
        143 => 7766,  // brown_mushroom_block
        144 => 7894,  // mushroom_stem
        145 => 37,   // acacia_sapling
        146 => 2326,   // allium
        147 => 21819,   // ancient_debris
        148 => 531,   // azalea_leaves
        149 => 2327,   // azure_bluet
        150 => 7001,   // basalt
        151 => 33,   // birch_sapling
        152 => 21831,   // blackstone
        153 => 2325,   // blue_orchid
        154 => 2336,   // brown_mushroom
        155 => 2251,   // bush
        156 => 6929,   // cactus
        157 => 6945,   // cactus_flower
        158 => 39,   // cherry_sapling
        159 => 32362,   // closed_eyeblossom
        160 => 2333,   // cornflower
        161 => 20975,   // crimson_fungus
        162 => 20974,   // crimson_nylium
        163 => 21031,   // crimson_roots
        164 => 20963,   // crimson_stem
        165 => 2321,   // dandelion
        166 => 41,   // dark_oak_sapling
        167 => 2250,   // dead_bush
        168 => 9574,   // deepslate_emerald_ore
        169 => 9573,   // emerald_ore
        170 => 2249,   // fern
        171 => 3406,   // fire
        172 => 32365,   // firefly_bush
        173 => 559,   // flowering_azalea_leaves
        174 => 12894,   // hay_block
        175 => 7023,   // jack_o_lantern
        176 => 35,   // jungle_sapling
        177 => 12926,   // large_fern
        178 => 12918,   // lilac
        179 => 2335,   // lily_of_the_valley
        180 => 8920,   // lily_pad
        181 => 14845,   // magma_block
        182 => 50,   // mangrove_propagule
        183 => 164,   // mangrove_roots
        184 => 8333,   // melon
        185 => 166,   // muddy_mangrove_roots
        186 => 135,   // nether_gold_ore
        187 => 11312,   // nether_quartz_ore
        188 => 20961,   // nether_sprouts
        189 => 14846,   // nether_wart_block
        190 => 6997,   // netherrack
        191 => 29,   // oak_sapling
        192 => 2329,   // orange_tulip
        193 => 2332,   // oxeye_daisy
        194 => 43,   // pale_oak_sapling
        195 => 12922,   // peony
        196 => 30307,   // pink_petals
        197 => 2331,   // pink_tulip
        198 => 2324,   // poppy
        199 => 8332,   // pumpkin
        200 => 2337,   // red_mushroom
        201 => 2328,   // red_tulip
        202 => 12920,   // rose_bush
        203 => 2252,   // short_dry_grass
        204 => 20976,   // shroomlight
        205 => 3887,   // soul_fire
        206 => 6998,   // soul_sand
        207 => 30303,   // spore_blossom
        208 => 31,   // spruce_sapling
        209 => 6947,   // sugar_cane
        210 => 12916,   // sunflower
        211 => 20941,   // sweet_berry_bush
        212 => 2253,   // tall_dry_grass
        213 => 20958,   // warped_fungus
        214 => 20957,   // warped_nylium
        215 => 20960,   // warped_roots
        216 => 20946,   // warped_stem
        217 => 20959,   // warped_wart_block
        218 => 2330,   // white_tulip
        219 => 30323,   // wildflowers
        136 => 30399, // small_dripleaf (default facing=north,half=lower,waterlogged=false)
        137 => 30357, // big_dripleaf (default facing=north,tilt=none,waterlogged=false)
        138 => 30389, // big_dripleaf_stem (default facing=north,waterlogged=false)
        246 => 7760,  // infested_stone
        247 => 32067, // infested_deepslate (default axis=y)
        248 => 7761,  // infested_cobblestone
        249 => 7762,  // infested_stone_bricks
        250 => 7763,  // infested_mossy_stone_bricks
        251 => 7764,  // infested_cracked_stone_bricks
        252 => 7765,  // infested_chiseled_stone_bricks
        253 => 20880,  // campfire
        254 => 23099,  // candle
        255 => 32063,  // chiseled_deepslate
        256 => 30419,  // cobbled_deepslate
        257 => 30503,  // cobbled_deepslate_slab
        258 => 30431,  // cobbled_deepslate_stairs
        259 => 30509,  // cobbled_deepslate_wall
        260 => 11264,  // comparator
        261 => 32064,  // cracked_deepslate_bricks
        262 => 32065,  // cracked_deepslate_tiles
        263 => 12905,  // cyan_carpet
        264 => 2302,  // cyan_wool
        265 => 13963,  // dark_oak_fence
        266 => 21,  // dark_oak_planks
        267 => 31736,  // deepslate_brick_slab
        268 => 31664,  // deepslate_brick_stairs
        269 => 31742,  // deepslate_brick_wall
        270 => 31652,  // deepslate_bricks
        271 => 31325,  // deepslate_tile_slab
        272 => 31253,  // deepslate_tile_stairs
        273 => 31331,  // deepslate_tile_wall
        274 => 31241,  // deepslate_tiles
        275 => 5328,  // furnace
        276 => 562,  // glass
        277 => 8331,  // glass_pane
        278 => 12582,  // iron_trapdoor
        279 => 5720,  // ladder
        280 => 20787,  // lectern
        281 => 6780,  // lever
        282 => 582,  // note_block
        283 => 2271,  // piston_head
        284 => 7004,  // polished_basalt
        285 => 30830,  // polished_deepslate
        286 => 30914,  // polished_deepslate_slab
        287 => 30842,  // polished_deepslate_stairs
        288 => 30920,  // polished_deepslate_wall
        289 => 11311,  // redstone_block
        290 => 9480,  // redstone_lamp
        291 => 6885,  // redstone_torch
        292 => 6887,  // redstone_wall_torch
        293 => 5171,  // redstone_wire
        294 => 32085,  // reinforced_deepslate
        295 => 7037,  // repeater
        296 => 10931,  // skeleton_skull
        297 => 20844,  // soul_lantern
        298 => 2241,  // sticky_piston
        299 => 6796,  // stone_pressure_plate
        300 => 21752,  // target
        301 => 3370,  // torch
        302 => 12907,  // blue_carpet
        303 => 2304,  // blue_wool
        304 => 12903,  // gray_carpet
        305 => 2300,  // gray_wool
        306 => 12899,  // light_blue_carpet
        307 => 2296,  // light_blue_wool
        308 => 23115,  // white_candle
        312 => 6919,   // snow (layers=1)
        _ => 1,       // unknown -> stone
    }
}

/// Internal biome u8 -> 26.2 synchronized biome protocol id.
pub fn biome_protocol_id(internal: u8) -> i32 {
    match internal {
        0 => 45,  // ocean
        1 => 1,   // plains
        2 => 5,   // desert
        3 => 8,   // forest
        4 => 16,  // taiga
        5 => 6,   // swamp
        6 => 37,  // river
        7 => 39,  // beach
        8 => 46,  // deep_ocean
        9 => 3,   // snowy_plains
        10 => 24, // jungle
        11 => 18, // savanna
        12 => 11, // dark_forest
        13 => 41, // stony_shore
        14 => 30, // meadow
        15 => 49, // frozen_ocean
        16 => 38, // frozen_river
        17 => 4,  // ice_spikes
        18 => 13, // old_growth_birch_forest
        19 => 14, // old_growth_pine_taiga
        20 => 20, // windswept_hills
        21 => 32, // grove
        22 => 33, // snowy_slopes
        23 => 35, // jagged_peaks
        24 => 34, // frozen_peaks
        25 => 36, // stony_peaks
        26 => 27, // badlands
        27 => 28, // eroded_badlands
        28 => 29, // wooded_badlands
        29 => 51, // mushroom_fields
        30 => 31, // cherry_grove
        31 => 54, // deep_dark
        32 => 7,  // mangrove_swamp
        33 => 10, // birch_forest
        34 => 53, // lush_caves
        35 => 52, // dripstone_caves
        36 => 55, // sulfur_caves
        _ => 1,   // plains
    }
}

/// Synchronized registries in send order. Entry order defines protocol IDs.
pub static SYNC_REGISTRIES: &[(&str, &[&str])] = &[
    (
        "minecraft:dimension_type",
        &[
            "minecraft:overworld",
            "minecraft:overworld_caves",
            "minecraft:the_end",
            "minecraft:the_nether",
        ],
    ),
    (
        "minecraft:worldgen/biome",
        &[
            "minecraft:the_void",
            "minecraft:plains",
            "minecraft:sunflower_plains",
            "minecraft:snowy_plains",
            "minecraft:ice_spikes",
            "minecraft:desert",
            "minecraft:swamp",
            "minecraft:mangrove_swamp",
            "minecraft:forest",
            "minecraft:flower_forest",
            "minecraft:birch_forest",
            "minecraft:dark_forest",
            "minecraft:pale_garden",
            "minecraft:old_growth_birch_forest",
            "minecraft:old_growth_pine_taiga",
            "minecraft:old_growth_spruce_taiga",
            "minecraft:taiga",
            "minecraft:snowy_taiga",
            "minecraft:savanna",
            "minecraft:savanna_plateau",
            "minecraft:windswept_hills",
            "minecraft:windswept_gravelly_hills",
            "minecraft:windswept_forest",
            "minecraft:windswept_savanna",
            "minecraft:jungle",
            "minecraft:sparse_jungle",
            "minecraft:bamboo_jungle",
            "minecraft:badlands",
            "minecraft:eroded_badlands",
            "minecraft:wooded_badlands",
            "minecraft:meadow",
            "minecraft:cherry_grove",
            "minecraft:grove",
            "minecraft:snowy_slopes",
            "minecraft:frozen_peaks",
            "minecraft:jagged_peaks",
            "minecraft:stony_peaks",
            "minecraft:river",
            "minecraft:frozen_river",
            "minecraft:beach",
            "minecraft:snowy_beach",
            "minecraft:stony_shore",
            "minecraft:warm_ocean",
            "minecraft:lukewarm_ocean",
            "minecraft:deep_lukewarm_ocean",
            "minecraft:ocean",
            "minecraft:deep_ocean",
            "minecraft:cold_ocean",
            "minecraft:deep_cold_ocean",
            "minecraft:frozen_ocean",
            "minecraft:deep_frozen_ocean",
            "minecraft:mushroom_fields",
            "minecraft:dripstone_caves",
            "minecraft:lush_caves",
            "minecraft:deep_dark",
            "minecraft:sulfur_caves",
            "minecraft:nether_wastes",
            "minecraft:warped_forest",
            "minecraft:crimson_forest",
            "minecraft:soul_sand_valley",
            "minecraft:basalt_deltas",
            "minecraft:the_end",
            "minecraft:end_highlands",
            "minecraft:end_midlands",
            "minecraft:small_end_islands",
            "minecraft:end_barrens",
        ],
    ),
    (
        "minecraft:banner_pattern",
        &[
            "minecraft:base",
            "minecraft:border",
            "minecraft:bricks",
            "minecraft:circle",
            "minecraft:creeper",
            "minecraft:cross",
            "minecraft:curly_border",
            "minecraft:diagonal_left",
            "minecraft:diagonal_right",
            "minecraft:diagonal_up_left",
            "minecraft:diagonal_up_right",
            "minecraft:flow",
            "minecraft:flower",
            "minecraft:globe",
            "minecraft:gradient",
            "minecraft:gradient_up",
            "minecraft:guster",
            "minecraft:half_horizontal",
            "minecraft:half_horizontal_bottom",
            "minecraft:half_vertical",
            "minecraft:half_vertical_right",
            "minecraft:mojang",
            "minecraft:piglin",
            "minecraft:rhombus",
            "minecraft:skull",
            "minecraft:small_stripes",
            "minecraft:square_bottom_left",
            "minecraft:square_bottom_right",
            "minecraft:square_top_left",
            "minecraft:square_top_right",
            "minecraft:straight_cross",
            "minecraft:stripe_bottom",
            "minecraft:stripe_center",
            "minecraft:stripe_downleft",
            "minecraft:stripe_downright",
            "minecraft:stripe_left",
            "minecraft:stripe_middle",
            "minecraft:stripe_right",
            "minecraft:stripe_top",
            "minecraft:triangle_bottom",
            "minecraft:triangle_top",
            "minecraft:triangles_bottom",
            "minecraft:triangles_top",
        ],
    ),
    (
        "minecraft:cat_sound_variant",
        &["minecraft:classic", "minecraft:royal"],
    ),
    (
        "minecraft:cat_variant",
        &[
            "minecraft:all_black",
            "minecraft:black",
            "minecraft:british_shorthair",
            "minecraft:calico",
            "minecraft:jellie",
            "minecraft:persian",
            "minecraft:ragdoll",
            "minecraft:red",
            "minecraft:siamese",
            "minecraft:tabby",
            "minecraft:white",
        ],
    ),
    (
        "minecraft:chat_type",
        &[
            "minecraft:chat",
            "minecraft:emote_command",
            "minecraft:msg_command_incoming",
            "minecraft:msg_command_outgoing",
            "minecraft:say_command",
            "minecraft:team_msg_command_incoming",
            "minecraft:team_msg_command_outgoing",
        ],
    ),
    (
        "minecraft:chicken_sound_variant",
        &["minecraft:classic", "minecraft:picky"],
    ),
    (
        "minecraft:chicken_variant",
        &["minecraft:cold", "minecraft:temperate", "minecraft:warm"],
    ),
    (
        "minecraft:cow_sound_variant",
        &["minecraft:classic", "minecraft:moody"],
    ),
    (
        "minecraft:cow_variant",
        &["minecraft:cold", "minecraft:temperate", "minecraft:warm"],
    ),
    (
        "minecraft:damage_type",
        &[
            "minecraft:arrow",
            "minecraft:bad_respawn_point",
            "minecraft:cactus",
            "minecraft:campfire",
            "minecraft:cramming",
            "minecraft:dragon_breath",
            "minecraft:drown",
            "minecraft:dry_out",
            "minecraft:ender_pearl",
            "minecraft:explosion",
            "minecraft:fall",
            "minecraft:falling_anvil",
            "minecraft:falling_block",
            "minecraft:falling_stalactite",
            "minecraft:fireball",
            "minecraft:fireworks",
            "minecraft:fly_into_wall",
            "minecraft:freeze",
            "minecraft:generic",
            "minecraft:generic_kill",
            "minecraft:hot_floor",
            "minecraft:in_fire",
            "minecraft:in_wall",
            "minecraft:indirect_magic",
            "minecraft:lava",
            "minecraft:lightning_bolt",
            "minecraft:mace_smash",
            "minecraft:magic",
            "minecraft:mob_attack",
            "minecraft:mob_attack_no_aggro",
            "minecraft:mob_projectile",
            "minecraft:on_fire",
            "minecraft:out_of_world",
            "minecraft:outside_border",
            "minecraft:player_attack",
            "minecraft:player_explosion",
            "minecraft:sonic_boom",
            "minecraft:spear",
            "minecraft:spit",
            "minecraft:stalagmite",
            "minecraft:starve",
            "minecraft:sting",
            "minecraft:sulfur_cube_hot",
            "minecraft:sweet_berry_bush",
            "minecraft:thorns",
            "minecraft:thrown",
            "minecraft:trident",
            "minecraft:unattributed_fireball",
            "minecraft:wind_charge",
            "minecraft:wither",
            "minecraft:wither_skull",
        ],
    ),
    (
        "minecraft:dialog",
        &[
            "minecraft:custom_options",
            "minecraft:quick_actions",
            "minecraft:server_links",
        ],
    ),
    (
        "minecraft:enchantment",
        &[
            "minecraft:aqua_affinity",
            "minecraft:bane_of_arthropods",
            "minecraft:binding_curse",
            "minecraft:blast_protection",
            "minecraft:breach",
            "minecraft:channeling",
            "minecraft:density",
            "minecraft:depth_strider",
            "minecraft:efficiency",
            "minecraft:feather_falling",
            "minecraft:fire_aspect",
            "minecraft:fire_protection",
            "minecraft:flame",
            "minecraft:fortune",
            "minecraft:frost_walker",
            "minecraft:impaling",
            "minecraft:infinity",
            "minecraft:knockback",
            "minecraft:looting",
            "minecraft:loyalty",
            "minecraft:luck_of_the_sea",
            "minecraft:lunge",
            "minecraft:lure",
            "minecraft:mending",
            "minecraft:multishot",
            "minecraft:piercing",
            "minecraft:power",
            "minecraft:projectile_protection",
            "minecraft:protection",
            "minecraft:punch",
            "minecraft:quick_charge",
            "minecraft:respiration",
            "minecraft:riptide",
            "minecraft:sharpness",
            "minecraft:silk_touch",
            "minecraft:smite",
            "minecraft:soul_speed",
            "minecraft:sweeping_edge",
            "minecraft:swift_sneak",
            "minecraft:thorns",
            "minecraft:unbreaking",
            "minecraft:vanishing_curse",
            "minecraft:wind_burst",
        ],
    ),
    (
        "minecraft:frog_variant",
        &["minecraft:cold", "minecraft:temperate", "minecraft:warm"],
    ),
    (
        "minecraft:instrument",
        &[
            "minecraft:admire_goat_horn",
            "minecraft:call_goat_horn",
            "minecraft:dream_goat_horn",
            "minecraft:feel_goat_horn",
            "minecraft:ponder_goat_horn",
            "minecraft:seek_goat_horn",
            "minecraft:sing_goat_horn",
            "minecraft:yearn_goat_horn",
        ],
    ),
    (
        "minecraft:jukebox_song",
        &[
            "minecraft:11",
            "minecraft:13",
            "minecraft:5",
            "minecraft:blocks",
            "minecraft:bounce",
            "minecraft:cat",
            "minecraft:chirp",
            "minecraft:creator",
            "minecraft:creator_music_box",
            "minecraft:far",
            "minecraft:lava_chicken",
            "minecraft:mall",
            "minecraft:mellohi",
            "minecraft:otherside",
            "minecraft:pigstep",
            "minecraft:precipice",
            "minecraft:relic",
            "minecraft:stal",
            "minecraft:strad",
            "minecraft:tears",
            "minecraft:wait",
            "minecraft:ward",
        ],
    ),
    (
        "minecraft:painting_variant",
        &[
            "minecraft:alban",
            "minecraft:aztec",
            "minecraft:aztec2",
            "minecraft:backyard",
            "minecraft:baroque",
            "minecraft:bomb",
            "minecraft:bouquet",
            "minecraft:burning_skull",
            "minecraft:bust",
            "minecraft:cavebird",
            "minecraft:changing",
            "minecraft:cotan",
            "minecraft:courbet",
            "minecraft:creebet",
            "minecraft:dennis",
            "minecraft:donkey_kong",
            "minecraft:earth",
            "minecraft:endboss",
            "minecraft:fern",
            "minecraft:fighters",
            "minecraft:finding",
            "minecraft:fire",
            "minecraft:graham",
            "minecraft:humble",
            "minecraft:kebab",
            "minecraft:lowmist",
            "minecraft:match",
            "minecraft:meditative",
            "minecraft:orb",
            "minecraft:owlemons",
            "minecraft:passage",
            "minecraft:pigscene",
            "minecraft:plant",
            "minecraft:pointer",
            "minecraft:pond",
            "minecraft:pool",
            "minecraft:prairie_ride",
            "minecraft:sea",
            "minecraft:skeleton",
            "minecraft:skull_and_roses",
            "minecraft:stage",
            "minecraft:sunflowers",
            "minecraft:sunset",
            "minecraft:tides",
            "minecraft:unpacked",
            "minecraft:void",
            "minecraft:wanderer",
            "minecraft:wasteland",
            "minecraft:water",
            "minecraft:wind",
            "minecraft:wither",
        ],
    ),
    (
        "minecraft:pig_sound_variant",
        &["minecraft:big", "minecraft:classic", "minecraft:mini"],
    ),
    (
        "minecraft:pig_variant",
        &["minecraft:cold", "minecraft:temperate", "minecraft:warm"],
    ),
    (
        "minecraft:sulfur_cube_archetype",
        &[
            "minecraft:bouncy",
            "minecraft:explosive",
            "minecraft:fast_flat",
            "minecraft:fast_sliding",
            "minecraft:high_resistance",
            "minecraft:hot",
            "minecraft:light",
            "minecraft:regular",
            "minecraft:slow_bouncy",
            "minecraft:slow_flat",
            "minecraft:slow_sliding",
            "minecraft:sticky",
        ],
    ),
    ("minecraft:test_environment", &["minecraft:default"]),
    ("minecraft:test_instance", &["minecraft:always_pass"]),
    (
        "minecraft:timeline",
        &[
            "minecraft:day",
            "minecraft:early_game",
            "minecraft:moon",
            "minecraft:villager_schedule",
        ],
    ),
    (
        "minecraft:trim_material",
        &[
            "minecraft:amethyst",
            "minecraft:copper",
            "minecraft:diamond",
            "minecraft:emerald",
            "minecraft:gold",
            "minecraft:iron",
            "minecraft:lapis",
            "minecraft:netherite",
            "minecraft:quartz",
            "minecraft:redstone",
            "minecraft:resin",
        ],
    ),
    (
        "minecraft:trim_pattern",
        &[
            "minecraft:bolt",
            "minecraft:coast",
            "minecraft:dune",
            "minecraft:eye",
            "minecraft:flow",
            "minecraft:host",
            "minecraft:raiser",
            "minecraft:rib",
            "minecraft:sentry",
            "minecraft:shaper",
            "minecraft:silence",
            "minecraft:snout",
            "minecraft:spire",
            "minecraft:tide",
            "minecraft:vex",
            "minecraft:ward",
            "minecraft:wayfinder",
            "minecraft:wild",
        ],
    ),
    (
        "minecraft:wolf_sound_variant",
        &[
            "minecraft:angry",
            "minecraft:big",
            "minecraft:classic",
            "minecraft:cute",
            "minecraft:grumpy",
            "minecraft:puglin",
            "minecraft:sad",
        ],
    ),
    (
        "minecraft:wolf_variant",
        &[
            "minecraft:ashen",
            "minecraft:black",
            "minecraft:chestnut",
            "minecraft:pale",
            "minecraft:rusty",
            "minecraft:snowy",
            "minecraft:spotted",
            "minecraft:striped",
            "minecraft:woods",
        ],
    ),
    (
        "minecraft:world_clock",
        &["minecraft:overworld", "minecraft:the_end"],
    ),
    (
        "minecraft:zombie_nautilus_variant",
        &["minecraft:temperate", "minecraft:warm"],
    ),
];

/// Tag data: (registry, tag_name without #, entry protocol ids).
pub static TAGS: &[(&str, &str, &[i32])] = &[
    (
        "minecraft:banner_pattern",
        "no_item_required",
        &[
            26, 27, 28, 29, 31, 38, 35, 37, 32, 36, 34, 33, 25, 5, 30, 39, 40, 41, 42, 7, 10, 9, 8,
            3, 23, 19, 17, 20, 18, 1, 14, 15,
        ],
    ),
    (
        "minecraft:banner_pattern",
        "pattern_item/bordure_indented",
        &[6],
    ),
    ("minecraft:banner_pattern", "pattern_item/creeper", &[4]),
    (
        "minecraft:banner_pattern",
        "pattern_item/field_masoned",
        &[2],
    ),
    ("minecraft:banner_pattern", "pattern_item/flow", &[11]),
    ("minecraft:banner_pattern", "pattern_item/flower", &[12]),
    ("minecraft:banner_pattern", "pattern_item/globe", &[13]),
    ("minecraft:banner_pattern", "pattern_item/guster", &[16]),
    ("minecraft:banner_pattern", "pattern_item/mojang", &[21]),
    ("minecraft:banner_pattern", "pattern_item/piglin", &[22]),
    ("minecraft:banner_pattern", "pattern_item/skull", &[24]),
    ("minecraft:block", "acacia_logs", &[53, 75, 64, 83]),
    ("minecraft:block", "air", &[0, 794, 795]),
    (
        "minecraft:block",
        "all_hanging_signs",
        &[
            234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250,
            251, 252, 253, 255, 256, 254, 257,
        ],
    ),
    (
        "minecraft:block",
        "all_signs",
        &[
            210, 211, 212, 213, 215, 216, 217, 901, 902, 218, 219, 214, 224, 225, 226, 227, 229,
            230, 231, 903, 904, 232, 233, 228, 234, 235, 236, 237, 238, 239, 240, 241, 242, 243,
            244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 255, 256, 254, 257,
        ],
    ),
    (
        "minecraft:block",
        "ancient_city_replaceable",
        &[
            1151, 1164, 1160, 1166, 1162, 1165, 1163, 1167, 1152, 1169, 1170, 147,
        ],
    ),
    ("minecraft:block", "animals_spawnable_on", &[8]),
    ("minecraft:block", "anvil", &[467, 468, 469]),
    (
        "minecraft:block",
        "armadillo_spawnable_on",
        &[8, 554, 484, 488, 485, 498, 496, 492, 39, 10],
    ),
    ("minecraft:block", "axolotls_spawnable_on", &[281]),
    (
        "minecraft:block",
        "azalea_grows_on",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 37, 39, 38, 554, 484, 485, 486, 487,
            488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 278, 1027,
        ],
    ),
    (
        "minecraft:block",
        "azalea_root_replaceable",
        &[
            1, 2, 4, 6, 984, 1151, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 554, 484, 485,
            486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 39, 281, 40, 37,
            278, 1027,
        ],
    ),
    (
        "minecraft:block",
        "badlands_terracotta",
        &[554, 484, 488, 485, 498, 496, 492],
    ),
    ("minecraft:block", "bamboo_blocks", &[60, 70]),
    (
        "minecraft:block",
        "banners",
        &[
            563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 579,
            580, 581, 582, 583, 584, 585, 586, 587, 588, 589, 590, 591, 592, 593, 594,
        ],
    ),
    (
        "minecraft:block",
        "bars",
        &[341, 342, 343, 344, 345, 346, 347, 348, 349],
    ),
    ("minecraft:block", "base_stone_nether", &[285, 288, 924]),
    (
        "minecraft:block",
        "base_stone_overworld",
        &[1, 2, 4, 6, 984, 1151],
    ),
    (
        "minecraft:block",
        "bats_spawnable_on",
        &[1, 2, 4, 6, 984, 1151],
    ),
    (
        "minecraft:block",
        "beacon_base_blocks",
        &[915, 403, 205, 174, 175],
    ),
    (
        "minecraft:block",
        "beds",
        &[
            110, 111, 112, 113, 114, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125,
        ],
    ),
    (
        "minecraft:block",
        "bee_attractive",
        &[
            157, 1191, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 171, 170, 159, 557, 558,
            560, 559, 664, 98, 1139, 33, 93, 1141, 1142, 657, 1137, 280,
        ],
    ),
    (
        "minecraft:block",
        "bee_growables",
        &[665, 441, 442, 207, 365, 364, 662, 663, 861, 1135, 1136],
    ),
    ("minecraft:block", "beehives", &[911, 912]),
    (
        "minecraft:block",
        "beneath_bamboo_podzol_replaceable",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373],
    ),
    (
        "minecraft:block",
        "beneath_tree_podzol_replaceable",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373],
    ),
    ("minecraft:block", "birch_logs", &[51, 73, 62, 81]),
    (
        "minecraft:block",
        "blocks_wind_charge_explosions",
        &[524, 34],
    ),
    (
        "minecraft:block",
        "buttons",
        &[
            443, 444, 445, 446, 447, 449, 450, 897, 898, 451, 452, 448, 275, 939,
        ],
    ),
    (
        "minecraft:block",
        "camel_sand_step_sound_blocks",
        &[
            37, 39, 38, 726, 727, 728, 729, 730, 731, 732, 733, 734, 735, 736, 737, 738, 739, 740,
            741,
        ],
    ),
    ("minecraft:block", "camels_spawnable_on", &[37, 39, 38]),
    ("minecraft:block", "campfires", &[859, 860]),
    (
        "minecraft:block",
        "can_glide_through",
        &[366, 880, 881, 878, 879, 1136, 1135],
    ),
    (
        "minecraft:block",
        "candle_cakes",
        &[
            961, 962, 963, 964, 965, 966, 967, 968, 969, 970, 971, 972, 973, 974, 975, 976, 977,
        ],
    ),
    (
        "minecraft:block",
        "candles",
        &[
            944, 945, 946, 947, 948, 949, 950, 951, 952, 953, 954, 955, 956, 957, 958, 959, 960,
        ],
    ),
    (
        "minecraft:block",
        "cannot_replace_below_tree_trunk",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 11],
    ),
    ("minecraft:block", "cannot_support_kelp", &[671]),
    ("minecraft:block", "cannot_support_seagrass", &[671]),
    (
        "minecraft:block",
        "cannot_support_snow_layer",
        &[277, 556, 524],
    ),
    ("minecraft:block", "cauldrons", &[387, 388, 389, 390]),
    (
        "minecraft:block",
        "causes_continuous_geyser_eruptions",
        &[36],
    ),
    (
        "minecraft:block",
        "causes_periodic_geyser_eruptions",
        &[671],
    ),
    ("minecraft:block", "cave_vines", &[1136, 1135]),
    (
        "minecraft:block",
        "ceiling_hanging_signs",
        &[234, 235, 236, 237, 238, 239, 240, 241, 242, 243, 244, 245],
    ),
    (
        "minecraft:block",
        "chains",
        &[350, 351, 352, 353, 354, 355, 356, 357, 358],
    ),
    ("minecraft:block", "cherry_logs", &[54, 76, 65, 84]),
    (
        "minecraft:block",
        "climbable",
        &[221, 366, 837, 878, 879, 880, 881, 1135, 1136],
    ),
    ("minecraft:block", "coal_ores", &[46, 47]),
    (
        "minecraft:block",
        "combination_step_sound_blocks",
        &[
            538, 539, 540, 541, 542, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553, 1140,
            1189, 276, 870, 869, 882, 368,
        ],
    ),
    (
        "minecraft:block",
        "completes_find_tree_tutorial",
        &[
            55, 77, 66, 85, 56, 20, 67, 86, 49, 71, 68, 79, 53, 75, 64, 83, 51, 73, 62, 81, 52, 74,
            63, 82, 50, 72, 61, 80, 57, 78, 69, 87, 54, 76, 65, 84, 871, 872, 873, 874, 862, 863,
            864, 865, 91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93, 672, 868,
        ],
    ),
    (
        "minecraft:block",
        "concrete",
        &[
            710, 711, 712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 722, 723, 724, 725,
        ],
    ),
    (
        "minecraft:block",
        "concrete_powders",
        &[
            726, 727, 728, 729, 730, 731, 732, 733, 734, 735, 736, 737, 738, 739, 740, 741,
        ],
    ),
    ("minecraft:block", "convertable_to_mud", &[9, 10, 1149]),
    (
        "minecraft:block",
        "copper",
        &[1034, 1035, 1036, 1037, 1038, 1039, 1040, 1041],
    ),
    (
        "minecraft:block",
        "copper_chests",
        &[1108, 1109, 1110, 1111, 1112, 1113, 1114, 1115],
    ),
    (
        "minecraft:block",
        "copper_golem_statues",
        &[1116, 1117, 1118, 1119, 1120, 1121, 1122, 1123],
    ),
    ("minecraft:block", "copper_ores", &[1042, 1043]),
    (
        "minecraft:block",
        "coral_blocks",
        &[753, 754, 755, 756, 757],
    ),
    (
        "minecraft:block",
        "coral_plants",
        &[763, 764, 765, 766, 767],
    ),
    (
        "minecraft:block",
        "corals",
        &[763, 764, 765, 766, 767, 773, 774, 775, 776, 777],
    ),
    ("minecraft:block", "crimson_stems", &[871, 872, 873, 874]),
    (
        "minecraft:block",
        "crops",
        &[665, 441, 442, 207, 365, 364, 662, 663],
    ),
    ("minecraft:block", "crystal_sound_blocks", &[978, 979]),
    (
        "minecraft:block",
        "dampens_vibrations",
        &[
            140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155, 538,
            539, 540, 541, 542, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553,
        ],
    ),
    ("minecraft:block", "dark_oak_logs", &[55, 77, 66, 85]),
    (
        "minecraft:block",
        "deepslate_ore_replaceables",
        &[1151, 984],
    ),
    ("minecraft:block", "diamond_ores", &[203, 204]),
    ("minecraft:block", "dirt", &[9, 10, 1149]),
    ("minecraft:block", "does_not_block_hoppers", &[911, 912]),
    (
        "minecraft:block",
        "doors",
        &[
            220, 646, 647, 648, 649, 651, 652, 899, 900, 653, 654, 650, 1076, 1077, 1078, 1079,
            1080, 1081, 1082, 1083, 260,
        ],
    ),
    (
        "minecraft:block",
        "dragon_immune",
        &[
            524, 34, 391, 392, 667, 407, 668, 669, 905, 906, 156, 193, 917, 393, 341, 918, 1182,
            907, 908,
        ],
    ),
    ("minecraft:block", "dragon_transparent", &[525, 196, 197]),
    (
        "minecraft:block",
        "dripstone_replaceable_blocks",
        &[1, 2, 4, 6, 984, 1151],
    ),
    ("minecraft:block", "edible_for_sheep", &[130, 134, 135, 131]),
    ("minecraft:block", "emerald_ores", &[398, 399]),
    ("minecraft:block", "enables_bubble_column_drag_down", &[671]),
    ("minecraft:block", "enables_bubble_column_push_up", &[286]),
    ("minecraft:block", "enchantment_power_provider", &[178]),
    (
        "minecraft:block",
        "enchantment_power_transmitter",
        &[
            0, 35, 36, 130, 131, 132, 133, 134, 135, 136, 137, 196, 197, 276, 366, 367, 368, 525,
            561, 562, 675, 794, 795, 796, 869, 870, 882, 1143, 1148,
        ],
    ),
    (
        "minecraft:block",
        "enderman_holdable",
        &[
            157, 1191, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 171, 170, 159, 1192, 158,
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 37, 39, 40, 172, 173, 177, 279, 281,
            360, 296, 361, 876, 875, 882, 867, 866, 869, 280,
        ],
    ),
    (
        "minecraft:block",
        "fall_damage_resetting",
        &[221, 366, 837, 878, 879, 880, 881, 1135, 1136, 861, 129],
    ),
    (
        "minecraft:block",
        "features_cannot_replace",
        &[34, 198, 201, 392, 1182, 1185, 1186],
    ),
    (
        "minecraft:block",
        "fence_gates",
        &[631, 629, 633, 634, 630, 369, 628, 893, 894, 635, 636, 632],
    ),
    (
        "minecraft:block",
        "fences",
        &[
            284, 640, 642, 643, 637, 638, 639, 889, 890, 644, 645, 641, 382,
        ],
    ),
    ("minecraft:block", "fire", &[196, 197]),
    (
        "minecraft:block",
        "flower_pots",
        &[
            411, 1193, 1194, 425, 426, 427, 428, 429, 430, 431, 432, 433, 423, 413, 414, 415, 416,
            417, 419, 420, 437, 438, 439, 422, 440, 434, 435, 436, 793, 919, 920, 921, 922, 1176,
            1177, 421, 418, 412, 424,
        ],
    ),
    (
        "minecraft:block",
        "flowers",
        &[
            157, 1191, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 171, 170, 159, 1192, 158,
            557, 558, 560, 559, 664, 98, 1139, 33, 93, 1141, 1142, 657, 1137, 280,
        ],
    ),
    (
        "minecraft:block",
        "forest_rock_can_place_on",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 1, 2, 4, 6, 984, 1151,
        ],
    ),
    ("minecraft:block", "fox_immune_to", &[861]),
    (
        "minecraft:block",
        "foxes_spawnable_on",
        &[8, 276, 278, 11, 10],
    ),
    ("minecraft:block", "frog_prefer_jump_to", &[374, 1145]),
    ("minecraft:block", "frogs_spawnable_on", &[8, 1150, 58, 59]),
    (
        "minecraft:block",
        "geode_invalid_blocks",
        &[34, 35, 36, 277, 556, 789],
    ),
    (
        "minecraft:block",
        "glazed_terracotta",
        &[
            694, 695, 696, 697, 698, 699, 700, 701, 702, 703, 704, 705, 706, 707, 708, 709,
        ],
    ),
    (
        "minecraft:block",
        "goats_spawnable_on",
        &[8, 1, 276, 278, 556, 40],
    ),
    ("minecraft:block", "gold_ores", &[42, 48, 43]),
    ("minecraft:block", "grass_blocks", &[8, 11, 373]),
    ("minecraft:block", "grows_crops", &[208]),
    (
        "minecraft:block",
        "guarded_by_piglins",
        &[
            1108, 1109, 1110, 1111, 1112, 1113, 1114, 1115, 174, 839, 201, 400, 935, 470, 1175,
            677, 678, 679, 680, 681, 682, 683, 684, 685, 686, 687, 688, 689, 690, 691, 692, 693,
            42, 48, 43,
        ],
    ),
    (
        "minecraft:block",
        "happy_ghast_avoids",
        &[861, 279, 170, 671, 196, 1133, 1134],
    ),
    (
        "minecraft:block",
        "hoglin_repellents",
        &[867, 920, 295, 918],
    ),
    (
        "minecraft:block",
        "huge_brown_mushroom_can_place_on",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 373, 11, 875, 866,
        ],
    ),
    (
        "minecraft:block",
        "huge_red_mushroom_can_place_on",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 373, 11, 875, 866,
        ],
    ),
    ("minecraft:block", "ice", &[277, 556, 789, 670]),
    (
        "minecraft:block",
        "ice_spike_replaceable",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 278, 277],
    ),
    (
        "minecraft:block",
        "impermeable",
        &[
            300, 301, 302, 303, 304, 305, 306, 307, 308, 309, 310, 311, 312, 313, 314, 315, 101,
            1026, 524,
        ],
    ),
    (
        "minecraft:block",
        "incorrect_for_copper_tool",
        &[
            193, 917, 915, 918, 916, 205, 203, 204, 398, 399, 403, 174, 1175, 42, 43, 271, 272,
        ],
    ),
    (
        "minecraft:block",
        "incorrect_for_gold_tool",
        &[
            193, 917, 915, 918, 916, 205, 203, 204, 398, 399, 403, 174, 1175, 42, 43, 271, 272,
            175, 1173, 44, 45, 104, 102, 103, 1174, 1042, 1043, 1184, 1108, 1109, 1110, 1111, 1112,
            1113, 1114, 1115, 1124, 1125, 1126, 1127, 1128, 1129, 1130, 1131, 1034, 1035, 1036,
            1037, 1038, 1039, 1040, 1041, 1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1044,
            1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1058,
            1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072,
            1073, 1074, 1075, 1084, 1085, 1086, 1087, 1088, 1089, 1090, 1091, 1092, 1093, 1094,
            1095, 1096, 1097, 1098, 1099,
        ],
    ),
    (
        "minecraft:block",
        "incorrect_for_iron_tool",
        &[193, 917, 915, 918, 916],
    ),
    (
        "minecraft:block",
        "incorrect_for_stone_tool",
        &[
            193, 917, 915, 918, 916, 205, 203, 204, 398, 399, 403, 174, 1175, 42, 43, 271, 272,
        ],
    ),
    (
        "minecraft:block",
        "incorrect_for_wooden_tool",
        &[
            193, 917, 915, 918, 916, 205, 203, 204, 398, 399, 403, 174, 1175, 42, 43, 271, 272,
            175, 1173, 44, 45, 104, 102, 103, 1174, 1042, 1043, 1184, 1108, 1109, 1110, 1111, 1112,
            1113, 1114, 1115, 1124, 1125, 1126, 1127, 1128, 1129, 1130, 1131, 1034, 1035, 1036,
            1037, 1038, 1039, 1040, 1041, 1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1044,
            1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1058,
            1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072,
            1073, 1074, 1075, 1084, 1085, 1086, 1087, 1088, 1089, 1090, 1091, 1092, 1093, 1094,
            1095, 1096, 1097, 1098, 1099,
        ],
    ),
    ("minecraft:block", "infiniburn_end", &[285, 671, 34]),
    ("minecraft:block", "infiniburn_nether", &[285, 671]),
    ("minecraft:block", "infiniburn_overworld", &[285, 671]),
    (
        "minecraft:block",
        "inside_step_sound_blocks",
        &[1027, 1031, 367, 374, 983, 1141, 1142, 1143],
    ),
    ("minecraft:block", "invalid_spawn_inside", &[391, 667]),
    ("minecraft:block", "iron_ores", &[44, 45]),
    ("minecraft:block", "jungle_logs", &[52, 74, 63, 82]),
    (
        "minecraft:block",
        "lanterns",
        &[849, 850, 851, 852, 853, 854, 855, 856, 857, 858],
    ),
    ("minecraft:block", "lapis_ores", &[102, 103]),
    (
        "minecraft:block",
        "lava_pool_stone_cannot_replace",
        &[
            34, 198, 201, 392, 1182, 1185, 1186, 91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93, 55,
            77, 66, 85, 56, 20, 67, 86, 49, 71, 68, 79, 53, 75, 64, 83, 51, 73, 62, 81, 52, 74, 63,
            82, 50, 72, 61, 80, 57, 78, 69, 87, 54, 76, 65, 84, 871, 872, 873, 874, 862, 863, 864,
            865,
        ],
    ),
    (
        "minecraft:block",
        "leaves",
        &[91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93],
    ),
    (
        "minecraft:block",
        "lightning_rods",
        &[1124, 1125, 1126, 1127, 1128, 1129, 1130, 1131],
    ),
    (
        "minecraft:block",
        "logs",
        &[
            55, 77, 66, 85, 56, 20, 67, 86, 49, 71, 68, 79, 53, 75, 64, 83, 51, 73, 62, 81, 52, 74,
            63, 82, 50, 72, 61, 80, 57, 78, 69, 87, 54, 76, 65, 84, 871, 872, 873, 874, 862, 863,
            864, 865,
        ],
    ),
    (
        "minecraft:block",
        "logs_that_burn",
        &[
            55, 77, 66, 85, 56, 20, 67, 86, 49, 71, 68, 79, 53, 75, 64, 83, 51, 73, 62, 81, 52, 74,
            63, 82, 50, 72, 61, 80, 57, 78, 69, 87, 54, 76, 65, 84,
        ],
    ),
    (
        "minecraft:block",
        "lush_ground_replaceable",
        &[
            1, 2, 4, 6, 984, 1151, 1136, 1135, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 281,
            40, 37,
        ],
    ),
    (
        "minecraft:block",
        "maintains_farmland",
        &[
            364, 362, 365, 363, 665, 441, 442, 662, 159, 663, 207, 156, 631, 629, 633, 634, 630,
            369, 628, 893, 894, 635, 636, 632,
        ],
    ),
    ("minecraft:block", "mangrove_logs", &[57, 78, 69, 87]),
    (
        "minecraft:block",
        "mangrove_logs_can_grow_through",
        &[1150, 59, 58, 96, 57, 33, 1140, 366],
    ),
    (
        "minecraft:block",
        "mangrove_roots_can_grow_through",
        &[1150, 59, 58, 1140, 366, 33, 276],
    ),
    (
        "minecraft:block",
        "mineable/axe",
        &[
            109, 792, 839, 911, 912, 1146, 1145, 178, 338, 859, 842, 296, 201, 657, 656, 396, 909,
            206, 474, 843, 367, 297, 283, 221, 845, 838, 361, 340, 360, 339, 846, 860, 470, 366,
            563, 564, 565, 566, 567, 568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 579,
            580, 581, 582, 583, 584, 585, 586, 587, 588, 589, 590, 591, 592, 593, 594, 631, 629,
            633, 634, 630, 369, 628, 893, 894, 635, 636, 632, 55, 77, 66, 85, 56, 20, 67, 86, 49,
            71, 68, 79, 53, 75, 64, 83, 51, 73, 62, 81, 52, 74, 63, 82, 50, 72, 61, 80, 57, 78, 69,
            87, 54, 76, 65, 84, 871, 872, 873, 874, 862, 863, 864, 865, 13, 14, 15, 16, 17, 19, 21,
            883, 884, 22, 23, 18, 210, 211, 212, 213, 215, 216, 217, 901, 902, 218, 219, 214, 224,
            225, 226, 227, 229, 230, 231, 903, 904, 232, 233, 228, 443, 444, 445, 446, 447, 449,
            450, 897, 898, 451, 452, 448, 220, 646, 647, 648, 649, 651, 652, 899, 900, 653, 654,
            650, 284, 640, 642, 643, 637, 638, 639, 889, 890, 644, 645, 641, 261, 262, 263, 264,
            265, 267, 268, 887, 888, 269, 270, 266, 599, 600, 601, 602, 603, 605, 606, 885, 886,
            607, 608, 604, 200, 404, 405, 406, 516, 518, 519, 895, 896, 520, 521, 517, 320, 318,
            322, 323, 319, 316, 317, 891, 892, 324, 325, 321, 58, 234, 235, 236, 237, 238, 239,
            240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 255, 256, 254,
            257, 24, 609, 522, 60, 70, 179, 180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190,
            191, 199,
        ],
    ),
    (
        "minecraft:block",
        "mineable/hoe",
        &[
            91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93, 672, 868, 537, 744, 910, 877, 99, 100,
            1028, 1029, 1144, 1140, 1188, 1189, 1030, 1032, 1031, 1033,
        ],
    ),
    (
        "minecraft:block",
        "mineable/pickaxe",
        &[
            1, 2, 3, 4, 5, 6, 7, 12, 42, 43, 44, 45, 46, 47, 48, 102, 103, 104, 105, 106, 107, 108,
            174, 175, 176, 192, 193, 198, 203, 204, 205, 209, 223, 259, 260, 271, 272, 285, 288,
            289, 326, 327, 328, 329, 370, 371, 381, 382, 383, 385, 386, 393, 397, 398, 399, 400,
            403, 471, 472, 475, 476, 477, 478, 479, 480, 481, 483, 526, 527, 528, 529, 530, 531,
            532, 533, 534, 535, 554, 555, 595, 596, 597, 598, 610, 611, 612, 613, 614, 615, 616,
            617, 619, 620, 621, 622, 623, 624, 625, 626, 627, 658, 659, 660, 661, 671, 673, 674,
            676, 748, 749, 750, 751, 752, 753, 754, 755, 756, 757, 758, 759, 760, 761, 762, 768,
            769, 770, 771, 772, 778, 779, 780, 781, 782, 797, 798, 799, 800, 801, 802, 803, 804,
            805, 806, 807, 808, 809, 810, 811, 812, 813, 814, 815, 816, 817, 818, 819, 820, 821,
            822, 823, 840, 841, 844, 847, 848, 866, 875, 915, 916, 917, 918, 923, 924, 925, 927,
            928, 929, 930, 931, 932, 933, 935, 936, 937, 938, 941, 942, 943, 984, 1025, 1042, 1043,
            1132, 1151, 1152, 1153, 1154, 1156, 1157, 1158, 1160, 1161, 1162, 1164, 1165, 1166,
            1168, 1169, 1170, 1172, 1173, 1174, 1175, 277, 556, 789, 138, 128, 139, 980, 983, 982,
            981, 978, 979, 333, 337, 336, 1171, 332, 335, 334, 275, 939, 409, 410, 824, 825, 826,
            827, 828, 829, 831, 832, 833, 834, 835, 836, 926, 934, 940, 1155, 1159, 1163, 1167,
            830, 987, 991, 996, 379, 1015, 1019, 1023, 1002, 1006, 1010, 677, 678, 679, 680, 681,
            682, 683, 684, 685, 686, 687, 688, 689, 690, 691, 692, 693, 467, 468, 469, 387, 388,
            389, 390, 222, 126, 127, 482, 790, 331, 372, 618, 330, 1184, 985, 986, 987, 992, 988,
            989, 990, 991, 993, 994, 995, 996, 997, 1187, 376, 378, 379, 377, 380, 1012, 1013,
            1014, 1015, 1016, 1017, 1018, 1019, 1020, 1021, 1022, 1023, 1024, 998, 999, 1000, 1001,
            1002, 1003, 1004, 1005, 1006, 1007, 1008, 1009, 1010, 1011, 1108, 1109, 1110, 1111,
            1112, 1113, 1114, 1115, 1116, 1117, 1118, 1119, 1120, 1121, 1122, 1123, 1124, 1125,
            1126, 1127, 1128, 1129, 1130, 1131, 849, 850, 851, 852, 853, 854, 855, 856, 857, 858,
            350, 351, 352, 353, 354, 355, 356, 357, 358, 341, 342, 343, 344, 345, 346, 347, 348,
            349, 1034, 1035, 1036, 1037, 1038, 1039, 1040, 1041, 1100, 1101, 1102, 1103, 1104,
            1105, 1106, 1107, 1044, 1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054,
            1055, 1056, 1057, 1058, 1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068,
            1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082,
            1083, 1084, 1085, 1086, 1087, 1088, 1089, 1090, 1091, 1092, 1093, 1094, 1095, 1096,
            1097, 1098, 1099, 694, 695, 696, 697, 698, 699, 700, 701, 702, 703, 704, 705, 706, 707,
            708, 709, 484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498,
            499, 710, 711, 712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 722, 723, 724, 725,
            1133, 1134,
        ],
    ),
    (
        "minecraft:block",
        "mineable/shovel",
        &[
            281, 9, 10, 11, 208, 8, 40, 373, 37, 39, 278, 276, 286, 666, 287, 1149, 59, 1150, 38,
            41, 726, 727, 728, 729, 730, 731, 732, 733, 734, 735, 736, 737, 738, 739, 740, 741,
        ],
    ),
    (
        "minecraft:block",
        "mob_interactable_doors",
        &[
            220, 646, 647, 648, 649, 651, 652, 899, 900, 653, 654, 650, 1076, 1077, 1078, 1079,
            1080, 1081, 1082, 1083,
        ],
    ),
    ("minecraft:block", "mooshrooms_spawnable_on", &[373]),
    ("minecraft:block", "moss_blocks", &[1144, 1188]),
    (
        "minecraft:block",
        "moss_replaceable",
        &[
            1, 2, 4, 6, 984, 1151, 1136, 1135, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373,
        ],
    ),
    ("minecraft:block", "mud", &[1150, 59]),
    (
        "minecraft:block",
        "needs_diamond_tool",
        &[193, 917, 915, 918, 916],
    ),
    (
        "minecraft:block",
        "needs_iron_tool",
        &[205, 203, 204, 398, 399, 403, 174, 1175, 42, 43, 271, 272],
    ),
    (
        "minecraft:block",
        "needs_stone_tool",
        &[
            175, 1173, 44, 45, 104, 102, 103, 1174, 1042, 1043, 1184, 1108, 1109, 1110, 1111, 1112,
            1113, 1114, 1115, 1124, 1125, 1126, 1127, 1128, 1129, 1130, 1131, 1034, 1035, 1036,
            1037, 1038, 1039, 1040, 1041, 1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1044,
            1045, 1046, 1047, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1058,
            1059, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072,
            1073, 1074, 1075, 1084, 1085, 1086, 1087, 1088, 1089, 1090, 1091, 1092, 1093, 1094,
            1095, 1096, 1097, 1098, 1099,
        ],
    ),
    (
        "minecraft:block",
        "nether_carver_replaceables",
        &[
            1, 2, 4, 6, 984, 1151, 285, 288, 924, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373,
            875, 866, 672, 868, 286, 287,
        ],
    ),
    ("minecraft:block", "nylium", &[875, 866]),
    ("minecraft:block", "oak_logs", &[49, 71, 68, 79]),
    (
        "minecraft:block",
        "occludes_vibration_signals",
        &[
            140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155,
        ],
    ),
    (
        "minecraft:block",
        "overrides_mushroom_light_requirement",
        &[373, 11, 875, 866],
    ),
    (
        "minecraft:block",
        "overworld_carver_replaceables",
        &[
            1, 2, 4, 6, 984, 1151, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 37, 39, 38, 554,
            484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 44, 45,
            1042, 1043, 276, 278, 1027, 35, 40, 41, 106, 595, 1025, 556, 1173, 1174, 1012, 998,
            999,
        ],
    ),
    (
        "minecraft:block",
        "overworld_natural_logs",
        &[53, 51, 49, 52, 50, 55, 56, 57, 54],
    ),
    ("minecraft:block", "pale_oak_logs", &[56, 20, 67, 86]),
    (
        "minecraft:block",
        "parrots_spawnable_on",
        &[
            8, 0, 91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93, 55, 77, 66, 85, 56, 20, 67, 86, 49,
            71, 68, 79, 53, 75, 64, 83, 51, 73, 62, 81, 52, 74, 63, 82, 50, 72, 61, 80, 57, 78, 69,
            87, 54, 76, 65, 84, 871, 872, 873, 874, 862, 863, 864, 865,
        ],
    ),
    (
        "minecraft:block",
        "piglin_repellents",
        &[197, 290, 850, 291, 860],
    ),
    (
        "minecraft:block",
        "planks",
        &[13, 14, 15, 16, 17, 19, 21, 883, 884, 22, 23, 18],
    ),
    ("minecraft:block", "polar_bear_immune_to", &[1027]),
    (
        "minecraft:block",
        "polar_bears_spawnable_on_alternate",
        &[277],
    ),
    ("minecraft:block", "portals", &[295, 391, 667]),
    (
        "minecraft:block",
        "pressure_plates",
        &[
            471, 472, 261, 262, 263, 264, 265, 267, 268, 887, 888, 269, 270, 266, 259, 938,
        ],
    ),
    (
        "minecraft:block",
        "prevent_mob_spawning_inside",
        &[222, 126, 127, 482],
    ),
    (
        "minecraft:block",
        "prevents_nearby_leaf_decay",
        &[
            55, 77, 66, 85, 56, 20, 67, 86, 49, 71, 68, 79, 53, 75, 64, 83, 51, 73, 62, 81, 52, 74,
            63, 82, 50, 72, 61, 80, 57, 78, 69, 87, 54, 76, 65, 84, 871, 872, 873, 874, 862, 863,
            864, 865,
        ],
    ),
    (
        "minecraft:block",
        "rabbits_spawnable_on",
        &[8, 276, 278, 37],
    ),
    ("minecraft:block", "rails", &[222, 126, 127, 482]),
    ("minecraft:block", "redstone_ores", &[271, 272]),
    (
        "minecraft:block",
        "replaceable",
        &[
            0, 35, 36, 130, 131, 132, 133, 134, 135, 136, 137, 196, 197, 276, 366, 367, 368, 525,
            561, 562, 675, 794, 795, 796, 869, 870, 882, 1143, 1148,
        ],
    ),
    (
        "minecraft:block",
        "replaceable_by_mushrooms",
        &[
            91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93, 157, 1191, 160, 161, 162, 163, 164, 165,
            166, 167, 168, 169, 171, 170, 159, 1192, 158, 1189, 130, 131, 132, 366, 367, 557, 558,
            559, 560, 561, 562, 1148, 664, 35, 136, 137, 172, 173, 338, 339, 869, 870, 882, 1143,
            134, 135, 133, 1195,
        ],
    ),
    (
        "minecraft:block",
        "replaceable_by_trees",
        &[
            91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93, 157, 1191, 160, 161, 162, 163, 164, 165,
            166, 167, 168, 169, 171, 170, 159, 1192, 158, 1189, 130, 131, 132, 366, 367, 557, 558,
            559, 560, 561, 562, 1148, 664, 35, 136, 137, 133, 1195, 869, 870, 882, 1143, 134, 135,
        ],
    ),
    ("minecraft:block", "sand", &[37, 39, 38]),
    (
        "minecraft:block",
        "saplings",
        &[25, 26, 27, 28, 29, 31, 32, 1138, 1139, 33, 30],
    ),
    (
        "minecraft:block",
        "sculk_replaceable",
        &[
            1, 2, 4, 6, 984, 1151, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 554, 484, 485,
            486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 875, 866, 285,
            288, 924, 37, 39, 40, 286, 287, 1025, 1172, 281, 1132, 393, 595, 106, 998, 1012,
        ],
    ),
    (
        "minecraft:block",
        "sculk_replaceable_world_gen",
        &[
            1, 2, 4, 6, 984, 1151, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 554, 484, 485,
            486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499, 875, 866, 285,
            288, 924, 37, 39, 40, 286, 287, 1025, 1172, 281, 1132, 393, 595, 106, 998, 1012, 1164,
            1160, 1152, 1169, 1170, 1156,
        ],
    ),
    (
        "minecraft:block",
        "shears_extreme_breaking_speed",
        &[91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93],
    ),
    (
        "minecraft:block",
        "shears_major_breaking_speed",
        &[
            140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155,
        ],
    ),
    (
        "minecraft:block",
        "shears_minor_breaking_speed",
        &[367, 366],
    ),
    (
        "minecraft:block",
        "shulker_boxes",
        &[
            677, 678, 679, 680, 681, 682, 683, 684, 685, 686, 687, 688, 689, 690, 691, 692, 693,
        ],
    ),
    (
        "minecraft:block",
        "signs",
        &[
            210, 211, 212, 213, 215, 216, 217, 901, 902, 218, 219, 214, 224, 225, 226, 227, 229,
            230, 231, 903, 904, 232, 233, 228,
        ],
    ),
    (
        "minecraft:block",
        "slabs",
        &[
            599, 600, 601, 602, 603, 605, 606, 885, 886, 607, 608, 604, 609, 610, 611, 617, 612,
            623, 620, 621, 616, 615, 619, 614, 533, 534, 535, 811, 812, 813, 814, 815, 816, 817,
            818, 819, 820, 821, 822, 823, 613, 622, 927, 932, 937, 1154, 1158, 1162, 1166, 618,
            985, 989, 994, 378, 1013, 1017, 1021, 1000, 1004, 1008, 1068, 1069, 1070, 1071, 1072,
            1073, 1074, 1075,
        ],
    ),
    (
        "minecraft:block",
        "small_flowers",
        &[
            157, 1191, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 171, 170, 159, 1192, 158,
        ],
    ),
    ("minecraft:block", "smelts_to_glass", &[37, 39]),
    (
        "minecraft:block",
        "snaps_goat_horn",
        &[
            53, 51, 49, 52, 50, 55, 56, 57, 54, 1, 556, 44, 46, 1042, 398,
        ],
    ),
    (
        "minecraft:block",
        "sniffer_diggable_block",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11],
    ),
    ("minecraft:block", "sniffer_egg_hatch_boost", &[1144]),
    ("minecraft:block", "snow", &[276, 278, 1027]),
    ("minecraft:block", "snow_golem_immune_to", &[1027]),
    ("minecraft:block", "soul_fire_base_blocks", &[286, 287]),
    ("minecraft:block", "soul_speed_blocks", &[286, 287]),
    ("minecraft:block", "speleothems", &[1133, 1134]),
    ("minecraft:block", "spruce_logs", &[50, 72, 61, 80]),
    (
        "minecraft:block",
        "stairs",
        &[
            200, 404, 405, 406, 516, 518, 519, 895, 896, 520, 521, 517, 522, 223, 397, 383, 371,
            370, 660, 481, 598, 531, 530, 532, 797, 798, 799, 800, 801, 802, 803, 804, 805, 806,
            807, 808, 809, 810, 925, 933, 936, 1153, 1157, 1161, 1165, 372, 986, 990, 995, 377,
            1014, 1018, 1022, 1001, 1005, 1009, 1060, 1061, 1062, 1063, 1064, 1065, 1066, 1067,
        ],
    ),
    (
        "minecraft:block",
        "standing_signs",
        &[210, 211, 212, 213, 215, 216, 217, 901, 902, 218, 219, 214],
    ),
    ("minecraft:block", "stone_bricks", &[326, 327, 328, 329]),
    ("minecraft:block", "stone_buttons", &[275, 939]),
    ("minecraft:block", "stone_ore_replaceables", &[1, 2, 4, 6]),
    ("minecraft:block", "stone_pressure_plates", &[259, 938]),
    ("minecraft:block", "stray_immune_to", &[1027]),
    ("minecraft:block", "strider_warm_blocks", &[36]),
    (
        "minecraft:block",
        "substrate_overworld",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373],
    ),
    (
        "minecraft:block",
        "sulfur_spike_replaceable_blocks",
        &[998, 1012],
    ),
    (
        "minecraft:block",
        "support_override_cactus_flower",
        &[279, 208],
    ),
    (
        "minecraft:block",
        "support_override_snow_layer",
        &[913, 286, 1150],
    ),
    (
        "minecraft:block",
        "supports_azalea",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 281],
    ),
    (
        "minecraft:block",
        "supports_bamboo",
        &[
            37, 39, 38, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 792, 791, 40, 41,
        ],
    ),
    (
        "minecraft:block",
        "supports_big_dripleaf",
        &[281, 1144, 9, 8, 11, 10, 373, 1149, 1144, 1150, 59, 208],
    ),
    ("minecraft:block", "supports_cactus", &[37, 39, 38]),
    ("minecraft:block", "supports_chorus_flower", &[393]),
    ("minecraft:block", "supports_chorus_plant", &[393]),
    ("minecraft:block", "supports_cocoa", &[52, 74, 63, 82]),
    (
        "minecraft:block",
        "supports_crimson_fungus",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 875, 866, 373, 287,
        ],
    ),
    (
        "minecraft:block",
        "supports_crimson_roots",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 875, 866, 287,
        ],
    ),
    ("minecraft:block", "supports_crops", &[208]),
    (
        "minecraft:block",
        "supports_dry_vegetation",
        &[
            37, 39, 38, 554, 484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497,
            498, 499, 9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208,
        ],
    ),
    (
        "minecraft:block",
        "supports_hanging_mangrove_propagule",
        &[96],
    ),
    ("minecraft:block", "supports_lily_pad", &[277, 670]),
    (
        "minecraft:block",
        "supports_mangrove_propagule",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 281],
    ),
    ("minecraft:block", "supports_melon_stem", &[208]),
    (
        "minecraft:block",
        "supports_melon_stem_fruit",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208],
    ),
    (
        "minecraft:block",
        "supports_nether_sprouts",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 875, 866, 287,
        ],
    ),
    ("minecraft:block", "supports_nether_wart", &[286]),
    ("minecraft:block", "supports_pumpkin_stem", &[208]),
    (
        "minecraft:block",
        "supports_pumpkin_stem_fruit",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208],
    ),
    ("minecraft:block", "supports_small_dripleaf", &[281, 1144]),
    ("minecraft:block", "supports_stem_crops", &[208]),
    (
        "minecraft:block",
        "supports_stem_fruit",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208],
    ),
    (
        "minecraft:block",
        "supports_sugar_cane",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 37, 39, 38],
    ),
    ("minecraft:block", "supports_sugar_cane_adjacently", &[670]),
    (
        "minecraft:block",
        "supports_vegetation",
        &[9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208],
    ),
    (
        "minecraft:block",
        "supports_warped_fungus",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 875, 866, 373, 287,
        ],
    ),
    (
        "minecraft:block",
        "supports_warped_roots",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 875, 866, 287,
        ],
    ),
    (
        "minecraft:block",
        "supports_wither_rose",
        &[
            9, 10, 1149, 1150, 59, 1144, 1188, 8, 11, 373, 208, 285, 286, 287,
        ],
    ),
    ("minecraft:block", "suppresses_bounce", &[913]),
    (
        "minecraft:block",
        "sword_efficient",
        &[
            91, 88, 89, 95, 94, 92, 90, 97, 98, 96, 93, 366, 367, 360, 296, 297, 361, 396, 1145,
            1146, 656, 657,
        ],
    ),
    ("minecraft:block", "sword_instantly_mines", &[792, 791]),
    (
        "minecraft:block",
        "terracotta",
        &[
            554, 484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499,
        ],
    ),
    ("minecraft:block", "trail_ruins_replaceable", &[40]),
    (
        "minecraft:block",
        "trapdoors",
        &[
            320, 318, 322, 323, 319, 316, 317, 891, 892, 324, 325, 321, 526, 1084, 1085, 1086,
            1087, 1088, 1089, 1090, 1091,
        ],
    ),
    (
        "minecraft:block",
        "triggers_ambient_desert_dry_vegetation_block_sounds",
        &[
            554, 484, 485, 486, 487, 488, 489, 490, 491, 492, 493, 494, 495, 496, 497, 498, 499,
            37, 39,
        ],
    ),
    (
        "minecraft:block",
        "triggers_ambient_desert_sand_block_sounds",
        &[37, 39],
    ),
    (
        "minecraft:block",
        "triggers_ambient_dried_ghast_block_sounds",
        &[286, 287],
    ),
    (
        "minecraft:block",
        "underwater_bonemeals",
        &[
            136, 763, 764, 765, 766, 767, 773, 774, 775, 776, 777, 783, 784, 785, 786, 787,
        ],
    ),
    (
        "minecraft:block",
        "unstable_bottom_center",
        &[631, 629, 633, 634, 630, 369, 628, 893, 894, 635, 636, 632],
    ),
    ("minecraft:block", "valid_spawn", &[8, 11]),
    ("minecraft:block", "vibration_resonators", &[978]),
    ("minecraft:block", "wall_corals", &[783, 784, 785, 786, 787]),
    (
        "minecraft:block",
        "wall_hanging_signs",
        &[246, 247, 248, 249, 250, 251, 252, 253, 255, 256, 254, 257],
    ),
    (
        "minecraft:block",
        "wall_post_override",
        &[
            194, 290, 273, 292, 402, 210, 211, 212, 213, 215, 216, 217, 901, 902, 218, 219, 214,
            224, 225, 226, 227, 229, 230, 231, 903, 904, 232, 233, 228, 563, 564, 565, 566, 567,
            568, 569, 570, 571, 572, 573, 574, 575, 576, 577, 578, 579, 580, 581, 582, 583, 584,
            585, 586, 587, 588, 589, 590, 591, 592, 593, 594, 471, 472, 261, 262, 263, 264, 265,
            267, 268, 887, 888, 269, 270, 266, 259, 938, 280,
        ],
    ),
    (
        "minecraft:block",
        "wall_signs",
        &[224, 225, 226, 227, 229, 230, 231, 903, 904, 232, 233, 228],
    ),
    (
        "minecraft:block",
        "walls",
        &[
            409, 410, 824, 825, 826, 827, 828, 829, 831, 832, 833, 834, 835, 836, 926, 934, 940,
            1155, 1159, 1163, 1167, 830, 987, 991, 996, 379, 1015, 1019, 1023, 1002, 1006, 1010,
        ],
    ),
    ("minecraft:block", "warped_stems", &[862, 863, 864, 865]),
    ("minecraft:block", "wart_blocks", &[672, 868]),
    (
        "minecraft:block",
        "wither_immune",
        &[
            524, 34, 391, 392, 667, 407, 668, 669, 905, 906, 156, 525, 1182, 907, 908,
        ],
    ),
    ("minecraft:block", "wither_immune_to", &[170]),
    ("minecraft:block", "wither_skeleton_immune_to", &[170]),
    ("minecraft:block", "wither_summon_base_blocks", &[286, 287]),
    (
        "minecraft:block",
        "wolves_spawnable_on",
        &[8, 276, 278, 10, 11],
    ),
    (
        "minecraft:block",
        "wooden_buttons",
        &[443, 444, 445, 446, 447, 449, 450, 897, 898, 451, 452, 448],
    ),
    (
        "minecraft:block",
        "wooden_doors",
        &[220, 646, 647, 648, 649, 651, 652, 899, 900, 653, 654, 650],
    ),
    (
        "minecraft:block",
        "wooden_fences",
        &[284, 640, 642, 643, 637, 638, 639, 889, 890, 644, 645, 641],
    ),
    (
        "minecraft:block",
        "wooden_pressure_plates",
        &[261, 262, 263, 264, 265, 267, 268, 887, 888, 269, 270, 266],
    ),
    (
        "minecraft:block",
        "wooden_shelves",
        &[180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191],
    ),
    (
        "minecraft:block",
        "wooden_slabs",
        &[599, 600, 601, 602, 603, 605, 606, 885, 886, 607, 608, 604],
    ),
    (
        "minecraft:block",
        "wooden_stairs",
        &[200, 404, 405, 406, 516, 518, 519, 895, 896, 520, 521, 517],
    ),
    (
        "minecraft:block",
        "wooden_trapdoors",
        &[320, 318, 322, 323, 319, 316, 317, 891, 892, 324, 325, 321],
    ),
    (
        "minecraft:block",
        "wool",
        &[
            140, 141, 142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153, 154, 155,
        ],
    ),
    (
        "minecraft:block",
        "wool_carpets",
        &[
            538, 539, 540, 541, 542, 543, 544, 545, 546, 547, 548, 549, 550, 551, 552, 553,
        ],
    ),
    (
        "minecraft:damage_type",
        "always_hurts_ender_dragons",
        &[15, 9, 35, 1],
    ),
    (
        "minecraft:damage_type",
        "always_kills_armor_stands",
        &[0, 46, 14, 50, 48],
    ),
    (
        "minecraft:damage_type",
        "always_most_significant_fall",
        &[32],
    ),
    ("minecraft:damage_type", "always_triggers_silverfish", &[27]),
    (
        "minecraft:damage_type",
        "avoids_guardian_thorns",
        &[27, 44, 15, 9, 35, 1],
    ),
    ("minecraft:damage_type", "burn_from_stepping", &[3, 20, 42]),
    ("minecraft:damage_type", "burns_armor_stands", &[31]),
    (
        "minecraft:damage_type",
        "bypasses_armor",
        &[
            31, 22, 4, 6, 16, 18, 49, 5, 40, 10, 8, 17, 39, 27, 23, 32, 19, 36, 33,
        ],
    ),
    ("minecraft:damage_type", "bypasses_effects", &[40]),
    ("minecraft:damage_type", "bypasses_enchantments", &[36]),
    (
        "minecraft:damage_type",
        "bypasses_invulnerability",
        &[32, 19],
    ),
    ("minecraft:damage_type", "bypasses_resistance", &[32, 19]),
    (
        "minecraft:damage_type",
        "bypasses_shield",
        &[
            31, 22, 4, 6, 16, 18, 49, 5, 40, 10, 8, 17, 39, 27, 23, 32, 19, 36, 33, 2, 3, 7, 11,
            13, 20, 42, 21, 24, 25, 43,
        ],
    ),
    (
        "minecraft:damage_type",
        "bypasses_wolf_armor",
        &[32, 19, 4, 6, 7, 17, 22, 23, 27, 33, 40, 44, 49],
    ),
    (
        "minecraft:damage_type",
        "can_break_armor_stand",
        &[35, 34, 37, 26],
    ),
    ("minecraft:damage_type", "damages_helmet", &[11, 12, 13]),
    ("minecraft:damage_type", "ignites_armor_stands", &[21, 3]),
    ("minecraft:damage_type", "is_drowning", &[6]),
    ("minecraft:damage_type", "is_explosion", &[15, 9, 35, 1]),
    ("minecraft:damage_type", "is_fall", &[10, 8, 39]),
    (
        "minecraft:damage_type",
        "is_fire",
        &[21, 3, 31, 24, 20, 42, 47, 14],
    ),
    ("minecraft:damage_type", "is_freezing", &[17]),
    ("minecraft:damage_type", "is_lightning", &[25]),
    ("minecraft:damage_type", "is_player_attack", &[34, 37, 26]),
    (
        "minecraft:damage_type",
        "is_projectile",
        &[0, 46, 30, 47, 14, 50, 45, 48],
    ),
    ("minecraft:damage_type", "mace_smash", &[26]),
    ("minecraft:damage_type", "no_anger", &[29]),
    ("minecraft:damage_type", "no_impact", &[6]),
    (
        "minecraft:damage_type",
        "no_knockback",
        &[
            9, 35, 1, 21, 25, 31, 24, 20, 42, 22, 4, 6, 40, 2, 10, 8, 16, 32, 18, 27, 49, 5, 7, 43,
            17, 39, 33, 19, 3, 37,
        ],
    ),
    (
        "minecraft:damage_type",
        "panic_causes",
        &[
            2, 17, 20, 42, 21, 24, 25, 31, 0, 5, 9, 14, 15, 23, 27, 28, 30, 35, 36, 41, 45, 46, 47,
            48, 49, 50, 34, 37, 26,
        ],
    ),
    (
        "minecraft:damage_type",
        "panic_environmental_causes",
        &[2, 17, 20, 42, 21, 24, 25, 31],
    ),
    (
        "minecraft:damage_type",
        "sulfur_cube_with_block_immune_to",
        &[
            0, 2, 7, 10, 11, 12, 13, 17, 26, 20, 28, 29, 30, 34, 37, 38, 39, 41, 42, 43, 45, 46,
            48, 15, 9, 35, 1,
        ],
    ),
    (
        "minecraft:damage_type",
        "witch_resistant_to",
        &[27, 23, 36, 44],
    ),
    ("minecraft:damage_type", "wither_immune_to", &[6]),
    // Vanilla 26.2 datapack tags (empty). The client dialogs custom_options /
    // quick_actions reference these; omitting them crashes Registry Loading.
    ("minecraft:dialog", "pause_screen_additions", &[]),
    ("minecraft:dialog", "quick_actions", &[]),
    ("minecraft:enchantment", "curse", &[2, 41]),
    (
        "minecraft:enchantment",
        "double_trade_price",
        &[2, 41, 38, 36, 14, 23, 42],
    ),
    (
        "minecraft:enchantment",
        "exclusive_set/armor",
        &[28, 3, 11, 27],
    ),
    ("minecraft:enchantment", "exclusive_set/boots", &[14, 7]),
    ("minecraft:enchantment", "exclusive_set/bow", &[16, 23]),
    ("minecraft:enchantment", "exclusive_set/crossbow", &[24, 25]),
    (
        "minecraft:enchantment",
        "exclusive_set/damage",
        &[33, 35, 1, 15, 6, 4],
    ),
    ("minecraft:enchantment", "exclusive_set/mining", &[13, 34]),
    ("minecraft:enchantment", "exclusive_set/riptide", &[19, 5]),
    (
        "minecraft:enchantment",
        "in_enchanting_table",
        &[
            28, 11, 9, 3, 27, 31, 0, 39, 7, 33, 35, 1, 17, 10, 18, 37, 8, 34, 40, 13, 26, 29, 12,
            16, 20, 22, 19, 15, 32, 5, 24, 30, 25, 6, 4, 21,
        ],
    ),
    (
        "minecraft:enchantment",
        "non_treasure",
        &[
            28, 11, 9, 3, 27, 31, 0, 39, 7, 33, 35, 1, 17, 10, 18, 37, 8, 34, 40, 13, 26, 29, 12,
            16, 20, 22, 19, 15, 32, 5, 24, 30, 25, 6, 4, 21,
        ],
    ),
    (
        "minecraft:enchantment",
        "on_mob_spawn_equipment",
        &[
            28, 11, 9, 3, 27, 31, 0, 39, 7, 33, 35, 1, 17, 10, 18, 37, 8, 34, 40, 13, 26, 29, 12,
            16, 20, 22, 19, 15, 32, 5, 24, 30, 25, 6, 4, 21,
        ],
    ),
    (
        "minecraft:enchantment",
        "on_random_loot",
        &[
            28, 11, 9, 3, 27, 31, 0, 39, 7, 33, 35, 1, 17, 10, 18, 37, 8, 34, 40, 13, 26, 29, 12,
            16, 20, 22, 19, 15, 32, 5, 24, 30, 25, 6, 4, 21, 2, 41, 14, 23,
        ],
    ),
    (
        "minecraft:enchantment",
        "on_traded_equipment",
        &[
            28, 11, 9, 3, 27, 31, 0, 39, 7, 33, 35, 1, 17, 10, 18, 37, 8, 34, 40, 13, 26, 29, 12,
            16, 20, 22, 19, 15, 32, 5, 24, 30, 25, 6, 4, 21,
        ],
    ),
    (
        "minecraft:enchantment",
        "prevents_bee_spawns_when_mining",
        &[34],
    ),
    (
        "minecraft:enchantment",
        "prevents_decorated_pot_shattering",
        &[34],
    ),
    ("minecraft:enchantment", "prevents_ice_melting", &[34]),
    ("minecraft:enchantment", "prevents_infested_spawns", &[34]),
    ("minecraft:enchantment", "smelts_loot", &[10]),
    (
        "minecraft:enchantment",
        "tooltip_order",
        &[
            2, 41, 32, 5, 42, 14, 21, 33, 35, 1, 15, 26, 6, 4, 25, 37, 24, 10, 12, 17, 29, 28, 3,
            11, 27, 9, 13, 18, 34, 20, 8, 30, 22, 31, 0, 36, 38, 7, 39, 19, 40, 16, 23,
        ],
    ),
    (
        "minecraft:enchantment",
        "tradeable",
        &[
            28, 11, 9, 3, 27, 31, 0, 39, 7, 33, 35, 1, 17, 10, 18, 37, 8, 34, 40, 13, 26, 29, 12,
            16, 20, 22, 19, 15, 32, 5, 24, 30, 25, 6, 4, 21, 2, 41, 14, 23,
        ],
    ),
    (
        "minecraft:enchantment",
        "treasure",
        &[2, 41, 38, 36, 14, 23, 42],
    ),
    ("minecraft:entity_type", "accepts_iron_golem_gift", &[28]),
    (
        "minecraft:entity_type",
        "aquatic",
        &[138, 7, 63, 40, 27, 107, 110, 137, 35, 127, 61, 131, 88, 153],
    ),
    ("minecraft:entity_type", "arrows", &[6, 123]),
    (
        "minecraft:entity_type",
        "arthropod",
        &[11, 42, 114, 124, 22],
    ),
    (
        "minecraft:entity_type",
        "axolotl_always_hostiles",
        &[38, 63, 40],
    ),
    (
        "minecraft:entity_type",
        "axolotl_hunt_targets",
        &[137, 107, 110, 27, 127, 61, 131],
    ),
    ("minecraft:entity_type", "beehive_inhabitors", &[11]),
    (
        "minecraft:entity_type",
        "boat",
        &[89, 125, 12, 74, 0, 23, 33, 94, 81, 9],
    ),
    (
        "minecraft:entity_type",
        "burn_in_daylight",
        &[115, 128, 147, 16, 151, 152, 154, 38, 153, 99],
    ),
    (
        "minecraft:entity_type",
        "can_breathe_under_water",
        &[
            115, 128, 147, 116, 16, 97, 152, 20, 151, 154, 155, 150, 38, 67, 153, 146, 99, 7, 55,
            63, 40, 138, 61, 27, 107, 110, 127, 137, 131, 5, 28, 88,
        ],
    ),
    ("minecraft:entity_type", "can_equip_harness", &[58]),
    (
        "minecraft:entity_type",
        "can_equip_saddle",
        &[66, 116, 152, 36, 87, 100, 129, 19, 20, 88, 153],
    ),
    (
        "minecraft:entity_type",
        "can_float_while_ridden",
        &[66, 152, 87, 36, 19, 20],
    ),
    ("minecraft:entity_type", "can_turn_in_boats", &[17]),
    ("minecraft:entity_type", "can_wear_horse_armor", &[66, 152]),
    (
        "minecraft:entity_type",
        "can_wear_nautilus_armor",
        &[88, 153],
    ),
    (
        "minecraft:entity_type",
        "candidate_for_iron_golem_gift",
        &[140, 28],
    ),
    (
        "minecraft:entity_type",
        "cannot_be_age_locked",
        &[152, 116, 140],
    ),
    (
        "minecraft:entity_type",
        "cannot_be_pushed_onto_boats",
        &[
            156, 40, 27, 107, 110, 137, 35, 127, 61, 131, 31, 88, 153, 130,
        ],
    ),
    ("minecraft:entity_type", "deflects_projectiles", &[17]),
    (
        "minecraft:entity_type",
        "dismounts_underwater",
        &[19, 26, 36, 58, 66, 78, 87, 100, 109, 124, 129, 135, 152],
    ),
    (
        "minecraft:entity_type",
        "fall_damage_immune",
        &[
            28, 70, 121, 112, 2, 10, 11, 14, 21, 26, 57, 58, 99, 80, 91, 98, 146, 17,
        ],
    ),
    (
        "minecraft:entity_type",
        "followable_friendly_mobs",
        &[
            4, 11, 19, 21, 26, 30, 36, 54, 62, 58, 66, 116, 78, 87, 91, 96, 98, 100, 104, 108, 111,
            119, 129, 140, 149,
        ],
    ),
    (
        "minecraft:entity_type",
        "freeze_hurts_extra_types",
        &[129, 14, 80],
    ),
    (
        "minecraft:entity_type",
        "freeze_immune_entity_types",
        &[128, 104, 121, 146],
    ),
    ("minecraft:entity_type", "frog_food", &[117, 80]),
    (
        "minecraft:entity_type",
        "ignores_poison_and_regen",
        &[
            115, 128, 147, 116, 16, 97, 152, 20, 151, 154, 155, 150, 38, 67, 153, 146, 99,
        ],
    ),
    ("minecraft:entity_type", "illager", &[46, 68, 103, 141]),
    (
        "minecraft:entity_type",
        "illager_friends",
        &[46, 68, 103, 141],
    ),
    ("minecraft:entity_type", "immune_to_infested", &[114]),
    ("minecraft:entity_type", "immune_to_oozing", &[117]),
    (
        "minecraft:entity_type",
        "impact_projectiles",
        &[6, 123, 53, 120, 52, 118, 39, 136, 37, 148, 144, 18],
    ),
    (
        "minecraft:entity_type",
        "inverted_healing_and_harm",
        &[
            115, 128, 147, 116, 16, 97, 152, 20, 151, 154, 155, 150, 38, 67, 153, 146, 99,
        ],
    ),
    ("minecraft:entity_type", "nautilus_hostiles", &[107]),
    (
        "minecraft:entity_type",
        "no_anger_from_wind_charge",
        &[17, 115, 16, 128, 151, 67, 124, 22, 117],
    ),
    (
        "minecraft:entity_type",
        "non_controlling_rider",
        &[117, 80, 130],
    ),
    ("minecraft:entity_type", "not_affected_by_geysers", &[43]),
    (
        "minecraft:entity_type",
        "not_scary_for_pufferfish",
        &[
            138, 63, 40, 27, 107, 110, 137, 35, 127, 61, 131, 88, 153, 130,
        ],
    ),
    (
        "minecraft:entity_type",
        "powder_snow_walkable_mobs",
        &[108, 42, 114, 54],
    ),
    (
        "minecraft:entity_type",
        "raiders",
        &[46, 103, 109, 141, 68, 145],
    ),
    (
        "minecraft:entity_type",
        "redirectable_projectile",
        &[52, 144, 18],
    ),
    (
        "minecraft:entity_type",
        "sensitive_to_bane_of_arthropods",
        &[11, 42, 114, 124, 22],
    ),
    (
        "minecraft:entity_type",
        "sensitive_to_impaling",
        &[138, 7, 63, 40, 27, 107, 110, 137, 35, 127, 61, 131, 88, 153],
    ),
    (
        "minecraft:entity_type",
        "sensitive_to_smite",
        &[
            115, 128, 147, 116, 16, 97, 152, 20, 151, 154, 155, 150, 38, 67, 153, 146, 99,
        ],
    ),
    (
        "minecraft:entity_type",
        "skeletons",
        &[115, 128, 147, 116, 16, 97],
    ),
    (
        "minecraft:entity_type",
        "undead",
        &[
            115, 128, 147, 116, 16, 97, 152, 20, 151, 154, 155, 150, 38, 67, 153, 146, 99,
        ],
    ),
    (
        "minecraft:entity_type",
        "wither_friends",
        &[
            115, 128, 147, 116, 16, 97, 152, 20, 151, 154, 155, 150, 38, 67, 153, 146, 99,
        ],
    ),
    (
        "minecraft:entity_type",
        "zombies",
        &[152, 20, 151, 154, 155, 150, 38, 67, 153],
    ),
    ("minecraft:fluid", "bubble_column_can_occupy", &[2]),
    ("minecraft:fluid", "lava", &[4, 3]),
    ("minecraft:fluid", "supports_frogspawn", &[2]),
    ("minecraft:fluid", "supports_lily_pad", &[2]),
    ("minecraft:fluid", "supports_sugar_cane_adjacently", &[2, 1]),
    ("minecraft:fluid", "water", &[2, 1]),
    ("minecraft:game_event", "allay_can_listen", &[34]),
    (
        "minecraft:game_event",
        "ignore_vibrations_sneaking",
        &[27, 37, 42, 43, 30, 29],
    ),
    ("minecraft:game_event", "shrieker_can_listen", &[38]),
    (
        "minecraft:game_event",
        "vibrations",
        &[
            1, 2, 3, 5, 6, 7, 8, 0, 4, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            25, 26, 27, 28, 29, 33, 34, 35, 36, 37, 39, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
            52, 53, 54, 55, 56, 57, 58, 59, 60, 24,
        ],
    ),
    (
        "minecraft:game_event",
        "warden_can_listen",
        &[
            1, 2, 3, 5, 6, 7, 8, 0, 4, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            25, 26, 27, 28, 29, 33, 34, 35, 36, 37, 39, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51,
            52, 53, 54, 55, 56, 57, 58, 59, 60, 40, 38,
        ],
    ),
    (
        "minecraft:instrument",
        "goat_horns",
        &[4, 6, 5, 3, 0, 1, 7, 2],
    ),
    ("minecraft:instrument", "regular_goat_horns", &[4, 6, 5, 3]),
    (
        "minecraft:instrument",
        "screaming_goat_horns",
        &[0, 1, 7, 2],
    ),
    ("minecraft:item", "acacia_logs", &[165, 202, 179, 190]),
    ("minecraft:item", "anvil", &[506, 507, 508]),
    ("minecraft:item", "armadillo_food", &[1151]),
    ("minecraft:item", "arrows", &[923, 1323, 1322]),
    (
        "minecraft:item",
        "axes",
        &[967, 952, 957, 972, 942, 962, 947],
    ),
    ("minecraft:item", "axolotl_food", &[1050]),
    ("minecraft:item", "bamboo_blocks", &[174, 197]),
    (
        "minecraft:item",
        "banners",
        &[
            1296, 1297, 1298, 1299, 1300, 1301, 1302, 1303, 1304, 1305, 1306, 1307, 1308, 1309,
            1310, 1311,
        ],
    ),
    (
        "minecraft:item",
        "bars",
        &[418, 419, 420, 421, 422, 423, 424, 425, 426],
    ),
    (
        "minecraft:item",
        "beacon_payment_items",
        &[937, 927, 926, 936, 932],
    ),
    (
        "minecraft:item",
        "beds",
        &[
            1115, 1116, 1117, 1118, 1119, 1120, 1121, 1122, 1123, 1124, 1125, 1126, 1127, 1128,
            1129, 1130,
        ],
    ),
    (
        "minecraft:item",
        "bee_food",
        &[
            256, 258, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269, 270, 271, 272, 552, 553,
            555, 554, 273, 219, 233, 84, 214, 286, 287, 353, 274, 369,
        ],
    ),
    ("minecraft:item", "birch_logs", &[163, 200, 177, 188]),
    (
        "minecraft:item",
        "boats",
        &[
            891, 893, 895, 897, 899, 903, 905, 907, 909, 901, 892, 894, 896, 898, 900, 904, 906,
            908, 910, 902,
        ],
    ),
    ("minecraft:item", "book_cloning_target", &[1250]),
    (
        "minecraft:item",
        "bookshelf_books",
        &[1058, 1251, 1274, 1250, 1337],
    ),
    (
        "minecraft:item",
        "breaks_decorated_pots",
        &[
            964, 949, 954, 969, 939, 959, 944, 967, 952, 957, 972, 942, 962, 947, 966, 951, 956,
            971, 941, 961, 946, 965, 950, 955, 970, 940, 960, 945, 968, 953, 958, 973, 943, 963,
            948, 1362, 1253,
        ],
    ),
    ("minecraft:item", "brewing_fuel", &[1153]),
    (
        "minecraft:item",
        "bundles",
        &[
            1065, 1066, 1067, 1068, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078,
            1079, 1080, 1081,
        ],
    ),
    (
        "minecraft:item",
        "buttons",
        &[
            779, 780, 781, 782, 783, 785, 786, 789, 790, 787, 788, 784, 777, 778,
        ],
    ),
    ("minecraft:item", "camel_food", &[368]),
    ("minecraft:item", "camel_husk_food", &[1282]),
    (
        "minecraft:item",
        "candles",
        &[
            1429, 1430, 1431, 1432, 1433, 1434, 1435, 1436, 1437, 1438, 1439, 1440, 1441, 1442,
            1443, 1444, 1445,
        ],
    ),
    (
        "minecraft:item",
        "cat_collar_dyes",
        &[
            1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108,
            1109, 1110,
        ],
    ),
    ("minecraft:item", "cat_food", &[1086, 1087]),
    (
        "minecraft:item",
        "cauldron_can_remove_dye",
        &[982, 983, 984, 985, 1290, 918],
    ),
    (
        "minecraft:item",
        "chains",
        &[427, 428, 429, 430, 431, 432, 433, 434, 435],
    ),
    ("minecraft:item", "cherry_logs", &[166, 203, 180, 191]),
    (
        "minecraft:item",
        "chest_armor",
        &[983, 987, 991, 1003, 995, 999, 1007],
    ),
    (
        "minecraft:item",
        "chest_boats",
        &[892, 894, 896, 898, 900, 904, 906, 908, 910, 902],
    ),
    (
        "minecraft:item",
        "chicken_food",
        &[979, 1138, 1137, 1318, 1315, 1316],
    ),
    (
        "minecraft:item",
        "cluster_max_harvestables",
        &[966, 956, 961, 971, 951, 941, 946],
    ),
    ("minecraft:item", "coal_ores", &[91, 92]),
    ("minecraft:item", "coals", &[924, 925]),
    ("minecraft:item", "compasses", &[1063, 1064]),
    (
        "minecraft:item",
        "completes_find_tree_tutorial",
        &[
            168, 205, 181, 192, 167, 204, 182, 193, 161, 198, 175, 186, 165, 202, 179, 190, 163,
            200, 177, 188, 164, 201, 178, 189, 162, 199, 176, 187, 169, 206, 183, 194, 166, 203,
            180, 191, 172, 184, 207, 195, 173, 185, 208, 196, 212, 209, 210, 216, 215, 213, 211,
            218, 219, 217, 214, 604, 605,
        ],
    ),
    (
        "minecraft:item",
        "concrete",
        &[
            642, 643, 644, 645, 646, 647, 648, 649, 650, 651, 652, 653, 654, 655, 656, 657,
        ],
    ),
    (
        "minecraft:item",
        "concrete_powders",
        &[
            658, 659, 660, 661, 662, 663, 664, 665, 666, 667, 668, 669, 670, 671, 672, 673,
        ],
    ),
    (
        "minecraft:item",
        "copper",
        &[118, 119, 120, 121, 122, 123, 124, 125],
    ),
    (
        "minecraft:item",
        "copper_chests",
        &[1516, 1517, 1518, 1519, 1520, 1521, 1522, 1523],
    ),
    (
        "minecraft:item",
        "copper_golem_statues",
        &[1524, 1525, 1526, 1527, 1528, 1529, 1530, 1531],
    ),
    ("minecraft:item", "copper_ores", &[95, 96]),
    ("minecraft:item", "copper_tool_materials", &[934]),
    ("minecraft:item", "cow_food", &[980]),
    (
        "minecraft:item",
        "creeper_drop_music_discs",
        &[
            1339, 1340, 1341, 1343, 1346, 1348, 1349, 1350, 1351, 1352, 1353, 1354,
        ],
    ),
    ("minecraft:item", "creeper_igniters", &[919, 1248]),
    ("minecraft:item", "crimson_stems", &[172, 184, 207, 195]),
    (
        "minecraft:item",
        "dampens_vibrations",
        &[
            240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255, 533,
            534, 535, 536, 537, 538, 539, 540, 541, 542, 543, 544, 545, 546, 547, 548,
        ],
    ),
    ("minecraft:item", "dark_oak_logs", &[168, 205, 181, 192]),
    (
        "minecraft:item",
        "decorated_pot_ingredients",
        &[
            1054, 1477, 1478, 1479, 1480, 1481, 1482, 1483, 1484, 1486, 1488, 1489, 1490, 1491,
            1492, 1493, 1494, 1496, 1497, 1498, 1499, 1485, 1487, 1495,
        ],
    ),
    (
        "minecraft:item",
        "decorated_pot_sherds",
        &[
            1477, 1478, 1479, 1480, 1481, 1482, 1483, 1484, 1486, 1488, 1489, 1490, 1491, 1492,
            1493, 1494, 1496, 1497, 1498, 1499, 1485, 1487, 1495,
        ],
    ),
    ("minecraft:item", "diamond_ores", &[105, 106]),
    ("minecraft:item", "diamond_tool_materials", &[926]),
    ("minecraft:item", "dirt", &[55, 56, 58]),
    (
        "minecraft:item",
        "doors",
        &[
            808, 809, 810, 811, 812, 814, 815, 818, 819, 816, 817, 813, 820, 821, 822, 823, 824,
            825, 826, 827, 807,
        ],
    ),
    ("minecraft:item", "drowned_preferred_weapons", &[1362]),
    ("minecraft:item", "duplicates_allays", &[930]),
    (
        "minecraft:item",
        "dyes",
        &[
            1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108,
            1109, 1110,
        ],
    ),
    ("minecraft:item", "eggs", &[1060, 1061, 1062]),
    ("minecraft:item", "emerald_ores", &[101, 102]),
    (
        "minecraft:item",
        "enchantable/armor",
        &[
            985, 989, 993, 1005, 997, 1001, 1009, 984, 988, 992, 1004, 996, 1000, 1008, 983, 987,
            991, 1003, 995, 999, 1007, 982, 986, 990, 1002, 994, 998, 1006, 915,
        ],
    ),
    ("minecraft:item", "enchantable/bow", &[922]),
    (
        "minecraft:item",
        "enchantable/chest_armor",
        &[983, 987, 991, 1003, 995, 999, 1007],
    ),
    ("minecraft:item", "enchantable/crossbow", &[1370]),
    (
        "minecraft:item",
        "enchantable/durability",
        &[
            985, 989, 993, 1005, 997, 1001, 1009, 984, 988, 992, 1004, 996, 1000, 1008, 983, 987,
            991, 1003, 995, 999, 1007, 982, 986, 990, 1002, 994, 998, 1006, 915, 890, 1325, 964,
            949, 954, 969, 939, 959, 944, 967, 952, 957, 972, 942, 962, 947, 966, 951, 956, 971,
            941, 961, 946, 965, 950, 955, 970, 940, 960, 945, 968, 953, 958, 973, 943, 963, 948,
            922, 1370, 1362, 919, 1134, 1457, 1082, 887, 888, 1253, 1331, 1327, 1330, 1332, 1326,
            1329, 1328,
        ],
    ),
    (
        "minecraft:item",
        "enchantable/equippable",
        &[
            985, 989, 993, 1005, 997, 1001, 1009, 984, 988, 992, 1004, 996, 1000, 1008, 983, 987,
            991, 1003, 995, 999, 1007, 982, 986, 990, 1002, 994, 998, 1006, 915, 890, 1265, 1267,
            1266, 1263, 1264, 1268, 1269, 385,
        ],
    ),
    (
        "minecraft:item",
        "enchantable/fire_aspect",
        &[
            964, 949, 954, 969, 939, 959, 944, 1331, 1327, 1330, 1332, 1326, 1329, 1328, 1253,
        ],
    ),
    ("minecraft:item", "enchantable/fishing", &[1082]),
    (
        "minecraft:item",
        "enchantable/foot_armor",
        &[985, 989, 993, 1005, 997, 1001, 1009],
    ),
    (
        "minecraft:item",
        "enchantable/head_armor",
        &[982, 986, 990, 1002, 994, 998, 1006, 915],
    ),
    (
        "minecraft:item",
        "enchantable/leg_armor",
        &[984, 988, 992, 1004, 996, 1000, 1008],
    ),
    (
        "minecraft:item",
        "enchantable/lunge",
        &[1331, 1327, 1330, 1332, 1326, 1329, 1328],
    ),
    ("minecraft:item", "enchantable/mace", &[1253]),
    (
        "minecraft:item",
        "enchantable/melee_weapon",
        &[
            964, 949, 954, 969, 939, 959, 944, 1331, 1327, 1330, 1332, 1326, 1329, 1328,
        ],
    ),
    (
        "minecraft:item",
        "enchantable/mining",
        &[
            967, 952, 957, 972, 942, 962, 947, 966, 951, 956, 971, 941, 961, 946, 965, 950, 955,
            970, 940, 960, 945, 968, 953, 958, 973, 943, 963, 948, 1134,
        ],
    ),
    (
        "minecraft:item",
        "enchantable/mining_loot",
        &[
            967, 952, 957, 972, 942, 962, 947, 966, 951, 956, 971, 941, 961, 946, 965, 950, 955,
            970, 940, 960, 945, 968, 953, 958, 973, 943, 963, 948,
        ],
    ),
    (
        "minecraft:item",
        "enchantable/sharp_weapon",
        &[
            964, 949, 954, 969, 939, 959, 944, 1331, 1327, 1330, 1332, 1326, 1329, 1328, 967, 952,
            957, 972, 942, 962, 947,
        ],
    ),
    (
        "minecraft:item",
        "enchantable/sweeping",
        &[964, 949, 954, 969, 939, 959, 944],
    ),
    ("minecraft:item", "enchantable/trident", &[1362]),
    (
        "minecraft:item",
        "enchantable/vanishing",
        &[
            985, 989, 993, 1005, 997, 1001, 1009, 984, 988, 992, 1004, 996, 1000, 1008, 983, 987,
            991, 1003, 995, 999, 1007, 982, 986, 990, 1002, 994, 998, 1006, 915, 890, 1325, 964,
            949, 954, 969, 939, 959, 944, 967, 952, 957, 972, 942, 962, 947, 966, 951, 956, 971,
            941, 961, 946, 965, 950, 955, 970, 940, 960, 945, 968, 953, 958, 973, 943, 963, 948,
            922, 1370, 1362, 919, 1134, 1457, 1082, 887, 888, 1253, 1331, 1327, 1330, 1332, 1326,
            1329, 1328, 1063, 385, 1265, 1267, 1266, 1263, 1264, 1268, 1269,
        ],
    ),
    (
        "minecraft:item",
        "enchantable/weapon",
        &[
            964, 949, 954, 969, 939, 959, 944, 1331, 1327, 1330, 1332, 1326, 1329, 1328, 967, 952,
            957, 972, 942, 962, 947, 1253,
        ],
    ),
    (
        "minecraft:item",
        "fence_gates",
        &[853, 851, 855, 856, 852, 849, 850, 859, 860, 857, 858, 854],
    ),
    (
        "minecraft:item",
        "fences",
        &[
            372, 376, 378, 379, 373, 374, 375, 382, 383, 380, 381, 377, 455,
        ],
    ),
    (
        "minecraft:item",
        "fishes",
        &[1086, 1090, 1087, 1091, 1089, 1088],
    ),
    (
        "minecraft:item",
        "flowers",
        &[
            256, 258, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269, 270, 271, 272, 259, 257,
            552, 553, 555, 554, 273, 219, 233, 84, 214, 286, 287, 353, 274, 369,
        ],
    ),
    (
        "minecraft:item",
        "foot_armor",
        &[985, 989, 993, 1005, 997, 1001, 1009],
    ),
    ("minecraft:item", "fox_food", &[1404, 1405]),
    (
        "minecraft:item",
        "freeze_immune_wearables",
        &[985, 984, 983, 982, 1290],
    ),
    ("minecraft:item", "frog_food", &[1059]),
    ("minecraft:item", "furnace_minecart_fuel", &[924, 925]),
    ("minecraft:item", "gaze_disguise_equipment", &[385]),
    (
        "minecraft:item",
        "glazed_terracotta",
        &[
            626, 627, 628, 629, 630, 631, 632, 633, 634, 635, 636, 637, 638, 639, 640, 641,
        ],
    ),
    ("minecraft:item", "goat_food", &[980]),
    ("minecraft:item", "gold_ores", &[97, 107, 98]),
    ("minecraft:item", "gold_tool_materials", &[936]),
    ("minecraft:item", "grass_blocks", &[54, 57, 450]),
    (
        "minecraft:item",
        "hanging_signs",
        &[
            1028, 1029, 1030, 1032, 1033, 1031, 1034, 1035, 1038, 1039, 1036, 1037,
        ],
    ),
    ("minecraft:item", "happy_ghast_food", &[1044]),
    (
        "minecraft:item",
        "happy_ghast_tempt_items",
        &[
            1044, 866, 867, 868, 869, 870, 871, 872, 873, 874, 875, 876, 877, 878, 879, 880, 881,
        ],
    ),
    (
        "minecraft:item",
        "harnesses",
        &[
            866, 867, 868, 869, 870, 871, 872, 873, 874, 875, 876, 877, 878, 879, 880, 881,
        ],
    ),
    (
        "minecraft:item",
        "head_armor",
        &[982, 986, 990, 1002, 994, 998, 1006, 915],
    ),
    (
        "minecraft:item",
        "hoes",
        &[968, 953, 958, 973, 943, 963, 948],
    ),
    ("minecraft:item", "hoglin_food", &[277]),
    (
        "minecraft:item",
        "horse_food",
        &[980, 1113, 532, 921, 1257, 1262, 1014, 1015],
    ),
    ("minecraft:item", "horse_tempt_items", &[1262, 1014, 1015]),
    ("minecraft:item", "ignored_by_piglin_babies", &[1045]),
    ("minecraft:item", "iron_ores", &[93, 94]),
    ("minecraft:item", "iron_tool_materials", &[932]),
    ("minecraft:item", "jungle_logs", &[164, 201, 178, 189]),
    (
        "minecraft:item",
        "lanterns",
        &[1394, 1395, 1396, 1397, 1398, 1399, 1400, 1401, 1402, 1403],
    ),
    ("minecraft:item", "lapis_ores", &[103, 104]),
    (
        "minecraft:item",
        "leaves",
        &[212, 209, 210, 216, 215, 213, 211, 218, 219, 217, 214],
    ),
    ("minecraft:item", "lectern_books", &[1251, 1250]),
    (
        "minecraft:item",
        "leg_armor",
        &[984, 988, 992, 1004, 996, 1000, 1008],
    ),
    (
        "minecraft:item",
        "lightning_rods",
        &[761, 762, 763, 764, 765, 766, 767, 768],
    ),
    ("minecraft:item", "llama_food", &[980, 532]),
    ("minecraft:item", "llama_tempt_items", &[532]),
    (
        "minecraft:item",
        "logs",
        &[
            168, 205, 181, 192, 167, 204, 182, 193, 161, 198, 175, 186, 165, 202, 179, 190, 163,
            200, 177, 188, 164, 201, 178, 189, 162, 199, 176, 187, 169, 206, 183, 194, 166, 203,
            180, 191, 172, 184, 207, 195, 173, 185, 208, 196,
        ],
    ),
    (
        "minecraft:item",
        "logs_that_burn",
        &[
            168, 205, 181, 192, 167, 204, 182, 193, 161, 198, 175, 186, 165, 202, 179, 190, 163,
            200, 177, 188, 164, 201, 178, 189, 162, 199, 176, 187, 169, 206, 183, 194, 166, 203,
            180, 191,
        ],
    ),
    (
        "minecraft:item",
        "loom_dyes",
        &[
            1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108,
            1109, 1110,
        ],
    ),
    (
        "minecraft:item",
        "loom_patterns",
        &[1373, 1374, 1375, 1376, 1377, 1378, 1379, 1380, 1381, 1382],
    ),
    ("minecraft:item", "mangrove_logs", &[169, 206, 183, 194]),
    ("minecraft:item", "map_invisibility_equipment", &[385]),
    (
        "minecraft:item",
        "meat",
        &[
            1139, 1141, 1140, 1142, 1295, 1012, 1280, 1294, 1011, 1279, 1143,
        ],
    ),
    ("minecraft:item", "metal_nuggets", &[1336, 1335, 1147]),
    ("minecraft:item", "moss_blocks", &[290, 293]),
    ("minecraft:item", "mud", &[59, 171]),
    (
        "minecraft:item",
        "nautilus_bucket_food",
        &[1047, 1049, 1048, 1050],
    ),
    (
        "minecraft:item",
        "nautilus_food",
        &[1086, 1090, 1087, 1091, 1089, 1088, 1047, 1049, 1048, 1050],
    ),
    ("minecraft:item", "nautilus_taming_items", &[1047, 1089]),
    ("minecraft:item", "netherite_tool_materials", &[937]),
    (
        "minecraft:item",
        "non_flammable_wood",
        &[
            173, 185, 208, 196, 172, 184, 207, 195, 73, 74, 309, 310, 805, 806, 382, 383, 839, 840,
            859, 860, 480, 481, 789, 790, 818, 819, 1026, 1027, 1039, 1038, 344, 337,
        ],
    ),
    (
        "minecraft:item",
        "noteblock_top_instruments",
        &[1266, 1263, 1267, 1268, 1264, 1269, 1265],
    ),
    ("minecraft:item", "oak_logs", &[161, 198, 175, 186]),
    ("minecraft:item", "ocelot_food", &[1086, 1087]),
    ("minecraft:item", "pale_oak_logs", &[167, 204, 182, 193]),
    ("minecraft:item", "panda_eats_from_ground", &[297, 1114]),
    ("minecraft:item", "panda_food", &[297]),
    (
        "minecraft:item",
        "parrot_food",
        &[979, 1138, 1137, 1318, 1315, 1316],
    ),
    ("minecraft:item", "parrot_poisonous_food", &[1131]),
    (
        "minecraft:item",
        "pickaxes",
        &[966, 951, 956, 971, 941, 961, 946],
    ),
    ("minecraft:item", "pig_food", &[1257, 1258, 1317]),
    ("minecraft:item", "piglin_food", &[1011, 1012]),
    (
        "minecraft:item",
        "piglin_loved",
        &[
            97, 107, 98, 126, 1419, 793, 936, 1393, 1083, 1262, 1158, 1014, 1015, 1002, 1003, 1004,
            1005, 1287, 1365, 954, 1330, 956, 955, 957, 958, 935, 113, 257,
        ],
    ),
    ("minecraft:item", "piglin_preferred_weapons", &[1370, 1330]),
    ("minecraft:item", "piglin_repellents", &[393, 1395, 1407]),
    (
        "minecraft:item",
        "piglin_safe_armor",
        &[1002, 1003, 1004, 1005],
    ),
    ("minecraft:item", "pillager_preferred_weapons", &[1370]),
    (
        "minecraft:item",
        "planks",
        &[63, 64, 65, 66, 67, 69, 70, 73, 74, 71, 72, 68],
    ),
    ("minecraft:item", "rabbit_food", &[1257, 1262, 256]),
    ("minecraft:item", "rails", &[863, 861, 862, 864]),
    ("minecraft:item", "redstone_ores", &[99, 100]),
    ("minecraft:item", "repairs_chain_armor", &[932]),
    ("minecraft:item", "repairs_copper_armor", &[934]),
    ("minecraft:item", "repairs_diamond_armor", &[926]),
    ("minecraft:item", "repairs_gold_armor", &[936]),
    ("minecraft:item", "repairs_iron_armor", &[932]),
    ("minecraft:item", "repairs_leather_armor", &[1045]),
    ("minecraft:item", "repairs_netherite_armor", &[937]),
    ("minecraft:item", "repairs_turtle_helmet", &[916]),
    ("minecraft:item", "repairs_wolf_armor", &[917]),
    ("minecraft:item", "sand", &[86, 89, 87]),
    (
        "minecraft:item",
        "saplings",
        &[76, 77, 78, 79, 80, 82, 83, 232, 233, 84, 81],
    ),
    ("minecraft:item", "shearable_from_copper_golem", &[260]),
    ("minecraft:item", "sheep_food", &[980]),
    (
        "minecraft:item",
        "shovels",
        &[965, 950, 955, 970, 940, 960, 945],
    ),
    (
        "minecraft:item",
        "shulker_boxes",
        &[
            609, 610, 611, 612, 613, 614, 615, 616, 617, 618, 619, 620, 621, 622, 623, 624, 625,
        ],
    ),
    (
        "minecraft:item",
        "signs",
        &[
            1016, 1017, 1018, 1020, 1019, 1022, 1023, 1026, 1027, 1024, 1025, 1021,
        ],
    ),
    ("minecraft:item", "skeleton_preferred_weapons", &[922]),
    (
        "minecraft:item",
        "skulls",
        &[1265, 1267, 1266, 1263, 1264, 1268, 1269],
    ),
    (
        "minecraft:item",
        "slabs",
        &[
            298, 299, 300, 301, 302, 304, 305, 309, 310, 306, 307, 303, 308, 311, 312, 318, 313,
            324, 321, 322, 317, 316, 320, 315, 325, 326, 327, 727, 728, 729, 730, 731, 732, 733,
            734, 735, 736, 737, 738, 739, 314, 323, 1417, 1425, 1421, 740, 741, 743, 742, 319, 13,
            18, 22, 444, 41, 45, 49, 28, 32, 36, 153, 154, 155, 156, 157, 158, 159, 160,
        ],
    ),
    (
        "minecraft:item",
        "small_flowers",
        &[
            256, 258, 260, 261, 262, 263, 264, 265, 266, 267, 268, 269, 270, 271, 272, 259, 257,
        ],
    ),
    ("minecraft:item", "smelts_to_glass", &[86, 89]),
    ("minecraft:item", "sniffer_food", &[1315]),
    ("minecraft:item", "soul_fire_base_blocks", &[388, 389]),
    (
        "minecraft:item",
        "spears",
        &[1331, 1327, 1330, 1332, 1326, 1329, 1328],
    ),
    ("minecraft:item", "spruce_logs", &[162, 199, 176, 187]),
    (
        "minecraft:item",
        "stairs",
        &[
            469, 470, 471, 472, 473, 475, 476, 480, 481, 477, 478, 474, 479, 364, 466, 456, 448,
            447, 356, 513, 600, 594, 593, 595, 709, 710, 711, 712, 713, 714, 715, 716, 717, 718,
            719, 720, 721, 722, 1418, 1426, 1422, 723, 724, 726, 725, 449, 14, 19, 23, 443, 42, 46,
            50, 29, 33, 37, 145, 146, 147, 148, 149, 150, 151, 152,
        ],
    ),
    ("minecraft:item", "stone_bricks", &[403, 404, 405, 406]),
    ("minecraft:item", "stone_buttons", &[777, 778]),
    ("minecraft:item", "stone_crafting_materials", &[62, 1416, 9]),
    ("minecraft:item", "stone_tool_materials", &[62, 1416, 9]),
    ("minecraft:item", "strider_food", &[278]),
    ("minecraft:item", "strider_tempt_items", &[278, 888]),
    (
        "minecraft:item",
        "sulfur_cube_archetype/bouncy",
        &[
            63, 64, 65, 66, 67, 69, 70, 73, 74, 71, 72, 68, 75, 168, 205, 181, 192, 167, 204, 182,
            193, 161, 198, 175, 186, 165, 202, 179, 190, 163, 200, 177, 188, 164, 201, 178, 189,
            162, 199, 176, 187, 169, 206, 183, 194, 166, 203, 180, 191, 172, 184, 207, 195, 173,
            185, 208, 196, 174, 197,
        ],
    ),
    ("minecraft:item", "sulfur_cube_archetype/explosive", &[774]),
    (
        "minecraft:item",
        "sulfur_cube_archetype/fast_flat",
        &[
            682, 683, 684, 685, 686, 677, 678, 679, 680, 681, 220, 221, 1056, 290, 293, 441, 442,
            446, 437, 532, 384, 385, 386, 1452, 1454, 1453,
        ],
    ),
    (
        "minecraft:item",
        "sulfur_cube_archetype/fast_sliding",
        &[707, 550, 367],
    ),
    (
        "minecraft:item",
        "sulfur_cube_archetype/high_resistance",
        &[388, 389],
    ),
    ("minecraft:item", "sulfur_cube_archetype/hot", &[603]),
    (
        "minecraft:item",
        "sulfur_cube_archetype/light",
        &[
            240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
        ],
    ),
    (
        "minecraft:item",
        "sulfur_cube_archetype/regular",
        &[
            658, 659, 660, 661, 662, 663, 664, 665, 666, 667, 668, 669, 670, 671, 672, 673, 59,
            171, 407, 110, 55, 56, 58, 57, 54, 370, 607,
        ],
    ),
    (
        "minecraft:item",
        "sulfur_cube_archetype/slow_bouncy",
        &[
            115, 6, 390, 1416, 332, 11, 52, 413, 454, 1423, 509, 598, 226, 406, 39, 16, 25, 40, 48,
            9, 62, 410, 412, 453, 1427, 405, 60, 1415, 599, 227, 592, 8, 409, 411, 127, 4, 53, 468,
            463, 464, 1419, 395, 2, 224, 348, 404, 408, 452, 387, 754, 349, 7, 391, 1420, 1424, 44,
            10, 5, 3, 31, 17, 590, 591, 354, 355, 510, 511, 108, 512, 606, 597, 775, 225, 596, 392,
            328, 329, 330, 331, 1, 403, 26, 35, 12, 21, 61, 642, 643, 644, 645, 646, 647, 648, 649,
            650, 651, 652, 653, 654, 655, 656, 657, 91, 92, 103, 104, 99, 100, 105, 106, 101, 102,
            549, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529,
            626, 627, 628, 629, 630, 631, 632, 633, 634, 635, 636, 637, 638, 639, 640, 641,
        ],
    ),
    (
        "minecraft:item",
        "sulfur_cube_archetype/slow_flat",
        &[
            117, 126, 112, 113, 111, 97, 107, 98, 93, 94, 95, 96, 128, 109, 118, 119, 120, 121,
            122, 123, 124, 125, 1508, 1509, 1510, 1511, 1512, 1513, 1514, 1515, 137, 138, 139, 140,
            141, 142, 143, 144, 129, 130, 131, 132, 133, 134, 135, 136,
        ],
    ),
    (
        "minecraft:item",
        "sulfur_cube_archetype/slow_sliding",
        &[415, 416, 417, 450, 604, 605, 1408],
    ),
    ("minecraft:item", "sulfur_cube_archetype/sticky", &[1413]),
    ("minecraft:item", "sulfur_cube_food", &[1059]),
    (
        "minecraft:item",
        "sulfur_cube_swallowable",
        &[
            63, 64, 65, 66, 67, 69, 70, 73, 74, 71, 72, 68, 75, 168, 205, 181, 192, 167, 204, 182,
            193, 161, 198, 175, 186, 165, 202, 179, 190, 163, 200, 177, 188, 164, 201, 178, 189,
            162, 199, 176, 187, 169, 206, 183, 194, 166, 203, 180, 191, 172, 184, 207, 195, 173,
            185, 208, 196, 174, 197, 658, 659, 660, 661, 662, 663, 664, 665, 666, 667, 668, 669,
            670, 671, 672, 673, 59, 171, 407, 110, 55, 56, 58, 57, 54, 370, 607, 117, 126, 112,
            113, 111, 97, 107, 98, 93, 94, 95, 96, 128, 109, 118, 119, 120, 121, 122, 123, 124,
            125, 1508, 1509, 1510, 1511, 1512, 1513, 1514, 1515, 137, 138, 139, 140, 141, 142, 143,
            144, 129, 130, 131, 132, 133, 134, 135, 136, 682, 683, 684, 685, 686, 677, 678, 679,
            680, 681, 220, 221, 1056, 290, 293, 441, 442, 446, 437, 532, 384, 385, 386, 1452, 1454,
            1453, 240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
            707, 550, 367, 415, 416, 417, 450, 604, 605, 1408, 1413, 388, 389, 774, 603, 115, 6,
            390, 1416, 332, 11, 52, 413, 454, 1423, 509, 598, 226, 406, 39, 16, 25, 40, 48, 9, 62,
            410, 412, 453, 1427, 405, 60, 1415, 599, 227, 592, 8, 409, 411, 127, 4, 53, 468, 463,
            464, 1419, 395, 2, 224, 348, 404, 408, 452, 387, 754, 349, 7, 391, 1420, 1424, 44, 10,
            5, 3, 31, 17, 590, 591, 354, 355, 510, 511, 108, 512, 606, 597, 775, 225, 596, 392,
            328, 329, 330, 331, 1, 403, 26, 35, 12, 21, 61, 642, 643, 644, 645, 646, 647, 648, 649,
            650, 651, 652, 653, 654, 655, 656, 657, 91, 92, 103, 104, 99, 100, 105, 106, 101, 102,
            549, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529,
            626, 627, 628, 629, 630, 631, 632, 633, 634, 635, 636, 637, 638, 639, 640, 641,
        ],
    ),
    (
        "minecraft:item",
        "swords",
        &[964, 949, 954, 969, 939, 959, 944],
    ),
    (
        "minecraft:item",
        "terracotta",
        &[
            549, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 524, 525, 526, 527, 528, 529,
        ],
    ),
    (
        "minecraft:item",
        "trapdoors",
        &[
            833, 831, 835, 836, 832, 829, 830, 839, 840, 837, 838, 834, 828, 841, 842, 843, 844,
            845, 846, 847, 848,
        ],
    ),
    (
        "minecraft:item",
        "trim_materials",
        &[930, 934, 926, 927, 936, 932, 928, 937, 929, 745, 1276],
    ),
    (
        "minecraft:item",
        "trimmable_armor",
        &[
            985, 989, 993, 1005, 997, 1001, 1009, 984, 988, 992, 1004, 996, 1000, 1008, 983, 987,
            991, 1003, 995, 999, 1007, 982, 986, 990, 1002, 994, 998, 1006, 915,
        ],
    ),
    ("minecraft:item", "turtle_food", &[238]),
    (
        "minecraft:item",
        "villager_picks_up",
        &[979, 1258, 1257, 1318, 1315, 1316, 981, 980, 1317],
    ),
    (
        "minecraft:item",
        "villager_plantable_seeds",
        &[979, 1258, 1257, 1318, 1315, 1316],
    ),
    (
        "minecraft:item",
        "walls",
        &[
            484, 485, 486, 487, 488, 489, 490, 491, 493, 494, 495, 496, 497, 498, 499, 501, 500,
            502, 503, 505, 504, 492, 15, 20, 24, 445, 43, 47, 51, 30, 34, 38,
        ],
    ),
    ("minecraft:item", "warped_stems", &[173, 185, 208, 196]),
    ("minecraft:item", "wart_blocks", &[604, 605]),
    (
        "minecraft:item",
        "wither_skeleton_disliked_weapons",
        &[922, 1370],
    ),
    (
        "minecraft:item",
        "wolf_collar_dyes",
        &[
            1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1103, 1104, 1105, 1106, 1107, 1108,
            1109, 1110,
        ],
    ),
    (
        "minecraft:item",
        "wolf_food",
        &[
            1139, 1141, 1140, 1142, 1295, 1012, 1280, 1294, 1011, 1279, 1143, 1086, 1090, 1087,
            1091, 1088, 1089, 1281,
        ],
    ),
    (
        "minecraft:item",
        "wooden_buttons",
        &[779, 780, 781, 782, 783, 785, 786, 789, 790, 787, 788, 784],
    ),
    (
        "minecraft:item",
        "wooden_doors",
        &[808, 809, 810, 811, 812, 814, 815, 818, 819, 816, 817, 813],
    ),
    (
        "minecraft:item",
        "wooden_fences",
        &[372, 376, 378, 379, 373, 374, 375, 382, 383, 380, 381, 377],
    ),
    (
        "minecraft:item",
        "wooden_pressure_plates",
        &[795, 796, 797, 798, 799, 801, 802, 805, 806, 803, 804, 800],
    ),
    (
        "minecraft:item",
        "wooden_shelves",
        &[333, 334, 335, 336, 337, 338, 339, 340, 341, 342, 343, 344],
    ),
    (
        "minecraft:item",
        "wooden_slabs",
        &[298, 299, 300, 301, 302, 304, 305, 309, 310, 306, 307, 303],
    ),
    (
        "minecraft:item",
        "wooden_stairs",
        &[469, 470, 471, 472, 473, 475, 476, 480, 481, 477, 478, 474],
    ),
    (
        "minecraft:item",
        "wooden_tool_materials",
        &[63, 64, 65, 66, 67, 69, 70, 73, 74, 71, 72, 68],
    ),
    (
        "minecraft:item",
        "wooden_trapdoors",
        &[833, 831, 835, 836, 832, 829, 830, 839, 840, 837, 838, 834],
    ),
    (
        "minecraft:item",
        "wool",
        &[
            240, 241, 242, 243, 244, 245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
        ],
    ),
    (
        "minecraft:item",
        "wool_carpets",
        &[
            533, 534, 535, 536, 537, 538, 539, 540, 541, 542, 543, 544, 545, 546, 547, 548,
        ],
    ),
    ("minecraft:item", "zombie_horse_food", &[276]),
    (
        "minecraft:painting_variant",
        "placeable",
        &[
            24, 1, 0, 2, 5, 32, 47, 35, 12, 37, 42, 13, 46, 22, 26, 8, 40, 45, 39, 50, 19, 33, 31,
            7, 38, 15, 4, 23, 27, 36, 44, 3, 6, 9, 10, 11, 17, 18, 20, 25, 28, 29, 30, 34, 41, 43,
            14,
        ],
    ),
    (
        "minecraft:point_of_interest_type",
        "acquirable_job_site",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
    ),
    ("minecraft:point_of_interest_type", "bee_home", &[15, 16]),
    (
        "minecraft:point_of_interest_type",
        "village",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
    ),
    ("minecraft:timeline", "in_end", &[3]),
    ("minecraft:timeline", "in_nether", &[3]),
    ("minecraft:timeline", "in_overworld", &[3, 0, 2, 1]),
    ("minecraft:timeline", "universal", &[3]),
    (
        "minecraft:worldgen/biome",
        "allows_surface_slime_spawns",
        &[6, 7],
    ),
    (
        "minecraft:worldgen/biome",
        "allows_tropical_fish_spawns_at_any_height",
        &[53],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ancient_city",
        &[54],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/bastion_remnant",
        &[58, 56, 59, 57],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/buried_treasure",
        &[39, 40],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/desert_pyramid",
        &[5],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/end_city",
        &[62, 63],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/igloo",
        &[17, 3, 33],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/jungle_temple",
        &[26, 24],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/mineshaft",
        &[
            50, 48, 46, 44, 49, 45, 47, 43, 42, 37, 38, 39, 40, 30, 34, 35, 36, 33, 31, 20, 22, 21,
            16, 17, 14, 15, 26, 24, 25, 8, 9, 10, 13, 11, 12, 32, 41, 51, 4, 23, 5, 18, 3, 1, 2, 6,
            7, 19, 52, 53, 55,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/mineshaft_mesa",
        &[27, 28, 29],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/nether_fortress",
        &[56, 59, 58, 57, 60],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/nether_fossil",
        &[59],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ocean_monument",
        &[50, 48, 46, 44],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ocean_ruin_cold",
        &[49, 47, 45, 50, 48, 46],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ocean_ruin_warm",
        &[43, 42, 44],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/pillager_outpost",
        &[5, 1, 18, 3, 16, 30, 34, 35, 36, 33, 31, 32],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ruined_portal_desert",
        &[5],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ruined_portal_jungle",
        &[26, 24, 25],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ruined_portal_mountain",
        &[27, 28, 29, 20, 22, 21, 19, 23, 41, 30, 34, 35, 36, 33, 31],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ruined_portal_nether",
        &[56, 59, 58, 57, 60],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ruined_portal_ocean",
        &[50, 48, 46, 44, 49, 45, 47, 43, 42],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ruined_portal_standard",
        &[
            39, 40, 37, 38, 16, 17, 14, 15, 8, 9, 10, 13, 11, 12, 32, 51, 4, 52, 53, 55, 18, 3, 1,
            2,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/ruined_portal_swamp",
        &[6, 7],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/shipwreck",
        &[50, 48, 46, 44, 49, 45, 47, 43, 42],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/shipwreck_beached",
        &[39, 40],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/stronghold",
        &[
            51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 6, 7, 33, 3, 40, 21, 32, 20, 17, 22, 16, 1,
            30, 39, 8, 15, 9, 10, 11, 12, 19, 18, 24, 27, 5, 29, 35, 36, 38, 37, 4, 14, 2, 13, 25,
            26, 28, 23, 31, 34, 52, 53, 55, 54,
        ],
    ),
    ("minecraft:worldgen/biome", "has_structure/swamp_hut", &[6]),
    (
        "minecraft:worldgen/biome",
        "has_structure/trail_ruins",
        &[16, 17, 14, 15, 13, 24],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/trial_chambers",
        &[
            51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 6, 7, 33, 3, 40, 21, 32, 20, 17, 22, 16, 1,
            30, 39, 8, 15, 9, 10, 11, 12, 19, 18, 24, 27, 5, 29, 35, 36, 38, 37, 4, 14, 2, 13, 25,
            26, 28, 23, 31, 34, 52, 53, 55,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/village_desert",
        &[5],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/village_plains",
        &[1, 30],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/village_savanna",
        &[18],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/village_snowy",
        &[3],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/village_taiga",
        &[16],
    ),
    (
        "minecraft:worldgen/biome",
        "has_structure/woodland_mansion",
        &[11, 12],
    ),
    ("minecraft:worldgen/biome", "is_badlands", &[27, 28, 29]),
    ("minecraft:worldgen/biome", "is_beach", &[39, 40]),
    (
        "minecraft:worldgen/biome",
        "is_deep_ocean",
        &[50, 48, 46, 44],
    ),
    ("minecraft:worldgen/biome", "is_end", &[61, 62, 63, 64, 65]),
    (
        "minecraft:worldgen/biome",
        "is_forest",
        &[8, 9, 10, 13, 11, 12, 32],
    ),
    ("minecraft:worldgen/biome", "is_hill", &[20, 22, 21]),
    ("minecraft:worldgen/biome", "is_jungle", &[26, 24, 25]),
    (
        "minecraft:worldgen/biome",
        "is_mountain",
        &[30, 34, 35, 36, 33, 31],
    ),
    (
        "minecraft:worldgen/biome",
        "is_nether",
        &[56, 59, 58, 57, 60],
    ),
    (
        "minecraft:worldgen/biome",
        "is_ocean",
        &[50, 48, 46, 44, 49, 45, 47, 43, 42],
    ),
    (
        "minecraft:worldgen/biome",
        "is_overworld",
        &[
            51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 6, 7, 33, 3, 40, 21, 32, 20, 17, 22, 16, 1,
            30, 39, 8, 15, 9, 10, 11, 12, 19, 18, 24, 27, 5, 29, 35, 36, 38, 37, 4, 14, 2, 13, 25,
            26, 28, 23, 31, 34, 52, 53, 55, 54,
        ],
    ),
    ("minecraft:worldgen/biome", "is_river", &[37, 38]),
    ("minecraft:worldgen/biome", "is_savanna", &[18, 19, 23]),
    ("minecraft:worldgen/biome", "is_taiga", &[16, 17, 14, 15]),
    ("minecraft:worldgen/biome", "mineshaft_blocking", &[54]),
    (
        "minecraft:worldgen/biome",
        "more_frequent_drowned_spawns",
        &[37, 38],
    ),
    (
        "minecraft:worldgen/biome",
        "polar_bears_spawn_on_alternate_blocks",
        &[49, 50],
    ),
    (
        "minecraft:worldgen/biome",
        "produces_corals_from_bonemeal",
        &[42],
    ),
    (
        "minecraft:worldgen/biome",
        "reduce_water_ambient_spawns",
        &[37, 38],
    ),
    (
        "minecraft:worldgen/biome",
        "required_ocean_monument_surrounding",
        &[50, 48, 46, 44, 49, 45, 47, 43, 42, 37, 38],
    ),
    (
        "minecraft:worldgen/biome",
        "spawns_cold_variant_farm_animals",
        &[
            3, 4, 34, 35, 33, 49, 50, 32, 54, 38, 17, 40, 61, 62, 63, 64, 65, 47, 48, 14, 15, 16,
            22, 21, 20, 36,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "spawns_cold_variant_frogs",
        &[
            3, 4, 34, 35, 33, 49, 50, 32, 54, 38, 17, 40, 61, 62, 63, 64, 65,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "spawns_coral_variant_zombie_nautilus",
        &[42],
    ),
    ("minecraft:worldgen/biome", "spawns_gold_rabbits", &[5]),
    (
        "minecraft:worldgen/biome",
        "spawns_snow_foxes",
        &[3, 4, 49, 17, 38, 40, 34, 35, 33, 32],
    ),
    (
        "minecraft:worldgen/biome",
        "spawns_warm_variant_farm_animals",
        &[
            5, 42, 26, 24, 25, 18, 19, 23, 56, 59, 58, 57, 60, 27, 28, 29, 7, 44, 43,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "spawns_warm_variant_frogs",
        &[
            5, 42, 26, 24, 25, 18, 19, 23, 56, 59, 58, 57, 60, 27, 28, 29, 7,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "spawns_white_rabbits",
        &[3, 4, 49, 17, 38, 40, 34, 35, 33, 32],
    ),
    (
        "minecraft:worldgen/biome",
        "stronghold_biased_to",
        &[
            1, 2, 3, 4, 5, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
            26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 51, 52, 53, 55,
        ],
    ),
    (
        "minecraft:worldgen/biome",
        "water_on_map_outlines",
        &[50, 48, 46, 44, 49, 45, 47, 43, 42, 37, 38, 6, 7],
    ),
    (
        "minecraft:worldgen/biome",
        "without_wandering_trader_spawns",
        &[0],
    ),
    ("minecraft:worldgen/biome", "without_zombie_sieges", &[51]),
];
