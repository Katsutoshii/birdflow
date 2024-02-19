use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::prelude::*;

use super::ZooidAssets;

pub struct FoodPlugin;
impl Plugin for FoodPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                Food::update.in_set(SystemStage::PreCompute),
                Food::spawn.in_set(SystemStage::Spawn),
                FoodBackground::update.in_set(SystemStage::Compute),
            ),
        );
    }
}

#[derive(Component, Default)]
pub struct Food {
    period_sec: f32,
}
impl Food {
    pub fn spawn(
        mut commands: Commands,
        assets: Res<ZooidAssets>,
        grid_spec: Res<GridSpec>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {
        if !keyboard_input.just_pressed(KeyCode::F) {
            return;
        }
        println!("Spawn food");
        for row in 0..2 {
            for col in 0..2 {
                commands
                    .spawn(Food { period_sec: 1.0 }.bundle(
                        Vec2 {
                            x: (0.5 + row as f32),
                            y: (0.5 + col as f32),
                        } * grid_spec.width
                            - Vec2 { x: 10., y: 10. } * grid_spec.width,
                        &assets,
                    ))
                    .with_children(|parent| {
                        parent.spawn(FoodBackground.bundle(&assets));
                    });
            }
        }
    }

    pub fn bundle(self, position: Vec2, assets: &ZooidAssets) -> impl Bundle {
        (
            self,
            Object::Food,
            Team::default(),
            GridEntity::default(),
            PhysicsBundle::default(),
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default()
                    .with_scale(Vec2::splat(10.0).extend(1.))
                    .with_translation(position.extend(zindex::FOOD)),
                material: assets.get_team_material(Team::None).primary,
                ..default()
            },
            Selected::default(),
            Name::new("Zooid"),
        )
    }

    pub fn update(time: Res<Time>, mut query: Query<(&Self, &mut Acceleration)>) {
        for (food, mut new_velocity) in &mut query {
            let (x, y) = (time.elapsed_seconds() * food.period_sec).sin_cos();
            new_velocity.0 += 0.01 * Vec2 { x, y }
        }
    }
}

#[derive(Component, Default)]
pub struct FoodBackground;
impl FoodBackground {
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
                        z: zindex::FOOD_BACKGROUND,
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
