use std::ops::Add;

use glam::{U64Vec3, Vec3};

use super::aabb::Aabb;

#[derive(Debug)]
pub struct ContextualVoxelDataMut<'d> {
    pub grid_index: U64Vec3,

    grid_starting_point: Vec3,
    grid_voxel_half_extent: f32,

    pub data: &'d mut VoxelData,
}

impl<'d> ContextualVoxelDataMut<'d> {
    pub fn center_coordinate_in_world_space(&self) -> Vec3 {
        let first_voxel_center = self.grid_starting_point.add(self.grid_voxel_half_extent);
        let full_voxel_extent = self.grid_voxel_half_extent * 2.0;

        Vec3::new(
            first_voxel_center.x + (full_voxel_extent * self.grid_index.x as f32),
            first_voxel_center.y + (full_voxel_extent * self.grid_index.y as f32),
            first_voxel_center.z + (full_voxel_extent * self.grid_index.z as f32),
        )
    }

    pub fn aabb(&self) -> Aabb {
        let full_voxel_extent = self.grid_voxel_half_extent * 2.0;


        let aabb_min = Vec3::new(
            self.grid_starting_point.x + (full_voxel_extent * self.grid_index.x as f32),
            self.grid_starting_point.y + (full_voxel_extent * self.grid_index.y as f32),
            self.grid_starting_point.z + (full_voxel_extent * self.grid_index.z as f32),
        );

        let aabb_max = aabb_min.add(full_voxel_extent);


        Aabb::from_min_and_max(aabb_min, aabb_max)
    }
}


#[derive(Debug)]
pub struct ContextualVoxelData<'d> {
    pub grid_index: U64Vec3,

    grid_starting_point: Vec3,
    grid_voxel_half_extent: f32,

    pub data: &'d VoxelData,
}

impl<'d> ContextualVoxelData<'d> {
    pub fn center_coordinate_in_world_space(&self) -> Vec3 {
        let first_voxel_center = self.grid_starting_point.add(self.grid_voxel_half_extent);
        let full_voxel_extent = self.grid_voxel_half_extent * 2.0;

        Vec3::new(
            first_voxel_center.x + (full_voxel_extent * self.grid_index.x as f32),
            first_voxel_center.y + (full_voxel_extent * self.grid_index.y as f32),
            first_voxel_center.z + (full_voxel_extent * self.grid_index.z as f32),
        )
    }
}



#[derive(Clone, Debug)]
pub enum VoxelData {
    Empty,
    Edge { base_color: Vec3 },
    InsideMesh,
}

impl VoxelData {
    #[inline]
    pub fn new_empty() -> Self {
        Self::Empty
    }

    #[inline]
    pub fn as_contextual(
        &self,
        grid_starting_point: Vec3,
        grid_voxel_half_extent: f32,
        grid_index: U64Vec3,
    ) -> ContextualVoxelData<'_> {
        ContextualVoxelData {
            grid_index,
            grid_starting_point,
            grid_voxel_half_extent,
            data: self,
        }
    }

    #[inline]
    pub fn as_contextual_mut(
        &mut self,
        grid_starting_point: Vec3,
        grid_voxel_half_extent: f32,
        grid_index: U64Vec3,
    ) -> ContextualVoxelDataMut<'_> {
        ContextualVoxelDataMut {
            grid_index,
            grid_starting_point,
            grid_voxel_half_extent,
            data: self,
        }
    }
}
