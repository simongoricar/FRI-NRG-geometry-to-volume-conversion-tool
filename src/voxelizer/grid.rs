use glam::{U64Vec3, Vec3};

use super::voxel::{ContextualVoxelData, ContextualVoxelDataMut, VoxelData};


pub struct VoxelGrid {
    pub starting_point: Vec3,

    pub voxel_half_extent: f32,

    x_length: u64,
    y_length: u64,
    z_length: u64,

    /// Indexed by x, y, and z, flattened out as a single Vec.
    grid: Vec<VoxelData>,
}

impl VoxelGrid {
    pub fn voxels(&self) -> &[VoxelData] {
        &self.grid
    }

    pub fn contextual_voxels(&self) -> VoxelGridContextualIterator {
        VoxelGridContextualIterator::from_voxel_grid(self)
    }

    /// `starting_point` is the bounding box edge of the lowest-x lowest-y lowest-z voxel
    /// (meaning the first voxel center is at `starting_point + voxel_half_size`).
    pub fn new(
        starting_point: Vec3,
        voxel_half_size: f32,
        x_size: u64,
        y_size: u64,
        z_size: u64,
    ) -> Self {
        let grid = (0..(x_size * y_size * z_size))
            .map(|_| VoxelData::new_empty())
            .collect::<Vec<_>>();


        Self {
            starting_point,
            voxel_half_extent: voxel_half_size,
            x_length: x_size,
            y_length: y_size,
            z_length: z_size,
            grid,
        }
    }

    pub fn voxel_mut_by_xyz_index_unchecked(&mut self, x: u64, y: u64, z: u64) -> &mut VoxelData {
        let target_voxel_index = x + (y * self.x_length) + (z * self.y_length * self.x_length);

        self.grid
            .get_mut(target_voxel_index as usize)
            .unwrap_or_else(|| panic!("index ({}, {}, {}) is out of range", x, y, z))
    }

    pub fn contextual_voxel_mut_by_xyz_index_unchecked(
        &mut self,
        x: u64,
        y: u64,
        z: u64,
    ) -> ContextualVoxelDataMut<'_> {
        let starting_point = self.starting_point;
        let half_extent = self.voxel_half_extent;

        self.voxel_mut_by_xyz_index_unchecked(x, y, z)
            .as_contextual_mut(starting_point, half_extent, U64Vec3::new(x, y, z))
    }

    /*
    pub fn overlapping_voxels_mut_by_bounding_box_in_world_spacee(
        &mut self,
        axis_aligned_bounding_box: Aabb,
    ) -> Vec<&mut Voxel> {
        todo!();
    } */
}



pub struct ContextualVoxelGrid {
    pub gltf_model_primitive_index: usize,
    pub grid: VoxelGrid,
}



pub struct VoxelGridContextualIterator<'g> {
    grid_starting_point: Vec3,

    voxel_half_extent: f32,

    #[allow(dead_code)]
    grid_x_length: u64,

    grid_y_length: u64,

    grid_z_length: u64,

    raw_voxels: &'g [VoxelData],

    next_grid_index: U64Vec3,

    next_index: usize,
}

impl<'g> VoxelGridContextualIterator<'g> {
    fn from_voxel_grid(voxel_grid: &'g VoxelGrid) -> Self {
        Self {
            grid_starting_point: voxel_grid.starting_point,
            voxel_half_extent: voxel_grid.voxel_half_extent,
            grid_x_length: voxel_grid.x_length,
            grid_y_length: voxel_grid.y_length,
            grid_z_length: voxel_grid.z_length,
            raw_voxels: voxel_grid.voxels(),
            next_grid_index: U64Vec3::new(0, 0, 0),
            next_index: 0,
        }
    }
}

impl<'g> Iterator for VoxelGridContextualIterator<'g> {
    type Item = ContextualVoxelData<'g>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index >= self.raw_voxels.len() {
            return None;
        }


        let contextual_voxel = self.raw_voxels[self.next_index].as_contextual(
            self.grid_starting_point,
            self.voxel_half_extent,
            self.next_grid_index,
        );


        self.next_grid_index.x += 1;
        if self.next_grid_index.x >= self.grid_x_length {
            self.next_grid_index.x = 0;
            self.next_grid_index.y += 1;
        }
        if self.next_grid_index.y >= self.grid_y_length {
            self.next_grid_index.y = 0;
            self.next_grid_index.z += 1;
        }

        self.next_index += 1;


        Some(contextual_voxel)
    }
}
