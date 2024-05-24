```bash
cargo run -- --voxel-size 0.2 --gltf-file-path ./assets/mesh-sample-01/mesh-sample-01_torus_textured.gltf --voxelization-bounds "(-10.0, -10.0, -10.0) / (10.0, 10.0, 10.0)" visualize --visualization-voxel-size 0.98

cargo run -- --voxel-size 0.2 --gltf-file-path ./assets/mesh-sample-01/mesh-sample-01_torus_textured.gltf --voxelization-bounds "(-10.0, -10.0, -10.0) / (10.0, 10.0, 10.0)" export --export-type linear-rgb8-color_u8 --output-file-path ./exports/torus-textured
```