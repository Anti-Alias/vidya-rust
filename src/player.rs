use bevy::prelude::*;

use crate::app::{AppState, AppLabel};
use crate::being::Direction;
use crate::platformer::{Platformer, PlatformerSignal};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::AppRunning)
            .with_system(emit_platformer_signals
                .label(AppLabel::Input)
                .after(AppLabel::TickStart)
            )
        );
    }
}

/// Tags an entity as being a Player that receives input from the keyboard, mouse, controller, etc
#[derive(Component, Debug, Clone)]
pub struct Player;

/// Emits platformer signals based on keyboard/controller input
fn emit_platformer_signals(
    input: Res<Input<KeyCode>>,
    mut player_entities: Query<&mut Platformer, With<Player>>
) {
    log::debug!("(SYSTEM) emit_platformer_signals");
    for mut platformer in player_entities.iter_mut() {
        let direction = match Direction::from_keyboard(&input) {
            Some(direction) => direction,
            None => continue
        };
        let radians = direction.to_radians();
        platformer.signals.push(PlatformerSignal::Move { direction: radians });
    }
}