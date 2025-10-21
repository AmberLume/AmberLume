use anyhow::Result;
use shader_compiler::ShaderCompiler;
use std::env::var;
use std::path::PathBuf;

fn main() -> Result<()> {
    let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
    let assets_dir = manifest_dir.join("assets");
    let gen_dir = PathBuf::from(var("GEN_OUT_DIR")?);
    let dist_dir = PathBuf::from(var("DIST_OUT_DIR")?);
    let profile = var("PROFILE")?;

    // Compile shaders
    let shaders_src_dir = assets_dir.join("shaders");
    println!("cargo:rerun-if-changed={}", shaders_src_dir.display());
    let shaders_dst_dir = gen_dir.join("shaders");

    let shader_compiler = ShaderCompiler::new()?;

    shader_compiler.compile_all(&shaders_src_dir, &shaders_dst_dir)?;

    Ok(())
}
