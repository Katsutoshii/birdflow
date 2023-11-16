use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle},
    window::PrimaryWindow,
};

use crate::MainCamera;

/// State for an individual bird.
#[derive(Component)]
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
        mut sprite_position: Query<(&mut Bird, &mut Transform)>,
    ) {
        let (camera, camera_transform) = q_camera.single();
        for (mut bird, mut transform) in &mut sprite_position {
            if let Some(position) = q_windows
                .single()
                .cursor_position()
                .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
            {
                bird.update_velocity(position, &transform);
            }
            transform.translation += bird.velocity.extend(0.0);
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
        self.velocity = self.velocity.clamp_length_max(10.0);
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
        )
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
