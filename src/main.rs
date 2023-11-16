use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

#[derive(Component)]
struct Bird {
    velocity: Vec2,
}
/// Used to help identify our main camera
#[derive(Component)]
struct MainCamera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, bird_update)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let circle_size = 10.0;
    commands.spawn((Camera2dBundle::default(), MainCamera));
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Circle::default())).into(),
            transform: Transform::default().with_scale(Vec3::splat(circle_size)),
            material: materials.add(ColorMaterial::from(Color::LIME_GREEN)),
            ..default()
        },
        Bird {
            velocity: Vec2::ZERO,
        },
    ));

    for i in 1..20 {
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Circle::default())).into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(circle_size))
                    .with_translation(Vec3::ONE * 10.0 * (i as f32)),
                material: materials.add(ColorMaterial::from(Color::YELLOW_GREEN)),
                ..default()
            },
            Bird {
                velocity: Vec2::ZERO,
            },
        ));
        commands.spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(shape::Circle::default())).into(),
                transform: Transform::default()
                    .with_scale(Vec3::splat(circle_size))
                    .with_translation(Vec3::NEG_ONE * 10.0 * (i as f32)),
                material: materials.add(ColorMaterial::from(Color::TOMATO)),
                ..default()
            },
            Bird {
                velocity: Vec2::ZERO,
            },
        ));
    }
}

fn bird_update(
    q_windows: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut sprite_position: Query<(&mut Bird, &mut Transform)>,
) {
    let (camera, camera_transform) = q_camera.single();
    for (mut bird, mut transform) in &mut sprite_position {
        if let Some(position) = q_windows
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        {
            let mut delta = (position - transform.translation.xy()).extend(0.0);
            if delta.length() < 50.0 {
                delta = -50.0 / delta.clamp_length_min(0.1);
            }
            bird.velocity += delta.normalize_or_zero().truncate() * 0.3;
        }
        bird.velocity = bird.velocity.clamp_length_max(10.0);
        transform.translation += bird.velocity.extend(0.0);
    }
}
