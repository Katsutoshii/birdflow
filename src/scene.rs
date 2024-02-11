use std::fs::File;
use std::io::Write;

use bevy::{prelude::*, tasks::IoTaskPool, utils::HashMap};

use crate::{
    grid::ObstaclesSpec,
    objects::{InteractionConfig, ObjectiveConfig},
    physics::PhysicsMaterials,
    prelude::*,
};

/// Plugin for saving and loading scenes.
pub struct LoadableScenePlugin;
impl Plugin for LoadableScenePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SaveEntity>()
            .register_type::<Name>()
            .register_type::<core::num::NonZeroU16>()
            .add_systems(PreStartup, load_system)
            .add_systems(FixedUpdate, save_system)
            .insert_resource(SceneSpec);
    }
}

/// Use this to tag entities that should be saved in the scene.
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct SceneSpec;

/// Use this to tag entities that should be saved in the scene.
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct SaveEntity;

// The initial scene file will be loaded below and not change when the scene is saved
const SCENE_FILE_PATH: &str = "test.scn.ron";

// The new, updated scene data will be saved here so that you can see the changes
const NEW_SCENE_FILE_PATH: &str = "test-new.scn.ron";

pub fn load_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // "Spawning" a scene bundle creates a new entity and spawns new instances
    // of the given scene's entities as children of that entity.
    // commands.spawn((
    //     DynamicSceneBundle {
    //         // Scenes are loaded just like any other asset.
    //         scene: asset_server.load(SCENE_FILE_PATH),
    //         ..default()
    //     },
    //     Name::new("DynamicScene"),
    // ));
    commands.insert_resource(PhysicsMaterials(hashmap! {
            PhysicsMaterialType::Default => PhysicsMaterial {
                max_velocity: 10.0,
                min_velocity: 3.0,
                velocity_smoothing: 0.0,
            },
            PhysicsMaterialType::Zooid => PhysicsMaterial{
                max_velocity: 7.0,
                min_velocity: 3.0,
                velocity_smoothing: 0.5,
            },
            PhysicsMaterialType::SlowZooid => PhysicsMaterial{
                max_velocity: 5.0,
                min_velocity: 0.0,
                velocity_smoothing: 0.5,
            },
            PhysicsMaterialType::Food => PhysicsMaterial{
                max_velocity: 1.0,
                min_velocity: 0.0,
                velocity_smoothing: 0.5,
            },
    }));
    commands.insert_resource(GridSpec {
        rows: 256,
        cols: 256,
        width: 64.0,
        visualize: false,
    });
    commands.insert_resource(ObstaclesSpec(Vec::default()));
    commands.insert_resource(Configs {
        window_size: Vec2 { x: 1600., y: 900. },
        player_team: Team::Blue,
        visibility_radius: 6,
        fog_radius: 5,
        worker: Config {
            physics_material: PhysicsMaterialType::Zooid,
            neighbor_radius: 300.0,
            spawn_velocity: 10.0,
            hit_radius: 12.0,
            death_speed: 5.,
            waypoint: ObjectiveConfig {
                max_acceleration: 3.5,
                repell_radius: 20.0,
                slow_factor: 0.0,
                attack_radius: 265.0,
            },
            worker: InteractionConfig {
                separation_radius: 100.0,
                separation_acceleration: 20.0,
                cohesion_acceleration: 2.0,
                alignment_factor: 10000.0,
                ..default()
            },
            head: InteractionConfig {
                separation_radius: 100.0,
                separation_acceleration: 0.7,
                cohesion_acceleration: 0.5,
                alignment_factor: 0.0,
                slow_factor: 0.1,
                ..default()
            },
            food: InteractionConfig {
                separation_radius: 10.0,
                separation_acceleration: 0.1,
                cohesion_acceleration: 0.1,
                alignment_factor: 0.0,
                chase: true,
                ..default()
            },
            ..default()
        },
        head: Config {
            physics_material: PhysicsMaterialType::SlowZooid,
            neighbor_radius: 100.0,
            spawn_velocity: 20.0,
            waypoint: ObjectiveConfig {
                max_acceleration: 3.5,
                repell_radius: 20.0,
                slow_factor: 0.0,
                ..default()
            },
            worker: InteractionConfig {
                separation_radius: 40.0,
                separation_acceleration: 0.2,
                cohesion_acceleration: 0.1,
                alignment_factor: 0.0,
                ..default()
            },
            head: InteractionConfig {
                separation_radius: 100.0,
                separation_acceleration: 0.5,
                cohesion_acceleration: 0.1,
                alignment_factor: 0.0,
                ..default()
            },
            ..default()
        },
        food: Config {
            physics_material: PhysicsMaterialType::Food,
            neighbor_radius: 128.0,
            worker: InteractionConfig {
                separation_radius: 100.0,
                separation_acceleration: 0.05,
                ..default()
            },
            food: InteractionConfig {
                separation_radius: 20.0,
                separation_acceleration: 1.2,
                cohesion_acceleration: 0.00,
                alignment_factor: 1000.0,
                ..default()
            },
            ..default()
        },
        ..default()
    });
}

pub fn save_system(
    world: &World,
    query: Query<Entity, With<SaveEntity>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if !keyboard_input.just_pressed(KeyCode::S) {
        return;
    }
    let scene = DynamicSceneBuilder::from_world(world)
        .extract_entities(query.iter())
        .allow_resource::<Config>()
        .allow_resource::<Grid2<EntitySet>>()
        .extract_resources()
        .build();

    // Scenes can be serialized like this:
    let type_registry = world.resource::<AppTypeRegistry>();
    let serialized_scene = scene.serialize_ron(type_registry).unwrap();
    // Showing the scene in the console
    info!("Saving scene: {}", serialized_scene);

    // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
    // as they are blocking
    // This can't work in WASM as there is no filesystem access
    #[cfg(not(target_arch = "wasm32"))]
    IoTaskPool::get()
        .spawn(async move {
            // Write the scene RON data to file
            File::create(format!("assets/{NEW_SCENE_FILE_PATH}"))
                .and_then(|mut file| file.write(serialized_scene.as_bytes()))
                .expect("Error while writing scene to file");
        })
        .detach();
}
