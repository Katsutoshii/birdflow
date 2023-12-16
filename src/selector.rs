use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::PrimaryWindow,
};

use crate::{
    grid::EntityGrid,
    objects::{Configs, Object, Team},
    prelude::*,
    zindex, Aabb2,
};

#[derive(Component, Default, PartialEq, Clone)]
pub enum Selected {
    #[default]
    Unselected,
    Selected {
        child_entity: Entity,
    },
}
impl Selected {
    pub fn is_selected(&self) -> bool {
        self != &Self::Unselected
    }
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

    #[allow(clippy::too_many_arguments)]
    pub fn update(
        mut commands: Commands,
        mut query: Query<(&mut Self, &mut Transform, &mut Visibility)>,
        camera_query: Query<(Entity, &Camera, &GlobalTransform), With<MainCamera>>,
        window_query: Query<&Window, With<PrimaryWindow>>,
        mouse_input: Res<Input<MouseButton>>,
        mut objects: Query<
            (&Object, &Transform, &Team, &mut Selected, &Mesh2dHandle),
            Without<Self>,
        >,
        grid: Res<EntityGrid>,
        assets: Res<SelectorAssets>,
        configs: Res<Configs>,
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
                for (_object, _transform, _team, mut selected, _mesh) in &mut objects {
                    if let Selected::Selected { child_entity } = selected.as_ref() {
                        commands.entity(*child_entity).despawn()
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
                for entity in grid.get_entities_in_aabb(&aabb) {
                    let (_object, transform, team, mut selected, mesh) =
                        objects.get_mut(entity).unwrap();
                    if aabb.contains(transform.translation.xy()) {
                        if selected.is_selected() || *team != configs.player_team {
                            continue;
                        }
                        let child_entity = commands
                            .spawn(Self::highlight_bundle(&assets, mesh.0.clone()))
                            .id();
                        commands.entity(entity).add_child(child_entity);
                        *selected = Selected::Selected { child_entity };
                    }
                }
            } else if mouse_input.just_released(MouseButton::Left) {
                *visibility = Visibility::Hidden;
            }
        }
    }

    fn highlight_bundle(assets: &SelectorAssets, mesh: Handle<Mesh>) -> impl Bundle {
        MaterialMesh2dBundle::<ColorMaterial> {
            mesh: mesh.clone().into(),
            transform: Transform::default()
                .with_scale(Vec2::splat(1.).extend(1.))
                .with_translation(Vec3 {
                    x: 0.0,
                    y: 0.0,
                    z: zindex::HIGHLIGHT,
                }),
            material: assets.white_material.clone(),
            visibility: Visibility::Visible,
            ..default()
        }
    }

    fn bundle(self, assets: &SelectorAssets) -> impl Bundle {
        (
            self,
            MaterialMesh2dBundle::<ColorMaterial> {
                mesh: assets.mesh.clone().into(),
                transform: Transform::default().with_scale(Vec2::splat(1.).extend(1.)),
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
    pub white_material: Handle<ColorMaterial>,
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
            white_material: materials.add(ColorMaterial::from(Color::ALICE_BLUE.with_a(0.15))),
        }
    }
}
