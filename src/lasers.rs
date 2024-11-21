use bevy::prelude::*;

use crate::Boid;

pub fn plugin(app: &mut App) {
    app.register_type::<Laser>();
    app.add_systems(Update, (shoot, move_lasers));
}

#[derive(Component, Reflect, Default)]
pub struct Laser;

#[derive(Component, Reflect, Default)]
pub struct Gun {
    last_fired: f64,
}

pub fn move_lasers(mut lasers: Query<&mut Transform, With<Laser>>, time: Res<Time>) {
    for mut transform in lasers.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * 15.0 * time.delta_secs();
    }
}

pub fn shoot(mut commands: Commands, mut guns: Query<(&GlobalTransform, &mut Gun)>, time: Res<Time>) {
    for (transform, mut gun) in guns.iter_mut() {
        let now = time.elapsed_secs_f64();
        if gun.last_fired < now {
            gun.last_fired = now;
            commands.spawn((Laser, Transform::default(), Visibility::default()));
        }
    }
}
