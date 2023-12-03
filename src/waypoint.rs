use std::f32::consts::PI;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

use crate::{camera, zindex};

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct WaypointPlugin;
impl Plugin for WaypointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointAssets>()
            .register_type::<WaypointConfig>()
            .add_systems(PreStartup, Waypoint::startup)
            .add_systems(FixedUpdate, Waypoint::update);
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
        camera_query: Query<(Entity, &Camera, &GlobalTransform), With<camera::MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        waypoint: Query<Entity, With<Waypoint>>,
        mouse_input: Res<Input<MouseButton>>,
        mut followers: Query<&mut WaypointFollower>,
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
            for mut follower in followers.iter_mut() {
                follower.waypoint_id = Some(waypoint_id);
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

#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct WaypointConfig {
    pub max_acceleration: f32,
    pub repell_radius: f32,
    pub slow_factor: f32,
}
impl Default for WaypointConfig {
    fn default() -> Self {
        Self {
            max_acceleration: 1.0,
            repell_radius: 30.0,
            slow_factor: 0.5,
        }
    }
}

/// For storing the reference to the waypoint.
#[derive(Component, Default, Debug, Clone)]
pub struct WaypointFollower {
    pub waypoint_id: Option<Entity>,
}
impl WaypointFollower {
    pub fn new(waypoint_id: Entity) -> Self {
        Self {
            waypoint_id: Some(waypoint_id),
        }
    }

    pub fn acceleration(
        &self,
        waypoints: &Query<(&Waypoint, &Transform), With<Waypoint>>,
        transform: &Transform,
        velocity: Vec2,
        config: &WaypointConfig,
    ) -> Vec2 {
        if let Some(waypoint_id) = self.waypoint_id {
            let (_waypoint, waypoint_transform) = waypoints
                .get(waypoint_id)
                .expect(&format!("Invalid waypoint ID: {:?}", &self.waypoint_id));

            let position_delta = waypoint_transform.translation.xy() - transform.translation.xy();
            let radius = config.repell_radius;
            let max_magnitude = config.max_acceleration;
            let dist_squared = position_delta.length_squared();
            let radius_squared = radius * radius;
            let magnitude = max_magnitude * (dist_squared / (radius_squared) - 1.);
            let slow_force = config.slow_factor
                * if dist_squared < radius_squared {
                    Vec2::ZERO
                } else {
                    -1.0 * velocity
                };
            return position_delta.normalize_or_zero()
                * magnitude.clamp(-max_magnitude, max_magnitude)
                + slow_force;
        }
        Vec2::ZERO
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
