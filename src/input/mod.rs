use crate::game::GameState;
use crate::sound;

mod alerts;
mod hotkeys;
mod overlay_clicks;
mod placement;

pub use alerts::check_alerts;

use hotkeys::handle_hotkeys;
use overlay_clicks::handle_overlay_and_toolbar_clicks;
use placement::handle_placement_and_removal;

/// Handles player input for building selection, placement, and hotkeys.
pub fn handle_input(state: &mut GameState, sfx: &mut sound::SoundEffects) {
    if handle_hotkeys(state, sfx) {
        return;
    }
    if handle_overlay_and_toolbar_clicks(state) {
        return;
    }
    handle_placement_and_removal(state, sfx);
}
