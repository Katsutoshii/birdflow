use crate::{meshes::UNIT_SQUARE, prelude::*};
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
};

use super::{Grid2, GridSpec, RowCol};

/// Plugin for obstacles.
/// Obstacles are implemented as a hacky force field in the center of each cell they are present in.
/// TODO: prevent glitchy movement when objects try to move past obstacles.
pub struct ObstaclesPlugin;
impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<ObstaclesShaderMaterial>::default())
            .register_type::<ObstaclesSpec>()
            .register_type::<Obstacle>()
            .register_type::<Vec<(RowCol, Obstacle)>>()
            .register_type::<(RowCol, Obstacle)>()
            .register_type::<RowCol>()
            .insert_resource(ObstaclesGrid::default())
            .init_resource::<ObstaclesAssets>()
            .add_systems(
                FixedUpdate,
                (
                    ObstaclesGrid::resize_on_change,
                    ObstaclesGrid::update.after(ObstaclesGrid::resize_on_change),
                    ObstaclesShaderMaterial::update.after(ObstaclesGrid::resize_on_change),
                ),
            );
    }
}

// Represents obstacle presence and orientation
#[derive(Default, Reflect, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum Obstacle {
    #[default]
    Empty = 0,
    UpRight = 1,
    UpLeft = 2,
    DownRight = 3,
    DownLeft = 4,
    Full = 5,
}

/// Grid of obstacle data.
#[derive(Resource, Default, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub struct ObstaclesSpec(pub Vec<(RowCol, Obstacle)>);

/// Grid of obstacle data.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct ObstaclesGrid(pub Grid2<Obstacle>);
impl ObstaclesGrid {
    pub fn resize_on_change(
        mut grid: ResMut<Self>,
        grid_spec: Res<GridSpec>,
        assets: Res<ObstaclesAssets>,
        query: Query<Entity, With<ObstaclesPlane>>,
        mut shader_assets: ResMut<Assets<ObstaclesShaderMaterial>>,
        mut commands: Commands,
    ) {
        if !grid_spec.is_changed() {
            return;
        }
        for entity in &query {
            commands.entity(entity).despawn();
        }

        grid.resize_with(grid_spec.clone());

        let material = shader_assets.get_mut(&assets.shader_material).unwrap();
        material.resize(&grid_spec);

        commands.spawn(ObstaclesPlane.bundle(&grid_spec, &assets));
    }

    pub fn update(mut grid: ResMut<Self>, spec: Res<ObstaclesSpec>) {
        if !spec.is_changed() {
            return;
        }
        // Reset all to 0.
        grid.cells.fill(Obstacle::Empty);
        for &((row, col), face) in spec.iter() {
            grid[(row, col)] = face;
        }
    }

    fn obstacle_acceleration(
        &self,
        position: Vec2,
        cell: RowCol,
        velocity: Velocity,
    ) -> Acceleration {
        if self[cell] == Obstacle::Empty {
            return Acceleration(Vec2::ZERO);
        }
        let obstacle_position = self.to_world_position(cell);
        let d = obstacle_position - position;
        let v_dot_d = velocity.dot(d);
        let d_dot_d = d.dot(d);

        // If moving towards the obstacle, accelerate away from the obstacle.
        if v_dot_d > 0.01 {
            let magnitude = (self.spec.width - position.distance(obstacle_position)).max(0.);
            let projection = d * (d_dot_d / v_dot_d);
            Acceleration(-magnitude * projection)
        } else {
            Acceleration(Vec2::ZERO)
        }
    }

    /// Compute acceleration due to neighboring obstacles.
    /// For each neighboring obstacle, if the object is moving towards the obstacle
    /// we apply a force away from the obstacle.
    pub fn obstacles_acceleration(
        &self,
        position: Vec2,
        velocity: Velocity,
        acceleration: Acceleration,
    ) -> Acceleration {
        // Apply one step of integration to anticipate movement from this step.
        let next_velocity = Velocity(velocity.0 + acceleration.0);
        let mut acceleration = Acceleration(Vec2::ZERO);

        for (row, col) in self.get_in_radius(position, self.width * 2.) {
            acceleration += self.obstacle_acceleration(position, (row, col), next_velocity)
        }
        Acceleration(acceleration.clamp_length(0., next_velocity.length()))
    }
}

/// Parameters passed to grid background shader.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ObstaclesShaderMaterial {
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
impl Default for ObstaclesShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::MIDNIGHT_BLUE,
            width: 64.,
            rows: 100,
            cols: 100,
            grid: Vec::default(),
        }
    }
}
impl ObstaclesShaderMaterial {
    pub fn resize(&mut self, spec: &GridSpec) {
        self.width = spec.width;
        self.rows = spec.rows.into();
        self.cols = spec.cols.into();
        self.grid.resize(
            spec.rows as usize * spec.cols as usize,
            Obstacle::Empty as u32,
        );
    }

    /// Update the grid shader material.
    pub fn update(
        grid_spec: Res<GridSpec>,
        spec: Res<ObstaclesSpec>,
        assets: Res<ObstaclesAssets>,
        mut shader_assets: ResMut<Assets<ObstaclesShaderMaterial>>,
    ) {
        if !spec.is_changed() {
            return;
        }
        let material: &mut ObstaclesShaderMaterial =
            shader_assets.get_mut(&assets.shader_material).unwrap();

        material.grid.fill(Obstacle::Empty as u32);
        for &(rowcol, face) in spec.iter() {
            material.grid[grid_spec.flat_index(rowcol)] = face as u32;
        }
    }
}
impl Material2d for ObstaclesShaderMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/obstacles.wgsl".into()
    }
}

/// Handles to common fog assets.
#[derive(Resource)]
pub struct ObstaclesAssets {
    pub mesh: Handle<Mesh>,
    pub shader_material: Handle<ObstaclesShaderMaterial>,
}
impl FromWorld for ObstaclesAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(UNIT_SQUARE))
        };
        let shader_material = {
            let mut materials = world
                .get_resource_mut::<Assets<ObstaclesShaderMaterial>>()
                .unwrap();
            materials.add(ObstaclesShaderMaterial::default())
        };
        Self {
            mesh,
            shader_material,
        }
    }
}

/// Fog plane between the world and the camera.
#[derive(Debug, Default, Component, Clone)]
#[component(storage = "SparseSet")]
pub struct ObstaclesPlane;
impl ObstaclesPlane {
    pub fn bundle(self, spec: &GridSpec, assets: &ObstaclesAssets) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ObstaclesShaderMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(spec.scale().extend(1.))
                    .with_translation(Vec3 {
                        x: 0.,
                        y: 0.,
                        z: zindex::OBSTACLES,
                    }),
                material: assets.shader_material.clone(),
                ..default()
            },
            Name::new("GridVis"),
            self,
        )
    }
}
