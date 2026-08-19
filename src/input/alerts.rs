//! Periodic non-input alert checks (power shortage, turret ammo, etc.)
//! run once per tick alongside input handling.

use crate::game::GameState;
use crate::sound;
use crate::types;

/// Periodic alert checks for critical game events (power, ammo, etc.).
/// Called every tick but internally gates on appropriate frequencies.
pub fn check_alerts(state: &mut GameState, sfx: &sound::SoundEffects) {
    let tick = state.stats.total_ticks;
    if !tick.is_multiple_of(100) {
        return;
    }

    // --- Power alerts ---
    if state.power.satisfaction < 0.1 && state.power.demand > 0.0 {
        state.alert(
            types::AlertKind::PowerBlackout,
            "BLACKOUT -- No power! Build more generators.".to_string(),
            120,
            types::AlertSeverity::Critical,
        );
        sfx.play(&sfx.wave_warning);
    } else if state.power.satisfaction < 0.5 && state.power.demand > 0.0 {
        state.alert(
            types::AlertKind::PowerBrownout,
            format!(
                "Low power: {:.0}% -- machines slowing down",
                state.power.satisfaction * 100.0
            ),
            100,
            types::AlertSeverity::Warning,
        );
    }

    // --- Turret ammo alerts ---
    let mut empty_turrets = 0u32;
    let mut low_turrets = 0u32;
    for (_, b) in state.buildings.iter() {
        if b.kind != types::BuildingKind::GunTurret {
            continue;
        }
        if let Some(ms) = &b.machine_state {
            let ammo = ms
                .input_buffer
                .iter()
                .filter(|r| {
                    matches!(
                        r,
                        types::Resource::BasicAmmo | types::Resource::PiercingAmmo
                    )
                })
                .count();
            if ammo == 0 {
                empty_turrets += 1;
            } else if ammo <= 2 {
                low_turrets += 1;
            }
        }
    }
    let enemies_alive = state.enemies.list.iter().any(|e| e.alive);
    if empty_turrets > 0 && enemies_alive {
        state.alert(
            types::AlertKind::TurretAmmoEmpty,
            format!("{} turret(s) have NO ammo!", empty_turrets),
            120,
            types::AlertSeverity::Critical,
        );
        sfx.play(&sfx.wave_warning);
    } else if low_turrets > 0 {
        state.alert(
            types::AlertKind::TurretAmmoLow,
            format!("{} turret(s) running low on ammo", low_turrets),
            80,
            types::AlertSeverity::Warning,
        );
    }
}
