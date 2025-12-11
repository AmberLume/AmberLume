use crate::data::paths::ResourcePaths;
use crate::utils::for_each_file;
use alpaca::compiler::resource_compiler::ResourceCompiler;
use alpaca::compiler::shaders::shader_compiler::ShaderCompiler;
use anyhow::Result;

pub fn compile_shaders(resource_paths: &ResourcePaths) -> Result<()> {
    println!("Compiling shaders: {}", &resource_paths.source.display());

    let shader_compiler = ShaderCompiler::new()?;

    for_each_file(&resource_paths.source, |path| {
        let name = path.file_name().unwrap().to_string_lossy().into_owned();

        println!("Compiling {}...", name);

        let source = resource_paths.source.join(path);

        shader_compiler.compile(&name.to_owned(), &source, &resource_paths.target)?;

        Ok(())
    })?;

    Ok(())
}
