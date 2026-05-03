//! Day/night cycle system.
//!
//! A full cycle is 10 minutes (7 min day, 3 min night). Solar panels produce
//! power proportional to sunlight. The world darkens slightly at night.
//! Accumulators charge during the day and discharge at night.

use serde::{Deserialize, Serialize};

use crate::constants::*;

/// Tracks the current time of day.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DayNightState {
    /// Current time in the cycle, in seconds. Ranges [0, FULL_CYCLE_SECS).
    pub time: f32,
}

impl Default for DayNightState {
    fn default() -> Self {
        Self { time: 0.0 } // Start at dawn
    }
}

impl DayNightState {
    /// Advances the cycle by one tick.
    pub fn tick(&mut self) {
        self.time += 1.0 / TICKS_PER_SECOND as f32;
        if self.time >= FULL_CYCLE_SECS {
            self.time -= FULL_CYCLE_SECS;
        }
    }

    /// Returns the current sunlight level (0.0 = full night, 1.0 = full day).
    ///
    /// Transitions smoothly at dawn and dusk over ~30 seconds.
    pub fn sunlight(&self) -> f32 {
        let t = self.time;
        let _dawn_start = 0.0;
        let dawn_end = 30.0;
        let dusk_start = DAY_DURATION_SECS - 30.0;
        let dusk_end = DAY_DURATION_SECS;
        let night_end = FULL_CYCLE_SECS;

        if t < dawn_end {
            // Dawn: ramp up from 0.3 to 1.0
            let progress = t / dawn_end;
            0.3 + 0.7 * progress
        } else if t < dusk_start {
            // Full day
            1.0
        } else if t < dusk_end {
            // Dusk: ramp down from 1.0 to 0.3
            let progress = (t - dusk_start) / (dusk_end - dusk_start);
            1.0 - 0.7 * progress
        } else if t < night_end {
            // Night
            0.3
        } else {
            0.3
        }
    }

    /// Whether it's currently daytime (sunlight > 0.5).
    pub fn is_day(&self) -> bool {
        self.sunlight() > 0.5
    }

    /// Returns the solar panel output multiplier (0.0 at night, 1.0 at full day).
    pub fn solar_multiplier(&self) -> f32 {
        // Solar only works when sunlight > 0.3
        let sun = self.sunlight();
        if sun <= 0.3 {
            0.0
        } else {
            (sun - 0.3) / 0.7 // normalize to 0.0-1.0
        }
    }

    /// Returns the world ambient darkness level for rendering (0.0 = no darkening, 0.4 = night).
    pub fn darkness(&self) -> f32 {
        let sun = self.sunlight();
        (1.0 - sun) * 0.5 // max 35% darkening at night
    }

    /// Returns a display string like "Day 12:34" or "Night 3:45".
    pub fn display(&self) -> String {
        let minutes = (self.time / 60.0) as u32;
        let seconds = (self.time % 60.0) as u32;
        let phase = if self.is_day() { "Day" } else { "Night" };
        format!("{} {}:{:02}", phase, minutes, seconds)
    }
}
