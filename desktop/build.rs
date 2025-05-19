use std::path::PathBuf;

use wesl::{
    CompileOptions, EscapeMangler, ModulePath, StandardResolver, compile_sourcemap,
    emit_rerun_if_changed, syntax::PathOrigin,
};

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
    compiled
        .write_to_file(output_file)
        .expect("failed to write output shader");
}
