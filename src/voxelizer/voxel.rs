use std::ops::Add;

use glam::{U64Vec3, Vec3};

use super::aabb::Aabb;

#[derive(Debug)]
pub struct ContextualNonFinalVoxelDataMut<'d> {
    pub grid_index: U64Vec3,

    grid_starting_point: Vec3,
    grid_voxel_half_extent: f32,

    pub data: &'d mut NonFinalVoxelData,
}

impl<'d> ContextualNonFinalVoxelDataMut<'d> {
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


#[inline]
fn average_f32_samples(samples: &[f32]) -> f32 {
    samples.iter().sum::<f32>() / (samples.len() as f32)
}


#[inline]
fn combine_rgb_colors(colors_to_compose: &[Vec3]) -> Vec3 {
    // Colors returned by the gltf crate's material sampler are linear RGB, see
    // <https://docs.rs/easy-gltf/latest/src/easy_gltf/scene/model/material/mod.rs.html#49-66>.

    // let first_color = LinSrgb::new(first_color.x, first_color.y, first_color.z);
    // let second_color = LinSrgb::new(second_color.x, second_color.y, second_color.z);

    let mut mixed_r: f32 = 0.0;
    let mut mixed_g: f32 = 0.0;
    let mut mixed_b: f32 = 0.0;

    for color in colors_to_compose {
        mixed_r += color.x.powi(2);
        mixed_g += color.y.powi(2);
        mixed_b += color.z.powi(2);
    }

    let mixed_r = (mixed_r / (colors_to_compose.len() as f32)).sqrt();
    let mixed_g = (mixed_g / (colors_to_compose.len() as f32)).sqrt();
    let mixed_b = (mixed_b / (colors_to_compose.len() as f32)).sqrt();

    Vec3::new(mixed_r, mixed_g, mixed_b)
}


#[derive(Clone, Debug)]
pub enum NonFinalVoxelData {
    Empty,
    Edge {
        base_color_or_texture_samples: Vec<Vec3>,
        metallic_value_samples: Vec<f32>,
        roughness_value_samples: Vec<f32>,
    },
    InsideMesh,
}

impl NonFinalVoxelData {
    #[inline]
    pub fn new_empty() -> Self {
        Self::Empty
    }

    #[inline]
    pub fn as_contextual_mut(
        &mut self,
        grid_starting_point: Vec3,
        grid_voxel_half_extent: f32,
        grid_index: U64Vec3,
    ) -> ContextualNonFinalVoxelDataMut<'_> {
        ContextualNonFinalVoxelDataMut {
            grid_index,
            grid_starting_point,
            grid_voxel_half_extent,
            data: self,
        }
    }

    pub fn into_final_voxel_data(self) -> VoxelData {
        match self {
            NonFinalVoxelData::Empty => VoxelData::Empty,
            NonFinalVoxelData::Edge {
                base_color_or_texture_samples,
                metallic_value_samples,
                roughness_value_samples,
            } => VoxelData::Edge {
                color: combine_rgb_colors(&base_color_or_texture_samples),
                metallic_value: average_f32_samples(&metallic_value_samples),
                rougness_value: average_f32_samples(&roughness_value_samples),
            },
            NonFinalVoxelData::InsideMesh => VoxelData::InsideMesh,
        }
    }
}



#[derive(Clone, Debug)]
pub enum VoxelData {
    Empty,
    Edge {
        color: Vec3,
        metallic_value: f32,
        rougness_value: f32,
    },
    InsideMesh,
}

impl VoxelData {
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
}
