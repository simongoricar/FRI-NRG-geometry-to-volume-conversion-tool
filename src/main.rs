use std::{path::Path, str::FromStr};

use clap::Parser;
use easy_gltf::model::Triangle;
use miette::{miette, Context, IntoDiagnostic, Result};
use tracing_subscriber::EnvFilter;

use crate::{cli::CliArgs, logging::initialize_tracing, voxelizer::voxelize_triangles};

mod cli;
mod logging;
mod voxelizer;


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


    let mesh_triangles = load_mesh_triangles_from_file(cli_args.input_file_path)
        .wrap_err("Failed to load mesh and parse triangles from GLTF file.")?;

    println!("Loaded {} triangles.", mesh_triangles.len());


    // TODO run through voxelizer
    voxelize_triangles(mesh_triangles);

    // TODO export to BVP format

    // TODO visualize?

    drop(logging_guard);
    Ok(())
}
