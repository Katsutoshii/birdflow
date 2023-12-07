use bevy::prelude::*;

use crate::{
    grid::EntityGrid,
    physics::{NewVelocity, Velocity},
    waypoint::{Waypoint, WaypointFollower},
    SystemStage,
};

pub use self::config::{Config, Configs, InteractionConfig};
use self::{
    food::{Food, FoodBackground},
    zooid_head::{ZooidHead, ZooidHeadBackground},
    zooid_worker::{ZooidWorker, ZooidWorkerBackground},
};

/// Plugin for running zooids simulation.
pub struct ObjectsPlugin;
impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Object>()
            .register_type::<Configs>()
            .register_type::<Config>()
            .register_type::<InteractionConfig>()
            .init_resource::<ZooidAssets>()
            .configure_sets(FixedUpdate, SystemStage::get_config())
            .add_systems(Startup, (ZooidHead::spawn))
            .add_systems(
                FixedUpdate,
                (
                    Food::update.in_set(SystemStage::PreCompute),
                    FoodBackground::update.in_set(SystemStage::Compute),
                    ZooidWorkerBackground::update.in_set(SystemStage::Compute),
                    ZooidHeadBackground::update.in_set(SystemStage::Compute),
                    Object::update_velocity.in_set(SystemStage::Compute),
                    Object::apply_velocity.in_set(SystemStage::Apply),
                    ZooidHead::spawn_zooids.in_set(SystemStage::Spawn),
                    Food::spawn.in_set(SystemStage::Spawn),
                    ZooidHead::despawn_zooids.in_set(SystemStage::Despawn),
                ),
            );
    }
}

mod config;
mod food;
mod zooid_head;
mod zooid_worker;

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub enum Object {
    Worker(ZooidWorker),
    Head,
    Food,
}
impl Default for Object {
    fn default() -> Self {
        Self::Worker(ZooidWorker::default())
    }
}

/// State for an individual zooid.
impl Object {
    /// Update velocity.
    pub fn update_velocity(
        mut objects: Query<(
            Entity,
            &Self,
            &Velocity,
            &mut NewVelocity,
            &Transform,
            &WaypointFollower,
        )>,
        other_objects: Query<(&Self, &Velocity, &Transform), Without<Waypoint>>,
        waypoints: Query<(&Waypoint, &Transform), With<Waypoint>>,
        grid: Res<EntityGrid>,
        configs: Res<Configs>,
    ) {
        objects.par_iter_mut().for_each(
            |(entity, zooid, velocity, mut new_velocity, transform, follower)| {
                let config = configs.get(zooid);
                let acceleration = zooid.acceleration(
                    entity,
                    velocity,
                    transform,
                    follower,
                    &other_objects,
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

    /// Apply velocity changes.
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

    /// Compute acceleration for this timestemp.
    pub fn acceleration(
        &self,
        entity: Entity,
        velocity: &Velocity,
        transform: &Transform,
        follower: &WaypointFollower,
        entities: &Query<(&Object, &Velocity, &Transform), Without<Waypoint>>,
        waypoints: &Query<(&Waypoint, &Transform), With<Waypoint>>,
        grid: &EntityGrid,
        config: &Config,
    ) -> Vec2 {
        let mut acceleration = Vec2::ZERO;

        // Forces from waypoint
        if let Object::Head = self {
            let mut waypoint_acceleration =
                follower.acceleration(&waypoints, transform, velocity.0, &config.waypoint);
            if let Object::Worker(ZooidWorker { theta }) = self {
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

    pub fn other_acceleration(
        &self,
        transform: &Transform,
        velocity: &Velocity,
        other: &Self,
        other_transform: &Transform,
        other_velocity: &Velocity,
        config: &Config,
        num_others: usize,
    ) -> Vec2 {
        let mut acceleration = Vec2::ZERO;
        let interaction = config.get_interaction(other);

        // Separation
        let position_delta =
            transform.translation.truncate() - other_transform.translation.truncate(); // Towards self, away from other.
        if position_delta.length_squared() > config.neighbor_radius * config.neighbor_radius {
            return acceleration;
        }
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

    /// Compute acceleration from separation.
    /// The direction is towards self away from each nearby bird.
    /// The magnitude is computed by
    /// $ magnitude = sep * (-x^2 / r^2 + 1)$
    fn separation_acceleration(
        position_delta: Vec2,
        velocity: Vec2,
        interaction: &InteractionConfig,
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

        let magnitude =
            interaction.separation_acceleration * (-dist_squared / (radius * radius) + 1.);
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
        config: &InteractionConfig,
    ) -> Vec2 {
        (other_velocity - velocity) * config.alignment_factor
            / (position_delta.length_squared() * other_count as f32)
    }
}

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct ZooidAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<ColorMaterial>,
    pub transparent_blue_material: Handle<ColorMaterial>,
    pub green_material: Handle<ColorMaterial>,
    pub tranparent_green_material: Handle<ColorMaterial>,
    pub dark_green_material: Handle<ColorMaterial>,
    pub transparent_dark_green_material: Handle<ColorMaterial>,
    pub tomato_material: Handle<ColorMaterial>,
    pub white_material: Handle<ColorMaterial>,
    pub transparent_white_material: Handle<ColorMaterial>,
}
impl FromWorld for ZooidAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(shape::Circle::default()))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            green_material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
            tranparent_green_material: materials
                .add(ColorMaterial::from(Color::LIME_GREEN.with_a(0.2))),

            dark_green_material: materials.add(ColorMaterial::from(Color::SEA_GREEN)),
            transparent_dark_green_material: materials
                .add(ColorMaterial::from(Color::SEA_GREEN.with_a(0.2))),
            tomato_material: materials.add(ColorMaterial::from(Color::TOMATO)),
            blue_material: materials.add(ColorMaterial::from(Color::TURQUOISE)),
            transparent_blue_material: materials
                .add(ColorMaterial::from(Color::TURQUOISE.with_a(0.2))),
            white_material: materials.add(ColorMaterial::from(Color::ALICE_BLUE)),
            transparent_white_material: materials
                .add(ColorMaterial::from(Color::ALICE_BLUE.with_a(0.2))),
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_update() {}
}
