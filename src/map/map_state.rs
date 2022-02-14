#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MapState {
    /// (1) Listening for map events
    Listening,

    /// (2) Map file (.tmx) is loading.
    /// Blocks remaining states until done.
    /// When finished, gets dependent texture files from map and begins loading them asynchronously.
    LoadingMap,

    /// (3) Map is used for firing map events which will be used to build the world's collision and graphics.
    FiringMapEvents,

    /// (4) Builds collision/graphics (w/o textures) using events fired in (3).
    HandlingMapEvents,

    /// (5) Waits for textures to finish loading from (2).
    FinishingLoadingMapGraphics
}