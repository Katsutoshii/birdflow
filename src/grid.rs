use bevy::{
    prelude::*,
    sprite::MaterialMesh2dBundle,
    utils::{HashMap, HashSet},
};

use crate::{zindex, Aabb2};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EntityGridSpec>()
            .init_resource::<GridAssets>()
            .insert_resource(EntityGrid::default())
            .add_systems(
                FixedUpdate,
                (
                    EntityGridSpec::visualize_on_change,
                    EntityGridSpec::resize_on_change,
                    CellVisualizer::update,
                ),
            );
    }
}

/// Specification describing how large the grid is.
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct EntityGridSpec {
    pub rows: u8,
    pub cols: u8,
    pub width: f32,
    pub visualize: bool,
}
impl Default for EntityGridSpec {
    fn default() -> Self {
        Self {
            rows: 10,
            cols: 10,
            width: 10.0,
            visualize: true,
        }
    }
}
impl EntityGridSpec {
    /// When the spec changes, update the grid spec and resize.
    pub fn resize_on_change(spec: Res<EntityGridSpec>, mut grid: ResMut<EntityGrid>) {
        if !spec.is_changed() {
            return;
        }

        grid.spec = spec.clone();
        grid.resize();
    }

    /// When the spec changes, remove all visualizers and respawn them with the updated spec.
    pub fn visualize_on_change(
        spec: ResMut<Self>,
        assets: Res<GridAssets>,
        query: Query<Entity, With<CellVisualizer>>,
        mut commands: Commands,
    ) {
        // Cleanup old cells on change.
        if !spec.is_changed() {
            return;
        }
        for entity in &query {
            commands.entity(entity).despawn();
        }

        // Spawn new cells
        if !spec.visualize {
            return;
        }
        for row in 0..spec.rows {
            for col in 0..spec.cols {
                commands.spawn(
                    CellVisualizer {
                        row,
                        col,
                        active: false,
                    }
                    .bundle(&spec, &assets),
                );
            }
        }
    }

    /// Compute the offset vector for this grid spec.
    pub fn offset(&self) -> Vec2 {
        Vec2 {
            x: self.width * self.cols as f32 / 2.,
            y: self.width * self.rows as f32 / 2.,
        }
    }

    /// Compute the (min, max) position for the grid.
    pub fn world2d_bounds(&self) -> Aabb2 {
        Aabb2 {
            min: -self.offset(),
            max: self.offset(),
        }
    }
}

/// A grid of cells that keep track of what entities are contained within them.
#[derive(Resource, Default)]
pub struct EntityGrid {
    pub spec: EntityGridSpec,
    pub cells: Vec<HashSet<Entity>>,
    pub entity_to_rowcol: HashMap<Entity, (u8, u8)>,
}
impl EntityGrid {
    pub fn resize(&mut self) {
        let num_cells = self.spec.rows as usize * self.spec.cols as usize;
        self.cells.resize(num_cells, HashSet::default());
    }

    /// Update an entity's position in the grid.
    pub fn update(&mut self, entity: Entity, position: Vec2) {
        let (row, col) = self.to_rowcol(position);

        // Remove this entity's old position if it was different.
        if let Some(&(old_row, old_col)) = self.entity_to_rowcol.get(&entity) {
            // If in same position, do nothing.
            if (old_row, old_col) == (row, col) {
                return;
            }

            if let Some(cell) = self.get_mut(old_row, old_col) {
                cell.remove(&entity);
            }
        }

        if let Some(cell) = self.get_mut(row, col) {
            cell.insert(entity);
            self.entity_to_rowcol.insert(entity, (row, col));
        }
    }

    /// Remove an entity from the grid entirely.
    #[allow(dead_code)]
    pub fn remove(&mut self, entity: Entity) {
        if let Some(&(row, col)) = self.entity_to_rowcol.get(&entity) {
            if let Some(cell) = self.get_mut(row, col) {
                cell.remove(&entity);
            } else {
                error!("No cell at {:?}.", (row, col))
            }
        } else {
            error!("No row col for {:?}", entity)
        }
    }

    /// Get the set of entities at the current position.
    pub fn get(&self, row: u8, col: u8) -> Option<&HashSet<Entity>> {
        let index = self.index(row, col);
        self.cells.get(index)
    }

    /// Get the mutable set of entities at the current position.
    pub fn get_mut(&mut self, row: u8, col: u8) -> Option<&mut HashSet<Entity>> {
        let index = self.index(row, col);
        self.cells.get_mut(index)
    }

    pub fn get_in_aabb(&self, aabb: &Aabb2) -> Vec<Entity> {
        let mut result = HashSet::default();

        let (min_row, min_col) = self.to_rowcol(aabb.min);
        let (max_row, max_col) = self.to_rowcol(aabb.max);
        for row in min_row..=max_row {
            for col in min_col..=max_col {
                if let Some(set) = self.get(row, col) {
                    result.extend(set.iter());
                }
            }
        }
        result.into_iter().collect()
    }

    /// Get all entities in radius.
    pub fn get_in_radius(&self, position: Vec2, radius: f32) -> Vec<Entity> {
        self.get_in_aabb(&Aabb2 {
            min: position + (Vec2::NEG_ONE * radius),
            max: position + (Vec2::ONE * radius),
        })
    }

    /// Returns (row, col) from a position in world space.
    fn to_rowcol(&self, mut position: Vec2) -> (u8, u8) {
        position += self.spec.offset();
        (
            (position.y / self.spec.width) as u8,
            (position.x / self.spec.width) as u8,
        )
    }

    // Covert row, col to a single index.
    fn index(&self, row: u8, col: u8) -> usize {
        row as usize * self.spec.cols as usize + col as usize
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct GridAssets {
    pub mesh: Handle<Mesh>,
    pub gray_material: Handle<ColorMaterial>,
    pub dark_gray_material: Handle<ColorMaterial>,
    pub blue_material: Handle<ColorMaterial>,
    pub dark_blue_material: Handle<ColorMaterial>,
}
impl FromWorld for GridAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            // Unit square
            meshes.add(Mesh::from(shape::Box {
                min_x: -0.5,
                max_x: 0.5,
                min_y: -0.5,
                max_y: 0.5,
                min_z: 0.0,
                max_z: 0.0,
            }))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            gray_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.15))),
            dark_gray_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.3))),
            blue_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.1))),
            dark_blue_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.1))),
        }
    }
}

/// Component to visualize a cell.
#[derive(Debug, Default, Component, Clone)]
pub struct CellVisualizer {
    pub row: u8,
    pub col: u8,
    pub active: bool,
}
impl CellVisualizer {
    pub fn bundle(self, spec: &EntityGridSpec, assets: &GridAssets) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(spec.width))
                    .with_translation(
                        Vec3 {
                            x: (0.5 + self.col as f32) * spec.width,
                            y: (0.5 + self.row as f32) * spec.width,
                            z: zindex::BACKGROUND,
                        } - spec.offset().extend(0.),
                    ),
                material: self.get_color_material(assets),
                ..default()
            },
            Name::new("Cell"),
            self,
        )
    }

    fn get_color_material(&self, assets: &GridAssets) -> Handle<ColorMaterial> {
        if self.active {
            if (self.row + self.col) % 2 == 0 {
                assets.blue_material.clone()
            } else {
                assets.dark_blue_material.clone()
            }
        } else {
            if (self.row + self.col) % 2 == 0 {
                assets.gray_material.clone()
            } else {
                assets.dark_gray_material.clone()
            }
        }
    }

    pub fn update(
        grid: Res<EntityGrid>,
        assets: Res<GridAssets>,
        mut query: Query<(&mut Self, &mut Handle<ColorMaterial>)>,
        input: Res<Input<KeyCode>>,
    ) {
        if input.just_pressed(KeyCode::G) {
            dbg!(&grid.cells);
        }
        if !grid.is_changed() {
            return;
        }
        for (mut cell, mut color) in &mut query {
            if let Some(entities) = grid.get(cell.row, cell.col) {
                let active = !entities.is_empty();
                if cell.active != active {
                    cell.active = active;
                    *color = cell.get_color_material(&assets);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::grid::EntityGridSpec;

    use super::EntityGrid;
    use bevy::{prelude::*, utils::HashMap};

    #[test]
    fn test_update() {
        let mut grid = EntityGrid {
            spec: EntityGridSpec {
                rows: 10,
                cols: 10,
                width: 10.0,
                visualize: false,
            },
            cells: Vec::default(),
            entity_to_rowcol: HashMap::default(),
        };
        grid.resize();
        assert_eq!(grid.spec.offset(), Vec2 { x: 50.0, y: 50.0 });
        let rowcol = grid.to_rowcol(Vec2 { x: 0., y: 0. });
        assert_eq!(rowcol, (5, 5));

        assert!(matches!(grid.get_mut(5, 5), Some(_)));
        assert!(matches!(grid.get(5, 5), Some(_)));
    }
}
