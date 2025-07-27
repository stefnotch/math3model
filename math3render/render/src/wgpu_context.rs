use std::sync::Arc;

use glam::UVec2;
use log::info;
use wgpu_profiler::{GpuProfiler, GpuProfilerSettings};
use winit::window::Window;

/// Guaranteed by the WebGPU spec
/// According to https://vulkan.gpuinfo.org/listsurfaceformats.php?platform=linux
/// it exists on basically all devices, except for Android
pub const VIEW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;

use crate::window_or_fallback::WindowOrFallback;
pub struct WgpuContext {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl WgpuContext {
    pub async fn new() -> anyhow::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                // Setting this is only needed for a fallback adapter. Which we don't want.
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await?;
        info!("Adapter: {:?}", adapter.get_info());

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::default()
                    | (adapter.features() & GpuProfiler::ALL_WGPU_TIMER_FEATURES)
                    | (adapter.features() & wgpu::Features::POLYGON_MODE_LINE),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await
            .unwrap();

        Ok(WgpuContext {
            instance,
            adapter,
            device,
            queue,
        })
    }
}

pub enum SurfaceTexture {
    Surface(wgpu::SurfaceTexture, wgpu::TextureView, Arc<Window>),
    Fallback(wgpu::TextureView),
}

impl SurfaceTexture {
    pub fn texture_view(&self) -> &wgpu::TextureView {
        match self {
            SurfaceTexture::Surface(_, view, _) => &view,
            SurfaceTexture::Fallback(view) => &view,
        }
    }

    pub fn present(self) {
        match self {
            SurfaceTexture::Surface(surface_texture, _, window) => {
                window.pre_present_notify();
                surface_texture.present();
            }
            SurfaceTexture::Fallback(_) => {}
        }
    }
}

pub enum WgpuSurface {
    Surface {
        surface: wgpu::Surface<'static>,
        config: wgpu::SurfaceConfiguration,
        window: Arc<Window>,
        size: UVec2,
    },
    Fallback {
        texture: wgpu::Texture,
        size: UVec2,
    },
}

impl WgpuSurface {
    pub fn new(context: &WgpuContext, window: WindowOrFallback) -> anyhow::Result<Self> {
        let size = window.size().max(UVec2::ONE);

        let surface = window
            .as_window()
            .map(|window| context.instance.create_surface(window))
            .transpose()?;

        match surface {
            Some(surface) => {
                let config = wgpu::SurfaceConfiguration {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    view_formats: vec![VIEW_FORMAT],
                    present_mode: wgpu::PresentMode::AutoVsync,
                    ..surface
                        .get_default_config(&context.adapter, size.x, size.y)
                        .ok_or_else(|| anyhow::anyhow!("No default surface config found"))?
                };
                surface.configure(&context.device, &config);
                Ok(WgpuSurface::Surface {
                    surface,
                    config,
                    window: window
                        .as_window()
                        .expect("Expected window if there is a surface"),
                    size,
                })
            }
            None => Ok(WgpuSurface::Fallback {
                texture: create_fallback_texture(&context.device, size),
                size,
            }),
        }
    }

    pub fn size(&self) -> UVec2 {
        match self {
            WgpuSurface::Surface { size, .. } => *size,
            WgpuSurface::Fallback { size, .. } => *size,
        }
    }

    /// Tries to resize the swapchain to the new size.
    /// Returns the actual size of the swapchain if it was resized.
    pub fn try_resize(&mut self, context: &WgpuContext, new_size: UVec2) -> Option<UVec2> {
        let new_size = new_size.max(UVec2::new(1, 1));
        if new_size == self.size() {
            return None;
        }
        match self {
            WgpuSurface::Surface {
                surface,
                config,
                size,
                ..
            } => {
                config.width = new_size.x;
                config.height = new_size.y;
                surface.configure(&context.device, config);
                *size = new_size;
                Some(new_size)
            }
            WgpuSurface::Fallback { texture, size } => {
                *texture = create_fallback_texture(&context.device, new_size);
                *size = new_size;
                Some(new_size)
            }
        }
    }

    pub fn recreate_swapchain(&self, context: &WgpuContext) {
        match self {
            WgpuSurface::Surface {
                surface, config, ..
            } => {
                surface.configure(&context.device, config);
            }
            WgpuSurface::Fallback { .. } => {
                // No-op
            }
        }
    }

    pub fn surface_texture(&self) -> Result<SurfaceTexture, wgpu::SurfaceError> {
        match self {
            WgpuSurface::Surface {
                surface, window, ..
            } => surface.get_current_texture().map(|surface_texture| {
                let view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor {
                        format: Some(VIEW_FORMAT),
                        ..Default::default()
                    });
                SurfaceTexture::Surface(surface_texture, view, window.clone())
            }),
            WgpuSurface::Fallback { texture, .. } => Ok(SurfaceTexture::Fallback(
                texture.create_view(&wgpu::TextureViewDescriptor {
                    format: Some(VIEW_FORMAT),
                    ..Default::default()
                }),
            )),
        }
    }

    pub fn pre_present_notify(&self) {
        match self {
            WgpuSurface::Surface { window, .. } => window.pre_present_notify(),
            WgpuSurface::Fallback { .. } => {}
        }
    }
}

fn create_fallback_texture(device: &wgpu::Device, size: UVec2) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Fallback surface"),
        size: wgpu::Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: VIEW_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    })
}

pub fn create_profiler(context: &WgpuContext) -> GpuProfiler {
    let gpu_profiler_settings = GpuProfilerSettings {
        enable_timer_queries: false, // Disabled by default
        enable_debug_groups: false,
        ..GpuProfilerSettings::default()
    };

    let profiler = GpuProfiler::new(&context.device, gpu_profiler_settings)
        .expect("Failed to create profiler");
    profiler
}
