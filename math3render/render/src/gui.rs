use wgpu::TextureView;

use crate::{
    time::TimeStats,
    wgpu_context::{VIEW_FORMAT, WgpuContext, WgpuSurface},
};

pub struct Gui {
    pub ctx: egui::Context,
    pub time_stats: TimeStats,
    renderer: egui_wgpu::Renderer,
}

impl Gui {
    pub fn new(context: &WgpuContext) -> Self {
        Self {
            ctx: egui::Context::default(),
            time_stats: TimeStats::default(),
            renderer: egui_wgpu::Renderer::new(&context.device, VIEW_FORMAT, None, 1, false),
        }
    }

    pub fn update<'a>(&'a mut self, raw_input: egui::RawInput) -> GuiRender<'a> {
        let mut full_output = self.ctx.run(raw_input, |ctx| {
            egui::TopBottomPanel::bottom("bottom_panel")
                .frame(egui::Frame::NONE)
                .show_separator_line(false)
                .show(&ctx, |ui| {
                    ui.label(format!(
                        "CPU {:.2}ms GPU {:.2}ms",
                        self.time_stats.avg_delta_time * 1000.0,
                        self.time_stats.avg_gpu_time * 1000.0
                    ));
                });
        });
        // LATER: Deal with copy-paste events from full_output.platform_output
        let clipped_primitives = self.ctx.tessellate(
            std::mem::take(&mut full_output.shapes),
            full_output.pixels_per_point,
        );

        GuiRender {
            full_output,
            clipped_primitives,
            renderer: &mut self.renderer,
        }
    }
}

pub struct GuiRender<'a> {
    pub full_output: egui::FullOutput,
    clipped_primitives: Vec<egui::ClippedPrimitive>,
    renderer: &'a mut egui_wgpu::Renderer,
}

impl<'a> GuiRender<'a> {
    pub fn render(
        &mut self,
        context: &WgpuContext,
        surface: &WgpuSurface,
        surface_texture: &TextureView,
        commands: &mut wgpu::CommandEncoder,
    ) {
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: surface.size().to_array(),
            pixels_per_point: self.full_output.pixels_per_point,
        };
        for (id, image_delta) in &self.full_output.textures_delta.set {
            self.renderer
                .update_texture(&context.device, &context.queue, *id, image_delta);
        }

        self.renderer.update_buffers(
            &context.device,
            &context.queue,
            commands,
            self.clipped_primitives.as_slice(),
            &screen_descriptor,
        );

        let mut render_pass = commands
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("GUI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: surface_texture,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: Default::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            })
            .forget_lifetime();

        self.renderer.render(
            &mut render_pass,
            self.clipped_primitives.as_slice(),
            &screen_descriptor,
        );
    }

    pub fn free_textures(&mut self) {
        for id in &self.full_output.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
