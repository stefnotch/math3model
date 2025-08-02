use crate::{
    scene::MaterialInfo,
    texture::Texture,
    wgpu_context::{VIEW_FORMAT, WgpuContext},
};
use glam::Vec4;
use shaders::{compute_patches, render_patches};
use wesl::PkgResolver;
use wgpu::ShaderModule;

pub struct ShaderPipelines {
    pub compute_patches: wgpu::ComputePipeline,
    pub render: wgpu::RenderPipeline,
    pub shaders: [ShaderModule; 2],
}

impl ShaderPipelines {
    pub fn new(
        label: &str,
        code: &str,
        context: &WgpuContext,
    ) -> Result<Self, wgpu::CompilationMessage> {
        let (compute_patches, shader_a) =
            create_compute_patches_pipeline(label, &context.device, code)
                .map_err(error_to_compilation_message)?;
        let (render, shader_b) =
            create_render_pipeline(label, context, code).map_err(error_to_compilation_message)?;

        Ok(Self {
            compute_patches,
            render,
            shaders: [shader_a, shader_b],
        })
    }

    pub fn get_compilation_info(
        &self,
    ) -> impl Future<Output = Vec<wgpu::CompilationMessage>> + use<> {
        let comp_info_1 = self.shaders[0].get_compilation_info();
        let comp_info_2 = self.shaders[0].get_compilation_info();
        async move {
            let mut messages = comp_info_1.await.messages;
            messages.extend(comp_info_2.await.messages);
            messages
        }
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

fn error_to_compilation_message(error: wesl::Error) -> wgpu::CompilationMessage {
    let span = error_span(&error);
    wgpu::CompilationMessage {
        message: error.to_string(),
        message_type: wgpu::CompilationMessageType::Error,
        // TODO: Correctly set line num and line pos
        location: span.map(|span| wgpu::SourceLocation {
            line_number: 0,
            line_position: 0,
            offset: span.start as u32,
            length: span.range().len() as u32,
        }),
    }
}

fn error_span(error: &wesl::Error) -> Option<wesl::syntax::Span> {
    match error {
        wesl::Error::ParseError(error) => Some(error.span),
        wesl::Error::Error(diagnostic) => diagnostic.detail.span,
        _ => None,
    }
}

fn create_render_pipeline(
    label: &str,
    context: &WgpuContext,
    code: &str,
) -> Result<(wgpu::RenderPipeline, ShaderModule), wesl::Error> {
    let device = &context.device;
    let source = compile_shader("render_patches", code)?;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("Render Shader {label}")),
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(source.to_string())),
    });
    Ok((
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
    ))
}

pub fn create_compute_patches_pipeline(
    label: &str,
    device: &wgpu::Device,
    code: &str,
) -> Result<(wgpu::ComputePipeline, ShaderModule), wesl::Error> {
    let source = compile_shader("compute_patches", code)?;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(source.to_string())),
    });
    Ok((
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(&format!("Compute Patches {label}")),
            layout: Some(&compute_patches::create_pipeline_layout(device)),
            module: &shader,
            entry_point: Some(compute_patches::ENTRY_COMPUTE_PATCHES_MAIN),
            compilation_options: Default::default(),
            cache: Default::default(),
        }),
        shader,
    ))
}

fn compile_shader(
    name: &str,
    sample_object_code: &str,
) -> Result<wesl::CompileResult, wesl::Error> {
    let resolver = OverlayResolver::new(sample_object_code);
    // Work around current wesl limitations
    let compile_options = wesl::CompileOptions {
        strip: false,
        lazy: false,
        mangle_root: true,
        ..Default::default()
    };
    let entry_point =
        wesl::ModulePath::new(wesl::syntax::PathOrigin::Absolute, vec![name.to_string()]);

    wesl::compile_sourcemap(
        &entry_point,
        &resolver,
        &wesl::EscapeMangler,
        &compile_options,
    )
}

struct OverlayResolver<'a> {
    sample_object_code: &'a str,
    pkg_resolver: PkgResolver,
}

impl<'a> OverlayResolver<'a> {
    fn new(sample_object_code: &'a str) -> Self {
        let mut pkg_resolver = PkgResolver::new();
        pkg_resolver.add_package(&shaders::PACKAGE);
        Self {
            sample_object_code,
            pkg_resolver,
        }
    }
}

impl<'source> wesl::Resolver for OverlayResolver<'source> {
    fn resolve_source<'a>(
        &'a self,
        path: &wesl::ModulePath,
    ) -> Result<std::borrow::Cow<'a, str>, wesl::ResolveError> {
        if let &wesl::ModulePath {
            origin: wesl::syntax::PathOrigin::Absolute,
            ref components,
        } = path
            && components == &["parametric_fn"]
        {
            Ok(std::borrow::Cow::Borrowed(self.sample_object_code))
        } else if let &wesl::ModulePath {
            origin: wesl::syntax::PathOrigin::Absolute,
            ref components,
        } = path
        {
            self.pkg_resolver.resolve_source(&wesl::ModulePath {
                origin: wesl::syntax::PathOrigin::Package(shaders::PACKAGE.root.name.to_string()),
                components: components.clone(),
            })
        } else {
            self.pkg_resolver.resolve_source(path)
        }
    }
    fn display_name(&self, path: &wesl::ModulePath) -> Option<String> {
        self.pkg_resolver.display_name(path)
    }
}
