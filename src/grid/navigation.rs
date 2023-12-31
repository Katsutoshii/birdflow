use std::{cmp::Ordering, collections::BinaryHeap};

use crate::{grid::Grid2, prelude::*};
use bevy::{prelude::*, utils::HashMap};

use super::{GridSpec, ObstaclesGrid, RowCol};

/// Plugin for flow-based navigation.
pub struct NavigationPlugin;
impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(NavigationFlowGrid::default())
            .add_event::<NavigationCostEvent>()
            .add_systems(
                FixedUpdate,
                (NavigationFlowGrid::resize_on_change.in_set(SystemStage::PreCompute),),
            );
    }
}

/// Communicates cost updates to the visualizer
#[derive(Event)]
pub struct NavigationCostEvent {
    pub entity: Entity,
    pub rowcol: RowCol,
    pub cost: f32,
}

/// State for running A* search to fill out flow cost grid.
/// See https://doc.rust-lang.org/std/collections/binary_heap/index.html#examples
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AStarState {
    rowcol: RowCol,
    cost: usize,
}
impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp(&self.cost)
            .then_with(|| self.rowcol.cmp(&other.rowcol))
    }
}
impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Flow grid applying forces for the shortest path to the objective.
#[derive(Resource, Debug, Clone, Default, DerefMut, Deref)]
pub struct NavigationFlowGrid(Grid2<HashMap<Entity, Acceleration>>);
impl NavigationFlowGrid {
    pub fn resize_on_change(mut grid: ResMut<Self>, grid_spec: Res<GridSpec>) {
        if !grid_spec.is_changed() {
            return;
        }
        grid.resize_with(grid_spec.clone());
    }

    /// Add a waypoint.
    /// Create flows from all points to the waypoint.
    pub fn add_waypoint(
        &mut self,
        entity: Entity,
        waypoint_rowcol: RowCol,
        traveler_rowcols: &[RowCol],
        obstacles: &ObstaclesGrid,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        let costs = Self::a_star(traveler_rowcols, waypoint_rowcol, obstacles);
        for (rowcol, cost) in costs {
            event_writer.send(NavigationCostEvent {
                entity,
                rowcol,
                cost,
            });
        }
    }

    /// Run A* search from destination to reach all sources.
    /// Alternate targeting
    pub fn a_star(
        _sources: &[RowCol],
        destination: RowCol,
        _obstacles: &ObstaclesGrid,
    ) -> HashMap<RowCol, f32> {
        // Initial setup.
        let mut costs: HashMap<RowCol, f32> = HashMap::new();
        let mut _heap: BinaryHeap<AStarState> = BinaryHeap::new();
        // TODO: actually run A* etc
        costs.insert(destination, 0.01);
        costs
    }
}
