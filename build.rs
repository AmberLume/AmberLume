use anyhow::Result;
use glob::glob;
use shaderc::{CompileOptions, Compiler, OptimizationLevel, ShaderKind, TargetEnv};
use std::env::var;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;

fn main() {
    build_shaders(String::from("assets/shader")).unwrap();
}

fn build_shaders(shader_root: String) -> Result<()> {
    let manifest_dir = PathBuf::from(var(String::from("CARGO_MANIFEST_DIR"))?);
    let target_dir = PathBuf::from(var(String::from("CARGO_TARGET_DIR")).unwrap_or_else(|_| {
        manifest_dir
            .join(String::from("target"))
            .to_string_lossy()
            .into()
    }));
    let profile = var(String::from("PROFILE"))?;

    let src_root = manifest_dir.join(shader_root.clone());
    let out_root = target_dir.join(profile);

    println!("cargo:rerun-if-changed={}", shader_root);

    let compiler = Compiler::new().expect("Could not create shader compiler");
    let mut compile_options =
        CompileOptions::new().expect("Could not create shader compile options");
    compile_options.set_optimization_level(OptimizationLevel::Performance);
    compile_options.set_target_env(TargetEnv::Vulkan, 0);

    let pattern = &format!("{}/**/*", &src_root.to_string_lossy().replace("\\", "/"));
    for entry in glob(pattern)? {
        let src_path = entry?;
        println!("cargo:warning=compiling_shader={}", src_path.display());
        let extension = src_path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let kind = match extension {
            "vert" => ShaderKind::Vertex,
            "frag" => ShaderKind::Fragment,
            _ => continue,
        };

        let source = read_to_string(src_path.clone())?;
        let virtual_name = src_path
            .strip_prefix(shader_root.clone())
            .unwrap_or(&src_path)
            .to_string_lossy()
            .to_string();

        let artifact = compiler
            .compile_into_spirv(&source, kind, &virtual_name, "main", Some(&compile_options))
            .unwrap_or_else(|err| panic!("Could not compile shader: {}", err));

        let relative = src_path.strip_prefix(manifest_dir.clone())?;
        let mut out_path = out_root.join(relative.to_path_buf());
        out_path.set_extension(format!("{}.spv", extension));

        if let Some(parent) = out_path.parent() {
            create_dir_all(parent.to_path_buf())?
        };

        write(out_path.clone(), artifact.as_binary_u8())?;
    }

    Ok(())
}
