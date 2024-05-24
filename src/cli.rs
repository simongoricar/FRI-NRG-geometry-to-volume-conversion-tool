use std::path::PathBuf;

use clap::Parser;
use glam::Vec3;
use miette::{miette, Context, IntoDiagnostic, Result};

use crate::voxelizer::aabb::Aabb;


#[derive(Parser)]
#[command(
    name = "nrg-m2v",
    author = "Simon G. <simon.peter.goricar@gmail.com>",
    version
)]
pub struct CliArgs {
    #[arg(
        long = "console-logging-level",
        help = "Specifies the console logging level, e.g. \"info,bevy=warn\". \
                If set, this value overrides the RUST_LOG environment variable."
    )]
    pub console_logging_output_level_filter: Option<String>,

    #[arg(
        short = 'i',
        long = "gltf-file-path",
        help = "Path to the GLTF file containing the scene to voxelize."
    )]
    pub gltf_file_path: PathBuf,

    #[arg(
        short = 's',
        long = "voxel-size",
        help = "Voxel size (full box width). The smaller the voxel, the higher the grid resolution, \
                but at some point you will likely run out of memory."
    )]
    pub voxel_size: f32,

    #[arg(
        long = "visualization-voxel-size",
        help = "Voxel size (full box width) to use for the visualization of the voxelized scene. \
                Defaults to 1 (meaning the size is equal to --voxel-size), but set it to e.g. 0.95 to \
                create a an effect of a tiny edge around each voxel."
    )]
    pub visualization_voxel_size_ratio: Option<f32>,

    #[arg(
        short = 'b',
        long = "voxelization-bounds",
        help = "Maximum voxelization bounds as an AABB (axis-aligned bounding box) in world space. \
                The actual voxel grid will be adapted to fit each GLTF model individually, but will not exceed this bound. \
                The format is as follows: \"(min_x, min_y, min_z) / (max_x, max_y, max_z)\". \
                Example: \"(-3, -2.5, -1) / (1, 1, 4.2)\""
    )]
    pub voxelization_bounds: Option<String>,
    // TODO
}



const INVLIAD_VOXELIZATION_FORMAT_MESSAGE: &str = "Invalid voxelization_bounds parameter: expected \
    \"(min_x, min_y, min_z) / (max_x, max_y, max_z)\" format.";


fn parse_xyz_components_from_str(xyz_string_in_parentheses: &str) -> Result<Vec3> {
    let xyz_without_parentheses = xyz_string_in_parentheses
        .trim()
        .strip_prefix("(")
        .ok_or_else(|| miette!(INVLIAD_VOXELIZATION_FORMAT_MESSAGE))?
        .strip_suffix(")")
        .ok_or_else(|| miette!(INVLIAD_VOXELIZATION_FORMAT_MESSAGE))?;


    let xyz_components = xyz_without_parentheses.split(',').collect::<Vec<_>>();
    if xyz_components.len() != 3 {
        return Err(miette!(INVLIAD_VOXELIZATION_FORMAT_MESSAGE));
    }

    let x_value = xyz_components[0]
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err(INVLIAD_VOXELIZATION_FORMAT_MESSAGE)?;

    let y_value = xyz_components[1]
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err(INVLIAD_VOXELIZATION_FORMAT_MESSAGE)?;

    let z_value = xyz_components[2]
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err(INVLIAD_VOXELIZATION_FORMAT_MESSAGE)?;


    Ok(Vec3::new(x_value, y_value, z_value))
}


impl CliArgs {
    pub fn voxelization_bounds(&self) -> Result<Option<Aabb>> {
        let Some(voxelization_bounds_str) = self.voxelization_bounds.as_ref() else {
            return Ok(None);
        };

        let Some((minimum_bounds_str, maximum_bounds_str)) = voxelization_bounds_str.split_once("/")
        else {
            return Err(miette!(INVLIAD_VOXELIZATION_FORMAT_MESSAGE));
        };

        let minimum_bounds = parse_xyz_components_from_str(minimum_bounds_str)?;
        let maximum_bounds = parse_xyz_components_from_str(maximum_bounds_str)?;


        Ok(Some(Aabb::from_min_and_max(
            minimum_bounds,
            maximum_bounds,
        )))
    }
}
