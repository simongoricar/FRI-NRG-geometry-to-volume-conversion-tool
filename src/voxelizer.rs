use std::ops::Div;

use easy_gltf::model::Triangle;
use nalgebra::{Point3, Vector3};
use parry3d::query::details::intersection_test_aabb_triangle;


#[derive(Clone, Debug)]
pub struct Voxel {
    grid_index: Vector3<usize>,
    pub is_filled: bool,
}

impl Voxel {
    pub fn get_center_coordinate_in_world_space(
        &self,
        starting_voxel_grid_point: &Vector3<f32>,
        voxel_half_size: f32,
    ) -> Vector3<f32> {
        let first_voxel_center = starting_voxel_grid_point.add_scalar(voxel_half_size);

        Vector3::new(
            first_voxel_center.x + (voxel_half_size * 2.0 * self.grid_index.x as f32),
            first_voxel_center.y + (voxel_half_size * 2.0 * self.grid_index.y as f32),
            first_voxel_center.z + (voxel_half_size * 2.0 * self.grid_index.z as f32),
        )
    }
}


pub struct VoxelGrid {
    pub starting_point: Vector3<f32>,
    pub voxel_half_size: f32,

    x_length: usize,
    y_length: usize,
    z_length: usize,

    /// Indexed by x, y, and z, flattened out as a single Vec.
    grid: Vec<Voxel>,
}

impl VoxelGrid {
    pub fn voxels(&self) -> &[Voxel] {
        &self.grid
    }
}


pub struct Aabb {
    pub min: Vector3<f32>,
    pub max: Vector3<f32>,
}

impl Aabb {
    #[inline]
    pub fn center(&self) -> Vector3<f32> {
        Vector3::new(
            (self.max.x + self.min.x) / 2.0,
            (self.max.y + self.min.y) / 2.0,
            (self.max.z + self.min.z) / 2.0,
        )
    }

    #[inline]
    pub fn half_reach(&self) -> Vector3<f32> {
        Vector3::new(
            (self.max.x - self.min.x) / 2.0,
            (self.max.y - self.min.y) / 2.0,
            (self.max.z - self.min.z) / 2.0,
        )
    }
}



impl VoxelGrid {
    /// `starting_point` is the bounding box edge of the lowest-x lowest-y lowest-z voxel
    /// (meaning the first voxel center is at `starting_point + voxel_half_size`).
    pub fn new(
        starting_point: Vector3<f32>,
        voxel_half_size: f32,
        x_size: usize,
        y_size: usize,
        z_size: usize,
    ) -> Self {
        let mut grid = Vec::with_capacity(x_size * y_size * z_size);

        for z_index in 0..z_size {
            for y_index in 0..y_size {
                for x_index in 0..x_size {
                    let voxel = Voxel {
                        grid_index: Vector3::new(x_index, y_index, z_index),
                        is_filled: false,
                    };

                    grid.push(voxel);
                }
            }
        }

        Self {
            starting_point,
            voxel_half_size,
            x_length: x_size,
            y_length: y_size,
            z_length: z_size,
            grid,
        }
    }

    pub fn voxel_mut_by_xyz_index_unchecked(&mut self, x: usize, y: usize, z: usize) -> &mut Voxel {
        self.grid
            .get_mut(x + (y * self.x_length) + (z * self.y_length * self.x_length))
            .unwrap()
    }

    pub fn overlapping_voxels_mut_by_bounding_box_in_world_spacee(
        &mut self,
        axis_aligned_bounding_box: Aabb,
    ) -> Vec<&mut Voxel> {
        todo!();
    }
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


    let mut voxel_grid = {
        let num_voxels_on_x =
            (voxelization_bounds.max.x - voxelization_bounds.min.x).div(voxel_size) as usize;
        let num_voxels_on_y =
            (voxelization_bounds.max.y - voxelization_bounds.min.y).div(voxel_size) as usize;
        let num_voxels_on_z =
            (voxelization_bounds.max.z - voxelization_bounds.min.z).div(voxel_size) as usize;

        VoxelGrid::new(
            voxelization_bounds.min,
            voxel_size / 2.0,
            num_voxels_on_x,
            num_voxels_on_y,
            num_voxels_on_z,
        )
    };

    for triangle in triangles {
        let triangle_aabb = {
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


            Aabb {
                min: Vector3::new(
                    minimum_triangle_x,
                    minimum_triangle_y,
                    minimum_triangle_z,
                ),
                max: Vector3::new(
                    maximum_triangle_x,
                    maximum_triangle_y,
                    maximum_triangle_z,
                ),
            }
        };

        let (index_x_start, index_y_start, index_z_start) = {
            let starting_x_index =
                (triangle_aabb.min.x - voxel_grid.starting_point.x).div(voxel_size) as usize;
            let starting_y_index =
                (triangle_aabb.min.y - voxel_grid.starting_point.y).div(voxel_size) as usize;
            let starting_z_index =
                (triangle_aabb.min.z - voxel_grid.starting_point.z).div(voxel_size) as usize;

            (
                starting_x_index,
                starting_y_index,
                starting_z_index,
            )
        };

        let (index_x_num, index_y_num, index_z_num) = {
            let x_index_length =
                (triangle_aabb.max.x - triangle_aabb.min.x).div(voxel_size) as usize + 2;
            let y_index_length =
                (triangle_aabb.max.y - triangle_aabb.min.y).div(voxel_size) as usize + 2;
            let z_index_length =
                (triangle_aabb.max.z - triangle_aabb.min.z).div(voxel_size) as usize + 2;

            (x_index_length, y_index_length, z_index_length)
        };


        for grid_index_x in index_x_start..(index_x_start + index_x_num) {
            for grid_index_y in index_y_start..(index_y_start + index_y_num) {
                for grid_index_z in index_z_start..(index_z_start + index_z_num) {
                    let intersects = intersection_test_aabb_triangle(
                        &parry3d::bounding_volume::Aabb::new(
                            Point3::from(triangle_aabb.min),
                            Point3::from(triangle_aabb.max),
                        ),
                        &parry3d::shape::Triangle::new(
                            Point3::new(
                                triangle[0].position.x,
                                triangle[0].position.y,
                                triangle[0].position.z,
                            ),
                            Point3::new(
                                triangle[1].position.x,
                                triangle[1].position.y,
                                triangle[1].position.z,
                            ),
                            Point3::new(
                                triangle[2].position.x,
                                triangle[2].position.y,
                                triangle[2].position.z,
                            ),
                        ),
                    );

                    if intersects {
                        let intersected_voxel = voxel_grid.voxel_mut_by_xyz_index_unchecked(
                            grid_index_x,
                            grid_index_y,
                            grid_index_z,
                        );

                        intersected_voxel.is_filled = true;
                    }

                    // todo!("get mutable Voxel and its center point");
                    // todo!("check for triangle-AABB intersection");
                    // todo!("if intersected, fill the voxel");
                }
            }
        }
    }


    voxel_grid
}
