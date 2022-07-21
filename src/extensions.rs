use std::path::{ Path, PathBuf };

use bevy::prelude::*;

pub trait PathExt {
    fn relativize(&self, parent: impl AsRef<Path>) -> PathBuf;
}

impl PathExt for Path {
    fn relativize(&self, parent: impl AsRef<Path>) -> PathBuf {
        let mut result = PathBuf::new();
        let mut parent_iter = parent.as_ref().components();
        let mut child_iter = self.components();
        loop {
            let parent_comp = parent_iter.next();
            let child_comp = child_iter.next();
            if let Some(parent_comp) = parent_comp {
                if let Some(child_comp) = child_comp {
                    if parent_comp != child_comp {
                        break;
                    }
                }
            }
            else if let Some(child_comp) = child_comp {
                result.push(child_comp);
                for comp in child_iter {
                    result.push(comp);
                }
                break;
            }
        }
        result
    }
}

impl PathExt for PathBuf {
    fn relativize(&self, parent: impl AsRef<Path>) -> PathBuf {
        self.as_path().relativize(parent)
    }
}

pub trait StandardMaterialExt {
    fn from_image(
        image_file: &str,
        alpha_mode: AlphaMode,
        assets: &AssetServer,
    ) -> StandardMaterial {
        let image_handle = assets.load(image_file);
        StandardMaterial {
            base_color_texture: Some(image_handle.clone()),
            metallic: 0.0,
            reflectance: 0.0,
            perceptual_roughness: 1.0,
            alpha_mode,
            ..Default::default()
        }
    }
}
impl StandardMaterialExt for StandardMaterial {}

pub trait TransformExt {
    fn looking_towards(&self, direction: Vec3, up: Vec3) -> Self;
}
impl TransformExt for Transform {
    fn looking_towards(&self, direction: Vec3, up: Vec3) -> Self {
        let target = self.translation + direction;
        self.looking_at(target, up)
    }
}

pub trait NodeBundleExt {
    fn cbox() -> Self;
    fn vbox(justify: JustifyContent) -> Self;
    fn packed_hbox() -> Self;
}
impl NodeBundleExt for NodeBundle {

    /// Container that centralizes it's content vertically and horizontally.
    /// Stores content left to right.
    fn cbox() -> Self {
        Self {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        }
    }

    /// Vertical container that takes up all the space it can
    fn vbox(justify: JustifyContent) -> Self {
        Self {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: justify,
                align_items: AlignItems::Center,
                ..default()
            },
            color: Color::NONE.into(),
            ..default()
        }
    }

    // Horizontal packed container
    fn packed_hbox() -> Self {
        Self {
            color: Color::NONE.into(),
            ..default()
        }
    }
}

#[test]
fn test_relativize() {
    let parent = PathBuf::from("parent/path");
    let child = PathBuf::from("parent/path/child/path.txt");
    let relativized = child.relativize(parent);
    assert_eq!(PathBuf::from("child/path.txt").as_path(), &relativized);
    let parent = PathBuf::from("parent/pathzzzz");
    let child = PathBuf::from("parent/path/child/path.txt");
    let relativized = child.relativize(parent);
    assert_eq!(PathBuf::from("").as_path(), &relativized);
}