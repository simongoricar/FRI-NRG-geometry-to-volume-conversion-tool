<h1 align="center">Geometry to volume conversion</h1>
<h3 align="center">

NRG seminar work<br>
<sub>Faculty for Computer and Information Science</sub><br>
<sub>University of Ljubljana</sub>
</h3>


<div align="center">

![MIT-licensed](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![Tested on 1.80.0-nightly Rust](https://img.shields.io/badge/tested_Rust_version-1.80.0--nightly-orange?style=flat-square)
</div>




# Compilation
To compile this tool, you'll need a reasonably recent version of the Rust compiler (tested on `1.80.0-nightly`, but that should not be required).

Clone the project (you will need Git LFS installed). 
Afterwards, run `cargo build --release` to compile the project in release mode. This will take quite a few minutes, depending on your machine, 
but generally less than twenty minutes.

If you wish to compile the project without visualization features, run `cargo build --release --no-default-features` instead.
Note that when compiled without the `visualization` featuer flag (as above), the `visualize` command will not be available.

You can find the resulting binary in `./target/release`.


# Usage
> Example invocations are available at the bottom.

When running the tool, you must select either the `export` or `visualize` subcommand (described below).
As far as global options go, see the following:

```md
Usage: nrg-mesh-to-volume-conversion [OPTIONS] 
            --gltf-file-path <GLTF_FILE_PATH> 
            --voxel-size <VOXEL_SIZE> <COMMAND>

Commands:
  visualize
  export
  help       Print this message or the help of the given subcommand(s)

Options:
      --console-logging-level <CONSOLE_LOGGING_OUTPUT_LEVEL_FILTER>
          Specifies the console logging level, e.g. "info,bevy=warn". If set, 
          this value overrides the RUST_LOG environment variable.
  -i, --gltf-file-path <GLTF_FILE_PATH>
          Path to the GLTF file containing the scene to voxelize.
  -s, --voxel-size <VOXEL_SIZE>
          Voxel size (full box width). The smaller the voxel, the higher the grid resolution, 
          but at some point you will likely run out of memory.
  -b, --voxelization-bounds <VOXELIZATION_BOUNDS>
          Maximum voxelization bounds as an AABB (axis-aligned bounding box) in world space. 
          The actual voxel grid will be adapted to fit each GLTF model individually, but 
          will not exceed this bound. 
          The format is as follows: "(min_x, min_y, min_z) / (max_x, max_y, max_z)". 
          Example: "(-3, -2.5, -1) / (1, 1, 4.2)"
  -h, --help
          Print help
  -V, --version
          Print version
```

---

For exporting, use the `export` subcommand and specify the two required flags:

```md
Usage: nrg-mesh-to-volume-conversion
        --gltf-file-path <GLTF_FILE_PATH> 
        --voxel-size <VOXEL_SIZE> 
        export 
            --output-file-path <OUTPUT_FILE_PATH>
            --export-type <EXPORT_FORMAT>

Options:
      --output-file-path <OUTPUT_FILE_PATH>

      --export-type <EXPORT_FORMAT>
          One of: binary-edge_u1, binary-fill_u1, linear-rgb8-color_u8, metallic-value_u8, roughness-value_u8.

  -h, --help
          Print help
```

---

For visualization, use the `visualize` subcommand:

```md
Usage: nrg-mesh-to-volume-conversion
        --gltf-file-path <GLTF_FILE_PATH>
        --voxel-size <VOXEL_SIZE>
        visualize [OPTIONS]

Options:
      --visualization-voxel-size <VISUALIZATION_VOXEL_SIZE_RATIO>
          Voxel size (full box width) to use for the visualization of the voxelized scene. 
          Defaults to 1 (meaning the size is equal to --voxel-size), 
          but set it to e.g. 0.95 to create a an effect of a tiny edge around each voxel.

      --initial-camera-position <INITIAL_CAMERA_POSITION>
          Initial camera position. The camera will be oriented to look towards the center of the scene. 
          Defaults to an auto-calculated position based on the provided scene.

  -h, --help
          Print help
```


## Example
```bash
cargo run --release -- \
    --voxel-size 0.07 \
    --gltf-file-path ./assets/mesh-sample-01/mesh-sample-01_torus_textured_v2.gltf \
    --voxelization-bounds "(-10.0, -10.0, -10.0) / (10.0, 10.0, 10.0)" \
    export --export-type binary-edge_u1 --output-file-path ./exports/torus-three

cargo run --release -- \
    --voxel-size 0.07 \
    --gltf-file-path ./assets/mesh-sample-01/mesh-sample-01_torus_textured_v2.gltf \
    --voxelization-bounds "(-10.0, -10.0, -10.0) / (10.0, 10.0, 10.0)" \
    visualize
```
