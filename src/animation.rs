use std::fmt::{self, Formatter};

use bevy::{prelude::*, utils::HashMap};

use crate::{app::AppState, sprite::{Region, Sprite3D, Sprite3DBundle}};

/// Plugin that plays/loops entities with animation components
pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_update(AppState::AppRunning).with_system(update_animations));
    }
}

/// A component consisting of a set of indexed sprite animations
#[derive(Component, Debug, Clone)]
pub struct SpriteAnimationSet {
    animations: HashMap<AnimationHandle, SpriteAnimation>,  // All animations stored
    current_handle: AnimationHandle,                        // Handle to current animation playing/looping
    frame_index: usize,                                     // Current frame in the animation playing/looping
    handle_counter: u32                                     // Counter used for generating unique handle
}

impl SpriteAnimationSet {

    // Creates empty animation set
    pub fn new() -> Self {
        Self {
            animations: HashMap::default(),
            current_handle: AnimationHandle(0),
            frame_index: 0,
            handle_counter: 0
        }
    }

    /// Current frame index of current animation.
    /// 0 if there are no animations present.
    pub fn frame_index(&self) -> usize { self.frame_index }

    /// Sets frame index of current animation
    pub fn set_frame_index(&mut self, frame_index: usize) -> Result<(), AnimationError> {
        let current_anim = &self.animations[&self.current_handle];
        if frame_index > current_anim.0.len() {
            return Err(AnimationError::FrameOutOfBounds);
        }
        self.frame_index = frame_index;
        Ok(())
    }

    /// Advances the animation the specified number of frames
    pub fn advance(&mut self, frames: usize) {
        if frames == 0 { return; }
        if let Some(anim) = self.current_animation() {
            self.frame_index = (self.frame_index + frames) % anim.0.len();
        }
    }

    /// Sets frame index to 0.
    pub fn reset(&mut self) {
        self.frame_index = 0;
    }

    /// Adds an animation, and returns a handle to that animation
    pub fn add_animation(&mut self, animation: SpriteAnimation) -> AnimationHandle {
        let handle = AnimationHandle(self.handle_counter);
        self.animations.insert(handle, animation);
        self.handle_counter += 1;
        handle
    }

    /// Removes an animation
    pub fn remove_animation(&mut self, handle: AnimationHandle) -> Result<SpriteAnimation, AnimationError> {
        if self.handle_counter < handle.0 {
            return Err(AnimationError::NoSuchAnimation);
        }
        Ok(self.animations.remove(&handle).unwrap())
    }

    /// Gets animation reference from handle
    pub fn animation(&self, handle: AnimationHandle) -> Option<&SpriteAnimation> {
        self.animations.get(&handle)
    }

    /// Gets mutable animation reference from handle
    pub fn animation_mut(&mut self, handle: AnimationHandle) -> Option<&mut SpriteAnimation> {
        self.animations.get_mut(&handle)
    }

    /// Gets animation reference from current handle
    pub fn current_animation(&self) -> Option<&SpriteAnimation> {
        self.animation(self.current_handle)
    }

    /// Gets mutable animation reference from current handle
    pub fn current_animation_mut(&mut self) -> Option<&mut SpriteAnimation> {
        self.animation_mut(self.current_handle)
    }

    /// Current frame of current animation
    pub fn current_frame(&self) -> Option<Frame> {
        let anim = self.current_animation()?;
        Some(anim.0[self.frame_index])
    }

    /// Handle to current animation.
    /// None if there are no animations present.
    pub fn current_handle(&self) -> Option<AnimationHandle> {
        if self.animations.is_empty() {
            None
        }
        else {
            Some(self.current_handle)
        }
    }

    /// Sets the current animation to be played/looped
    /// Sets frame to 0.
    pub fn set_animation(&mut self, handle: AnimationHandle) -> Result<(), AnimationError>  {
        if self.handle_counter < handle.0 {
            return Err(AnimationError::NoSuchAnimation);
        }
        self.current_handle = handle;
        Ok(())
    }
}

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

#[derive(Bundle)]
pub struct SpriteAnimationBundle {
    pub animation_set: SpriteAnimationSet,
    pub timer: AnimationTimer,
    #[bundle]
    pub sprite_bundle: Sprite3DBundle
}
impl SpriteAnimationBundle {
    pub fn new(
        animation_set: SpriteAnimationSet,
        timer: AnimationTimer,
        material: Handle<StandardMaterial>,
        transform: Transform,
        global_transform: GlobalTransform
    ) -> Self {
        Self {
            animation_set,
            timer,
            sprite_bundle: Sprite3DBundle {
                sprite: Sprite3D::default(),
                material,
                transform,
                global_transform,
            }
        }
    }
}

/// Frame in an animation
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Frame {
    // Size of frame in pixels
    pub size: Vec2,
    // UV region
    pub region: Region
}

/// A set of frames
#[derive(Debug, Clone, PartialEq)]
pub struct SpriteAnimation(pub Vec<Frame>);
impl SpriteAnimation {

    // Constructs a [`SpriteAnimation`] an image, assuming that the sprites are aligned from left to right, top to bottom with no spacing between.
    pub fn from_grid(
        start_x: u32,
        start_y: u32,
        frame_width: u32,
        frame_height: u32,
        image_width: u32,
        image_height: u32,
        total_frames: u32
    ) -> Self {

        // Simple case
        if total_frames == 0 {
            return Self(Vec::new());
        }

        // Accumultes frames
        let cap = (image_width / frame_width) * (image_height/frame_height);
        let mut frames = Vec::with_capacity(cap as usize);
        let mut y = start_y;
        let sprite_size = Vec2::new(frame_width as f32, frame_height as f32);
        while y + frame_height <= image_height {
            let mut x = start_x;
            let v = y as f32 / image_height as f32;
            let v2 = (y + frame_height) as f32 / image_height as f32;
            while x + frame_width <= image_width {
                let u = x as f32 / image_width as f32;
                let u2 = (x + frame_width) as f32 / image_width as f32;
                let frame = Frame {
                    size: sprite_size,
                    region: Region {
                        min: Vec2::new(u, v),
                        max: Vec2::new(u2, v2)
                    }
                };
                frames.push(frame);

                // If we're past our capacity for frames, quit early with the frames we have
                if frames.len() == total_frames as usize {
                    return Self(frames);
                }
                x += frame_width;
            }
            y += frame_height;
        }

        // Done
        Self(frames)
    }
}

/// Represents a handle to a [`SpriteAnimation`] in a [`SpriteAnimationSet`]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnimationHandle(u32);

/// Updates animation entities
fn update_animations(
    time: Res<Time>,
    mut anim_entities: Query<(&mut Sprite3D, &mut SpriteAnimationSet, &mut AnimationTimer)>
) {
    for (mut sprite, mut anim_set, mut anim_timer) in anim_entities.iter_mut() {
        let timer = &mut anim_timer.0;
        timer.tick(time.delta());
        let times_finished = timer.times_finished();
        if times_finished > 0 {
            anim_set.advance(times_finished as usize);
            if let Some(frame) = anim_set.current_frame() {
                sprite.size = frame.size;
                sprite.region = frame.region;
            }
        }
    }
}


/// Various animation-related errors
#[derive(Debug, Clone)]
pub enum AnimationError {
    NoSuchAnimation,
    FrameOutOfBounds
}
impl fmt::Display for AnimationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NoSuchAnimation => write!(f, "No such animation")?,
            Self::FrameOutOfBounds => write!(f, "Frame out of bounds")?
        }
        Ok(())
    }
}


#[test]
fn test_from_grid() {
    let expected = SpriteAnimation(vec![Frame {
        size: Vec2::new(32.0, 32.0),
        region: Region {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(1.0, 1.0)
        }
    }]);
    let actual = SpriteAnimation::from_grid(32, 32, 32, 32);
    assert_eq!(expected, actual);
}

#[test]
fn test_from_grid_2() {
    let expected = SpriteAnimation(vec![Frame {
        size: Vec2::new(32.0, 32.0),
        region: Region {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(0.50793654, 0.50793654)
        }
    }]);
    let actual = SpriteAnimation::from_grid(32, 32, 63, 63);
    assert_eq!(expected, actual);
}

#[test]
fn test_from_grid_3() {
    let expected = SpriteAnimation(vec![
        Frame {
            size: Vec2::new(32.0, 32.0),
            region: Region {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(0.5, 0.5)
            }
        },
        Frame {
            size: Vec2::new(32.0, 32.0),
            region: Region {
                min: Vec2::new(0.5, 0.0),
                max: Vec2::new(1.0, 0.5)
            }
        },
        Frame {
            size: Vec2::new(32.0, 32.0),
            region: Region {
                min: Vec2::new(0.0, 0.5),
                max: Vec2::new(0.5, 1.0)
            }
        },
        Frame {
            size: Vec2::new(32.0, 32.0),
            region: Region {
                min: Vec2::new(0.5, 0.5),
                max: Vec2::new(1.0, 1.0)
            }
        }
    ]);
    let actual = SpriteAnimation::from_grid(32, 32, 64, 64);
    assert_eq!(expected, actual);
}