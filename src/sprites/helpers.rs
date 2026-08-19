//! Shared palette and low-level image construction helpers for sprite generation.

use macroquad::prelude::*;

/// 64-color palette for production-quality sprites.
/// Organized in color ramps (4 shades per hue) for smooth gradients.
/// Index 0 = transparent. RGBA format.
///
/// Layout: Each color family has 4 entries (darkest → lightest).
/// This allows smooth dithering and anti-aliasing between shades.
pub(super) static PALETTE: [(u8, u8, u8, u8); 64] = [
    // 0: Transparent
    (0, 0, 0, 0),
    // 1-4: Neutral grays (outlines, shadows, metals)
    (25, 22, 40, 255),    // 1: Deepest shadow (soft black-purple)
    (48, 44, 65, 255),    // 2: Dark shadow
    (85, 80, 110, 255),   // 3: Mid gray-purple
    (130, 125, 155, 255), // 4: Light gray-purple
    // 5-8: Whites and highlights
    (175, 172, 195, 255), // 5: Silver
    (210, 208, 225, 255), // 6: Pale highlight
    (238, 236, 245, 255), // 7: Near-white
    (255, 253, 250, 255), // 8: Pure specular
    // 9-12: Greens (ground, forest)
    (35, 65, 40, 255),    // 9: Deep forest
    (55, 95, 55, 255),    // 10: Dark grass
    (78, 135, 72, 255),   // 11: Grass
    (120, 180, 105, 255), // 12: Light grass / highlight
    // 13-16: Browns (ores, wood)
    (65, 38, 28, 255),   // 13: Dark brown
    (105, 62, 42, 255),  // 14: Mid brown (iron ore)
    (148, 95, 58, 255),  // 15: Warm brown
    (192, 138, 85, 255), // 16: Light brown / tan
    // 17-20: Oranges/Coral (miner, copper)
    (135, 58, 35, 255),   // 17: Dark coral
    (180, 88, 52, 255),   // 18: Coral
    (220, 135, 72, 255),  // 19: Light coral / amber
    (245, 185, 110, 255), // 20: Peach highlight
    // 21-24: Reds (furnace, enemies)
    (95, 28, 28, 255),    // 21: Dark red
    (148, 48, 45, 255),   // 22: Mid red
    (205, 78, 65, 255),   // 23: Warm red
    (240, 125, 105, 255), // 24: Light salmon
    // 25-28: Blues (assembler, water)
    (30, 48, 105, 255),   // 25: Deep blue
    (52, 78, 155, 255),   // 26: Mid blue
    (88, 125, 205, 255),  // 27: Periwinkle
    (135, 175, 235, 255), // 28: Light sky blue
    // 29-32: Purples (lab, FORGE)
    (68, 35, 105, 255),   // 29: Deep purple
    (108, 62, 155, 255),  // 30: Mid purple
    (155, 98, 205, 255),  // 31: Lilac
    (198, 148, 235, 255), // 32: Light lavender
    // 33-36: Teals (power, generators)
    (28, 72, 72, 255),    // 33: Deep teal
    (48, 115, 112, 255),  // 34: Mid teal
    (82, 162, 155, 255),  // 35: Mint
    (128, 205, 195, 255), // 36: Light mint
    // 37-40: Yellows (belts, warnings)
    (105, 95, 28, 255),   // 37: Dark gold
    (165, 152, 42, 255),  // 38: Gold
    (225, 210, 68, 255),  // 39: Yellow
    (248, 238, 125, 255), // 40: Light lemon
    // 41-44: Pinks (enemies, highlights)
    (115, 35, 62, 255),   // 41: Dark rose
    (172, 58, 88, 255),   // 42: Rose
    (225, 95, 125, 255),  // 43: Pink
    (248, 155, 175, 255), // 44: Light pink
    // 45-48: Stone/Earth (walls, stone)
    (72, 68, 62, 255),    // 45: Dark stone
    (112, 108, 98, 255),  // 46: Mid stone
    (155, 150, 138, 255), // 47: Light stone
    (198, 195, 185, 255), // 48: Pale stone
    // 49-52: Copper/Gold metallic
    (125, 72, 22, 255),   // 49: Dark copper
    (178, 108, 38, 255),  // 50: Mid copper
    (225, 155, 65, 255),  // 51: Bright copper
    (248, 205, 115, 255), // 52: Gold highlight
    // 53-56: Cool metals (steel, inserters)
    (62, 72, 85, 255),    // 53: Dark steel
    (95, 108, 125, 255),  // 54: Mid steel
    (135, 148, 168, 255), // 55: Light steel
    (178, 192, 208, 255), // 56: Steel highlight
    // 57-60: Greens bright (circuits, science)
    (22, 95, 45, 255),    // 57: Dark circuit green
    (48, 155, 72, 255),   // 58: Circuit green
    (82, 210, 105, 255),  // 59: Bright green
    (145, 238, 158, 255), // 60: Light green glow
    // 61-63: Special (fire glow, uranium, warning)
    (255, 155, 32, 255), // 61: Fire orange
    (48, 225, 85, 255),  // 62: Uranium glow
    (255, 68, 68, 255),  // 63: Alert red
];

/// Convert palette index to macroquad Color.
pub(super) fn pal_color(idx: usize) -> Color {
    let (r, g, b, a) = PALETTE[idx];
    Color::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

/// Check if a Color matches a palette index.
pub(super) fn is_pal(c: Color, idx: usize) -> bool {
    let p = pal_color(idx);
    (c.r - p.r).abs() < 0.01 && (c.g - p.g).abs() < 0.01 && (c.b - p.b).abs() < 0.01
}

/// Creates an [`Image`] from a 2D grid of palette indices at the given size.
pub(super) fn make_image_rect(pixels: &[&[u8]], width: u16, height: u16) -> Image {
    let mut image = Image::gen_image_color(width, height, Color::new(0.0, 0.0, 0.0, 0.0));
    for (y, row) in pixels.iter().enumerate() {
        for (x, &idx) in row.iter().enumerate() {
            if idx == 0 || x >= width as usize || y >= height as usize {
                continue;
            }
            let (r, g, b, a) = PALETTE[idx as usize];
            image.set_pixel(
                x as u32,
                y as u32,
                Color::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ),
            );
        }
    }
    image
}

pub(super) fn make_image(pixels: &[&[u8]], size: u16) -> Image {
    let mut image = Image::gen_image_color(size, size, Color::new(0.0, 0.0, 0.0, 0.0));

    for (y, row) in pixels.iter().enumerate() {
        for (x, &idx) in row.iter().enumerate() {
            if idx == 0 || x >= size as usize || y >= size as usize {
                continue;
            }
            let (r, g, b, a) = PALETTE[idx as usize];
            image.set_pixel(
                x as u32,
                y as u32,
                Color::new(
                    r as f32 / 255.0,
                    g as f32 / 255.0,
                    b as f32 / 255.0,
                    a as f32 / 255.0,
                ),
            );
        }
    }

    image
}
