//! Procedural sound effect generation and playback.
//!
//! All sounds are generated as WAV byte arrays at startup — no external audio
//! files needed. Uses simple waveforms (sine, noise, decay envelopes) to create
//! satisfying 8-bit style effects that match the pixel art aesthetic.

use macroquad::audio::{self, Sound, PlaySoundParams};

/// All loaded sound effects.
#[allow(dead_code)]
pub struct SoundEffects {
    pub place: Sound,
    pub remove: Sound,
    pub turret_fire: Sound,
    pub enemy_death: Sound,
    pub recipe_done: Sound,
    pub wave_warning: Sound,
    pub research_done: Sound,
    pub error: Sound,
    /// Master volume (0.0–1.0).
    pub volume: f32,
}

impl SoundEffects {
    /// Generates all sound effects and loads them. Call once at startup.
    pub async fn generate() -> Self {
        Self {
            place: load_wav(&gen_place_sound()).await,
            remove: load_wav(&gen_remove_sound()).await,
            turret_fire: load_wav(&gen_turret_sound()).await,
            enemy_death: load_wav(&gen_death_sound()).await,
            recipe_done: load_wav(&gen_recipe_done_sound()).await,
            wave_warning: load_wav(&gen_wave_warning_sound()).await,
            research_done: load_wav(&gen_research_done_sound()).await,
            error: load_wav(&gen_error_sound()).await,
            volume: 0.5,
        }
    }

    /// Play a sound at the current master volume.
    pub fn play(&self, sound: &Sound) {
        audio::play_sound(sound, PlaySoundParams {
            looped: false,
            volume: self.volume,
        });
    }
}

/// Load a WAV byte array as a Sound.
async fn load_wav(data: &[u8]) -> Sound {
    audio::load_sound_from_bytes(data).await.unwrap()
}

// ============================================================================
// WAV file generation
// ============================================================================

const SAMPLE_RATE: u32 = 22050;

/// Creates a minimal WAV file header + PCM data.
fn make_wav(samples: &[i16]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;
    let mut wav = Vec::with_capacity(file_len as usize + 8);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes());  // PCM format
    wav.extend_from_slice(&1u16.to_le_bytes());  // mono
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes()); // byte rate
    wav.extend_from_slice(&2u16.to_le_bytes());  // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());
    for &s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }

    wav
}

/// Simple sine wave at given frequency, duration, with exponential decay.
fn sine_decay(freq: f32, duration_secs: f32, volume: f32) -> Vec<i16> {
    let n = (SAMPLE_RATE as f32 * duration_secs) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let env = (1.0 - t / duration_secs).max(0.0).powi(2); // quadratic decay
        let val = (t * freq * std::f32::consts::TAU).sin() * env * volume;
        samples.push((val * 32000.0) as i16);
    }
    samples
}

/// White noise burst with decay.
fn noise_decay(duration_secs: f32, volume: f32) -> Vec<i16> {
    let n = (SAMPLE_RATE as f32 * duration_secs) as usize;
    let mut samples = Vec::with_capacity(n);
    let mut rng = 12345u64;
    for i in 0..n {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        let noise = (rng % 65536) as f32 / 32768.0 - 1.0;
        let env = (1.0 - i as f32 / n as f32).powi(2);
        samples.push((noise * env * volume * 32000.0) as i16);
    }
    samples
}

// ============================================================================
// Sound effect generators
// ============================================================================

/// Place building: quick rising two-note chime.
fn gen_place_sound() -> Vec<u8> {
    let mut s = sine_decay(880.0, 0.06, 0.5);
    s.extend(sine_decay(1320.0, 0.08, 0.4));
    make_wav(&s)
}

/// Remove building: low descending thud.
fn gen_remove_sound() -> Vec<u8> {
    let n = (SAMPLE_RATE as f32 * 0.12) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 220.0 - t * 800.0; // descending pitch
        let env = (1.0 - t / 0.12).max(0.0).powi(2);
        let val = (t * freq * std::f32::consts::TAU).sin() * env * 0.5;
        samples.push((val * 32000.0) as i16);
    }
    make_wav(&samples)
}

/// Turret fire: sharp noise pop.
fn gen_turret_sound() -> Vec<u8> {
    make_wav(&noise_decay(0.04, 0.4))
}

/// Enemy death: descending chirp + noise.
fn gen_death_sound() -> Vec<u8> {
    let n = (SAMPLE_RATE as f32 * 0.1) as usize;
    let mut samples = Vec::with_capacity(n);
    let mut rng = 99999u64;
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let freq = 600.0 - t * 3000.0;
        let env = (1.0 - t / 0.1).max(0.0);
        let sine = (t * freq * std::f32::consts::TAU).sin() * 0.3;
        rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17;
        let noise = ((rng % 65536) as f32 / 32768.0 - 1.0) * 0.2;
        samples.push(((sine + noise) * env * 32000.0) as i16);
    }
    make_wav(&samples)
}

/// Recipe completion: pleasant two-note ding (major third).
fn gen_recipe_done_sound() -> Vec<u8> {
    let mut s = sine_decay(660.0, 0.08, 0.3);
    s.extend(sine_decay(830.0, 0.12, 0.25));
    make_wav(&s)
}

/// Wave warning: pulsing alarm tone.
fn gen_wave_warning_sound() -> Vec<u8> {
    let n = (SAMPLE_RATE as f32 * 0.4) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let pulse = ((t * 6.0 * std::f32::consts::TAU).sin() * 0.5 + 0.5).powi(2); // 6 Hz pulse
        let tone = (t * 440.0 * std::f32::consts::TAU).sin();
        let env = (1.0 - t / 0.4).max(0.0);
        samples.push((tone * pulse * env * 0.4 * 32000.0) as i16);
    }
    make_wav(&samples)
}

/// Research complete: triumphant ascending arpeggio.
fn gen_research_done_sound() -> Vec<u8> {
    let notes = [523.0, 659.0, 784.0, 1047.0]; // C5, E5, G5, C6
    let mut s = Vec::new();
    for &freq in &notes {
        s.extend(sine_decay(freq, 0.08, 0.35));
    }
    make_wav(&s)
}

/// Error/can't afford: flat buzz.
fn gen_error_sound() -> Vec<u8> {
    let n = (SAMPLE_RATE as f32 * 0.1) as usize;
    let mut samples = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SAMPLE_RATE as f32;
        let square = if (t * 150.0 * std::f32::consts::TAU).sin() > 0.0 { 1.0f32 } else { -1.0 };
        let env = (1.0 - t / 0.1).max(0.0);
        samples.push((square * env * 0.25 * 32000.0) as i16);
    }
    make_wav(&samples)
}
