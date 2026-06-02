//! Camera system for panning and zooming the game world.
//!
//! Wraps a 2D camera with smooth pan (WASD / middle-mouse drag) and
//! zoom (scroll wheel, zooms toward cursor). Provides screen ↔ world
//! coordinate conversion.

use macroquad::prelude::*;

use crate::constants::*;

/// Tracks the camera's world-space target and zoom level.
pub struct GameCamera {
    /// World position the camera is centered on.
    pub target: Vec2,
    /// Zoom multiplier — 1.0 means 1 world pixel = 1 screen pixel.
    pub zoom: f32,
    /// Whether the map overlay (M key) is showing.
    pub map_view: bool,
    /// Last mouse position for drag-pan delta calculation.
    last_mouse: Vec2,
    /// Whether middle mouse was down last frame.
    middle_was_down: bool,
}

impl GameCamera {
    /// Creates a new camera centered on the middle of the world.
    pub fn new() -> Self {
        Self {
            target: Vec2::new(
                GRID_WIDTH as f32 * TILE_SIZE * 0.5,
                GRID_HEIGHT as f32 * TILE_SIZE * 0.5,
            ),
            zoom: 1.0,
            map_view: false,
            last_mouse: Vec2::ZERO,
            middle_was_down: false,
        }
    }

    /// Updates pan and zoom based on input. Call once per frame with frame delta time.
    pub fn update(&mut self, dt: f32) {
        // Cap dt to prevent camera sliding when FPS is low.
        let dt = dt.min(0.05); // max 50ms per frame for camera (effectively 20 FPS floor)

        // --- WASD / arrow key pan ---
        let mut pan = Vec2::ZERO;
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            pan.y -= 1.0;
        }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            pan.y += 1.0;
        }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            pan.x -= 1.0;
        }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            pan.x += 1.0;
        }
        if pan != Vec2::ZERO {
            pan = pan.normalize();
        }
        self.target += pan * PAN_SPEED * dt / self.zoom;

        // --- Middle mouse drag pan ---
        let current_mouse = Vec2::new(mouse_position().0, mouse_position().1);
        if is_mouse_button_down(MouseButton::Middle) {
            if self.middle_was_down {
                let delta = current_mouse - self.last_mouse;
                self.target -= delta / self.zoom;
            }
            self.middle_was_down = true;
        } else {
            self.middle_was_down = false;
        }
        self.last_mouse = current_mouse;

        // --- Scroll wheel zoom (toward cursor) ---
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            let mouse_world_before = self.screen_to_world(current_mouse);

            self.zoom = (self.zoom * (1.0 + wheel_y.signum() * ZOOM_SPEED)).clamp(ZOOM_MIN, ZOOM_MAX);

            // Adjust target so the point under the cursor stays fixed.
            let mouse_world_after = self.screen_to_world(current_mouse);
            self.target += mouse_world_before - mouse_world_after;
        }

        // --- Toggle map view ---
        if is_key_pressed(KeyCode::M) {
            self.map_view = !self.map_view;
        }
        if self.map_view && is_key_pressed(KeyCode::Escape) {
            self.map_view = false;
        }
    }

    /// Converts a screen-space position to world-space coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
        self.target + (screen_pos - screen_center) / self.zoom
    }

    /// Converts a world-space position to screen-space coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let screen_center = Vec2::new(screen_width() * 0.5, screen_height() * 0.5);
        screen_center + (world_pos - self.target) * self.zoom
    }

    /// Builds the macroquad [`Camera2D`] for world-space rendering.
    ///
    /// macroquad's Camera2D zoom maps world units to NDC [-1, 1], so we compute
    /// `zoom_ndc = zoom * 2.0 / screen_dimension`.
    pub fn to_macroquad_camera(&self) -> Camera2D {
        Camera2D {
            target: self.target,
            zoom: vec2(
                self.zoom * 2.0 / screen_width(),
                self.zoom * 2.0 / screen_height(),
            ),
            ..Default::default()
        }
    }

    /// Returns the visible world-space rectangle as `(min, max)` in world coordinates.
    ///
    /// Useful for frustum culling — only draw tiles/entities within this bounds.
    pub fn visible_bounds(&self) -> (Vec2, Vec2) {
        let half_screen = Vec2::new(screen_width() * 0.5, screen_height() * 0.5) / self.zoom;
        (self.target - half_screen, self.target + half_screen)
    }
}
