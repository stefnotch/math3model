#[cfg(not(target_arch = "wasm32"))]
use crate::input::KeyboardInputHelpers;
use crate::{
    game::GameRes,
    gui::Gui,
    input::WindowInputCollector,
    renderer::GpuApplication,
    scene::ShaderId,
    time::TimeCounters,
    wgpu_context::{WgpuContext, WgpuSurface},
    window_or_fallback::WindowOrFallback,
};
use glam::UVec2;
use log::{error, info, warn};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent,
    event_loop::EventLoopProxy, window::Window,
};

pub struct WasmCanvas {
    #[cfg(target_arch = "wasm32")]
    pub canvas: web_sys::HtmlCanvasElement,
}
impl WasmCanvas {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        Self {}
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self { canvas }
    }
}

pub struct Application {
    pub app: GameRes,
    pub gui: Gui,
    window: Option<Arc<Window>>,
    pub renderer: GpuApplication,
    pub surface: Option<WgpuSurface>,
    time_counters: TimeCounters,
    input: WindowInputCollector,
    _app_commands: EventLoopProxy<AppCommand>,
    on_exit_callback: Option<Box<dyn FnOnce(&mut Application)>>,
    pub on_shader_compiled: Option<ShaderCompiledCallback>,
    _canvas: WasmCanvas,
}
#[derive(Clone)]
pub struct ShaderCompiledCallback(pub Arc<dyn Fn(&ShaderId, Vec<wgpu::CompilationMessage>)>);

impl Application {
    pub async fn new(
        app_commands: EventLoopProxy<AppCommand>,
        on_exit: impl FnOnce(&mut Application) + 'static,
        canvas: WasmCanvas,
    ) -> anyhow::Result<Self> {
        let context = WgpuContext::new().await?;
        Ok(Self {
            app: GameRes::new(),
            gui: Gui::new(&context),
            window: None,
            renderer: GpuApplication::new(context),
            surface: None,
            time_counters: TimeCounters::default(),
            input: Default::default(),
            _app_commands: app_commands,
            on_exit_callback: Some(Box::new(on_exit)),
            on_shader_compiled: None,
            _canvas: canvas,
        })
    }

    fn on_exit(&mut self) {
        self.window.take();
        if let Some(on_exit_callback) = self.on_exit_callback.take() {
            on_exit_callback(self);
        }
    }

    fn create_surface(&mut self, window: Window) {
        let window = Arc::new(window);
        self.window = Some(window.clone());
        let mut surface =
            WgpuSurface::new(&self.renderer.context, WindowOrFallback::Window(window)).unwrap();
        let size = surface.size();
        self.renderer.resize(&mut surface, size);
        self.surface = Some(surface);
    }
}

pub enum AppCommand {
    RunCallback(Box<dyn FnOnce(&mut Application)>),
}

/// Run a function on the main thread and awaits its result.
/// Not a part of the Application, because we want to be able to call this without the lifetime constraint of the Application.
#[must_use]
pub async fn run_on_main<Callback, T>(
    app_commands: EventLoopProxy<AppCommand>,
    callback: Callback,
) -> T
where
    Callback: (FnOnce(&mut Application) -> T) + 'static,
    T: Send + 'static,
{
    let (sender, receiver) = futures_channel::oneshot::channel();
    let callback = move |app: &mut Application| {
        let return_value = callback(app);
        _ = sender.send(return_value);
    };
    app_commands
        .send_event(AppCommand::RunCallback(Box::new(callback)))
        .map_err(|_| ())
        .expect("Failed to send event, event loop not running?");
    receiver.await.expect("Was the main thread stopped?")
}

impl ApplicationHandler<AppCommand> for Application {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
            return;
        }

        let window_attributes = Window::default_attributes();
        #[cfg(target_arch = "wasm32")]
        let window_attributes = {
            use winit::platform::web::WindowAttributesExtWebSys;
            window_attributes.with_canvas(Some(self._canvas.canvas.clone()))
        };

        let window = event_loop.create_window(window_attributes).unwrap();
        // someday winit will natively support having a future here. instead of the dance that create_surface has to do
        // https://github.com/rust-windowing/winit/issues/3626#issuecomment-2097916252
        self.create_surface(window);
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, event: AppCommand) {
        match event {
            AppCommand::RunCallback(callback) => callback(self),
        }
    }

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
    ) {
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        use winit::keyboard::{Key, KeyCode, NamedKey};
        self.input.handle_window_event(&event);
        match event {
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                if let Some(surface) = &mut self.surface {
                    // And this will be followed up with a rendering event
                    self.renderer.resize(surface, UVec2::new(width, height));
                }
            }
            WindowEvent::CloseRequested => {
                self.on_exit();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.update_cursor_capture();
                self.render(event_loop);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                #[cfg(not(target_arch = "wasm32"))]
                if event.just_released_logical(Key::Named(NamedKey::Escape)) {
                    self.on_exit();
                    return event_loop.exit();
                }
                // Press P to print profiling data
                #[cfg(not(target_arch = "wasm32"))]
                if event.just_pressed_physical(KeyCode::KeyP) {
                    match &self.time_counters.last_results {
                        Some(data) => {
                            let file_name = format!(
                                "profile-{}.json",
                                // use the current time as a unique-enugh identifier
                                std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis()
                            );
                            wgpu_profiler::chrometrace::write_chrometrace(
                                std::path::Path::new(&file_name),
                                data,
                            )
                            .unwrap();
                            info!("Profiling data written to {file_name}");
                        }
                        None => {
                            warn!("Profiling data not available");
                        }
                    }
                }
                _ = event;
            }
            _ => (),
        }
    }
    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.input.handle_device_event(&event);
    }
}

impl Application {
    pub fn render(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.gui.time_stats = self.time_counters.stats();
        let input = self.input.step();
        self.app.update(&input);

        if let Some(surface) = self.surface.as_mut() {
            let raw_input = egui::RawInput {
                viewport_id: egui::ViewportId::ROOT,
                viewports: egui::ViewportIdMap::from_iter([(
                    egui::ViewportId::ROOT,
                    egui::ViewportInfo {
                        native_pixels_per_point: self
                            .window
                            .as_ref()
                            .map(|w| w.scale_factor() as f32),
                        ..Default::default()
                    },
                )]),
                screen_rect: Some(egui::Rect::from_min_size(
                    Default::default(),
                    egui::Vec2::new(surface.size().x as f32, surface.size().y as f32),
                )),

                ..Default::default()
            };

            let gui_render = self.gui.update(raw_input);

            match self.renderer.render(surface, &self.app, gui_render) {
                Ok(Some(render_results)) => {
                    self.time_counters
                        .push_frame(render_results.delta_time, render_results.profiler_results);
                }
                Ok(None) => {
                    // Skipped a frame
                }
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    info!("Lost or outdated surface");
                    // Nothing to do, surface will be recreated
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    error!("Out of memory");
                    self.on_exit();
                    event_loop.exit()
                }
                Err(e) => {
                    warn!("Unexpected error: {e:?}");
                }
            }
        }
    }

    fn update_cursor_capture(&mut self) {
        if let Some(window) = &self.window {
            self.input
                .cursor_capture
                .update(self.app.cursor_capture, window);
        }
    }
}
