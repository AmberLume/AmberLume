use crate::assembler::resource_compiler::ResourceCompiler;
use crate::data::common::model_data::ModelData;
use anyhow::Result;
use rkyv::rancor::Error;
use rkyv::to_bytes;

pub struct ModelCompiler {}

impl ModelCompiler {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

pub struct ModelResource {
    pub data: ModelData,
}

impl ResourceCompiler for ModelCompiler {
    type Input = ModelResource;

    fn compile(&self, input: Self::Input) -> Result<Vec<u8>> {
        let slice = to_bytes::<Error>(&input.data)?.into_vec();

        Ok(slice)
    }
}
