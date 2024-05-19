use std::{path::Path, str::FromStr};

use bevy::{
    app::{App, PreUpdate}, asset::Handle, core_pipeline::core_3d::Camera3dBundle, ecs::{query::With, system::{Commands, Local, Query, ResMut}}, math::Vec3A, pbr::{CascadeShadowConfigBuilder, DirectionalLight, DirectionalLightBundle}, prelude::{AssetServer, Res, Resource, SceneBundle, Startup, Transform, Vec3}, render::{camera::{Camera, PerspectiveProjection}, mesh::Mesh, primitives::Sphere}, transform::components::GlobalTransform, DefaultPlugins
};
use clap::Parser;
use easy_gltf::model::Triangle;
use miette::{miette, Context, IntoDiagnostic, Result};
use nalgebra::Vector3;
use scene_viewer_plugin::SceneHandle;
use tracing::info;
use tracing_subscriber::EnvFilter;
use voxelizer::VoxelGrid;

use crate::{
    camera_controller::{CameraController, CameraControllerPlugin}, cli::CliArgs, logging::initialize_tracing, scene_viewer_plugin::SceneViewerPlugin, voxelizer::{voxelize_triangles, Aabb}
};

// #![allow(clippy::type_complexity)]

mod camera_controller;
mod cli;
mod logging;
mod voxelizer;
mod scene_viewer_plugin;


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


fn load_mesh_triangles_from_file<P>(gltf_file_path: P) -> Result<Vec<Triangle>>
where
    P: AsRef<Path>,
{
    let gltf_scenes = easy_gltf::load(gltf_file_path)
        .map_err(|error| miette!("Failed to load GLTF file: {error:?}"))?;

    let Some(first_scene) = gltf_scenes.first() else {
        return Err(miette!("Provided GLTF file contains no scenes."));
    };


    let mut collected_triangles: Vec<Triangle> = Vec::new();

    for model in &first_scene.models {
        let model_triangles: Vec<Triangle> = model
            .triangles()
            .map_err(|error| miette!("Failed to get triangles for model. Reason: {error:?}"))?;

        collected_triangles.extend(model_triangles);
    }

    Ok(collected_triangles)
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
        EnvFilter::from_str("error").unwrap(),
        "logs",
        "nrg-m2v",
    );

    /*
       let gltf_scene = load_gltf_scene_from_file(cli_args.input_file_path)
           .wrap_err("Failed to load GLTF scene.")?;

       let mesh_triangles =
           collect_mesh_triangles(&gltf_scene).wrap_err("Failed to collect triangles from scene.")?;

       println!("Loaded {} triangles.", mesh_triangles.len());


       // TODO run through voxelizer
       let voxel_grid = voxelize_triangles(
           mesh_triangles,
           Aabb {
               min: Vector3::new(-3.0, -3.0, -3.0),
               max: Vector3::new(3.0, 3.0, 3.0),
           },
           0.5,
       );
    */

    println!("Voxelization complete, starting visualization.");




    // TODO export to BVP format

    // TODO visualize?


    App::new()
        .add_plugins((DefaultPlugins, CameraControllerPlugin, SceneViewerPlugin))
        // .insert_resource(Scene { gltf_scene })
        // .insert_resource(VoxelizedMesh { voxel_grid })
        .add_systems(Startup, startup)
        .add_systems(PreUpdate, setup_scene_after_load)    
        .run();


    drop(logging_guard);
    Ok(())
}


#[derive(Resource)]
pub struct Scene {
    gltf_scene: easy_gltf::Scene,
}

#[derive(Resource)]
pub struct VoxelizedMesh {
    voxel_grid: VoxelGrid,
}

/*
fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut projection = PerspectiveProjection::default();
    projection.far = projection.far.max(20.0);

    let camera_controller = CameraController::default();

    info!("{}", camera_controller);

    commands.spawn((
        Camera3dBundle {
            projection: projection.into(),
            transform: Transform::from_xyz(4.0, 4.0, 4.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            camera: Camera {
                is_active: false,
                ..Default::default()
            },
            ..Default::default()
        },
        camera_controller,
    ));

    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(4.0, 4.0, 4.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    //     ..Default::default()
    // });


    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });


    commands.spawn(SceneBundle {
        scene: asset_server.load("mesh-sample-02/mesh-sample-02_simple-cube.glb#Scene0"),
        ..Default::default()
    });
}
*/

fn parse_scene(scene_path: String) -> (String, usize) {
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
    (scene_path, 0)
}


fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let (file_path, scene_index) = parse_scene("mesh-sample-01/mesh-sample-01_torus.glb".to_string());

    commands.insert_resource(SceneHandle::new(asset_server.load(file_path), scene_index));
} 


fn setup_scene_after_load(
    mut commands: Commands,
    mut setup: Local<bool>,
    mut scene_handle: ResMut<SceneHandle>,
    meshes: Query<(&GlobalTransform, Option<&bevy::render::primitives::Aabb>), With<Handle<Mesh>>>,
) {
    if scene_handle.is_loaded && !*setup {
        println!("Scene loaded, running setup!");

        *setup = true;
        // Find an approximate bounding box of the scene from its meshes
        if meshes.iter().any(|(_, maybe_aabb)| maybe_aabb.is_none()) {
            return;
        }

        let mut min = Vec3A::splat(f32::MAX);
        let mut max = Vec3A::splat(f32::MIN);
        for (transform, maybe_aabb) in &meshes {
            let aabb = maybe_aabb.unwrap();
            // If the Aabb had not been rotated, applying the non-uniform scale would produce the
            // correct bounds. However, it could very well be rotated and so we first convert to
            // a Sphere, and then back to an Aabb to find the conservative min and max points.
            let sphere = Sphere {
                center: Vec3A::from(transform.transform_point(Vec3::from(aabb.center))),
                radius: transform.radius_vec3a(aabb.half_extents),
            };
            let aabb = bevy::render::primitives::Aabb::from(sphere);
            min = min.min(aabb.min());
            max = max.max(aabb.max());
        }

        let size = (max - min).length();
        let aabb = bevy::render::primitives::Aabb::from_min_max(Vec3::from(min), Vec3::from(max));

        info!("Spawning a controllable 3D perspective camera");
        let mut projection = PerspectiveProjection::default();
        projection.far = projection.far.max(size * 10.0);

        let camera_controller = CameraController::default();

        // Display the controls of the scene viewer
        info!("{}", camera_controller);
        info!("{}", *scene_handle);

        commands.spawn((
            Camera3dBundle {
                projection: projection.into(),
                transform: Transform::from_translation(
                    Vec3::from(aabb.center) + size * Vec3::new(0.5, 0.25, 0.5),
                )
                .looking_at(Vec3::from(aabb.center), Vec3::Y),
                camera: Camera {
                    is_active: false,
                    ..Default::default()
                },
                ..Default::default()
            },
            camera_controller,
        ));

        // Spawn a default light if the scene does not have one
        if !scene_handle.has_light {
            info!("Spawning a directional light");
            commands.spawn(DirectionalLightBundle {
                transform: Transform::from_xyz(4.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..Default::default()
            });

            scene_handle.has_light = true;
        }
    }
}
