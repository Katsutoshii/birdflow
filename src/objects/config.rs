use bevy::prelude::*;
use bevy::utils::HashMap;

use crate::prelude::*;
use crate::{objects::objective::ObjectiveConfig, physics::PhysicsMaterialType};

#[derive(Resource, Clone, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct TestInteractionConfigs(pub HashMap<PhysicsMaterialType, PhysicsMaterial>);

#[derive(Resource, Clone, Default, Deref, DerefMut, Reflect, Debug)]
#[reflect(Resource)]
pub struct InteractionConfigs(pub HashMap<Object, InteractionConfig>);
/// Describes interactions between two objects
#[derive(Clone, Reflect, Debug)]
pub struct InteractionConfig {
    pub separation_radius: f32,
    pub separation_acceleration: f32,
    pub cohesion_acceleration: f32,
    pub alignment_factor: f32,
    pub slow_factor: f32,
}
impl Default for InteractionConfig {
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

#[derive(Resource, Clone, Default, Deref, DerefMut, Reflect, Debug)]
#[reflect(Resource)]
pub struct ObjectConfigs(pub HashMap<Object, ObjectConfig>);

#[derive(Clone, Reflect, Debug)]
/// Specifies stats per object type.
pub struct ObjectConfig {
    physics_material: PhysicsMaterialType,
    pub neighbor_radius: f32,
    pub obstacle_acceleration: f32,
    pub nav_flow_factor: f32,
    pub attack_velocity: f32,
    pub spawn_velocity: f32,
    pub objective: ObjectiveConfig,
    pub hit_radius: f32,
    pub death_speed: f32,
    // Interactions
    pub interactions: InteractionConfigs,
}
impl Default for ObjectConfig {
    fn default() -> Self {
        Self {
            physics_material: PhysicsMaterialType::Default,
            neighbor_radius: 10.0,
            obstacle_acceleration: 3.,
            nav_flow_factor: 1.,
            attack_velocity: 40.,
            spawn_velocity: 2.0,
            objective: ObjectiveConfig::default(),
            hit_radius: 10.0,
            death_speed: 9.0,
            interactions: InteractionConfigs({
                let mut interactions = HashMap::new();
                interactions.insert(Object::Worker, InteractionConfig::default());
                interactions.insert(Object::Head, InteractionConfig::default());
                interactions.insert(Object::Plankton, InteractionConfig::default());
                interactions
            }),
        }
    }
}
impl ObjectConfig {
    /// Returns true if it's a hit.
    pub fn is_hit(&self, distance_squared: f32, velocity_squared: f32) -> bool {
        // info!("{}", velocity_squared);
        (distance_squared < self.hit_radius * self.hit_radius)
            && (velocity_squared > self.death_speed * self.death_speed)
    }
}
