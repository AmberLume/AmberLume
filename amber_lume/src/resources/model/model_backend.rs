use crate::render::vulkan::buffer::index_buffer::IndexBuffer;
use crate::render::vulkan::buffer::resource_context::ResourceContext;
use crate::render::vulkan::buffer::transfer_context::TransferContext;
use crate::render::vulkan::buffer::vertex_buffer::VertexBuffer;
use crate::render::vulkan::data::vertex::Vertex;
use crate::render::vulkan::device_context::DeviceContext;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::index::resource_index::ResourceIndex;
use crate::resources::model::model_config::ModelConfig;
use alpaca::data::common::model_data::{ArchivedModelData, ModelData};
use alpaca::data::common::primitive_data::PrimitiveData;
use anyhow::Result;
use ash::Device;
use ash::vk::DeviceSize;
use glam::{Vec2, Vec3};
use rkyv::rancor::Error;
use rkyv::{access, deserialize};
use std::sync::{Arc, Mutex};
use tracing::info;

pub struct ModelBackend {
    device: Device,

    transfer_context: Arc<Mutex<TransferContext>>,

    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    resource_index: Arc<ResourceIndex>,
}

impl ModelBackend {
    pub fn new(
        device_context: &DeviceContext,
        resource_index: Arc<ResourceIndex>,
        resource_context: &mut ResourceContext,
    ) -> Self {
        let vertex_buffer =
            VertexBuffer::create(&device_context, &mut resource_context.allocator, 100_000)
                .unwrap();
        let index_buffer =
            IndexBuffer::create(&device_context, &mut resource_context.allocator, 100_000).unwrap();

        Self {
            device: device_context.device.clone(),

            transfer_context: resource_context.transfer_context.clone(),

            vertex_buffer,
            index_buffer,

            resource_index,
        }
    }

    fn upload_primitive(
        &self,
        transfer_context: &mut TransferContext,
        primitive_data: &PrimitiveData,
    ) -> Result<PrimitiveAllocation> {
        let mut vertices = Vec::with_capacity(primitive_data.vertices.len());
        for vertex in &primitive_data.vertices {
            let vertex = Vertex {
                position: Vec3::new(vertex[0], vertex[1], vertex[2]),
                normal: Vec3::Y,
                uv: Vec2::ZERO,
            };

            vertices.push(vertex);
        }

        let indices_offset = self
            .index_buffer
            .allocate_space(primitive_data.indices.len())?;
        let vertices_offset = self.vertex_buffer.allocate_space(vertices.len())?;

        transfer_context.copy_to_buffer_at(
            &self.index_buffer.buffer,
            indices_offset,
            &primitive_data.indices,
        )?;
        transfer_context.copy_to_buffer_at(
            &self.vertex_buffer.buffer,
            vertices_offset,
            &vertices,
        )?;

        let primitive_allocation = PrimitiveAllocation {
            index_offset: indices_offset,
            index_size: primitive_data.indices.len() as u32,
            vertex_offset: vertices_offset,
            vertex_size: vertices.len() as u32,
        };

        Ok(primitive_allocation)
    }
}

pub struct ModelDependencies {
    pub model_data: ModelData,
}

#[derive(Debug)]
pub struct PrimitiveAllocation {
    pub index_offset: DeviceSize,
    pub index_size: u32,
    pub vertex_offset: DeviceSize,
    pub vertex_size: u32,
}

impl ResourceBackend for ModelBackend {
    type Config = ModelConfig;
    type Dependencies = ModelDependencies;
    type Output = ();

    fn key_from(config: &Self::Config) -> ResourceKey {
        config.hash()
    }

    fn collect_dependencies(&self, config: &Self::Config) -> Self::Dependencies {
        let mesh_bytes = self.resource_index.get_resource(&config.name).unwrap();

        let archived = access::<ArchivedModelData, Error>(&mesh_bytes).unwrap();

        let model_data = deserialize::<ModelData, Error>(archived).unwrap();

        Self::Dependencies { model_data }
    }

    fn create(
        &self,
        config: Self::Config,
        dependencies: Self::Dependencies,
    ) -> Result<Self::Output> {
        let mut transfer_context = self.transfer_context.lock().unwrap();

        transfer_context.begin()?;

        let mut primitive_allocations = Vec::new();

        for mesh_data in dependencies.model_data.meshes {
            for primitive_data in mesh_data.primitives {
                let primitive_allocation =
                    self.upload_primitive(&mut transfer_context, &primitive_data)?;

                primitive_allocations.push(primitive_allocation);
            }
        }

        transfer_context.submit()?;

        println!("Primitive allocations: {:?}", primitive_allocations);

        Ok(())
    }

    fn destroy_resource(&self, resource: Self::Output) -> Result<()> {
        Ok(())
    }

    fn destroy(&mut self) -> Result<()> {
        self.index_buffer.destroy()?;
        self.vertex_buffer.destroy()?;

        info!("BufferManager destroyed");

        Ok(())
    }
}
