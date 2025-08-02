use crate::{
    buffer::{DeviceBufferExt, TypedBuffer},
    mesh::Mesh,
    renderer::virtual_model::ShaderPipelines,
    scene::{ShaderId, TextureData, TextureId, TextureInfo},
    texture::Texture,
    wgpu_context::WgpuContext,
};
use shaders::{compute_patches, copy_patches, utils};
use std::collections::HashMap;

pub const PATCH_SIZES: [u32; 5] = [2, 4, 8, 16, 32];
pub const MAX_PATCH_COUNT: u32 = 524_288;

pub type ArcShift<T> = std::sync::Arc<std::sync::RwLock<T>>;

pub struct ParametricRenderer {
    /// size/2 - 1 == one quad per four pixels
    pub quad_meshes: Vec<Mesh>,
    pub missing_shader: ArcShift<ShaderPipelines>,
    pub empty_texture: ArcShift<Texture>,
    pub shaders: HashMap<ShaderId, ArcShift<ShaderPipelines>>,
    pub textures: HashMap<TextureId, ArcShift<Texture>>,

    pub copy_patches_pipeline: wgpu::ComputePipeline,
    pub compute_patches: ComputePatches,
}

impl ParametricRenderer {
    pub fn new(context: &WgpuContext) -> Self {
        Self {
            quad_meshes: PATCH_SIZES
                .iter()
                .map(|size| *size / 2 - 1)
                .map(|splits| Mesh::new_tesselated_quad(&context.device, splits))
                .collect::<Vec<_>>(),
            missing_shader: ArcShift::new(
                ShaderPipelines::new("Missing Shader", shaders::DEFAULT_PARAMETRIC, context)
                    .unwrap()
                    .into(),
            ),
            empty_texture: ArcShift::new(
                Texture::new_rgba(
                    &context.device,
                    &context.queue,
                    &TextureInfo {
                        width: 1,
                        height: 1,
                        data: TextureData::Bytes(vec![u8::MAX, u8::MAX, u8::MAX, u8::MAX]),
                    },
                )
                .into(),
            ),
            shaders: Default::default(),
            textures: Default::default(),
            copy_patches_pipeline: context.device.create_compute_pipeline(
                &wgpu::ComputePipelineDescriptor {
                    label: Some("Copy Patches"),
                    layout: Some(&copy_patches::create_pipeline_layout(&context.device)),
                    module: &copy_patches::create_shader_module(&context.device),
                    entry_point: Some(copy_patches::ENTRY_MAIN),
                    compilation_options: Default::default(),
                    cache: Default::default(),
                },
            ),
            compute_patches: ComputePatches::new(context),
        }
    }
}

pub struct ComputePatches {
    pub patches_buffer_reset: TypedBuffer<utils::Patches>,
    pub indirect_compute_buffer_reset: TypedBuffer<utils::DispatchIndirectArgs>,
    pub force_render_false: TypedBuffer<compute_patches::ForceRenderFlag>,
    pub force_render_true: TypedBuffer<compute_patches::ForceRenderFlag>,
}

impl ComputePatches {
    pub fn new(context: &WgpuContext) -> Self {
        Self {
            patches_buffer_reset: context.device.storage_buffer_with_array(
                "Patches Buffer Reset",
                &utils::Patches {
                    patches_length: 0,
                    patches_capacity: MAX_PATCH_COUNT,
                    patches: vec![],
                },
                1,
                wgpu::BufferUsages::COPY_SRC,
            ),
            indirect_compute_buffer_reset: context.device.storage_buffer(
                "Indirect Compute Dispatch Buffer Reset",
                // We only write to x. y and z have their default value.
                &utils::DispatchIndirectArgs { x: 0, y: 1, z: 1 },
                wgpu::BufferUsages::COPY_SRC,
            ),
            force_render_false: context.device.uniform_buffer(
                "Disable Force Render",
                &compute_patches::ForceRenderFlag { flag: 0 },
                wgpu::BufferUsages::COPY_SRC,
            ),
            force_render_true: context.device.uniform_buffer(
                "Enable Force Render",
                &compute_patches::ForceRenderFlag { flag: 1 },
                wgpu::BufferUsages::COPY_SRC,
            ),
        }
    }
}
