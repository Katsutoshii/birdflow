use std::f32::consts::PI;

use bevy::prelude::*;

mod bird;
use bird::{Bird, BirdBundler};

/// Used to help identify our main camera
#[derive(Component)]
pub struct MainCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, Bird::update)
        .run();
}

const THETA_FACTOR: f32 = 0.001;
const TRANSLATION_FACTOR: f32 = 10.0;
const NUM_BIRDS: usize = 40;

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Circle::default()));
    let green_material = materials.add(ColorMaterial::from(Color::LIME_GREEN));
    let tomato_material = materials.add(ColorMaterial::from(Color::TOMATO));

    commands.spawn((Camera2dBundle::default(), MainCamera));

    for i in 1..(NUM_BIRDS / 2) {
        commands.spawn(
            BirdBundler {
                bird: Bird {
                    theta: PI * THETA_FACTOR * (i as f32),
                    ..default()
                },
                mesh: mesh.clone(),
                material: green_material.clone(),
                translation: Vec3::ONE * TRANSLATION_FACTOR * (i as f32),
            }
            .bundle(),
        );
        commands.spawn(
            BirdBundler {
                bird: Bird {
                    theta: PI * THETA_FACTOR * (i as f32),
                    ..default()
                },
                mesh: mesh.clone(),
                material: tomato_material.clone(),
                translation: Vec3::NEG_ONE * TRANSLATION_FACTOR * (i as f32),
            }
            .bundle(),
        );
    }
}
