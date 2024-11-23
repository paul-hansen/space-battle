use std::time::Duration;

use crate::{spawners::Spawner, ShipAssets, Team};
use bevy::{
    animation::{AnimationTarget, AnimationTargetId}, prelude::*, render::mesh::CylinderMeshBuilder, utils::HashMap,
};
use glam::vec3;

pub fn plugin(app: &mut App) {
    app.add_systems(PreStartup, setup);
}
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    let mesh = meshes.add(CylinderMeshBuilder {
        resolution: 6,
        segments: 1,
        caps: true,
        ..default()
    });
    let mut meshes = HashMap::new();
    meshes.insert(Team::Red, mesh.clone());
    meshes.insert(Team::Blue, mesh);
    commands.insert_resource(CapitalShipAssets { meshes });
}
pub struct SpawnCapitalShip {
    pub transform: Transform,
    pub team: Team,
}

#[derive(Resource, Clone)]
pub struct CapitalShipAssets {
    meshes: HashMap<Team, Handle<Mesh>>,
}

const LENGTH: f32 = 80.0;

impl Command for SpawnCapitalShip {
    fn apply(self, world: &mut World) {
        let capital_ship_assets = world.resource::<CapitalShipAssets>().clone();
        let ship_assets = world.resource::<ShipAssets>().clone();
        let root_name = Name::new(format!("Capital Ship {:?}", self.team));
        let (graph, animation_index, target) = {
            let mut animation = AnimationClip::default();
            let target =
                AnimationTargetId::from_name(&root_name);

            animation.add_curve_to_target(
                target,
                UnevenSampleAutoCurve::new([
                    (0.3, self.transform.translation - (self.transform.forward() * -2000.0)),
                    (1.2, self.transform.translation)
                ])
                    .map(TranslationCurve)
                    .expect("animation curve samples should be valid"),
            );
            let mut animations = world.resource_mut::<Assets<AnimationClip>>();
            let (graph, index) = AnimationGraph::from_clip(animations.add(animation));
            let mut graphs = world.resource_mut::<Assets<AnimationGraph>>();
            let graph = graphs.add(graph);
            (graph, index, target)
        };
        let mut player = AnimationPlayer::default();
        player.play(animation_index);

        let root = world.spawn_empty().id();
        world.entity_mut(root)
            .insert((
                self.transform,
                Visibility::default(),
                AnimationGraphHandle(graph),
                player,
                AnimationTarget {
                    id: target,
                    player: root,
                },
            ))
            .with_children(|child_builder| {
                child_builder.spawn((
                    Mesh3d(
                        capital_ship_assets
                            .meshes
                            .get(&self.team)
                            .expect("capital ship assets are missing")
                            .clone(),
                    ),
                    Transform {
                        scale: Vec3::new(10., LENGTH, 10.),
                        rotation: Quat::from_axis_angle(Vec3::X, 90.0_f32.to_radians()),
                        ..default()
                    },
                    MeshMaterial3d(ship_assets.materials.get(&self.team).unwrap().clone()),
                ));
                // Wings
                for side in [-2., -1., 1., 2.] {
                    child_builder.spawn((
                        Mesh3d(
                            capital_ship_assets
                                .meshes
                                .get(&self.team)
                                .expect("capital ship assets are missing")
                                .clone(),
                        ),
                        Transform {
                            translation: Vec3::new(5. * side, 0., -LENGTH * 0.15 * side.abs()),
                            scale: Vec3::new(10., LENGTH * 0.6, 10.),
                            rotation: Quat::from_axis_angle(Vec3::X, 90.0_f32.to_radians()),
                        },
                        MeshMaterial3d(ship_assets.materials.get(&self.team).unwrap().clone()),
                    ));
                }
                // Fighter Spawners
                for y in [-0.75, 0.75] {
                    for z in -2..=4 {
                        let z = z as f32;
                        for side in [-1., 1.] {
                            child_builder.spawn((
                                Spawner {
                                    max: Some(200),
                                    delay: Duration::from_secs_f32(0.2),
                                    team: self.team,
                                    last_spawn: None,
                                    spawned: 0,
                                },
                                Transform::from_translation(vec3(
                                    4. * side,
                                    y * 5.,
                                    (4.0 * z) - LENGTH * 0.30,
                                ))
                                .with_rotation(
                                    Quat::from_rotation_y(-side * 90.0_f32.to_radians()),
                                ),
                            ));
                        }
                    }
                }
            });
    }
}
