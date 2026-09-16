//! IcebergFeature port (packed ice / snow, ellipse shapes, cut-out carving).
use super::*;
use crate::feature_catalog;
use crate::feature_dispatch;
use crate::feature_dispatch::*;
use crate::feature_rng::FeatureRandom;
use crate::generator::{WORLD_BOTTOM, WORLD_TOP};
use crate::legacy_rng::LegacyRandom;
use crate::region_buf::RegionBuf;
use crate::surface::BlockId;
use crate::worldgen::WorldgenState;
use serde_json::Value;


// ---------------------------------------------------------------------------
// iceberg
// ---------------------------------------------------------------------------

/// `IcebergFeature.place` (26.2). `state` is the configured block
/// (blue_ice / packed_ice); snow_on_top uses `BlockId::Snow`.
pub(crate) fn place_iceberg(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    z: i32,
    cfg: &Value,
) {
    let main_state = cfg["config"]["state"]["Name"]
        .as_str()
        .and_then(BlockId::from_name)
        .unwrap_or(BlockId::PackedIce);
    let origin_y = SEA_LEVEL;
    let snow_on_top = rng.next_f64() > 0.7;
    let shape_angle = rng.next_f64() * 2.0 * std::f64::consts::PI;
    let shape_ellipse_a = 11 - rng.next_int(5);
    let shape_ellipse_c = 3 + rng.next_int(3);
    let is_ellipse = rng.next_f64() > 0.7;
    let mut over_water_height = if is_ellipse {
        rng.next_int(6) + 6
    } else {
        rng.next_int(15) + 3
    };
    if !is_ellipse && rng.next_f64() > 0.9 {
        over_water_height += rng.next_int(19) + 7;
    }
    let under_water_height = (over_water_height + rng.next_int(11)).min(18);
    let width = (over_water_height + rng.next_int(7) - rng.next_int(5)).min(11);
    let a = if is_ellipse { shape_ellipse_a } else { 11 };

    for xo in -a..a {
        for zo in -a..a {
            for y_off in 0..over_water_height {
                let radius = if is_ellipse {
                    height_dependent_radius_ellipse(y_off, over_water_height, width)
                } else {
                    height_dependent_radius_round(rng, y_off, over_water_height, width)
                };
                if is_ellipse || xo < radius {
                    generate_iceberg_block(
                        rng,
                        region,
                        x,
                        origin_y,
                        z,
                        over_water_height,
                        xo,
                        y_off,
                        zo,
                        radius,
                        a,
                        is_ellipse,
                        shape_ellipse_c,
                        shape_angle,
                        snow_on_top,
                        main_state,
                    );
                }
            }
        }
    }
    smooth_iceberg(region, x, origin_y, z, width, over_water_height, is_ellipse, shape_ellipse_a);
    for xo in -a..a {
        for zo in -a..a {
            for y_off in -1..-under_water_height {
                let new_a = if is_ellipse {
                    ((a as f32) * (1.0 - (y_off * y_off) as f32 / (under_water_height * 8) as f32)).ceil() as i32
                } else {
                    a
                };
                let radius = height_dependent_radius_steep(rng, -y_off, under_water_height, width);
                if xo < radius {
                    generate_iceberg_block(
                        rng,
                        region,
                        x,
                        origin_y,
                        z,
                        under_water_height,
                        xo,
                        y_off,
                        zo,
                        radius,
                        new_a,
                        is_ellipse,
                        shape_ellipse_c,
                        shape_angle,
                        snow_on_top,
                        main_state,
                    );
                }
            }
        }
    }
    let do_cut_out = if is_ellipse {
        rng.next_f64() > 0.1
    } else {
        rng.next_f64() > 0.7
    };
    if do_cut_out {
        generate_cut_out(
            rng,
            region,
            x,
            origin_y,
            z,
            width,
            over_water_height,
            is_ellipse,
            shape_ellipse_a,
            shape_angle,
            shape_ellipse_c,
        );
    }
}

fn height_dependent_radius_round(rng: &mut FeatureRandom, y_off: i32, height: i32, width: i32) -> i32 {
    let k = 3.5 - rng.next_f32();
    let mut scale = (1.0 - (y_off * y_off) as f32 / (height as f32 * k)) * width as f32;
    if height > 15 + rng.next_int(5) {
        let temp_y_off = if y_off < 3 + rng.next_int(6) { y_off / 2 } else { y_off };
        scale = (1.0 - temp_y_off as f32 / (height as f32 * k * 0.4)) * width as f32;
    }
    (scale / 2.0).ceil() as i32
}

fn height_dependent_radius_ellipse(y_off: i32, height: i32, width: i32) -> i32 {
    let scale = (1.0 - (y_off * y_off) as f32 / height as f32) * width as f32;
    (scale / 2.0).ceil() as i32
}

fn height_dependent_radius_steep(rng: &mut FeatureRandom, y_off: i32, height: i32, width: i32) -> i32 {
    let k = 1.0 + rng.next_f32() / 2.0;
    let scale = (1.0 - y_off as f32 / (height as f32 * k)) * width as f32;
    (scale / 2.0).ceil() as i32
}

fn generate_iceberg_block(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    origin_y: i32,
    z: i32,
    height: i32,
    xo: i32,
    y_off: i32,
    zo: i32,
    radius: i32,
    a: i32,
    is_ellipse: bool,
    shape_ellipse_c: i32,
    shape_angle: f64,
    snow_on_top: bool,
    main_state: BlockId,
) {
    let signed_dist = if is_ellipse {
        let c = get_ellipse_c(y_off, height, shape_ellipse_c);
        signed_distance_ellipse(xo, zo, a, c, shape_angle)
    } else {
        signed_distance_circle(rng, xo, zo, radius)
    };
    if signed_dist < 0.0 {
        let compare_val = if is_ellipse { -0.5 } else { -6.0 - rng.next_int(3) as f64 };
        if signed_dist > compare_val && rng.next_f64() > 0.9 {
            return;
        }
        set_iceberg_block(
            region,
            x + xo,
            origin_y + y_off,
            z + zo,
            rng,
            height - y_off,
            height,
            is_ellipse,
            snow_on_top,
            main_state,
        );
    }
}

fn get_ellipse_c(y_off: i32, height: i32, shape_ellipse_c: i32) -> i32 {
    let mut c = shape_ellipse_c;
    if y_off > 0 && height - y_off <= 3 {
        c -= 4 - (height - y_off);
    }
    c
}

fn signed_distance_circle(rng: &mut FeatureRandom, xo: i32, zo: i32, radius: i32) -> f64 {
    let off = 10.0 * rng.next_f32().clamp(0.2, 0.8) / radius as f32;
    (off as f64) + (xo * xo + zo * zo) as f64 - (radius * radius) as f64
}

fn signed_distance_ellipse(xo: i32, zo: i32, a: i32, c: i32, angle: f64) -> f64 {
    let (s, c_angle) = angle.sin_cos();
    let xr = (xo as f64 * c_angle - zo as f64 * s) / a as f64;
    let zr = (xo as f64 * s + zo as f64 * c_angle) / c as f64;
    xr * xr + zr * zr - 1.0
}

fn set_iceberg_block(
    region: &mut RegionBuf,
    x: i32,
    y: i32,
    z: i32,
    rng: &mut FeatureRandom,
    h_diff: i32,
    height: i32,
    is_ellipse: bool,
    snow_on_top: bool,
    main_state: BlockId,
) {
    let state = region.get(x, y, z);
    if state == BlockId::Air
        || state == BlockId::CaveAir
        || state == BlockId::Snow
        || state == BlockId::Ice
        || state == BlockId::Water
    {
        let randomness = !is_ellipse || rng.next_f64() > 0.05;
        let divisor = if is_ellipse { 3 } else { 2 };
        if snow_on_top
            && state != BlockId::Water
            && h_diff <= rng.next_int(1.max(height / divisor)) + (height as f64 * 0.6) as i32
            && randomness
        {
            region.set(x, y, z, BlockId::Snow);
        } else {
            region.set(x, y, z, main_state);
        }
    }
}

fn is_iceberg_state(b: BlockId) -> bool {
    matches!(b, BlockId::PackedIce | BlockId::Snow | BlockId::BlueIce)
}

fn smooth_iceberg(
    region: &mut RegionBuf,
    x: i32,
    origin_y: i32,
    z: i32,
    width: i32,
    height: i32,
    is_ellipse: bool,
    shape_ellipse_a: i32,
) {
    let a = if is_ellipse { shape_ellipse_a } else { width / 2 };
    for dx in -a..=a {
        for dz in -a..=a {
            for y_off in 0..=height {
                let b = region.get(x + dx, origin_y + y_off, z + dz);
                if is_iceberg_state(b) || b == BlockId::Snow {
                    if region.get(x + dx, origin_y + y_off - 1, z + dz).is_air() {
                        region.set(x + dx, origin_y + y_off, z + dz, BlockId::Air);
                        region.set(x + dx, origin_y + y_off + 1, z + dz, BlockId::Air);
                    } else if is_iceberg_state(b) {
                        let mut counter = 0;
                        for &(sdx, sdz) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {
                            if !is_iceberg_state(region.get(x + dx + sdx, origin_y + y_off, z + dz + sdz)) {
                                counter += 1;
                            }
                        }
                        if counter >= 3 {
                            region.set(x + dx, origin_y + y_off, z + dz, BlockId::Air);
                        }
                    }
                }
            }
        }
    }
}

fn generate_cut_out(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    origin_y: i32,
    z: i32,
    width: i32,
    height: i32,
    is_ellipse: bool,
    shape_ellipse_a: i32,
    shape_angle: f64,
    shape_ellipse_c: i32,
) {
    let random_sign_x = if rng.next_boolean() { -1 } else { 1 };
    let random_sign_z = if rng.next_boolean() { -1 } else { 1 };
    let mut x_off = rng.next_int(1.max(width / 2 - 2));
    if rng.next_boolean() {
        x_off = width / 2 + 1 - rng.next_int(1.max(width - width / 2 - 1));
    }
    let mut z_off = rng.next_int(1.max(width / 2 - 2));
    if rng.next_boolean() {
        z_off = width / 2 + 1 - rng.next_int(1.max(width - width / 2 - 1));
    }
    if is_ellipse {
        x_off = rng.next_int(1.max(shape_ellipse_a - 5));
        z_off = x_off;
    }
    let local_ox = random_sign_x * x_off;
    let local_oz = random_sign_z * z_off;
    let angle = if is_ellipse {
        shape_angle + std::f64::consts::FRAC_PI_2
    } else {
        rng.next_f64() * 2.0 * std::f64::consts::PI
    };
    for y_off in 0..height - 3 {
        let radius = height_dependent_radius_round(rng, y_off, height, width);
        carve_iceberg(
            rng,
            region,
            x,
            origin_y,
            z,
            radius,
            y_off,
            false,
            angle,
            local_ox,
            local_oz,
            shape_ellipse_a,
            shape_ellipse_c,
        );
    }
    for y_off in -1..-(height - rng.next_int(5)) {
        let radius = height_dependent_radius_steep(rng, -y_off, height, width);
        carve_iceberg(
            rng,
            region,
            x,
            origin_y,
            z,
            radius,
            y_off,
            true,
            angle,
            local_ox,
            local_oz,
            shape_ellipse_a,
            shape_ellipse_c,
        );
    }
}

fn carve_iceberg(
    rng: &mut FeatureRandom,
    region: &mut RegionBuf,
    x: i32,
    origin_y: i32,
    z: i32,
    radius: i32,
    y_off: i32,
    under_water: bool,
    angle: f64,
    local_ox: i32,
    local_oz: i32,
    shape_ellipse_a: i32,
    shape_ellipse_c: i32,
) {
    let a = radius + 1 + shape_ellipse_a / 3;
    let c = (radius - 3).min(3) + shape_ellipse_c / 2 - 1;
    for xo in -a..a {
        for zo in -a..a {
            let signed_dist = signed_distance_ellipse(xo - local_ox, zo - local_oz, a, c, angle);
            if signed_dist < 0.0 {
                let b = region.get(x + xo, origin_y + y_off, z + zo);
                if is_iceberg_state(b) || b == BlockId::Snow {
                    if under_water {
                        region.set(x + xo, origin_y + y_off, z + zo, BlockId::Water);
                    } else {
                        region.set(x + xo, origin_y + y_off, z + zo, BlockId::Air);
                        // removeFloatingSnowLayer: clears a floating
                        // `minecraft:snow` LAYER (not snow_block) above.
                        if region.get(x + xo, origin_y + y_off + 1, z + zo) == BlockId::SnowLayer {
                            region.set(x + xo, origin_y + y_off + 1, z + zo, BlockId::Air);
                        }
                    }
                }
            }
        }
    }
}
