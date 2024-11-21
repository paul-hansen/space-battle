use bevy::{
    math::vec3,
    pbr::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::mesh::ConeMeshBuilder,
    utils::HashMap,
};

use crate::lasers::Gun;

pub fn plugin(app: &mut App) {
    app.register_type::<Ship>();
    app.register_type::<Team>();
    app.add_systems(PreStartup, setup);
    app.add_systems(Update, (move_ships, rotate_towards_target));
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
            Gun::default(),
            NotShadowReceiver,
            NotShadowCaster,
        ));
    }
}

#[derive(Reflect, Component, Default)]
#[require(Transform, Visibility)]
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

#[derive(Component, Reflect, Default)]
pub struct Target(pub Team);

#[derive(Resource, Reflect, Default)]
pub struct ShipAssets {
    pub materials: HashMap<Team, Handle<StandardMaterial>>,
    pub mesh: HashMap<Team, Handle<Mesh>>,
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
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
}

fn move_ships(mut ships: Query<&mut Transform, With<Ship>>, time: Res<Time>) {
    for mut transform in ships.iter_mut() {
        let forward = transform.forward();
        transform.translation += forward * 15.0 * time.delta_secs();
    }
}

pub fn rotate_towards_target(
    mut ships: Query<(&mut Transform, &GlobalTransform, &Team), With<Ship>>,
    targets: Query<(&GlobalTransform, &Target)>,
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
            .rotate_towards(look, 20.0_f32.to_radians() * time.delta_secs());
    }
}
