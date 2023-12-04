use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{
    grid::EntityGrid,
    physics::{NewVelocity, Velocity},
    waypoint::{Waypoint, WaypointFollower},
    zindex,
};

use super::{
    zooid_worker::{ZooidBundler, ZooidWorker},
    Configs, Object, ZooidAssets,
};

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
        commands.spawn(ZooidHead::default().bundle(&assets, waypoint_id));
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
                material: assets.transparent_blue_material.clone(),
                ..default()
            },
            Velocity::default(),
            NewVelocity::default(),
            WaypointFollower::new(waypoint_id),
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
                commands.spawn(
                    ZooidBundler {
                        zooid: Object::Worker(ZooidWorker { theta: 0.0 }),
                        mesh: assets.mesh.clone(),
                        material: assets.green_material.clone(),
                        translation: transform.translation.xy().extend(0.0) + Vec3::Z * zindex,
                        follower: WaypointFollower::default(),
                        velocity: -config.spawn_velocity * Vec2::Y,
                    }
                    .bundle(),
                );
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
                commands.entity(entity).despawn();
            }
        }
    }
}
