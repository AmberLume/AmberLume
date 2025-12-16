use anyhow::Result;

pub trait ResourceCompiler {
    type Input;

    fn compile(&self, input: Self::Input) -> Result<Vec<u8>>;
}
