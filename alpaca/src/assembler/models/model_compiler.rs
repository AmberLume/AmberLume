use crate::assembler::resource_compiler::ResourceCompiler;
use crate::data::flatbuffers::model_generated::alpaca::{
    AABB, MeshPrimitive, MeshPrimitiveArgs, Model, ModelArgs, ModelMesh, ModelMeshArgs,
};
use anyhow::Result;
use flatbuffers::FlatBufferBuilder;

pub struct ModelCompiler {}

impl ModelCompiler {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }

    pub fn to_aabb(&self, bounds: &[f32; 6]) -> AABB {
        AABB::new(
            bounds[0], bounds[1], bounds[2], bounds[3], bounds[4], bounds[5],
        )
    }
}

pub struct ModelResource {
    pub meshes: Vec<Mesh>,
    pub bounds: [f32; 6],
}

pub struct Mesh {
    pub name: String,
    pub primitives: Vec<String>,
    pub bounds: [f32; 6],
}

impl ResourceCompiler for ModelCompiler {
    type Input = ModelResource;

    fn compile(&self, input: Self::Input) -> Result<Vec<u8>> {
        let mut builder = FlatBufferBuilder::new();

        let mut mesh_offsets = Vec::new();

        for mesh in input.meshes {
            let name_offset = builder.create_string(&mesh.name);

            let mut primitive_offsets = Vec::new();

            for primitive in mesh.primitives {
                println!("primitive_name: {}", primitive);

                let lod0_offset = builder.create_string(&primitive);

                let primitive = MeshPrimitive::create(
                    &mut builder,
                    &MeshPrimitiveArgs {
                        lod0: Some(lod0_offset),
                    },
                );

                primitive_offsets.push(primitive);
            }

            let primitives_vec = builder.create_vector(&primitive_offsets);

            let mesh_bounds = self.to_aabb(&mesh.bounds);

            let model_mesh = ModelMesh::create(
                &mut builder,
                &ModelMeshArgs {
                    name: Some(name_offset),
                    primitives: Some(primitives_vec),
                    bounds: Some(&mesh_bounds),
                },
            );

            mesh_offsets.push(model_mesh);
        }

        let meshes_vec = builder.create_vector(&mesh_offsets);

        let model_bounds = self.to_aabb(&input.bounds);

        let model = Model::create(
            &mut builder,
            &ModelArgs {
                meshes: Some(meshes_vec),
                bounds: Some(&model_bounds),
            },
        );

        builder.finish(model, None);

        Ok(builder.finished_data().to_vec())
    }
}
