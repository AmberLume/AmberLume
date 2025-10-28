use alpaca::packer::alpaca_writer::AlpacaWriter;
use anyhow::Result;
use std::env::var;
use std::fs::read;
use std::path::PathBuf;
use walkdir::WalkDir;

fn main() -> Result<()> {
    // let manifest_dir = PathBuf::from(var("CARGO_MANIFEST_DIR")?);
    // let assets_dir = manifest_dir.join("assets");
    // let gen_dir = PathBuf::from(var("GEN_OUT_DIR")?);
    // let dist_dir = PathBuf::from(var("DIST_OUT_DIR")?);
    // let profile = var("PROFILE")?;
    //
    // // Compile shaders
    // let shaders_src_dir = assets_dir.join("shaders");
    // println!("cargo:rerun-if-changed={}", shaders_src_dir.display());
    // let shaders_dst_dir = gen_dir.join("shaders");
    //
    // let shader_compiler = ShaderCompiler::new()?;
    //
    // shader_compiler.compile_all(&shaders_src_dir, &shaders_dst_dir)?;
    //
    // // Alpaca
    // let alpaca_src_dir = gen_dir;
    // let alpaca_dst_dir = dist_dir.join("alpaca");
    //
    // let mut alpaca_writer =
    //     AlpacaWriter::create(String::from("shaders"), alpaca_dst_dir, 64 * 1024)?;
    //
    // for entry in WalkDir::new(alpaca_src_dir)
    //     .into_iter()
    //     .filter_map(Result::ok)
    // {
    //     if entry.file_type().is_file() {
    //         let path = entry.path().to_path_buf();
    //
    //         let name = path.file_name().unwrap().to_string_lossy().into_owned();
    //         let data = read(path)?;
    //
    //         alpaca_writer.push(name, &data)?;
    //     }
    // }
    //
    // alpaca_writer.pack()?;

    Ok(())
}
