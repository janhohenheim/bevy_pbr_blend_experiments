use std::f32::consts::FRAC_PI_2;

use bevy::{
    asset::io::embedded::GetAssetServer,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    mesh::{SphereKind, SphereMeshBuilder},
    pbr::{ExtendedMaterial, MaterialExtension, MeshMaterial3d},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    scene::SceneInstanceReady,
    shader::ShaderRef,
    utils::default,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

static SHADER_ASSET_PATH: &str = "shader.wgsl";

#[derive(Asset, Clone, Reflect, AsBindGroup)]
#[data(50, GpuBlendedPbr, binding_array(101))]
#[bindless(index_table(range(50..57), binding(100)))]
struct BlendedPbr {
    strength: f32,

    #[texture(51)]
    #[sampler(52)]
    mask: Option<Handle<Image>>,

    #[texture(53)]
    #[sampler(54)]
    blend_a: Option<Handle<Image>>,

    #[texture(55)]
    #[sampler(56)]
    blend_b: Option<Handle<Image>>,
}

#[derive(Clone, Default, ShaderType)]
struct GpuBlendedPbr {
    strength: f32,
}

impl MaterialExtension for BlendedPbr {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

impl<'a> From<&'a BlendedPbr> for GpuBlendedPbr {
    fn from(material_extension: &'a BlendedPbr) -> Self {
        GpuBlendedPbr {
            strength: material_extension.strength,
        }
    }
}

/// The entry point.
fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FreeCameraPlugin))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, BlendedPbr>,
        >::default())
        .add_systems(Update, load_assets)
        .add_systems(Update, fuck)
        .add_systems(Startup, setup)
        .init_asset::<AppAssets>()
        .add_observer(process_assets)
        .run();
}

#[derive(Resource, Asset, Clone, Reflect)]
struct AppAssets {
    #[dependency]
    level: Handle<Gltf>,
    #[dependency]
    brick_material: Handle<StandardMaterial>,
    #[dependency]
    render_material: Handle<StandardMaterial>,
}

impl FromWorld for AppAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_asset_server();
        Self {
            level: assets.load("models/test.gltf"),

            brick_material: assets.add(StandardMaterial {
                base_color_texture: assets.load("models/brick_basecolor.png").into(),
                normal_map_texture: assets.load("models/brick_normal.png").into(),
                occlusion_texture: assets.load("models/brick_arm.png").into(),
                metallic_roughness_texture: assets.load("models/brick_arm.png").into(),
                metallic: 1.0,
                perceptual_roughness: 1.0,
                ..default()
            }),
            render_material: assets.add(StandardMaterial {
                base_color_texture: assets.load("models/render_basecolor.png").into(),
                normal_map_texture: assets.load("models/render_normal.png").into(),
                occlusion_texture: assets.load("models/render_arm.png").into(),
                metallic_roughness_texture: assets.load("models/render_arm.png").into(),
                metallic: 1.0,
                perceptual_roughness: 1.0,
                ..default()
            }),
        }
    }
}

fn fuck(mut asset_events: MessageReader<AssetEvent<AppAssets>>) {
    for event in asset_events.read() {
        info!(?event);
    }
}

fn load_assets(
    world: &mut World,
    mut loaded: Local<bool>,
    mut app_assets_handle: Local<Option<Handle<AppAssets>>>,
) {
    if *loaded {
        return;
    }
    let app_assets_handle = app_assets_handle.get_or_insert_with(|| {
        let asset = AppAssets::from_world(world);
        world.get_asset_server().add(asset)
    });
    info!(state=?world.get_asset_server().get_recursive_dependency_load_state(&app_assets_handle.clone()));
    if world
        .get_asset_server()
        .is_loaded_with_dependencies(&app_assets_handle.clone().untyped())
    {
        let app_assets = world
            .resource::<Assets<AppAssets>>()
            .get(app_assets_handle)
            .unwrap();
        world.insert_resource(app_assets.clone());
        world.trigger(AppAssetsDone);
        *loaded = true;
    }
}

#[derive(Event)]
struct AppAssetsDone;

fn process_assets(
    _done: On<AppAssetsDone>,
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    app_assets: Res<AppAssets>,
    std_mats: Res<Assets<StandardMaterial>>,
) {
    let gltf = gltfs.get(&app_assets.level).unwrap();
    let brick = std_mats.get(&app_assets.brick_material).unwrap().clone();
    let render = std_mats.get(&app_assets.render_material).unwrap().clone();

    commands
        .spawn(SceneRoot(gltf.default_scene.clone().unwrap()))
        .observe(
            move |ready: On<SceneInstanceReady>,
                  mut commands: Commands,
                  assets: Res<AssetServer>,
                  children: Query<&Children>,
                  names: Query<(Entity, &Name)>| {
                for (entity, name) in names.iter_many(children.iter_descendants(ready.entity)) {
                    if name.to_lowercase().ends_with("img_render_basecolor.png") {
                        commands
                            .entity(entity)
                            .remove::<MeshMaterial3d<StandardMaterial>>()
                            .insert(MeshMaterial3d(assets.add(ExtendedMaterial {
                                base: StandardMaterial {
                                    base_color: Color::WHITE,
                                    ..default()
                                },
                                extension: BlendedPbr {
                                    strength: 0.75,
                                    mask: Some(assets.load("galvanic.jpg")),
                                    blend_a: brick.base_color_texture.clone(),
                                    blend_b: render.base_color_texture.clone(),
                                },
                            })));
                    }
                }
            },
        );

    /*

    */
}

fn setup(mut commands: Commands) {
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        FreeCamera::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
