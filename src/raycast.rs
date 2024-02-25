use bevy::{ecs::query::QueryIter, prelude::*, sprite::Mesh2dHandle, utils::FloatOrd};

#[derive(Component, Default, PartialEq, Debug, Clone, Copy)]
pub enum RaycastTarget {
    #[default]
    None,
    WorldGrid,
    Minimap,
}

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct RaycastPlugin;
impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RaycastEvent>();
    }
}

#[derive(Event, Debug, Clone)]
pub struct RaycastEvent {
    pub entity: Entity,
    pub world_position: Vec2,
    pub position: Vec2,
    pub target: RaycastTarget,
}

/// Send a raycast and return an event with intersection data.
pub fn raycast(
    ray: Ray3d,
    meshes: QueryIter<(Entity, &RaycastTarget, &Mesh2dHandle, &GlobalTransform), ()>,
    mesh_assets: &Assets<Mesh>,
) -> Option<RaycastEvent> {
    let mut hits = Vec::default();
    for (entity, &target, mesh_handle, transform) in meshes {
        let mesh = mesh_assets.get(&mesh_handle.0).unwrap();
        let mesh_to_world = transform.compute_matrix();
        let world_to_mesh = mesh_to_world.inverse();
        if let Some(intersection) = bevy_mod_raycast::prelude::ray_intersection_over_mesh(
            mesh,
            &mesh_to_world,
            ray,
            bevy_mod_raycast::prelude::Backfaces::Include,
        ) {
            let distance = FloatOrd(intersection.distance());
            let event = RaycastEvent {
                entity,
                position: world_to_mesh.transform_point3(intersection.position()).xy(),
                world_position: intersection.position().xy(),
                target,
            };
            hits.push((distance, event))
        }
    }

    hits.sort_by_key(|&(distance, _)| distance);
    if let Some((_distance, event)) = hits.first() {
        return Some(event.clone());
    }
    None
}
