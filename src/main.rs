use std::{
    ops::{Add, Sub},
    path::Path,
    str::FromStr,
    time::Instant,
};

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
use clap::Parser;
use easy_gltf::model::Triangle;
use glam::Vec3 as GlamVec3;
use miette::{miette, Context, IntoDiagnostic, Result};
use scene_loader::{GltfSceneHandle, GltfSceneLoaderPlugin};
use tracing::{info, trace, warn};
use tracing_subscriber::EnvFilter;
use voxelizer::VoxelGrid;

use crate::{
    camera_controller::{CameraController, CameraControllerPlugin},
    cli::CliArgs,
    logging::initialize_tracing,
    voxelizer::{voxelize_triangles, Aabb},
};


mod camera_controller;
mod cli;
mod logging;
mod scene_loader;
mod voxelizer;


fn load_gltf_scene_from_file<P>(gltf_file_path: P) -> Result<easy_gltf::Scene>
where
    P: AsRef<Path>,
{
    let gltf_scenes = easy_gltf::load(gltf_file_path)
        .map_err(|error| miette!("Failed to load GLTF file: {error:?}"))?;

    let Some(first_scene) = gltf_scenes.into_iter().next() else {
        return Err(miette!("Provided GLTF file contains no scenes."));
    };

    Ok(first_scene)
}

fn collect_mesh_triangles(gltf_scene: &easy_gltf::Scene) -> Result<Vec<Triangle>> {
    let mut collected_triangles: Vec<Triangle> = Vec::new();

    for model in &gltf_scene.models {
        let model_triangles: Vec<Triangle> = model
            .triangles()
            .map_err(|error| miette!("Failed to get triangles for model. Reason: {error:?}"))?;

        collected_triangles.extend(model_triangles);
    }

    Ok(collected_triangles)
}


fn compute_minimum_aabb_for_mesh(mesh_triangles: &[Triangle], padding: f32) -> Aabb {
    let mut current_minimum = GlamVec3::MAX;
    let mut current_maximum = GlamVec3::MIN;

    for triangle in mesh_triangles {
        for vertex in triangle {
            current_minimum.x = current_minimum.x.min(vertex.position.x);
            current_minimum.y = current_minimum.y.min(vertex.position.y);
            current_minimum.z = current_minimum.z.min(vertex.position.z);

            current_maximum.x = current_maximum.x.max(vertex.position.x);
            current_maximum.y = current_maximum.y.max(vertex.position.y);
            current_maximum.z = current_maximum.z.max(vertex.position.z);
        }
    }

    Aabb::from_min_and_max(
        current_minimum.sub(padding),
        current_maximum.add(padding),
    )
}



fn main() -> Result<()> {
    let cli_args = CliArgs::parse();


    let console_logging_output_level_filter = {
        if let Some(output_filter) = cli_args.console_logging_output_level_filter {
            EnvFilter::from_str(&output_filter)
                .into_diagnostic()
                .wrap_err("invalid console-logging-level option")?
        } else {
            EnvFilter::from_default_env()
        }
    };

    let logging_guard = initialize_tracing(
        console_logging_output_level_filter,
        EnvFilter::from_str("warn").unwrap(),
        "logs",
        "nrg-m2v",
    );


    let gltf_scene = load_gltf_scene_from_file(&cli_args.input_file_path)
        .wrap_err("Failed to load GLTF scene.")?;

    let mesh_triangles =
        collect_mesh_triangles(&gltf_scene).wrap_err("Failed to collect triangles from scene.")?;

    println!("Loaded {} triangles.", mesh_triangles.len());

    let minimum_voxelization_aabb = compute_minimum_aabb_for_mesh(&mesh_triangles, 1.0);

    println!(
        "Minimum AABB for voxelization is from ({}, {}, {}) to ({}, {}, {})",
        minimum_voxelization_aabb.min.x,
        minimum_voxelization_aabb.min.y,
        minimum_voxelization_aabb.min.z,
        minimum_voxelization_aabb.max.x,
        minimum_voxelization_aabb.max.y,
        minimum_voxelization_aabb.max.z,
    );


    let time_voxelization_start = Instant::now();

    let voxel_grid = voxelize_triangles(
        mesh_triangles,
        minimum_voxelization_aabb,
        cli_args.voxel_size,
    );

    let time_voxelization_total = time_voxelization_start.elapsed();


    println!(
        "Voxelization complete in {:.1} seconds, starting visualization.",
        time_voxelization_total.as_secs_f32()
    );




    // TODO export to BVP format

    // TODO visualize?


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
            scene_path: cli_args.input_file_path.to_string_lossy().to_string(),
        })
        .insert_resource(VoxelizedMesh {
            voxel_grid,
            is_visible: false,
        })
        .add_systems(Startup, (set_up_scene, set_up_volume))
        .add_systems(PreUpdate, set_up_scene_after_load)
        .add_systems(
            Update,
            handle_user_input_for_volume_visibility_toggle,
        )
        .run();


    drop(logging_guard);
    Ok(())
}


#[derive(Resource)]
pub struct OriginalSceneInfo {
    scene_path: String,
}


#[derive(Resource)]
pub struct VoxelizedMesh {
    pub voxel_grid: VoxelGrid,
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


fn set_up_volume(
    mut commands: Commands,
    mut voxelized_mesh: ResMut<VoxelizedMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_mesh_handle = meshes.add(Cuboid::from_size(Vec3::splat(
        (voxelized_mesh.voxel_grid.voxel_half_extent * 2.0) * 0.95,
    )));


    let box_mesh_material = standard_materials.add(Color::GRAY);

    for voxel in voxelized_mesh.voxel_grid.voxels() {
        if !voxel.is_filled {
            continue;
        }


        let voxel_position_in_world_space = voxel.center_coordinate_in_world_space(
            &voxelized_mesh.voxel_grid.starting_point,
            voxelized_mesh.voxel_grid.voxel_half_extent,
        );

        let voxel_transform = Transform::from_translation(Vec3::new(
            voxel_position_in_world_space.x,
            voxel_position_in_world_space.y,
            voxel_position_in_world_space.z,
        ));

        trace!(
            "Generating voxel at ({}, {}, {}).",
            voxel_position_in_world_space.x,
            voxel_position_in_world_space.y,
            voxel_position_in_world_space.z,
        );


        commands.spawn((
            PbrBundle {
                mesh: box_mesh_handle.clone(),
                material: box_mesh_material.clone(),
                transform: voxel_transform,
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            VoxelMarker,
        ));
    }

    voxelized_mesh.is_visible = false;
}


fn handle_user_input_for_volume_visibility_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut voxelized_mesh: ResMut<VoxelizedMesh>,
    mut voxels: Query<&mut Visibility, With<VoxelMarker>>,
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

    for mut voxel_visiblity in voxels.iter_mut() {
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
