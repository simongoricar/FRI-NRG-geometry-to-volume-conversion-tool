use std::ops::{Add, Div};

use easy_gltf::model::{Triangle, Vertex};
use glam::{U64Vec3, Vec3};
use parry3d::{
    bounding_volume::Aabb as Parry3dAabb,
    na::Point3 as Parry3dPoint3,
    query::details::intersection_test_aabb_triangle,
    shape::Triangle as Parry3dTriangle,
};


#[derive(Clone, Debug)]
pub struct Voxel {
    grid_index: U64Vec3,
    pub is_filled: bool,
}

impl Voxel {
    pub fn center_coordinate_in_world_space(
        &self,
        starting_voxel_grid_point: &Vec3,
        voxel_half_size: f32,
    ) -> Vec3 {
        let first_voxel_center = starting_voxel_grid_point.add(voxel_half_size);

        Vec3::new(
            first_voxel_center.x + (voxel_half_size * 2.0 * self.grid_index.x as f32),
            first_voxel_center.y + (voxel_half_size * 2.0 * self.grid_index.y as f32),
            first_voxel_center.z + (voxel_half_size * 2.0 * self.grid_index.z as f32),
        )
    }

    pub fn aabb(&self, starting_voxel_grid_point: &Vec3, voxel_half_size: f32) -> Aabb {
        let aabb_min = Vec3::new(
            starting_voxel_grid_point.x + (voxel_half_size * 2.0 * self.grid_index.x as f32),
            starting_voxel_grid_point.y + (voxel_half_size * 2.0 * self.grid_index.y as f32),
            starting_voxel_grid_point.z + (voxel_half_size * 2.0 * self.grid_index.z as f32),
        );

        let aabb_max = aabb_min.add(voxel_half_size * 2.0);


        Aabb::from_min_and_max(aabb_min, aabb_max)
    }
}


pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    #[inline]
    pub fn from_min_and_max(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.max.x + self.min.x) / 2.0,
            (self.max.y + self.min.y) / 2.0,
            (self.max.z + self.min.z) / 2.0,
        )
    }

    #[inline]
    pub fn half_reach(&self) -> Vec3 {
        Vec3::new(
            (self.max.x - self.min.x) / 2.0,
            (self.max.y - self.min.y) / 2.0,
            (self.max.z - self.min.z) / 2.0,
        )
    }
}



pub struct VoxelGrid {
    pub starting_point: Vec3,

    pub voxel_half_extent: f32,

    x_length: u64,
    y_length: u64,
    z_length: u64,

    /// Indexed by x, y, and z, flattened out as a single Vec.
    grid: Vec<Voxel>,
}

impl VoxelGrid {
    pub fn voxels(&self) -> &[Voxel] {
        &self.grid
    }
}

impl VoxelGrid {
    /// `starting_point` is the bounding box edge of the lowest-x lowest-y lowest-z voxel
    /// (meaning the first voxel center is at `starting_point + voxel_half_size`).
    pub fn new(
        starting_point: Vec3,
        voxel_half_size: f32,
        x_size: u64,
        y_size: u64,
        z_size: u64,
    ) -> Self {
        let mut grid = Vec::with_capacity((x_size * y_size * z_size) as usize);

        for z_index in 0..z_size {
            for y_index in 0..y_size {
                for x_index in 0..x_size {
                    let voxel = Voxel {
                        grid_index: U64Vec3::new(x_index, y_index, z_index),
                        is_filled: false,
                    };

                    grid.push(voxel);
                }
            }
        }

        Self {
            starting_point,
            voxel_half_extent: voxel_half_size,
            x_length: x_size,
            y_length: y_size,
            z_length: z_size,
            grid,
        }
    }

    pub fn voxel_mut_by_xyz_index_unchecked(&mut self, x: u64, y: u64, z: u64) -> &mut Voxel {
        let target_voxel_index = x + (y * self.x_length) + (z * self.y_length * self.x_length);

        self.grid
            .get_mut(target_voxel_index as usize)
            .unwrap_or_else(|| panic!("index ({}, {}, {}) is out of range", x, y, z))
    }

    /*
    pub fn overlapping_voxels_mut_by_bounding_box_in_world_spacee(
        &mut self,
        axis_aligned_bounding_box: Aabb,
    ) -> Vec<&mut Voxel> {
        todo!();
    } */
}


/*
fn check_box_triangle_collision(triangle: &[Vertex; 3], aabb: &Aabb) -> bool {
    let mut vertex_first = Vector3::new(
        triangle[0].position.x,
        triangle[0].position.y,
        triangle[0].position.z,
    );

    let mut vertex_second = Vector3::new(
        triangle[1].position.x,
        triangle[1].position.y,
        triangle[1].position.z,
    );

    let mut vertex_third = Vector3::new(
        triangle[2].position.x,
        triangle[2].position.y,
        triangle[2].position.z,
    );


    let aabb_center = aabb.center();
    let aabb_half_reach = aabb.half_reach();

    // Move the vertices to the origin.
    vertex_first -= aabb_center;
    vertex_second -= aabb_center;
    vertex_third -= aabb_center;


    // Compute edge vectors for the triangle.
    let edge_one = vertex_second - vertex_first;
    let edge_two = vertex_third - vertex_second;
    let edge_three = vertex_first - vertex_third;


    let aabb_normal_one = Vector3::new(1.0, 0.0, 0.0);
    let aabb_normal_two = Vector3::new(0.0, 1.0, 0.0);
    let aabb_normal_three = Vector3::new(0.0, 0.0, 1.0);


    // Construct 9 axis for the SAT.
    let axis_edge_one_normal_one = aabb_normal_one.cross(&edge_one);
    let axis_edge_two_normal_one = aabb_normal_one.cross(&edge_two);
    let axis_edge_three_normal_one = aabb_normal_one.cross(&edge_three);

    let axis_edge_one_normal_two = aabb_normal_two.cross(&edge_one);
    let axis_edge_two_normal_two = aabb_normal_two.cross(&edge_two);
    let axis_edge_three_normal_two = aabb_normal_two.cross(&edge_three);

    let axis_edge_one_normal_three = aabb_normal_three.cross(&edge_one);
    let axis_edge_two_normal_three = aabb_normal_three.cross(&edge_two);
    let axis_edge_three_normal_three = aabb_normal_three.cross(&edge_three);


    // Test `axis_edge_one_normal_one` axis.

    // Project all three vertices onto the separating axis.
    let projection_one = vertex_first.dot(&axis_edge_one_normal_one);
    let projection_two = vertex_second.dot(&axis_edge_one_normal_one);
    let projection_three = vertex_third.dot(&axis_edge_one_normal_one);

    todo!();
} */

fn initialize_voxel_grid(voxelization_bounds: Aabb, voxel_full_size: f32) -> VoxelGrid {
    let num_voxels_on_x =
        (voxelization_bounds.max.x - voxelization_bounds.min.x).div(voxel_full_size) as u64;
    let num_voxels_on_y =
        (voxelization_bounds.max.y - voxelization_bounds.min.y).div(voxel_full_size) as u64;
    let num_voxels_on_z =
        (voxelization_bounds.max.z - voxelization_bounds.min.z).div(voxel_full_size) as u64;

    VoxelGrid::new(
        voxelization_bounds.min,
        voxel_full_size / 2.0,
        num_voxels_on_x,
        num_voxels_on_y,
        num_voxels_on_z,
    )
}


fn compute_aabb_for_mesh_triangle(triangle: &[Vertex; 3]) -> Aabb {
    let minimum_triangle_x = triangle[0]
        .position
        .x
        .min(triangle[1].position.x)
        .min(triangle[2].position.x);

    let minimum_triangle_y = triangle[0]
        .position
        .y
        .min(triangle[1].position.y)
        .min(triangle[2].position.y);

    let minimum_triangle_z = triangle[0]
        .position
        .z
        .min(triangle[1].position.z)
        .min(triangle[2].position.z);


    let maximum_triangle_x = triangle[0]
        .position
        .x
        .max(triangle[1].position.x)
        .max(triangle[2].position.x);

    let maximum_triangle_y = triangle[0]
        .position
        .y
        .max(triangle[1].position.y)
        .max(triangle[2].position.y);

    let maximum_triangle_z = triangle[0]
        .position
        .z
        .max(triangle[1].position.z)
        .max(triangle[2].position.z);

    Aabb::from_min_and_max(
        Vec3::new(
            minimum_triangle_x,
            minimum_triangle_y,
            minimum_triangle_z,
        ),
        Vec3::new(
            maximum_triangle_x,
            maximum_triangle_y,
            maximum_triangle_z,
        ),
    )
}




pub fn voxelize_triangles(
    triangles: Vec<Triangle>,
    voxelization_bounds: Aabb,
    voxel_size: f32,
) -> VoxelGrid {
    // TODO

    //! ## Pseudocode (creates a hollow volume)
    //!
    //! ```text
    //! for each triangle:
    //!   find minimum and maximum coordinates in 3D space (a bounding box)
    //!   construct a set of volume points (voxels) that at least partially match the bounding box
    //!   for each of those voxels:
    //!     check if voxel (its "box", really) intersects the current triangle
    //!       (see Fast 3D Triangle-Box Overlap Testing by Tomas Akenine-Möller)
    //!     if it does:
    //!       sample model's texture at provide texture coordinates
    //!         (each triangle in GLTF provides a texture coordinate,
    //!          and its parent model provides the material from which
    //!          we can sample the texture)
    //!       set voxel to sampled texture value
    //! ```
    //!
    //! This will create a hollow volume (with color).
    //!
    //! ## Pseudocode for filling in the volume
    //!
    //! ```text
    //! TODO
    //! ```
    //!
    //!


    let mut voxel_grid = initialize_voxel_grid(voxelization_bounds, voxel_size);

    let voxel_grid_starting_point = voxel_grid.starting_point;
    let voxel_grid_half_extend = voxel_grid.voxel_half_extent;


    for triangle in triangles {
        let triangle_aabb = compute_aabb_for_mesh_triangle(&triangle);

        let (index_x_start, index_y_start, index_z_start) = {
            let starting_x_index =
                (triangle_aabb.min.x - voxel_grid.starting_point.x).div(voxel_size) as u64;
            let starting_y_index =
                (triangle_aabb.min.y - voxel_grid.starting_point.y).div(voxel_size) as u64;
            let starting_z_index =
                (triangle_aabb.min.z - voxel_grid.starting_point.z).div(voxel_size) as u64;

            (
                starting_x_index,
                starting_y_index,
                starting_z_index,
            )
        };

        let (index_x_num, index_y_num, index_z_num) = {
            let x_index_length =
                (triangle_aabb.max.x - triangle_aabb.min.x).div(voxel_size) as u64 + 2;
            let y_index_length =
                (triangle_aabb.max.y - triangle_aabb.min.y).div(voxel_size) as u64 + 2;
            let z_index_length =
                (triangle_aabb.max.z - triangle_aabb.min.z).div(voxel_size) as u64 + 2;

            (x_index_length, y_index_length, z_index_length)
        };



        for grid_index_x in index_x_start..(index_x_start + index_x_num) {
            for grid_index_y in index_y_start..(index_y_start + index_y_num) {
                for grid_index_z in index_z_start..(index_z_start + index_z_num) {
                    // FIXME find aabb of grid voxel

                    let corresponding_voxel = voxel_grid.voxel_mut_by_xyz_index_unchecked(
                        grid_index_x,
                        grid_index_y,
                        grid_index_z,
                    );

                    let corresponding_voxel_aabb =
                        corresponding_voxel.aabb(&voxel_grid_starting_point, voxel_grid_half_extend);


                    // Checks whether the voxel "box" intersects the triangle.
                    let intersects = intersection_test_aabb_triangle(
                        &Parry3dAabb::new(
                            Parry3dPoint3::new(
                                corresponding_voxel_aabb.min.x,
                                corresponding_voxel_aabb.min.y,
                                corresponding_voxel_aabb.min.z,
                            ),
                            Parry3dPoint3::new(
                                corresponding_voxel_aabb.max.x,
                                corresponding_voxel_aabb.max.y,
                                corresponding_voxel_aabb.max.z,
                            ),
                        ),
                        &Parry3dTriangle::new(
                            Parry3dPoint3::new(
                                triangle[0].position.x,
                                triangle[0].position.y,
                                triangle[0].position.z,
                            ),
                            Parry3dPoint3::new(
                                triangle[1].position.x,
                                triangle[1].position.y,
                                triangle[1].position.z,
                            ),
                            Parry3dPoint3::new(
                                triangle[2].position.x,
                                triangle[2].position.y,
                                triangle[2].position.z,
                            ),
                        ),
                    );

                    if intersects {
                        corresponding_voxel.is_filled = true;
                    }
                }
            }
        }
    }


    voxel_grid
}
