use bevy::reflect::TypeUuid;
use bevy::prelude::*;
use tiled::{Map, parse_with_path};
use crate::map::{CurrentMap, MapState};


// Helper function for navigating to other maps
pub fn goto_map(
    map_name: &str,
    mut map_state: ResMut<State<MapState>>,
    current_map: Option<ResMut<CurrentMap>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands
) {


}