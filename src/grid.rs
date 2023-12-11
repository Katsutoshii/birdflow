use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    utils::{HashMap, HashSet},
};

use crate::{zindex, Aabb2, SystemStage};

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct GridPlugin;
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EntityGridSpec>()
            .add_plugins(Material2dPlugin::<GridShaderMaterial>::default())
            .init_resource::<GridAssets>()
            .insert_resource(EntityGrid::default())
            .add_systems(
                FixedUpdate,
                (
                    GridEntity::update.in_set(SystemStage::PostApply),
                    EntityGridSpec::visualize_on_change,
                    EntityGridSpec::resize_on_change,
                ),
            );
    }
}

// Component to track an entity in the grid.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct GridEntity;
impl GridEntity {
    pub fn update(
        query: Query<(Entity, &Transform), With<Self>>,
        mut grid: ResMut<EntityGrid>,
        grid_assets: Res<GridAssets>,
        mut assets: ResMut<Assets<GridShaderMaterial>>,
        spec: Res<EntityGridSpec>,
    ) {
        // Initialize the shader if not yet initialized.
        let material: &mut GridShaderMaterial =
            assets.get_mut(&grid_assets.grid_shader_material).unwrap();
        if spec.visualize && material.grid.is_empty() {
            material
                .grid
                .resize(grid.spec.rows as usize * grid.spec.cols as usize, 0);
        }

        for (entity, transform) in &query {
            if spec.visualize {
                grid.update_entity_visualizer(
                    entity,
                    transform.translation.xy(),
                    &mut material.grid,
                )
            } else {
                grid.update_entity(entity, transform.translation.xy());
            }
        }
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

    pub fn visualize_on_change(
        spec: Res<Self>,
        grid_assets: Res<GridAssets>,
        query: Query<Entity, With<CellVisualizerShader>>,

        mut assets: ResMut<Assets<GridShaderMaterial>>,
        mut commands: Commands,
    ) {
        // Cleanup old visualizer on change.
        if !spec.is_changed() {
            return;
        }
        for entity in &query {
            commands.entity(entity).despawn();
        }

        // Initialize the shader
        let material: &mut GridShaderMaterial =
            assets.get_mut(&grid_assets.grid_shader_material).unwrap();
        if spec.visualize && material.grid.is_empty() {
            material
                .grid
                .resize(spec.rows as usize * spec.cols as usize, 0);
        }

        commands.spawn(CellVisualizerShader { active: false }.bundle(&spec, &grid_assets));
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

    pub fn scale(&self) -> Vec2 {
        Vec2 {
            x: self.width * self.cols as f32,
            y: self.width * self.rows as f32,
        }
    }
}

/// A grid of cells that keep track of what entities are contained within them.
#[derive(Resource, Default)]
pub struct EntityGrid {
    pub spec: EntityGridSpec,
    pub cells: Vec<HashSet<Entity>>,
    pub entity_to_rowcol: HashMap<Entity, (u8, u8)>,
    pub shader_material: Handle<GridShaderMaterial>,
}
impl EntityGrid {
    pub fn resize(&mut self) {
        let num_cells = self.spec.rows as usize * self.spec.cols as usize;
        self.cells.resize(num_cells, HashSet::default());
    }

    /// Update an entity's position in the grid.
    pub fn update_entity(&mut self, entity: Entity, position: Vec2) {
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

    /// Update an entity's position in the grid.
    pub fn update_entity_visualizer(&mut self, entity: Entity, position: Vec2, grid: &mut [u32]) {
        let (row, col) = self.to_rowcol(position);

        // Remove this entity's old position if it was different.
        if let Some(&(old_row, old_col)) = self.entity_to_rowcol.get(&entity) {
            // If in same position, do nothing.
            if (old_row, old_col) == (row, col) {
                return;
            }

            if let Some(cell) = self.get_mut(old_row, old_col) {
                cell.remove(&entity);
                if cell.is_empty() {
                    grid[self.index(old_row, old_col)] = 0;
                }
            }
        }

        if let Some(cell) = self.get_mut(row, col) {
            cell.insert(entity);
            self.entity_to_rowcol.insert(entity, (row, col));
            grid[self.index(row, col)] = 1;
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
    pub grid_shader_material: Handle<GridShaderMaterial>,
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
        let shader_material = {
            let mut materials = world
                .get_resource_mut::<Assets<GridShaderMaterial>>()
                .unwrap();
            materials.add(GridShaderMaterial::default())
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            gray_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.15))),
            dark_gray_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.3))),
            blue_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.1))),

            dark_blue_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.1))),
            grid_shader_material: shader_material,
        }
    }
}

/// Component to visualize a cell.
#[derive(Debug, Default, Component, Clone)]
pub struct CellVisualizerShader {
    pub active: bool,
}
impl CellVisualizerShader {
    pub fn bundle(self, spec: &EntityGridSpec, assets: &GridAssets) -> impl Bundle {
        (
            MaterialMesh2dBundle::<GridShaderMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(spec.scale().extend(1.))
                    .with_translation(Vec3 {
                        x: 0.,
                        y: 0.,
                        z: zindex::SHADER_BACKGROUND,
                    }),
                material: assets.grid_shader_material.clone(),
                ..default()
            },
            Name::new("GridVis"),
            self,
        )
    }
}

// This is the struct that will be passed to your shader
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GridShaderMaterial {
    #[uniform(0)]
    color: Color,
    #[uniform(1)]
    width: f32,
    #[uniform(2)]
    rows: u32,
    #[uniform(3)]
    cols: u32,
    #[storage(4, read_only)]
    grid: Vec<u32>,
}
impl Default for GridShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            width: 100.,
            rows: 50,
            cols: 100,
            grid: Vec::default(),
        }
    }
}
/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
/// When using the GLSL shading language for your shader, the specialize method must be overridden.
impl Material2d for GridShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/grid_background.wgsl".into()
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
            shader_material: Handle::default(),
        };
        grid.resize();
        assert_eq!(grid.spec.offset(), Vec2 { x: 50.0, y: 50.0 });
        let rowcol = grid.to_rowcol(Vec2 { x: 0., y: 0. });
        assert_eq!(rowcol, (5, 5));

        assert!(grid.get_mut(5, 5).is_some());
        assert!(grid.get(5, 5).is_some());
    }
}
