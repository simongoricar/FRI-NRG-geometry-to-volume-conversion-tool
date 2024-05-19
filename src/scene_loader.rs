//! Partially adapted from bevy's
//! <https://github.com/bevyengine/bevy/tree/main/examples/tools/scene_viewer>.

use bevy::{asset::LoadState, gltf::Gltf, prelude::*, scene::InstanceId};


#[derive(Resource)]
pub struct SceneHandle {
    pub gltf_handle: Handle<Gltf>,

    scene_index: usize,

    instance_id: Option<InstanceId>,

    pub is_loaded: bool,

    pub has_light: bool,
}

impl SceneHandle {
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
        app.add_systems(PreUpdate, scene_load_check);
    }
}

fn scene_load_check(
    asset_server: Res<AssetServer>,
    mut scenes: ResMut<Assets<Scene>>,
    gltf_assets: Res<Assets<Gltf>>,
    mut scene_handle: ResMut<SceneHandle>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if let Some(scene_instance_id) = scene_handle.instance_id {
        if !scene_handle.is_loaded && scene_spawner.instance_is_ready(scene_instance_id) {
            info!("GLTF scene is ready!");
            scene_handle.is_loaded = true;
        }

        return;
    }

    if asset_server.load_state(&scene_handle.gltf_handle) != LoadState::Loaded {
        return;
    }

    info!("Scene has been loaded.");


    let gltf_data = gltf_assets
        .get(&scene_handle.gltf_handle)
        .expect("failed to obtain reference to GLTF scene from assert handle");

    if gltf_data.scenes.len() > 1 {
        warn!("The provided GLTF file contains more than one scene; we'll load the first one only.");
    }


    let gltf_scene_handle = gltf_data
        .scenes
        .get(scene_handle.scene_index)
        .unwrap_or_else(|| {
            panic!(
                "glTF file doesn't contain scene {}!",
                scene_handle.scene_index
            )
        });

    let gltf_scene = scenes.get_mut(gltf_scene_handle).unwrap();


    let mut query = gltf_scene
        .world
        .query::<(Option<&DirectionalLight>, Option<&PointLight>)>();

    scene_handle.has_light =
        query
            .iter(&gltf_scene.world)
            .any(|(maybe_directional_light, maybe_point_light)| {
                maybe_directional_light.is_some() || maybe_point_light.is_some()
            });

    if scene_handle.has_light {
        info!("Scene already has light.");
    } else {
        info!("Scene does not have a light.");
    }

    info!("Spawning scene.");
    scene_handle.instance_id = Some(scene_spawner.spawn(gltf_scene_handle.clone_weak()));
}
