//! The fixed-timestep game simulation tick, called at [`crate::constants::TICKS_PER_SECOND`]
//! from the main loop once per accumulated tick.

use crate::constants::*;
use crate::game::GameState;
use crate::sound;
use crate::types;
use crate::{
    belt, combat, enemy, fluid, input, inserter, machine, milestones, pollution, power, recipe,
    research, splitter, story,
};

/// Runs one simulation tick (called at fixed [`TICKS_PER_SECOND`] rate).
///
/// **Performance strategy**: Systems are frequency-gated based on how fast they
/// need to update. Critical systems (belts, inserters) run every tick. Slower
/// systems (pollution, enemies) run less frequently to reduce CPU load on
/// low-end devices.
///
/// | System | Frequency | Reason |
/// |--------|-----------|--------|
/// | Belts | Every tick | Smooth visual movement |
/// | Inserters | Every tick | Must keep up with belts |
/// | Machines | Every tick | Timer countdown accuracy |
/// | Labs | Every 2 ticks | Not time-critical |
/// | Enemies | Every 2 ticks | Movement still smooth at 10 updates/sec |
/// | Combat | Every 2 ticks | Turret fire rate is already 10+tick cooldowns |
/// | Pollution | Every 10 ticks | Changes very slowly, expensive diffusion |
pub(super) fn simulation_tick(state: &mut GameState, sfx: &sound::SoundEffects) {
    state.stats.total_ticks += 1;
    let tick = state.stats.total_ticks;

    // Day/night cycle advances every tick for smooth transitions.
    let was_day = state.daynight.is_day();
    state.daynight.tick();
    if was_day && !state.daynight.is_day() {
        state.toast("Night is falling... solar power reduces.".to_string(), 80);
    } else if !was_day && state.daynight.is_day() {
        state.toast("Dawn breaks! Solar panels at full power.".to_string(), 80);
    }

    // Tick toast notifications.
    state.tick_toasts();

    // Tick robot workers (move from ship to target).
    for robot in &mut state.robots {
        robot.2 += 0.05; // 20 ticks to reach target
    }
    state.robots.retain(|r| r.2 < 1.0);

    // Tick placement flash.
    if let Some((_, ref mut ticks)) = state.placement_flash {
        if *ticks > 0 {
            *ticks -= 1;
        } else {
            state.placement_flash = None;
        }
    }

    // Tick combat FX (fade out and remove expired).
    for fx in state.combat_fx.iter_mut() {
        if fx.4 > 0 {
            fx.4 -= 1;
        }
    }
    state.combat_fx.retain(|fx| fx.4 > 0);

    // --- EVERY TICK (20 Hz) --- Critical path for smooth gameplay ---

    // 1. Machines process: count down timers, complete recipes, start new ones.
    let crafted_before = state.stats.items_crafted;
    let mining_bonus = state.research.mining_bonus();
    machine::tick_machines(
        &mut state.grid,
        &mut state.buildings,
        &mut state.items,
        &mut state.stats,
        state.power.satisfaction,
        mining_bonus,
    );
    // First item celebration + periodic ding.
    if crafted_before == 0 && state.stats.items_crafted > 0 {
        sfx.play(&sfx.research_done);
        state.toast("Your first item! You're on your way~".to_string(), 100);
    } else if state.stats.items_crafted / 50 > crafted_before / 50 {
        sfx.play(&sfx.recipe_done);
    }
    // Prune old production log entries (keep last 60 seconds = 1200 ticks).
    if tick.is_multiple_of(100) {
        let cutoff = tick.saturating_sub(1200);
        state.stats.production_log.retain(|&(_, t)| t >= cutoff);
    }

    // Check for depleted miners using the alert system.
    if tick.is_multiple_of(200) && tick >= 200 {
        let depleted_count = state
            .buildings
            .iter()
            .filter(|(_, b)| b.kind == types::BuildingKind::Miner)
            .filter(|(_, b)| {
                state
                    .grid
                    .get_tile(b.pos)
                    .map(|t| t.deposit.is_none())
                    .unwrap_or(false)
            })
            .count();
        if depleted_count > 0 {
            state.alert(
                types::AlertKind::MinerDepleted,
                format!(
                    "{} miner(s) on depleted ore — relocate them!",
                    depleted_count
                ),
                120,
                types::AlertSeverity::Warning,
            );
        }
    }

    // 2. Machine output: eject finished items onto output belts.
    machine::tick_machine_output(&mut state.grid, &mut state.buildings, &mut state.items);

    // 3. Inserters: move items between belts and machines.
    inserter::tick_inserters(&mut state.grid, &mut state.buildings, &mut state.items);

    // 4. Belts: advance item progress, transfer between tiles.
    belt::tick_belts(&mut state.grid, &state.buildings, &mut state.items);

    // 4b. Splitters: route items at split points.
    splitter::tick_splitters(&mut state.grid, &mut state.buildings, &mut state.items);

    // 4c. Pump jacks: extract oil from deposits.
    fluid::tick_pump_jacks(&state.grid, &mut state.buildings);

    // --- EVERY 2 TICKS (10 Hz) --- Still responsive, saves 50% CPU for these ---

    if tick.is_multiple_of(2) {
        // 5. Labs: consume science packs, advance research.
        let techs_before: Vec<bool> = state.research.completed.clone();
        research::tick_labs(&mut state.buildings, &mut state.research);
        // Check for newly completed research.
        let newly_completed: Vec<usize> = techs_before
            .iter()
            .zip(state.research.completed.iter())
            .enumerate()
            .filter(|(_, (&was, &now))| !was && now)
            .map(|(i, _)| i)
            .collect();
        for i in newly_completed {
            if i < research::TECHNOLOGIES.len() {
                sfx.play(&sfx.research_done);
                state.toast(
                    format!("Research complete: {}!", research::TECHNOLOGIES[i].name),
                    120,
                );
            }
        }

        // 5b. Storage chests feed player inventory (the key progression mechanic).
        // Any items in a StorageChest's input_buffer are added to the player's inventory.
        let chest_ids = state.buildings.alive_ids();
        for bid in chest_ids {
            let kind = state.buildings.get(bid).map(|b| b.kind);
            if kind != Some(types::BuildingKind::StorageChest) {
                continue;
            }
            // Drain the chest buffer, then fold into inventory via the shared API.
            let drained: Vec<types::Resource> = match state.buildings.get_mut(bid) {
                Some(building) => match &mut building.machine_state {
                    Some(ms) => ms.input_buffer.drain(..).collect(),
                    None => Vec::new(),
                },
                None => Vec::new(),
            };
            for resource in drained {
                state.add_to_inventory(resource, 1);
            }
        }

        // 5c. Mark first wave for story.
        if !state.story.first_wave_arrived && state.enemies.wave_number > 0 {
            state.story.first_wave_arrived = true;
        }

        // Wave warning toast.
        if state.enemies.wave_warned && state.enemies.list.iter().filter(|e| e.alive).count() == 0 {
            state.toast("!! WAVE INCOMING — Prepare defenses! !!".to_string(), 100);
            // Only show once per warning cycle (wave_warned resets after spawn).
        }

        // 6. Enemy AI: movement, attacking buildings.
        let wave_before = state.enemies.wave_number;
        let buildings_before = state.buildings.alive_ids().len();
        enemy::tick_enemies(
            &mut state.grid,
            &mut state.buildings,
            &mut state.enemies,
            &state.nests,
            &mut state.evolution,
            tick,
            &mut state.stats.enemies_killed,
        );
        let buildings_after = state.buildings.alive_ids().len();
        if buildings_after < buildings_before {
            let lost = buildings_before - buildings_after;
            // Check if any walls/turrets were lost (defense breach).
            let wall_count_after = state
                .buildings
                .iter()
                .filter(|(_, b)| {
                    b.kind == types::BuildingKind::Wall
                        || b.kind == types::BuildingKind::Gate
                        || b.kind == types::BuildingKind::GunTurret
                        || b.kind == types::BuildingKind::LaserTurret
                })
                .count();
            let had_defenses = buildings_before > buildings_after; // any building lost
            if wall_count_after == 0 && had_defenses {
                state.toast(
                    "!! DEFENSES BREACHED — Factory under attack! !!".to_string(),
                    120,
                );
                sfx.play(&sfx.wave_warning);
            } else {
                state.toast(format!("Building destroyed! ({} lost)", lost), 60);
            }
        }

        if state.enemies.wave_number > wave_before {
            sfx.play(&sfx.wave_warning);
        }

        // 7. Trains: disabled pending full implementation (no cargo loading/unloading yet).
        // train::tick_trains(&state.grid, &state.buildings, &mut state.trains);

        // 8. Combat: turrets shoot enemies.
        let kills_before = state.stats.enemies_killed;
        combat::tick_combat(
            &state.grid,
            &mut state.buildings,
            &mut state.enemies,
            &mut state.stats.enemies_killed,
        );
        // Generate combat visual FX: laser/bullet lines from turrets to nearest enemies.
        for (_bid, building) in state.buildings.iter() {
            if building.kind != types::BuildingKind::GunTurret
                && building.kind != types::BuildingKind::LaserTurret
            {
                continue;
            }
            if let Some(ref ms) = building.machine_state {
                // progress_ticks == COOLDOWN means turret JUST fired this tick.
                if ms.progress_ticks == 10 {
                    // just fired (cooldown = 10 ticks)
                    let bx = building.pos.x as f32 * TILE_SIZE + TILE_SIZE * 0.5;
                    let by = building.pos.y as f32 * TILE_SIZE + TILE_SIZE * 0.5;
                    // Find closest alive enemy for the line target.
                    let mut closest = None;
                    let mut closest_dist = f32::MAX;
                    for enemy in &state.enemies.list {
                        if !enemy.alive {
                            continue;
                        }
                        let dx = enemy.x - bx;
                        let dy = enemy.y - by;
                        let d = dx * dx + dy * dy;
                        if d < closest_dist {
                            closest_dist = d;
                            closest = Some((enemy.x, enemy.y));
                        }
                    }
                    if let Some((tx, ty)) = closest {
                        let (r, g, b) = if building.kind == types::BuildingKind::LaserTurret {
                            (0.3, 0.5, 1.0) // blue laser
                        } else {
                            (1.0, 0.8, 0.2) // yellow bullet
                        };
                        state.combat_fx.push((bx, by, tx, ty, 4, r, g, b));
                    }
                }
            }
        }

        // Loot drops + sounds.
        let new_kills = state.stats.enemies_killed - kills_before;
        if new_kills > 0 {
            sfx.play(&sfx.turret_fire);
            sfx.play(&sfx.enemy_death);
            let n = new_kills as u32;
            *state
                .inventory
                .entry(types::Resource::IronPlate)
                .or_insert(0) += n * 2;
            *state.inventory.entry(types::Resource::Coal).or_insert(0) += n;
            // Higher evolution = rarer drops.
            if state.evolution > 0.3 {
                *state
                    .inventory
                    .entry(types::Resource::CopperPlate)
                    .or_insert(0) += n;
            }
            if state.evolution > 0.5 {
                *state.inventory.entry(types::Resource::Gear).or_insert(0) += n;
            }
            if state.evolution > 0.7 {
                *state
                    .inventory
                    .entry(types::Resource::GreenCircuit)
                    .or_insert(0) += n;
            }
            if state.evolution > 0.9 {
                *state
                    .inventory
                    .entry(types::Resource::SteelPlate)
                    .or_insert(0) += n;
            }
        }
    }

    // --- EVERY 10 TICKS: Passive building regeneration (walls/turrets heal 1 HP) ---
    if tick.is_multiple_of(10) {
        let ids = state.buildings.alive_ids();
        for bid in ids {
            if let Some(b) = state.buildings.get_mut(bid) {
                if b.hp < b.max_hp && b.hp > 0.0 {
                    b.hp = (b.hp + 0.5).min(b.max_hp);
                }
            }
        }
    }

    // --- EVERY 20 TICKS: Roboport logistics (auto-distribute items) ---
    if tick.is_multiple_of(20) {
        // Find all roboports, then for each, scan nearby machines that need inputs.
        let roboport_ids: Vec<(types::BuildingId, types::GridPos)> = state
            .buildings
            .alive_ids()
            .iter()
            .filter_map(|&bid| {
                state.buildings.get(bid).and_then(|b| {
                    if b.kind == types::BuildingKind::Roboport {
                        Some((bid, b.pos))
                    } else {
                        None
                    }
                })
            })
            .collect();

        for (_rbid, rpos) in &roboport_ids {
            let radius = 10i32;
            // Find machines in range that have a recipe set and need inputs.
            let nearby_ids: Vec<types::BuildingId> = state
                .buildings
                .alive_ids()
                .iter()
                .filter_map(|&bid| {
                    state.buildings.get(bid).and_then(|b| {
                        let d = b.pos.distance(*rpos);
                        if d < radius as f32
                            && b.kind != types::BuildingKind::StorageChest
                            && b.kind != types::BuildingKind::Roboport
                        {
                            match &b.machine_state {
                                Some(ms)
                                    if ms.selected_recipe.is_some()
                                        && ms.input_buffer.len() < 4 =>
                                {
                                    Some(bid)
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    })
                })
                .collect();

            // For each needy machine, try to supply from player inventory (simulating bot delivery).
            for mid in nearby_ids {
                if let Some(machine) = state.buildings.get(mid) {
                    if let Some(ms) = &machine.machine_state {
                        if let Some(rid) = ms.selected_recipe {
                            if rid.0 < recipe::RECIPES.len() {
                                let recipe_inputs = recipe::RECIPES[rid.0].inputs;
                                // Deliver one unit of each needed input from inventory.
                                for &(res, _count) in recipe_inputs {
                                    if state.inventory_count(res) > 0 {
                                        let m = state.buildings.get_mut(mid).unwrap();
                                        let ms = m.machine_state.as_mut().unwrap();
                                        if ms.input_buffer.len() < 8 {
                                            ms.input_buffer.push(res);
                                            state.remove_from_inventory(res, 1);
                                        }
                                        break; // one item per tick per machine
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // --- EVERY 5 TICKS (4 Hz) --- Medium frequency systems ---

    if tick.is_multiple_of(5) {
        // 6. Power: calculate supply/demand, consume fuel in boilers/engines.
        power::update_power(&mut state.buildings, &mut state.power, &state.daynight);

        // 6b. Check for alert conditions (power, turret ammo, etc.).
        input::check_alerts(state, sfx);
    }

    // --- EVERY 10 TICKS (2 Hz) --- Expensive systems that change slowly ---

    if tick.is_multiple_of(10) {
        // 7. Pollution: generate from machines, diffuse across grid.
        pollution::tick_pollution(&mut state.grid, &state.buildings);
    }

    // --- EVERY 20 TICKS (1 Hz) --- Story triggers + milestones ---
    if tick.is_multiple_of(20) {
        let new_beats = story::check_story_triggers(
            &mut state.story,
            state.stats.items_crafted,
            state.stats.enemies_killed,
            &state.research.completed,
            tick,
        );
        for (text, subtext) in new_beats {
            state.toast(text, 120);
            state.toast(subtext, 160);
        }

        // Check rocket silo launches — the primary win condition.
        // A rocket silo with 100+ Rocket Parts triggers a launch.
        {
            let silo_ids = state.buildings.alive_ids();
            for bid in silo_ids {
                let is_silo = state
                    .buildings
                    .get(bid)
                    .map(|b| b.kind == types::BuildingKind::RocketSilo)
                    .unwrap_or(false);
                if !is_silo {
                    continue;
                }
                let parts = state
                    .buildings
                    .get(bid)
                    .and_then(|b| b.machine_state.as_ref())
                    .map(|ms| {
                        ms.input_buffer
                            .iter()
                            .filter(|r| **r == types::Resource::RocketPart)
                            .count()
                    })
                    .unwrap_or(0);
                if parts >= ROCKET_PARTS_PER_LAUNCH {
                    // Launch!
                    let building = state.buildings.get_mut(bid).unwrap();
                    let ms = building.machine_state.as_mut().unwrap();
                    // Remove rocket parts consumed by the launch.
                    let mut removed = 0usize;
                    ms.input_buffer.retain(|r| {
                        if *r == types::Resource::RocketPart && removed < ROCKET_PARTS_PER_LAUNCH {
                            removed += 1;
                            false
                        } else {
                            true
                        }
                    });
                    state.stats.rockets_launched += 1;
                    sfx.play(&sfx.research_done);
                    state.toast("*** ROCKET LAUNCHED! ***".to_string(), 200);
                    if !state.game_won {
                        state.game_won = true;
                        state.toast(
                            "CONSCIOUSNESS RESTORED! The signal reaches your crew!".to_string(),
                            300,
                        );
                        state.toast("Thank you for playing AutoForge <3".to_string(), 300);
                    }
                }
            }
        }

        // Fallback win condition: 50,000 items crafted (for players who skip the silo).
        if state.stats.items_crafted >= 50000 && !state.game_won {
            state.game_won = true;
            state.toast(
                "CONSCIOUSNESS RESTORED! You found your crew!".to_string(),
                300,
            );
            state.toast("Thank you for playing AutoForge <3".to_string(), 300);
        }

        // Soft-lock detection: if player has no buildings and no resources to rebuild,
        // give them emergency supplies so they can recover.
        if tick.is_multiple_of(600) && state.buildings.alive_ids().is_empty() {
            let total_resources: u32 = state.inventory.values().sum();
            if total_resources < 10 {
                state.toast(
                    "FORGE: Don't give up! Emergency supplies incoming~".to_string(),
                    120,
                );
                *state
                    .inventory
                    .entry(types::Resource::IronPlate)
                    .or_insert(0) += 50;
                *state
                    .inventory
                    .entry(types::Resource::CopperPlate)
                    .or_insert(0) += 30;
                *state.inventory.entry(types::Resource::Stone).or_insert(0) += 20;
                *state.inventory.entry(types::Resource::Coal).or_insert(0) += 30;
                *state.inventory.entry(types::Resource::Gear).or_insert(0) += 20;
            }
        }

        // Check milestones.
        let new_milestones = milestones::check_milestones(
            &state.milestones_completed,
            state.stats.items_crafted,
            state.stats.enemies_killed,
            &state.research.completed,
            &state.inventory,
            tick,
            state.stats.buildings_placed,
        );
        for idx in new_milestones {
            state.milestones_completed[idx] = true;
            let milestone = &milestones::MILESTONES[idx];
            // Award rewards to inventory.
            for &(resource, count) in milestone.reward {
                *state.inventory.entry(resource).or_insert(0) += count;
            }
            sfx.play(&sfx.research_done); // celebratory arpeggio
            state.toast(format!("*** MILESTONE: {} ***", milestone.name), 120);
            state.toast(
                format!("+{} resource types rewarded!", milestone.reward.len()),
                80,
            );
        }

        // Expand build zone with research milestones.
        let base_radius = 40.0f32;
        let bonus = state.research.completed.iter().filter(|&&c| c).count() as f32 * 3.0;
        state.build_radius = base_radius + bonus;
    }
}
