use bevy::{prelude::*, window::{PrimaryWindow, CursorGrabMode}};

#[derive(Resource)]
pub struct InputEnabled(pub bool);

#[derive(Resource, Default)]
pub struct PlayerInput {
    // Directional
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    // Action
    pub jump: bool,
    pub toggle_view: bool,
}

pub fn update_cursor(
    mut input_enabled: ResMut<InputEnabled>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>
) {
    for mut window in window_query.iter_mut() {
        if buttons.just_pressed(MouseButton::Left) {
            window.cursor.grab_mode = CursorGrabMode::Locked;
            window.cursor.visible = false;
            input_enabled.0 = true;
        }

        if keys.just_pressed(KeyCode::Escape) {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
            input_enabled.0 = false;
        }
    }
}

pub fn update_input(
    input_enabled: Res<InputEnabled>,
    keys: Res<Input<KeyCode>>,
    mut input: ResMut<PlayerInput>,
) {
    if !input_enabled.0 { return }

    if keys.pressed(KeyCode::W) { input.forward = true } else { input.forward = false }
    if keys.pressed(KeyCode::S) { input.backward = true } else { input.backward = false }
    if keys.pressed(KeyCode::A) { input.left = true } else { input.left = false }
    if keys.pressed(KeyCode::D) { input.right = true } else { input.right = false }

    if keys.pressed(KeyCode::Space) { input.jump = true } else { input.jump = false }
    if keys.just_pressed(KeyCode::Tab) { input.toggle_view = true } else { input.toggle_view = false }
}