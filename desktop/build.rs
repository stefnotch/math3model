use std::path::PathBuf;

use wesl::{
    CompileOptions, EscapeMangler, Mangler, ModulePath, StandardResolver, compile_sourcemap,
    emit_rerun_if_changed, syntax::PathOrigin,
};
use wgsl_to_wgpu::{MatrixVectorTypes, TypePath, WriteOptions, create_shader_modules};

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
    println!("cargo::rerun-if-changed=build.rs");

    let resolver = StandardResolver::new("src/shaders");
    let mangler = EscapeMangler::default();
    let compile_options = CompileOptions::default();

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut output_file = PathBuf::from(&out_dir);
    output_file.push("main_shader");
    output_file.set_extension("wgsl");

    log!("Generated shaders {}", &out_dir);

    let mut entry_point: ModulePath = "main.wesl".into();
    entry_point.origin = PathOrigin::Absolute; // we force absolute paths
    let compiled = compile_sourcemap(&entry_point, &resolver, &mangler, &compile_options)
        .inspect_err(|e| {
            eprintln!("failed to build WESL shader. {}\n{e}", entry_point);
            panic!();
        })
        .unwrap();
    emit_rerun_if_changed(&compiled.modules, &resolver);
    let compiled_code = compiled.to_string();
    std::fs::write(&output_file, &compiled_code).unwrap();

    // TODO: This part here is tricky. If I have more than one entry point that share some structs,
    // then I'll generate multiple structs.
    // So I need a "super duper" entry point.
    let modules = create_shader_modules(
        &compiled_code,
        WriteOptions {
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
        wesl_unmangler,
    )
    .unwrap();

    output_file.set_extension("rs");
    std::fs::write(&output_file, &modules).unwrap();
}

fn wesl_unmangler(mangled_name: &str) -> TypePath {
    let Some((module_path, name)) = EscapeMangler.unmangle(mangled_name) else {
        panic!("Failed to unmangle {mangled_name}")
    };

    assert_eq!(
        module_path.origin,
        wesl::syntax::PathOrigin::Absolute,
        "Generated WGSL paths are absolute paths"
    );

    TypePath {
        parents: module_path.components,
        name,
    }
}
