use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle},
    window::PrimaryWindow,
};

use crate::{grid::EntityGrid, MainCamera};

/// Plugin for running birds.
pub struct BirdsPlugin;
impl Plugin for BirdsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Bird>()
            .register_type::<BirdSpawner>()
            .init_resource::<BirdAssets>()
            .add_systems(
                FixedUpdate,
                (BirdSpawner::spawn, BirdSpawner::despawn, Bird::update),
            );
    }
}

/// State for an individual bird.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Bird {
    pub velocity: Vec2,
    pub theta: f32,
    pub max_velocity: f32,
}
impl Default for Bird {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            theta: 0.0,
            max_velocity: 10.0,
        }
    }
}
impl Bird {
    pub fn update(
        q_windows: Query<&Window, With<PrimaryWindow>>,
        // query to get camera transform
        q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        mut sprite_position: Query<(Entity, &mut Bird, &mut Transform)>,
        mut grid: ResMut<EntityGrid>,
    ) {
        let (camera, camera_transform) = q_camera.single();
        for (entity, mut bird, mut transform) in &mut sprite_position {
            if let Some(position) = q_windows
                .single()
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
            {
                bird.update_velocity(position, &transform);
            } else {
                bird.velocity = Vec2::ZERO;
            }
            transform.translation += bird.velocity.extend(0.0);
            grid.update(entity, transform.translation.truncate());
        }
    }

    fn update_velocity(&mut self, cursor_position: Vec2, transform: &Transform) {
        let mut delta = cursor_position - transform.translation.xy();
        let rotation_mat = Mat2::from_angle(self.theta);
        delta = rotation_mat * delta;

        if delta.length_squared() < 2500.0 {
            delta = -50.0 / delta.clamp_length_min(0.1)
        }
        self.velocity += delta.normalize_or_zero() * 0.5;
        self.velocity = self.velocity.clamp_length_max(self.max_velocity);
    }
}

/// Creates bundle for the Bird with its associated material mesh.
#[derive(Default)]
pub struct BirdBundler<M: Material2d> {
    pub bird: Bird,
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
    pub translation: Vec3,
}
impl<M: Material2d> BirdBundler<M> {
    pub fn bundle(self) -> impl Bundle {
        (
            self.bird,
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
        buttons: Res<Input<MouseButton>>,
    ) {
        if !buttons.just_pressed(MouseButton::Left) {
            return;
        }

        for i in 1..(spawner.num_birds / 2 + 1) {
            commands.spawn(
                BirdBundler {
                    bird: Bird {
                        theta: PI * spawner.theta_factor * (i as f32),
                        max_velocity: spawner.max_velocity,
                        ..default()
                    },
                    mesh: assets.mesh.clone(),
                    material: assets.green_material.clone(),
                    translation: (Vec3::X + Vec3::Y) * spawner.translation_factor * (i as f32),
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
                    translation: -(Vec3::X + Vec3::Y) * spawner.translation_factor * (i as f32),
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
        let mut bird = Bird {
            velocity: Vec2::ZERO,
            ..Default::default()
        };
        let cursor_position = Vec2 { x: 10.0, y: 10.0 };
        let mut transform = Transform::default();
        bird.update_velocity(cursor_position, &mut transform);
        println!("translation: {:?}", transform.translation);
    }
}
