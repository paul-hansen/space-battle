//! A minimal example that outputs "hello world"
mod boids;
mod lasers;

use bevy::{
    core_pipeline::bloom::Bloom,
    math::vec3,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        mesh::ConeMeshBuilder,
        settings::{PowerPreference, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_dev_tools::fps_overlay::FpsOverlayPlugin;
use boids::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        focused: false,
                        title: "Space Battle".to_string(),
                        name: Some("bevy.space-battle".into()),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        power_preference: PowerPreference::HighPerformance,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            FpsOverlayPlugin::default(),
            boids::plugin,
            lasers::plugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    ambient_light.brightness = 0.5;
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(15000.0, 15000.0, 10000.0),
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Target,
    ));

    // boids
    let mut rng = rand::thread_rng();
    let space = 80.;
    let mat = materials.add(StandardMaterial {
        base_color: Color::srgb_u8(124, 144, 255),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });
    let mut cone_mesh = ConeMeshBuilder::new(0.5, 2.0, 32).build();
    cone_mesh.transform_by(Transform::from_rotation(Quat::from_rotation_x(
        -90.0_f32.to_radians(),
    )));
    let mut left_cone = cone_mesh.clone();
    left_cone.transform_by(
        Transform::from_translation(vec3(0.7, 0.0, 0.0)).with_scale(Vec3::splat(0.6)),
    );
    let mut right_cone = cone_mesh.clone();
    right_cone.transform_by(
        Transform::from_translation(vec3(-0.7, 0.0, 0.0)).with_scale(Vec3::splat(0.6)),
    );
    cone_mesh.merge(&left_cone);
    cone_mesh.merge(&right_cone);
    let cone = meshes.add(cone_mesh);
    for _ in 0..=10000 {
        commands.spawn((
            MeshMaterial3d(mat.clone()),
            Mesh3d(cone.clone()),
            Transform::from_translation(vec3(
                (rng.gen::<f32>() * space) - space / 2.,
                (rng.gen::<f32>() * space) - space / 2.,
                (rng.gen::<f32>() * space) - space / 2.,
            )),
            Visibility::Visible,
            BoidGroup::Red,
            NotShadowReceiver,
            NotShadowCaster,
        ));
    }
    // light
    commands.spawn((
        PointLight {
            intensity: 60_200_000.,
            range: 3600.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Bloom::NATURAL,
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..Camera::default()
        },
        Transform::from_xyz(125.0, 45., 85.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
