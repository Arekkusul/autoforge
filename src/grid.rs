//! World grid: tile storage, spatial item index, and coordinate conversion.
//!
//! The grid is a flat `Vec<Tile>` indexed by `y * width + x` for cache-friendly
//! O(1) lookups. A parallel spatial index tracks which items occupy each tile.

use macroquad::prelude::Vec2;
use serde::{Deserialize, Serialize};

use crate::constants::TILE_SIZE;
use crate::types::*;

/// A single tile in the world grid.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    /// The terrain type (grass, water, forest, etc.).
    pub terrain: Terrain,
    /// Natural ore deposit on this tile, if any.
    pub deposit: Option<OreDeposit>,
    /// Remaining ore in the deposit (depletes over time when mined).
    pub ore_amount: u32,
    /// Whether this tile is the top-left origin of a 2×2 ore node.
    /// Only origin tiles render the large rock sprite.
    pub ore_origin: bool,
    /// Building placed on this tile, if any.
    pub building: Option<BuildingId>,
    /// Current pollution level on this tile.
    pub pollution: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            terrain: Terrain::Grass,
            deposit: None,
            ore_amount: 0,
            ore_origin: false,
            building: None,
            pollution: 0.0,
        }
    }
}

/// The world grid. All tile data is stored in flat vectors for cache efficiency.
pub struct Grid {
    /// Grid width in tiles.
    pub width: i32,
    /// Grid height in tiles.
    pub height: i32,
    /// Tile data, indexed by `y * width + x`.
    tiles: Vec<Tile>,
    /// Spatial index: which [`ItemId`]s are on each tile. Same indexing as `tiles`.
    items_on_tile: Vec<Vec<ItemId>>,
}

impl Grid {
    /// Creates a new grid filled with default grass tiles.
    pub fn new(width: i32, height: i32) -> Self {
        let count = (width * height) as usize;
        Self {
            width,
            height,
            tiles: (0..count).map(|_| Tile::default()).collect(),
            items_on_tile: vec![Vec::new(); count],
        }
    }

    /// Converts a [`GridPos`] to a flat array index. Returns `None` if out of bounds.
    #[inline]
    fn index(&self, pos: GridPos) -> Option<usize> {
        if pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height {
            Some((pos.y * self.width + pos.x) as usize)
        } else {
            None
        }
    }

    /// Returns whether the position is within grid bounds.
    #[inline]
    pub fn in_bounds(&self, pos: GridPos) -> bool {
        pos.x >= 0 && pos.x < self.width && pos.y >= 0 && pos.y < self.height
    }

    /// Returns a reference to the tile at `pos`, or `None` if out of bounds.
    #[inline]
    pub fn get_tile(&self, pos: GridPos) -> Option<&Tile> {
        self.index(pos).map(|i| &self.tiles[i])
    }

    /// Returns a mutable reference to the tile at `pos`, or `None` if out of bounds.
    #[inline]
    pub fn get_tile_mut(&mut self, pos: GridPos) -> Option<&mut Tile> {
        self.index(pos).map(|i| &mut self.tiles[i])
    }

    /// Direct access to the tile array for iteration (e.g., pollution diffusion).
    pub fn tiles(&self) -> &[Tile] {
        &self.tiles
    }

    /// Direct mutable access to the tile array.
    pub fn tiles_mut(&mut self) -> &mut [Tile] {
        &mut self.tiles
    }

    /// Returns the flat index for a position (for direct tile array access).
    #[inline]
    pub fn pos_to_index(&self, pos: GridPos) -> Option<usize> {
        self.index(pos)
    }

    /// Converts a flat index back to a [`GridPos`].
    #[inline]
    pub fn index_to_pos(&self, idx: usize) -> GridPos {
        GridPos {
            x: (idx as i32) % self.width,
            y: (idx as i32) / self.width,
        }
    }

    // -----------------------------------------------------------------------
    // Spatial item index
    // -----------------------------------------------------------------------

    /// Registers an item as present on the given tile.
    pub fn add_item_to_tile(&mut self, pos: GridPos, id: ItemId) {
        if let Some(i) = self.index(pos) {
            self.items_on_tile[i].push(id);
        }
    }

    /// Removes an item from the given tile's spatial index.
    pub fn remove_item_from_tile(&mut self, pos: GridPos, id: ItemId) {
        if let Some(i) = self.index(pos) {
            self.items_on_tile[i].retain(|&existing| existing != id);
        }
    }

    /// Returns a slice of item IDs present on the given tile.
    pub fn items_at(&self, pos: GridPos) -> &[ItemId] {
        match self.index(pos) {
            Some(i) => &self.items_on_tile[i],
            None => &[],
        }
    }

    /// Returns a mutable reference to the item list for a tile.
    pub fn items_at_mut(&mut self, pos: GridPos) -> Option<&mut Vec<ItemId>> {
        self.index(pos).map(|i| &mut self.items_on_tile[i])
    }

    // -----------------------------------------------------------------------
    // Coordinate conversions
    // -----------------------------------------------------------------------

    /// Converts world-space coordinates to the grid tile position containing that point.
    pub fn world_to_grid(world: Vec2) -> GridPos {
        GridPos {
            x: (world.x / TILE_SIZE).floor() as i32,
            y: (world.y / TILE_SIZE).floor() as i32,
        }
    }

    /// Converts a grid position to the world-space top-left corner of that tile.
    pub fn grid_to_world(pos: GridPos) -> Vec2 {
        Vec2::new(pos.x as f32 * TILE_SIZE, pos.y as f32 * TILE_SIZE)
    }

    /// Converts a grid position to the world-space center of that tile.
    pub fn grid_to_world_center(pos: GridPos) -> Vec2 {
        Vec2::new(
            pos.x as f32 * TILE_SIZE + TILE_SIZE * 0.5,
            pos.y as f32 * TILE_SIZE + TILE_SIZE * 0.5,
        )
    }
}
