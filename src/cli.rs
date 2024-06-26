use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use glam::Vec3;
use miette::{miette, Context, IntoDiagnostic, Result};

use crate::{exporter::VoxelExportType, voxelizer::aabb::Aabb};


#[cfg(feature = "visualization")]
#[derive(Args)]
pub struct VisualizationArgs {
    #[arg(
        long = "visualization-voxel-size",
        help = "Voxel size (full box width) to use for the visualization of the voxelized scene. \
                Defaults to 1 (meaning the size is equal to --voxel-size), but set it to e.g. 0.95 to \
                create a an effect of a tiny edge around each voxel."
    )]
    pub visualization_voxel_size_ratio: Option<f32>,

    #[arg(
        long = "initial-camera-position",
        help = "Initial camera position. The camera will be oriented to look towards the center of the scene. \
                Defaults to an auto-calculated position based on the provided scene."
    )]
    pub initial_camera_position: Option<String>,
}

impl VisualizationArgs {
    pub fn initial_camera_position(&self) -> Result<Option<Vec3>> {
        let Some(initial_camera_position) = &self.initial_camera_position else {
            return Ok(None);
        };

        Ok(Some(parse_xyz_components_from_str(
            initial_camera_position,
        )?))
    }
}



#[derive(Args)]
pub struct ExportArgs {
    #[arg(long = "output-file-path")]
    pub output_file_path: PathBuf,

    #[arg(
        long = "export-type",
        help = "One of: binary-edge_u1, binary-fill_u1, linear-rgb8-color_u8, metallic-value_u8, roughness-value_u8."
    )]
    pub export_format: String,
}

impl ExportArgs {
    pub fn export_format(&self) -> Result<VoxelExportType> {
        let export_format = self.export_format.to_ascii_lowercase();

        match export_format.as_str() {
            "binary-edge_u1" => Ok(VoxelExportType::BinaryEdgeStateU1),
            "binary-fill_u1" => Ok(VoxelExportType::BinaryFillStateU1),
            "linear-rgb8-color_u8" => Ok(VoxelExportType::LinearRgb8ColorU8),
            "metallic-value_u8" => Ok(VoxelExportType::MetallicValueU8),
            "roughness-value_u8" => Ok(VoxelExportType::RoughnessValueU8),
            _ => Err(miette!(
                "Invalid export type, must be one of: binary-edge_u1, binary-fill_u1, \
                linear-rgb8-color_u8, metallic-value_u8, roughness-value_u8."
            )),
        }
    }
}



#[derive(Subcommand)]
pub enum CliCommand {
    #[cfg(feature = "visualization")]
    #[command(name = "visualize")]
    Visualize(VisualizationArgs),

    #[command(name = "export")]
    Export(ExportArgs),
}


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
        short = 'b',
        long = "voxelization-bounds",
        help = "Maximum voxelization bounds as an AABB (axis-aligned bounding box) in world space. \
                The actual voxel grid will be adapted to fit each GLTF model individually, but will not exceed this bound. \
                The format is as follows: \"(min_x, min_y, min_z) / (max_x, max_y, max_z)\". \
                Example: \"(-3, -2.5, -1) / (1, 1, 4.2)\""
    )]
    pub voxelization_bounds: Option<String>,

    #[command(subcommand)]
    pub command: CliCommand,
}



const INVLIAD_VOXELIZATION_FORMAT_MESSAGE: &str = "Invalid voxelization_bounds parameter: expected \
    \"(min_x, min_y, min_z) / (max_x, max_y, max_z)\" format.";


fn parse_xyz_components_from_str(xyz_string_in_parentheses: &str) -> Result<Vec3> {
    let xyz_without_parentheses = xyz_string_in_parentheses
        .trim()
        .strip_prefix("(")
        .unwrap_or(xyz_string_in_parentheses)
        .strip_suffix(")")
        .unwrap_or(xyz_string_in_parentheses);


    let xyz_components = xyz_without_parentheses.split(',').collect::<Vec<_>>();
    if xyz_components.len() != 3 {
        return Err(miette!(INVLIAD_VOXELIZATION_FORMAT_MESSAGE));
    }

    let x_value = xyz_components[0]
        .trim()
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| miette!("Failed to parse {}", xyz_components[0]))
        .wrap_err(INVLIAD_VOXELIZATION_FORMAT_MESSAGE)?;

    let y_value = xyz_components[1]
        .trim()
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| miette!("Failed to parse {}", xyz_components[1]))
        .wrap_err(INVLIAD_VOXELIZATION_FORMAT_MESSAGE)?;

    let z_value = xyz_components[2]
        .trim()
        .parse::<f32>()
        .into_diagnostic()
        .wrap_err_with(|| miette!("Failed to parse {}", xyz_components[2]))
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
