//! Enemy, robot, and FORGE avatar character sprites.

use macroquad::prelude::*;

use super::helpers::*;

/// Alien guardian — 3-segment insectoid: head with mandibles, thorax, abdomen.
/// 6 jointed legs, glowing eyes, jagged carapace outline.
/// Professional pixel art anatomy: distinct body segments, sharp silhouette.
pub(super) fn make_enemy_sprite(body: u8, shadow: u8, frame: u8) -> Image {
    // Frame 0: legs extended left. Frame 1: legs extended right (walk cycle).
    let (l0, l1) = if frame == 0 {
        (shadow, 0u8)
    } else {
        (0u8, shadow)
    };
    let pixels: Vec<Vec<u8>> = vec![
        //     mandibles    head     mandibles
        vec![shadow, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, shadow],
        vec![
            0, shadow, 0, 0, shadow, shadow, shadow, shadow, 0, 0, shadow, 0,
        ],
        vec![0, 0, 0, shadow, body, 63, 8, 63, body, shadow, 0, 0],
        vec![
            0, 0, shadow, body, body, shadow, shadow, body, body, shadow, 0, 0,
        ],
        // legs alternate per frame
        vec![
            l0, l1, shadow, body, body, body, body, body, body, shadow, l1, l0,
        ],
        vec![
            l1, shadow, body, body, shadow, body, body, shadow, body, body, shadow, l1,
        ],
        vec![
            l0, l1, shadow, body, body, body, body, body, body, shadow, l1, l0,
        ],
        // abdomen
        vec![
            l1, shadow, l0, shadow, body, body, body, body, shadow, l0, shadow, l1,
        ],
        vec![
            l0, l1, 0, shadow, shadow, body, body, shadow, shadow, 0, l1, l0,
        ],
        vec![0, 0, 0, 0, shadow, shadow, shadow, shadow, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, shadow, shadow, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let refs: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&refs, 12)
}

/// Robot worker — cute flying construction bot with propeller and glowing eye.
/// 10×10 sprite. Frame 0: propeller left. Frame 1: propeller right.
pub(super) fn make_robot_sprite(frame: u8) -> Image {
    // Palette: 25=deep blue, 26=mid blue, 27=periwinkle, 28=sky blue
    //          3=mid gray, 4=light gray, 5=silver, 6=pale, 8=white
    //          29=purple, 30=mid purple, 62=bright green (eye)
    let (p0, p1) = if frame == 0 { (4u8, 0u8) } else { (0u8, 4u8) }; // propeller alternation
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        // Row 0-1: Propeller blades (alternate per frame)
        &[0, 0, p0, 4, 5, 5, 4, p1, 0, 0],
        &[0, 0, 0, 0, 3, 3, 0, 0, 0, 0],
        // Row 2-3: Head (rounded top with antenna)
        &[0, 0, 1,26,27,27,26, 1, 0, 0],
        &[0, 1,26,27,62, 8,27,26, 1, 0],
        // Row 4-5: Body (boxy with panel detail)
        &[0, 1,25,26, 5, 5,26,25, 1, 0],
        &[1,25,26, 4, 3, 3, 4,26,25, 1],
        // Row 6-7: Lower body + arm stubs
        &[0, 1,25,26,27,27,26,25, 1, 0],
        &[0, 3, 1,25,26,26,25, 1, 3, 0],
        // Row 8-9: Thruster glow
        &[0, 0, 0, 1,29,29, 1, 0, 0, 0],
        &[0, 0, 0, 0,30,30, 0, 0, 0, 0],
    ];
    make_image_rect(pixels, 10, 10)
}

/// Big biter — heavier armored body with thicker legs and a spiked carapace.
/// Uses dark shadow colors for armored feel.
pub(super) fn make_big_biter_sprite(body: u8, shadow: u8, frame: u8) -> Image {
    let (l0, l1) = if frame == 0 {
        (shadow, 0u8)
    } else {
        (0u8, shadow)
    };
    let armor = 3u8; // mid gray for armor plates
    let pixels: Vec<Vec<u8>> = vec![
        // mandibles (wider)
        vec![shadow, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, shadow],
        vec![
            body, shadow, 0, shadow, shadow, shadow, shadow, shadow, shadow, 0, shadow, body,
        ],
        // head with armored plates + eyes
        vec![
            0, shadow, shadow, body, armor, 63, 8, 63, armor, body, shadow, 0,
        ],
        vec![
            0, shadow, body, armor, body, shadow, shadow, body, armor, body, shadow, 0,
        ],
        // armored thorax (thicker)
        vec![
            l0, shadow, armor, body, armor, body, body, armor, body, armor, shadow, l0,
        ],
        vec![
            l1, body, armor, armor, shadow, armor, armor, shadow, armor, armor, body, l1,
        ],
        vec![
            l0, shadow, armor, body, armor, body, body, armor, body, armor, shadow, l0,
        ],
        // abdomen with spikes
        vec![
            l1, shadow, body, shadow, body, body, body, body, shadow, body, shadow, l1,
        ],
        vec![
            l0, body, 0, shadow, armor, body, body, armor, shadow, 0, body, l0,
        ],
        vec![
            0, shadow, 0, 0, shadow, shadow, shadow, shadow, 0, 0, shadow, 0,
        ],
        vec![0, 0, 0, 0, shadow, body, body, shadow, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, shadow, shadow, 0, 0, 0, 0, 0],
    ];
    let refs: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&refs, 12)
}

/// Spitter — slender body with bulbous acid sac and thin legs.
/// Green-tinted with glowing projectile organ on the abdomen.
pub(super) fn make_spitter_sprite(body: u8, shadow: u8, frame: u8) -> Image {
    let (l0, l1) = if frame == 0 {
        (shadow, 0u8)
    } else {
        (0u8, shadow)
    };
    let glow = 62u8; // bright green for acid sac
    let pixels: Vec<Vec<u8>> = vec![
        // thin mandibles
        vec![0, 0, 0, 0, shadow, 0, 0, shadow, 0, 0, 0, 0],
        vec![0, 0, 0, shadow, body, shadow, shadow, body, shadow, 0, 0, 0],
        // small head with glowing eyes
        vec![0, 0, shadow, body, glow, 8, glow, 8, body, shadow, 0, 0],
        vec![
            0, 0, shadow, body, body, shadow, shadow, body, body, shadow, 0, 0,
        ],
        // thin thorax
        vec![
            l0, 0, shadow, body, body, body, body, body, body, shadow, 0, l0,
        ],
        vec![
            0, l1, shadow, body, shadow, body, body, shadow, body, shadow, l1, 0,
        ],
        // acid sac (glowing bulge)
        vec![
            l0, 0, shadow, body, glow, glow, glow, glow, body, shadow, 0, l0,
        ],
        vec![0, l1, 0, shadow, glow, 8, 8, glow, shadow, 0, l1, 0],
        vec![l0, 0, 0, shadow, glow, glow, glow, glow, shadow, 0, 0, l0],
        // tapered tail
        vec![0, 0, 0, 0, shadow, body, body, shadow, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, shadow, shadow, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ];
    let refs: Vec<&[u8]> = pixels.iter().map(|r| r.as_slice()).collect();
    make_image(&refs, 12)
}

/// 80x48 pixel art crashed colony ship "Horizon's Promise".
/// Hull is dark blue-gray (1-4), damage exposes warm brown internals (13-16),
/// cockpit glows blue-purple (29-31), scorch marks in dark tones.
pub(super) fn make_crashed_ship_sprite() -> Image {
    // Palette reference:
    // 1=deep shadow, 2=dark shadow, 3=mid gray, 4=light gray
    // 5=silver, 6=pale, 7=near-white, 8=specular
    // 13=dark brown, 14=mid brown, 15=warm brown, 16=tan
    // 17=dark coral, 18=coral, 19=amber, 20=peach
    // 25=deep blue, 26=mid blue, 27=periwinkle, 28=sky blue
    // 29=deep purple, 30=mid purple, 31=lilac
    // 37=dark gold, 38=gold, 39=yellow
    let _ = 0u8;
    let H = 2u8;
    let D = 1u8;
    let M = 3u8;
    let L = 4u8;
    let S = 5u8;
    let B = 14u8;
    let W = 15u8;
    let G = 27u8;
    let P = 30u8;
    let K = 29u8;
    let F = 18u8;
    let Y = 38u8;
    let R = 17u8;
    #[rustfmt::skip]
    let pixels: &[&[u8]] = &[
        // Row 0-3: antenna + top debris
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,G,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,G,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 4-5: broken top wing stub
        &[0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,H,H,H,D,D,D,D,H,H,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,H,H,D,D,H,H,H,M,M,H,H,H,H,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 6-7: top hull surface
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,M,M,M,M,H,H,M,L,M,M,H,H,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,D,H,H,M,M,L,L,M,M,M,M,L,L,S,L,M,H,M,L,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 8-9: engine block (rear) + hull top with damage
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,D,D,D,D,D,D,H,H,H,H,H,H,M,M,M,M,M,M,H,H,M,M,M,L,L,S,S,L,L,M,M,B,W,B,M,M,L,L,S,L,L,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,H,H,H,H,M,M,M,M,M,M,M,M,L,L,L,L,M,M,M,L,L,L,S,S,S,S,S,L,M,B,W,B,W,B,M,L,S,S,L,L,M,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 10-11: engine + main fuselage with exposed internals
        &[0,0,0,0,0,0,0,0,0,0,D,D,H,R,F,D,H,H,M,M,M,L,L,L,L,L,M,M,L,S,S,L,L,M,L,L,S,S,L,L,L,L,L,L,B,W,F,R,F,W,B,L,L,S,S,L,M,M,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,D,D,H,H,F,R,D,H,M,M,L,L,L,S,S,L,L,L,L,L,S,S,S,L,L,L,S,S,L,L,M,M,M,L,B,W,Y,F,R,Y,W,B,L,L,S,L,L,M,M,M,M,M,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 12-13: mid fuselage with cockpit area
        &[0,0,0,0,0,0,0,0,0,D,H,H,M,D,D,H,M,M,L,L,S,S,S,L,L,L,L,L,S,S,L,L,L,L,L,L,S,L,M,M,M,M,L,B,B,B,B,B,B,B,L,L,L,L,L,M,M,M,M,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,D,D,H,M,M,H,H,M,M,L,L,S,S,L,L,L,L,L,M,L,L,S,L,L,M,M,M,L,L,M,M,M,M,L,L,M,M,L,L,M,M,L,L,L,M,M,M,M,H,K,P,K,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 14-15: cockpit glow area
        &[0,0,0,0,0,0,0,0,D,H,M,M,M,M,M,M,L,L,S,S,L,L,M,M,M,M,M,M,L,L,M,M,M,M,M,L,M,M,H,H,M,L,L,M,M,M,M,M,M,M,L,M,M,M,H,K,K,P,G,P,K,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,D,H,M,M,M,L,L,M,L,S,S,L,L,M,M,M,H,M,M,M,L,M,M,H,H,M,M,M,M,H,H,H,M,L,L,M,M,H,H,M,M,L,L,M,H,H,K,P,G,G,G,G,P,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 16-17: main hull center
        &[0,0,0,0,0,0,0,D,H,M,M,M,L,L,S,L,L,S,L,L,M,M,H,H,H,M,M,M,M,M,H,H,H,H,M,M,H,H,D,H,M,M,L,M,H,H,H,H,M,L,M,M,H,K,P,G,G,28,G,G,P,K,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,M,M,L,L,S,S,L,L,L,L,M,M,H,H,D,H,H,M,M,M,H,H,D,D,H,H,H,H,D,D,H,M,L,L,M,H,D,D,H,M,L,M,M,H,K,P,G,28,28,28,G,P,K,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 18-19: hull plates + rivet lines
        &[0,0,0,0,0,0,0,D,H,M,L,L,S,S,L,L,M,M,M,M,H,H,D,D,D,H,M,M,H,H,D,D,D,D,H,H,D,D,D,H,M,L,L,M,H,D,D,H,M,L,M,M,H,K,P,G,G,28,G,G,P,K,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,M,L,S,S,L,L,M,M,M,M,H,H,D,D,D,H,H,M,M,H,D,D,D,D,D,D,D,D,D,H,M,M,L,L,M,H,H,D,H,M,L,M,M,H,K,P,G,G,G,G,P,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 20-21: hull bottom + damage hole
        &[0,0,0,0,0,0,0,D,H,M,L,L,L,M,M,M,M,H,H,H,D,D,D,H,H,M,M,H,H,D,D,D,D,D,D,D,D,H,M,M,L,L,M,M,H,H,H,M,M,L,M,M,M,H,K,P,P,P,K,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,M,M,L,L,M,M,H,H,H,D,D,D,D,H,H,M,M,M,H,D,D,B,W,B,D,D,D,H,M,M,L,L,L,M,M,H,H,M,M,L,L,M,M,M,M,H,H,K,K,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 22-23: lower fuselage
        &[0,0,0,0,0,0,0,D,H,M,M,M,L,L,M,H,H,D,D,D,D,H,H,M,M,M,H,D,D,B,W,Y,W,B,D,H,M,M,L,L,L,M,M,H,H,M,M,L,L,M,M,M,M,M,H,H,D,D,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,H,H,M,M,M,L,M,H,D,D,D,D,H,H,M,M,M,H,D,D,B,W,F,R,F,W,B,M,M,L,L,L,M,M,H,H,M,M,L,L,M,M,M,H,H,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 24-25: hull shadow + lower wing stub
        &[0,0,0,0,0,0,0,0,D,H,H,M,M,M,M,H,D,D,D,H,H,M,M,M,H,H,D,D,D,B,W,B,D,D,H,M,L,L,L,M,M,H,H,D,M,L,L,M,M,M,H,H,D,D,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,D,D,H,H,M,M,M,H,D,D,H,H,M,M,M,H,H,D,D,D,D,D,D,D,D,H,M,M,L,L,M,M,H,H,D,D,M,L,M,M,H,H,D,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 26-27: bottom wing + hull underside
        &[0,0,0,0,0,0,0,0,0,D,D,H,H,M,M,H,D,H,H,M,M,M,H,H,D,D,D,D,D,D,D,D,H,M,M,L,L,M,M,H,H,D,D,D,M,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,D,D,H,H,M,H,H,H,M,M,H,H,H,D,D,0,0,0,0,0,0,H,M,M,L,L,M,M,H,H,D,D,0,D,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 28-29: lower hull taper + broken bits
        &[0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,H,H,M,M,H,H,D,D,0,0,0,0,0,0,0,0,H,M,L,L,M,M,H,D,D,D,0,0,D,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,M,M,H,H,D,D,0,0,0,0,0,0,0,0,H,M,M,L,M,M,H,D,D,0,0,0,0,0,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 30-31: scattered debris below
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,H,H,H,D,D,0,0,0,0,0,0,0,0,0,H,M,M,L,M,M,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,D,D,0,0,0,0,0,0,0,0,0,0,D,D,H,D,D,0,0,0,0,0,0,0,0,0,H,M,M,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 32-35: ground debris / scorch marks
        &[0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,D,D,D,0,0,0,0,D,D,0,0,0,H,M,M,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,D,0,0,0,0,D,0,0,0,0,0,0,0,0,D,0,0,0,0,D,H,D,0,0,0,0,H,H,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,D,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,D,D,0,0,0,0,0,0,0,D,D,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,D,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 36-39: more scattered debris
        &[0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,H,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,D,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        // Row 40-47: empty padding
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
        &[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],
    ];
    make_image_rect(pixels, 80, 48)
}

// ===========================================================================
// Additional building sprites
// ===========================================================================

/// FORGE avatar — cute round robot AI face (24×24).
/// Purple body, big white eyes with pupils, antenna with glowing tip, smile.
/// Frame 0: happy (open eyes). Frame 1: blink (^_^ face).
pub(super) fn make_forge_avatar_sprite(frame: u8) -> Image {
    #[rustfmt::skip]
    let base: Vec<Vec<u8>> = vec![
        // Row 0-2: antenna
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,19, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,30, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        // Row 3-4: top of head
        vec![0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1,31,31,32,32,32,32,32,32,32,31,31, 1, 0, 0, 0, 0, 0, 0],
        // Row 5-6: forehead
        vec![0, 0, 0, 0, 1,31,32,32,32,32,32,32,32,32,32,32,32,31, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 1,31,32,32,32,32,32,32,32,32,32,32,32,32,32,31, 1, 0, 0, 0, 0],
        // Row 7-10: eyes region
        vec![0, 0, 0, 1,30,32,32, 7, 7, 8, 7,32,32, 7, 7, 8, 7,32,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32, 7, 8, 8, 8, 8, 7,32, 8, 8, 8, 8, 7,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32, 7, 8,29, 1, 8, 7,32, 8,29, 1, 8, 7,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32, 7, 8, 1, 1, 8, 7,32, 8, 1, 1, 8, 7,30, 1, 0, 0, 0, 0],
        // Row 11-12: cheeks + blush
        vec![0, 0, 0, 1,30,32,32, 7, 7, 7, 7,32,32, 7, 7, 7, 7,32,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32,23,32,32,32,32,32,32,32,32,32,32,23,30, 1, 0, 0, 0, 0],
        // Row 13-14: mouth (happy smile)
        vec![0, 0, 0, 1,30,32,32,32,32, 1, 1, 1, 1, 1, 1,32,32,32,30, 1, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30,32,32,32,32,32, 1,30,30, 1,32,32,32,32,30, 1, 0, 0, 0, 0],
        // Row 15-17: chin + body top
        vec![0, 0, 0, 0, 1,31,32,32,32,32,32,32,32,32,32,32,32,31, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1,31,31,31,30,30,30,30,30,31,31,31, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 1,30,30,29,29,29,29,29,30,30, 1, 0, 0, 0, 0, 0, 0, 0],
        // Row 18-19: body middle with chest light
        vec![0, 0, 0, 0, 0, 1,30,29,29,29, 5, 6, 5,29,29,29,30, 1, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 1,30,29,29, 5, 6, 8, 6, 5,29,29,30, 1, 0, 0, 0, 0, 0, 0],
        // Row 20-21: arms
        vec![0, 0, 0, 0, 1,30,29, 1,29,29, 5, 6, 5,29,29, 1,29,30, 1, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 1,30, 1, 0, 0, 1,29,29,29,29,29, 1, 0, 0, 1,30, 1, 0, 0, 0, 0],
        // Row 22-23: feet
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1,29,29,29,29, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0],
    ];

    let refs: Vec<&[u8]> = base.iter().map(|r| r.as_slice()).collect();
    let mut img = make_image_rect(&refs, 24, 24);

    if frame == 1 {
        // Blink frame: replace eye region with closed ^_^ lines.
        for y in 8..11u32 {
            for x in 7..11u32 {
                img.set_pixel(x, y, pal_color(32));
            }
            for x in 13..17u32 {
                img.set_pixel(x, y, pal_color(32));
            }
        }
        // Draw ^_^ (left eye)
        img.set_pixel(7, 9, pal_color(1));
        img.set_pixel(8, 8, pal_color(1));
        img.set_pixel(9, 9, pal_color(1));
        // Draw ^_^ (right eye)
        img.set_pixel(13, 9, pal_color(1));
        img.set_pixel(14, 8, pal_color(1));
        img.set_pixel(15, 9, pal_color(1));
    }

    img
}
