use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle},
};

use crate::{
    physics::{NewVelocity, Velocity},
    waypoint::WaypointFollower,
};

use super::Object;

/// State for an individual zooid.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ZooidWorker {
    pub theta: f32,
}
impl Default for ZooidWorker {
    fn default() -> Self {
        Self { theta: 0.0 }
    }
}

/// Creates bundle for the Bird with its associated material mesh.
#[derive(Default)]
pub struct ZooidBundler<M: Material2d> {
    pub zooid: Object,
    pub mesh: Handle<Mesh>,
    pub material: Handle<M>,
    pub translation: Vec3,
    pub follower: WaypointFollower,
    pub velocity: Vec2,
}
impl<M: Material2d> ZooidBundler<M> {
    pub fn bundle(self) -> impl Bundle {
        (
            self.zooid,
            Velocity(self.velocity),
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
            Name::new("Zooid"),
        )
    }
}
