use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        PbrBundle{
            mesh: meshes.add(Mesh::from(shape::Plane{
                size: 100.0,
                ..default()
            })),
            material: materials.add(StandardMaterial{
                base_color: Color::DARK_GREEN,
                ..default()
            }),
            ..default()
        },
        Collider::halfspace(Vec3::Y).unwrap()
    ));

    // Light
    commands.spawn(DirectionalLightBundle{
        directional_light: DirectionalLight{
            illuminance: 30_000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 1.5, -1.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Obstacles!
    commands.spawn((
        PbrBundle{
            mesh: meshes.add(Mesh::from(shape::Box{
                min_x: -5.0,
                max_x: 5.0,
                min_y: -15.0,
                max_y: 15.0,
                min_z: -5.0,
                max_z: 5.0,
            })),
            material: materials.add(StandardMaterial{
                base_color: Color::WHITE,
                ..default()
            }),
            transform: Transform::from_xyz(8.0, 1.0, 8.0),
            ..default()
        },
        Collider::cuboid(5.0, 15.0, 5.0),
        RigidBody::Fixed
    ));

    let mut ramp_transform = Transform::from_xyz(-8.0, 1.0, -18.0);

    ramp_transform.rotate_local_x(-0.6);

    commands.spawn((
        PbrBundle{
            mesh: meshes.add(Mesh::from(shape::Box{
                min_x: -5.0,
                max_x: 5.0,
                min_y: -5.0,
                max_y: 5.0,
                min_z: -15.0,
                max_z: 15.0,
            })),
            material: materials.add(StandardMaterial{
                base_color: Color::WHITE,
                ..default()
            }),
            transform: ramp_transform,
            ..default()
        },
        Collider::cuboid(5.0, 5.0, 15.0),
        RigidBody::Fixed
    ));
}