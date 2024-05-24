use std::{path::Path, str::FromStr, time::Instant};

use clap::Parser;
use cli::CliCommand;
use easy_gltf::model::Triangle;
use glam::Vec3 as GlamVec3;
use miette::{miette, Context, IntoDiagnostic, Result};
use tracing_subscriber::EnvFilter;
use voxelizer::grid::ContextualVoxelGrid;

use crate::{
    cli::CliArgs,
    exporter::export_voxel_grid_as_raw,
    logging::initialize_tracing,
    voxelizer::voxelize_models,
};


mod cli;
mod exporter;
mod logging;

#[cfg(feature = "visualization")]
mod visualization;
#[cfg(feature = "visualization")]
use crate::visualization::run_visualization;

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


#[deprecated]
#[allow(dead_code)]
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



fn perform_voxelization(cli_args: &CliArgs) -> Result<Vec<ContextualVoxelGrid>> {
    let voxelization_bounds = cli_args
        .voxelization_bounds()
        .wrap_err("Invalid voxelization bounds.")?
        .unwrap_or(voxelizer::aabb::Aabb {
            min: GlamVec3::MIN,
            max: GlamVec3::MAX,
        });




    let gltf_scene = load_gltf_scene_from_file(&cli_args.gltf_file_path)
        .wrap_err("Failed to load GLTF scene.")?;


    let time_voxelization_start = Instant::now();

    let voxelized_models = voxelize_models(
        &gltf_scene.models,
        voxelization_bounds,
        cli_args.voxel_size,
    );

    let time_voxelization_total = time_voxelization_start.elapsed();


    println!(
        "Voxelization complete in {:.1} seconds, starting visualization.",
        time_voxelization_total.as_secs_f32()
    );

    Ok(voxelized_models)
}


fn main() -> Result<()> {
    let cli_args = CliArgs::parse();

    let console_logging_output_level_filter = {
        if let Some(output_filter) = &cli_args.console_logging_output_level_filter {
            EnvFilter::from_str(output_filter)
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


    let voxelized_scene = perform_voxelization(&cli_args).wrap_err("Failed to voxelize.")?;


    match cli_args.command {
        #[cfg(feature = "visualization")]
        CliCommand::Visualize(visualization_args) => {
            let visualization_voxel_size = cli_args.voxel_size
                * visualization_args
                    .visualization_voxel_size_ratio
                    .unwrap_or(1.0);

            run_visualization(
                &cli_args.gltf_file_path,
                voxelized_scene,
                visualization_voxel_size,
            );
        }

        CliCommand::Export(export_args) => {
            let output_file_name = export_args
                .output_file_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap();

            for (model_index, model) in voxelized_scene.into_iter().enumerate() {
                let model_output_file_path = format!("{}.m-{}.bin", output_file_name, model_index);

                println!(
                    "Exporting model {} to {}...",
                    model_index, model_output_file_path
                );

                export_voxel_grid_as_raw(
                    &export_args
                        .output_file_path
                        .with_file_name(model_output_file_path),
                    &model.grid,
                    export_args.export_format()?,
                )?;
            }
        }
    };



    drop(logging_guard);
    Ok(())
}
