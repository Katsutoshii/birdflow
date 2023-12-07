use std::f32::consts::PI;

use crate::prelude::*;
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

use super::chaser::Chaser;

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct WaypointPlugin;
impl Plugin for WaypointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointAssets>()
            .add_systems(PreStartup, Waypoint::startup)
            .add_systems(FixedUpdate, Waypoint::update.in_set(SystemStage::Compute));
    }
}

#[derive(Component, Debug)]
pub struct Waypoint {
    pub active: bool,
    pub size: f32,
}
impl Default for Waypoint {
    fn default() -> Self {
        Self {
            active: false,
            size: 10.0,
        }
    }
}
impl Waypoint {
    pub fn startup(mut commands: Commands, assets: Res<WaypointAssets>) {
        commands.spawn(Waypoint::default().bundle(&assets));
    }

    pub fn update(
        mut query: Query<(&Self, &mut Transform)>,
        camera_query: Query<(Entity, &Camera, &GlobalTransform), With<MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        waypoint: Query<Entity, With<Waypoint>>,
        mouse_input: Res<Input<MouseButton>>,
        mut followers: Query<(&Selected, &mut Chaser)>,
    ) {
        if !mouse_input.pressed(MouseButton::Right) {
            return;
        }
        let (_camera_entity, camera, camera_transform) = camera_query.single();
        if let Some(position) = window_query
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        {
            let (_waypoint, mut waypoint_transform) = query.single_mut();
            waypoint_transform.translation = position.extend(zindex::WAYPOINT);

            let waypoint_id = waypoint.single();

            for (selected, mut follower) in followers.iter_mut() {
                if selected.is_selected() {
                    follower.target_entity = Some(waypoint_id);
                } else {
                    follower.target_entity = None;
                }
            }

            // Debug positions
            if mouse_input.just_pressed(MouseButton::Right) {
                info!("Clicked on position: {}", position);
            }
        }
    }

    pub fn bundle(self, assets: &WaypointAssets) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(self.size))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, PI))
                    .with_translation(Vec3 {
                        x: 0.,
                        y: 0.,
                        z: zindex::WAYPOINT,
                    }),
                material: assets.blue_material.clone(),
                ..default()
            },
            self,
        )
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct WaypointAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<ColorMaterial>,
}
impl FromWorld for WaypointAssets {
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
            blue_material: materials.add(ColorMaterial::from(Color::TURQUOISE.with_a(0.5))),
        }
    }
}
