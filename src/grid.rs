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

    // -----------------------------------------------------------------------
    // Pathfinding
    // -----------------------------------------------------------------------

    /// Whether a tile can be walked across by a ground unit (enemy or bot).
    ///
    /// Impassable terrain (water, cliff) blocks movement. A tile with a building
    /// on it is treated as passable only when `allow_buildings` is set — enemies
    /// path *around* structures, but a debug/overview path may cross them.
    #[allow(dead_code)] // Grid navigation API used by pathfinding and its tests.
    pub fn is_walkable(&self, pos: GridPos, allow_buildings: bool) -> bool {
        match self.get_tile(pos) {
            None => false,
            Some(tile) => {
                let terrain_ok = !matches!(tile.terrain, Terrain::Water | Terrain::Cliff);
                let building_ok = allow_buildings || tile.building.is_none();
                terrain_ok && building_ok
            }
        }
    }

    /// Finds a shortest 4-connected path from `start` to `goal` using breadth-first
    /// search, returning the sequence of tiles from `start` to `goal` inclusive.
    ///
    /// Returns `None` if no path exists or either endpoint is off the grid. Search
    /// is bounded by `max_steps` explored tiles to keep worst-case cost predictable
    /// on the large world grid. The `goal` tile is always considered reachable even
    /// if it holds a building (so enemies can target the structure they attack).
    #[allow(dead_code)] // Grid navigation API used by tests and future AI routing.
    pub fn find_path(
        &self,
        start: GridPos,
        goal: GridPos,
        allow_buildings: bool,
        max_steps: usize,
    ) -> Option<Vec<GridPos>> {
        if !self.in_bounds(start) || !self.in_bounds(goal) {
            return None;
        }
        if start == goal {
            return Some(vec![start]);
        }

        let width = self.width;
        let cell = |p: GridPos| (p.y * width + p.x) as usize;
        let mut came_from: std::collections::HashMap<usize, GridPos> =
            std::collections::HashMap::new();
        let mut visited: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut queue: std::collections::VecDeque<GridPos> = std::collections::VecDeque::new();

        visited.insert(cell(start));
        queue.push_back(start);
        let mut explored = 0usize;

        while let Some(current) = queue.pop_front() {
            explored += 1;
            if explored > max_steps {
                return None;
            }
            for dir in Direction::all() {
                let next = current.neighbor(dir);
                if !self.in_bounds(next) {
                    continue;
                }
                let key = cell(next);
                if visited.contains(&key) {
                    continue;
                }
                // The goal is reachable regardless of a building on it.
                let passable = next == goal || self.is_walkable(next, allow_buildings);
                if !passable {
                    continue;
                }
                visited.insert(key);
                came_from.insert(key, current);
                if next == goal {
                    return Some(Self::reconstruct_path(&came_from, start, goal, cell));
                }
                queue.push_back(next);
            }
        }
        None
    }

    /// Walks the `came_from` chain backward from `goal` to `start`, producing the
    /// forward-ordered path.
    fn reconstruct_path(
        came_from: &std::collections::HashMap<usize, GridPos>,
        start: GridPos,
        goal: GridPos,
        cell: impl Fn(GridPos) -> usize,
    ) -> Vec<GridPos> {
        let mut path = vec![goal];
        let mut current = goal;
        while current != start {
            match came_from.get(&cell(current)) {
                Some(&prev) => {
                    path.push(prev);
                    current = prev;
                }
                None => break,
            }
        }
        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_grid() -> Grid {
        Grid::new(8, 8)
    }

    #[test]
    fn straight_path_on_open_grid() {
        let grid = open_grid();
        let path = grid
            .find_path(GridPos::new(0, 0), GridPos::new(3, 0), false, 1000)
            .expect("path exists on open grid");
        assert_eq!(path.first(), Some(&GridPos::new(0, 0)));
        assert_eq!(path.last(), Some(&GridPos::new(3, 0)));
        // Manhattan-optimal: 4 tiles including both endpoints.
        assert_eq!(path.len(), 4);
    }

    #[test]
    fn path_routes_around_water_wall() {
        let mut grid = open_grid();
        // Build a vertical water wall at x=2 for rows 0..=6, leaving a gap at y=7.
        for y in 0..=6 {
            grid.get_tile_mut(GridPos::new(2, y)).unwrap().terrain = Terrain::Water;
        }
        let path = grid
            .find_path(GridPos::new(0, 0), GridPos::new(4, 0), false, 10_000)
            .expect("path should route around the wall");
        // Never steps on a water tile.
        for step in &path {
            assert!(grid.is_walkable(*step, false), "stepped on {step:?}");
        }
    }

    #[test]
    fn no_path_when_fully_blocked() {
        let mut grid = open_grid();
        for y in 0..8 {
            grid.get_tile_mut(GridPos::new(2, y)).unwrap().terrain = Terrain::Cliff;
        }
        assert!(grid
            .find_path(GridPos::new(0, 0), GridPos::new(5, 0), false, 10_000)
            .is_none());
    }

    #[test]
    fn same_start_and_goal_is_trivial() {
        let grid = open_grid();
        let path = grid
            .find_path(GridPos::new(1, 1), GridPos::new(1, 1), false, 10)
            .unwrap();
        assert_eq!(path, vec![GridPos::new(1, 1)]);
    }

    #[test]
    fn goal_reachable_even_with_building_on_it() {
        let mut grid = open_grid();
        let goal = GridPos::new(3, 0);
        grid.get_tile_mut(goal).unwrap().building = Some(BuildingId {
            index: 0,
            generation: 0,
        });
        let path = grid.find_path(GridPos::new(0, 0), goal, false, 1000);
        assert!(path.is_some(), "enemies must reach a building they target");
    }
}
