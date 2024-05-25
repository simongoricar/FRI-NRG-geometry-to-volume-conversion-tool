use std::ops::{Add, Div, Sub};

use easy_gltf::{
    model::{Triangle, Vertex},
    Material,
    Model,
};
use glam::Vec3;
use parry3d::{
    bounding_volume::Aabb as Parry3dAabb,
    na::Point3 as Parry3dPoint3,
    query::details::intersection_test_aabb_triangle,
    shape::Triangle as Parry3dTriangle,
};

use self::{
    aabb::Aabb,
    grid::{ContextualVoxelGrid, NonFinalVoxelGrid},
    voxel::NonFinalVoxelData,
};

pub mod aabb;
pub mod grid;
pub mod voxel;




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


fn initialize_voxel_work_grid(
    voxelization_bounds: &Aabb,
    voxel_full_size: f32,
) -> NonFinalVoxelGrid {
    let num_voxels_on_x =
        (voxelization_bounds.max.x - voxelization_bounds.min.x).div(voxel_full_size) as u64;
    let num_voxels_on_y =
        (voxelization_bounds.max.y - voxelization_bounds.min.y).div(voxel_full_size) as u64;
    let num_voxels_on_z =
        (voxelization_bounds.max.z - voxelization_bounds.min.z).div(voxel_full_size) as u64;

    NonFinalVoxelGrid::new(
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



fn check_for_triangle_aabb_collision(voxel_aabb: &Aabb, triangle: &[Vertex; 3]) -> bool {
    intersection_test_aabb_triangle(
        &Parry3dAabb::new(
            Parry3dPoint3::new(
                voxel_aabb.min.x,
                voxel_aabb.min.y,
                voxel_aabb.min.z,
            ),
            Parry3dPoint3::new(
                voxel_aabb.max.x,
                voxel_aabb.max.y,
                voxel_aabb.max.z,
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
    )
}



fn compute_minimum_aabb_for_mesh(mesh_triangles: &[Triangle], padding: f32) -> Aabb {
    let mut current_minimum = Vec3::MAX;
    let mut current_maximum = Vec3::MIN;

    for triangle in mesh_triangles {
        for vertex in triangle {
            current_minimum.x = current_minimum.x.min(vertex.position.x);
            current_minimum.y = current_minimum.y.min(vertex.position.y);
            current_minimum.z = current_minimum.z.min(vertex.position.z);

            current_maximum.x = current_maximum.x.max(vertex.position.x);
            current_maximum.y = current_maximum.y.max(vertex.position.y);
            current_maximum.z = current_maximum.z.max(vertex.position.z);
        }
    }

    Aabb::from_min_and_max(
        current_minimum.sub(padding),
        current_maximum.add(padding),
    )
}


fn get_closest_vertex(triangle: &[Vertex; 3], target_voxel_center: Vec3) -> &Vertex {
    // Choose closest vertex and sample its color.
    let vertex_one_distance = target_voxel_center.distance(Vec3::new(
        triangle[0].position.x,
        triangle[0].position.y,
        triangle[0].position.z,
    ));

    let mut closest_vertex = &triangle[0];
    let mut closest_vertex_distance = vertex_one_distance;


    let vertex_two_distance = target_voxel_center.distance(Vec3::new(
        triangle[1].position.x,
        triangle[1].position.y,
        triangle[1].position.z,
    ));

    if vertex_two_distance < closest_vertex_distance {
        closest_vertex_distance = vertex_two_distance;
        closest_vertex = &triangle[1];
    }


    let vertex_three_distance = target_voxel_center.distance(Vec3::new(
        triangle[2].position.x,
        triangle[2].position.y,
        triangle[2].position.z,
    ));

    if vertex_three_distance < closest_vertex_distance {
        // closest_vertex_distance = vertex_three_distance;
        closest_vertex = &triangle[2];
    }

    closest_vertex
}


#[inline]
fn sample_vertex_color(vertex: &Vertex, model_material: &Material) -> Vec3 {
    let sampled_color = model_material.get_base_color(vertex.tex_coords);

    Vec3::new(sampled_color.x, sampled_color.y, sampled_color.z)
}


fn voxelize_individual_model(
    model: &Model,
    max_voxelization_bounds: &Aabb,
    voxel_size: f32,
) -> ContextualVoxelGrid {
    let model_triangles = model
        .triangles()
        .expect("expected the mesh to contain triangles");


    let minimum_voxelization_bounds_to_cover_model =
        compute_minimum_aabb_for_mesh(&model_triangles, voxel_size * 2.0);

    // We don't want to waste memory on useless voxel space, so we reduce the user-provided
    // maximum voxelization bound according to the intersection between the extend of the mesh
    // and the maximum voxelization extent.
    let actual_voxelization_bounds =
        minimum_voxelization_bounds_to_cover_model.compute_intersection(max_voxelization_bounds);


    let mut voxel_grid = initialize_voxel_work_grid(&actual_voxelization_bounds, voxel_size);


    let model_material = model.material();

    println!("Voxelizing {} triangles.", model_triangles.len());

    for triangle in model_triangles {
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
                    let target_voxel = voxel_grid
                        .contextual_non_final_voxel_mut_by_xyz_index_unchecked(
                            grid_index_x,
                            grid_index_y,
                            grid_index_z,
                        );

                    let target_voxel_center = target_voxel.center_coordinate_in_world_space();
                    let target_voxel_aabb = target_voxel.aabb();


                    // Checks whether the voxel "box" intersects the triangle.
                    let triangle_intersects_with_voxel =
                        check_for_triangle_aabb_collision(&target_voxel_aabb, &triangle);

                    if triangle_intersects_with_voxel {
                        // DEBUGONLY
                        // println!(
                        //     "Found intersecting voxel at ({}, {}, {})",
                        //     target_voxel_center.x, target_voxel_center.y, target_voxel_center.z,
                        // );

                        let best_vertex = get_closest_vertex(&triangle, target_voxel_center);

                        let sampled_color = sample_vertex_color(best_vertex, &model_material);
                        let sampled_metallic_value =
                            model_material.get_metallic(best_vertex.tex_coords);
                        let sampled_roughness_value =
                            model_material.get_roughness(best_vertex.tex_coords);


                        match target_voxel.data {
                            NonFinalVoxelData::Empty | NonFinalVoxelData::InsideMesh => {
                                *target_voxel.data = NonFinalVoxelData::Edge {
                                    color_samples: vec![sampled_color],
                                    metallic_value_samples: vec![sampled_metallic_value],
                                    roughness_value_samples: vec![sampled_roughness_value],
                                }
                            }
                            NonFinalVoxelData::Edge {
                                color_samples: base_color_or_texture_samples,
                                metallic_value_samples,
                                roughness_value_samples,
                            } => {
                                base_color_or_texture_samples.push(sampled_color);
                                metallic_value_samples.push(sampled_metallic_value);
                                roughness_value_samples.push(sampled_roughness_value);
                            }
                        }
                    }
                }
            }
        }
    }


    // Compute binary fill state.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    enum BinaryState {
        OutsideMesh,
        InsideMesh,
    }

    for grid_index_x in 0..voxel_grid.x_length {
        for grid_index_y in 0..voxel_grid.y_length {
            let mut current_state = BinaryState::OutsideMesh;
            let mut previous_was_edge = false;

            for grid_index_z in 0..voxel_grid.z_length {
                let current_voxel = voxel_grid
                    .contextual_non_final_voxel_mut_by_xyz_index_unchecked(
                        grid_index_x,
                        grid_index_y,
                        grid_index_z,
                    );

                match current_voxel.data {
                    NonFinalVoxelData::Edge { .. } => {
                        previous_was_edge = true;
                    }
                    NonFinalVoxelData::Empty => {
                        if previous_was_edge {
                            if current_state == BinaryState::OutsideMesh {
                                current_state = BinaryState::InsideMesh;
                            } else {
                                current_state = BinaryState::OutsideMesh;
                            }

                            previous_was_edge = false;
                        }

                        if current_state == BinaryState::InsideMesh {
                            *current_voxel.data = NonFinalVoxelData::InsideMesh;
                        }
                    }
                    NonFinalVoxelData::InsideMesh => {
                        panic!("encountered InsideMesh voxel while generating them")
                    }
                }
            }


            if current_state == BinaryState::OutsideMesh {
                continue;
            }

            for grid_index_z in (0..voxel_grid.z_length).rev() {
                let current_voxel = voxel_grid
                    .contextual_non_final_voxel_mut_by_xyz_index_unchecked(
                        grid_index_x,
                        grid_index_y,
                        grid_index_z,
                    );

                match current_voxel.data {
                    NonFinalVoxelData::Edge { .. } => break,
                    NonFinalVoxelData::InsideMesh => {
                        *current_voxel.data = NonFinalVoxelData::Empty;
                    }
                    NonFinalVoxelData::Empty => {}
                }
            }
        }
    }

    for grid_index_z in 0..voxel_grid.z_length {
        for grid_index_y in 0..voxel_grid.y_length {
            for grid_index_x in 0..voxel_grid.x_length {
                let current_voxel = voxel_grid
                    .contextual_non_final_voxel_mut_by_xyz_index_unchecked(
                        grid_index_x,
                        grid_index_y,
                        grid_index_z,
                    );

                match current_voxel.data {
                    NonFinalVoxelData::Edge { .. } => {
                        break;
                    }
                    NonFinalVoxelData::Empty => {}
                    NonFinalVoxelData::InsideMesh => {
                        *current_voxel.data = NonFinalVoxelData::Empty;
                    }
                }
            }

            for grid_index_x in (0..voxel_grid.x_length).rev() {
                let current_voxel = voxel_grid
                    .contextual_non_final_voxel_mut_by_xyz_index_unchecked(
                        grid_index_x,
                        grid_index_y,
                        grid_index_z,
                    );

                match current_voxel.data {
                    NonFinalVoxelData::Edge { .. } => break,
                    NonFinalVoxelData::Empty => break,
                    NonFinalVoxelData::InsideMesh => {
                        *current_voxel.data = NonFinalVoxelData::Empty;
                    }
                }
            }
        }
    }

    for grid_index_z in 0..voxel_grid.z_length {
        for grid_index_x in 0..voxel_grid.x_length {
            for grid_index_y in 0..voxel_grid.y_length {
                let current_voxel = voxel_grid
                    .contextual_non_final_voxel_mut_by_xyz_index_unchecked(
                        grid_index_x,
                        grid_index_y,
                        grid_index_z,
                    );

                match current_voxel.data {
                    NonFinalVoxelData::Edge { .. } => break,
                    NonFinalVoxelData::Empty => {}
                    NonFinalVoxelData::InsideMesh => {
                        *current_voxel.data = NonFinalVoxelData::Empty;
                    }
                }
            }

            for grid_index_y in (0..voxel_grid.y_length).rev() {
                let current_voxel = voxel_grid
                    .contextual_non_final_voxel_mut_by_xyz_index_unchecked(
                        grid_index_x,
                        grid_index_y,
                        grid_index_z,
                    );

                match current_voxel.data {
                    NonFinalVoxelData::Edge { .. } => break,
                    NonFinalVoxelData::Empty => {}
                    NonFinalVoxelData::InsideMesh => {
                        *current_voxel.data = NonFinalVoxelData::Empty;
                    }
                }
            }
        }
    }



    ContextualVoxelGrid {
        gltf_model_primitive_index: model.primitive_index(),
        grid: voxel_grid.into_final_grid(),
    }
}


pub fn voxelize_models(
    models: &[Model],
    voxelization_bounds: Aabb,
    voxel_size: f32,
) -> Vec<ContextualVoxelGrid> {
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


    let mut voxelized_models: Vec<ContextualVoxelGrid> = Vec::with_capacity(models.len());

    for model in models {
        let voxelized_model = voxelize_individual_model(model, &voxelization_bounds, voxel_size);

        voxelized_models.push(voxelized_model);
    }


    voxelized_models
}
