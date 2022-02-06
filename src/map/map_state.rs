#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MapState {
    Absent,
    LoadingMap,
    LoadingMapGraphics,
    Loaded,
    GraphicsLoading,
    GraphicsLoaded,
    Finished
}