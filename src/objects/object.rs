use crate::prelude::*;
use bevy::prelude::*;

use super::{zooid_worker::ZooidWorker, InteractionConfig};

/// Plugin for running zooids simulation.
pub struct ObjectPlugin;
impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Object>()
            .add_systems(FixedUpdate, (Object::update.in_set(SystemStage::Compute),));
    }
}

/// Entities that can interact with each other.
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
impl Object {
    /// Update objects acceleration and objectives.
    pub fn update(
        mut objects: Query<(
            Entity,
            &Self,
            &Velocity,
            &mut Acceleration,
            &mut Objective,
            &Transform,
            &Team,
        )>,
        other_objects: Query<(&Self, &Velocity, &Transform, &Team)>,
        obstacles: Res<Grid2<Obstacle>>,
        grid: Res<Grid2<EntitySet>>,
        configs: Res<Configs>,
    ) {
        objects.par_iter_mut().for_each(
            |(entity, zooid, &velocity, mut acceleration, mut objective, transform, team)| {
                let config = configs.get(zooid);
                let (neighbor_acceleration, new_objective) = zooid.process_neighbors(
                    entity,
                    team,
                    velocity,
                    transform,
                    &other_objects,
                    &grid,
                    &obstacles,
                    config,
                    &objective,
                );
                *acceleration += neighbor_acceleration;
                if let Some(new_objective) = new_objective {
                    *objective = new_objective;
                }
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    /// Compute acceleration for this timestemp.
    pub fn process_neighbors(
        &self,
        entity: Entity,
        team: &Team,
        velocity: Velocity,
        transform: &Transform,
        entities: &Query<(&Object, &Velocity, &Transform, &Team)>,
        grid: &Grid2<EntitySet>,
        obstacles: &Grid2<Obstacle>,
        config: &Config,
        objective: &Objective,
    ) -> (Acceleration, Option<Objective>) {
        let mut acceleration = Acceleration::ZERO;
        // Forces from other entities
        let position = transform.translation.truncate();
        let other_entities = grid.get_entities_in_radius(position, config);

        let mut closest_enemy_entity: Option<Entity> = None;
        let mut closest_enemy_distance_squared = f32::INFINITY;

        for other_entity in &other_entities {
            if entity == *other_entity {
                continue;
            }

            let (other, &other_velocity, other_transform, other_team) =
                entities.get(*other_entity).expect("Invalid grid entity.");

            let other_position = other_transform.translation.truncate();
            let delta = other_position - position;
            acceleration += self.other_acceleration(
                transform,
                velocity,
                other,
                other_transform,
                other_velocity,
                config,
                other_entities.len(),
            ) * (1.0 / (other_entities.len() as f32));

            if other_team != team {
                let distance_squared = delta.length_squared();
                if distance_squared < closest_enemy_distance_squared {
                    closest_enemy_distance_squared = distance_squared;
                    closest_enemy_entity = Some(*other_entity);
                }
            }
        }
        acceleration += obstacles.obstacles_acceleration(position, velocity, acceleration) * 3.;

        (acceleration, objective.next(closest_enemy_entity))
    }

    #[allow(clippy::too_many_arguments)]
    pub fn other_acceleration(
        &self,
        transform: &Transform,
        velocity: Velocity,
        other: &Self,
        other_transform: &Transform,
        other_velocity: Velocity,
        config: &Config,
        num_others: usize,
    ) -> Acceleration {
        let mut acceleration = Acceleration::ZERO;
        let interaction = config.get_interaction(other);

        let position_delta =
            transform.translation.truncate() - other_transform.translation.truncate(); // Towards self, away from other.
        let distance_squared = position_delta.length_squared();
        if distance_squared > config.neighbor_radius * config.neighbor_radius {
            return acceleration;
        }

        // Separation
        acceleration +=
            Self::separation_acceleration(position_delta, distance_squared, velocity, interaction);

        // Alignment
        acceleration += Self::alignment_acceleration(
            distance_squared,
            velocity,
            other_velocity,
            num_others,
            interaction,
        );
        acceleration
    }

    /// Compute acceleration from separation.
    /// The direction is towards self away from each nearby bird.
    /// The magnitude is computed by
    /// $ magnitude = sep * (-x^2 / r^2 + 1)$
    fn separation_acceleration(
        position_delta: Vec2,
        distance_squared: f32,
        velocity: Velocity,
        interaction: &InteractionConfig,
    ) -> Acceleration {
        let radius = interaction.separation_radius;
        let radius_squared = radius * radius;

        let slow_force = interaction.slow_factor
            * if distance_squared < radius_squared {
                Vec2::ZERO
            } else {
                -1.0 * velocity.0
            };

        let magnitude =
            interaction.separation_acceleration * (-distance_squared / (radius_squared) + 1.);
        Acceleration(
            position_delta.normalize_or_zero()
                * magnitude.clamp(
                    -interaction.cohesion_acceleration,
                    interaction.separation_acceleration,
                )
                + slow_force,
        )
    }

    /// ALignment acceleration.
    /// For now we just nudge the birds in the direction of all the other birds.
    /// We normalize by number of other birds to prevent a large flock
    /// from being unable to turn.
    fn alignment_acceleration(
        distance_squared: f32,
        velocity: Velocity,
        other_velocity: Velocity,
        other_count: usize,
        config: &InteractionConfig,
    ) -> Acceleration {
        Acceleration(
            (other_velocity.0 - velocity.0) * config.alignment_factor
                / (distance_squared.max(0.1) * other_count as f32),
        )
    }
}
