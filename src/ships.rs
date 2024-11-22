use std::time::Duration;

use bevy::{
    math::vec3, prelude::*, render::mesh::ConeMeshBuilder, time::common_conditions::on_timer,
    utils::HashMap,
};
use bevy_spatial::{kdtree::KDTree3A, SpatialAccess};

use crate::{
    lasers::{Gun, Laser},
    TrackedByKDTree,
};

pub fn plugin(app: &mut App) {
    app.register_type::<Ship>();
    app.register_type::<Team>();
    app.add_systems(PreStartup, setup);
    app.add_systems(
        Update,
        (
            move_ships,
            rotate_towards_target,
            rotate_away_from_obstacles,
            update_ship_count.run_if(on_timer(Duration::from_secs(1))),
        ),
    );
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: ResMut<AssetServer>,
) {
    let mut assets = ShipAssets::default();
    for &team in Team::ALL.iter() {
        assets.materials.insert(
            team,
            materials.add(StandardMaterial {
                base_color: Color::from(team),
                alpha_mode: AlphaMode::Opaque,
                ..default()
            }),
        );
        let mut cone_mesh = ConeMeshBuilder::new(0.5, 2.0, 32).build();
        cone_mesh.transform_by(Transform::from_rotation(Quat::from_rotation_x(
            -90.0_f32.to_radians(),
        )));
        let mut left_cone = cone_mesh.clone();
        left_cone.transform_by(
            Transform::from_translation(vec3(0.7, 0.0, 0.0)).with_scale(Vec3::splat(0.6)),
        );
        let mut right_cone = cone_mesh.clone();
        right_cone.transform_by(
            Transform::from_translation(vec3(-0.7, 0.0, 0.0)).with_scale(Vec3::splat(0.6)),
        );
        cone_mesh.merge(&left_cone);
        cone_mesh.merge(&right_cone);
        assets.mesh.insert(team, meshes.add(cone_mesh));
    }
    commands.insert_resource(assets);
    commands
        .spawn(Node {
            margin: UiRect {
                left: Val::Px(24.0),
                top: Val::Px(48.0),
                ..default()
            },
            ..default()
        })
        .with_child((
            Text::default(),
            TextFont {
                font: asset_server.load("IBM Plex Mono/IBMPlexMono-Regular.ttf"),
                font_size: 16.,
                ..default()
            },
            StatsText,
        ));
}

#[derive(Component)]
struct StatsText;

fn update_ship_count(
    mut text: Query<&mut Text, With<StatsText>>,
    ships: Query<Entity, With<Ship>>,
    lasers: Query<Entity, With<Laser>>,
    entities: Query<Entity>,
) {
    let ship_count = ships.iter().len();
    let laser_count = lasers.iter().len();
    let mut text = text.single_mut();
    let entity_count = entities.iter().len();
    text.0 = format!(
        "\
        Entities: {entity_count}\n\
        ├ Ships: {ship_count}\n\
        └ Lasers: {laser_count}\n\
    "
    );
}

pub struct SpawnShip {
    pub transform: Transform,
    pub team: Team,
}

impl Command for SpawnShip {
    fn apply(self, world: &mut World) {
        let ship_assets = world
            .get_resource::<ShipAssets>()
            .expect("ship_assets resource was missing");
        world.spawn((
            MeshMaterial3d(
                ship_assets
                    .materials
                    .get(&self.team)
                    .expect("ship_assets should be initialized")
                    .clone(),
            ),
            Mesh3d(
                ship_assets
                    .mesh
                    .get(&self.team)
                    .expect("ship_assets should be initialized")
                    .clone(),
            ),
            self.transform,
            Visibility::Visible,
            self.team,
        ));
    }
}

#[derive(Reflect, Component, Default)]
#[require(Transform, Visibility, Gun, TrackedByKDTree)]
pub struct Ship;

#[derive(Copy, Clone, Component, Reflect, Default, Hash, Eq, PartialEq)]
#[require(Ship)]
pub enum Team {
    #[default]
    Red,
    Blue,
    Green,
    Yellow,
}

impl Team {
    const ALL: [Self; 4] = [Self::Red, Self::Blue, Self::Green, Self::Yellow];
}

impl From<Team> for Color {
    fn from(value: Team) -> Self {
        match value {
            Team::Red => Color::linear_rgb(1.0, 0.0, 0.0),
            Team::Blue => Color::linear_rgb(0.0, 0.0, 1.0),
            Team::Green => Color::linear_rgb(0.0, 1.0, 0.0),
            Team::Yellow => Color::linear_rgb(1.0, 1.0, 0.0),
        }
    }
}

impl TryFrom<usize> for Team {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Red,
            1 => Self::Blue,
            2 => Self::Green,
            3 => Self::Yellow,
            _ => return Err(()),
        })
    }
}

/// A priority target for the given team to attack
#[derive(Component, Reflect, Default)]
pub struct TeamTarget(pub Team);

#[derive(Component, Reflect, Default)]
pub struct Obstacle {
    pub radius: f32,
}

#[derive(Resource, Reflect, Default)]
pub struct ShipAssets {
    pub materials: HashMap<Team, Handle<StandardMaterial>>,
    pub mesh: HashMap<Team, Handle<Mesh>>,
}

fn move_ships(mut ships: Query<&mut Transform, With<Ship>>, time: Res<Time>) {
    for mut transform in ships.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * 15.0 * time.delta_secs();
    }
}

pub fn rotate_towards_target(
    mut ships: Query<(&mut Transform, &GlobalTransform, &Team), With<Ship>>,
    targets: Query<(&GlobalTransform, &TeamTarget)>,
    time: Res<Time>,
) {
    let targets: Vec<_> = targets.into_iter().collect();

    for (mut transform, global_transform, team) in ships.iter_mut() {
        let global_transform = global_transform.compute_transform();
        let target = targets.iter().fold(
            None::<(f32, Vec3)>,
            |previous, (next_transform, next_target)| {
                if next_target.0 != *team {
                    return previous;
                }
                let pos = global_transform.translation;
                let next_pos = next_transform.translation();
                let dist = next_pos.distance_squared(pos);
                let Some(previous) = previous else {
                    return Some((dist, next_pos));
                };
                if dist < previous.0 {
                    Some((dist, next_pos))
                } else {
                    Some(previous)
                }
            },
        );
        let Some((_, target)) = target else {
            continue;
        };
        let look = global_transform.looking_at(target, Vec3::Y).rotation;
        transform.rotation = transform
            .rotation
            .rotate_towards(look, 40.0_f32.to_radians() * time.delta_secs());
    }
}

pub fn rotate_away_from_obstacles(
    mut ships: Query<(&mut Transform, &GlobalTransform), With<Ship>>,
    tree: Res<KDTree3A<TrackedByKDTree>>,
    time: Res<Time>,
) {
    ships
        .iter_mut()
        .for_each(|(mut transform, global_transform)| {
            let translation = global_transform.translation();
            let Some((other_pos, _)) = tree.nearest_neighbour(global_transform.translation_vec3a())
            else {
                return;
            };

            // Create a target direction pointing the ship away from the other entity.
            let target_direction = Vec3::from(other_pos) - translation;
            let distance = target_direction.length();
            if !(0.1..=2.0).contains(&distance) {
                return;
            }
            let target_direction = target_direction.normalize();
            // println!("{:?} {:?}", transform.rotation, target_direction);
            transform.rotation = rotate_towards(
                transform.rotation,
                target_direction,
                100.0 * time.delta_secs(),
            );
            assert!(transform.forward().is_normalized());
            // gizmos.line(
            //     global_transform.translation(),
            //     (global_transform.translation_vec3a() + away_direction * 3.0).into(),
            //     Color::WHITE,
            // );

            // transform.rotation = transform
            //     .rotation
            //     .rotate_towards(look, 20.0_f32.to_radians() * time.delta_secs());
        });
}

fn rotate_towards(quat: Quat, target_dir: Vec3, angle_deg: f32) -> Quat {
    // Ensure the target direction is normalized
    let target_dir = target_dir.normalize();

    // Compute the current direction (transform the Z-axis vector by the quaternion)
    let current_dir = quat * Vec3::Z;

    // Calculate the axis of rotation using the cross product
    let rotation_axis = current_dir.cross(target_dir);

    // If the cross product is zero, the vectors are parallel, no rotation needed
    if rotation_axis.length_squared() < f32::EPSILON {
        return quat.normalize();
    }

    // Normalize the rotation axis
    let rotation_axis = rotation_axis.normalize();

    // Convert angle from degrees to radians
    let angle_rad = angle_deg.to_radians();

    // Create a quaternion representing the rotation by `angle_rad` around the `rotation_axis`
    let rotation_quat = Quat::from_axis_angle(rotation_axis, angle_rad);

    // Combine the current quaternion with the rotation quaternion
    let result_quat = rotation_quat * quat;

    // Normalize the result to ensure it remains a valid rotation quaternion
    result_quat.normalize()
}
