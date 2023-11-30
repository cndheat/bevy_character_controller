use std::f32::consts::{PI, FRAC_PI_2};

use bevy::{prelude::*, input::mouse::MouseMotion, window::PresentMode};
use bevy_rapier3d::{prelude::*, na::wrap};
use controller::{setup_player_controller, PlayerCamera, PlayerController, PlayerCameraPivot, CameraMode, PlayerModel};
use input::{update_input, PlayerInput, update_cursor, InputEnabled};
use setup::setup_world;

mod setup;
mod input;
mod controller;

fn main() {
    App::new()
    .add_plugins((
        DefaultPlugins.set(WindowPlugin{
            primary_window: Some(Window{
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }),
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default(),
    ))
    .insert_resource(ClearColor(Color::CYAN))
    .insert_resource(RapierContext::default())
    .insert_resource(PlayerInput::default())
    .insert_resource(InputEnabled(false))
    .add_systems(Startup, (
        setup_world,
        setup_player_controller,
    ))
    .add_systems(Update, (
        update_cursor,
        update_input,
        (
            move_character,
            update_camera
        ),
    ).chain())
    .run();
}

fn move_character(
    mut commands: Commands,
    mut mouse_evr: EventReader<MouseMotion>,
    mut player_query: Query<(Entity, &Collider, &mut Transform, &mut PlayerController)>,
    mut pivot_query: Query<&mut Transform, (With<PlayerCameraPivot>, Without<PlayerController>)>,
    mut rapier_context: ResMut<RapierContext>,
    input: Res<PlayerInput>,
    input_enabled: Res<InputEnabled>,
    time: Res<Time>,
) {
    let Ok((player_entity, player_collider, mut player_transform, mut player_controller)) = player_query.get_single_mut() else { return };
    
    let mut mouse_delta = Vec2::ZERO;

    if input_enabled.0 { 
        for mouse_event in mouse_evr.read() {
            mouse_delta += mouse_event.delta;
        }
    }   
    // Multiply by sensitivity
    mouse_delta *= 0.0015;

    // Calculate new rotation values based on mouse movements
    player_controller.rotation.x = (player_controller.rotation.x - mouse_delta.y).clamp(-FRAC_PI_2 + 0.001953125, FRAC_PI_2 - 0.001953125);
    player_controller.rotation.y = wrap(player_controller.rotation.y - mouse_delta.x, 0.0, 2.0 * PI);

    // Update camera rotation
    let Ok(mut pivot_transform) = pivot_query.get_single_mut() else { return };
    pivot_transform.rotation = Quat::from_euler(EulerRot::XYZ, player_controller.rotation.x, 0.0, 0.0);  

    let mut normalized_move = Vec3::ZERO;
    
    // Assign initial input directional values
    let mut forward = 0.0;
    let mut sideways = 0.0;

    if input.forward { forward -= 1.0 }
    if input.backward { forward += 1.0 }
    if input.left { sideways -= 1.0 }
    if input.right { sideways += 1.0 }

    // Calculate directional move value
    let x_fac = player_controller.rotation.y.cos();
    let z_fac = player_controller.rotation.y.sin();

    normalized_move.x = (z_fac * forward) + (x_fac * sideways);
    normalized_move.z = (x_fac * forward) + (-z_fac * sideways);

    // Normalize (x and z axis movement)
    normalized_move = normalized_move.normalize_or_zero() * player_controller.movement_speed;

    // Update Y-velocity (jump/fly/gravity)
    if player_controller.grounded {
        // Cancel any downwards acceleration and velocity since we are grounded
        if player_controller.acceleration.y < 0.0 || player_controller.velocity.y < 0.0 {
            player_controller.acceleration.y = 0.0;
            player_controller.velocity.y = 0.0;
        }

        if input.jump {
            player_controller.velocity.y += player_controller.jump_force;
        }

    } else {
        // Update gravity
        if player_controller.velocity.y >= 0.0 {
            player_controller.acceleration.y = -player_controller.gravity;
        }
    }

    let dt = time.delta().as_secs_f32();
    let velocity_change = player_controller.acceleration * dt;

    // Add on acceleration * dt
    player_controller.velocity += velocity_change;

    // Clamp y-speed to terminal velocity values
    player_controller.velocity.y = player_controller.velocity.y.clamp(-player_controller.terminal_velocity, player_controller.terminal_velocity);

    // Add velocity to move
    normalized_move += player_controller.velocity;

    // Apply delta time to queued move
    normalized_move *= dt;

    // We only want auto-stepping/ground-snapping (ramps) if the player is grounded
    let (step, snap) = if player_controller.grounded {
        (Some(CharacterAutostep{
            max_height: CharacterLength::Absolute(1.65),
            min_width: CharacterLength::Absolute(0.1),
            include_dynamic_bodies: true,
        }), 
        Some(CharacterLength::Absolute(0.05)))
    } else {
        (None, None)
    };
    
    let move_output = rapier_context.move_shape(
        normalized_move,
        player_collider,
        player_transform.translation,
        Quat::IDENTITY, // We do not care about rotation - collider is locked
        player_controller.mass, 
        &MoveShapeOptions{
            autostep: step,
            snap_to_ground: snap,
            slide: true,
            ..default()
        }, 
        QueryFilter::new().exclude_collider(player_entity),
        |_| {},
    );

    // Update grounded ( we cannot use move_output grounded, it is inaccurate :D )

    if let Some(_) = rapier_context.intersection_with_shape(
        player_transform.translation + Vec3::new(0.0, -2.25, 0.0),
        Quat::IDENTITY,
        // Collider with a slightly smaller radius than our main collider and a very small y-value
        &Collider::cylinder(0.05, 1.24),
        QueryFilter::new().exclude_collider(player_entity)
    ) {
        if player_controller.velocity.y <= 0.0 {
            player_controller.grounded = true;
        } else {
            player_controller.grounded = false;
        }

    } else {
        player_controller.grounded = false;
    }

    // Update player position
    player_transform.translation += move_output.effective_translation;
}

fn update_camera(
    input: Res<PlayerInput>,
    mut camera_query: Query<(&mut Transform, &mut PlayerCamera)>,
    mut playermodel_query: Query<&mut Transform, (With<PlayerModel>, Without<PlayerCamera>)>,
    pivot_query: Query<&GlobalTransform, With<PlayerCameraPivot>>,
    player_query: Query<(Entity, &PlayerController)>,
    rapier_context: Res<RapierContext>,
) {
    let Ok((mut camera_transform, mut camera_data)) = camera_query.get_single_mut() else { return };

    if input.toggle_view {
        match camera_data.mode {
            CameraMode::FirstPerson => {
                camera_data.mode = CameraMode::ThirdPerson;
                camera_transform.translation = Vec3::new(0.0, 0.0, 10.0);
            },
            CameraMode::ThirdPerson => {
                camera_data.mode = CameraMode::FirstPerson;
                camera_transform.translation = Vec3::ZERO;
            },
        }
    }

    // Update player rotation here as to avoid query hell in the prior system
    let Ok((collider_entity, player_controller)) = player_query.get_single() else { return };
    let Ok(mut playermodel_transform) = playermodel_query.get_single_mut() else { return };

    playermodel_transform.rotation = Quat::from_euler(EulerRot::XYZ, 0.0, player_controller.rotation.y, 0.0);

    if let CameraMode::ThirdPerson = camera_data.mode {
        let Ok(pivot_transform) = pivot_query.get_single() else { return };
    
        // Check if camera is colliding with anything - if it is, move it closer
        if let Some((_, ray_intersection)) = rapier_context.cast_ray_and_get_normal(
            pivot_transform.translation(), 
            Vec3::ZERO - pivot_transform.forward(),
            11.0, 
            false,
            QueryFilter::new().exclude_collider(collider_entity)
        ) {
            // Ray hit something, lets set the camera position to distance of impact
            camera_transform.translation = Vec3::new(0.0, 0.0, ray_intersection.toi - 1.0); 
        } else {    
            // Nothing hit by the ray, lets set standard translation
            camera_transform.translation = Vec3::new(0.0, 0.0, 10.0);
        }
    }
}