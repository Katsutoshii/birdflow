/// Sparse grid flow for path finding.
use std::{
    cmp::Ordering,
    collections::{BTreeSet, BinaryHeap},
};

use crate::prelude::*;
use bevy::{
    prelude::*,
    utils::{Entry, HashMap, HashSet},
};

use super::SparseGrid2;

/// Plugin for flow-based navigation.
pub struct NavigationPlugin;
impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<NavigationCostEvent>()
            .add_event::<CreateWaypointEvent>()
            .insert_resource(EntityFlowGrid2::default())
            .add_systems(
                FixedUpdate,
                (
                    EntityFlowGrid2::resize_on_change,
                    EntityFlowGrid2::create_waypoints.after(Waypoint::update),
                    EntityFlowGrid2::delete_waypoints.before(EntityFlowGrid2::create_waypoints),
                ),
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
#[derive(Copy, Clone, PartialEq)]
pub struct AStarState {
    rowcol: RowCol,
    cost: f32,
    heuristic: f32,
}
impl AStarState {
    // Priority scoring function f.
    fn f(&self) -> f32 {
        self.cost + self.heuristic
    }
}
impl Eq for AStarState {}
impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .f()
            .partial_cmp(&self.f())
            .expect("NaN cost found in A*.")
            .then_with(|| self.rowcol.cmp(&other.rowcol))
    }
}
impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Struct to allow running A* on demand while re-using old results.
pub struct AStarRunner {
    pub destination: RowCol,
    pub costs: HashMap<RowCol, f32>,
    heap: BinaryHeap<AStarState>,
}
impl AStarRunner {
    /// Initialize AStarRunner to a goal.
    pub fn new(destination: RowCol) -> Self {
        let mut heap = BinaryHeap::new();
        heap.push(AStarState {
            cost: 0.,
            rowcol: destination,
            heuristic: 0.,
        });
        Self {
            destination,
            costs: HashMap::new(),
            heap,
        }
    }

    /// Compute heuristic factor on naive 8-distance heuristic.
    /// We want to use the heuristic more at higher distances from the destination.
    pub fn heuristic_factor(&self, source: RowCol) -> f32 {
        let min_grid_dist = 1.;
        let max_grid_dist = 30.;
        let dist = self.destination.distance8(source);
        let max_heuristic = 0.9;
        let final_dist = dist.clamp(min_grid_dist, max_grid_dist);
        max_heuristic * final_dist / max_grid_dist
    }
    /// Runs A star from the given source to the destination.
    pub fn a_star_from_source(
        &mut self,
        source: RowCol,
        grid: &SparseFlowGrid2,
        obstacles: &Grid2<Obstacle>,
    ) {
        // We're at `start`, with a zero cost
        if grid.is_boundary(self.destination) {
            return;
        }

        let heuristic_factor = self.heuristic_factor(source);

        // Examine the frontier with lower cost nodes first (min-heap)
        while let Some(AStarState {
            cost,
            rowcol,
            heuristic: _,
        }) = self.heap.pop()
        {
            // Skip already finalized cells.
            if self.costs.contains_key(&rowcol) {
                continue;
            }

            // Costs inserted here are guaranteed to be the best costs seen so far.
            self.costs.insert(rowcol, cost);

            // If at the goal, we're done!
            if rowcol == source {
                return;
            }

            // For each node we can reach, see if we can find a way with
            // a lower cost going through this node
            for (neighbor_rowcol, neighbor_cost) in grid.neighbors8(rowcol) {
                // Skip out of bounds positions.
                if grid.is_boundary(neighbor_rowcol) {
                    continue;
                }
                if obstacles[neighbor_rowcol] != Obstacle::Empty {
                    continue;
                }

                self.heap.push(AStarState {
                    cost: cost + neighbor_cost,
                    rowcol: neighbor_rowcol,
                    heuristic: heuristic_factor * neighbor_rowcol.distance8(source),
                });
            }
        }
    }
}

/// Sparse storage for flow vectors.
#[derive(Default, DerefMut, Deref, Clone)]
pub struct SparseFlowGrid2(SparseGrid2<Acceleration>);
impl SparseFlowGrid2 {
    /// Compute the weighted acceleration for flow from a single cell.
    pub fn flow_acceleration(&self, position: Vec2, rowcol: RowCol) -> Acceleration {
        if let Some(&acceleration) = self.get(rowcol) {
            // Weight each neighboring acceleration by width - distance.
            let weight = {
                let cell_center = self.to_world_position(rowcol);
                (self.spec.width * self.spec.width - cell_center.distance_squared(position)).max(0.)
            };
            return acceleration * weight;
        }
        Acceleration::ZERO
    }

    pub fn flow_acceleration5(&self, position: Vec2) -> Acceleration {
        let mut total_acceleration = Acceleration::ZERO;

        let rowcol = self.to_rowcol(position);

        total_acceleration += self.flow_acceleration(position, rowcol);
        if self.is_boundary(rowcol) {
            return Acceleration::ZERO;
        }
        // Add accelerations from neighboring cells.
        for (neighbor_rowcol, _) in self.neighbors8(rowcol) {
            if self.is_boundary(neighbor_rowcol) {
                continue;
            }
            total_acceleration += self.flow_acceleration(position, neighbor_rowcol);
        }
        Acceleration(total_acceleration.normalize_or_zero())
    }

    /// Add a waypoint.
    /// Create flows from all points to the waypoint.
    pub fn add_waypoint(
        &mut self,
        event: &CreateWaypointEvent,
        obstacles: &Grid2<Obstacle>,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        let mut sources = Vec::with_capacity(event.sources.len());
        for &source in &event.sources {
            let rowcol = self.spec.to_rowcol(source);
            for neighbor_rowcol in self.get_in_radius_discrete(rowcol, 2) {
                sources.push(neighbor_rowcol);
            }
        }
        let destination = self.to_rowcol(event.destination);
        let costs = self.a_star(&sources, destination, obstacles);
        // Compute flow direction.
        for (&rowcol, &cost) in &costs {
            let mut min_neighbor_rowcol = rowcol;
            let mut min_neighbor_cost = cost;
            for (neighbor_rowcol, _) in self.neighbors8(rowcol) {
                if let Some(&neighbor_cost) = costs.get(&neighbor_rowcol) {
                    if neighbor_cost < min_neighbor_cost {
                        min_neighbor_rowcol = neighbor_rowcol;
                        min_neighbor_cost = neighbor_cost;
                    }
                }
            }
            self.cells.insert(
                rowcol,
                Acceleration(rowcol.signed_delta8(min_neighbor_rowcol)),
            );

            event_writer.send(NavigationCostEvent {
                entity: event.entity,
                rowcol,
                cost,
            });
        }
    }

    /// Run A* search from destination to reach all sources.
    /// Alternate targeting
    pub fn a_star(
        &self,
        sources: &[RowCol],
        destination: RowCol,
        obstacles: &Grid2<Obstacle>,
    ) -> HashMap<RowCol, f32> {
        let sources: BTreeSet<RowCol> = sources
            .iter()
            .filter(|&&rowcol| self.in_bounds(rowcol) && obstacles[rowcol] == Obstacle::Empty)
            .copied()
            .collect();
        let mut runner = AStarRunner::new(destination);
        for source in sources {
            if runner.costs.contains_key(&source) {
                continue;
            }
            runner.a_star_from_source(source, self, obstacles);
        }
        runner.costs
    }
}

#[derive(Default, Resource, DerefMut, Deref)]
pub struct EntityFlowGrid2(HashMap<Entity, SparseFlowGrid2>);

/// Stores a flow grid per targeted entity.
impl EntityFlowGrid2 {
    // Resize all grids when spec is updated.
    pub fn resize_on_change(spec: Res<GridSpec>, mut grid: ResMut<Self>) {
        if spec.is_changed() {
            for (_entity, flow_grid) in grid.iter_mut() {
                flow_grid.resize_with(spec.clone());
            }
        }
    }

    /// Compute acceleration using the weighted sum of the 4 neighboring cells and the current cell.
    pub fn flow_acceleration5(&mut self, position: Vec2, entity: Entity) -> Acceleration {
        if let Some(flow_grid) = self.get_mut(&entity) {
            flow_grid.flow_acceleration5(position)
        } else {
            Acceleration::ZERO
        }
    }

    /// Consumes CreateWaypointEvent events and populates the navigation grid.
    pub fn create_waypoints(
        mut grid: ResMut<Self>,
        mut event_reader: EventReader<CreateWaypointEvent>,
        mut event_writer: EventWriter<NavigationCostEvent>,
        spec: Res<GridSpec>,
        obstacles: Res<Grid2<Obstacle>>,
    ) {
        for event in event_reader.read() {
            let flow_grid = match grid.entry(event.entity) {
                Entry::Occupied(o) => o.into_mut(),
                Entry::Vacant(v) => v.insert(SparseFlowGrid2(SparseGrid2 {
                    spec: spec.clone(),
                    ..default()
                })),
            };
            flow_grid.add_waypoint(event, &obstacles, &mut event_writer);
        }
    }

    /// Consumes CreateWaypointEvent events and populates the navigation grid.
    pub fn delete_waypoints(
        all_objectives: Query<&Objectives, Without<Waypoint>>,
        mut grid: ResMut<Self>,
    ) {
        let mut followed_entities = HashSet::new();
        for objectives in all_objectives.iter() {
            if let Some(entity) = objectives.last().get_followed_entity() {
                followed_entities.insert(entity);
            }
        }
        let entities_to_remove: Vec<Entity> = grid
            .keys()
            .filter(|&entity| !followed_entities.contains(entity))
            .copied()
            .collect();
        for entity in entities_to_remove {
            grid.remove(&entity);
        }
    }
}

/// Event to request waypoint creation.
#[derive(Event, Clone, Debug)]
pub struct CreateWaypointEvent {
    pub entity: Entity,
    pub destination: Vec2,
    pub sources: Vec<Vec2>,
}
