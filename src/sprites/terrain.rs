//! Ground, forest, water, ore, and oil deposit sprites.

use macroquad::prelude::*;

use super::helpers::*;

/// Creates a [`Texture2D`] from a 2D grid of palette indices.
/// Creates a high-quality ground tile with Bayer dithering, subtle texture variation,
/// and occasional tiny detail pixels (flowers/pebbles).
pub(super) fn make_ground_image(base: u8, highlight: u8, alt: u8) -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    // Bayer 4×4 ordered dithering matrix for smooth gradient blending.
    let bayer: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            let threshold = bayer[(y % 4) as usize][(x % 4) as usize];
            // Hash for pseudo-random variation.
            let hash =
                ((x as u32).wrapping_mul(2654435761) ^ (y as u32).wrapping_mul(340573321)) % 100;

            let pixel = if hash < 3 {
                // Rare bright detail (tiny flower or pebble) — 3% chance.
                highlight
            } else if hash < 8 {
                // Occasional alternate shade — 5% chance.
                alt
            } else if threshold > 11 {
                // Dithered transition to highlight (top 25% of Bayer).
                highlight
            } else if threshold < 4 {
                // Dithered transition to alt/shadow (bottom 25% of Bayer).
                alt
            } else {
                // Base color (majority of pixels).
                base
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

/// Returns forest tile as Image — deep green canopy with layered depth.
/// Uses green ramp (9-12) with dark shadows (1-2) for dense forest feel.
pub(super) fn make_forest_image() -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            let hash = ((x as u32).wrapping_mul(7919) ^ (y as u32).wrapping_mul(104729)) % 100;
            let pixel = if hash < 5 {
                12 // Bright green (dappled sunlight through canopy)
            } else if hash < 15 {
                11 // Mid green (leaves)
            } else if hash < 35 {
                10 // Dark green (canopy shadow)
            } else if hash < 55 {
                9 // Deepest green (dense shadow)
            } else if hash < 60 {
                14 // Brown (tree trunk peek-through)
            } else if hash < 65 {
                2 // Dark shadow (depth)
            } else {
                1 // Deepest shadow (makes forest look VERY dark and dense)
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

/// Returns water tile as Image — soft blue with wave patterns and sparkle.
/// Uses blue ramp (25-28) for depth.
pub(super) fn make_water_image(base: u8, highlight: u8) -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            // Wave pattern — sinusoidal ripples.
            let wave_phase = (x as f32 * 0.8 + y as f32 * 0.4).sin();
            let hash = ((x as u32).wrapping_mul(48271) ^ (y as u32).wrapping_mul(12345)) % 100;

            let pixel = if hash < 3 {
                8 // Rare white sparkle (sunlight on water)
            } else if wave_phase > 0.7 {
                highlight // Wave crest (lighter)
            } else if wave_phase < -0.5 {
                25 // Deep blue (wave trough)
            } else if hash < 20 {
                26 // Mid blue variation
            } else {
                base // Base water color
            };
            row.push(pixel);
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

/// Creates an ore deposit sprite — a large rocky formation with colored veins.
///
/// The rock has a 3D appearance with highlights on top-left and shadows on
/// bottom-right, with ore veins (colored specks) showing through the stone.
/// Creates ore deposit sprite — crystalline rock with colored veins.
/// Uses stone ramp (45-48) for rock body, ore color (dark/light) for veins,
/// ambient occlusion at base, specular highlight at top-left.
pub(super) fn make_ore_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1,48, 8,48,47,47,46, 1, 0, 0, 0, 0],
        &[0, 0, 0, 1,48, 8,light,48,47,light,47,46, 1, 0, 0, 0],
        &[0, 0, 1,48, 8,48,47,light,47,46,dark,46,45, 1, 0, 0],
        &[0, 1,48,48,light,48,47,47,46,dark,46,45,45, 1, 0, 0],
        &[0, 1,48,47,47,light,47,46,46,46,dark,45,45, 2, 1, 0],
        &[1,47,48,47,47,46,light,46,45,dark,45,45, 2, 2, 1, 0],
        &[1,47,47,46,46,46,dark,45,45,45, 2,dark, 2, 2, 2, 1],
        &[1,47,46,46,dark,45,45, 2,dark, 2, 2, 2, 2, 2, 2, 1],
        &[1,46,46,45,45,dark, 2, 2, 2, 2, 2, 2, 1, 2, 1, 0],
        &[0, 1,45,45, 2, 2,dark, 2, 2, 2, 2, 1, 1, 1, 0, 0],
        &[0, 1, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0],
        &[0, 0, 1, 1, 2, 2, 2, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 16)
}

/// Creates an oil well indicator sprite.
pub(super) fn make_oil_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,1,27,27,1,0,0,0,0,0,0],
        &[0,0,0,0,0,1,27,27,27,27,1,0,0,0,0,0],
        &[0,0,0,0,0,1,27,2,2,27,1,0,0,0,0,0],
        &[0,0,0,0,0,0,1,27,27,1,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,1,27,27,1,0,0,0,0,0,0],
        &[0,0,0,0,0,1,27,27,27,27,1,0,0,0,0,0],
        &[0,0,0,0,1,27,27,2,2,27,27,1,0,0,0,0],
        &[0,0,0,0,1,27,27,27,27,27,27,1,0,0,0,0],
        &[0,0,0,0,0,1,1,1,1,1,1,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];
    make_image(pixels, 16)
}
