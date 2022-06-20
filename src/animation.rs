use std::{fmt, time::Duration};

use bevy::prelude::*;

use crate::game::{GameState, SystemLabels};
use crate::sprite::{Region, Sprite3D, Sprite3DBundle};

/// Plugin that plays/loops entities with animation components
pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(GameState::GameRunning)
                .label(SystemLabels::UpdateAnimations)
                .after(SystemLabels::ControlAnimations)
                .with_system(update_animations)
        );
    }
}

/// A component consisting of a set of indexed sprite animations
#[derive(Component, Debug, Clone)]
pub struct AnimationSet {
    animations: Vec<Animation>,         // All animations stored
    groups: Vec<Vec<AnimationHandle>>,  // All animation groups stored
    current_animation: AnimationHandle, // Handle to current animation playing/looping
    frame: usize                        // Current frame in the animation playing/looping
}

impl AnimationSet {

    // Creates empty animation set
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
            groups: Vec::new(),
            current_animation: AnimationHandle(0),
            frame: 0
        }
    }

    /// Current frame index of current animation.
    /// 0 if there are no animations present.
    pub fn frame_index(&self) -> usize { self.frame }

    /// Sets frame index of current animation
    pub fn set_frame_index(&mut self, frame_index: usize) -> Result<(), AnimationError> {
        match self.current_animation() {
            Some(anim) => {
                if frame_index >= anim.frames.len() {
                    return Err(AnimationError::FrameOutOfBounds);
                }
            },
            None => {
                if frame_index != 0 {
                    return Err(AnimationError::FrameOutOfBounds);
                }
            }
        }
        self.frame = frame_index;
        Ok(())
    }

    /// Advances the animation the specified number of frames
    pub fn advance(&mut self, frames: usize) {
        if frames == 0 { return; }
        if let Some(anim) = self.current_animation() {
            self.frame = (self.frame + frames) % anim.frames.len();
        }
    }

    /// Sets frame index to 0.
    pub fn reset(&mut self) {
        self.frame = 0;
    }

    /// Adds an animation, and returns a handle to that animation
    pub fn add_animation(&mut self, animation: Animation) -> AnimationHandle {
        let len = self.animations.len();
        self.animations.push(animation);
        AnimationHandle(len)
    }

    /// Adds a group of animations
    pub fn add_animation_group(&mut self, group: &[Animation]) -> AnimationGroupHandle {
        let start = self.animations.len();
        let end = start + group.len();
        let group_anim_handles: Vec<AnimationHandle> = (start..end)
            .map(|idx| AnimationHandle(idx))
            .collect();
        self.animations.extend_from_slice(group);
        let group_handle = AnimationGroupHandle(self.groups.len());
        self.groups.push(group_anim_handles);
        group_handle
    }

    /// Gets animation reference from handle
    pub fn animation(&self, handle: AnimationHandle) -> Option<&Animation> {
        self.animations.get(handle.0)
    }

    /// Gets mutable animation reference from handle
    pub fn animation_mut(&mut self, handle: AnimationHandle) -> Option<&mut Animation> {
        self.animations.get_mut(handle.0)
    }

    /// Gets animation reference from current handle
    pub fn current_animation(&self) -> Option<&Animation> {
        self.animation(self.current_animation)
    }

    /// Gets mutable animation reference from current handle
    pub fn current_animation_mut(&mut self) -> Option<&mut Animation> {
        self.animation_mut(self.current_animation)
    }

    /// Current frame of current animation
    pub fn current_frame_info(&self) -> Option<FrameInfo> {
        let anim = self.current_animation()?;
        Some(FrameInfo {
            frame: anim.frames[self.frame],
            offset: anim.offset
        })
    }

    /// Handle to current animation.
    /// None if there are no animations present.
    pub fn current_handle(&self) -> Option<AnimationHandle> {
        if self.animations.is_empty() {
            None
        }
        else {
            Some(self.current_animation)
        }
    }

    /// Sets the current animation to be played/looped
    /// Sets frame to 0.
    pub fn set_animation(&mut self, handle: AnimationHandle) -> Result<(), AnimationError>  {
        if handle.0 >= self.animations.len() {
            return Err(AnimationError::NoSuchAnimation);
        }
        self.frame = 0;
        self.current_animation = handle;
        Ok(())
    }

    /// Gets the animation handle of a grouped animation
    pub fn get_grouped_animation_handle(&self, group_handle: AnimationGroupHandle, index: usize) -> Result<AnimationHandle, AnimationError> {
        if group_handle.0 >= self.groups.len() {
            return Err(AnimationError::NoSuchAnimation);
        }
        let group = &self.groups[group_handle.0];
        if index >= group.len() {
            return Err(AnimationError::NoSuchAnimation);
        }
        Ok(group[index])
    }

    /// Sets the current animation to be played/looped based on a group handle and index
    /// Sets frame to 0.
    pub fn set_grouped_animation(
        &mut self,
        group_handle: AnimationGroupHandle,
        index: usize,
        preserve_frame_index: bool
    ) -> Result<(), AnimationError> {

        // Switches animation
        let next_anim_handle = self.get_grouped_animation_handle(group_handle, index)?;

        // Determines what to do with current frame index
        if preserve_frame_index {
            let next_anim = &self.animations[next_anim_handle.0];
            if self.frame > next_anim.frames.len() {
                self.frame = 0;
            }
        }
        else if next_anim_handle != self.current_animation {
            self.frame = 0;
        }

        // Done
        self.current_animation = next_anim_handle;
        Ok(())
    }
}

#[derive(Component)]
pub struct AnimationTimer(Timer);
impl AnimationTimer {
    pub fn new(frame_time: Duration) -> Self {
        Self(Timer::new(frame_time, true))
    }
}

/// Bundle for an AnimationSet + auxiliary components.
#[derive(Bundle)]
pub struct AnimationSetBundle {
    /// Set of animations available to the entity
    pub animation_set: AnimationSet,

    /// Timer used to power all of the animations
    pub timer: AnimationTimer,

    /// Sprite3D bundle. The Sprite3D component will be continuously written to when frames are updated.
    #[bundle]
    pub sprite_bundle: Sprite3DBundle
}
impl AnimationSetBundle {
    pub fn new(
        animation_set: AnimationSet,
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
    pub region: Region,
}

/// Represents a frame with extra metadata
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FrameInfo {
    frame: Frame,
    offset: Vec3
}

/// A set of frames
#[derive(Debug, Clone, PartialEq)]
pub struct Animation {
    pub frames: Vec<Frame>,
    pub offset: Vec3
}
impl Animation {

    // Constructs a [`SpriteAnimation`] an image, assuming that the sprites are aligned from left to right, top to bottom with no spacing between.
    pub fn from_grid(
        start_x: u32,
        start_y: u32,
        frame_width: u32,
        frame_height: u32,
        image_width: u32,
        image_height: u32,
        total_frames: u32,
        offset: Vec3
    ) -> Self {

        // Simple case
        if total_frames == 0 {
            return Self {
                frames: Vec::new(),
                offset
            };
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
                    return Self {
                        frames,
                        offset
                    };
                }
                x += frame_width;
            }
            y += frame_height;
        }

        // Done
        return Self {
            frames,
            offset
        };
    }
}

/// Represents a handle to an [`Animation`] in an [`AnimationSet`]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnimationHandle(usize);

/// Represents a handle to a group of animation variants in an [`AnimationSet`]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AnimationGroupHandle(usize);

/// Updates animation entities
fn update_animations(
    time: Res<Time>,
    mut anim_entities: Query<(&mut Sprite3D, &mut AnimationSet, &mut AnimationTimer)>
) {
    log::debug!("(SYSTEM) update_animations");
    for (mut sprite, mut anim_set, mut anim_timer) in anim_entities.iter_mut() {
        let timer = &mut anim_timer.0;
        timer.tick(time.delta());
        let times_finished = timer.times_finished();
        if times_finished > 0 {
            anim_set.advance(times_finished as usize);
            if let Some(info) = anim_set.current_frame_info() {
                sprite.size = info.frame.size;
                sprite.region = info.frame.region;
                sprite.offset = info.offset;
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
    let expected = Animation{
        frames: vec![Frame {
            size: Vec2::new(32.0, 32.0),
            region: Region {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(1.0, 1.0)
            },
        }],
        offset: Vec3::ZERO
    };
    let actual = Animation::from_grid(0, 0, 32, 32, 32, 32, 100, Vec3::ZERO);
    assert_eq!(expected, actual);
}

#[test]
fn test_from_grid_2() {
    let expected = Animation {
        frames: vec![Frame {
            size: Vec2::new(32.0, 32.0),
            region: Region {
                min: Vec2::new(0.0, 0.0),
                max: Vec2::new(0.50793654, 0.50793654)
            }
        }],
        offset: Vec3::ZERO
    };
    let actual = Animation::from_grid(0, 0, 32, 32, 63, 63, 100, Vec3::ZERO);
    assert_eq!(expected, actual);
}

#[test]
fn test_from_grid_3() {
    let expected = Animation {
        frames: vec![
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
        ],
        offset: Vec3::ZERO
    };
    let actual = Animation::from_grid(0, 0, 32, 32, 64, 64, 100, Vec3::ZERO);
    assert_eq!(expected, actual);
}