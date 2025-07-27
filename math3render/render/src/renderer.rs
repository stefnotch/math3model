mod frame_data;
mod ground_plane;
pub mod parametric_model;
pub mod parametric_renderer;
mod scene;
mod skybox;
mod virtual_model;

pub use frame_data::FrameData;
use ground_plane::GroundPlane;
use skybox::Skybox;

use std::sync::Arc;

use glam::UVec2;

use scene::SceneData;
use virtual_model::ShaderPipelines;
use wgpu_profiler::GpuProfiler;

use crate::{
    game::GameRes,
    gui::GuiRender,
    renderer::{parametric_model::ParametricModel, parametric_renderer::ParametricRenderer},
    scene::{Model, ShaderId, TextureId, TextureInfo},
    texture::Texture,
    time::{FrameCounter, Seconds},
    wgpu_context::{WgpuContext, WgpuSurface, create_profiler},
};

pub struct GpuApplication {
    pub context: Arc<WgpuContext>,
    profiler: GpuProfiler,
    force_wait: bool,
    /// Sets the threshold factor for the LOD algorithm
    threshold_factor: f32,
    frame_counter: FrameCounter,
    scene_data: SceneData,
    depth_texture: Texture,
    object_id_texture: Texture,
    skybox: Skybox,
    ground_plane: GroundPlane,
    parametric_renderer: ParametricRenderer,
    models: Vec<(Model, ParametricModel)>,
}

impl GpuApplication {
    pub fn new(context: WgpuContext) -> Self {
        let context = Arc::new(context);
        Self {
            profiler: create_profiler(&context),
            threshold_factor: 1.0,
            force_wait: false,
            frame_counter: FrameCounter::new(),
            depth_texture: Texture::create_depth_texture(
                &context.device,
                UVec2::ONE,
                "Init Depth Texture",
            ),
            object_id_texture: Texture::create_object_id_texture(
                &context.device,
                UVec2::ONE,
                "Init Object ID Texture",
            ),
            scene_data: SceneData::new(&context.device),
            skybox: Skybox::new(&context),
            ground_plane: GroundPlane::new(&context),
            parametric_renderer: ParametricRenderer::new(&context),
            models: Vec::new(),
            context,
        }
    }

    pub fn update_models(&mut self, game_models: &[Model]) {
        for ((model_info, parametric_model), game_model) in
            self.models.iter_mut().zip(game_models.iter())
        {
            if model_info == game_model {
                continue;
            }

            if model_info.shader_id != game_model.shader_id
                || model_info.material_info.diffuse_texture
                    != game_model.material_info.diffuse_texture
            {
                // Recreate
                *parametric_model =
                    ParametricModel::new(&self.context, &self.parametric_renderer, game_model);
            }

            *model_info = game_model.clone();
        }
        match self.models.len().cmp(&game_models.len()) {
            std::cmp::Ordering::Less => {
                for game_model in game_models.iter().skip(self.models.len()) {
                    let parametric_model =
                        ParametricModel::new(&self.context, &self.parametric_renderer, game_model);
                    self.models.push((game_model.clone(), parametric_model));
                }
            }
            std::cmp::Ordering::Equal => {}
            std::cmp::Ordering::Greater => self.models.truncate(game_models.len()),
        }
    }

    pub fn set_shader(
        &mut self,
        shader_id: ShaderId,
        info: &crate::scene::ShaderInfo,
    ) -> impl Future<Output = Result<(), Vec<wgpu::CompilationMessage>>> + use<> {
        let new_shaders = Arc::new(ShaderPipelines::new(&info.label, &info.code, &self.context));
        // Make sure to do this synchronously, otherwise this function would have a race condition
        self.parametric_renderer
            .shaders
            .insert(shader_id, new_shaders.clone());

        async move {
            let compilation_results = new_shaders.get_compilation_info().await;
            let is_error = compilation_results
                .iter()
                .any(|v| v.message_type == wgpu::CompilationMessageType::Error);

            if is_error {
                Err(compilation_results)
            } else {
                Ok(())
            }
        }
    }

    pub fn remove_shader(&mut self, shader_id: &ShaderId) {
        self.parametric_renderer.shaders.remove(shader_id);
    }

    pub fn set_texture(&mut self, id: TextureId, info: &TextureInfo) {
        let texture = Texture::new_rgba(&self.context.device, &self.context.queue, info);
        self.parametric_renderer.textures.insert(id, texture);
    }

    pub fn remove_texture(&mut self, id: &TextureId) {
        self.parametric_renderer.textures.remove(id);
    }

    pub fn render(
        &mut self,
        surface: &mut WgpuSurface,
        game: &GameRes,
        gui_render: GuiRender<'_>,
    ) -> Result<Option<RenderResults>, wgpu::SurfaceError> {
        let profiling_enabled = game.profiler_settings.gpu;
        if self.profiler.settings().enable_timer_queries != profiling_enabled {
            self.profiler
                .change_settings(wgpu_profiler::GpuProfilerSettings {
                    enable_timer_queries: profiling_enabled,
                    enable_debug_groups: profiling_enabled,
                    ..Default::default()
                })
                .unwrap();
        }

        let render_data = FrameData {
            camera: game.camera.clone(),
            mouse_pos: game.mouse,
            mouse_held: game.mouse_held,
        };

        self.render_internal(surface, &render_data, gui_render)
    }

    pub fn resize(&mut self, surface: &mut WgpuSurface, new_size: UVec2) {
        let new_size = surface
            .try_resize(&self.context, new_size)
            .unwrap_or(new_size);
        if self.depth_texture.size2d() != new_size {
            self.depth_texture =
                Texture::create_depth_texture(&self.context.device, new_size, "Depth Texture");
            self.object_id_texture = Texture::create_object_id_texture(
                &self.context.device,
                new_size,
                "Object ID Texture",
            );
        }
    }

    pub fn force_wait(&mut self) {
        self.force_wait = true;
    }

    pub fn set_threshold_factor(&mut self, factor: f32) {
        self.threshold_factor = factor.clamp(0.0001, 100000.0);
    }
}

impl GpuApplication {
    pub fn render_internal(
        &mut self,
        surface: &WgpuSurface,
        render_data: &FrameData,
        mut gui_render: GuiRender<'_>,
    ) -> Result<Option<RenderResults>, wgpu::SurfaceError> {
        let context = &self.context;

        let frame_time = self.frame_counter.new_frame();
        // 2. Render
        let surface_texture = match surface.surface_texture() {
            Ok(v) => v,
            err @ Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Roughly based on https://github.com/gfx-rs/wgpu/blob/a0c185a28c232ee2ab63f72d6fd3a63a3f787309/examples/src/framework.rs#L216
                surface.recreate_swapchain(&context);
                return err.map(|_| None);
            }
            err => {
                return err.map(|_| None);
            }
        };

        self.scene_data
            .update(surface.size(), &render_data, &frame_time, &context.queue);

        for (model_info, parametric_model) in self.models.iter() {
            parametric_model.update(
                &self.context.queue,
                surface.size(),
                model_info.transform,
                &model_info.material_info,
                self.threshold_factor,
                render_data,
            );
        }

        let mut command_encoder =
            context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            // Profiling
            let mut commands = self.profiler.scope("Render", &mut command_encoder);

            for (model_info, parametric_model) in self.models.iter_mut() {
                parametric_model.lod_stage(
                    context,
                    &self.scene_data,
                    &self.parametric_renderer.compute_patches,
                    &self.parametric_renderer.copy_patches_pipeline,
                    model_info.instance_count,
                    &mut commands,
                );
            }

            let render_pass_descriptor = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: surface_texture.texture_view(),
                        resolve_target: None,
                        ops: Default::default(),
                        depth_slice: Default::default(),
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.object_id_texture.view,
                        resolve_target: None,
                        ops: Default::default(),
                        depth_slice: Default::default(),
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0), // Reverse Z checklist https://iolite-engine.com/blog_posts/reverse_z_cheatsheet
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            };

            {
                let mut render_pass =
                    commands.scoped_render_pass("Render Pass", render_pass_descriptor.clone());

                // Render the models
                for (_, parametric_model) in self.models.iter_mut() {
                    parametric_model.render(
                        context,
                        &mut render_pass,
                        &self.scene_data.scene_bind_group,
                        &self.parametric_renderer.quad_meshes,
                    );
                }

                // Skybox is rendered after opaque objects
                self.skybox.update(context, surface, render_data);
                self.skybox.render(&mut render_pass);

                // And now overlay transparent objects
                self.ground_plane.update(context, surface, render_data);
                self.ground_plane.render(&mut render_pass);
            }
            gui_render.render(
                context,
                surface,
                &surface_texture.texture_view(),
                &mut commands.recorder,
            );
        };
        self.profiler.resolve_queries(&mut command_encoder);
        context
            .queue
            .submit(std::iter::once(command_encoder.finish()));

        surface.pre_present_notify();

        if self.force_wait {
            context.instance.poll_all(true);
        }

        surface_texture.present();

        gui_render.free_textures();

        self.profiler.end_frame().unwrap();
        let render_results = Some(RenderResults {
            delta_time: frame_time.delta,
            profiler_results: if self.profiler.settings().enable_timer_queries {
                self.profiler
                    .process_finished_frame(context.queue.get_timestamp_period())
            } else {
                None
            },
        });

        if self.force_wait {
            context.instance.poll_all(true);
        }

        Ok(render_results)
    }
}

#[derive(Default)]
pub struct RenderResults {
    pub delta_time: Seconds,
    pub profiler_results: Option<Vec<wgpu_profiler::GpuTimerQueryResult>>,
}
