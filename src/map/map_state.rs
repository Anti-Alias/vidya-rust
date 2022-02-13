#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MapState {
    /// No map
    Absent,

    /// TMX file is loading
    LoadingMap,

    /// Image files referenced by TMX file are loading
    LoadingMapGraphics,

    /// Map entities (physics/graphics) are being populated
    PopulatingMap,

    // Waits for map graphics to finish loading
    FinishLoadingMapGraphics,

    /// Map is finished and in use
    Finished
}