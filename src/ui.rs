use bevy::prelude::*;

/// Keeps track of node layers where UI entities may be placed.
pub struct UiLayers {
    /// Screen-sized layer that contains all nodes associated with transitions between screens.
    /// Useful for loading screens, fade-to-black transitions, etc.
    pub transition_layer: Entity,

    /// Screen-sized UI layer
    pub ui_layer: Entity
}