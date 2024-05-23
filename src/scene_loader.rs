//! Partially adapted from bevy's
//! <https://github.com/bevyengine/bevy/tree/main/examples/tools/scene_viewer>.

use bevy::{asset::LoadState, gltf::Gltf, prelude::*, scene::InstanceId};



#[derive(Resource)]
pub struct GltfSceneHandle {
    pub gltf_handle: Handle<Gltf>,

    scene_index: usize,

    instance_id: Option<InstanceId>,

    pub is_loaded: bool,

    pub has_light: bool,
}

impl GltfSceneHandle {
    pub fn new(gltf_handle: Handle<Gltf>, scene_index: usize) -> Self {
        Self {
            gltf_handle,
            scene_index,
            instance_id: None,
            is_loaded: false,
            has_light: false,
        }
    }
}



pub struct GltfSceneLoaderPlugin;

impl Plugin for GltfSceneLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, handle_gltf_scene_load);
    }
}


/// This system checks if the GLTF scene contained in the [`SceneHandle`] resource
/// has been loaded. Once it has, this system computes some secondary properties,
/// and finally spawns the scene into the world.
///
/// Once the scene is loaded and spawned, [`SceneHandle::instance_id`] is no longer None,
/// and [`SceneHandle::is_loaded`] is set to `true`.
fn handle_gltf_scene_load(
    asset_server: Res<AssetServer>,
    mut scene_assets: ResMut<Assets<Scene>>,
    gltf_assets: Res<Assets<Gltf>>,
    mut gltf_scene_resource: ResMut<GltfSceneHandle>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if let Some(scene_instance_id) = gltf_scene_resource.instance_id {
        if !gltf_scene_resource.is_loaded && scene_spawner.instance_is_ready(scene_instance_id) {
            info!("GLTF scene is ready!");
            gltf_scene_resource.is_loaded = true;
        }

        return;
    }

    if asset_server.load_state(&gltf_scene_resource.gltf_handle) != LoadState::Loaded {
        return;
    }

    info!("Scene has been loaded.");


    let gltf_data = gltf_assets
        .get(&gltf_scene_resource.gltf_handle)
        .expect("failed to obtain reference to GLTF scene from assert handle");

    if gltf_data.scenes.len() > 1 {
        warn!("The provided GLTF file contains more than one scene; we'll load the first one only.");
    }


    let gltf_scene_handle = gltf_data
        .scenes
        .get(gltf_scene_resource.scene_index)
        .unwrap_or_else(|| {
            panic!(
                "glTF file doesn't contain scene {}!",
                gltf_scene_resource.scene_index
            )
        });

    let gltf_scene = scene_assets.get_mut(gltf_scene_handle).unwrap();


    let mut query = gltf_scene
        .world
        .query::<(Option<&DirectionalLight>, Option<&PointLight>)>();

    gltf_scene_resource.has_light =
        query
            .iter(&gltf_scene.world)
            .any(|(maybe_directional_light, maybe_point_light)| {
                maybe_directional_light.is_some() || maybe_point_light.is_some()
            });

    if gltf_scene_resource.has_light {
        info!("Scene already has light.");
    } else {
        info!("Scene does not have a light.");
    }

    info!("Spawning scene.");
    gltf_scene_resource.instance_id = Some(scene_spawner.spawn(gltf_scene_handle.clone_weak()));
}
