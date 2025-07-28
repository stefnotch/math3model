use crate::{
    buffer::{CommandEncoderBufferExt, DeviceBufferExt, TypedBuffer},
    mesh::Mesh,
    renderer::{
        FrameData,
        parametric_renderer::{ComputePatches, MAX_PATCH_COUNT, PATCH_SIZES, ParametricRenderer},
        scene::SceneData,
        virtual_model::ShaderPipelines,
    },
    scene::{MaterialInfo, Model},
    texture::Texture,
    transform::Transform,
    wgpu_context::WgpuContext,
};
use encase::ShaderType;
use glam::UVec2;
use shaders::{compute_patches, copy_patches, render_patches, utils};
use std::sync::Arc;
use wgpu::Queue;

pub struct ParametricModel {
    model: TypedBuffer<render_patches::Model>,
    material: TypedBuffer<render_patches::Material>,
    t_diffuse: Texture,
    shader: Arc<ShaderPipelines>,
    render: ParametricModelRender,
    lod: ParametricModelLod,
}

pub struct ParametricModelLod {
    pub input_buffer: TypedBuffer<compute_patches::InputBuffer>,
    pub patches_buffer: [TypedBuffer<utils::Patches>; 2],
    pub indirect_compute_buffer: [TypedBuffer<utils::DispatchIndirectArgs>; 2],
    pub force_render_uniform: TypedBuffer<compute_patches::ForceRenderFlag>,
    pub bind_group_2: [compute_patches::bind_groups::BindGroup2; 2],
}

pub struct ParametricModelRender {
    pub render_buffer: Vec<TypedBuffer<utils::RenderBuffer>>,
    pub indirect_draw: TypedBuffer<Vec<copy_patches::DrawIndexedIndirectArgs>>,
    pub copy_patches_bind_group_0: copy_patches::bind_groups::BindGroup0,
    pub compute_patches_bind_group_1: compute_patches::bind_groups::BindGroup1,
}

impl ParametricModel {
    pub fn new(context: &WgpuContext, renderer: &ParametricRenderer, model: &Model) -> Self {
        let device = &context.device;

        Self {
            model: device.uniform_buffer(
                "Model Buffer",
                &render_patches::Model {
                    model_similarity: glam::Mat4::IDENTITY,
                    object_id: 0,
                },
                wgpu::BufferUsages::COPY_DST,
            ),
            material: device.uniform_buffer(
                "Material Buffer",
                &MaterialInfo::missing().to_shader(),
                wgpu::BufferUsages::COPY_DST,
            ),
            t_diffuse: model
                .material_info
                .diffuse_texture
                .as_ref()
                .and_then(|id| renderer.textures.get(id).cloned())
                .unwrap_or_else(|| renderer.empty_texture.clone()),
            shader: renderer
                .shaders
                .get(&model.shader_id)
                .cloned()
                .unwrap_or_else(|| renderer.missing_shader.clone()),

            lod: ParametricModelLod::new(context),
            render: ParametricModelRender::new(context, &renderer.quad_meshes),
        }
    }
    pub fn update(
        &self,
        queue: &Queue,
        screen_size: UVec2,
        transform: Transform,
        material_info: &MaterialInfo,
        threshold_factor: f32,
        render_data: &FrameData,
    ) {
        self.model.write_buffer(
            queue,
            &render_patches::Model {
                model_similarity: transform.to_matrix(),
                object_id: 0, // TODO: set this
            },
        );
        self.material
            .write_buffer(queue, &material_info.to_shader());

        let model_view_projection = render_data.camera.projection_matrix(screen_size)
            * render_data.camera.view_matrix()
            * transform.to_matrix();
        self.lod.input_buffer.write_buffer(
            queue,
            &compute_patches::InputBuffer {
                model_view_projection,
                threshold_factor,
            },
        );
    }

    pub fn lod_stage(
        &mut self,
        context: &WgpuContext,
        scene_data: &SceneData,
        compute_patches: &ComputePatches,
        copy_patches_pipeline: &wgpu::ComputePipeline,
        instance_count: u32,

        commands: &mut wgpu_profiler::Scope<'_, wgpu::CommandEncoder>,
    ) {
        let queue = &context.queue;
        self.lod
            .force_render_uniform
            .write_buffer(queue, &compute_patches::ForceRenderFlag { flag: 0 });

        self.lod.patches_buffer[0].write_buffer(
            queue,
            &utils::Patches {
                patches_length: instance_count,
                patches_capacity: MAX_PATCH_COUNT,
                patches: (0..instance_count)
                    .map(|i| {
                        utils::EncodedPatch {
                            // Just the leading 1 bit
                            u: 1,
                            v: 1,
                            instance: i,
                        }
                    })
                    .collect(),
            },
        );
        self.lod.indirect_compute_buffer[0].write_buffer(
            queue,
            &utils::DispatchIndirectArgs {
                x: instance_count,
                y: 1,
                z: 1,
            },
        );

        let render_buffer_reset = utils::RenderBuffer {
            patches_length: 0,
            patches_capacity: MAX_PATCH_COUNT,
            patches: vec![],
        };
        for render_buffer in self.render.render_buffer.iter() {
            render_buffer.write_buffer(queue, &render_buffer_reset);
        }

        // Each round, we do a ping-pong and pong-ping
        // 2*4 rounds is enough to subdivide a 4k screen into 16x16 pixel patches
        let double_number_of_rounds = 4;
        for i in 0..double_number_of_rounds {
            let is_last_round = i == double_number_of_rounds - 1;
            // TODO: Should I create many compute passes, or just one?
            {
                commands.copy_tbuffer_to_tbuffer(
                    &compute_patches.patches_buffer_reset,
                    &self.lod.patches_buffer[1],
                );
                commands.copy_tbuffer_to_tbuffer(
                    &compute_patches.indirect_compute_buffer_reset,
                    &self.lod.indirect_compute_buffer[1],
                );
                let mut compute_pass =
                    commands.scoped_compute_pass(format!("Compute Patches From-To {i}"));
                compute_pass.set_pipeline(&self.shader.compute_patches);
                compute_patches::set_bind_groups(
                    &mut compute_pass.recorder,
                    &scene_data.scene_bind_group_compute,
                    &self.render.compute_patches_bind_group_1,
                    &self.lod.bind_group_2[0],
                );
                compute_pass.dispatch_workgroups_indirect(&self.lod.indirect_compute_buffer[0], 0);
            }
            if is_last_round {
                commands.copy_tbuffer_to_tbuffer(
                    &compute_patches.force_render_true,
                    &self.lod.force_render_uniform,
                );
            }
            {
                commands.copy_tbuffer_to_tbuffer(
                    &compute_patches.patches_buffer_reset,
                    &self.lod.patches_buffer[0],
                );
                commands.copy_tbuffer_to_tbuffer(
                    &compute_patches.indirect_compute_buffer_reset,
                    &self.lod.indirect_compute_buffer[0],
                );
                let mut compute_pass =
                    commands.scoped_compute_pass(format!("Compute Patches To-From {i}"));
                compute_pass.set_pipeline(&self.shader.compute_patches);
                compute_patches::set_bind_groups(
                    &mut compute_pass.recorder,
                    &scene_data.scene_bind_group_compute,
                    &self.render.compute_patches_bind_group_1,
                    &self.lod.bind_group_2[1],
                );
                compute_pass.dispatch_workgroups_indirect(&self.lod.indirect_compute_buffer[1], 0);
            }
            if is_last_round {
                commands.copy_tbuffer_to_tbuffer(
                    &compute_patches.force_render_false,
                    &self.lod.force_render_uniform,
                );
            }
        }
        {
            let mut compute_pass = commands.scoped_compute_pass("Copy Patch Sizes Pass");
            compute_pass.set_pipeline(copy_patches_pipeline);
            copy_patches::set_bind_groups(
                &mut compute_pass.recorder,
                &self.render.copy_patches_bind_group_0,
            );
            compute_pass.dispatch_workgroups(1, 1, 1);
        }
    }

    pub fn render(
        &mut self,
        context: &WgpuContext,
        render_pass: &mut wgpu_profiler::OwningScope<'_, wgpu::RenderPass<'_>>,
        scene_bind_group: &render_patches::bind_groups::BindGroup0,
        quad_meshes: &[Mesh],
    ) {
        render_pass.set_pipeline(&self.shader.render);

        for (i, (render, mesh)) in self
            .render
            .render_buffer
            .iter()
            .zip(quad_meshes.iter())
            .enumerate()
        {
            let buffer_offset = (i as u64)
                * Vec::<copy_patches::DrawIndexedIndirectArgs>::METADATA
                    .extra
                    .stride
                    .get();
            render_patches::set_bind_groups(
                &mut render_pass.recorder,
                scene_bind_group,
                &render_patches::bind_groups::BindGroup1::from_bindings(
                    &context.device,
                    render_patches::bind_groups::BindGroupLayout1 {
                        model: self.model.as_buffer_binding(),
                        render_buffer: render.as_buffer_binding(),
                        material: self.material.as_buffer_binding(),
                        t_diffuse: &self.t_diffuse.view,
                    },
                ),
            );
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed_indirect(&self.render.indirect_draw, buffer_offset);
        }
    }
}

impl ParametricModelLod {
    pub fn new(context: &WgpuContext) -> Self {
        let input_buffer = context.device.uniform_buffer(
            "Compute Patches Input Buffer",
            &compute_patches::InputBuffer {
                model_view_projection: glam::Mat4::IDENTITY,
                threshold_factor: 1.0,
            },
            wgpu::BufferUsages::COPY_DST,
        );
        let patches_buffer_empty = utils::Patches {
            patches_length: 0,
            patches_capacity: 0,
            patches: vec![],
        };

        let force_render_uniform = context.device.uniform_buffer(
            "Force Render Uniform",
            &compute_patches::ForceRenderFlag { flag: 0 },
            wgpu::BufferUsages::COPY_DST,
        );

        let patches_buffer = [
            context.device.storage_buffer_with_array(
                "Patches Buffer 0",
                &patches_buffer_empty,
                MAX_PATCH_COUNT as u64,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            ),
            context.device.storage_buffer_with_array(
                "Patches Buffer 1",
                &patches_buffer_empty,
                MAX_PATCH_COUNT as u64,
                wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            ),
        ];

        let indirect_compute_buffer = [
            context.device.storage_buffer(
                "Indirect Compute Dispatch Buffer 0",
                // None of these values will ever be read
                &utils::DispatchIndirectArgs { x: 0, y: 0, z: 0 },
                wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            ),
            context.device.storage_buffer(
                "Indirect Compute Dispatch Buffer 1",
                &utils::DispatchIndirectArgs { x: 0, y: 0, z: 0 },
                wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
            ),
        ];

        Self {
            bind_group_2: [
                compute_patches::bind_groups::BindGroup2::from_bindings(
                    &context.device,
                    compute_patches::bind_groups::BindGroupLayout2 {
                        input_buffer: input_buffer.as_buffer_binding(),
                        patches_from_buffer: patches_buffer[0].as_buffer_binding(),
                        patches_to_buffer: patches_buffer[1].as_buffer_binding(),
                        dispatch_next: indirect_compute_buffer[1].as_buffer_binding(),
                        force_render: force_render_uniform.as_buffer_binding(),
                    },
                ),
                compute_patches::bind_groups::BindGroup2::from_bindings(
                    &context.device,
                    compute_patches::bind_groups::BindGroupLayout2 {
                        input_buffer: input_buffer.as_buffer_binding(),
                        patches_from_buffer: patches_buffer[1].as_buffer_binding(), // Swap the order :)
                        patches_to_buffer: patches_buffer[0].as_buffer_binding(),
                        dispatch_next: indirect_compute_buffer[0].as_buffer_binding(),
                        force_render: force_render_uniform.as_buffer_binding(),
                    },
                ),
            ],
            input_buffer,
            patches_buffer,
            indirect_compute_buffer,
            force_render_uniform,
        }
    }
}

impl ParametricModelRender {
    pub fn new(context: &WgpuContext, meshes: &[Mesh]) -> Self {
        let render_buffer_initial = utils::RenderBuffer {
            patches_length: 0,
            patches_capacity: 0,
            patches: vec![],
        };
        let render_buffer: Vec<_> = PATCH_SIZES
            .iter()
            .map(|size| {
                context.device.storage_buffer_with_array(
                    &format!("Render Buffer {size}"),
                    &render_buffer_initial,
                    MAX_PATCH_COUNT as u64,
                    wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
                )
            })
            .collect();

        let indirect_draw_data = copy_patches::DrawIndexedIndirectArgs {
            index_count: 0,
            instance_count: 0, // Our shader sets this
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        };

        let indirect_draw = context.device.storage_buffer(
            "Indirect Draw Buffers",
            &meshes
                .iter()
                .map(|mesh| copy_patches::DrawIndexedIndirectArgs {
                    index_count: mesh.num_indices,
                    ..indirect_draw_data
                })
                .collect::<Vec<_>>(),
            wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_SRC,
        );

        Self {
            // TODO: Share those bind groups
            copy_patches_bind_group_0: copy_patches::bind_groups::BindGroup0::from_bindings(
                &context.device,
                copy_patches::bind_groups::BindGroupLayout0 {
                    render_buffer_2: render_buffer[0].as_buffer_binding(),
                    render_buffer_4: render_buffer[1].as_buffer_binding(),
                    render_buffer_8: render_buffer[2].as_buffer_binding(),
                    render_buffer_16: render_buffer[3].as_buffer_binding(),
                    render_buffer_32: render_buffer[4].as_buffer_binding(),
                    indirect_draw: indirect_draw.as_buffer_binding(),
                },
            ),
            compute_patches_bind_group_1: compute_patches::bind_groups::BindGroup1::from_bindings(
                &context.device,
                compute_patches::bind_groups::BindGroupLayout1 {
                    render_buffer_2: render_buffer[0].as_buffer_binding(),
                    render_buffer_4: render_buffer[1].as_buffer_binding(),
                    render_buffer_8: render_buffer[2].as_buffer_binding(),
                    render_buffer_16: render_buffer[3].as_buffer_binding(),
                    render_buffer_32: render_buffer[4].as_buffer_binding(),
                },
            ),
            render_buffer,
            indirect_draw,
        }
    }
}
