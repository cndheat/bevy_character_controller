use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Component)]
pub struct PlayerController {
    pub acceleration: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec2,
    pub movement_speed: f32,
    pub jump_force: f32,
    pub mass: f32,
    pub gravity: f32,
    pub terminal_velocity: f32,
    pub grounded: bool,
}

#[derive(Component)]
pub struct PlayerModel();

#[derive(Component)]
pub struct PlayerCameraPivot();

#[derive(Component, Default)]
pub struct PlayerCamera {
    pub mode: CameraMode,
}

#[derive(Default)]
pub enum CameraMode {
    #[default]
    FirstPerson,
    ThirdPerson
}

impl Default for PlayerController {
    fn default() -> Self {
        Self {
            acceleration: Vec3::ZERO,
            velocity: Vec3::ZERO,
            rotation: Vec2::ZERO,
            grounded: false,
            movement_speed: 10.0,
            jump_force: 9.8,
            mass: 100.0,
            gravity: 9.8,
            terminal_velocity: 180.0,
        }
    }
}

pub fn setup_player_controller(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Collider::cylinder(2.25, 1.25),
        RigidBody::KinematicPositionBased,
        Ccd{enabled: true},
        TransformBundle::from_transform(Transform::from_xyz(0.0, 3.0, 0.0)),
        PlayerController::default(),
    ))
    .with_children(|parent| {
        // Child entity with transformbundle so we can rotate it individually
        parent.spawn((

            PbrBundle{
                mesh: meshes.add(Mesh::from(shape::Cylinder{
                    radius: 1.25,
                    height: 4.5,
                    ..default()
                })),
                material: materials.add(StandardMaterial{
                    base_color: Color::PINK,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            PlayerModel(),
        ))
        .with_children(|parent| {

            parent.spawn((
                PlayerCameraPivot(),
                TransformBundle::from_transform(Transform::IDENTITY),
            )).with_children(|parent| {
                parent.spawn((
                    PlayerCamera::default(),
                    Camera3dBundle::default()
                ));
            });

        });

    });
}