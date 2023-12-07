use bevy::{prelude::*, sprite::MaterialMesh2dBundle, window::PrimaryWindow};

use crate::{
    camera,
    grid::EntityGrid,
    objects::{Object, ZooidAssets},
    zindex, Aabb2,
};

#[derive(Component, Default, PartialEq, Clone)]
pub enum Selected {
    #[default]
    Unselected,
    Selected {
        previous_material: Handle<ColorMaterial>,
    },
}

/// Plugin for an spacial entity paritioning grid with optional debug functionality.
pub struct SelectorPlugin;
impl Plugin for SelectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectorAssets>()
            .add_systems(Startup, Selector::startup)
            .add_systems(FixedUpdate, Selector::update);
    }
}

#[derive(Component, Default)]
pub struct Selector {
    pub active: bool,
    pub aabb: Aabb2,
}
impl Selector {
    pub fn startup(mut commands: Commands, assets: Res<SelectorAssets>) {
        commands.spawn(Self::default().bundle(&assets));
    }

    pub fn update(
        mut query: Query<(&mut Self, &mut Transform, &mut Visibility)>,
        camera_query: Query<(Entity, &Camera, &GlobalTransform), With<camera::MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mouse_input: Res<Input<MouseButton>>,
        mut objects: Query<
            (
                &Object,
                &Transform,
                &mut Selected,
                &mut Handle<ColorMaterial>,
            ),
            Without<Self>,
        >,
        grid: Res<EntityGrid>,
        assets: Res<ZooidAssets>,
    ) {
        let (_entity, camera, camera_transform) = camera_query.single();
        let (mut selector, mut transform, mut visibility) = query.single_mut();

        if let Some(position) = window_query
            .single()
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
        {
            if mouse_input.just_pressed(MouseButton::Left) {
                // Reset other selections.
                for (_object, _transform, mut selected, mut material) in &mut objects {
                    if let Selected::Selected { previous_material } = selected.as_ref() {
                        *material = previous_material.clone();
                        // TODO check previous selected status
                    }
                    *selected = Selected::Unselected;
                }

                selector.aabb.min = position;
                selector.aabb.max = position;

                *visibility = Visibility::Visible;
                transform.scale = Vec3::ZERO;
                transform.translation = position.extend(zindex::SELECTOR);
            } else if mouse_input.pressed(MouseButton::Left) {
                selector.aabb.max = position;
                // Resize the square to match the bounding box.
                transform.translation = selector.aabb.center().extend(zindex::SELECTOR);
                transform.scale = selector.aabb.size().extend(0.0);

                // Correct the bounding box before we check entity collision, since it might be backwards.
                let mut aabb = selector.aabb.clone();
                aabb.enforce_minmax();
                // Check the grid for entities in this bounding box.
                for entity in grid.get_in_aabb(&aabb) {
                    let (_object, transform, mut selected, mut material) =
                        objects.get_mut(entity).unwrap();
                    if aabb.contains(transform.translation.xy()) {
                        *selected = Selected::Selected {
                            previous_material: material.clone(),
                        };
                        *material = assets.white_material.clone();
                    }
                }
            } else if mouse_input.just_released(MouseButton::Left) {
                *visibility = Visibility::Hidden;
            }
        }
    }

    fn bundle(self, assets: &SelectorAssets) -> impl Bundle {
        (
            self,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default().with_scale(Vec3::splat(1.0)),
                material: assets.blue_material.clone(),
                visibility: Visibility::Hidden,
                ..default()
            },
        )
    }
}

/// Handles to common grid assets.
#[derive(Resource)]
pub struct SelectorAssets {
    pub mesh: Handle<Mesh>,
    pub blue_material: Handle<ColorMaterial>,
}

impl FromWorld for SelectorAssets {
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
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        Self {
            mesh,
            blue_material: materials.add(ColorMaterial::from(Color::BLUE.with_a(0.04))),
        }
    }
}
