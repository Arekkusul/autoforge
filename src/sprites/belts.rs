//! Conveyor belt sprites (straight runs and corners, all colors/frames).

use macroquad::prelude::*;

use super::helpers::*;

/// Creates a belt sprite. `frame` shifts the chevron arrows for animation.
/// Belt sprite with smooth metallic track and animated chevron arrows.
/// Uses steel ramp (53-56) for track body, arrow_color for directional indicators.
/// Rounded edge with shadow on right, highlight on left.
pub(super) fn make_belt_sprite(base_color: u8, arrow_color: u8, frame: u8) -> Image {
    let mut rows: Vec<Vec<u8>> = Vec::new();
    for y in 0..16u8 {
        let mut row = Vec::new();
        for x in 0..16u8 {
            // Rounded belt edges with gradient.
            if x == 0 || x == 15 {
                row.push(1); // Outer outline (darkest)
            } else if x == 1 {
                row.push(55); // Left highlight edge (steel light)
            } else if x == 14 {
                row.push(53); // Right shadow edge (steel dark)
            } else if x == 2 {
                row.push(56); // Inner left highlight
            } else if x == 13 {
                row.push(54); // Inner right shadow
            } else {
                // Belt surface with smooth chevrons.
                let shifted_y = (y + frame * 3) % 6;
                let center_dist = (x as i8 - 7).unsigned_abs();
                let is_chevron = shifted_y < 2 && center_dist <= (2u8.saturating_sub(shifted_y));
                let is_chevron_edge = shifted_y == 2 && center_dist <= 1;

                if is_chevron {
                    row.push(arrow_color);
                } else if is_chevron_edge {
                    // Chevron anti-alias edge (dimmer version of arrow)
                    row.push(base_color);
                } else {
                    // Belt surface — subtle gradient from left (lighter) to right (darker).
                    if x < 7 {
                        row.push(55); // Lighter left side
                    } else {
                        row.push(54); // Darker right side
                    }
                }
            }
        }
        rows.push(row);
    }
    let row_refs: Vec<&[u8]> = rows.iter().map(|r| r.as_slice()).collect();
    make_image(&row_refs, 16)
}

/// Corner belt sprite matching straight belt design. Fills the entire tile with
/// the same edge pattern (outline→highlight→inner hl→surface→inner sh→shadow→outline)
/// plus a rounded cutout in the inner corner.
/// `mirror=false`: left-turn (W→N). `mirror=true`: right-turn (E→N).
// The per-edge palette selection intentionally spells out each branch for
// readability; several branches happen to yield the same shade.
#[allow(clippy::if_same_then_else)]
pub(super) fn make_belt_corner_sprite(
    _base_color: u8,
    _arrow_color: u8,
    _frame: u8,
    mirror: bool,
) -> Image {
    let mut pixels = [[0u8; 16]; 16];

    for y in 0..16u8 {
        for x in 0..16u8 {
            // Work in canonical left-turn space; mirror flips x.
            let sx = if mirror { 15 - x } else { x };

            // Cutout in bottom-right: rounded inner corner.
            let cdx = 15.5 - sx as f32;
            let cdy = 15.5 - y as f32;
            let cdist = (cdx * cdx + cdy * cdy).sqrt();
            let cut_r = 3.0f32;

            if cdist < cut_r {
                pixels[y as usize][x as usize] = 0; // transparent cutout
                continue;
            }

            // Determine edge layers exactly like the straight belt.
            // Straight belt: x=0→outline(1), x=1→hl(55), x=2→ihl(56),
            //                x=13→ish(54), x=14→sh(53), x=15→outline(1)
            //                surface: x<7→55, x>=7→54
            // Corner: apply same pattern on BOTH axes, pick the outermost.

            // Distance from each edge.
            let d_left = sx;
            let d_top = y;
            let d_right = 15 - sx;
            let d_bottom = 15 - y;
            // Distance from inner corner curve.
            let d_curve = cdist - cut_r;

            // The "depth" into the belt from the nearest boundary.
            let d_min = d_left
                .min(d_top)
                .min(d_right)
                .min(d_bottom)
                .min(d_curve as u8);

            let pixel = match d_min {
                0 => 1, // outline
                1 => {
                    // Highlight or shadow depending on which edge is closest.
                    if d_left <= d_right && d_left <= d_bottom && d_left <= (d_curve as u8) {
                        if mirror {
                            53
                        } else {
                            55
                        } // left edge
                    } else if d_top <= d_bottom && d_top <= d_right && d_top <= (d_curve as u8) {
                        55 // top edge (always lit)
                    } else if d_curve as u8 <= d_right && (d_curve as u8) <= d_bottom {
                        55 // inner curve highlight
                    } else if d_right <= d_bottom {
                        if mirror {
                            55
                        } else {
                            53
                        } // right edge shadow
                    } else {
                        53 // bottom edge shadow
                    }
                }
                2 => {
                    if d_left <= d_right && d_left <= d_bottom && d_left <= (d_curve as u8) {
                        if mirror {
                            54
                        } else {
                            56
                        }
                    } else if d_top <= d_bottom && d_top <= d_right && d_top <= (d_curve as u8) {
                        56
                    } else if d_curve as u8 <= d_right && (d_curve as u8) <= d_bottom {
                        56
                    } else if d_right <= d_bottom {
                        if mirror {
                            56
                        } else {
                            54
                        }
                    } else {
                        54
                    }
                }
                _ => {
                    // Belt surface — same gradient as straight belt.
                    if sx < 8 {
                        55
                    } else {
                        54
                    }
                }
            };

            pixels[y as usize][x as usize] = pixel;
        }
    }

    let rows: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&rows, 16)
}

// ===========================================================================
// Hand-crafted 16×16 machine sprites
// Light source: top-left. Outlines use selective outlining (selout).
// ===========================================================================
