use std::time::Duration;

use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::{
    ecs::{component::ComponentId, world::DeferredWorld},
    prelude::*,
};
use rand::{thread_rng, Rng};

use crate::lifetimes::DespawnAfter;

pub fn plugin(app: &mut App) {
    app.register_type::<Laser>();
    app.add_systems(Startup, setup);
    app.add_systems(Update, (shoot, move_lasers));
    app.add_systems(Update, laser_hit_detect);
}

#[derive(Resource, Reflect)]
struct LaserAssets {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Component, Reflect, Default)]
pub struct Laser;

#[derive(Component, Reflect)]
#[component(on_insert=gun_on_add)]
pub struct Gun {
    pub(crate) last_fired: f64,
}

fn gun_on_add(mut world: DeferredWorld, entity: Entity, _id: ComponentId) {
    let last_fired = world.resource::<Time>().elapsed_secs_f64();
    if let Some(mut entity) = world.get_mut::<Gun>(entity) {
        entity.last_fired = last_fired;
    }
}

impl Default for Gun {
    fn default() -> Self {
        Self {
            last_fired: (thread_rng().gen::<f64>() - 1.0) * 5.0,
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

fn laser_hit_detect(
    mut commands: Commands,
    lasers: Query<(Entity, &mut GlobalTransform), With<Laser>>,
    spatial_query: SpatialQuery,
) {
    lasers.iter().for_each(|(entity, transform)| {
        if let Some(first_hit) = spatial_query.cast_ray(
            transform.translation(),
            Dir3::NEG_Z,
            0.5,
            true,
            &SpatialQueryFilter::from_excluded_entities([entity]),
        ) {
            if let Some(e) = commands.get_entity(first_hit.entity) {
                e.try_despawn_recursive();
            }
        }
    });
}
