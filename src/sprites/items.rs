//! Item icon sprites shown on belts and in inventories.

use macroquad::prelude::*;

use super::helpers::*;

/// Ore chunk — rough angular rock with colored facets and highlight.
pub(super) fn make_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 1, 1, 1, 1, 0, 0],
        &[0, 1,light,light,light, 8, 1, 0],
        &[1,light,light,dark,dark,light,light, 1],
        &[1,light,dark,dark,dark,dark,light, 1],
        &[1,dark,dark,dark,dark,dark,dark, 1],
        &[1,dark,dark,dark,dark,dark, 1, 0],
        &[0, 1, 1,dark,dark, 1, 1, 0],
        &[0, 0, 0, 1, 1, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Battery — green rectangular cell with terminals.
pub(super) fn make_battery_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 4, 4, 0, 0, 0],
        &[0, 0, 1, 3, 3, 1, 0, 0],
        &[0, 1, 9,10,10, 9, 1, 0],
        &[0, 1, 9,10,11, 9, 1, 0],
        &[0, 1, 9,10,10, 9, 1, 0],
        &[0, 1, 9, 9, 9, 9, 1, 0],
        &[0, 0, 1, 1, 1, 1, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Ammo — brass cartridge shape.
pub(super) fn make_ammo_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 17, 0, 0, 0, 0],
        &[0, 0,17,19,17, 0, 0, 0],
        &[0, 0,16,17,16, 0, 0, 0],
        &[0, 0,16,17,16, 0, 0, 0],
        &[0, 0,16,16,16, 0, 0, 0],
        &[0, 0,16,16,16, 0, 0, 0],
        &[0, 0, 1,16, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Grenade — green sphere with pin.
pub(super) fn make_grenade_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 4, 3, 0, 0, 0],
        &[0, 0, 1, 1, 1, 1, 0, 0],
        &[0, 1, 9,10,11,10, 1, 0],
        &[0, 1, 9,10,10,10, 1, 0],
        &[0, 1, 9, 9,10, 9, 1, 0],
        &[0, 0, 1, 9, 9, 1, 0, 0],
        &[0, 0, 0, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Engine unit — mechanical assembly with piston.
pub(super) fn make_engine_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 1, 3, 3, 3, 3, 1, 0],
        &[1, 3, 4, 5, 4, 5, 3, 1],
        &[1, 4,53,54,53,54, 4, 1],
        &[1, 3, 4, 4, 4, 4, 3, 1],
        &[1, 4,53, 3,53, 3, 4, 1],
        &[1, 3, 4, 4, 4, 4, 3, 1],
        &[0, 1, 3, 3, 3, 3, 1, 0],
        &[0, 0, 1, 1, 1, 1, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Rocket part — white cylinder with red stripe.
pub(super) fn make_rocket_part_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 7, 7, 0, 0, 0],
        &[0, 0, 5, 6, 7, 5, 0, 0],
        &[0, 1, 5, 6, 7, 5, 1, 0],
        &[0, 1,22,23,23,22, 1, 0],
        &[0, 1, 5, 6, 7, 5, 1, 0],
        &[0, 1, 5, 6, 7, 5, 1, 0],
        &[0, 0, 1, 3, 3, 1, 0, 0],
        &[0, 0, 0, 1, 1, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Coal chunk — dark jagged irregular shape (distinct from round ores).
pub(super) fn make_coal_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 1, 0, 0, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
        &[0, 1, 1, 2, 1, 1, 0, 0],
        &[1, 1, 2, 3, 2, 1, 1, 0],
        &[0, 1, 1, 2, 1, 1, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
        &[0, 0, 0, 1, 1, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Stone piece — flat layered sedimentary shape (visually distinct from ores).
pub(super) fn make_stone_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0, 0, 0, 0, 0, 0],
        &[0,45,46,47,47,46, 0, 0],
        &[45,46,47,48,47,46,45, 0],
        &[45,45,46,47,46,45,45, 0],
        &[0,45,46,47,47,46,45, 0],
        &[0,45,45,46,46,45, 0, 0],
        &[0, 0,45,45,45, 0, 0, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Metal plate — flat shiny rectangle with beveled edges and specular corner.
pub(super) fn make_plate_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 1, 1, 1, 1, 1, 1, 0],
        &[1, 8,light,light,light,light,dark, 1],
        &[1,light,light,light,light,dark,dark, 1],
        &[1,light,light,dark,dark,dark,dark, 1],
        &[1,light,dark,dark,dark,dark,dark, 1],
        &[1,dark,dark,dark,dark,dark, 1, 1],
        &[0, 1, 1, 1, 1, 1, 1, 0],
        &[0, 0, 0, 0, 0, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

/// Gear — cogwheel with visible teeth around a central hole.
pub(super) fn make_gear_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 55, 0,55,55, 0,55, 0],
        &[55,56,55,55,55,55,54, 55],
        &[0, 55,55, 1, 1,55,54, 0],
        &[55,55, 1, 0, 0, 1,54,55],
        &[55,54, 1, 0, 0, 1,53,55],
        &[0, 54,53, 1, 1,53,53, 0],
        &[55,53,54,53,53,54,53,55],
        &[0, 55, 0,55,55, 0,55, 0],
    ];
    make_image(pixels, 8)
}

/// Wire — copper coil with multiple visible loops and shine.
pub(super) fn make_wire_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0,51, 0, 0, 0, 0, 0],
        &[0,51,52,51, 0, 0, 0, 0],
        &[0, 0,50,51,50, 0, 0, 0],
        &[0, 0, 0,49,51,49, 0, 0],
        &[0, 0,49,51,52,51, 0, 0],
        &[0,49,51,49, 0,49,51, 0],
        &[49,51,49, 0, 0, 0,49,51],
        &[0,49, 0, 0, 0, 0, 0,49],
    ];
    make_image(pixels, 8)
}

/// Circuit board — green PCB with visible traces, components, and solder points.
pub(super) fn make_circuit_item_sprite() -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[57,57,57,57,57,57,57,57],
        &[57, 8,58,58,58,58, 8,57],
        &[57,59, 1,58,58, 1,59,57],
        &[57,58,58,60,59,58,58,57],
        &[57,58,58,59,60,58,58,57],
        &[57,59, 1,58,58, 1,59,57],
        &[57, 8,58,58,58,58, 8,57],
        &[57,57,57,57,57,57,57,57],
    ];
    make_image(pixels, 8)
}

/// Colored circuit board — same shape, different colors (for red/blue circuits).
pub(super) fn make_circuit_item_sprite_colored(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[dark,dark,dark,dark,dark,dark,dark,dark],
        &[dark, 8,light,light,light,light, 8,dark],
        &[dark,light, 1,light,light, 1,light,dark],
        &[dark,light,light,dark,light,light,light,dark],
        &[dark,light,light,light,dark,light,light,dark],
        &[dark,light, 1,light,light, 1,light,dark],
        &[dark, 8,light,light,light,light, 8,dark],
        &[dark,dark,dark,dark,dark,dark,dark,dark],
    ];
    make_image(pixels, 8)
}

/// Science flask — rounded glass bottle with colored liquid and cork top.
pub(super) fn make_flask_item_sprite(dark: u8, light: u8) -> Image {
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        &[0, 0, 0,16,16, 0, 0, 0],
        &[0, 0, 7, 7, 7, 7, 0, 0],
        &[0, 0, 1,light, 1, 0, 0, 0],
        &[0, 1,light, 8,light, 1, 0, 0],
        &[1,dark,light,light,light,dark, 1, 0],
        &[1,dark,dark,light,dark,dark, 1, 0],
        &[0, 1,dark,dark,dark, 1, 0, 0],
        &[0, 0, 1, 1, 1, 0, 0, 0],
    ];
    make_image(pixels, 8)
}

// --- Enemy sprites (12×12) ---
