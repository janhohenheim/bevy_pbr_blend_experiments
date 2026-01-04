use std::f32::consts::FRAC_PI_2;

use bevy::{
    mesh::{SphereKind, SphereMeshBuilder},
    pbr::{ExtendedMaterial, MaterialExtension, MeshMaterial3d},
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderType},
    shader::ShaderRef,
    utils::default,
};

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
        .add_plugins(DefaultPlugins)
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, BlendedPbr>,
        >::default())
        .add_systems(Startup, setup)
        .add_systems(Update, rotate_sphere)
        .run();
}

/// Creates the scene.
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, BlendedPbr>>>,
) {
    // Create a gray sphere, modulated with a red-tinted Bevy logo.
    commands.spawn((
        Mesh3d(meshes.add(SphereMeshBuilder::new(
            1.0,
            SphereKind::Uv {
                sectors: 20,
                stacks: 20,
            },
        ))),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: Color::WHITE,
                ..default()
            },
            extension: BlendedPbr {
                strength: 0.75,
                mask: Some(asset_server.load("galvanic.jpg")),
                blend_a: Some(asset_server.load("uv_checker_bw.png")),
                blend_b: Some(asset_server.load("cobblestone_pavement_diff_2k.jpg")),
            },
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(1.0, 1.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn rotate_sphere(mut meshes: Query<&mut Transform, With<Mesh3d>>, time: Res<Time>) {
    for mut transform in &mut meshes {
        transform.rotation =
            Quat::from_euler(EulerRot::YXZ, -time.elapsed_secs(), FRAC_PI_2 * 3.0, 0.0);
    }
}
