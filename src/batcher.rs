//! Production-grade sprite batcher using macroquad's Mesh API.
//!
//! Instead of calling `draw_texture_ex` hundreds of times (each potentially a separate
//! draw call with texture switches), this module collects all sprite draws into a
//! single vertex buffer (Mesh) and renders them in ONE GPU draw call.
//!
//! # How it works
//!
//! 1. Each frame, call `batch.begin()` to clear the buffer.
//! 2. For each sprite, call `batch.add_sprite(...)` — this appends 4 vertices + 6 indices.
//! 3. At end of frame, call `batch.flush()` — uploads the mesh and draws in one call.
//!
//! This is the technique used by LibGDX, MonoGame, Love2D, and all production 2D engines.
//! With a single texture atlas, ALL world rendering becomes 1 GPU draw call.

use macroquad::prelude::*;

/// Maximum sprites per batch before an automatic flush.
const MAX_SPRITES_PER_BATCH: usize = 8192;

/// A production sprite batcher that minimizes draw calls.
pub struct SpriteBatcher {
    /// Vertex buffer (4 vertices per sprite: position + UV + color).
    vertices: Vec<Vertex>,
    /// Index buffer (6 indices per sprite: 2 triangles).
    indices: Vec<u16>,
    /// The texture all sprites in this batch use (must be same texture for batching).
    /// Texture2D is Copy in macroquad (it's just an ID handle).
    texture: Texture2D,
    /// How many sprites are currently in the buffer.
    sprite_count: usize,
}

impl SpriteBatcher {
    /// Creates a new batcher with pre-allocated buffers.
    /// Creates a new batcher bound to a texture (the atlas).
    pub fn new(texture: Texture2D) -> Self {
        Self {
            vertices: Vec::with_capacity(MAX_SPRITES_PER_BATCH * 4),
            indices: Vec::with_capacity(MAX_SPRITES_PER_BATCH * 6),
            texture,
            sprite_count: 0,
        }
    }

    /// Begins a new batch frame. Clears the buffers.
    pub fn begin(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.sprite_count = 0;
    }

    /// Adds a sprite (textured quad) to the batch.
    ///
    /// - `pos`: world-space top-left corner
    /// - `size`: width and height in world units
    /// - `source`: UV rectangle within the atlas texture (in pixels)
    /// - `color`: tint color (WHITE = no tint)
    /// - `tex_size`: total atlas texture size (for UV normalization)
    #[inline]
    pub fn add_sprite(
        &mut self,
        pos: Vec2,
        size: Vec2,
        source: Rect,
        color: Color,
        tex_size: Vec2,
    ) {
        if self.sprite_count >= MAX_SPRITES_PER_BATCH {
            self.flush(); // auto-flush if buffer is full
        }

        let base_idx = self.vertices.len() as u16;

        // Normalize UVs from pixel coordinates to [0,1].
        let u0 = source.x / tex_size.x;
        let v0 = source.y / tex_size.y;
        let u1 = (source.x + source.w) / tex_size.x;
        let v1 = (source.y + source.h) / tex_size.y;

        let x0 = pos.x;
        let y0 = pos.y;
        let x1 = pos.x + size.x;
        let y1 = pos.y + size.y;

        // 4 vertices (top-left, top-right, bottom-right, bottom-left).
        self.vertices.push(Vertex::new(x0, y0, 0.0, u0, v0, color));
        self.vertices.push(Vertex::new(x1, y0, 0.0, u1, v0, color));
        self.vertices.push(Vertex::new(x1, y1, 0.0, u1, v1, color));
        self.vertices.push(Vertex::new(x0, y1, 0.0, u0, v1, color));

        // 6 indices (2 triangles).
        self.indices.push(base_idx);
        self.indices.push(base_idx + 1);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx);
        self.indices.push(base_idx + 2);
        self.indices.push(base_idx + 3);

        self.sprite_count += 1;
    }

    /// Adds a colored rectangle (no texture) to the batch.
    ///
    /// Uses a 1x1 white pixel region of the atlas for solid fills.
    #[inline]
    pub fn add_rect(&mut self, pos: Vec2, size: Vec2, color: Color, tex_size: Vec2) {
        // Use the very first pixel of the atlas (should be opaque white or we bake a white pixel).
        let source = Rect::new(0.0, 0.0, 1.0, 1.0);
        self.add_sprite(pos, size, source, color, tex_size);
    }

    /// Flushes all buffered sprites to the GPU in ONE draw call.
    pub fn flush(&mut self) {
        if self.sprite_count == 0 {
            return;
        }

        let mesh = Mesh {
            vertices: self.vertices.clone(),
            indices: self.indices.clone(),
            texture: Some(self.texture.clone()),
        };
        draw_mesh(&mesh);

        self.vertices.clear();
        self.indices.clear();
        self.sprite_count = 0;
    }

    /// Returns the number of sprites currently buffered.
    pub fn count(&self) -> usize {
        self.sprite_count
    }
}
