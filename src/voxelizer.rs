use easy_gltf::model::Triangle;

pub fn voxelize_triangles(triangles: Vec<Triangle>) -> ! {
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

    todo!();
}
