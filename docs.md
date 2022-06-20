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

# Frame vs Tick
On the client, every tick is a frame, but not every frame is a tick.
Consider a monitor with a 120hz refresh rate.
Now, if the game's tick rate is 60tps, this means that every frame will redraw the game's graphics (with some interpolation for smoother animations).
However, the game's logic will run roughly every other frame since the refresh rate is 2x the tick rate.
There's always some varition between each frame, of course.

Here is an example of the system's that run during a frame vs a tick.

## Tick + frame
vidya_rust::game: (SYSTEM) ----- update_partial_ticks -----     
vidya_rust::map: (SYSTEM) map_listen                        // Listens for map events
vidya_rust::player: (SYSTEM) keyboard_control_platformer    // Maps keyboard inputs to platformer signals for a player
vidya_rust::platformer: (SYSTEM) process_signals            // Processes the platformer's signals into actions
vidya_rust::platformer: (SYSTEM) control_animations         // Controls platformer's animations based on their state
vidya_rust::physics::movement: (SYSTEM) sync_previous_state // Syncs the previous state of a physics simulation in preparatation for the upcomming update
vidya_rust::physics::movement: (SYSTEM) apply_gravity       // Applies gravity to all velocities
vidya_rust::animation: (SYSTEM) update_animations           // Updates entities with AnimationSets. (Play, loop, etc)
vidya_rust::physics::movement: (SYSTEM) apply_friction      // Applies entity friction to entity velocity before velocity gets applied to entity positions.
vidya_rust::physics::movement: (SYSTEM) apply_velocity      // Applies entity velocity to entity positions
vidya_rust::physics: (SYSTEM) collide_with_terrain          // Applies terrain collision onto entities
vidya_rust::camera: (SYSTEM) camera_target                  // Has camera follow the entith with a "Targettable" component
vidya_rust::graphics: (SYSTEM) interpolate_graphics         // Interpolates entity "Transform"'s with the entity's "Position" and "PreviousPosition". For interpolating graphics.
vidya_rust::sprite: (SYSTEM) draw_sprites                   // Renders Sprite3D components to their respective entity batches. Kinda convoluted.

## Frame only
vidya_rust::player: (SYSTEM) keyboard_control_platformer    
vidya_rust::animation: (SYSTEM) update_animations    
vidya_rust::graphics: (SYSTEM) interpolate_graphics    
vidya_rust::sprite: (SYSTEM) draw_sprites

The "frame only" frames only run a subset of the necessary systems, as they're really only geared towards graphics and input processing.