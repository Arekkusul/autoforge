//! Unified texture atlas for all game sprites.
//!
//! Packs all individual sprites into a single 512×512 texture at startup.
//! Drawing uses `source` rectangles to select sub-regions, eliminating
//! texture switches between draw calls. macroquad auto-batches draws that
//! use the same texture, so this reduces hundreds of draw calls to ~1-3.
//!
//! # How it works
//!
//! 1. At startup, each sprite is generated as an `Image` (same as before).
//! 2. All images are blitted into a single 512×512 `Image` at grid positions.
//! 3. The combined image becomes one `Texture2D`.
//! 4. Each sprite's position in the atlas is stored as a `Rect` (source UV).
//! 5. All rendering uses `draw_texture_ex` with `source: Some(rect)`.

use macroquad::prelude::*;

/// A sprite's location within the unified atlas texture.
#[derive(Clone, Copy, Debug)]
pub struct SpriteRect {
    /// Source rectangle in pixel coordinates within the atlas.
    pub source: Rect,
}

/// The unified texture atlas containing all game sprites.
///
/// All rendering should use `atlas.draw()` instead of individual texture draws.
pub struct UnifiedAtlas {
    /// The single GPU texture containing all sprites.
    pub texture: Texture2D,
    /// Atlas dimensions.
    pub width: u16,
    pub height: u16,
    /// Current packing position (for sequential packing at startup).
    next_x: u16,
    next_y: u16,
    row_height: u16,
    /// The backing image (only used during construction).
    image: Option<Image>,
}

impl UnifiedAtlas {
    /// Creates a new empty atlas of the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let image = Image::gen_image_color(width, height, Color::new(0.0, 0.0, 0.0, 0.0));
        Self {
            texture: Texture2D::empty(), // placeholder, finalized later
            width,
            height,
            next_x: 0,
            next_y: 0,
            row_height: 0,
            image: Some(image),
        }
    }

    /// Packs a sprite image into the atlas and returns its source rect.
    ///
    /// Uses a simple row-based packing algorithm (fast, good enough for ~40 sprites).
    pub fn pack(&mut self, sprite: &Image) -> SpriteRect {
        let sw = sprite.width() as u16;
        let sh = sprite.height() as u16;

        // Check if sprite fits in current row.
        if self.next_x + sw > self.width {
            // Move to next row.
            self.next_y += self.row_height + 1; // 1px padding
            self.next_x = 0;
            self.row_height = 0;
        }

        let x = self.next_x;
        let y = self.next_y;

        // Blit sprite pixels into atlas image.
        if let Some(ref mut atlas_img) = self.image {
            for py in 0..sh {
                for px in 0..sw {
                    let color = sprite.get_pixel((px as u32), (py as u32));
                    atlas_img.set_pixel((x + px) as u32, (y + py) as u32, color);
                }
            }
        }

        self.next_x += sw + 1; // 1px padding between sprites
        self.row_height = self.row_height.max(sh);

        SpriteRect {
            source: Rect::new(x as f32, y as f32, sw as f32, sh as f32),
        }
    }

    /// Finalizes the atlas — uploads the combined image to GPU as a single texture.
    ///
    /// Call this AFTER all sprites have been packed.
    pub fn finalize(&mut self) {
        if let Some(image) = self.image.take() {
            self.texture = Texture2D::from_image(&image);
            self.texture.set_filter(FilterMode::Nearest);
        }
    }

    /// Draws a sprite from the atlas at the given world position.
    ///
    /// This is the primary draw function — all sprites should use this.
    /// macroquad batches consecutive draws of the same texture, so calling
    /// this 1000 times results in ~1 actual GPU draw call.
    #[inline]
    pub fn draw(&self, sprite: &SpriteRect, x: f32, y: f32, dest_w: f32, dest_h: f32, color: Color) {
        draw_texture_ex(
            &self.texture,
            x,
            y,
            color,
            DrawTextureParams {
                source: Some(sprite.source),
                dest_size: Some(Vec2::new(dest_w, dest_h)),
                ..Default::default()
            },
        );
    }

    /// Draws a sprite from the atlas with rotation.
    #[inline]
    pub fn draw_rotated(
        &self,
        sprite: &SpriteRect,
        x: f32,
        y: f32,
        dest_size: f32,
        rotation: f32,
        color: Color,
    ) {
        draw_texture_ex(
            &self.texture,
            x,
            y,
            color,
            DrawTextureParams {
                source: Some(sprite.source),
                dest_size: Some(Vec2::splat(dest_size)),
                rotation,
                pivot: Some(Vec2::new(x + dest_size * 0.5, y + dest_size * 0.5)),
                ..Default::default()
            },
        );
    }
}
