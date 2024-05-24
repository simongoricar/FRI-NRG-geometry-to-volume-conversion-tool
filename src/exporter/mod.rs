use std::{
    fs::File,
    io::{
        self,
        prelude::{Read, Write},
        BufWriter,
    },
    path::Path,
};

use miette::{Context, IntoDiagnostic, Result};

use crate::voxelizer::{grid::VoxelGrid, voxel::VoxelData};

pub enum VoxelExportType {
    BinaryEdgeStateU1,
    BinaryFillStateU1,
    LinearRgb8ColorU8,
    MetallicValueU8,
}



pub struct BinaryEdgeStateU1RawWriter<'g> {
    grid_voxels: &'g [VoxelData],
    next_index: usize,
}

impl<'g> BinaryEdgeStateU1RawWriter<'g> {
    pub fn from_grid(grid: &'g VoxelGrid) -> Self {
        Self {
            grid_voxels: grid.voxels(),
            next_index: 0,
        }
    }

    fn next_voxel_fill_state(&mut self) -> Option<bool> {
        if self.next_index >= self.grid_voxels.len() {
            None
        } else {
            let is_next_voxel_filled = matches!(
                self.grid_voxels[self.next_index],
                VoxelData::Edge { .. }
            );

            self.next_index += 1;


            Some(is_next_voxel_filled)
        }
    }
}

impl<'g> Read for BinaryEdgeStateU1RawWriter<'g> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.next_index >= self.grid_voxels.len() {
            return Ok(0);
        }


        // Collect eight next voxels and collect them into a single byte
        // (one bit per voxel - 1 if filled, 0 otherwise).

        let voxel_1 = self.next_voxel_fill_state().unwrap();
        let voxel_2 = self.next_voxel_fill_state().unwrap_or(false);
        let voxel_3 = self.next_voxel_fill_state().unwrap_or(false);
        let voxel_4 = self.next_voxel_fill_state().unwrap_or(false);
        let voxel_5 = self.next_voxel_fill_state().unwrap_or(false);
        let voxel_6 = self.next_voxel_fill_state().unwrap_or(false);
        let voxel_7 = self.next_voxel_fill_state().unwrap_or(false);
        let voxel_8 = self.next_voxel_fill_state().unwrap_or(false);


        // Pack eight booleans into a u8.

        let next_u8_value = {
            let mut value: u8 = 0;

            for (i, b) in [
                voxel_1, voxel_2, voxel_3, voxel_4, voxel_5, voxel_6, voxel_7, voxel_8,
            ]
            .iter()
            .rev()
            .enumerate()
            {
                let bit = if *b { 1 } else { 0 };

                value |= bit << i;
            }

            value
        };


        buf[0] = next_u8_value;
        Ok(1)
    }
}



pub struct LinearRgb8ColorU8RawWriter<'g> {
    grid_voxels: &'g [VoxelData],
    next_index: usize,
}

impl<'g> LinearRgb8ColorU8RawWriter<'g> {
    pub fn from_grid(grid: &'g VoxelGrid) -> Self {
        Self {
            grid_voxels: grid.voxels(),
            next_index: 0,
        }
    }
}


impl<'g> Read for LinearRgb8ColorU8RawWriter<'g> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.next_index >= self.grid_voxels.len() {
            return Ok(0);
        }

        if buf.len() < 3 {
            panic!("expected a buffer of size at least 3");
        }


        let voxel = &self.grid_voxels[self.next_index];

        let VoxelData::Edge { color, .. } = voxel else {
            buf[0] = 0;
            buf[1] = 0;
            buf[2] = 0;

            self.next_index += 1;

            return Ok(3);
        };


        let red_value = (color.x * (u8::MAX as f32)) as u8;
        let green_value = (color.y * (u8::MAX as f32)) as u8;
        let blue_value = (color.z * (u8::MAX as f32)) as u8;

        buf[0] = red_value;
        buf[1] = green_value;
        buf[2] = blue_value;

        self.next_index += 1;

        Ok(3)
    }
}



pub fn export_voxel_grid_as_raw<P>(
    output_file_path: P,
    grid: &VoxelGrid,
    voxel_export_type: VoxelExportType,
) -> Result<()>
where
    P: AsRef<Path>,
{
    match voxel_export_type {
        VoxelExportType::BinaryEdgeStateU1 => {
            let mut file_data_producer = BinaryEdgeStateU1RawWriter::from_grid(grid);

            let file = File::create(output_file_path)
                .into_diagnostic()
                .wrap_err("Failed to open file.")?;

            let mut buffered_file = BufWriter::new(file);

            io::copy(&mut file_data_producer, &mut buffered_file)
                .into_diagnostic()
                .wrap_err("Failed to write to file.")?;

            let mut file = buffered_file
                .into_inner()
                .into_diagnostic()
                .wrap_err("Failed to flush buffered writer.")?;

            file.flush()
                .into_diagnostic()
                .wrap_err("Failed to flush unbuffered file.")?;

            drop(file);
        }
        VoxelExportType::BinaryFillStateU1 => todo!(),
        VoxelExportType::LinearRgb8ColorU8 => {
            let mut file_data_producer = LinearRgb8ColorU8RawWriter::from_grid(grid);

            let file = File::create(output_file_path)
                .into_diagnostic()
                .wrap_err("Failed to open file.")?;

            let mut buffered_file = BufWriter::new(file);

            io::copy(&mut file_data_producer, &mut buffered_file)
                .into_diagnostic()
                .wrap_err("Failed to write to file.")?;

            let mut file = buffered_file
                .into_inner()
                .into_diagnostic()
                .wrap_err("Failed to flush buffered writer.")?;

            file.flush()
                .into_diagnostic()
                .wrap_err("Failed to flush unbuffered file.")?;

            drop(file);
        }
        VoxelExportType::MetallicValueU8 => todo!(),
    }


    Ok(())
}
