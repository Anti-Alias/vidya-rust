// Fired to kick off map loading
#[derive(Debug, Clone)]
pub struct LoadMapEvent(pub String);

// Fired when map has fully spawned
#[derive(Debug, Clone)]
pub struct MapSpawnedEvent(pub String);