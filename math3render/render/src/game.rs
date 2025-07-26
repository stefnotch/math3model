use crate::{
    camera::{
        Camera, CameraSettings,
        camera_controller::{
            CameraController, ChosenKind, GeneralController, GeneralControllerSettings,
        },
    },
    input::{CursorCaptureRequest, WindowCursorCapture, WindowInputs},
};
use glam::{Vec2, Vec3};
use web_time::Instant;

#[derive(Clone, Default)]
pub struct ProfilerSettings {
    pub gpu: bool,
}
pub struct GameRes {
    pub camera_controller: CameraController,
    last_update_instant: Option<Instant>,
    pub camera: Camera,
    pub mouse: Vec2,
    pub mouse_held: bool,
    pub cursor_capture: WindowCursorCapture,
    pub profiler_settings: ProfilerSettings,
}

impl GameRes {
    pub fn new() -> Self {
        let camera = Camera::new(CameraSettings::default());
        let camera_controller = CameraController::new(
            GeneralController {
                position: Vec3::new(0.0, 0.0, 4.0),
                orientation: glam::Quat::IDENTITY,
                distance_to_center: 4.0,
            },
            GeneralControllerSettings {
                fly_speed: 5.0,
                pan_speed: 0.1,
                rotation_sensitivity: 0.01,
            },
            ChosenKind::Freecam,
        );

        Self {
            camera,
            camera_controller,
            last_update_instant: None,
            mouse: Vec2::ZERO,
            mouse_held: false,
            cursor_capture: WindowCursorCapture::Free,
            profiler_settings: ProfilerSettings::default(),
        }
    }

    pub fn update(&mut self, inputs: &WindowInputs) {
        let now = Instant::now();
        if let Some(last_update_instant) = self.last_update_instant {
            let delta = (now - last_update_instant).as_secs_f32();
            self.cursor_capture = match self.camera_controller.update(inputs, delta) {
                CursorCaptureRequest::Free => WindowCursorCapture::Free,
                CursorCaptureRequest::LockedAndHidden => {
                    WindowCursorCapture::LockedAndHidden(inputs.mouse.position)
                }
            };
        }
        self.last_update_instant = Some(now);
        self.camera.update_camera(&self.camera_controller);
        self.mouse = Vec2::new(
            inputs.mouse.position.x as f32,
            inputs.mouse.position.y as f32,
        );
        self.mouse_held = inputs.mouse.pressed(winit::event::MouseButton::Left);
    }
}
