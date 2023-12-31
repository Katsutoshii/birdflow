use crate::prelude::*;
use bevy::prelude::*;

pub use self::{
    config::{Config, Configs, InteractionConfig},
    objective::Objective,
};
use self::{
    food::FoodPlugin,
    objective::ObjectivePlugin,
    waypoint::WaypointPlugin,
    zooid_head::ZooidHeadPlugin,
    zooid_worker::{ZooidWorker, ZooidWorkerPlugin},
};

/// Plugin for running zooids simulation.
pub struct ObjectsPlugin;
impl Plugin for ObjectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ObjectivePlugin,
            ZooidHeadPlugin,
            ZooidWorkerPlugin,
            FoodPlugin,
            WaypointPlugin,
        ))
        .register_type::<Vec2>()
        .register_type::<Object>()
        .register_type::<Configs>()
        .register_type::<Config>()
        .register_type::<Team>()
        .register_type::<InteractionConfig>()
        .init_resource::<ZooidAssets>()
        .configure_sets(FixedUpdate, SystemStage::get_config())
        .add_systems(
            FixedUpdate,
            (Object::update_velocity.in_set(SystemStage::Compute),),
        );
    }
}

mod config;
mod food;
mod objective;
mod waypoint;
mod zooid_head;
mod zooid_worker;

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
    /// Update objects velocity and objectives.
    pub fn update_velocity(
        mut objects: Query<(Entity, &Self, &Velocity, &mut Acceleration, &Transform)>,
        other_objects: Query<(&Self, &Velocity, &Transform)>,
        obstacles: Res<Grid2<Obstacle>>,
        grid: Res<Grid2<EntitySet>>,
        configs: Res<Configs>,
    ) {
        objects.par_iter_mut().for_each(
            |(entity, zooid, &velocity, mut acceleration, transform)| {
                let config = configs.get(zooid);
                *acceleration += zooid.acceleration(
                    entity,
                    velocity,
                    transform,
                    &other_objects,
                    &grid,
                    &obstacles,
                    config,
                );
            },
        )
    }

    #[allow(clippy::too_many_arguments)]
    /// Compute acceleration for this timestemp.
    pub fn acceleration(
        &self,
        entity: Entity,
        velocity: Velocity,
        transform: &Transform,
        entities: &Query<(&Object, &Velocity, &Transform)>,
        grid: &Grid2<EntitySet>,
        obstacles: &Grid2<Obstacle>,
        config: &Config,
    ) -> Acceleration {
        let mut acceleration = Acceleration(Vec2::ZERO);

        // Forces from other entities
        let position = transform.translation.truncate();
        let other_entities = grid.get_entities_in_radius(position, config);
        for other_entity in &other_entities {
            if entity == *other_entity {
                continue;
            }

            let (other, &other_velocity, other_transform) =
                entities.get(*other_entity).expect("Invalid grid entity.");
            acceleration += self.other_acceleration(
                transform,
                velocity,
                other,
                other_transform,
                other_velocity,
                config,
                other_entities.len(),
            ) * (1.0 / (other_entities.len() as f32 + 1.));
        }

        acceleration += obstacles.obstacles_acceleration(position, velocity, acceleration) * 3.;

        acceleration
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
        let mut acceleration = Acceleration(Vec2::ZERO);
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

/// Enum to specify the team of the given object.
#[derive(Component, Default, Debug, PartialEq, Eq, Reflect, Clone, Copy, Hash)]
#[reflect(Component)]
#[repr(u8)]
pub enum Team {
    #[default]
    None = 0,
    Blue = 1,
    Red = 2,
}
impl Team {
    /// Number of teams.
    pub const fn count() -> usize {
        3
    }
}

#[derive(Default, Clone)]
pub struct TeamMaterials {
    pub primary: Handle<ColorMaterial>,
    pub background: Handle<ColorMaterial>,
}
impl TeamMaterials {
    pub fn new(color: Color, assets: &mut Assets<ColorMaterial>) -> Self {
        Self {
            primary: assets.add(ColorMaterial::from(color)),
            background: assets.add(ColorMaterial::from(color.with_a(0.2))),
        }
    }
}

/// Handles to common zooid assets.
#[derive(Resource)]
pub struct ZooidAssets {
    pub mesh: Handle<Mesh>,
    team_materials: Vec<TeamMaterials>,
}
impl ZooidAssets {
    fn get_team_material(&self, team: Team) -> TeamMaterials {
        self.team_materials.get(team as usize).unwrap().clone()
    }
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
            team_materials: vec![
                // Team::None
                TeamMaterials::new(Color::SEA_GREEN, &mut materials),
                // Team::Blue
                TeamMaterials::new(Color::TEAL, &mut materials),
                // Team::Red
                TeamMaterials::new(Color::TOMATO, &mut materials),
            ],
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_update() {}
}
