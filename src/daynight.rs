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
    /// Creates a state at the given cycle time, normalized into `[0, FULL_CYCLE_SECS)`.
    ///
    /// This is the safe entry point for externally-sourced values (e.g. a loaded
    /// save file). It wraps large or negative inputs into range and coerces
    /// non-finite values (`NaN`/`inf`) to dawn, guaranteeing the resulting `time`
    /// always yields a valid sunlight/solar reading. Assigning a raw out-of-range
    /// value would instead peg `sunlight()` at its night floor forever, silently
    /// disabling solar power and leaving the night overlay stuck on.
    pub fn from_time(secs: f32) -> Self {
        let time = if secs.is_finite() {
            secs.rem_euclid(FULL_CYCLE_SECS)
        } else {
            0.0
        };
        Self { time }
    }

    /// Advances the cycle by one tick, wrapping at the end of the cycle.
    ///
    /// Uses euclidean remainder so the time stays in range even if it somehow
    /// starts beyond one full cycle (defensive against a bad loaded value).
    pub fn tick(&mut self) {
        self.time = (self.time + 1.0 / TICKS_PER_SECOND as f32).rem_euclid(FULL_CYCLE_SECS);
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

    /// Returns the world ambient darkness level for rendering (0.0 = no darkening, 0.35 = night).
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_time_wraps_values_beyond_one_cycle() {
        // A value just past the cycle length maps back near dawn.
        let s = DayNightState::from_time(FULL_CYCLE_SECS + 10.0);
        assert!((s.time - 10.0).abs() < 1e-3);

        // Many cycles collapse to the same in-range remainder.
        let s = DayNightState::from_time(FULL_CYCLE_SECS * 5.0 + 42.0);
        assert!((s.time - 42.0).abs() < 1e-3);
    }

    #[test]
    fn from_time_wraps_negative_values_into_range() {
        let s = DayNightState::from_time(-5.0);
        assert!(s.time >= 0.0 && s.time < FULL_CYCLE_SECS);
        assert!((s.time - (FULL_CYCLE_SECS - 5.0)).abs() < 1e-3);
    }

    #[test]
    fn from_time_coerces_non_finite_to_dawn() {
        assert_eq!(DayNightState::from_time(f32::NAN).time, 0.0);
        assert_eq!(DayNightState::from_time(f32::INFINITY).time, 0.0);
        assert_eq!(DayNightState::from_time(f32::NEG_INFINITY).time, 0.0);
    }

    #[test]
    fn from_time_leaves_in_range_values_untouched() {
        let s = DayNightState::from_time(123.4);
        assert!((s.time - 123.4).abs() < 1e-3);
    }

    #[test]
    fn tick_advances_by_one_game_tick() {
        let mut s = DayNightState::default();
        s.tick();
        assert!((s.time - 1.0 / TICKS_PER_SECOND as f32).abs() < 1e-6);
    }

    #[test]
    fn tick_wraps_at_end_of_cycle() {
        // Start one tick before the wrap point; the next tick must land back near zero.
        let step = 1.0 / TICKS_PER_SECOND as f32;
        let mut s = DayNightState::from_time(FULL_CYCLE_SECS - step * 0.5);
        s.tick();
        assert!(s.time < step); // wrapped, not stuck at the top
        assert!(s.time >= 0.0);
    }

    #[test]
    fn tick_recovers_from_an_out_of_range_time() {
        // Even if a bad value slipped in, ticking must pull it back into range.
        let mut s = DayNightState {
            time: FULL_CYCLE_SECS * 3.0 + 1.0,
        };
        s.tick();
        assert!(s.time >= 0.0 && s.time < FULL_CYCLE_SECS);
    }

    #[test]
    fn sunlight_ramps_up_at_dawn() {
        assert!((DayNightState { time: 0.0 }.sunlight() - 0.3).abs() < 1e-6);
        // Full brightness once dawn completes (30s in).
        assert!((DayNightState { time: 30.0 }.sunlight() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sunlight_is_full_during_the_day() {
        let midday = DAY_DURATION_SECS * 0.5;
        assert!((DayNightState { time: midday }.sunlight() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn sunlight_holds_at_night_floor() {
        let midnight = DAY_DURATION_SECS + NIGHT_DURATION_SECS * 0.5;
        assert!((DayNightState { time: midnight }.sunlight() - 0.3).abs() < 1e-6);
    }

    #[test]
    fn sunlight_is_continuous_across_the_cycle_boundary() {
        // Last instant of night and first instant of dawn must agree (no visual jump).
        let end = DayNightState {
            time: FULL_CYCLE_SECS - 1e-3,
        }
        .sunlight();
        let start = DayNightState { time: 0.0 }.sunlight();
        assert!((end - start).abs() < 1e-3);
    }

    #[test]
    fn sunlight_stays_within_bounds_all_cycle() {
        let mut t = 0.0;
        while t < FULL_CYCLE_SECS {
            let sun = DayNightState { time: t }.sunlight();
            assert!((0.3..=1.0).contains(&sun), "sunlight {sun} out of range at t={t}");
            t += 1.0;
        }
    }

    #[test]
    fn solar_multiplier_is_zero_at_night_and_full_at_midday() {
        let midnight = DAY_DURATION_SECS + NIGHT_DURATION_SECS * 0.5;
        assert_eq!(DayNightState { time: midnight }.solar_multiplier(), 0.0);

        let midday = DAY_DURATION_SECS * 0.5;
        assert!((DayNightState { time: midday }.solar_multiplier() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn solar_multiplier_tracks_sunlight_between_floor_and_peak() {
        // Halfway up the dawn ramp: sunlight 0.65 -> multiplier (0.65-0.3)/0.7 = 0.5.
        let s = DayNightState { time: 15.0 };
        assert!((s.sunlight() - 0.65).abs() < 1e-6);
        assert!((s.solar_multiplier() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn darkness_peaks_at_night_and_vanishes_at_midday() {
        let midnight = DAY_DURATION_SECS + NIGHT_DURATION_SECS * 0.5;
        assert!((DayNightState { time: midnight }.darkness() - 0.35).abs() < 1e-6);

        let midday = DAY_DURATION_SECS * 0.5;
        assert!(DayNightState { time: midday }.darkness().abs() < 1e-6);
    }

    #[test]
    fn is_day_matches_the_sunlight_threshold() {
        let midday = DAY_DURATION_SECS * 0.5;
        assert!(DayNightState { time: midday }.is_day());

        let midnight = DAY_DURATION_SECS + NIGHT_DURATION_SECS * 0.5;
        assert!(!DayNightState { time: midnight }.is_day());
    }

    #[test]
    fn display_formats_minutes_and_seconds_with_phase() {
        assert_eq!(DayNightState { time: 125.0 }.display(), "Day 2:05");
        let midnight = DAY_DURATION_SECS + NIGHT_DURATION_SECS * 0.5;
        assert!(DayNightState { time: midnight }.display().starts_with("Night "));
    }
}
