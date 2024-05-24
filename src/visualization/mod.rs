use std::path::Path;

use bevy::{
    app::{App, PreUpdate},
    asset::Handle,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        query::With,
        system::{Commands, Local, Query, ResMut},
    },
    input::{keyboard::KeyCode, ButtonInput},
    math::Vec3A,
    pbr::{DirectionalLight, DirectionalLightBundle, PbrBundle},
    prelude::{
        AssetPlugin,
        AssetServer,
        Assets,
        Color,
        Component,
        Cuboid,
        PluginGroup,
        Projection,
        Res,
        Resource,
        StandardMaterial,
        Startup,
        Transform,
        Update,
        Vec3,
        Visibility,
        Without,
    },
    render::{
        camera::{Camera, PerspectiveProjection},
        mesh::Mesh,
        primitives::Aabb as BevyAabb,
    },
    DefaultPlugins,
};
use scene_loader::{GltfSceneHandle, GltfSceneLoaderPlugin};
use tracing::{info, warn};

use self::camera_controller::CameraControllerPlugin;
use crate::{
    visualization::camera_controller::CameraController,
    voxelizer::{grid::ContextualVoxelGrid, voxel::VoxelData},
};


mod camera_controller;
mod scene_loader;


pub fn run_visualization(
    gltf_scene_file_path: &Path,
    voxelized_models: Vec<ContextualVoxelGrid>,
    voxel_size: f32,
) {
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: ".".to_string(),
                ..Default::default()
            }),
            CameraControllerPlugin,
            GltfSceneLoaderPlugin,
        ))
        .insert_resource(OriginalSceneInfo {
            scene_path: gltf_scene_file_path.to_string_lossy().to_string(),
        })
        .insert_resource(VoxelizedScene {
            voxelized_models,
            voxel_size,
            is_visible: false,
        })
        .add_systems(Startup, (set_up_scene, set_up_volume))
        .add_systems(PreUpdate, set_up_scene_after_load)
        .add_systems(
            Update,
            handle_user_input_for_volume_visibility_toggle,
        )
        .run();
}


#[derive(Resource)]
pub struct OriginalSceneInfo {
    scene_path: String,
}


#[derive(Resource)]
pub struct VoxelizedScene {
    pub voxelized_models: Vec<ContextualVoxelGrid>,
    pub voxel_size: f32,
    pub is_visible: bool,
}


fn parse_scene_path(scene_path: &str) -> (String, usize) {
    if scene_path.contains('#') {
        let gltf_and_scene = scene_path.split('#').collect::<Vec<_>>();
        if let Some((last, path)) = gltf_and_scene.split_last() {
            if let Some(index) = last
                .strip_prefix("Scene")
                .and_then(|index| index.parse::<usize>().ok())
            {
                return (path.join("#"), index);
            }
        }
    }

    (scene_path.to_string(), 0)
}

fn set_up_scene(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    original_mesh: Res<OriginalSceneInfo>,
) {
    let (file_path, scene_index) = parse_scene_path(&original_mesh.scene_path);

    commands.insert_resource(GltfSceneHandle::new(
        asset_server.load(file_path),
        scene_index,
    ));
}


#[derive(Component)]
pub struct VoxelMarker;

#[derive(Component)]
pub struct VoxelEdgeMarker;

#[derive(Component)]
pub struct VoxelInsideMeshMarker;


fn set_up_volume(
    mut commands: Commands,
    mut voxelized_scene: ResMut<VoxelizedScene>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_mesh_handle = meshes.add(Cuboid::from_size(Vec3::splat(
        voxelized_scene.voxel_size * 0.95,
    )));

    let box_inside_mesh_material = standard_materials.add(Color::SEA_GREEN);


    for voxelized_model in &voxelized_scene.voxelized_models {
        for contextual_voxel in voxelized_model.grid.contextual_voxels() {
            match contextual_voxel.data {
                VoxelData::Empty => {}
                VoxelData::Edge { base_color } => {
                    let voxel_position_in_world_space =
                        contextual_voxel.center_coordinate_in_world_space();

                    // DEBUGONLY
                    // println!(
                    //     "Got edge voxel at ({}, {}, {})",
                    //     voxel_position_in_world_space.x,
                    //     voxel_position_in_world_space.y,
                    //     voxel_position_in_world_space.z,
                    // );

                    let voxel_transform = Transform::from_translation(Vec3::new(
                        voxel_position_in_world_space.x,
                        voxel_position_in_world_space.y,
                        voxel_position_in_world_space.z,
                    ));


                    commands.spawn((
                        PbrBundle {
                            mesh: box_mesh_handle.clone(),
                            material: standard_materials.add(Color::rgb(
                                base_color.x,
                                base_color.y,
                                base_color.z,
                            )),
                            transform: voxel_transform,
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        },
                        VoxelMarker,
                        VoxelEdgeMarker,
                    ));
                }
                VoxelData::InsideMesh => {
                    let voxel_position_in_world_space =
                        contextual_voxel.center_coordinate_in_world_space();

                    let voxel_transform = Transform::from_translation(Vec3::new(
                        voxel_position_in_world_space.x,
                        voxel_position_in_world_space.y,
                        voxel_position_in_world_space.z,
                    ));


                    commands.spawn((
                        PbrBundle {
                            mesh: box_mesh_handle.clone(),
                            material: box_inside_mesh_material.clone(),
                            transform: voxel_transform,
                            visibility: Visibility::Hidden,
                            ..Default::default()
                        },
                        VoxelMarker,
                        VoxelInsideMeshMarker,
                    ));
                }
            }
        }
    }

    voxelized_scene.is_visible = false;
}


fn handle_user_input_for_volume_visibility_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut voxelized_mesh: ResMut<VoxelizedScene>,
    mut mesh_edge_voxels: Query<&mut Visibility, (With<VoxelMarker>, With<VoxelEdgeMarker>)>,
    mut meshes: Query<&mut Visibility, (With<Handle<Mesh>>, Without<VoxelMarker>)>,
) {
    if !key_input.just_pressed(KeyCode::KeyV) {
        return;
    }

    voxelized_mesh.is_visible = !voxelized_mesh.is_visible;
    info!("is_visible = {}", voxelized_mesh.is_visible);

    let updated_voxel_visibility = match voxelized_mesh.is_visible {
        true => Visibility::Visible,
        false => Visibility::Hidden,
    };

    let updated_mesh_visibility = match !voxelized_mesh.is_visible {
        true => Visibility::Visible,
        false => Visibility::Hidden,
    };

    info!("Toggling volume visibility (voxels now {updated_voxel_visibility:?}).");

    for mut voxel_visiblity in mesh_edge_voxels.iter_mut() {
        *voxel_visiblity = updated_voxel_visibility;
    }

    for mut mesh_visibility in meshes.iter_mut() {
        *mesh_visibility = updated_mesh_visibility;
    }
}


#[allow(clippy::type_complexity)]
fn set_up_scene_after_load(
    mut commands: Commands,
    mut scene_has_been_set_up: Local<bool>,
    mut scene_handle: ResMut<GltfSceneHandle>,
    meshes: Query<Option<&BevyAabb>, (With<Handle<Mesh>>, Without<VoxelMarker>)>,
) {
    if scene_handle.is_loaded && !*scene_has_been_set_up {
        *scene_has_been_set_up = true;

        println!("Scene is fully loaded, setting up view.");

        // Finds an approximate bounding box of the scene from its meshes.

        // If any of the meshes don't have an associated AABB, we skip that calculation.
        if meshes.iter().any(|maybe_aabb| maybe_aabb.is_none()) {
            warn!("At least one scene mesh does not have an AABB, skipping camera view setup.");
            return;
        }


        let mut coordinate_min = Vec3A::splat(f32::MAX);
        let mut coordinate_max = Vec3A::splat(f32::MIN);

        for potential_mesh_aabb in &meshes {
            // PANIC SAFETY: This cannot panic, because we checked all the meshes above.
            let mesh_aabb = potential_mesh_aabb.unwrap();

            coordinate_min = coordinate_min.min(mesh_aabb.min());
            coordinate_max = coordinate_max.max(mesh_aabb.max());
        }


        let scene_aabb_size = (coordinate_max - coordinate_min).length();
        let scene_aabb = BevyAabb::from_min_max(
            Vec3::from(coordinate_min),
            Vec3::from(coordinate_max),
        );


        let mut camera_projection = PerspectiveProjection::default();
        camera_projection.far = camera_projection.far.max(scene_aabb_size * 10.0);

        let camera_controller = CameraController::default();


        let camera_transform = Transform::from_translation(
            Vec3::from(scene_aabb.center) + scene_aabb_size * Vec3::new(1.8, 1.6, 1.8),
        )
        .looking_at(Vec3::from(scene_aabb.center), Vec3::Y);

        info!(
            "Spawning a controllable camera at ({}, {}, {}) \
            looking towards ({}, {}, {}).",
            camera_transform.translation.x,
            camera_transform.translation.y,
            camera_transform.translation.z,
            scene_aabb.center.x,
            scene_aabb.center.y,
            scene_aabb.center.z,
        );


        commands.spawn((
            Camera3dBundle {
                projection: Projection::Perspective(camera_projection),
                transform: camera_transform,
                camera: Camera {
                    is_active: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            camera_controller,
        ));

        commands.spawn(DirectionalLightBundle {
            directional_light: DirectionalLight {
                shadows_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        });

        // Spawn a default light if the scene does not have one.

        if !scene_handle.has_light {
            info!("Spawning a directional light, because the scene does not have one.");

            commands.spawn(DirectionalLightBundle {
                directional_light: DirectionalLight {
                    shadows_enabled: true,
                    ..Default::default()
                },
                transform: camera_transform.looking_at(Vec3::from(scene_aabb.center), Vec3::Y),
                ..Default::default()
            });

            scene_handle.has_light = true;
        }
    }
}
