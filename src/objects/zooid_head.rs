use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    grid::EntityGrid,
    physics::{NewVelocity, Velocity},
    selector::Selected,
    waypoint::{Waypoint, WaypointFollower},
    zindex,
};

use super::{
    zooid_worker::{ZooidBundler, ZooidWorker},
    Configs, Object, ZooidAssets,
};

#[derive(Component, Default)]
pub struct ZooidHeadBackground;
impl ZooidHeadBackground {
    pub fn bundle(self, assets: &ZooidAssets) -> impl Bundle {
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
                material: assets.transparent_blue_material.clone(),
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
        waypoint: Query<Entity, With<Waypoint>>,
    ) {
        let waypoint_id = waypoint.single();
        commands
            .spawn(ZooidHead::default().bundle(&assets, waypoint_id))
            .with_children(|parent| {
                parent.spawn(ZooidHeadBackground::default().bundle(&assets));
            });
    }

    pub fn bundle(self, assets: &ZooidAssets, waypoint_id: Entity) -> impl Bundle {
        (
            self,
            Object::Head,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(20.0))
                    .with_translation(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: zindex::ZOOID_HEAD,
                    }),
                material: assets.blue_material.clone(),
                ..default()
            },
            Velocity::default(),
            NewVelocity::default(),
            WaypointFollower::new(waypoint_id),
            Selected::default(),
            Name::new("ZooidHead"),
        )
    }

    /// System to spawn birds on left mouse button.
    pub fn spawn_zooids(
        mut commands: Commands,
        query: Query<(&Self, Entity, &Transform, &Velocity, &WaypointFollower)>,
        configs: Res<Configs>,
        assets: Res<ZooidAssets>,
        keyboard: Res<Input<KeyCode>>,
    ) {
        if !keyboard.just_pressed(KeyCode::Z) {
            return;
        }

        let config = configs.get(&Object::Worker(ZooidWorker::default()));

        for (_head, _head_id, transform, _velocity, _follower) in &query {
            for i in 1..2 {
                let zindex = zindex::ZOOIDS_MIN
                    + (i as f32) * 0.00001 * (zindex::ZOOIDS_MAX - zindex::ZOOIDS_MIN);
                ZooidBundler {
                    zooid: Object::Worker(ZooidWorker { theta: 0.0 }),
                    mesh: assets.mesh.clone(),
                    material: assets.green_material.clone(),
                    background_material: assets.tranparent_green_material.clone(),
                    translation: transform.translation.xy().extend(0.0) + Vec3::Z * zindex,
                    follower: WaypointFollower::default(),
                    velocity: config.spawn_velocity * Vec2::Y,
                }
                .spawn(&mut commands);
            }
        }
    }

    /// System to despawn all zooids.
    pub fn despawn_zooids(
        objects: Query<(Entity, &Object)>,
        mut commands: Commands,
        mut grid: ResMut<EntityGrid>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        if !keyboard_input.just_pressed(KeyCode::D) {
            return;
        }
        for (entity, object) in &objects {
            if let Object::Worker(_) = object {
                grid.remove(entity);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
