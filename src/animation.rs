use bevy::prelude::*;

use crate::map::AppState;


pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::AppRunning).with_system(update_animations)
        );
    }
}

#[derive(Default, Clone, Copy, Debug, Reflect)]
pub struct Rect {
    /// The beginning point of the rect
    pub min: Vec2,
    /// The ending point of the rect
    pub max: Vec2,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }
}

/// Animation facade type
pub type SpriteAnimation = Vec<Rect>;

#[derive(Component)]
pub struct SpriteAnimationSet {
    frame_size: Vec2,
    animations: Vec<SpriteAnimation>,
    animation_index: usize,
    frame_index: usize
}

impl SpriteAnimationSet {

    // Creates a new SpriteAnimations
    pub fn new(
        frame_size: Vec2,
        animation_index: usize,
        animations: Vec<SpriteAnimation>
    ) -> Self {
        if animation_index >= animations.len() { panic!("Animation index out of bounds"); }
        for anim in &animations {
            if anim.is_empty() { panic!("Animations must have at least 1 frame"); }
        }
        Self {
            frame_size,
            animations,
            animation_index,
            frame_index: 0,
        }
    }

    /// Size of each frame
    pub fn frame_size(&self) -> Vec2 { self.frame_size }

    // Frames of current animation
    pub fn current_animation(&self) -> &SpriteAnimation {
        &self.animations[self.animation_index]
    }

    /// Current animation index
    pub fn current_animation_index(&self) -> usize { self.animation_index }

    /// Sets animation to play
    pub fn set_animation(&mut self, animation_index: usize) {
        if animation_index >= self.animations.len() { panic!("Animation index out of bounds"); }
        if animation_index != self.animation_index {
            self.animation_index = animation_index;
            self.frame_index = 0;
        }
    }

    /// Current frame of current animation
    pub fn frame(&self) -> Rect {
        let anim = &self.animations[self.animation_index];
        anim[self.frame_index]
    }

    pub fn frame_index(&self) -> usize { self.frame_index }

    /// Sets frame index of current animation
    pub fn set_frame(&mut self, frame_index: usize) {
        let frame_count = &self.animations[self.animation_index].len();
        self.frame_index = frame_index % frame_count;
    }
}

/// Timer that is use to update a SpriteAnimationSet
#[derive(Component)]
pub struct AnimationTimer(Timer);

/// Updates entities that have a SpriteAnimationSet
pub fn update_animations(
    mut query: Query<(
        &mut SpriteAnimationSet,
        &mut Handle<Mesh>,
        &mut AnimationTimer
    )>,
    time: Res<Time>
){

    // For all animations
    for (mut animation_set, mut mesh_handle, mut timer) in query.iter_mut() {

        // Go to next frame if it's time
        let timer = &mut timer.0;
        timer.tick(time.delta());
        for _ in 0..timer.times_finished() {

            // Goes to next frame
            let frame_count = animation_set.current_animation().len();
            animation_set.frame_index = (animation_set.frame_index + 1) % frame_count;
            
            // Updates mesh UVs
            
        }
    }
}