use std::time::Duration;

use crate::prelude::*;
use bevy::prelude::*;
use rand::Rng;

pub struct ObjectivePlugin;
impl Plugin for ObjectivePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ObjectiveConfig>().add_systems(
            FixedUpdate,
            Objectives::update.in_set(SystemStage::PreCompute),
        );
    }
}
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ObjectiveConfig {
    pub max_acceleration: f32,
    pub repell_radius: f32,
    pub slow_factor: f32,
}
impl Default for ObjectiveConfig {
    fn default() -> Self {
        Self {
            max_acceleration: 0.0,
            repell_radius: 1.0,
            slow_factor: 0.0,
        }
    }
}
impl ObjectiveConfig {
    /// Apply a slowing force.
    pub fn slow_force(
        &self,
        velocity: Velocity,
        position: Vec2,
        target_position: Vec2,
    ) -> Acceleration {
        let position_delta = target_position - position;
        let dist_squared = position_delta.length_squared();
        let radius = self.repell_radius;
        let radius_squared = radius * radius;
        Acceleration(
            self.slow_factor
                * if dist_squared < radius_squared {
                    -1.0 * velocity.0
                } else {
                    Vec2::ZERO
                },
        )
    }
}

#[derive(Debug, Clone)]
// Entity will attack nearest enemy in surrounding grid
pub struct AttackEntity {
    pub entity: Entity,
    pub cooldown: Timer,
}

// A stack of objectives.
/// Represents the objective of the owning entity.
#[derive(Component, Default, Debug, Clone, DerefMut, Deref)]
pub struct Objectives(pub Vec<Objective>);
impl Objectives {
    /// Update acceleration from the current objective.
    pub fn update(
        mut query: Query<(&mut Self, &Object, &Transform, &Velocity, &mut Acceleration)>,
        transforms: Query<&Transform>,
        configs: Res<Configs>,
        navigation_grid: Res<EntityFlowGrid2>,
        time: Res<Time>,
    ) {
        for (mut objectives, object, transform, velocity, mut acceleration) in &mut query {
            let config = configs.get(object);
            objectives.remove_invalid(&transforms);
            if let Some(objective) = objectives.last_mut() {
                *acceleration += objective.acceleration(
                    &transforms,
                    transform,
                    *velocity,
                    &config.waypoint,
                    &navigation_grid,
                    &time,
                );
            }
        }
    }

    // When we are near a new  nearest enemy, if we are doing nothing or following an entity, then we should target the new enemy.
    pub fn update_new_enemy(&mut self, event: &CreateWaypointEvent) {
        if let Some(objective) = self.last() {
            if objective.can_attack() {
                self.push(Objective::attack(event));
            }
        } else {
            self.push(Objective::attack(event));
        }
    }

    pub fn remove_invalid(&mut self, transforms: &Query<&Transform>) {
        while let Some(objective) = self.last() {
            if !objective.is_valid(transforms) {
                self.pop();
            }
        }
    }
}

/// Represents the objective of the owning entity.
#[derive(Debug, Clone)]
pub enum Objective {
    /// Entity wants to follow the transform of another entity.
    FollowEntity(Entity),
    /// Attack Entity
    AttackEntity(AttackEntity),
}
impl Objective {
    // Returns acceleration for following an entity.
    pub fn accelerate_to_entity(
        entity: Entity,
        transform: &Transform,
        transforms: &Query<&Transform>,
        config: &ObjectiveConfig,
        velocity: Velocity,
        flow_grid: &EntityFlowGrid2,
    ) -> Acceleration {
        let target_transform = transforms.get(entity);
        let target_transform = match target_transform {
            Ok(transform) => transform,
            Err(_) => return Acceleration::ZERO,
        };
        if let Some(flow_grid) = flow_grid.get(&entity) {
            let target_cell = flow_grid.to_rowcol(target_transform.translation.xy());
            let target_cell_position = flow_grid.to_world_position(target_cell);
            flow_grid.flow_acceleration5(transform.translation.xy())
                + config.slow_force(velocity, transform.translation.xy(), target_cell_position)
        } else {
            warn!(
                "Missing entity. This is okay if it's only for one frame. Entity: {:?}",
                entity
            );
            Acceleration::ZERO
        }
    }

    // Returns true iff the objective contains valid entity references.
    pub fn is_valid(&self, transforms: &Query<&Transform>) -> bool {
        match self {
            Self::FollowEntity(entity)
            | Self::AttackEntity(AttackEntity {
                entity,
                cooldown: _,
            }) => transforms.get(*entity).is_ok(),
        }
    }

    /// Returns true iff the objective can follow up into an attack
    pub fn can_attack(&self) -> bool {
        matches!(self, Self::FollowEntity(_))
    }

    // Returns acceleration for this objective.
    pub fn acceleration(
        &mut self,
        transforms: &Query<&Transform>,
        transform: &Transform,
        velocity: Velocity,
        config: &ObjectiveConfig,
        navigation_grid: &EntityFlowGrid2,
        time: &Time,
    ) -> Acceleration {
        match self {
            Self::FollowEntity(entity) => Self::accelerate_to_entity(
                *entity,
                transform,
                transforms,
                config,
                velocity,
                navigation_grid,
            ),
            Self::AttackEntity(AttackEntity { entity, cooldown }) => {
                cooldown.tick(time.delta());
                if cooldown.finished() {
                    cooldown.set_duration(Self::attack_cooldown());
                    let target_transform = transforms.get(*entity);
                    let target_transform = match target_transform {
                        Ok(transform) => transform,
                        Err(_) => return Acceleration::ZERO,
                    };
                    let delta = target_transform.translation.xy() - transform.translation.xy();
                    Acceleration(delta.normalize() * 1000.0)
                } else {
                    Self::accelerate_to_entity(
                        *entity,
                        transform,
                        transforms,
                        config,
                        velocity,
                        navigation_grid,
                    )
                }
            }
        }
    }

    pub fn get_followed_entity(&self) -> Option<Entity> {
        match self {
            Self::AttackEntity(AttackEntity {
                entity,
                cooldown: _,
            }) => Some(*entity),
            Self::FollowEntity(entity) => Some(*entity),
        }
    }
    /// Gets a random attack cooldown.
    pub fn attack_cooldown() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..1200))
    }

    /// Gets a random attack delay.
    pub fn attack_delay() -> Duration {
        Duration::from_millis(rand::thread_rng().gen_range(0..300))
    }

    /// Given an objective, get the next one (if there should be a next one, else None).
    pub fn attack(event: &CreateWaypointEvent) -> Self {
        Self::AttackEntity(AttackEntity {
            entity: event.entity,
            cooldown: Timer::from_seconds(Self::attack_delay().as_secs_f32(), TimerMode::Repeating),
        })
    }
}
