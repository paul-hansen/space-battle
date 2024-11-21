use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<Boid>();
    app.register_type::<BoidGroup>();
    app.add_systems(Update, (move_boids, rotate_towards_target));
}

#[derive(Reflect, Component, Default)]
pub struct Boid;

#[derive(Component, Reflect, Default)]
#[require(Boid)]
pub enum BoidGroup {
    #[default]
    Red,
    Blue,
    Green,
    Yellow,
}

#[derive(Component, Reflect, Default)]
pub struct Target;

pub fn move_boids(mut boids: Query<&mut Transform, With<Boid>>, time: Res<Time>) {
    for mut transform in boids.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * 15.0 * time.delta_secs();
    }
}

pub fn rotate_towards_target(mut boids: Query<(&mut Transform, &GlobalTransform), With<Boid>>,  target: Query<&GlobalTransform, With<Target>>, time: Res<Time>) {
    let Ok(target)=target.get_single() else {
        return;
    };

    for (mut transform, global_transform) in boids.iter_mut() {
        let look = global_transform.compute_transform().looking_at(target.translation(), Vec3::Y).rotation;
        transform.rotation = transform.rotation.rotate_towards(look, 20.0_f32.to_radians() * time.delta_secs());
    }
}
