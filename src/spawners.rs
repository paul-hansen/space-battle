use std::time::Duration;

use crate::{SpawnShip, Team};
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<Spawner>();
    app.add_systems(Update, spawn);
}

#[derive(Component, Reflect, Default)]
pub struct Spawner {
    pub max: Option<usize>,
    pub delay: Duration,
    pub team: Team,
    pub last_spawn: Option<f64>,
    pub spawned: usize,
}

fn spawn(
    mut commands: Commands,
    mut query: Query<(&GlobalTransform, &mut Spawner)>,
    time: Res<Time>,
) {
    for (transform, mut spawner) in query.iter_mut() {
        if let Some(max) = spawner.max{
            if spawner.spawned > max{
                continue;
            }
        }
        if time.elapsed_secs_f64()
            > spawner.last_spawn.unwrap_or_default() + spawner.delay.as_secs_f64()
        {
            spawner.last_spawn = time.elapsed_secs_f64().into();
            spawner.spawned += 1;
            commands.queue(SpawnShip {
                transform: transform.compute_transform(),
                team: spawner.team,
            });
        }
    }
}
