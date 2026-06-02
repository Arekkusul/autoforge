//! Achievement roadmap system.
//!
//! Milestones are organized as a guided progression path through the game.
//! Each phase teaches the player a new concept and rewards resources that
//! bootstrap the next phase. The "next goal" is always visible in the UI.

use std::collections::HashMap;

use crate::types::Resource;

/// A milestone the player can achieve.
pub struct Milestone {
    pub name: &'static str,
    pub description: &'static str,
    /// Hint text that tells the player HOW to achieve this.
    pub hint: &'static str,
    pub phase: Phase,
    pub check: MilestoneCheck,
    /// Bonus resources awarded on completion.
    pub reward: &'static [(Resource, u32)],
}

/// Game phases for visual grouping.
#[derive(Clone, Copy, PartialEq)]
pub enum Phase {
    /// First 5 minutes — learn the basics.
    Early,
    /// 5-15 min — automation and research.
    Mid,
    /// 15-40 min — expansion and defense.
    Late,
    /// 40+ min — final push to victory.
    Endgame,
}

impl Phase {
    pub fn label(self) -> &'static str {
        match self {
            Phase::Early => "EARLY GAME",
            Phase::Mid => "MID GAME",
            Phase::Late => "LATE GAME",
            Phase::Endgame => "ENDGAME",
        }
    }
    pub fn color(self) -> (f32, f32, f32) {
        match self {
            Phase::Early => (0.4, 0.8, 0.4),
            Phase::Mid => (0.4, 0.6, 0.9),
            Phase::Late => (0.9, 0.7, 0.3),
            Phase::Endgame => (0.9, 0.4, 0.8),
        }
    }
}

/// What condition triggers a milestone.
pub enum MilestoneCheck {
    ItemsCrafted(u64),
    EnemiesKilled(u64),
    ResearchDone(usize),
    InventoryHas(Resource, u32),
    TickReached(u64),
    BuildingsPlaced(u64),
}

/// All milestones — ordered as a progression roadmap.
pub static MILESTONES: &[Milestone] = &[
    // ========== EARLY GAME ==========
    Milestone {
        name: "First Ore",
        description: "Place a miner on ore",
        hint: "Press 2 to select Miner, place it on the colored rocks near your ship",
        phase: Phase::Early,
        check: MilestoneCheck::ItemsCrafted(5),
        reward: &[(Resource::IronPlate, 20), (Resource::Coal, 20)],
    },
    Milestone {
        name: "Smelting Basics",
        description: "Craft 30 items (smelt some ore!)",
        hint: "Place a Furnace (3), feed it ore + coal using Inserters (4) and Belts (1)",
        phase: Phase::Early,
        check: MilestoneCheck::ItemsCrafted(30),
        reward: &[(Resource::IronPlate, 40), (Resource::CopperPlate, 30)],
    },
    Milestone {
        name: "Assembly Line",
        description: "Craft 100 items",
        hint: "Build an Assembler (5), click it to set a recipe (Gears are great!)",
        phase: Phase::Early,
        check: MilestoneCheck::ItemsCrafted(100),
        reward: &[(Resource::Gear, 20), (Resource::Wire, 20), (Resource::GreenCircuit, 10)],
    },
    Milestone {
        name: "First Research",
        description: "Complete Automation research",
        hint: "Craft Red Science (Gear+Copper) in Assembler, feed to Lab (8) via Inserter",
        phase: Phase::Early,
        check: MilestoneCheck::ResearchDone(0),
        reward: &[(Resource::GreenCircuit, 25), (Resource::Gear, 25)],
    },
    // ========== MID GAME ==========
    Milestone {
        name: "Green Science",
        description: "Complete Electronics research",
        hint: "Craft Green Circuits (Iron+Wire), then Inserter items, then Green Science packs",
        phase: Phase::Mid,
        check: MilestoneCheck::ResearchDone(6),
        reward: &[(Resource::GreenCircuit, 40), (Resource::SteelPlate, 20)],
    },
    Milestone {
        name: "Steel Age",
        description: "Complete Steel Processing",
        hint: "Steel = 5 Iron Plates in a Furnace. Unlocks Steel Furnace (faster smelting!)",
        phase: Phase::Mid,
        check: MilestoneCheck::ResearchDone(4),
        reward: &[(Resource::SteelPlate, 40), (Resource::Coal, 60)],
    },
    Milestone {
        name: "Factory Expansion",
        description: "Craft 500 items",
        hint: "Scale up! More miners, more furnaces, more assemblers. Use Storage Chests (9).",
        phase: Phase::Mid,
        check: MilestoneCheck::ItemsCrafted(500),
        reward: &[(Resource::GreenCircuit, 30), (Resource::Gear, 30), (Resource::IronPlate, 50)],
    },
    Milestone {
        name: "Advanced Circuits",
        description: "Complete Advanced Electronics",
        hint: "Research requires Green Science. Unlocks Red Circuits for advanced recipes.",
        phase: Phase::Mid,
        check: MilestoneCheck::ResearchDone(7),
        reward: &[(Resource::RedCircuit, 30), (Resource::SteelPlate, 30)],
    },
    Milestone {
        name: "First Defense",
        description: "Kill 20 enemies",
        hint: "Build Gun Turrets (T) and Walls (G). Feed ammo via Inserters!",
        phase: Phase::Mid,
        check: MilestoneCheck::EnemiesKilled(20),
        reward: &[(Resource::BasicAmmo, 100), (Resource::IronPlate, 60)],
    },
    // ========== LATE GAME ==========
    Milestone {
        name: "Solar Power",
        description: "Research Solar Energy",
        hint: "Solar Panels (P) generate free power during the day. No coal needed!",
        phase: Phase::Late,
        check: MilestoneCheck::ResearchDone(11),
        reward: &[(Resource::GreenCircuit, 50), (Resource::CopperPlate, 50)],
    },
    Milestone {
        name: "Mass Production",
        description: "Craft 2,000 items",
        hint: "Upgrade to Red Belts (scroll wheel on belt), Steel Furnaces, Fast Inserters.",
        phase: Phase::Late,
        check: MilestoneCheck::ItemsCrafted(2000),
        reward: &[(Resource::SteelPlate, 60), (Resource::RedCircuit, 25)],
    },
    Milestone {
        name: "War Machine",
        description: "Kill 100 enemies",
        hint: "Research Laser Turrets (L key) — they use power instead of ammo!",
        phase: Phase::Late,
        check: MilestoneCheck::EnemiesKilled(100),
        reward: &[(Resource::PiercingAmmo, 60), (Resource::SteelPlate, 40)],
    },
    Milestone {
        name: "Industrial Scale",
        description: "Craft 5,000 items",
        hint: "Blue Belts, Assembler T3, Chemical Plants (C). Expand your build zone with research!",
        phase: Phase::Late,
        check: MilestoneCheck::ItemsCrafted(5000),
        reward: &[(Resource::BlueCircuit, 20), (Resource::SpeedModule, 5)],
    },
    Milestone {
        name: "Veteran",
        description: "Survive 30 minutes",
        hint: "Keep your defenses strong. Enemies evolve over time!",
        phase: Phase::Late,
        check: MilestoneCheck::TickReached(36000),
        reward: &[(Resource::SteelPlate, 80), (Resource::GreenCircuit, 60)],
    },
    // ========== ENDGAME ==========
    Milestone {
        name: "Consciousness Rising",
        description: "Craft 10,000 items",
        hint: "You're getting close! Keep scaling production. FORGE is remembering...",
        phase: Phase::Endgame,
        check: MilestoneCheck::ItemsCrafted(10000),
        reward: &[(Resource::RocketPart, 10), (Resource::BlueCircuit, 25)],
    },
    Milestone {
        name: "Approaching the Horizon",
        description: "Craft 25,000 items",
        hint: "Halfway to FORGE restoration. Use Roboports for auto-delivery!",
        phase: Phase::Endgame,
        check: MilestoneCheck::ItemsCrafted(25000),
        reward: &[(Resource::RocketPart, 25), (Resource::SpeedModule, 10)],
    },
    Milestone {
        name: "FORGE Restored",
        description: "Craft 50,000 items — consciousness restored!",
        hint: "This is it! The final push to restore FORGE's consciousness!",
        phase: Phase::Endgame,
        check: MilestoneCheck::ItemsCrafted(50000),
        reward: &[(Resource::RocketPart, 50), (Resource::SpeedModule, 20), (Resource::BlueCircuit, 50)],
    },
];

/// Returns the index of the next uncompleted milestone (the player's current goal).
pub fn next_milestone(completed: &[bool]) -> Option<usize> {
    for (i, _) in MILESTONES.iter().enumerate() {
        if !completed.get(i).copied().unwrap_or(true) {
            return Some(i);
        }
    }
    None
}

/// Checks milestones and returns newly completed ones (indices).
pub fn check_milestones(
    completed: &[bool],
    items_crafted: u64,
    enemies_killed: u64,
    research_completed: &[bool],
    inventory: &HashMap<Resource, u32>,
    total_ticks: u64,
    buildings_placed: u64,
) -> Vec<usize> {
    let mut newly_completed = Vec::new();

    for (i, milestone) in MILESTONES.iter().enumerate() {
        if i < completed.len() && completed[i] {
            continue;
        }
        let achieved = match &milestone.check {
            MilestoneCheck::ItemsCrafted(n) => items_crafted >= *n,
            MilestoneCheck::EnemiesKilled(n) => enemies_killed >= *n,
            MilestoneCheck::ResearchDone(idx) => {
                *idx < research_completed.len() && research_completed[*idx]
            }
            MilestoneCheck::InventoryHas(resource, count) => {
                inventory.get(resource).copied().unwrap_or(0) >= *count
            }
            MilestoneCheck::TickReached(tick) => total_ticks >= *tick,
            MilestoneCheck::BuildingsPlaced(n) => buildings_placed >= *n,
        };
        if achieved {
            newly_completed.push(i);
        }
    }

    newly_completed
}
