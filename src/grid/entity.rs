use bevy::{prelude::*, utils::HashSet};

use crate::{objects::Config, prelude::Aabb2};

use super::{Grid2, GridSpec};
use std::ops::{Deref, DerefMut};

/// Component to track an entity in the grid.
/// Holds its cell position so it can move/remove itself from the grid.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GridEntity {
    pub cell: Option<(u16, u16)>,
}
impl GridEntity {
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut query: Query<(Entity, &mut Self, &Transform)>,
        mut grid: ResMut<EntityGrid>,
        mut event_writer: EventWriter<EntityGridEvent>,
    ) {
        for (entity, mut grid_entity, transform) in &mut query {
            if let Some(event) =
                grid.update_entity(entity, grid_entity.cell, transform.translation.xy())
            {
                grid_entity.cell = Some(event.cell);
                event_writer.send(event);
            }
        }
    }
}

/// Communicates updates to the grid to other symptoms.
#[derive(Event)]
pub struct EntityGridEvent {
    pub entity: Entity,
    pub prev_cell: Option<(u16, u16)>,
    pub prev_cell_empty: bool,
    pub cell: (u16, u16),
}
impl Default for EntityGridEvent {
    fn default() -> Self {
        Self {
            entity: Entity::PLACEHOLDER,
            prev_cell: None,
            prev_cell_empty: false,
            cell: (0, 0),
        }
    }
}

/// A grid of cells that keep track of what entities are contained within them.
#[derive(Resource, Default)]
pub struct EntityGrid(Grid2<HashSet<Entity>>);
impl Deref for EntityGrid {
    type Target = Grid2<HashSet<Entity>>;
    fn deref(&self) -> &Grid2<HashSet<Entity>> {
        &self.0
    }
}
impl DerefMut for EntityGrid {
    fn deref_mut(&mut self) -> &mut Grid2<HashSet<Entity>> {
        &mut self.0
    }
}
impl EntityGrid {
    /// When the spec changes, update the grid spec and resize.
    pub fn resize_on_change(mut grid: ResMut<Self>, spec: Res<GridSpec>) {
        if !spec.is_changed() {
            return;
        }
        grid.resize_with(spec.clone())
    }

    /// Update an entity's position in the grid.
    pub fn update_entity(
        &mut self,
        entity: Entity,
        cell: Option<(u16, u16)>,
        position: Vec2,
    ) -> Option<EntityGridEvent> {
        let (row, col) = self.spec.to_rowcol(position);

        // Remove this entity's old position if it was different.
        let mut prev_cell: Option<(u16, u16)> = None;
        let mut prev_cell_empty: bool = false;
        if let Some((prev_row, prev_col)) = cell {
            // If in same position, do nothing.
            if (prev_row, prev_col) == (row, col) {
                return None;
            }

            if let Some(entities) = self.get_mut(prev_row, prev_col) {
                entities.remove(&entity);
                prev_cell = Some((prev_row, prev_col));
                prev_cell_empty = entities.is_empty();
            }
        }

        if let Some(entities) = self.get_mut(row, col) {
            entities.insert(entity);
            return Some(EntityGridEvent {
                entity,
                prev_cell,
                prev_cell_empty,
                cell: (row, col),
            });
        }
        None
    }

    pub fn get_entities_in_radius(&self, position: Vec2, config: &Config) -> HashSet<Entity> {
        let mut other_entities: HashSet<Entity> = HashSet::default();
        let positions = self.get_in_radius(position, config.neighbor_radius);
        for (row, col) in positions {
            other_entities.extend(self.get(row, col).unwrap());
        }
        other_entities
    }
    /// Remove an entity from the grid entirely.
    pub fn remove(&mut self, entity: Entity, grid_entity: &GridEntity) {
        if let Some((row, col)) = grid_entity.cell {
            if let Some(cell) = self.get_mut(row, col) {
                cell.remove(&entity);
            } else {
                error!("No cell at {:?}.", (row, col))
            }
        } else {
            error!("No row col for {:?}", entity)
        }
    }

    /// Get all entities in a given bounding box.
    pub fn get_entities_in_aabb(&self, aabb: &Aabb2) -> Vec<Entity> {
        let mut result = HashSet::default();

        for (row, col) in self.get_in_aabb(aabb) {
            if let Some(set) = self.get(row, col) {
                result.extend(set.iter());
            }
        }
        result.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::{Grid2, GridSpec};

    use super::EntityGrid;
    use bevy::prelude::*;

    #[test]
    fn test_update() {
        let mut grid = EntityGrid(Grid2 {
            spec: GridSpec {
                rows: 10,
                cols: 10,
                width: 10.0,
                visualize: false,
            },
            ..Default::default()
        });
        grid.resize();
        assert_eq!(grid.spec.offset(), Vec2 { x: 50.0, y: 50.0 });
        let rowcol = grid.spec.to_rowcol(Vec2 { x: 0., y: 0. });
        assert_eq!(rowcol, (5, 5));

        assert!(grid.get_mut(5, 5).is_some());
        assert!(grid.get(5, 5).is_some());
    }
}
