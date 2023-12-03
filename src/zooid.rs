use std::f32::consts::PI;

use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle},
};

use crate::{
    grid::EntityGrid,
    physics::{NewVelocity, Velocity},
    waypoint::{Waypoint, WaypointConfig, WaypointFollower},
    zindex, SystemStage,
};

/// Plugin for running birds.
pub struct ZooidPlugin;
impl Plugin for ZooidPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Zooid>()
            .register_type::<ZooidConfigs>()
            .register_type::<ZooidConfig>()
            .register_type::<ZooidInteractionConfig>()
            .init_resource::<ZooidAssets>()
            .configure_sets(FixedUpdate, SystemStage::get_config())
            .add_systems(Startup, ZooidHead::spawn)
            .add_systems(
                FixedUpdate,
                (
                    Zooid::update_velocity.in_set(SystemStage::Compute),
                    Zooid::apply_velocity.in_set(SystemStage::Apply),
                    ZooidHead::spawn_zooids.in_set(SystemStage::Spawn),
                    ZooidHead::despawn_zooids.in_set(SystemStage::Despawn),
                ),
            );
    }
}

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

#[derive(Component, Reflect)]
#[reflect(Component)]
pub enum Zooid {
    Worker(ZooidWorker),
    Head,
    Food,
}
impl Default for Zooid {
    fn default() -> Self {
        Self::Worker(ZooidWorker::default())
    }
}

/// State for an individual zooid.
impl Zooid {
    pub fn other_acceleration(
        &self,
        transform: &Transform,
        velocity: &Velocity,
        other: &Self,
        other_transform: &Transform,
        other_velocity: &Velocity,
        config: &ZooidConfig,
        num_others: usize,
    ) -> Vec2 {
        let mut acceleration = Vec2::ZERO;
        let interaction = config.get_interaction(other);

        // Separation
        let position_delta =
            transform.translation.truncate() - other_transform.translation.truncate(); // Towards self, away from other.
        acceleration += Self::separation_acceleration(position_delta, velocity.0, &interaction);

        // Alignment
        acceleration += Self::alignment_acceleration(
            position_delta,
            velocity.0,
            other_velocity.0,
            num_others,
            &interaction,
        );
        acceleration
    }

    pub fn acceleration(
        &self,
        entity: Entity,
        velocity: &Velocity,
        transform: &Transform,
        follower: &WaypointFollower,
        entities: &Query<(&Zooid, &Velocity, &Transform), Without<Waypoint>>,
        waypoints: &Query<(&Waypoint, &Transform), With<Waypoint>>,
        grid: &EntityGrid,
        config: &ZooidConfig,
    ) -> Vec2 {
        let mut acceleration = Vec2::ZERO;

        // Forces from waypoint
        if let Zooid::Head = self {
            let mut waypoint_acceleration =
                follower.acceleration(&waypoints, transform, velocity.0, &config.waypoint);
            if let Zooid::Worker(ZooidWorker { theta }) = self {
                waypoint_acceleration += Mat2::from_angle(*theta) * waypoint_acceleration;
            }
            acceleration += waypoint_acceleration;
        }

        // Forces from other entities
        let others = grid.get_in_radius(transform.translation.truncate(), config.neighbor_radius);
        for other_entity in &others {
            if entity == *other_entity {
                continue;
            }

            let (other, other_velocity, other_transform) =
                entities.get(*other_entity).expect("Invalid grid entity.");
            acceleration += self.other_acceleration(
                transform,
                velocity,
                other,
                other_transform,
                other_velocity,
                config,
                others.len(),
            );
        }
        acceleration
    }

    pub fn update_velocity(
        mut zooids: Query<(
            Entity,
            &Zooid,
            &Velocity,
            &mut NewVelocity,
            &Transform,
            &WaypointFollower,
        )>,
        other_zooids: Query<(&Zooid, &Velocity, &Transform), Without<Waypoint>>,
        waypoints: Query<(&Waypoint, &Transform), With<Waypoint>>,
        grid: Res<EntityGrid>,
        configs: Res<ZooidConfigs>,
    ) {
        zooids.par_iter_mut().for_each(
            |(entity, zooid, velocity, mut new_velocity, transform, follower)| {
                let config = configs.get(zooid);
                let acceleration = zooid.acceleration(
                    entity,
                    velocity,
                    transform,
                    follower,
                    &other_zooids,
                    &waypoints,
                    &grid,
                    &config,
                );
                // Update new velocity.
                new_velocity.0 += acceleration;
                new_velocity.0 = new_velocity.0.clamp_length_max(config.max_velocity);
                new_velocity.0 = (1. - config.velocity_smoothing) * new_velocity.0
                    + config.velocity_smoothing * velocity.0;
            },
        )
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

    /// Compute acceleration from separation.
    /// The direction is towards self away from each nearby bird.
    /// The magnitude is computed by
    /// $ magnitude = sep * (-x^2 / r^2 + 1)$
    fn separation_acceleration(
        position_delta: Vec2,
        velocity: Vec2,
        interaction: &ZooidInteractionConfig,
    ) -> Vec2 {
        let radius = interaction.separation_radius;
        let dist_squared = position_delta.length_squared();
        let radius_squared = radius * radius;

        let slow_force = interaction.slow_factor
            * if dist_squared < radius_squared {
                Vec2::ZERO
            } else {
                -1.0 * velocity
            };

        let magnitude = interaction.separation_acceleration
            * (-position_delta.length_squared() / (radius * radius) + 1.);
        position_delta.normalize()
            * magnitude.clamp(
                -interaction.cohesion_acceleration,
                interaction.separation_acceleration,
            )
            + slow_force
    }

    /// ALignment acceleration.
    /// For now we just nudge the birds in the direction of all the other birds.
    /// We normalize by number of other birds to prevent a large flock
    /// from being unable to turn.
    fn alignment_acceleration(
        position_delta: Vec2,
        velocity: Vec2,
        other_velocity: Vec2,
        other_count: usize,
        config: &ZooidInteractionConfig,
    ) -> Vec2 {
        (other_velocity - velocity) * config.alignment_factor
            / (position_delta.length_squared() * other_count as f32)
    }
}

/// State for an individual bird.
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
            Zooid::Head,
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
        configs: Res<ZooidConfigs>,
        assets: Res<ZooidAssets>,
        keyboard: Res<Input<KeyCode>>,
    ) {
        if !keyboard.just_pressed(KeyCode::Z) {
            return;
        }

        let config = configs.get(&Zooid::Worker(ZooidWorker::default()));

        for (_head, _head_id, transform, _velocity, _follower) in &query {
            for i in 1..2 {
                let zindex = zindex::ZOOIDS_MIN
                    + (i as f32) * 0.00001 * (zindex::ZOOIDS_MAX - zindex::ZOOIDS_MIN);
                commands.spawn(
                    ZooidBundler {
                        zooid: Zooid::Worker(ZooidWorker {
                            theta: PI * configs.theta_factor * (i as f32),
                        }),
                        mesh: assets.mesh.clone(),
                        material: assets.green_material.clone(),
                        translation: transform.translation.xy().extend(0.0)
                            + (Vec3::Y) * -configs.translation_factor * (i as f32)
                            + Vec3::Z * zindex,
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
        zooids: Query<Entity, With<Zooid>>,
        mut commands: Commands,
        mut grid: ResMut<EntityGrid>,
        keyboard_input: Res<Input<KeyCode>>,
    ) {
        if !keyboard_input.just_pressed(KeyCode::D) {
            return;
        }
        for entity in &zooids {
            grid.remove(entity);
            commands.entity(entity).despawn();
        }
    }
}

/// Creates bundle for the Bird with its associated material mesh.
#[derive(Default)]
pub struct ZooidBundler<M: Material2d> {
    pub zooid: Zooid,
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

/// Handles to common bird assets.
#[derive(Resource)]
pub struct ZooidAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<ColorMaterial>,
    pub transparent_blue_material: Handle<ColorMaterial>,
    pub green_material: Handle<ColorMaterial>,
    pub tomato_material: Handle<ColorMaterial>,
}
impl FromWorld for ZooidAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(shape::Circle::default()))
        };
        let (green_material, tomato_material, blue_material, transparent_blue_material) = {
            let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
            (
                materials.add(ColorMaterial::from(Color::LIME_GREEN)),
                materials.add(ColorMaterial::from(Color::TOMATO)),
                materials.add(ColorMaterial::from(Color::ALICE_BLUE)),
                materials.add(ColorMaterial::from(Color::ALICE_BLUE.with_a(0.5))),
            )
        };
        Self {
            mesh,
            green_material,
            tomato_material,
            blue_material,
            transparent_blue_material,
        }
    }
}

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ZooidInteractionConfig {
    pub separation_radius: f32,
    pub separation_acceleration: f32,
    pub cohesion_acceleration: f32,
    pub alignment_factor: f32,
    pub slow_factor: f32,
}
impl Default for ZooidInteractionConfig {
    fn default() -> Self {
        Self {
            separation_radius: 1.0,
            separation_acceleration: 0.0,
            cohesion_acceleration: 0.0,
            alignment_factor: 0.0,
            slow_factor: 0.0,
        }
    }
}

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ZooidConfigs {
    pub theta_factor: f32,
    pub translation_factor: f32,
    // Configs for each Zooid type.
    pub worker: ZooidConfig,
    pub head: ZooidConfig,
    pub food: ZooidConfig,
}
impl Default for ZooidConfigs {
    fn default() -> Self {
        Self {
            theta_factor: 0.001,
            translation_factor: 10.0,
            worker: ZooidConfig::default(),
            head: ZooidConfig::default(),
            food: ZooidConfig::default(),
        }
    }
}
impl ZooidConfigs {
    fn get(&self, zooid: &Zooid) -> &ZooidConfig {
        match zooid {
            Zooid::Worker(_) => &self.worker,
            Zooid::Head => &self.head,
            Zooid::Food => &self.food,
        }
    }
}

/// Singleton that spawns birds with specified stats.
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ZooidConfig {
    pub max_velocity: f32,
    pub neighbor_radius: f32,
    pub alignment_factor: f32,
    pub velocity_smoothing: f32,
    pub spawn_velocity: f32,
    pub waypoint: WaypointConfig,

    // Interactions
    pub worker: ZooidInteractionConfig,
    pub head: ZooidInteractionConfig,
    pub food: ZooidInteractionConfig,
}
impl Default for ZooidConfig {
    fn default() -> Self {
        Self {
            max_velocity: 10.0,
            neighbor_radius: 10.0,
            alignment_factor: 0.1,
            velocity_smoothing: 0.5,
            spawn_velocity: 2.0,
            waypoint: WaypointConfig::default(),
            worker: ZooidInteractionConfig::default(),
            head: ZooidInteractionConfig::default(),
            food: ZooidInteractionConfig::default(),
        }
    }
}
impl ZooidConfig {
    fn get_interaction(&self, zooid: &Zooid) -> &ZooidInteractionConfig {
        match zooid {
            Zooid::Worker(_) => &self.worker,
            Zooid::Head => &self.head,
            Zooid::Food => &self.food,
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_update() {}
}
