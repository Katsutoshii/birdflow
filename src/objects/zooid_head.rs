use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::physics::{PhysicsBundle, PhysicsMaterialType};
use crate::prelude::*;

use super::Team;
use super::{
    zooid_worker::{ZooidWorker, ZooidWorkerBundler},
    Object, ZooidAssets,
};

pub struct ZooidHeadPlugin;
impl Plugin for ZooidHeadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                ZooidHead::spawn.in_set(SystemStage::Spawn),
                ZooidHead::spawn_zooids.in_set(SystemStage::Spawn),
                ZooidHead::despawn_zooids.in_set(SystemStage::Despawn),
                ZooidHeadBackground::update.in_set(SystemStage::Compute),
            ),
        );
    }
}

#[derive(Component, Default)]
pub struct ZooidHeadBackground;
impl ZooidHeadBackground {
    pub fn bundle(self, assets: &ZooidAssets, team: Team) -> impl Bundle {
        (
            self,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(1.5))
                    .with_translation(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: zindex::ZOOID_HEAD_BACKGROUND,
                    }),
                material: assets.get_team_material(team).background,
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
            transform.translation = -0.05 * parent_velocity.0.extend(0.);
        }
    }
}

/// State for a head.
#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct ZooidHead;
impl ZooidHead {
    pub fn spawn(
        mut commands: Commands,
        assets: Res<ZooidAssets>,
        configs: Res<Configs>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        if !keyboard_input.just_pressed(KeyCode::Return) {
            return;
        }
        let team = configs.player_team;
        commands
            .spawn(ZooidHead.bundle(&assets, team))
            .with_children(|parent| {
                parent.spawn(ZooidHeadBackground.bundle(&assets, team));
            });
    }

    pub fn bundle(self, assets: &ZooidAssets, team: Team) -> impl Bundle {
        (
            self,
            Object::Head,
            team,
            GridEntity::default(),
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(20.0).extend(1.))
                    .with_translation(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: zindex::ZOOID_HEAD,
                    }),
                material: assets.get_team_material(team).primary,
                ..default()
            },
            PhysicsBundle {
                material: PhysicsMaterialType::SlowZooid,
                ..default()
            },
            Objective::default(),
            Selected::default(),
            Name::new("ZooidHead"),
        )
    }

    /// System to spawn zooids on Z key.
    pub fn spawn_zooids(
        mut commands: Commands,
        query: Query<(&Self, Entity, &Transform, &Velocity, &Team)>,
        configs: Res<Configs>,
        assets: Res<ZooidAssets>,
        keyboard: Res<Input<KeyCode>>,
    ) {
        if !keyboard.just_pressed(KeyCode::Z) {
            return;
        }

        let config = configs.get(&Object::Worker(ZooidWorker::default()));

        for (_head, _head_id, transform, velocity, team) in &query {
            for i in 1..2 {
                let zindex = zindex::ZOOIDS_MIN
                    + (i as f32) * 0.00001 * (zindex::ZOOIDS_MAX - zindex::ZOOIDS_MIN);
                let velocity: Vec2 = Vec2::Y * config.spawn_velocity + velocity.0;
                ZooidWorkerBundler {
                    worker: ZooidWorker { theta: 0.0 },
                    team: *team,
                    mesh: assets.mesh.clone(),
                    team_materials: assets.get_team_material(*team),
                    translation: transform.translation.xy().extend(0.0)
                        + velocity.extend(0.)
                        + Vec3::Z * zindex,
                    objective: Objective::default(),
                    velocity,
                }
                .spawn(&mut commands);
            }
        }
    }

    /// System to despawn all zooids.
    pub fn despawn_zooids(
        objects: Query<(Entity, &GridEntity, &Object)>,
        mut commands: Commands,
        mut grid: ResMut<Grid2<EntitySet>>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        if !keyboard_input.just_pressed(KeyCode::D) {
            return;
        }
        for (entity, grid_entity, object) in &objects {
            grid.remove(entity, grid_entity);
            if let Object::Worker(_) = object {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
