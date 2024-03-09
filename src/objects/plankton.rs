use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::prelude::*;

use super::ZooidAssets;

pub struct PlanktonPlugin;
impl Plugin for PlanktonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                Plankton::spawn.in_set(SystemStage::Spawn),
                PlanktonBackground::update.in_set(SystemStage::Compute),
            ),
        );
    }
}

#[derive(Component, Default)]
pub struct Plankton;
impl Plankton {
    pub fn spawn(
        mut commands: Commands,
        assets: Res<ZooidAssets>,
        // grid_spec: Res<GridSpec>,
        mut control_events: EventReader<ControlEvent>,
    ) {
        for control_event in control_events.read() {
            if control_event.is_pressed(ControlAction::SpawnPlankton) {
                info!("Is pressed");
                commands
                    .spawn(Plankton.bundle(control_event.position, &assets))
                    .with_children(|parent| {
                        parent.spawn(PlanktonBackground.bundle(&assets));
                    });
                // Old code to spawn lots of food.
                // for row in 0..20 {
                //     for col in 0..20 {
                //         commands
                //             .spawn(Food { period_sec: 1.0 }.bundle(
                //                 Vec2 {
                //                     x: (0.5 + row as f32),
                //                     y: (0.5 + col as f32),
                //                 } * grid_spec.width
                //                     - Vec2 { x: 10., y: 10. } * grid_spec.width,
                //                 &assets,
                //             ))
                //             .with_children(|parent| {
                //                 parent.spawn(FoodBackground.bundle(&assets));
                //             });
                //     }
                // }
            }
        }
    }

    pub fn bundle(self, position: Vec2, assets: &ZooidAssets) -> impl Bundle {
        (
            self,
            Object::Plankton,
            Team::None,
            GridEntity::default(),
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(10.0).extend(1.))
                    .with_translation(position.extend(zindex::PLANKTON)),
                material: assets.get_team_material(Team::None).primary,
                ..default()
            },
            PhysicsBundle {
                material: PhysicsMaterialType::Plankton,
                velocity: Velocity(Vec2::ONE),
                ..default()
            },
            Objectives::default(),
            Health::new(1),
            Selected::default(),
            Name::new("Plankton"),
        )
    }
}

#[derive(Component, Default)]
pub struct PlanktonBackground;
impl PlanktonBackground {
    pub fn bundle(self, assets: &ZooidAssets) -> impl Bundle {
        (
            self,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(1.5).extend(1.))
                    .with_translation(Vec3 {
                        x: 0.0,
                        y: 0.0,
                        z: zindex::PLANKTON_BACKGROUND,
                    }),
                material: assets.get_team_material(Team::None).background,
                ..default()
            },
        )
    }
    pub fn update(
        mut query: Query<(&mut Transform, &Parent), With<Self>>,
        parent_velocities: Query<&Velocity, With<Children>>,
    ) {
        for (mut transform, parent) in &mut query {
            let parent_velocity = parent_velocities
                .get(parent.get())
                .expect("Invalid parent.");
            transform.translation = -0.1 * parent_velocity.extend(0.);
        }
    }
}
