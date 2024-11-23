//! A minimal example that outputs "hello world"
mod capital_ships;
mod fps_overlay;
mod lasers;
mod lifetimes;
mod ships;
mod spawners;

use std::time::Duration;

use avian3d::PhysicsPlugins;
use bevy::{
    core_pipeline::bloom::Bloom,
    math::vec3,
    prelude::*,
    render::{
        settings::{PowerPreference, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use bevy_spatial::AutomaticUpdate;
use capital_ships::SpawnCapitalShip;
use fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use ships::*;

#[derive(Component, Default)]
struct TrackedByKDTree;

fn main() {
    color_backtrace::install();
    App::new()
        .add_plugins(EmbeddedAssetPlugin {
            mode: PluginMode::ReplaceDefault,
        })
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        focused: false,
                        title: "Space Battle".to_string(),
                        name: Some("bevy.space-battle".into()),
                        present_mode: bevy::window::PresentMode::AutoNoVsync,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
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
            PhysicsPlugins::default(),
            FpsOverlayPlugin::default(),
            // avian3d::prelude::PhysicsDebugPlugin::default(),
            ships::plugin,
            lasers::plugin,
            lifetimes::plugin,
            spawners::plugin,
            capital_ships::plugin,
            AutomaticUpdate::<TrackedByKDTree>::new()
                .with_frequency(Duration::from_secs_f32(0.2))
                .with_spatial_ds(bevy_spatial::SpatialStructure::KDTree3A),
        ))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
    asset_server: ResMut<AssetServer>,
    mut fps_overlay_config: ResMut<FpsOverlayConfig>,
) {
    fps_overlay_config.text_config = TextFont {
        font: asset_server.load("IBM Plex Mono/IBMPlexMono-Regular.ttf"),
        font_size: 16.,
        ..default()
    };
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

    let red_capital_ship_center = vec3(-80., -13., 35.);
    commands.queue(SpawnCapitalShip {
        transform: Transform::from_translation(red_capital_ship_center)
            .with_rotation(Quat::from_axis_angle(Vec3::Y, 180.0_f32.to_radians())),
        team: Team::Red,
    });
    let blue_capital_ship_center = vec3(80., 1., 0.);
    commands.queue(SpawnCapitalShip {
        transform: Transform::from_translation(blue_capital_ship_center),
        team: Team::Blue,
    });

    commands.spawn((
        TeamTarget(Team::Red),
        Transform::from_translation(blue_capital_ship_center),
    ));
    commands.spawn((
        TeamTarget(Team::Blue),
        Transform::from_translation(red_capital_ship_center),
    ));
}
