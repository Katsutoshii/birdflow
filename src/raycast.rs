use bevy::prelude::*;
use bevy_mod_raycast::prelude::*;

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct RaycastPlugin;
impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, DeferredRaycastingPlugin::<()>::default()))
            .add_systems(Update, print_intersections::<()>);
    }
}

/// Used to debug [`RaycastMesh`] intersections.
pub fn print_intersections<T: TypePath + Send + Sync>(query: Query<&RaycastMesh<T>>) {
    for (_, intersection) in query.iter().flat_map(|mesh| mesh.intersections.iter()) {
        info!(
            "Distance {:?}, Position {:?}",
            intersection.distance(),
            intersection.position()
        );
    }
}
