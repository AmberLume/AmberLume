use crate::compiler::resource_compiler::ResourceCompiler;
use crate::compiler::shaders::model_compiler::ModelCompiler;
use crate::compiler::shaders::shader_compiler::ShaderCompiler;
use crate::compiler::utils::extension_of;
use anyhow::Result;
use std::path::Path;

pub struct Pipeline {
    compilers: Vec<Box<dyn ResourceCompiler>>,

    compiled_resources: Vec<String>,
}

pub struct CompilationResult {
    pub name: String,
}

impl Pipeline {
    pub fn new() -> Result<Self> {
        let compilers: Vec<Box<dyn ResourceCompiler>> = vec![
            Box::from(ShaderCompiler::new()?),
            Box::from(ModelCompiler::new()?),
        ];

        Ok(Self {
            compilers,

            compiled_resources: Vec::new(),
        })
    }

    pub fn compile(
        &mut self,
        name: &String,
        src: &Path,
        dst_dir: &Path,
    ) -> Result<Vec<CompilationResult>> {
        println!("Compiling {}...", src.display());
        let extension = extension_of(src);

        for compiler in &mut self.compilers {
            if compiler.extensions().contains(&extension.as_str()) {
                let compilation_results = compiler.compile(name, src, dst_dir)?;

                for compilation_result in &compilation_results {
                    self.register(&compilation_result.name);
                }

                return Ok(compilation_results);
            }
        }

        println!("Unsupported compilation: {}", src.display());
        Ok(Vec::new())
    }

    fn register(&mut self, name: &String) {
        if !self.compiled_resources.contains(&name) {
            self.compiled_resources.push(name.clone());
        }
    }
}
