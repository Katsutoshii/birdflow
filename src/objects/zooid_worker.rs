use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    grid::GridEntity,
    objects::objective::Objective,
    physics::{NewVelocity, Velocity},
    selector::Selected,
    zindex, SystemStage,
};

use super::{Object, Team, TeamMaterials};

pub struct ZooidWorkerPlugin;
impl Plugin for ZooidWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (ZooidWorkerBackground::update.in_set(SystemStage::Compute),),
        );
    }
}

/// State for an individual zooid.
#[derive(Component, Reflect, Clone)]
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
#[derive(Default, Clone)]
pub struct ZooidWorkerBundler {
    pub worker: ZooidWorker,
    pub team: Team,
    pub mesh: Handle<Mesh>,
    pub team_materials: TeamMaterials,
    pub translation: Vec3,
    pub objective: Objective,
    pub velocity: Vec2,
}
impl ZooidWorkerBundler {
    pub fn spawn(self, commands: &mut Commands) {
        commands
            .spawn(self.clone().bundle())
            .with_children(|parent| {
                parent.spawn(
                    ZooidWorkerBackground
                        .bundle(self.mesh.clone(), self.team_materials.background.clone()),
                );
            });
    }

    pub fn bundle(self) -> impl Bundle {
        (
            Object::Worker(self.worker),
            self.team,
            GridEntity,
            Velocity(self.velocity),
            NewVelocity::default(),
            self.objective,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: self.mesh.into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(10.0))
                    .with_translation(self.translation),
                material: self.team_materials.primary,
                ..default()
            },
            Selected::default(),
            Name::new("Zooid"),
        )
    }
}

#[derive(Component, Default)]
pub struct ZooidWorkerBackground;
impl ZooidWorkerBackground {
    pub fn bundle(self, mesh: Handle<Mesh>, material: Handle<ColorMaterial>) -> impl Bundle {
        (
            self,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: mesh.into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(1.5))
                    .with_translation(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: zindex::ZOOID_HEAD_BACKGROUND,
                    }),
                material,
                ..default()
            },
        )
    }
    pub fn update(
        mut query: Query<(&mut Transform, &Parent), With<Self>>,
        parent_velocities: Query<&Velocity, With<Children>>,
    ) {
        for (mut transform, parent) in &mut query {
            let parent_velocity = parent_velocities
                .get(parent.get())
                .expect("Invalid parent.");
            transform.translation = -0.1 * parent_velocity.0.extend(0.);
        }
    }
}
