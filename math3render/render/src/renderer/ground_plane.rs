use crate::{
    buffer::DeviceBufferExt,
    mesh::Mesh,
    renderer::FrameData,
    shaders::ground_plane,
    texture::Texture,
    wgpu_context::{VIEW_FORMAT, WgpuContext, WgpuSurface},
};

pub struct GroundPlane {
    mesh: Mesh,
    pipeline: wgpu::RenderPipeline,
    uniforms: crate::buffer::TypedBuffer<ground_plane::Uniforms>,
    bind_group_0: ground_plane::bind_groups::BindGroup0,
}

impl GroundPlane {
    pub fn new(context: &WgpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Ground Plane Grid"),
                source: wgpu::ShaderSource::Wgsl(ground_plane::SOURCE.into()),
            });

        let uniforms = context.device.uniform_buffer(
            "Ground Plane Uniforms",
            &ground_plane::Uniforms {
                model_matrix: Default::default(),
                view_projection_matrix: Default::default(),
                grid_scale: 0.0,
            },
            wgpu::BufferUsages::COPY_DST,
        );

        let bind_group_0 = ground_plane::bind_groups::BindGroup0::from_bindings(
            &context.device,
            ground_plane::bind_groups::BindGroupLayout0 {
                uniforms: uniforms.as_buffer_binding(),
            },
        );

        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Ground Plane Grid"),
                layout: Some(&ground_plane::create_pipeline_layout(&context.device)),
                vertex: ground_plane::vertex_state(
                    &shader,
                    &ground_plane::vs_main_entry(wgpu::VertexStepMode::Vertex),
                ),
                fragment: Some(ground_plane::fragment_state(
                    &shader,
                    &ground_plane::fs_main_entry([
                        Some(wgpu::ColorTargetState {
                            format: VIEW_FORMAT,
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::R32Uint,
                            blend: None,
                            write_mask: wgpu::ColorWrites::empty(),
                        }),
                    ]),
                )),
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: Default::default(),
            });

        Self {
            mesh: Mesh::new_tesselated_quad(&context.device, 2),
            pipeline,
            uniforms,
            bind_group_0,
        }
    }

    pub fn update(
        &mut self,
        context: &WgpuContext,
        surface: &WgpuSurface,
        render_data: &FrameData,
    ) {
        let size = 100.0;
        let grid_scale = 1.0;
        self.uniforms.write_buffer(
            &context.queue,
            &ground_plane::Uniforms {
                model_matrix: glam::Mat4::from_scale_rotation_translation(
                    glam::Vec3::splat(size),
                    glam::Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    glam::Vec3::new(-size / 2., 0.0, size / 2.),
                ),
                view_projection_matrix: render_data.view_projection_matrix(surface.size()),
                grid_scale,
            },
        );
    }

    pub fn render(&self, render_pass: &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.bind_group_0.set(&mut render_pass.recorder);
        render_pass.draw_indexed(0..self.mesh.num_indices, 0, 0..1);
    }
}
