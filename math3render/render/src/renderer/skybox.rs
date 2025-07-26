use glam::Vec3;

use crate::{
    buffer::{DeviceBufferExt, TypedBuffer},
    mesh::Mesh,
    renderer::FrameData,
    shaders::skybox,
    texture::Texture,
    wgpu_context::{VIEW_FORMAT, WgpuContext, WgpuSurface},
};

pub struct Skybox {
    mesh: Mesh,
    pipeline: wgpu::RenderPipeline,
    uniforms: TypedBuffer<skybox::Uniforms>,
    bind_group_0: skybox::bind_groups::BindGroup0,
}

impl Skybox {
    pub fn new(context: &WgpuContext) -> Self {
        let shader = context
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Skybox"),
                source: wgpu::ShaderSource::Wgsl(skybox::SOURCE.into()),
            });

        let uniforms = context.device.uniform_buffer(
            "Skybox Uniforms",
            &skybox::Uniforms {
                view_projection_matrix: Default::default(),
                background_color: Default::default(),
                sun_direction: Default::default(),
            },
            wgpu::BufferUsages::COPY_DST,
        );

        let bind_group_0 = skybox::bind_groups::BindGroup0::from_bindings(
            &context.device,
            skybox::bind_groups::BindGroupLayout0 {
                uniforms: uniforms.as_buffer_binding(),
            },
        );

        let pipeline = context
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Skybox"),
                layout: Some(&skybox::create_pipeline_layout(&context.device)),
                vertex: skybox::vertex_state(
                    &shader,
                    &skybox::vs_main_entry(wgpu::VertexStepMode::Vertex),
                ),
                fragment: Some(skybox::fragment_state(
                    &shader,
                    &skybox::fs_main_entry([
                        Some(wgpu::ColorTargetState {
                            format: VIEW_FORMAT,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                        Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::R32Uint,
                            blend: None,
                            write_mask: wgpu::ColorWrites::empty(),
                        }),
                    ]),
                )),
                primitive: wgpu::PrimitiveState {
                    cull_mode: Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: false,
                    // Reverse-Z range: [1.0, 0.0]
                    // Depth buffer starts at far-away (0.0)
                    // Skybox is at 0.0 (faaar away), and we want to replace the depth buffer values
                    depth_compare: wgpu::CompareFunction::GreaterEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                multiview: None,
                cache: Default::default(),
            });

        Self {
            mesh: Mesh::cubemap_cube(&context.device, Vec3::NEG_ONE, Vec3::ONE),
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
        let view_projection_matrix = render_data.camera.projection_matrix(surface.size())
            * glam::Mat4::from_mat3(glam::Mat3::from_mat4(render_data.camera.view_matrix()));
        self.uniforms.write_buffer(
            &context.queue,
            &skybox::Uniforms {
                view_projection_matrix,
                background_color: (Vec3::new(0.09, 0.59, 0.85) * 0.8).extend(1.0),
                sun_direction: Vec3::new(1., 1., 1.).normalize().extend(1.0),
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
