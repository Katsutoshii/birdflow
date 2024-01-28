use std::f32::consts::PI;

use crate::prelude::*;
use bevy::{
    input::mouse::MouseMotion, prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow,
};

/// Plugin to manage a virtual cursor.
pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorAssets>()
            .add_systems(Update, Cursor::update.in_set(SystemStage::Compute));
    }
}

#[derive(Component, Debug, Default)]
pub struct Cursor;
impl Cursor {
    pub fn update(
        mut cursor: Query<&mut Transform, With<Self>>,
        mut mouse_motion: EventReader<MouseMotion>,
        mut window: Query<&mut Window, With<PrimaryWindow>>,
        //configs: Res<Configs>,
    ) {
        let mut window = window.single_mut();
        let window_size = Vec2 {
            x: window.physical_width() as f32,
            y: window.physical_height() as f32,
        } / window.scale_factor() as f32;

        let mut cursor_transform = cursor.single_mut();
        for &MouseMotion { mut delta } in mouse_motion.read() {
            delta *= Vec2 { x: 1., y: -1. };
            cursor_transform.translation += delta.extend(0.);
        }
        cursor_transform.translation = cursor_transform
            .translation
            .xy()
            .clamp(-0.5 * window_size, 0.5 * window_size)
            .extend(cursor_transform.translation.z);

        let center = Vec2 {
            x: window.width(),
            y: window.height(),
        } / 2.;
        window.set_cursor_position(Some(center))
    }

    pub fn bundle(self, assets: &CursorAssets, translation: Vec3) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2 { x: 10., y: 20. }.extend(1.))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, PI / 4.))
                    .with_translation(translation),
                material: assets.blue_material.clone(),
                ..default()
            },
            self,
        )
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct CursorAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<ColorMaterial>,
}
impl FromWorld for CursorAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(shape::RegularPolygon {
                radius: 2f32.sqrt() / 2.,
                sides: 3,
            }))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            blue_material: materials.add(ColorMaterial::from(Color::ALICE_BLUE.with_a(0.5))),
        }
    }
}
