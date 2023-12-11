use std::ops::RangeInclusive;

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    utils::{HashMap, HashSet},
};

use crate::{
    fog::{FogAssets, FogPlane, FogShaderMaterial},
    objects::{Configs, Team},
    zindex, Aabb2, SystemStage,
};

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
    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut query: Query<(Entity, &Transform, &Team, &mut Visibility), With<Self>>,
        mut grid: ResMut<EntityGrid>,
        grid_assets: Res<GridAssets>,
        mut grid_shader_assets: ResMut<Assets<GridShaderMaterial>>,
        fog_assets: Res<FogAssets>,
        mut fog_shader_assets: ResMut<Assets<FogShaderMaterial>>,
        spec: Res<EntityGridSpec>,
        configs: Res<Configs>,
    ) {
        // Initialize the shader if not yet initialized.
        let grid_material: &mut GridShaderMaterial = grid_shader_assets
            .get_mut(&grid_assets.shader_material)
            .unwrap();
        let fog_material = fog_shader_assets
            .get_mut(&fog_assets.shader_material)
            .unwrap();
        for (entity, transform, team, mut visibility) in &mut query {
            let grid_material = if spec.visualize {
                Some(grid_material.grid.as_mut_slice())
            } else {
                None
            };
            grid.update_entity(
                entity,
                transform.translation.xy(),
                *team,
                &configs,
                &mut fog_material.grid,
                grid_material,
            );
            *visibility = grid.get_visibility(transform.translation.xy(), configs.player_team)
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

    #[allow(clippy::too_many_arguments)]
    pub fn visualize_on_change(
        spec: Res<Self>,
        grid_assets: Res<GridAssets>,
        fog_assets: Res<FogAssets>,

        query: Query<Entity, With<CellVisualizer>>,
        fog_query: Query<Entity, With<FogPlane>>,

        mut grid_shader_assets: ResMut<Assets<GridShaderMaterial>>,
        mut fog_shader_assets: ResMut<Assets<FogShaderMaterial>>,
        mut commands: Commands,
    ) {
        if !spec.is_changed() {
            return;
        }

        // Cleanup entities on change.
        for entity in &query {
            commands.entity(entity).despawn();
        }
        for entity in &fog_query {
            commands.entity(entity).despawn();
        }

        // Initialize the grid visualization shader.
        if spec.visualize {
            let material = grid_shader_assets
                .get_mut(&grid_assets.shader_material)
                .unwrap();
            material.resize(&spec);
        }

        // Initialize the fog shader, which also uses the grid spec.
        {
            let material = fog_shader_assets
                .get_mut(&fog_assets.shader_material)
                .unwrap();
            material.resize(&spec);
        }

        commands.spawn(CellVisualizer { active: true }.bundle(&spec, &grid_assets));
        commands.spawn(FogPlane::default().bundle(&spec, &fog_assets));
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
    pub entities: Vec<HashSet<Entity>>,
    pub team_visibility: Vec<Vec<u32>>,
    pub entity_to_rowcol: HashMap<Entity, (u8, u8)>,
}
impl EntityGrid {
    pub fn resize(&mut self) {
        let num_cells = self.spec.rows as usize * self.spec.cols as usize;
        self.entities.resize(num_cells, HashSet::default());
        self.team_visibility
            .resize(num_cells, vec![0; Team::count()])
    }

    /// Update an entity's position in the grid.
    pub fn update_entity(
        &mut self,
        entity: Entity,
        position: Vec2,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
        mut grid: Option<&mut [u32]>,
    ) {
        let (row, col) = self.to_rowcol(position);

        // Remove this entity's old position if it was different.
        if let Some(&(old_row, old_col)) = self.entity_to_rowcol.get(&entity) {
            // If in same position, do nothing.
            if (old_row, old_col) == (row, col) {
                return;
            }

            if let Some(entities) = self.get_mut(old_row, old_col) {
                entities.remove(&entity);
                if let Some(grid) = grid.as_deref_mut() {
                    if entities.is_empty() {
                        grid[self.index(old_row, old_col)] = 0;
                    }
                }
                self.remove_visibility(old_row, old_col, team, configs, visibility);
            }
        }

        if let Some(entities) = self.get_mut(row, col) {
            entities.insert(entity);
            self.entity_to_rowcol.insert(entity, (row, col));
            if let Some(grid) = grid {
                grid[self.index(row, col)] = 1;
            }
            self.add_visibility(row, col, team, configs, visibility);
        }
    }

    fn in_radius(row: u8, col: u8, other_row: u8, other_col: u8, radius: u8) -> bool {
        let row_dist = other_row as i8 - row as i8;
        let col_dist = other_col as i8 - col as i8;
        row_dist * row_dist + col_dist * col_dist < radius as i8 * radius as i8
    }

    fn cell_range(center: u8, radius: u8) -> RangeInclusive<u8> {
        let (min, max) = ((center as i8 - radius as i8).max(0) as u8, center + radius);
        min..=max
    }

    fn remove_visibility(
        &mut self,
        row: u8,
        col: u8,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for other_row in Self::cell_range(row, radius) {
            for other_col in Self::cell_range(col, radius) {
                if !Self::in_radius(row, col, other_row, other_col, radius) {
                    continue;
                }

                let i = self.index(other_row, other_col);
                if let Some(grid_visibility) = self.team_visibility.get_mut(i) {
                    if grid_visibility[team as usize] > 0 {
                        grid_visibility[team as usize] -= 1;
                        if team == configs.player_team && grid_visibility[team as usize] == 0 {
                            visibility[i] = 0.5;
                        }
                    }
                }
            }
        }
    }

    fn add_visibility(
        &mut self,
        row: u8,
        col: u8,
        team: Team,
        configs: &Configs,
        visibility: &mut [f32],
    ) {
        let radius = configs.visibility_radius;
        for other_row in Self::cell_range(row, radius) {
            for other_col in Self::cell_range(col, radius) {
                if !Self::in_radius(row, col, other_row, other_col, radius) {
                    continue;
                }

                let i = self.index(other_row, other_col);
                if let Some(grid_visibility) = self.team_visibility.get_mut(i) {
                    grid_visibility[team as usize] += 1;
                    if team == configs.player_team {
                        visibility[i] = 0.
                    }
                }
            }
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
        self.entities.get(index)
    }

    pub fn get_visibility(&self, position: Vec2, team: Team) -> Visibility {
        let (row, col) = self.to_rowcol(position);
        let i = self.index(row, col);
        if self.team_visibility[i][team as usize] > 0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        }
    }

    /// Get the mutable set of entities at the current position.
    pub fn get_mut(&mut self, row: u8, col: u8) -> Option<&mut HashSet<Entity>> {
        let index = self.index(row, col);
        self.entities.get_mut(index)
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
    pub shader_material: Handle<GridShaderMaterial>,
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
            shader_material,
            gray_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.15))),
            dark_gray_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.3))),
            blue_material: materials.add(ColorMaterial::from(Color::GRAY.with_a(0.1))),

            dark_blue_material: materials.add(ColorMaterial::from(Color::DARK_GRAY.with_a(0.1))),
        }
    }
}

/// Component to visualize a cell.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct CellVisualizer {
    pub active: bool,
}
impl CellVisualizer {
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
                material: assets.shader_material.clone(),
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
impl GridShaderMaterial {
    fn resize(&mut self, spec: &EntityGridSpec) {
        self.width = spec.width;
        self.rows = spec.rows.into();
        self.cols = spec.cols.into();
        self.grid.resize(spec.rows as usize * spec.cols as usize, 0);
    }
}
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
            entities: Vec::default(),
            entity_to_rowcol: HashMap::default(),
            team_visibility: Vec::default(),
        };
        grid.resize();
        assert_eq!(grid.spec.offset(), Vec2 { x: 50.0, y: 50.0 });
        let rowcol = grid.to_rowcol(Vec2 { x: 0., y: 0. });
        assert_eq!(rowcol, (5, 5));

        assert!(grid.get_mut(5, 5).is_some());
        assert!(grid.get(5, 5).is_some());
    }

    #[test]
    fn grid_radius() {
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (2, 2);
            let radius = 2;
            assert!(EntityGrid::in_radius(
                row, col, other_row, other_col, radius
            ));
        }
        {
            let (row, col) = (1, 1);
            let (other_row, other_col) = (4, 4);
            let radius = 2;
            assert!(!EntityGrid::in_radius(
                row, col, other_row, other_col, radius
            ));
        }
    }
}
