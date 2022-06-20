# Plugins:

## Vidya Plugins:
    VidyaCorePlugin, GraphicsPlugin, SpritePlugin, AnimationPlugin,
    MapPlugin, CameraPlugin, PhysicsPlugin, PlayerPlugin
    
## VidyaCorePlugin
Adds Bevy default plugins. Introduces the PartialTicks resource, which is used for graphical interpolation.
Adds the app's root config.


## MapPlugin
Dictates map loading functionality. It listens for a LoadMapEvent and loads it accordingly.
When loading starts, the game's state will do the following:

* Push MapLoadingFile state (typically starts in the AppRunning state)
* Switch to MapConstructing state
* Switch to MapSpawning state
* Pop the current state (most likely will revert to the AppRunning state)



# Map loading


# Physics

The Terrain struct is used to 
