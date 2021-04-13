use bevy::{asset::AssetPath, prelude::*};
use bevy_editor_pls::{extensions::EditorExtensionSpawn, EditorPlugin, EditorSettings};

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource({
            let mut settings = EditorSettings::new();
            settings.auto_flycam = true;
            settings.auto_pickable = true;
            settings.click_to_inspect = true;

            settings.on_file_drop(&["gltf", "glb"], |path, world| {
                let asset_path = AssetPath::new_ref(path, Some("Scene0".into()));
                let asset_server = world.get_resource_mut::<AssetServer>().unwrap();
                let scene_handle = asset_server.load(asset_path);

                let mut spawner = world.get_resource_mut::<SceneSpawner>().unwrap();
                spawner.spawn(scene_handle);
            });

            settings
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(EditorPlugin)
        .add_plugin(EditorExtensionSpawn)
        .add_startup_system(setup.system())
        .run();
}

pub fn setup(mut commands: Commands, mut _meshes: ResMut<Assets<Mesh>>, mut _materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
