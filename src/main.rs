//! A minimal example that outputs "hello world"
mod lasers;
mod lifetimes;
mod ships;
mod spawners;

use std::time::Duration;

use bevy::{
    core_pipeline::bloom::Bloom,
    math::vec3,
    prelude::*,
    render::{
        settings::{PowerPreference, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_dev_tools::fps_overlay::FpsOverlayPlugin;
use ships::*;
use spawners::Spawner;

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
            spawners::plugin,
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

    let red_ship_center = vec3(-50., 0., 0.);
    let blue_ship_center = vec3(50., 0., 0.);
    for y in 0..=1 {
        for z in 0..=11 {
            let z = z as f32;
            let y = y as f32;
            commands.spawn((
                Spawner {
                    max: Some(100),
                    delay: Duration::from_secs_f32(0.1),
                    team: Team::Red,
                    last_spawn: None,
                    spawned: 0,
                },
                Transform::from_translation(red_ship_center + vec3(0., y * 5., 4.0 * z))
                    .with_rotation(Quat::from_rotation_y(-90.0_f32.to_radians())),
            ));
        }
    }
    for y in 0..=1 {
        for z in 0..=11 {
            let z = z as f32;
            let y = y as f32;
            commands.spawn((
                Spawner {
                    max: Some(100),
                    delay: Duration::from_secs_f32(0.1),
                    team: Team::Blue,
                    last_spawn: None,
                    spawned: 0,
                },
                Transform::from_translation(blue_ship_center + vec3(0., y * 5., 4.0 * z))
                    .with_rotation(Quat::from_rotation_y(90.0_f32.to_radians())),
            ));
        }
    }

    commands.spawn((
        Target(Team::Red),
        Transform::from_translation(blue_ship_center),
    ));
    commands.spawn((
        Target(Team::Blue),
        Transform::from_translation(red_ship_center),
    ));
}
