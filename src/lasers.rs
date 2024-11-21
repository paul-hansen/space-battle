use std::time::Duration;

use bevy::prelude::*;
use rand::{thread_rng, Rng};

use crate::lifetimes::DespawnAfter;

pub fn plugin(app: &mut App) {
    app.register_type::<Laser>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, (shoot, move_lasers));
}

#[derive(Resource, Reflect)]
struct LaserAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Reflect, Default)]
pub struct Laser;

#[derive(Component, Reflect)]
pub struct Gun {
    pub(crate) last_fired: f64,
}
impl Default for Gun{
    fn default() -> Self {
        Self{
            last_fired: (thread_rng().gen::<f64>() - 1.0) * 5.0
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut mesh = Capsule3d::new(0.01, 0.8).mesh().build();
    mesh.transform_by(Transform::from_rotation(Quat::from_rotation_x(
        90.0_f32.to_radians(),
    )));
    let mesh = meshes.add(mesh);
    let material = materials.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0., 0.),
        emissive: LinearRgba::rgb(100.0, 1.0, 1.0),
        ..default()
    });
    commands.insert_resource(LaserAssets { mesh, material });
}

fn move_lasers(mut lasers: Query<&mut Transform, With<Laser>>, time: Res<Time>) {
    for mut transform in lasers.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * 35.0 * time.delta_secs();
    }
}

fn shoot(
    mut commands: Commands,
    mut guns: Query<(&GlobalTransform, &mut Gun)>,
    laser_assets: Res<LaserAssets>,
    time: Res<Time>,
) {
    for (transform, mut gun) in guns.iter_mut() {
        let now = time.elapsed_secs_f64();
        if gun.last_fired + 5.0 < now {
            gun.last_fired = now;
            commands.spawn((
                Laser,
                Mesh3d(laser_assets.mesh.clone()),
                MeshMaterial3d(laser_assets.material.clone()),
                DespawnAfter::new(Duration::from_secs(2), &time),
                transform.compute_transform(),
                *transform,
                Visibility::default(),
            ));
        }
    }
}
