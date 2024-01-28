use std::f32::consts::PI;

use crate::{grid::NavigationCostEvent, prelude::*};
use bevy::{prelude::*, sprite::MaterialMesh2dBundle, utils::hashbrown::HashSet};

/// Plugin to add a waypoint system where the player can click to create a waypoint.
pub struct WaypointPlugin;
impl Plugin for WaypointPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaypointAssets>().add_systems(
            FixedUpdate,
            (
                Waypoint::update.in_set(SystemStage::Compute),
                Waypoint::cleanup.in_set(SystemStage::PostApply),
            ),
        );
    }
}

#[derive(Component, Debug)]
pub struct Waypoint {
    pub active: bool,
    pub size: f32,
}
impl Default for Waypoint {
    fn default() -> Self {
        Self {
            active: false,
            size: 10.0,
        }
    }
}
impl Waypoint {
    pub fn cleanup(
        objectives: Query<&Objective, Without<Waypoint>>,
        waypoints: Query<Entity, With<Waypoint>>,
        mut commands: Commands,
        mut input_actions: EventReader<ControlEvent>,
    ) {
        for &ControlEvent {
            action,
            state: _,
            position: _,
        } in input_actions.read()
        {
            if action != ControlAction::Move {
                continue;
            }

            let mut followed_entities = HashSet::new();
            for objective in objectives.iter() {
                if let Objective::FollowEntity(entity) = objective {
                    followed_entities.insert(entity);
                }
            }
            for waypoint_entity in waypoints.iter() {
                if !followed_entities.contains(&waypoint_entity) {
                    commands.entity(waypoint_entity).despawn();
                }
            }
        }
    }

    pub fn update(
        mut control_events: EventReader<ControlEvent>,
        mut selection: Query<(&Selected, &mut Objective, &Transform), Without<Self>>,
        mut nav_grid: ResMut<Grid2<EntityFlow>>,
        obstacles: Res<Grid2<Obstacle>>,
        mut event_writer: EventWriter<NavigationCostEvent>,
        mut commands: Commands,
        assets: Res<WaypointAssets>,
    ) {
        for &ControlEvent {
            action,
            state: _,
            position,
        } in control_events.read()
        {
            if action != ControlAction::Move {
                return;
            }

            // Spawn a new waypoint.
            let waypoint_bundle =
                Waypoint::default().bundle(&assets, position.extend(zindex::WAYPOINT));
            let waypoint_entity = commands.spawn(waypoint_bundle).id();

            let mut positions = Vec::new();
            for (selected, mut objective, transform) in selection.iter_mut() {
                if selected.is_selected() {
                    *objective = Objective::FollowEntity(waypoint_entity);
                    let rowcol = nav_grid.spec.to_rowcol(transform.translation.xy());
                    for neighbor_rowcol in nav_grid.get_in_radius_discrete(rowcol, 2) {
                        positions.push(neighbor_rowcol);
                    }
                }
            }
            let target = nav_grid.spec.to_rowcol(position);
            nav_grid.add_waypoint(
                waypoint_entity,
                target,
                &positions,
                obstacles.as_ref(),
                &mut event_writer,
            );
        }
    }

    pub fn bundle(self, assets: &WaypointAssets, translation: Vec3) -> impl Bundle {
        (
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(self.size).extend(1.))
                    .with_rotation(Quat::from_axis_angle(Vec3::Z, PI))
                    .with_translation(translation),
                material: assets.blue_material.clone(),
                ..default()
            },
            self,
        )
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct WaypointAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<ColorMaterial>,
}
impl FromWorld for WaypointAssets {
    fn from_world(world: &mut World) -> Self {
        let mesh = {
            let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
            meshes.add(Mesh::from(shape::RegularPolygon {
                radius: 2f32.sqrt() / 2.,
                sides: 3,
            }))
        };
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            blue_material: materials.add(ColorMaterial::from(Color::TURQUOISE.with_a(0.5))),
        }
    }
}
