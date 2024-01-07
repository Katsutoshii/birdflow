use crate::prelude::*;
use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::Material2d,
};

use super::{
    shader_plane::{ShaderPlaneAssets, ShaderPlanePlugin},
    GridShaderMaterial,
};

/// Plugin for obstacles.
/// Obstacles are implemented as a hacky force field in the center of each cell they are present in.
/// TODO: prevent glitchy movement when objects try to move past obstacles.
pub struct ObstaclesPlugin;
impl Plugin for ObstaclesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShaderPlanePlugin::<ObstaclesShaderMaterial>::default())
            .add_plugins(Grid2Plugin::<Obstacle>::default())
            .register_type::<ObstaclesSpec>()
            .register_type::<Obstacle>()
            .register_type::<Vec<(RowCol, Obstacle)>>()
            .register_type::<(RowCol, Obstacle)>()
            .register_type::<RowCol>()
            .add_systems(
                FixedUpdate,
                (
                    Grid2::<Obstacle>::update.after(Grid2::<Obstacle>::resize_on_change),
                    ObstaclesShaderMaterial::update.after(Grid2::<Obstacle>::resize_on_change),
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

impl Grid2<Obstacle> {
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
    size: GridSize,
    #[storage(2, read_only)]
    grid: Vec<u32>,
}
impl Default for ObstaclesShaderMaterial {
    fn default() -> Self {
        Self {
            color: Color::MIDNIGHT_BLUE,
            size: GridSize::default(),
            grid: Vec::default(),
        }
    }
}
impl GridShaderMaterial for ObstaclesShaderMaterial {
    fn zindex() -> f32 {
        zindex::OBSTACLES
    }

    fn resize(&mut self, spec: &GridSpec) {
        self.size.width = spec.width;
        self.size.rows = spec.rows.into();
        self.size.cols = spec.cols.into();
        self.grid.resize(
            spec.rows as usize * spec.cols as usize,
            Obstacle::Empty as u32,
        );
    }
}
impl ObstaclesShaderMaterial {
    /// Update the grid shader material.
    pub fn update(
        grid_spec: Res<GridSpec>,
        spec: Res<ObstaclesSpec>,
        assets: Res<ShaderPlaneAssets<Self>>,
        mut shader_assets: ResMut<Assets<Self>>,
    ) {
        if !spec.is_changed() {
            return;
        }
        let material = shader_assets.get_mut(&assets.shader_material).unwrap();

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
