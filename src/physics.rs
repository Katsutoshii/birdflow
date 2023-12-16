use bevy::prelude::*;

use crate::{grid::GridSpec, SystemStage};

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, update.in_set(SystemStage::Apply));
    }
}

/// Tracks velocity per entity.
#[derive(Component, Debug, Default, Clone)]
pub struct Velocity(pub Vec2);

/// Tracks new velocity per entity, which can be used for double-buffering
/// velocity updates.
#[derive(Component, Debug, Default, Clone)]
pub struct NewVelocity(pub Vec2);

/// Apply velocity changes.
pub fn update(
    mut query: Query<(&mut Velocity, &NewVelocity, &mut Transform)>,
    grid_spec: Res<GridSpec>,
) {
    for (mut velocity, new_velocity, mut transform) in &mut query {
        velocity.0 = new_velocity.0;
        transform.translation += velocity.0.extend(0.);
        grid_spec
            .world2d_bounds()
            .clamp3(&mut transform.translation)
    }
}

#[derive(Bundle, Clone, Default)]
pub struct PhysicsBundle {
    pub velocity: Velocity,
    pub new_velocity: NewVelocity,
}
