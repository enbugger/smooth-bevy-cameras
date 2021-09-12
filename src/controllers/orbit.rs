use crate::{LookAngles, LookTransform, LookTransformBundle, Smoother, ControllerEnabled};

use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseWheel},
        prelude::*,
    },
    math::prelude::*,
    render::prelude::*,
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(map_orbit_input.system())
            .add_system(control_orbit_camera.system())
            .add_event::<OrbitControlEvent>();
    }
}


#[derive(Bundle)]
pub struct OrbitCameraBundle {
    controller: OrbitCameraController,
}

impl OrbitCameraBundle {
    pub fn new( controller: OrbitCameraController) -> Self {
        Self { controller }
    }
}

/// A 3rd person camera that orbits around the target.
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct OrbitCameraController {
    pub enabled: bool,
    pub mouse_rotate_sensitivity: Vec2,
    pub mouse_translate_sensitivity: Vec2,
    pub mouse_wheel_zoom_sensitivity: f32,
}

impl Default for OrbitCameraController {
    fn default() -> Self {
        Self {
            mouse_rotate_sensitivity: Vec2::splat(0.006),
            mouse_translate_sensitivity: Vec2::splat(0.008),
            mouse_wheel_zoom_sensitivity: 0.15,
            enabled: true,
        }
    }
}

pub enum OrbitControlEvent {
    Orbit(Vec2),
    TranslateTarget(Vec2),
    Zoom(f32),
}

pub fn map_orbit_input(
    mut events: EventWriter<OrbitControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    _keyboard: Res<Input<KeyCode>>,
    controllers: Query<&OrbitCameraController, With<Transform>>,
) {
    // Can only control one camera at a time.
    let controller = if let Some(controller) = controllers.iter().next() {
        controller
    } else {
        return;
    };
    let OrbitCameraController {
        enabled,
        mouse_rotate_sensitivity,
        mouse_translate_sensitivity,
        mouse_wheel_zoom_sensitivity,
        ..
    } = *controller;

    if !enabled {
        return;
    }

    let mut cursor_delta = Vec2::ZERO;
    for event in mouse_motion_events.iter() {
        cursor_delta += event.delta;
    }

    events.send(OrbitControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));

    if mouse_buttons.pressed(MouseButton::Middle) {
        events.send(OrbitControlEvent::TranslateTarget(
            mouse_translate_sensitivity * cursor_delta,
        ));
    }

    let mut scalar = 1.0;
    for event in mouse_wheel_reader.iter() {
        scalar *= 1.0 + -event.y * mouse_wheel_zoom_sensitivity;
    }
    events.send(OrbitControlEvent::Zoom(scalar));
}

pub fn control_orbit_camera(
    mut events: EventReader<OrbitControlEvent>,
    mut cameras: Query<(&OrbitCameraController, &mut LookTransform, &Transform, With<Transform>)>,
) {
    // Can only control one camera at a time.
    let (controller, mut transform, scene_transform) =
        if let Some((controller, transform, scene_transform, _)) = cameras.iter_mut().next() {
            (controller, transform, scene_transform)
        } else {
            return;
        };

    if controller.enabled {
        let mut look_angles = LookAngles::from_vector(-transform.look_direction());
        let mut radius_scalar = 1.0;

        for event in events.iter() {
            match event {
                OrbitControlEvent::Orbit(delta) => {
                    look_angles.add_yaw(-delta.x);
                    look_angles.add_pitch(delta.y);
                }
                OrbitControlEvent::TranslateTarget(delta) => {
                    let right_dir = scene_transform.rotation * -Vec3::X;
                    let up_dir = scene_transform.rotation * Vec3::Y;
                    transform.target += delta.x * right_dir + delta.y * up_dir;
                }
                OrbitControlEvent::Zoom(scalar) => {
                    radius_scalar *= scalar;
                }
            }
        }

        look_angles.assert_not_looking_up();

        transform.eye =
            transform.target + radius_scalar * transform.radius() * look_angles.unit_vector();
    } else {
        events.iter(); // Drop the events.
    }
}
