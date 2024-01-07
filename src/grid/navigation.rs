/// Sparse grid flow for path finding.
use std::{
    cmp::Ordering,
    collections::{BTreeSet, BinaryHeap},
};

use crate::{grid::Grid2, prelude::*, Obstacle, RowCol, RowColDistance};
use bevy::{prelude::*, utils::HashMap};

/// Plugin for flow-based navigation.
pub struct NavigationPlugin;
impl Plugin for NavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Grid2Plugin::<EntityFlow>::default())
            .add_event::<NavigationCostEvent>();
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

/// Flow vector per entity.
#[derive(Default, DerefMut, Deref, Clone)]
pub struct EntityFlow(pub HashMap<Entity, Acceleration>);

impl Grid2<EntityFlow> {
    /// Compute the weighted acceleration for flow from a single cell.
    pub fn flow_acceleration(
        &self,
        position: Vec2,
        rowcol: RowCol,
        entity: Entity,
    ) -> Acceleration {
        if let Some(&acceleration) = self[rowcol].get(&entity) {
            // Weight each neighboring acceleration by width - distance.
            let weight = {
                let cell_center = self.to_world_position(rowcol);
                (self.spec.width * self.spec.width - cell_center.distance_squared(position)).max(0.)
            };
            return acceleration * weight;
        }
        Acceleration(Vec2::ZERO)
    }

    /// Compute acceleration using the weighted sum of the 4 neighboring cells and the current cell.
    pub fn flow_acceleration5(&self, position: Vec2, entity: Entity) -> Acceleration {
        let mut total_acceleration = Acceleration(Vec2::ZERO);
        let rowcol = self.to_rowcol(position);

        total_acceleration += self.flow_acceleration(position, rowcol, entity);

        // Add accelerations from neighboring cells.
        for (neighbor_rowcol, _) in self.neighbors8(rowcol) {
            if self.is_boundary(neighbor_rowcol) {
                continue;
            }
            total_acceleration += self.flow_acceleration(position, neighbor_rowcol, entity);
        }
        Acceleration(total_acceleration.normalize_or_zero())
    }

    /// Add a waypoint.
    /// Create flows from all points to the waypoint.
    pub fn add_waypoint(
        &mut self,
        entity: Entity,
        waypoint_rowcol: RowCol,
        traveler_rowcols: &[RowCol],
        obstacles: &Grid2<Obstacle>,
        event_writer: &mut EventWriter<NavigationCostEvent>,
    ) {
        if traveler_rowcols.is_empty() {
            return;
        }
        let costs = self.a_star(traveler_rowcols, waypoint_rowcol, obstacles);
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
            self[rowcol].insert(
                entity,
                Acceleration(rowcol.signed_delta8(min_neighbor_rowcol)),
            );

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
        &self,
        sources: &[RowCol],
        destination: RowCol,
        obstacles: &Grid2<Obstacle>,
    ) -> HashMap<RowCol, f32> {
        // Initial setup.
        let mut costs: HashMap<RowCol, f32> = HashMap::new();
        let mut heap: BinaryHeap<AStarState> = BinaryHeap::new();
        let mut goals: BTreeSet<RowCol> = sources
            .iter()
            .filter(|&&rowcol| obstacles[rowcol] == Obstacle::Empty)
            .copied()
            .collect();
        let mut current_goal = *goals.first().unwrap();

        let min_grid_dist = 1.;
        let max_grid_dist = 30.;
        let max_dist = goals
            .iter()
            .map(|&rowcol| destination.distance8(rowcol))
            .fold(0f32, |a, b| a.max(b));
        let max_heuristic = 0.9;
        let final_dist = max_dist.clamp(min_grid_dist, max_grid_dist);
        let heuristic_factor = max_heuristic * final_dist / max_grid_dist;
        // We're at `start`, with a zero cost
        heap.push(AStarState {
            cost: 0.,
            rowcol: destination,
            heuristic: 0.,
        });

        // Examine the frontier with lower cost nodes first (min-heap)
        while let Some(AStarState {
            cost,
            rowcol,
            heuristic: _,
        }) = heap.pop()
        {
            // Skip already finalized cells.
            if costs.contains_key(&rowcol) {
                continue;
            }

            // Costs inserted here are guaranteed to be the best costs seen so far.
            costs.insert(rowcol, cost);

            // If the current goal has been reached, clear the heap and move on to the next goal.
            if goals.remove(&rowcol) {
                if goals.is_empty() {
                    break;
                }
                if rowcol == current_goal {
                    current_goal = *goals.first().unwrap();
                }
            }
            // For each node we can reach, see if we can find a way with
            // a lower cost going through this node
            for (neighbor_rowcol, neighbor_cost) in self.neighbors8(rowcol) {
                // Skip out of bounds positions.
                if self.is_boundary(neighbor_rowcol) {
                    continue;
                }

                if obstacles[neighbor_rowcol] != Obstacle::Empty {
                    continue;
                }

                heap.push(AStarState {
                    cost: cost + neighbor_cost,
                    rowcol: neighbor_rowcol,
                    heuristic: heuristic_factor * neighbor_rowcol.distance8(current_goal),
                });
            }
        }
        costs
    }
}
