use crate::{LookAngles, LookTransform, LookTransformBundle, Smoother};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{mouse::MouseMotion, prelude::*},
    math::prelude::*,
    render::prelude::*,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

pub struct FpsCameraPlugin;

impl Plugin for FpsCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(map_fps_input.system())
            .add_system(control_fps_camera.system())
            .add_event::<FPSControlEvent>();
    }
}

#[derive(Bundle)]
pub struct FpsCameraBundle {
    controller: FpsCameraController,
}

impl FpsCameraBundle {
    pub fn new( controller: FpsCameraController) -> Self {
        Self { controller }
    }
}

/// Your typical first-person camera controller.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct FpsCameraController {
    pub enabled: bool,
    pub mouse_rotate_sensitivity: Vec2,
    pub translate_sensitivity: f32,
}

impl Default for FpsCameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            mouse_rotate_sensitivity: Vec2::splat(0.002),
            translate_sensitivity: 0.5,
        }
    }
}

pub enum FPSControlEvent {
    Rotate(Vec2),
    TranslateEye(Vec3),
}

pub fn map_fps_input(
    mut events: EventWriter<FPSControlEvent>,
    keyboard: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    controllers: Query<&FpsCameraController, With<Transform>>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().next() {
        controller
    } else {
        return;
    };
    let FpsCameraController {
        enabled,
        translate_sensitivity,
        mouse_rotate_sensitivity,
        ..
    } = *controller;

    if !enabled {
        return;
    }

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    events.send(FPSControlEvent::Rotate(
        mouse_rotate_sensitivity * cursor_delta,
    ));

    for (key, dir) in [
        (KeyCode::W, Vec3::Z),
        (KeyCode::A, Vec3::X),
        (KeyCode::S, -Vec3::Z),
        (KeyCode::D, -Vec3::X),
        (KeyCode::LShift, -Vec3::Y),
        (KeyCode::Space, Vec3::Y),
    ]
    .iter()
    .cloned()
    {
        if keyboard.pressed(key) {
            events.send(FPSControlEvent::TranslateEye(translate_sensitivity * dir));
        }
    }
}

pub fn control_fps_camera(
    mut events: EventReader<FPSControlEvent>,
    mut cameras: Query<(&FpsCameraController, &mut LookTransform, With<Transform>)>,
) {
    // Can only control one camera at a time.
    let (controller, mut transform) =
        if let Some((controller, transform, _)) = cameras.iter_mut().next() {
            (controller, transform)
        } else {
            return;
        };

    if controller.enabled {
        let look_vector = transform.look_direction();
        let mut look_angles = LookAngles::from_vector(look_vector);

        let yaw_rot = Quat::from_axis_angle(Vec3::Y, look_angles.get_yaw());
        let rot_x = yaw_rot * Vec3::X;
        let rot_y = yaw_rot * Vec3::Y;
        let rot_z = yaw_rot * Vec3::Z;

        for event in events.iter() {
            match event {
                FPSControlEvent::Rotate(delta) => {
                    // Rotates with pitch and yaw.
                    look_angles.add_yaw(-delta.x);
                    look_angles.add_pitch(-delta.y);
                }
                FPSControlEvent::TranslateEye(delta) => {
                    // Translates up/down (Y) left/right (X) and forward/back (Z).
                    transform.eye += delta.x * rot_x + delta.y * rot_y + delta.z * rot_z;
                }
            }
        }

        look_angles.assert_not_looking_up();

        transform.target = transform.eye + transform.radius() * look_angles.unit_vector();
    } else {
        events.iter(); // Drop the events.
    }
}
