use crate::{
    scene::MaterialInfo,
    texture::Texture,
    wgpu_context::{VIEW_FORMAT, WgpuContext},
};
use glam::Vec4;
use shaders::{compute_patches, render_patches};
use wgpu::ShaderModule;

pub struct ShaderPipelines {
    /// Pipeline per model, for different parametric functions.
    pub compute_patches: wgpu::ComputePipeline,
    /// Pipeline per model, for different parametric functions.
    pub render: wgpu::RenderPipeline,
    pub shaders: [ShaderModule; 2],
}

impl ShaderPipelines {
    pub fn new(label: &str, code: &str, context: &WgpuContext) -> Self {
        let (compute_patches, shader_a) =
            create_compute_patches_pipeline(label, &context.device, code);
        let (render, shader_b) = create_render_pipeline(label, context, code);

        Self {
            compute_patches,
            render,
            shaders: [shader_a, shader_b],
        }
    }

    pub async fn get_compilation_info(&self) -> Vec<wgpu::CompilationMessage> {
        let mut messages = self.shaders[0].get_compilation_info().await.messages;
        messages.extend(self.shaders[1].get_compilation_info().await.messages);
        messages
    }
}

impl MaterialInfo {
    pub fn to_shader(&self) -> render_patches::Material {
        render_patches::Material {
            color_roughness: Vec4::new(self.color.x, self.color.y, self.color.z, self.roughness),
            emissive_metallic: self.emissive.extend(self.metallic),
            has_texture: if self.diffuse_texture.is_some() { 1 } else { 0 },
            texture_scale: self.texture_scale,
        }
    }
}

fn create_render_pipeline(
    label: &str,
    context: &WgpuContext,
    code: &str,
) -> (wgpu::RenderPipeline, ShaderModule) {
    let device = &context.device;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("Render Shader {label}")),
        source: wgpu::ShaderSource::Wgsl(replace_render_code(render_patches::SOURCE, code).into()),
    });
    (
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("Render Pipeline {label}")),
            layout: Some(&render_patches::create_pipeline_layout(device)),
            vertex: render_patches::vertex_state(
                &shader,
                &render_patches::vs_main_entry(wgpu::VertexStepMode::Vertex),
            ),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(render_patches::ENTRY_FS_MAIN),
                targets: &[
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
                ],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill, // Wireframe mode can be toggled here on the desktop backend
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Greater, // Reverse Z
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview: None,
            cache: Default::default(),
        }),
        shader,
    )
}

pub fn create_compute_patches_pipeline(
    label: &str,
    device: &wgpu::Device,
    code: &str,
) -> (wgpu::ComputePipeline, ShaderModule) {
    let source = replace_compute_code(compute_patches::SOURCE, code);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(source.as_ref())),
    });
    (
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("Compute Patches {label}")),
            layout: Some(&compute_patches::create_pipeline_layout(device)),
            module: &shader,
            entry_point: Some(compute_patches::ENTRY_COMPUTE_PATCHES_MAIN),
            compilation_options: Default::default(),
            cache: Default::default(),
        }),
        shader,
    )
}

fn replace_render_code(source: &str, sample_object_code: &str) -> String {
    // LATER use wesl-rs instead of this
    let range_1 = fn_range("fn package__1render_patches_sampleObject", source);
    let range_2 = fn_range("fn package__1render_patches_getColor", source);

    let mut result = String::new();
    result.push_str(&source[..range_1.start]);
    result.push_str(sample_object_code);
    result.push_str("fn package__1render_patches_sampleObject(input: vec2f) -> vec3f { return sampleObject(input); }\n");
    result.push_str(&source[range_1.end..range_2.start]);

    if sample_object_code.contains("fn getColor") {
        result.push_str("fn package__1render_patches_getColor(input: vec2f) -> vec3f { return getColor(input); }\n");
        result.push_str(&source[range_2.end..]);
    } else {
        result.push_str(&source[range_2.start..]);
    }

    result
}

fn replace_compute_code(source: &str, sample_object_code: &str) -> String {
    // LATER use wesl-rs instead of this
    let range_1 = fn_range("fn package__1compute_patches_sampleObject", source);

    let mut result = String::new();
    result.push_str(&source[..range_1.start]);
    if sample_object_code.contains("fn getColor") {
        let get_color = sample_object_code.find("fn getColor").unwrap();
        result.push_str(&sample_object_code[0..get_color]);
    } else {
        result.push_str(sample_object_code);
    }
    result.push_str("fn package__1compute_patches_sampleObject(input: vec2f) -> vec3f { return sampleObject(input); }\n");
    result.push_str(&source[range_1.end..]);
    result
}

fn fn_range(fn_header: &str, source: &str) -> std::ops::Range<usize> {
    let start = source.find(fn_header).unwrap();
    // LATER Count the curly braces (parsing)
    let end = source[start..].find("}").unwrap() + 1 + start;
    start..end
}
