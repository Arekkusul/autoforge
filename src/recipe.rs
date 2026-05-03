//! Recipe definitions and matching.
//!
//! Recipes define what inputs a machine consumes and what outputs it produces.
//! All recipes are static data — they never change at runtime. Machines look up
//! which recipe to use based on their type and input buffer contents.

use crate::types::*;

/// Index into the [`RECIPES`] array.
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct RecipeId(pub usize);

/// A single recipe: inputs consumed → outputs produced, over a number of ticks.
#[derive(Clone, Debug)]
pub struct Recipe {
    /// Which machine types can execute this recipe.
    pub machine: BuildingKind,
    /// Items consumed (resource, count). Empty for miners.
    pub inputs: &'static [(Resource, u32)],
    /// Items produced (resource, count).
    pub outputs: &'static [(Resource, u32)],
    /// Base processing time in ticks (before speed modifiers).
    pub base_ticks: u32,
    /// Human-readable name for UI display.
    pub name: &'static str,
}

/// All recipes in the game. Index = RecipeId.
pub static RECIPES: &[Recipe] = &[
    // ===== SMELTING (Stone Furnace / Steel Furnace / Electric Furnace) =====
    // ID 0: Iron Ore → Iron Plate
    Recipe {
        machine: BuildingKind::StoneFurnace,
        inputs: &[(Resource::IronOre, 1)],
        outputs: &[(Resource::IronPlate, 1)],
        base_ticks: 60,
        name: "Smelt Iron Plate",
    },
    // ID 1: Copper Ore → Copper Plate
    Recipe {
        machine: BuildingKind::StoneFurnace,
        inputs: &[(Resource::CopperOre, 1)],
        outputs: &[(Resource::CopperPlate, 1)],
        base_ticks: 60,
        name: "Smelt Copper Plate",
    },
    // ID 2: Stone → Stone Brick
    Recipe {
        machine: BuildingKind::StoneFurnace,
        inputs: &[(Resource::Stone, 2)],
        outputs: &[(Resource::StoneBrick, 1)],
        base_ticks: 60,
        name: "Smelt Stone Brick",
    },
    // ID 3: Iron Plate → Steel Plate (5 iron plates!)
    Recipe {
        machine: BuildingKind::StoneFurnace,
        inputs: &[(Resource::IronPlate, 5)],
        outputs: &[(Resource::SteelPlate, 1)],
        base_ticks: 100,
        name: "Smelt Steel Plate",
    },
    // ===== ASSEMBLY (Assembler T1/T2/T3) =====
    // ID 4: Gear
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::IronPlate, 2)],
        outputs: &[(Resource::Gear, 1)],
        base_ticks: 40,
        name: "Craft Gear",
    },
    // ID 5: Wire (1 copper plate → 2 wire)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::CopperPlate, 1)],
        outputs: &[(Resource::Wire, 2)],
        base_ticks: 20,
        name: "Craft Wire",
    },
    // ID 6: Green Circuit
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::IronPlate, 1), (Resource::Wire, 3)],
        outputs: &[(Resource::GreenCircuit, 1)],
        base_ticks: 40,
        name: "Craft Green Circuit",
    },
    // ID 7: Pipe
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::IronPlate, 1)],
        outputs: &[(Resource::Pipe, 1)],
        base_ticks: 20,
        name: "Craft Pipe",
    },
    // ID 8: Iron Stick
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::IronPlate, 1)],
        outputs: &[(Resource::IronStick, 2)],
        base_ticks: 20,
        name: "Craft Iron Stick",
    },
    // ID 9: Science Pack Red
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::Gear, 1), (Resource::CopperPlate, 1)],
        outputs: &[(Resource::ScienceRed, 1)],
        base_ticks: 100,
        name: "Craft Red Science",
    },
    // ID 10: Inserter (item, used in Green Science recipe)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::GreenCircuit, 1), (Resource::Gear, 1), (Resource::IronPlate, 1)],
        outputs: &[(Resource::Inserter, 1)],
        base_ticks: 40,
        name: "Craft Inserter",
    },
    // ID 11: Science Pack Green
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::Inserter, 1), (Resource::IronPlate, 1)],
        outputs: &[(Resource::ScienceGreen, 1)],
        base_ticks: 120,
        name: "Craft Green Science",
    },
    // ID 12: Red Circuit
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::GreenCircuit, 2), (Resource::Plastic, 2), (Resource::Wire, 4)],
        outputs: &[(Resource::RedCircuit, 1)],
        base_ticks: 80,
        name: "Craft Red Circuit",
    },
    // ID 13: Engine Unit
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::SteelPlate, 1), (Resource::Gear, 1), (Resource::Pipe, 2)],
        outputs: &[(Resource::EngineUnit, 1)],
        base_ticks: 80,
        name: "Craft Engine Unit",
    },
    // ID 14: Basic Ammo
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::IronPlate, 1)],
        outputs: &[(Resource::BasicAmmo, 1)],
        base_ticks: 20,
        name: "Craft Basic Ammo",
    },
    // ID 15: Piercing Ammo
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::BasicAmmo, 1), (Resource::SteelPlate, 1), (Resource::CopperPlate, 1)],
        outputs: &[(Resource::PiercingAmmo, 1)],
        base_ticks: 40,
        name: "Craft Piercing Ammo",
    },
    // ID 16: Grenade
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::Coal, 1), (Resource::IronPlate, 5)],
        outputs: &[(Resource::Grenade, 1)],
        base_ticks: 60,
        name: "Craft Grenade",
    },
    // ID 17: Rail
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::Stone, 1), (Resource::SteelPlate, 1), (Resource::IronStick, 1)],
        outputs: &[(Resource::Rail, 2)],
        base_ticks: 40,
        name: "Craft Rail",
    },
    // ===== CHEMICAL PLANT =====
    // ID 18: Sulfur (from coal + iron plate — simplified from fluid-based)
    Recipe {
        machine: BuildingKind::ChemicalPlant,
        inputs: &[(Resource::Coal, 2), (Resource::IronPlate, 1)],
        outputs: &[(Resource::Sulfur, 2)],
        base_ticks: 60,
        name: "Process Sulfur",
    },
    // ID 19: Plastic (from coal + copper plate — simplified)
    Recipe {
        machine: BuildingKind::ChemicalPlant,
        inputs: &[(Resource::Coal, 1), (Resource::CopperPlate, 1)],
        outputs: &[(Resource::Plastic, 2)],
        base_ticks: 60,
        name: "Process Plastic",
    },
    // ID 20: Battery (from copper + iron + sulfur)
    Recipe {
        machine: BuildingKind::ChemicalPlant,
        inputs: &[(Resource::CopperPlate, 1), (Resource::IronPlate, 1), (Resource::Sulfur, 1)],
        outputs: &[(Resource::Battery, 1)],
        base_ticks: 80,
        name: "Craft Battery",
    },
    // ===== MORE ASSEMBLY =====
    // ID 21: Blue Circuit
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::RedCircuit, 2), (Resource::GreenCircuit, 5), (Resource::Wire, 10)],
        outputs: &[(Resource::BlueCircuit, 1)],
        base_ticks: 120,
        name: "Craft Blue Circuit",
    },
    // ID 22: Speed Module
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::RedCircuit, 2), (Resource::GreenCircuit, 5)],
        outputs: &[(Resource::SpeedModule, 1)],
        base_ticks: 100,
        name: "Craft Speed Module",
    },
    // ID 23: Efficiency Module
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::RedCircuit, 2), (Resource::GreenCircuit, 5)],
        outputs: &[(Resource::EfficiencyModule, 1)],
        base_ticks: 100,
        name: "Craft Efficiency Module",
    },
    // ID 24: Concrete
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::StoneBrick, 5), (Resource::IronOre, 1)],
        outputs: &[(Resource::Concrete, 10)],
        base_ticks: 60,
        name: "Craft Concrete",
    },
    // ID 25: Low Density Structure
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::SteelPlate, 2), (Resource::CopperPlate, 5), (Resource::Plastic, 5)],
        outputs: &[(Resource::LowDensityStructure, 1)],
        base_ticks: 100,
        name: "Craft Low Density Structure",
    },
    // ID 26: Rocket Fuel (from coal + steel)
    Recipe {
        machine: BuildingKind::ChemicalPlant,
        inputs: &[(Resource::Coal, 5), (Resource::SteelPlate, 1)],
        outputs: &[(Resource::RocketFuel, 1)],
        base_ticks: 120,
        name: "Process Rocket Fuel",
    },
    // ID 27: Rocket Part (endgame)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::BlueCircuit, 5), (Resource::SpeedModule, 1), (Resource::RocketFuel, 5), (Resource::LowDensityStructure, 5)],
        outputs: &[(Resource::RocketPart, 1)],
        base_ticks: 200,
        name: "Craft Rocket Part",
    },
    // ID 28: Science Pack Blue/Military
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::PiercingAmmo, 1), (Resource::Grenade, 1), (Resource::StoneBrick, 2)],
        outputs: &[(Resource::ScienceBlue, 1)],
        base_ticks: 140,
        name: "Craft Blue Science",
    },
    // ID 29: Science Pack Purple
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::Rail, 5), (Resource::EngineUnit, 1), (Resource::BlueCircuit, 1)],
        outputs: &[(Resource::SciencePurple, 1)],
        base_ticks: 160,
        name: "Craft Purple Science",
    },
    // ID 30: Science Pack Yellow
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::BlueCircuit, 2), (Resource::SpeedModule, 1), (Resource::Battery, 2)],
        outputs: &[(Resource::ScienceYellow, 1)],
        base_ticks: 180,
        name: "Craft Yellow Science",
    },
    // ===== ALTERNATIVE PATHS (give players choices) =====
    // ID 31: Nuclear Fuel Cell (endgame power)
    Recipe {
        machine: BuildingKind::ChemicalPlant,
        inputs: &[(Resource::IronPlate, 10), (Resource::SteelPlate, 1)],
        outputs: &[(Resource::NuclearFuelCell, 1)],
        base_ticks: 150,
        name: "Craft Nuclear Fuel Cell",
    },
    // ID 32: Electric Engine (for advanced machines)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::EngineUnit, 1), (Resource::GreenCircuit, 2)],
        outputs: &[(Resource::ElectricEngine, 1)],
        base_ticks: 80,
        name: "Craft Electric Engine",
    },
    // ID 33: Flying Robot Frame (for logistics bots later)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::ElectricEngine, 1), (Resource::Battery, 2), (Resource::SteelPlate, 1), (Resource::GreenCircuit, 3)],
        outputs: &[(Resource::FlyingRobotFrame, 1)],
        base_ticks: 120,
        name: "Craft Robot Frame",
    },
    // ID 34: Productivity Module (alternative to Speed Module)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::RedCircuit, 2), (Resource::GreenCircuit, 5)],
        outputs: &[(Resource::ProductivityModule, 1)],
        base_ticks: 100,
        name: "Craft Productivity Module",
    },
    // ID 35: Solar Panel (as item for building from inventory)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::SteelPlate, 5), (Resource::CopperPlate, 10), (Resource::GreenCircuit, 10)],
        outputs: &[(Resource::SolarPanelItem, 1)],
        base_ticks: 120,
        name: "Craft Solar Panel",
    },
    // ID 36: Accumulator (as item)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::IronPlate, 5), (Resource::Battery, 5)],
        outputs: &[(Resource::AccumulatorItem, 1)],
        base_ticks: 80,
        name: "Craft Accumulator",
    },
    // ===== SMELTING ADDITIONS =====
    // ID 37: Copper Ore → Copper Plate (also in furnace, already ID 1 but let's add steel variant)
    // Actually ID 1 already covers this. Let's add uranium processing for centrifuge.
    // ID 37: Uranium Processing (Centrifuge)
    Recipe {
        machine: BuildingKind::Centrifuge,
        inputs: &[(Resource::UraniumOre, 10)],
        outputs: &[(Resource::Uranium238, 9), (Resource::Uranium235, 1)],
        base_ticks: 200,
        name: "Process Uranium",
    },
    // ===== MORE ASSEMBLY FOR DIVERSITY =====
    // ID 38: Repair Pack (for repairing damaged buildings)
    Recipe {
        machine: BuildingKind::AssemblerT1,
        inputs: &[(Resource::Gear, 2), (Resource::GreenCircuit, 2)],
        outputs: &[(Resource::Gear, 1)], // placeholder output - would be RepairPack
        base_ticks: 30,
        name: "Craft Repair Pack",
    },
];

/// Finds a recipe that a given machine kind can execute with the provided input buffer.
///
/// If `locked_recipe` is set, ONLY that specific recipe is checked. This prevents
/// assemblers from making the wrong item when multiple recipes share inputs.
///
/// If `locked_recipe` is None, falls back to checking all compatible recipes (for
/// furnaces which auto-detect based on input).
pub fn find_matching_recipe(
    machine_kind: BuildingKind,
    input_buffer: &[Resource],
    locked_recipe: Option<RecipeId>,
) -> Option<RecipeId> {
    // If a specific recipe is locked, only check that one.
    if let Some(rid) = locked_recipe {
        if rid.0 < RECIPES.len() {
            let recipe = &RECIPES[rid.0];
            if recipe_matches_machine(recipe, machine_kind)
                && !recipe.inputs.is_empty()
                && buffer_satisfies(input_buffer, recipe.inputs)
            {
                return Some(rid);
            }
        }
        return None; // locked but can't satisfy — wait for correct inputs
    }

    // No lock — auto-detect (used by furnaces).
    for (idx, recipe) in RECIPES.iter().enumerate() {
        if !recipe_matches_machine(recipe, machine_kind) {
            continue;
        }
        if recipe.inputs.is_empty() {
            continue;
        }
        if buffer_satisfies(input_buffer, recipe.inputs) {
            return Some(RecipeId(idx));
        }
    }
    None
}

/// Returns all recipes compatible with a given machine kind (for UI recipe selector).
pub fn recipes_for_machine(machine_kind: BuildingKind) -> Vec<RecipeId> {
    RECIPES
        .iter()
        .enumerate()
        .filter(|(_, recipe)| recipe_matches_machine(recipe, machine_kind) && !recipe.inputs.is_empty())
        .map(|(idx, _)| RecipeId(idx))
        .collect()
}

/// Checks if a recipe can run on this machine kind.
///
/// Smelting recipes work on any furnace type. Assembly recipes work on any assembler tier.
fn recipe_matches_machine(recipe: &Recipe, kind: BuildingKind) -> bool {
    match recipe.machine {
        BuildingKind::StoneFurnace => matches!(
            kind,
            BuildingKind::StoneFurnace | BuildingKind::SteelFurnace | BuildingKind::ElectricFurnace
        ),
        BuildingKind::AssemblerT1 => matches!(
            kind,
            BuildingKind::AssemblerT1 | BuildingKind::AssemblerT2 | BuildingKind::AssemblerT3
        ),
        BuildingKind::ChemicalPlant => kind == BuildingKind::ChemicalPlant,
        other => kind == other,
    }
}

/// Checks if the buffer contains at least the required quantities of each resource.
fn buffer_satisfies(buffer: &[Resource], requirements: &[(Resource, u32)]) -> bool {
    for &(resource, count) in requirements {
        let available = buffer.iter().filter(|&&r| r == resource).count() as u32;
        if available < count {
            return false;
        }
    }
    true
}

/// Consumes the required inputs from the buffer for a given recipe.
///
/// Call this only after [`buffer_satisfies`] returns true.
pub fn consume_inputs(buffer: &mut Vec<Resource>, recipe: &Recipe) {
    for &(resource, count) in recipe.inputs {
        let mut remaining = count;
        buffer.retain(|&r| {
            if r == resource && remaining > 0 {
                remaining -= 1;
                false
            } else {
                true
            }
        });
    }
}
