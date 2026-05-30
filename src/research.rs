//! Research tree and lab processing.
//!
//! Labs consume science packs to unlock technologies. Each technology requires
//! a certain number of science packs and grants bonuses or unlocks when complete.

use serde::{Deserialize, Serialize};

use crate::building::Buildings;
use crate::constants::*;
use crate::types::*;

/// A technology that can be researched.
#[derive(Clone, Debug)]
pub struct Technology {
    /// Display name.
    pub name: &'static str,
    /// Science packs required per unit of research.
    pub science_cost: &'static [Resource],
    /// Number of science pack sets needed to complete.
    pub units_needed: u32,
    /// What this tech unlocks (description for UI).
    pub description: &'static str,
    /// Prerequisite tech indices (must be completed first).
    pub prerequisites: &'static [usize],
}

/// All technologies in the game.
pub static TECHNOLOGIES: &[Technology] = &[
    // === TIER 1: Red Science ===
    // 0: Automation
    Technology {
        name: "Automation",
        science_cost: &[Resource::ScienceRed],
        units_needed: 10,
        description: "Unlocks Assembler T1",
        prerequisites: &[],
    },
    // 1: Logistics 1
    Technology {
        name: "Logistics 1",
        science_cost: &[Resource::ScienceRed],
        units_needed: 10,
        description: "Unlocks faster belts",
        prerequisites: &[],
    },
    // 2: Military 1
    Technology {
        name: "Military 1",
        science_cost: &[Resource::ScienceRed],
        units_needed: 10,
        description: "Unlocks Gun Turret + Walls",
        prerequisites: &[],
    },
    // 3: Stone Walls
    Technology {
        name: "Stone Walls",
        science_cost: &[Resource::ScienceRed],
        units_needed: 10,
        description: "Unlocks Wall building",
        prerequisites: &[2],
    },
    // 4: Steel Processing
    Technology {
        name: "Steel Processing",
        science_cost: &[Resource::ScienceRed],
        units_needed: 15,
        description: "Unlocks Steel Plate smelting",
        prerequisites: &[0],
    },
    // 5: Fast Inserter
    Technology {
        name: "Fast Inserter",
        science_cost: &[Resource::ScienceRed],
        units_needed: 15,
        description: "Unlocks Fast Inserter",
        prerequisites: &[0],
    },
    // 6: Electronics
    Technology {
        name: "Electronics",
        science_cost: &[Resource::ScienceRed],
        units_needed: 15,
        description: "Unlocks Green Circuit production",
        prerequisites: &[0],
    },
    // === TIER 2: Red + Green Science ===
    // 7: Advanced Electronics
    Technology {
        name: "Advanced Electronics",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 20,
        description: "Unlocks Red Circuit",
        prerequisites: &[6],
    },
    // 8: Engine
    Technology {
        name: "Engine",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 20,
        description: "Unlocks Engine Unit",
        prerequisites: &[4],
    },
    // 9: Logistics 2
    Technology {
        name: "Logistics 2",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 20,
        description: "Unlocks Red Belts (2x speed)",
        prerequisites: &[1, 6],
    },
    // 10: Military 2
    Technology {
        name: "Military 2",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 20,
        description: "Unlocks Piercing Ammo + Grenades",
        prerequisites: &[2, 4],
    },
    // 11: Solar Energy
    Technology {
        name: "Solar Energy",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 25,
        description: "Unlocks Solar Panel + Accumulator",
        prerequisites: &[6],
    },
    // 12: Steel Furnace
    Technology {
        name: "Steel Furnace",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 20,
        description: "Unlocks Steel Furnace (1.5x speed)",
        prerequisites: &[4],
    },
    // 13: Stack Inserter
    Technology {
        name: "Stack Inserter",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 25,
        description: "Unlocks Stack Inserter (moves 4 items)",
        prerequisites: &[5, 6],
    },
    // === TIER 3: Needs Blue/Military Science ===
    // 14: Laser Turret
    Technology {
        name: "Laser Turret",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 30,
        description: "Unlocks Laser Turret (uses power, no ammo)",
        prerequisites: &[7, 10],
    },
    // 15: Assembler T2
    Technology {
        name: "Assembler T2",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 25,
        description: "Unlocks Assembler Tier 2 (1.33x speed)",
        prerequisites: &[6],
    },
    // 16: Mining Productivity 1
    Technology {
        name: "Mining Productivity 1",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 50,
        description: "Miners +10% output chance",
        prerequisites: &[0],
    },
    // 17: Chemical Processing
    Technology {
        name: "Chemical Processing",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 30,
        description: "Unlocks Chemical Plant (plastics, sulfur, batteries)",
        prerequisites: &[7],
    },
    // 18: Logistics 3
    Technology {
        name: "Logistics 3",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 40,
        description: "Unlocks Blue Belts (3x speed)",
        prerequisites: &[9],
    },
    // 19: Nuclear Power
    Technology {
        name: "Nuclear Power",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 60,
        description: "Unlocks Nuclear Reactor (40000 kW!)",
        prerequisites: &[7, 11],
    },
    // 20: Rocketry
    Technology {
        name: "Rocketry",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 100,
        description: "Unlocks Rocket Silo + Rocket Parts",
        prerequisites: &[7, 14],
    },
    // 21: Robot Logistics
    Technology {
        name: "Robot Logistics",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 50,
        description: "Unlocks Roboport + Construction Bots",
        prerequisites: &[8, 7],
    },
    // 22: Assembler T3
    Technology {
        name: "Assembler T3",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 40,
        description: "Unlocks Assembler Tier 3 (2x speed)",
        prerequisites: &[15],
    },
    // 23: Electric Furnace
    Technology {
        name: "Electric Furnace",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 30,
        description: "Unlocks Electric Furnace (2x speed, no fuel needed)",
        prerequisites: &[12],
    },
    // 24: Mining Productivity 2 (infinite-style, expensive)
    Technology {
        name: "Mining Productivity 2",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 100,
        description: "Miners +25% output (stacks with previous)",
        prerequisites: &[16],
    },
    // 25: Turret Damage 2
    Technology {
        name: "Turret Damage 2",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 80,
        description: "All turrets deal +50% damage",
        prerequisites: &[14],
    },
    // 26: Belt Speed 2
    Technology {
        name: "Belt Speed 2",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen],
        units_needed: 60,
        description: "All belts move items 25% faster",
        prerequisites: &[18],
    },
    // 27: FORGE Consciousness (endgame research — very expensive)
    Technology {
        name: "FORGE Consciousness",
        science_cost: &[Resource::ScienceRed, Resource::ScienceGreen, Resource::ScienceBlue],
        units_needed: 200,
        description: "Restore full consciousness. The final goal.",
        prerequisites: &[19, 20],
    },
];

/// Tracks which technologies are researched and current research progress.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResearchState {
    /// Which technologies have been completed (indexed by tech ID).
    pub completed: Vec<bool>,
    /// Currently researching tech index. `None` if idle.
    pub current_tech: Option<usize>,
    /// Science pack units consumed so far for current research.
    pub progress: u32,
}

impl ResearchState {
    /// Creates a new research state with no techs completed.
    pub fn new() -> Self {
        let len = TECHNOLOGIES.len();
        Self {
            completed: vec![false; len],
            current_tech: None,
            progress: 0,
        }
    }

    /// Whether a technology's prerequisites are met.
    pub fn can_research(&self, tech_idx: usize) -> bool {
        if tech_idx >= TECHNOLOGIES.len() {
            return false;
        }
        if self.completed[tech_idx] {
            return false;
        }
        let tech = &TECHNOLOGIES[tech_idx];
        tech.prerequisites.iter().all(|&prereq| self.completed[prereq])
    }

    /// Sets the next technology to research.
    pub fn start_research(&mut self, tech_idx: usize) {
        if self.can_research(tech_idx) {
            self.current_tech = Some(tech_idx);
            self.progress = 0;
        }
    }
}

/// Ticks all labs: consume science packs, advance research.
pub fn tick_labs(buildings: &mut Buildings, research: &mut ResearchState) {
    let tech_idx = match research.current_tech {
        Some(idx) => idx,
        None => return,
    };

    if tech_idx >= TECHNOLOGIES.len() {
        return;
    }
    let tech = &TECHNOLOGIES[tech_idx];

    let ids = buildings.alive_ids();
    for bid in ids {
        let building = match buildings.get(bid) {
            Some(b) => b,
            None => continue,
        };
        if building.kind != BuildingKind::Lab {
            continue;
        }
        let ms = match &building.machine_state {
            Some(ms) => ms,
            None => continue,
        };

        // Check if lab is on cooldown.
        if ms.progress_ticks > 0 {
            let building = buildings.get_mut(bid).unwrap();
            let ms = building.machine_state.as_mut().unwrap();
            ms.progress_ticks -= 1;
            continue;
        }

        // Check if lab has all required science packs.
        let has_all = tech.science_cost.iter().all(|&req| {
            ms.input_buffer.iter().any(|&r| r == req)
        });

        if !has_all {
            continue;
        }

        // Consume one of each required science pack.
        let building = buildings.get_mut(bid).unwrap();
        let ms = building.machine_state.as_mut().unwrap();
        for &req in tech.science_cost {
            if let Some(pos) = ms.input_buffer.iter().position(|&r| r == req) {
                ms.input_buffer.remove(pos);
            }
        }
        ms.progress_ticks = LAB_TICKS;
        ms.total_ticks = LAB_TICKS;

        research.progress += 1;

        // Check if research is complete.
        if research.progress >= tech.units_needed {
            research.completed[tech_idx] = true;
            research.current_tech = None;
            research.progress = 0;
        }
    }
}
