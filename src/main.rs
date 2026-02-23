use bevy::{
    asset::io::embedded::GetAssetServer,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    gltf::{GltfLoaderSettings, GltfPlugin, convert_coordinates::GltfConvertCoordinates},
    image::{ImageAddressMode, ImageLoaderSettings, ImageSamplerDescriptor},
    pbr::{Atmosphere, ExtendedMaterial, MaterialExtension, MeshMaterial3d, ScatteringMedium},
    post_process::bloom::Bloom,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    scene::SceneInstanceReady,
    shader::ShaderRef,
    utils::default,
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use bevy_layered_materials::{LayeredMaterial, LayeredMaterialsPlugin};

static SHADER_ASSET_PATH: &str = "shader.wgsl";

#[derive(Asset, Clone, Reflect, AsBindGroup)]
#[data(50, GpuBlendedPbr, binding_array(101))]
#[bindless(index_table(range(50..53), binding(100)))]
struct BlendedPbr {
    #[texture(51)]
    #[sampler(52)]
    mask: Option<Handle<Image>>,
}

#[derive(Clone, Default, ShaderType)]
struct GpuBlendedPbr {
    _unused: f32,
}

impl MaterialExtension for BlendedPbr {
    fn fragment_shader() -> ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

impl<'a> From<&'a BlendedPbr> for GpuBlendedPbr {
    fn from(_material_extension: &'a BlendedPbr) -> Self {
        GpuBlendedPbr { _unused: 0.0 }
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
        .add_plugins(LayeredMaterialsPlugin)
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<LayeredMaterial, BlendedPbr>,
        >::default())
        .add_systems(Update, load_assets)
        .add_systems(Update, fuck)
        .init_asset::<AppAssets>()
        .add_observer(process_assets)
        .add_observer(setup_camera)
        .add_observer(setup_directional_light)
        .insert_resource(GlobalAmbientLight::NONE)
        .run();
}

#[derive(Resource, Asset, Clone, Reflect)]
struct AppAssets {
    #[dependency]
    level: Handle<Gltf>,
    #[dependency]
    wear_mask: Handle<Image>,
    #[dependency]
    base_color_texture: Handle<Image>,
    #[dependency]
    normal_map_texture: Handle<Image>,
    #[dependency]
    arm_texture: Handle<Image>,
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
                    "textures/wear_mask.ktx2",
                    |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
                )
                .into(),
            base_color_texture: assets
                .load_with_settings(
                    "textures/base_color.ktx2",
                    |settings: &mut ImageLoaderSettings| settings.is_srgb = true,
                )
                .into(),
            normal_map_texture: assets
                .load_with_settings(
                    "textures/normal.ktx2",
                    |settings: &mut ImageLoaderSettings| settings.is_srgb = false,
                )
                .into(),
            arm_texture: assets
                .load_with_settings("textures/arm.ktx2", |settings: &mut ImageLoaderSettings| {
                    settings.is_srgb = false
                })
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
                                base: LayeredMaterial {
                                    base_color_texture: Some(
                                        assets.load("textures/base_color.ktx2"),
                                    ),
                                    normal_map_texture: Some(assets.load("textures/normal.ktx2")),
                                    metallic_roughness_texture: Some(
                                        assets.load("textures/arm.ktx2"),
                                    ),
                                    occlusion_texture: Some(assets.load("textures/arm.ktx2")),
                                    perceptual_roughness: 1.0,
                                    metallic: 1.0,
                                    ..default()
                                },
                                extension: BlendedPbr {
                                    mask: Some(app_assets.wear_mask.clone()),
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
        PointLight {
            shadows_enabled: true,
            intensity: 100_000.0,
            ..default()
        },
    ));
}

fn setup_directional_light(
    add: On<Add, DirectionalLight>,
    mut directional_lights: Query<&mut DirectionalLight>,
) {
    let Ok(mut directional_light) = directional_lights.get_mut(add.entity) else {
        return;
    };
    directional_light.shadows_enabled = true;
}
