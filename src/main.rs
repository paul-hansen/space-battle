//! A minimal example that outputs "hello world"
mod lasers;
mod lifetimes;
mod ships;

use bevy::{
    core_pipeline::bloom::Bloom,
    math::vec3,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::{
        settings::{PowerPreference, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_dev_tools::fps_overlay::FpsOverlayPlugin;
use lasers::Gun;
use rand::Rng;
use ships::*;

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
            ships::plugin,
            lasers::plugin,
            lifetimes::plugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    ship_assets: ResMut<ShipAssets>,
) {
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
    ambient_light.brightness = 10.5;
    // star
    commands.spawn((
        PointLight {
            intensity: 600_200_000.,
            range: 56000.,
            shadows_enabled: true,
            ..default()
        },
        Mesh3d(meshes.add(Sphere::new(4.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            emissive: LinearRgba::rgb(15000.0, 15000.0, 10000.0),
            ..default()
        })),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_translation(vec3(-200., 50., -100.)),
    ));

    commands.spawn((Target(Team::Red), Transform::from_translation(vec3(-30.,0.,0.))));
    commands.spawn((Target(Team::Blue), Transform::from_translation(vec3(30.,0.,0.))));

    // ships
    let mut rng = rand::thread_rng();
    let space = 80.;
    for _ in 0..=10000 {
        let team =
            Team::try_from(rng.gen_range(0..=1)).expect("group number should have been in range");

        commands.spawn((
            MeshMaterial3d(
                ship_assets
                    .materials
                    .get(&team)
                    .expect("ship_assets should be initialized")
                    .clone(),
            ),
            Mesh3d(
                ship_assets
                    .mesh
                    .get(&team)
                    .expect("ship_assets should be initialized")
                    .clone(),
            ),
            Transform::from_translation(vec3(
                (rng.gen::<f32>() * space) - space / 2.,
                (rng.gen::<f32>() * space) - space / 2.,
                (rng.gen::<f32>() * space) - space / 2.,
            )),
            Visibility::Visible,
            team,
            Gun {
                last_fired: (rng.gen::<f64>() - 1.0) * 5.0,
            },
            NotShadowReceiver,
            NotShadowCaster,
        ));
    }
}
