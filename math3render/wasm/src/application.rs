use glam::Vec3;
use log::error;
use render::{
    application::{AppCommand, Application, ShaderCompiledCallback, WasmCanvas, run_on_main},
    camera::{
        Angle,
        camera_controller::{self, CameraController, IsCameraController},
        orbitcam_controller::LogarithmicDistance,
    },
    input::WinitAppHelper,
    scene::{Model, ShaderId, ShaderInfo, TextureData, TextureId, TextureInfo},
};
use std::sync::Arc;
use wasm_bindgen::{JsError, JsValue, prelude::wasm_bindgen};
use web_sys::{HtmlCanvasElement, ImageBitmap};
use winit::event_loop::{EventLoop, EventLoopProxy};

use crate::wasm_abi::{WasmCompilationMessage, WasmModelInfo, WasmPosition, WasmShaderInfo};

#[wasm_bindgen]
pub struct WasmApplication {
    event_loop_proxy: Option<EventLoopProxy<AppCommand>>,
}

#[wasm_bindgen]
impl WasmApplication {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<WasmApplication, JsError> {
        Ok(Self {
            event_loop_proxy: None,
        })
    }

    pub async fn run(&mut self, _canvas: HtmlCanvasElement) -> Result<(), JsError> {
        let event_loop = EventLoop::<AppCommand>::with_user_event().build()?;
        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
        let event_loop_proxy = event_loop.create_proxy();
        self.event_loop_proxy = Some(event_loop_proxy.clone());
        #[cfg(target_arch = "wasm32")]
        let wasm_canvas = WasmCanvas::new(_canvas);
        #[cfg(not(target_arch = "wasm32"))]
        let wasm_canvas = WasmCanvas::new();
        let mut application = Application::new(event_loop_proxy, |_| {}, wasm_canvas)
            .await
            .map_err(|e| JsError::from(&*e))?;
        application.app.profiler_settings.gpu = true;
        application.app.camera_controller = CameraController::new(
            render::camera::orbitcam_controller::OrbitcamController {
                center: Vec3::ZERO,
                pitch: Angle::from_degrees(-20.),
                yaw: Angle::from_degrees(190.),
                logarithmic_distance: LogarithmicDistance::new(8.0),
            }
            .general_controller(),
            application.app.camera_controller.settings,
            camera_controller::ChosenKind::Orbitcam,
        );
        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(WinitAppHelper::new(application));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            event_loop.run_app(&mut WinitAppHelper::new(application))?;
        }
        Ok(())
    }

    pub async fn update_models(&self, js_models: Vec<WasmModelInfo>) {
        let models = js_models
            .into_iter()
            .map(|v| Model {
                name: v.id,
                transform: v.transform.into(),
                material_info: v.material_info.into(),
                shader_id: ShaderId(v.shader_id),
                instance_count: v.instance_count,
            })
            .collect::<Vec<_>>();
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            app.renderer.update_models(&models);
        })
        .await;
    }

    pub async fn update_shader(&self, shader_info: WasmShaderInfo) {
        let shader_id = ShaderId(shader_info.id);
        let shader_info = ShaderInfo {
            label: shader_info.label,
            code: shader_info.code,
        };

        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            let on_shader_compiled = app.on_shader_compiled.clone();
            let shader_result = app.renderer.set_shader(shader_id.clone(), &shader_info);
            wasm_bindgen_futures::spawn_local(async move {
                match shader_result.await {
                    Ok(()) => {}
                    Err(err) => {
                        on_shader_compiled.map(|v| (v.0)(&shader_id, err));
                    }
                }
            });
        })
        .await;
    }

    pub async fn remove_shader(&self, id: String) {
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), |app| {
            let shader_id = ShaderId(id);
            app.renderer.remove_shader(&shader_id);
        })
        .await;
    }

    pub async fn update_texture(&self, texture_id: String, image: ImageBitmap) {
        let id = TextureId(texture_id);
        let info = TextureInfo {
            width: image.width(),
            height: image.height(),
            #[cfg(target_arch = "wasm32")]
            data: TextureData::Image(image),
            #[cfg(not(target_arch = "wasm32"))]
            data: TextureData::Bytes(vec![0, 0, 0]),
        };

        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), {
            let id = id.clone();
            move |app| {
                app.renderer.set_texture(id, &info);
            }
        })
        .await;
    }

    pub async fn remove_texture(&self, id: String) {
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), |app| {
            app.renderer.remove_texture(&TextureId(id));
        })
        .await;
    }

    pub async fn set_on_shader_compiled(
        &mut self,
        on_shader_compiled: Option<web_sys::js_sys::Function>,
    ) {
        let wrapped = on_shader_compiled.map(|on_shader_compiled| -> ShaderCompiledCallback {
            ShaderCompiledCallback(Arc::new(
                move |shader_id: &ShaderId, messages: Vec<wgpu::CompilationMessage>| {
                    let this = wasm_bindgen::JsValue::NULL;
                    let messages = messages
                        .into_iter()
                        .map(|message| WasmCompilationMessage::from(message))
                        .collect::<Vec<_>>();
                    match on_shader_compiled.call2(
                        &this,
                        &JsValue::from_str(&shader_id.0),
                        &serde_wasm_bindgen::to_value(&messages).unwrap(),
                    ) {
                        Ok(_) => (),
                        Err(e) => error!("Error calling on_shader_compiled: {:?}", e),
                    }
                },
            ))
        });
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            app.on_shader_compiled = wrapped;
        })
        .await;
    }

    pub async fn set_threshold_factor(&self, factor: f32) {
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            app.renderer.set_threshold_factor(factor);
        })
        .await;
    }

    pub async fn focus_on(&self, position: WasmPosition) {
        let _ = run_on_main(self.event_loop_proxy.clone().unwrap(), move |app| {
            app.app.camera_controller.focus_on(position.into());
        })
        .await;
    }
}
