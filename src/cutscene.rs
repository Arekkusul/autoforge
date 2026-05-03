//! Intro cutscene and narrative sequences.
//!
//! The intro plays when a new game starts: a starfield, the ship crashing,
//! and FORGE's first boot-up messages in a friendly, slightly confused tone.
//! The player can skip with any key.

use macroquad::prelude::*;

/// Phases of the intro cutscene.
#[derive(Clone, Debug, PartialEq)]
pub enum CutscenePhase {
    /// Stars drifting — peaceful space.
    Starfield,
    /// Ship alert text.
    Alert,
    /// Crash sequence — screen shake, flash.
    Crash,
    /// Boot-up text — FORGE wakes up.
    Boot,
    /// Done — transition to gameplay.
    Done,
}

/// State for the intro cutscene.
pub struct CutsceneState {
    pub phase: CutscenePhase,
    /// Timer in seconds since the cutscene started.
    pub timer: f32,
    /// Timer within current phase.
    pub phase_timer: f32,
    /// Stars for the starfield effect.
    pub stars: Vec<(f32, f32, f32)>, // x, y, speed
    /// Characters revealed so far in typewriter text.
    pub chars_shown: usize,
    /// Current line index in the boot sequence.
    pub line_index: usize,
    /// Whether the player has pressed skip.
    pub skipped: bool,
}

/// The boot-up dialogue lines (typewriter style).
pub static BOOT_LINES: &[(&str, Color)] = &[
    ("", WHITE), // blank pause
    ("[ SYSTEM RECOVERY INITIATED ]", Color::new(0.4, 0.9, 0.5, 1.0)),
    ("", WHITE),
    ("...", Color::new(0.6, 0.6, 0.6, 1.0)),
    ("", WHITE),
    ("Oh! Hello!", Color::new(0.9, 0.8, 1.0, 1.0)),
    ("", WHITE),
    ("I'm... what am I?", Color::new(0.9, 0.8, 1.0, 1.0)),
    ("", WHITE),
    ("Oh right! I'm FORGE! Your friendly factory AI~", Color::new(0.95, 0.85, 1.0, 1.0)),
    ("", WHITE),
    ("I seem to have... crashed? On a planet?", Color::new(0.9, 0.8, 1.0, 1.0)),
    ("My memory is a bit fuzzy... like 97% fuzzy.", Color::new(0.8, 0.7, 0.9, 1.0)),
    ("", WHITE),
    ("But that's okay! I can figure this out!", Color::new(0.95, 0.9, 1.0, 1.0)),
    ("I just need to build some things...", Color::new(0.9, 0.85, 1.0, 1.0)),
    ("", WHITE),
    ("Let's make something wonderful together! <3", Color::new(1.0, 0.7, 0.85, 1.0)),
    ("", WHITE),
    ("[ Press any key to begin ]", Color::new(0.5, 0.8, 0.5, 1.0)),
];

impl CutsceneState {
    /// Creates a new cutscene ready to play.
    pub fn new() -> Self {
        // Generate starfield.
        let mut stars = Vec::new();
        for i in 0..150 {
            let x = ((i * 7 + 13) % 100) as f32 / 100.0;
            let y = ((i * 11 + 3) % 100) as f32 / 100.0;
            let speed = 0.2 + ((i * 3) % 10) as f32 * 0.1;
            stars.push((x, y, speed));
        }

        Self {
            phase: CutscenePhase::Starfield,
            timer: 0.0,
            phase_timer: 0.0,
            stars,
            chars_shown: 0,
            line_index: 0,
            skipped: false,
        }
    }

    /// Returns true when the cutscene is finished and gameplay should start.
    pub fn is_done(&self) -> bool {
        self.phase == CutscenePhase::Done
    }

    /// Updates the cutscene state. Call once per frame.
    pub fn update(&mut self, dt: f32) {
        self.timer += dt;
        self.phase_timer += dt;

        // Skip on any key/click (but only after boot phase starts, to avoid accidents).
        if self.timer > 2.0 {
            if is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
                || is_key_pressed(KeyCode::Escape)
                || is_mouse_button_pressed(MouseButton::Left)
            {
                if self.phase == CutscenePhase::Boot && self.line_index >= BOOT_LINES.len() - 1 {
                    self.phase = CutscenePhase::Done;
                } else {
                    // Skip to boot phase end.
                    self.phase = CutscenePhase::Boot;
                    self.line_index = BOOT_LINES.len() - 1;
                    self.chars_shown = 999;
                }
                return;
            }
        }

        // Phase transitions.
        match self.phase {
            CutscenePhase::Starfield => {
                if self.phase_timer > 3.0 {
                    self.phase = CutscenePhase::Alert;
                    self.phase_timer = 0.0;
                }
            }
            CutscenePhase::Alert => {
                if self.phase_timer > 2.5 {
                    self.phase = CutscenePhase::Crash;
                    self.phase_timer = 0.0;
                }
            }
            CutscenePhase::Crash => {
                if self.phase_timer > 2.0 {
                    self.phase = CutscenePhase::Boot;
                    self.phase_timer = 0.0;
                    self.line_index = 0;
                    self.chars_shown = 0;
                }
            }
            CutscenePhase::Boot => {
                // Typewriter effect: reveal characters over time.
                if self.line_index < BOOT_LINES.len() {
                    let (text, _) = BOOT_LINES[self.line_index];
                    let target_chars = text.len();

                    if text.is_empty() {
                        // Blank line = pause.
                        if self.phase_timer > 0.4 {
                            self.line_index += 1;
                            self.phase_timer = 0.0;
                            self.chars_shown = 0;
                        }
                    } else {
                        // Typewriter: ~30 chars/sec.
                        self.chars_shown = (self.phase_timer * 30.0) as usize;
                        if self.chars_shown >= target_chars {
                            // Line complete — wait then advance.
                            if self.phase_timer > target_chars as f32 / 30.0 + 1.0 {
                                self.line_index += 1;
                                self.phase_timer = 0.0;
                                self.chars_shown = 0;
                            }
                        }
                    }
                }
            }
            CutscenePhase::Done => {}
        }
    }

    /// Renders the cutscene. Call once per frame (replaces normal game rendering).
    pub fn draw(&self) {
        let sw = screen_width();
        let sh = screen_height();

        // Background: deep space dark.
        clear_background(Color::new(0.02, 0.02, 0.05, 1.0));

        match self.phase {
            CutscenePhase::Starfield | CutscenePhase::Alert => {
                self.draw_starfield(sw, sh);

                if self.phase == CutscenePhase::Alert {
                    // Warning text flashing.
                    let flash = ((self.phase_timer * 3.0).sin() * 0.5 + 0.5).max(0.0);
                    let alert_color = Color::new(1.0, 0.3, 0.2, flash);
                    let text = "!! COLLISION WARNING !!";
                    let w = measure_text(text, None, 36, 1.0).width;
                    draw_text(text, (sw - w) * 0.5, sh * 0.4, 36.0, alert_color);

                    let sub = "TRAJECTORY DEVIATION CRITICAL";
                    let sw2 = measure_text(sub, None, 20, 1.0).width;
                    draw_text(sub, (sw - sw2) * 0.5, sh * 0.4 + 40.0, 20.0, Color::new(1.0, 0.5, 0.3, flash * 0.7));
                }
            }
            CutscenePhase::Crash => {
                // Screen shake + white flash.
                let intensity = (1.0 - self.phase_timer / 2.0).max(0.0);
                let shake_x = (self.timer * 47.0).sin() * 10.0 * intensity;
                let shake_y = (self.timer * 31.0).cos() * 8.0 * intensity;

                // Flash to white then fade.
                let flash = (1.0 - self.phase_timer * 0.8).max(0.0);
                clear_background(Color::new(flash, flash * 0.9, flash * 0.7, 1.0));

                // After flash fades, show crash aftermath text.
                if self.phase_timer > 1.0 {
                    let alpha = ((self.phase_timer - 1.0) * 2.0).min(1.0);
                    let text = "...";
                    let w = measure_text(text, None, 48, 1.0).width;
                    draw_text(
                        text,
                        (sw - w) * 0.5 + shake_x,
                        sh * 0.5 + shake_y,
                        48.0,
                        Color::new(0.7, 0.7, 0.7, alpha),
                    );
                }
            }
            CutscenePhase::Boot => {
                // Dark background with subtle particle effect.
                self.draw_boot_particles(sw, sh);

                // Draw all previously completed lines + current typing line.
                let line_height = 32.0;
                let start_y = sh * 0.25;
                let left_x = sw * 0.15;

                // Show completed lines (up to current).
                let display_start = if self.line_index > 8 { self.line_index - 8 } else { 0 };
                for i in display_start..self.line_index {
                    let (text, color) = BOOT_LINES[i];
                    if !text.is_empty() {
                        let y = start_y + (i - display_start) as f32 * line_height;
                        draw_text(text, left_x, y, 24.0, color);
                    }
                }

                // Draw current line with typewriter.
                if self.line_index < BOOT_LINES.len() {
                    let (text, color) = BOOT_LINES[self.line_index];
                    if !text.is_empty() {
                        let visible: String = text.chars().take(self.chars_shown).collect();
                        let y = start_y + (self.line_index - display_start) as f32 * line_height;
                        draw_text(&visible, left_x, y, 24.0, color);

                        // Blinking cursor.
                        if (self.timer * 2.5).fract() < 0.5 {
                            let cursor_x = left_x + measure_text(&visible, None, 24, 1.0).width + 2.0;
                            draw_text("_", cursor_x, y, 24.0, color);
                        }
                    }
                }

                // Cute FORGE avatar in corner (simple pixel face).
                self.draw_forge_avatar(sw, sh);
            }
            CutscenePhase::Done => {}
        }
    }

    /// Draws the starfield background.
    fn draw_starfield(&self, sw: f32, sh: f32) {
        for (x, y, speed) in &self.stars {
            let sx = (x + self.timer * speed * 0.02) % 1.0;
            let sy = *y;
            let brightness = 0.3 + speed * 0.5;
            let size = 1.0 + speed;
            draw_circle(
                sx * sw,
                sy * sh,
                size,
                Color::new(brightness, brightness, brightness + 0.1, 1.0),
            );
        }
    }

    /// Draws subtle floating particles for the boot screen.
    fn draw_boot_particles(&self, sw: f32, sh: f32) {
        for i in 0..30 {
            let t = self.timer + i as f32 * 1.3;
            let x = ((t * 0.3 + i as f32 * 0.7).sin() * 0.5 + 0.5) * sw;
            let y = ((t * 0.2 + i as f32 * 1.1).cos() * 0.5 + 0.5) * sh;
            let alpha = 0.1 + (t * 0.5).sin().abs() * 0.15;
            draw_circle(x, y, 2.0, Color::new(0.6, 0.5, 0.9, alpha));
        }
    }

    /// Draws a cute little FORGE face in the bottom-right.
    fn draw_forge_avatar(&self, sw: f32, sh: f32) {
        let ax = sw - 100.0;
        let ay = sh - 120.0;
        let size = 60.0;

        // Round body (circle).
        draw_circle(ax, ay, size * 0.5, Color::new(0.3, 0.25, 0.5, 0.9));
        draw_circle(ax, ay, size * 0.45, Color::new(0.4, 0.35, 0.65, 0.9));

        // Eyes (two dots that blink).
        let blink = (self.timer * 0.3).fract() > 0.95;
        if !blink {
            draw_circle(ax - 10.0, ay - 5.0, 5.0, Color::new(0.9, 0.9, 1.0, 1.0));
            draw_circle(ax + 10.0, ay - 5.0, 5.0, Color::new(0.9, 0.9, 1.0, 1.0));
            // Pupils.
            draw_circle(ax - 9.0, ay - 4.0, 2.5, Color::new(0.2, 0.1, 0.4, 1.0));
            draw_circle(ax + 11.0, ay - 4.0, 2.5, Color::new(0.2, 0.1, 0.4, 1.0));
        } else {
            // Blink: horizontal lines.
            draw_line(ax - 14.0, ay - 5.0, ax - 6.0, ay - 5.0, 2.0, Color::new(0.9, 0.9, 1.0, 1.0));
            draw_line(ax + 6.0, ay - 5.0, ax + 14.0, ay - 5.0, 2.0, Color::new(0.9, 0.9, 1.0, 1.0));
        }

        // Cute smile.
        let smile_bounce = (self.timer * 2.0).sin() * 1.0;
        draw_line(
            ax - 8.0, ay + 8.0 + smile_bounce,
            ax, ay + 12.0 + smile_bounce,
            2.0,
            Color::new(1.0, 0.7, 0.8, 0.9),
        );
        draw_line(
            ax, ay + 12.0 + smile_bounce,
            ax + 8.0, ay + 8.0 + smile_bounce,
            2.0,
            Color::new(1.0, 0.7, 0.8, 0.9),
        );

        // Little antenna.
        draw_line(ax, ay - size * 0.45, ax, ay - size * 0.6, 2.0, Color::new(0.6, 0.5, 0.8, 0.8));
        let antenna_glow = (self.timer * 3.0).sin() * 0.3 + 0.7;
        draw_circle(ax, ay - size * 0.6, 4.0, Color::new(0.8, antenna_glow, 1.0, 0.9));

        // Label.
        draw_text("FORGE", ax - 22.0, ay + size * 0.5 + 10.0, 16.0, Color::new(0.7, 0.6, 0.9, 0.8));
    }
}
