//! Pre-allocated item pool with generational indices.
//!
//! Items are the physical objects that move along belts and sit in machine buffers.
//! The [`ItemPool`] avoids per-item heap allocations by using a fixed-capacity arena
//! with a free-list for O(1) spawn/despawn and generational indices to detect stale
//! handles.

use serde::{Deserialize, Serialize};

use crate::types::*;

/// A live item in the world (on a belt, waiting to be picked up, etc.).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    /// What resource this item represents.
    pub resource: Resource,
    /// The grid tile this item is currently on.
    pub pos: GridPos,
    /// Movement progress across the current tile, in `[0.0, 1.0)`.
    ///
    /// At 0.0 the item just entered the tile; at ~1.0 it's ready to move to the next.
    pub progress: f32,
    /// Whether this slot is occupied (false = free-list slot).
    pub alive: bool,
    /// Generation counter matching the pool's slot generation.
    pub generation: u32,
}

/// Arena-style pool for items. Avoids per-item heap allocation.
///
/// Uses a free-list for O(1) spawn/despawn and generational indices so that
/// stale [`ItemId`] handles are safely rejected.
pub struct ItemPool {
    items: Vec<Item>,
    generations: Vec<u32>,
    free_list: Vec<u32>,
}

impl ItemPool {
    /// Creates a new pool with the given initial capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            items: Vec::with_capacity(capacity),
            generations: Vec::with_capacity(capacity),
            free_list: Vec::new(),
        }
    }

    /// Spawns a new item at the given position. Returns its [`ItemId`] handle.
    pub fn spawn(&mut self, resource: Resource, pos: GridPos) -> ItemId {
        if let Some(index) = self.free_list.pop() {
            let i = index as usize;
            self.generations[i] += 1;
            self.items[i] = Item {
                resource,
                pos,
                progress: 0.0,
                alive: true,
                generation: self.generations[i],
            };
            ItemId {
                index,
                generation: self.generations[i],
            }
        } else {
            let index = self.items.len() as u32;
            let generation = 0;
            self.items.push(Item {
                resource,
                pos,
                progress: 0.0,
                alive: true,
                generation,
            });
            self.generations.push(generation);
            ItemId { index, generation }
        }
    }

    /// Despawns an item, returning its slot to the free list.
    ///
    /// Increments the generation so any remaining [`ItemId`] handles become stale.
    pub fn despawn(&mut self, id: ItemId) {
        if let Some(item) = self.get_mut(id) {
            item.alive = false;
            self.free_list.push(id.index);
        }
    }

    /// Returns a reference to the item if the handle is still valid.
    pub fn get(&self, id: ItemId) -> Option<&Item> {
        let i = id.index as usize;
        if i < self.items.len()
            && self.generations[i] == id.generation
            && self.items[i].alive
        {
            Some(&self.items[i])
        } else {
            None
        }
    }

    /// Returns a mutable reference to the item if the handle is still valid.
    pub fn get_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        let i = id.index as usize;
        if i < self.items.len()
            && self.generations[i] == id.generation
            && self.items[i].alive
        {
            Some(&mut self.items[i])
        } else {
            None
        }
    }

    /// Iterates over all alive items with their IDs.
    pub fn iter(&self) -> impl Iterator<Item = (ItemId, &Item)> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.alive)
            .map(move |(i, item)| {
                (
                    ItemId {
                        index: i as u32,
                        generation: self.generations[i],
                    },
                    item,
                )
            })
    }

    /// Collects all alive item IDs into a vec (for iteration that needs mutation).
    pub fn alive_ids(&self) -> Vec<ItemId> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.alive)
            .map(|(i, _)| ItemId {
                index: i as u32,
                generation: self.generations[i],
            })
            .collect()
    }
}
