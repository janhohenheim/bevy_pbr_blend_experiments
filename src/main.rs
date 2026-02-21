use std::f32::consts::FRAC_PI_2;

use bevy::{
    asset::io::embedded::GetAssetServer,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    gltf::{GltfLoaderSettings, GltfPlugin, convert_coordinates::GltfConvertCoordinates},
    image::{ImageAddressMode, ImageLoaderSettings, ImageSamplerDescriptor},
    mesh::{SphereKind, SphereMeshBuilder},
    pbr::{Atmosphere, ExtendedMaterial, MaterialExtension, MeshMaterial3d, ScatteringMedium},
    post_process::bloom::Bloom,
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
#[bindless(index_table(range(50..55), binding(100)))]
struct BlendedPbr {
    strength: f32,

    #[texture(51)]
    #[sampler(52)]
    mask: Option<Handle<Image>>,

    #[texture(53, dimension = "2d_array")]
    #[sampler(54)]
    base_color_texture: Option<Handle<Image>>,
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
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin {
                    default_sampler: ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        address_mode_w: ImageAddressMode::Repeat,
                        anisotropy_clamp: 16,
                        ..ImageSamplerDescriptor::linear()
                    },
                })
                .set(GltfPlugin {
                    default_sampler: ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        address_mode_w: ImageAddressMode::Repeat,
                        anisotropy_clamp: 16,
                        ..ImageSamplerDescriptor::linear()
                    },
                    convert_coordinates: GltfConvertCoordinates {
                        rotate_scene_entity: true,
                        rotate_meshes: true,
                    },
                    ..default()
                }),
            FreeCameraPlugin,
        ))
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, BlendedPbr>,
        >::default())
        .add_systems(Update, load_assets)
        .add_systems(Update, fuck)
        .init_asset::<AppAssets>()
        .add_observer(process_assets)
        .add_observer(setup_camera)
        .insert_resource(GlobalAmbientLight::NONE)
        .run();
}

#[derive(Resource, Asset, Clone, Reflect)]
struct AppAssets {
    #[dependency]
    level: Handle<Gltf>,
    #[dependency]
    wear_mask: Handle<Image>,
}

impl FromWorld for AppAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.get_asset_server();
        Self {
            level: assets.load_with_settings(
                "models/test.gltf",
                |settings: &mut GltfLoaderSettings| {
                    settings.load_lights = true;
                    settings.load_cameras = true;
                },
            ),
            wear_mask: assets
                .load_with_settings(
                    "textures/raw/wear_mask.png",
                    |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
                )
                .into(),
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
) {
    let gltf = gltfs.get(&app_assets.level).unwrap();

    commands
        .spawn(SceneRoot(gltf.default_scene.clone().unwrap()))
        .observe(
            move |ready: On<SceneInstanceReady>,
                  mut commands: Commands,
                  assets: Res<AssetServer>,
                  children: Query<&Children>,
                  names: Query<(Entity, &Name)>,
                  app_assets: Res<AppAssets>| {
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
                                    mask: Some(app_assets.wear_mask.clone()),
                                    base_color_texture: Some(
                                        assets.load("textures/base_color.ktx2"),
                                    ),
                                },
                            })));
                    }
                }
            },
        );

    /*

    */
}

fn setup_camera(
    add: On<Add, Camera3d>,
    mut commands: Commands,
    mut scatter_media: ResMut<Assets<ScatteringMedium>>,
) {
    commands.entity(add.entity).insert((
        FreeCamera::default(),
        Bloom::NATURAL,
        Atmosphere::earthlike(scatter_media.add(ScatteringMedium::default())),
    ));
}
