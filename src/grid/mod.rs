use crate::prelude::*;
use bevy::prelude::*;

mod spec;
pub use spec::GridSpec;
mod fog;
pub use fog::FogPlugin;
mod visualizer;
pub use visualizer::{GridShaderMaterial, GridVisualizer};
mod entity;
pub use entity::{EntityGridEvent, EntitySet, GridEntity};
mod obstacles;
pub use obstacles::{Obstacle, ObstaclesPlugin};
mod grid2;
pub use grid2::{Grid2, RowCol, RowColDistance};

mod navigation;
pub use navigation::{EntityFlow, NavigationCostEvent};
mod navigation_visualizer;
pub use self::{
    grid2::Grid2Plugin, navigation::NavigationPlugin,
    navigation_visualizer::NavigationVisualizerPlugin, visualizer::GridVisualizerPlugin,
};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GridSpec>()
            .add_event::<EntityGridEvent>()
            .add_plugins(GridVisualizerPlugin)
            .add_plugins(ObstaclesPlugin)
            .add_plugins(NavigationPlugin)
            .add_plugins(NavigationVisualizerPlugin)
            .add_plugins(FogPlugin)
            .add_plugins(Grid2Plugin::<EntitySet>::default())
            .add_systems(
                FixedUpdate,
                GridEntity::update.in_set(SystemStage::PostApply),
            );
    }
}
#[cfg(test)]
mod tests {
    use crate::grid::Grid2;

    #[test]
    fn grid_radius() {
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (2, 2);
            let radius = 2;
            assert!(Grid2::<()>::in_radius(
                (row, col),
                (other_row, other_col),
                radius
            ));
        }
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (4, 4);
            let radius = 2;
            assert!(!Grid2::<()>::in_radius(
                (row, col),
                (other_row, other_col),
                radius
            ));
        }
    }
}
