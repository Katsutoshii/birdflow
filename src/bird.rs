use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle},
};

use crate::{
    grid::EntityGrid,
    physics::{NewVelocity, Velocity},
    waypoint::{Waypoint, WaypointFollower},
    zindex, SystemStage,
};

/// Plugin for running birds.
pub struct BirdsPlugin;
impl Plugin for BirdsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Bird>()
            .register_type::<BirdSpawner>()
            .init_resource::<BirdAssets>()
            .configure_sets(FixedUpdate, SystemStage::get_config())
            .add_systems(
                FixedUpdate,
                (
                    BirdSpawner::spawn.in_set(SystemStage::Spawn),
                    Bird::update_velocity.in_set(SystemStage::Compute),
                    Bird::apply_velocity.in_set(SystemStage::Apply),
                    BirdSpawner::despawn.in_set(SystemStage::Despawn),
                ),
            );
    }
}

/// State for an individual bird.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bird {
    pub theta: f32,
    pub max_velocity: f32,
}
impl Default for Bird {
    fn default() -> Self {
        Self {
            theta: 0.0,
            max_velocity: 10.0,
        }
    }
}
impl Bird {
    pub fn update_velocity(
        mut birds: Query<(
            Entity,
            &Bird,
            &Velocity,
            &mut NewVelocity,
            &Transform,
            &WaypointFollower,
        )>,
        waypoints: Query<(&Waypoint, &Transform), Without<Bird>>,
        _grid: Res<EntityGrid>,
    ) {
        for (_entity, bird, _velocity, mut new_velocity, transform, follower) in &mut birds {
            let mut acceleration = Vec2::ZERO;
            if let Some(waypoint_id) = follower.waypoint_id {
                let (_waypoint, waypoint_transform) = waypoints
                    .get(waypoint_id)
                    .expect(&format!("Invalid waypoint ID: {:?}", &follower.waypoint_id));
                acceleration += bird.get_waypoint_acceleration(
                    waypoint_transform.translation.truncate(),
                    &transform,
                );
            } else {
                acceleration = Vec2::ZERO;
            }

            new_velocity.0 += acceleration;
            new_velocity.0 = new_velocity.0.clamp_length_max(bird.max_velocity);
        }
    }

    pub fn apply_velocity(
        mut birds: Query<(Entity, &mut Velocity, &NewVelocity, &mut Transform), With<Self>>,
        mut grid: ResMut<EntityGrid>,
    ) {
        for (entity, mut velocity, new_velocity, mut transform) in &mut birds {
            velocity.0 = new_velocity.0;
            transform.translation += velocity.0.extend(0.);
            grid.update(entity, transform.translation.truncate());
        }
    }

    fn get_waypoint_acceleration(&self, cursor_position: Vec2, transform: &Transform) -> Vec2 {
        let mut delta = cursor_position - transform.translation.xy();
        let rotation_mat = Mat2::from_angle(self.theta);
        delta = rotation_mat * delta;

        if delta.length_squared() < 2500.0 {
            delta = -50.0 / delta.clamp_length_min(0.1)
        }
        delta.normalize_or_zero() * 0.5
    }
}

/// Creates bundle for the Bird with its associated material mesh.
#[derive(Default)]
pub struct BirdBundler<M: Material2d> {
    pub bird: Bird,
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
    pub translation: Vec3,
    pub follower: WaypointFollower,
}
impl<M: Material2d> BirdBundler<M> {
    pub fn bundle(self) -> impl Bundle {
        (
            self.bird,
            Velocity::default(),
            NewVelocity::default(),
            self.follower,
            MaterialMesh2dBundle::<M> {
                mesh: self.mesh.into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(10.0))
                    .with_translation(self.translation),
                material: self.material,
                ..default()
            },
            Name::new("Bird"),
        )
    }
}

/// Handles to common bird assets.
#[derive(Resource)]
pub struct BirdAssets {
    pub mesh: Handle<Mesh>,
    pub green_material: Handle<ColorMaterial>,
    pub tomato_material: Handle<ColorMaterial>,
}
impl FromWorld for BirdAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(shape::Circle::default()))
        };
        let (green_material, tomato_material) = {
            let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
            (
                materials.add(ColorMaterial::from(Color::LIME_GREEN)),
                materials.add(ColorMaterial::from(Color::TOMATO)),
            )
        };
        Self {
            mesh,
            green_material,
            tomato_material,
        }
    }
}

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct BirdSpawner {
    num_birds: usize,
    theta_factor: f32,
    translation_factor: f32,
    max_velocity: f32,
}
impl Default for BirdSpawner {
    fn default() -> Self {
        Self {
            num_birds: 40,
            theta_factor: 0.001,
            translation_factor: 10.0,
            max_velocity: 10.0,
        }
    }
}
impl BirdSpawner {
    /// System to spawn birds on left mouse button.
    pub fn spawn(
        mut commands: Commands,
        spawner: Res<Self>,
        assets: Res<BirdAssets>,
        keyboard: Res<Input<KeyCode>>,
        waypoint: Query<Entity, With<Waypoint>>,
    ) {
        if !keyboard.just_pressed(KeyCode::B) {
            return;
        }

        let waypoint_id = waypoint.single();

        for i in 1..(spawner.num_birds / 2 + 1) {
            let zindex =
                zindex::BIRDS_MIN + (i as f32) * 0.00001 * (zindex::BIRDS_MAX - zindex::BIRDS_MIN);

            commands.spawn(
                BirdBundler {
                    bird: Bird {
                        theta: PI * spawner.theta_factor * (i as f32),
                        max_velocity: spawner.max_velocity,
                        ..default()
                    },
                    mesh: assets.mesh.clone(),
                    material: assets.green_material.clone(),
                    translation: (Vec3::X + Vec3::Y) * spawner.translation_factor * (i as f32)
                        + Vec3::Z * zindex,
                    follower: WaypointFollower::new(waypoint_id),
                }
                .bundle(),
            );
            commands.spawn(
                BirdBundler {
                    bird: Bird {
                        theta: PI * spawner.theta_factor * (i as f32),
                        max_velocity: spawner.max_velocity,
                        ..default()
                    },
                    mesh: assets.mesh.clone(),
                    material: assets.tomato_material.clone(),
                    translation: -(Vec3::X + Vec3::Y) * spawner.translation_factor * (i as f32)
                        + Vec3::Z * zindex,
                    follower: WaypointFollower::new(waypoint_id),
                }
                .bundle(),
            );
        }
    }

    /// System to despawn all birds.
    pub fn despawn(
        birds: Query<Entity, With<Bird>>,
        mut commands: Commands,
        mut grid: ResMut<EntityGrid>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        if !keyboard_input.just_pressed(KeyCode::D) {
            return;
        }
        for entity in &birds {
            grid.remove(entity);
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Bird;
    use bevy::prelude::*;

    #[test]
    fn test_update() {
        let bird = Bird {
            ..Default::default()
        };
        let cursor_position = Vec2 { x: 10.0, y: 10.0 };
        let mut transform = Transform::default();
        let velocity = bird.get_waypoint_acceleration(cursor_position, &mut transform);
        println!("velocity: {:?}", velocity);
    }
}
