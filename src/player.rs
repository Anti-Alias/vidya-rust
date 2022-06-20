use bevy::prelude::*;

use crate::game::{GameState, SystemLabels};
use crate::direction::{Direction, CardinalDirection};
use crate::platformer::{Platformer, PlatformerSignal};

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(GameState::GameRunning)
            .with_system(control_with_keyboard
                .label(SystemLabels::Input)
                .after(SystemLabels::TickStart)
            )
        );
    }
}

/// Tags an entity as being a Player that receives input from the keyboard, mouse, controller, etc
#[derive(Component, Debug, Clone)]
pub struct Player;

/// Emits platformer signals based on keyboard/controller input
fn control_with_keyboard(
    input: Res<Input<KeyCode>>,
    mut player_entities: Query<&mut Platformer, With<Player>>
) {
    log::debug!("(SYSTEM) keyboard_control_platformer");
    for mut platformer in player_entities.iter_mut() {
        
        // Reads arrow keys and determines which way to "move"
        match Direction::from_keyboard(&input) {
            Some(direction) => {
                let radians = direction.to_radians();
                platformer.signals.push(PlatformerSignal::Move { direction: radians });
            },
            None => {}
        };

        // Reads arrow keys and determines which way to "look"
        match CardinalDirection::from_keyboard(&input) {
            Some(direction) => {
                let radians = direction.to_radians();
                platformer.signals.push(PlatformerSignal::Look { direction: radians });
            }
            None => {}
        }

        // Reads spacebar and jumps if its pressed
        if input.just_pressed(KeyCode::Space) {
            platformer.signals.push(PlatformerSignal::Jump);
        }
    }
}