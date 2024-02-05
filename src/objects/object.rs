use std::sync::Mutex;

use crate::prelude::*;
use bevy::prelude::*;

use super::{zooid_worker::ZooidWorker, InteractionConfig};

/// Plugin for running zooids simulation.
pub struct ObjectPlugin;
impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Object>().add_systems(
            FixedUpdate,
            (
                Object::update.in_set(SystemStage::Compute),
                Object::death.in_set(SystemStage::Despawn),
            ),
        );
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

#[derive(Component)]
pub struct Health {
    pub health: i32,
    pub hit_timer: Timer,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            health: 2,
            hit_timer: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
}

impl Default for Object {
    fn default() -> Self {
        Self::Worker(ZooidWorker::default())
    }
}
impl Object {
    /// Update objects acceleration and objectives.\
    #[allow(clippy::type_complexity)]
    pub fn update(
        mut objects: Query<(
            Entity,
            &Self,
            &Velocity,
            &mut Acceleration,
            &mut Objective,
            &mut Health,
            &Transform,
            &Team,
        )>,
        other_objects: Query<(&Self, &Velocity, &Transform, &Team)>,
        obstacles: Res<Grid2<Obstacle>>,
        grid: Res<Grid2<EntitySet>>,
        configs: Res<Configs>,
        mut waypoint_event_writer: EventWriter<CreateWaypointEvent>,
        time: Res<Time>,
    ) {
        let writer_mutex: Mutex<&mut EventWriter<CreateWaypointEvent>> =
            Mutex::new(&mut waypoint_event_writer);
        objects.par_iter_mut().for_each(
            |(
                entity,
                zooid,
                &velocity,
                mut acceleration,
                mut objective,
                mut health,
                transform,
                team,
            )| {
                let config = configs.get(zooid);
                let (neighbor_acceleration, new_objective, waypoint_event) = zooid
                    .process_neighbors(
                        entity,
                        team,
                        velocity,
                        transform,
                        &other_objects,
                        &grid,
                        &obstacles,
                        &mut health,
                        config,
                        &objective,
                        &time,
                    );
                *acceleration += neighbor_acceleration;
                if let Some(new_objective) = new_objective {
                    *objective = new_objective;
                }
                // if let Some(waypoint_event) = waypoint_event {
                //     writer_mutex.lock().unwrap().send(waypoint_event);
                // }
            },
        )
    }

    pub fn death(
        mut objects: Query<(Entity, &GridEntity, &Health)>,
        mut commands: Commands,
        mut grid: ResMut<Grid2<EntitySet>>,
    ) {
        for (entity, grid_entity, health) in &mut objects {
            if health.health <= 0 {
                grid.remove(entity, grid_entity);
                commands.entity(entity).despawn_recursive();
            }
        }
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
        health: &mut Health,
        config: &Config,
        objective: &Objective,
        time: &Time,
    ) -> (Acceleration, Option<Objective>, Option<CreateWaypointEvent>) {
        let mut acceleration = Acceleration::ZERO;
        // Forces from other entities
        let position = transform.translation.xy();
        let other_entities = grid.get_entities_in_radius(position, config);

        let mut closest_enemy_distance_squared = f32::INFINITY;
        let mut enemy_waypoint_event: Option<CreateWaypointEvent> = None;

        for other_entity in &other_entities {
            if entity == *other_entity {
                continue;
            }

            let (other, &other_velocity, other_transform, other_team) =
                entities.get(*other_entity).expect("Invalid grid entity.");

            let other_position = other_transform.translation.xy();
            let delta = other_position - position;
            acceleration += self.other_acceleration(
                transform,
                velocity,
                team,
                other,
                other_transform,
                other_velocity,
                other_team,
                config,
                other_entities.len(),
            ) * (1.0 / (other_entities.len() as f32));

            if other_team != team {
                let distance_squared = delta.length_squared();
                if distance_squared < closest_enemy_distance_squared {
                    closest_enemy_distance_squared = distance_squared;
                    enemy_waypoint_event = Some(CreateWaypointEvent {
                        entity: *other_entity,
                        destination: other_position,
                        sources: vec![position],
                    })
                }
                health.hit_timer.tick(time.delta());
                if distance_squared < config.hit_radius.powi(2)
                    && health.hit_timer.finished()
                    && velocity.length_squared() > config.death_speed.powi(2)
                {
                    health.health -= 1;
                    health.hit_timer = Timer::from_seconds(1.0, TimerMode::Once);
                }
            }
        }
        acceleration += obstacles.obstacles_acceleration(position, velocity, acceleration) * 3.;

        (
            acceleration,
            objective.next(&enemy_waypoint_event),
            enemy_waypoint_event,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn other_acceleration(
        &self,
        transform: &Transform,
        velocity: Velocity,
        team: &Team,
        other: &Self,
        other_transform: &Transform,
        other_velocity: Velocity,
        other_team: &Team,
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

        if team == other_team {
            // Separation
            acceleration += Self::separation_acceleration(
                position_delta,
                distance_squared,
                velocity,
                interaction,
            );
            // Alignment
            acceleration += Self::alignment_acceleration(
                distance_squared,
                velocity,
                other_velocity,
                num_others,
                interaction,
            );
        }

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
