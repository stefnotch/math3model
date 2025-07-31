use std::{error::Error, path::Path};
use wesl::{
    CompileOptions, EscapeMangler, Mangler, ModulePath, StandardResolver, compile_sourcemap,
    emit_rerun_if_changed, syntax::PathOrigin,
};
use wgsl_to_wgpu::{MatrixVectorTypes, TypePath, WriteOptions};

/// Taken from https://docs.rs/build-print/latest/build_print/macro.println.html
macro_rules! log {
    () => {
        ::std::println!("cargo:warning=\x1b[2K\r");
    };
    ($($arg:tt)*) => {
        ::std::println!("cargo:warning=\x1b[2K\r{}", ::std::format!($($arg)*));
    }
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let start_time = std::time::Instant::now();
    if let Err(err) = compile_shaders(&out_dir) {
        eprintln!("{err:#?}");
    }
    log!(
        "Compiled shaders {:?} in {}ms",
        &out_dir,
        start_time.elapsed().as_millis()
    );
}

fn compile_shaders(out_dir: &Path) -> Result<(), Box<dyn Error>> {
    let shader_directory = "wgsl";
    let mut shader_compiler = ShaderCompiler {
        resolver: StandardResolver::new(shader_directory),
        // Work around current wesl limitations
        compile_options: CompileOptions {
            strip: false,
            lazy: false,
            mangle_root: true,
            ..Default::default()
        },
        out_module: wgsl_to_wgpu::Module::default(),
        write_options: WriteOptions {
            // We need to use bytemuck for vertex buffer
            derive_bytemuck_vertex: true,
            derive_bytemuck_host_shareable: false,
            // And encase for uniform buffers and storage buffers
            derive_encase_host_shareable: true,
            derive_serde: false,
            matrix_vector_types: MatrixVectorTypes::Glam,
            rustfmt: false,
            validate: None,
        },
        out_dir,
    };

    shader_compiler.compile("compute_patches")?;
    shader_compiler.compile("copy_patches")?;
    shader_compiler.compile("ground_plane")?;
    shader_compiler.compile("skybox")?;
    shader_compiler.compile("render_patches")?;

    wesl::PkgBuilder::new("my_package")
        .scan_root(shader_directory)?
        .build_artifact()?;

    std::fs::write(
        shader_compiler.out_dir.join("shaders.rs"),
        shader_compiler
            .out_module
            .to_generated_bindings(shader_compiler.write_options),
    )?;
    Ok(())
}

struct ShaderCompiler<'a> {
    resolver: wesl::StandardResolver,
    compile_options: wesl::CompileOptions,
    out_module: wgsl_to_wgpu::Module,
    write_options: wgsl_to_wgpu::WriteOptions,
    out_dir: &'a Path,
}

impl<'a> ShaderCompiler<'a> {
    fn compile(&mut self, shader_name: &str) -> Result<wesl::CompileResult, Box<dyn Error>> {
        let entry_point = ModulePath::new(PathOrigin::Absolute, vec![shader_name.to_string()]);
        let compiled = compile_sourcemap(
            &entry_point,
            &self.resolver,
            &EscapeMangler,
            &self.compile_options,
        )
        .inspect_err(|e| {
            eprintln!("failed to build WESL shader. {entry_point}\n{e}");
        })?;
        emit_rerun_if_changed(&compiled.modules, &self.resolver);
        let compiled_code = compiled.to_string();
        self.out_module
            .add_shader_module(
                &compiled_code,
                None,
                self.write_options,
                wgsl_to_wgpu::ModulePath {
                    components: vec![shader_name.to_string()],
                },
                wesl_unmangler,
            )
            .inspect_err(|e| {
                eprintln!("failed to build WESL shader. {entry_point}\n{e}");
            })?;

        std::fs::write(
            self.out_dir.join(format!("{shader_name}.wgsl")),
            &compiled_code,
        )?;

        Ok(compiled)
    }
}

fn wesl_unmangler(mangled_name: &str) -> TypePath {
    let Some((parent, name)) = EscapeMangler.unmangle(mangled_name) else {
        panic!("Failed to unmangle {mangled_name}")
    };

    assert_eq!(
        parent.origin,
        wesl::syntax::PathOrigin::Absolute,
        "Expected path ({parent:?}) from name ({mangled_name}) to be an absolute path",
    );

    TypePath {
        parent: wgsl_to_wgpu::ModulePath {
            components: parent.components,
        },
        name,
    }
}
#[allow(dead_code)]
struct RustStrLiteral<'a>(&'a str);
impl std::fmt::Display for RustStrLiteral<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut max_hash_count = 0;
        let mut chars = self.0.chars();
        while chars.any(|c| c == '#') {
            let count = 1 + chars.by_ref().take_while(|c| *c == '#').count();
            max_hash_count = max_hash_count.max(count);
        }

        write!(f, "r\"")?;
        for _ in 0..max_hash_count {
            write!(f, "#")?;
        }
        write!(f, "{}", self.0)?;
        for _ in 0..max_hash_count {
            write!(f, "#")?;
        }
        write!(f, "\"")?;
        Ok(())
    }
}
