use self::effects::{EffectCommands, EffectSize, FireworkSpec};

use super::{
    neighbors::{AlliedNeighbors, EnemyNeighbors},
    DamageEvent, InteractionConfig, ObjectSpec,
};
use crate::prelude::*;
use bevy::prelude::*;

/// Plugin for running zooids simulation.
pub struct ObjectPlugin;
impl Plugin for ObjectPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Object>().add_systems(
            FixedUpdate,
            (
                Object::update_acceleration.in_set(SystemStage::Compute),
                Object::update_attack.in_set(SystemStage::Compute),
                Object::death.in_set(SystemStage::Despawn),
                ObjectBackground::update.in_set(SystemStage::Compute),
            ),
        );
    }
}

/// Entities that can interact with each other.
#[derive(Component, Reflect, Default, Copy, Clone, PartialEq, Eq, Hash, Debug, clap::ValueEnum)]
#[reflect(Component)]
pub enum Object {
    #[default]
    Worker,
    Head,
    Plankton,
    Food,
}
impl Object {
    pub fn update_acceleration(
        mut query: Query<(&Self, &Velocity, &mut Acceleration, &AlliedNeighbors)>,
        others: Query<(&Self, &Velocity)>,
        configs: Res<Configs>,
    ) {
        query
            .par_iter_mut()
            .for_each(|(object, velocity, mut final_acceleration, neighbors)| {
                let mut acceleration = Acceleration::ZERO;
                let config = &configs.objects[object];
                for neighbor in neighbors.iter() {
                    let (other_object, other_velocity) = others.get(neighbor.entity).unwrap();
                    let interaction = &config.interactions[other_object];
                    let distance_squared = neighbor.delta.length_squared();
                    // Separation
                    acceleration += Self::separation_acceleration(
                        -neighbor.delta,
                        distance_squared,
                        *velocity,
                        interaction,
                    );
                    // Alignment
                    acceleration += Self::alignment_acceleration(
                        distance_squared,
                        config.neighbor_radius * config.neighbor_radius,
                        *velocity,
                        *other_velocity,
                        interaction,
                    );
                }
                if !neighbors.is_empty() {
                    *final_acceleration += acceleration * (1.0 / (neighbors.len() as f32));
                }
            });
    }

    pub fn update_attack(
        mut query: Query<(Entity, &Self, &mut Objectives, &Health, &EnemyNeighbors)>,
        others: Query<(&Self, &Velocity)>,
        configs: Res<Configs>,
        mut damage_events: EventWriter<DamageEvent>,
    ) {
        for (entity, object, mut objectives, health, neighbors) in &mut query {
            let config = &configs.objects[object];

            let mut closest_enemy_distance_squared = f32::INFINITY;
            let mut closest_enemy_entity: Option<Entity> = None;

            // Food doesn't attack or get attacked.
            if *object == Object::Food {
                continue;
            }

            for neighbor in neighbors.iter() {
                let (other_object, other_velocity) = others.get(neighbor.entity).unwrap();

                // Food can't be targeted.
                if *other_object == Object::Food {
                    continue;
                }

                let distance_squared = neighbor.delta.length_squared();
                // Try attacking, only workers can attack.
                if *object == Self::Worker && distance_squared < closest_enemy_distance_squared {
                    closest_enemy_distance_squared = distance_squared;
                    closest_enemy_entity = Some(neighbor.entity);
                }

                // If we got hit.
                if config.is_hit(distance_squared, other_velocity.length_squared())
                    && health.damageable()
                {
                    damage_events.send(DamageEvent {
                        damager: neighbor.entity,
                        damaged: entity,
                        amount: 1,
                        velocity: *other_velocity,
                    });
                }
            }
            if let Some(entity) = closest_enemy_entity {
                objectives.start_attacking(entity)
            }
        }
    }

    pub fn death(
        mut objects: Query<(Entity, &Self, &GridEntity, &Health, &Transform, &Team)>,
        mut commands: Commands,
        mut object_commands: ObjectCommands,
        mut effect_commands: EffectCommands,
        mut grid: ResMut<Grid2<EntitySet>>,
    ) {
        for (entity, object, grid_entity, health, transform, team) in &mut objects {
            if health.health <= 0 {
                grid.remove(entity, grid_entity);
                commands.entity(entity).despawn_recursive();
                effect_commands.make_fireworks(FireworkSpec {
                    size: EffectSize::Medium,
                    transform: *transform,
                    team: *team,
                });
                if object == &Object::Plankton {
                    object_commands.spawn(ObjectSpec {
                        object: Object::Food,
                        position: transform.translation.xy(),
                        ..default()
                    })
                }
            }
        }
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

    /// Alignment acceleration.
    /// Compute the difference between this object's velocity and the other object's velocity.
    fn alignment_acceleration(
        distance_squared: f32,
        radius_squared: f32,
        velocity: Velocity,
        other_velocity: Velocity,
        config: &InteractionConfig,
    ) -> Acceleration {
        let magnitude = (radius_squared - distance_squared) / radius_squared;
        Acceleration((other_velocity.0 - velocity.0) * config.alignment_factor * magnitude)
    }
}

#[derive(Component, Default)]
pub struct ObjectBackground;
impl ObjectBackground {
    pub fn update(
        mut query: Query<(&mut Transform, &Parent), With<Self>>,
        parent_velocities: Query<&Velocity, With<Children>>,
    ) {
        for (mut transform, parent) in &mut query {
            let parent_velocity = parent_velocities
                .get(parent.get())
                .expect("Invalid parent.");
            transform.translation = -0.1 * parent_velocity.extend(0.);
        }
    }
}
