//! Milestone/achievement system.
//!
//! Milestones give players tangible goals beyond the story and research tree.
//! Each milestone awards bonus resources when achieved.

use std::collections::HashMap;

use crate::types::Resource;

/// A milestone the player can achieve.
pub struct Milestone {
    pub name: &'static str,
    pub description: &'static str,
    pub check: MilestoneCheck,
    /// Bonus resources awarded on completion.
    pub reward: &'static [(Resource, u32)],
}

/// What condition triggers a milestone.
pub enum MilestoneCheck {
    /// Total items crafted reaches this value.
    ItemsCrafted(u64),
    /// Total enemies killed reaches this value.
    EnemiesKilled(u64),
    /// A specific research is completed (tech index).
    ResearchDone(usize),
    /// Player inventory has at least this many of a resource.
    InventoryHas(Resource, u32),
    /// Total buildings placed (tracked via items_crafted as proxy).
    TickReached(u64),
}

/// All milestones.
pub static MILESTONES: &[Milestone] = &[
    Milestone {
        name: "First Steps",
        description: "Craft 10 items",
        check: MilestoneCheck::ItemsCrafted(10),
        reward: &[(Resource::IronPlate, 20), (Resource::Gear, 5)],
    },
    Milestone {
        name: "Getting Started",
        description: "Craft 100 items",
        check: MilestoneCheck::ItemsCrafted(100),
        reward: &[(Resource::IronPlate, 50), (Resource::CopperPlate, 30)],
    },
    Milestone {
        name: "Industrialist",
        description: "Craft 500 items",
        check: MilestoneCheck::ItemsCrafted(500),
        reward: &[(Resource::GreenCircuit, 20), (Resource::Gear, 20)],
    },
    Milestone {
        name: "Factory Boss",
        description: "Craft 2000 items",
        check: MilestoneCheck::ItemsCrafted(2000),
        reward: &[(Resource::SteelPlate, 30), (Resource::RedCircuit, 10)],
    },
    Milestone {
        name: "Mass Production",
        description: "Craft 5000 items",
        check: MilestoneCheck::ItemsCrafted(5000),
        reward: &[(Resource::BlueCircuit, 10), (Resource::SpeedModule, 5)],
    },
    Milestone {
        name: "Consciousness Rising",
        description: "Craft 10000 items",
        check: MilestoneCheck::ItemsCrafted(10000),
        reward: &[(Resource::RocketPart, 5)],
    },
    Milestone {
        name: "Defender",
        description: "Kill 20 enemies",
        check: MilestoneCheck::EnemiesKilled(20),
        reward: &[(Resource::BasicAmmo, 50), (Resource::IronPlate, 30)],
    },
    Milestone {
        name: "Exterminator",
        description: "Kill 100 enemies",
        check: MilestoneCheck::EnemiesKilled(100),
        reward: &[(Resource::PiercingAmmo, 30), (Resource::SteelPlate, 20)],
    },
    Milestone {
        name: "Warlord",
        description: "Kill 500 enemies",
        check: MilestoneCheck::EnemiesKilled(500),
        reward: &[(Resource::Grenade, 20), (Resource::SteelPlate, 50)],
    },
    Milestone {
        name: "First Research",
        description: "Complete Automation research",
        check: MilestoneCheck::ResearchDone(0),
        reward: &[(Resource::GreenCircuit, 10), (Resource::Gear, 10)],
    },
    Milestone {
        name: "Advanced Science",
        description: "Complete Advanced Electronics",
        check: MilestoneCheck::ResearchDone(7),
        reward: &[(Resource::RedCircuit, 15)],
    },
    Milestone {
        name: "Survivor",
        description: "Survive 10 minutes",
        check: MilestoneCheck::TickReached(12000),
        reward: &[(Resource::IronPlate, 100), (Resource::Coal, 50)],
    },
    Milestone {
        name: "Veteran",
        description: "Survive 30 minutes",
        check: MilestoneCheck::TickReached(36000),
        reward: &[(Resource::SteelPlate, 50), (Resource::GreenCircuit, 50)],
    },
];

/// Checks milestones and returns newly completed ones (indices).
pub fn check_milestones(
    completed: &[bool],
    items_crafted: u64,
    enemies_killed: u64,
    research_completed: &[bool],
    inventory: &HashMap<Resource, u32>,
    total_ticks: u64,
) -> Vec<usize> {
    let mut newly_completed = Vec::new();

    for (i, milestone) in MILESTONES.iter().enumerate() {
        if completed[i] {
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
        };
        if achieved {
            newly_completed.push(i);
        }
    }

    newly_completed
}
