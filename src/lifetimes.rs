use std::time::Duration;

use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<DespawnAfter>();
    app.add_systems(Update, despawn_after);
}

#[derive(Component, Reflect)]
pub struct DespawnAfter{
    despawn_at: f64,
}

impl DespawnAfter{
    pub fn new(duration: Duration, time: &Time) -> Self {
        Self{
            despawn_at: time.elapsed_secs_f64() + duration.as_secs_f64(),
        }
    }
}

fn despawn_after(mut commands: Commands, query: Query<(Entity, &DespawnAfter)>, time: Res<Time>) {
    for (entity, DespawnAfter{despawn_at}) in query.iter(){
        if time.elapsed_secs_f64() > *despawn_at{
            commands.entity(entity).despawn_recursive();
        }
    }
}
