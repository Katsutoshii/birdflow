use bevy::prelude::*;

/// Tracks velocity per entity.
#[derive(Component, Debug, Default, Clone)]
pub struct Velocity(pub Vec2);

/// Tracks new velocity per entity, which can be used for double-buffering
/// velocity updates.
#[derive(Component, Debug, Default, Clone)]
pub struct NewVelocity(pub Vec2);
