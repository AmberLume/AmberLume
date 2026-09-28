use anyhow::Result;
use gpu::RangeAllocation;
use gpu::ResourceTransfer;
use gpu::SingleAllocation;
use gpu_data::MeshGPU;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;
use index_allocator::ResourceId;
use std::sync::Arc;
use tracing::info;

pub struct MeshTable {
    pub mesh: Arc<SingleAllocation<MeshGPU>>,
    pub submesh: Arc<RangeAllocation<SubmeshGPU>>,

    resource_transfer: Arc<ResourceTransfer>,
}

impl MeshTable {
    pub fn create(
        mesh: Arc<SingleAllocation<MeshGPU>>,
        submesh: Arc<RangeAllocation<SubmeshGPU>>,
        resource_transfer: Arc<ResourceTransfer>,
    ) -> Self {
        Self {
            mesh,
            submesh,

            resource_transfer,
        }
    }

    pub fn write(
        &self,
        mesh_id: ResourceId,
        submeshes_allocation: Allocation,
        submeshes: &[SubmeshGPU],
        bone_offset: u32,
    ) -> Result<()> {
        let mesh_gpu = MeshGPU::create(
            submeshes_allocation.offset,
            submeshes_allocation.size,
            bone_offset,
        );

        self.resource_transfer.load_buffer_at(
            self.submesh.slice(submeshes_allocation.offset, submeshes.len() as u32),
            submeshes,
        )?;

        self.resource_transfer.load_buffer_at(
            self.mesh.at(mesh_id.inner),
            &[mesh_gpu],
        )?;
        info!("Uploaded mesh: index: {}, data: {:?}", mesh_id.inner, mesh_gpu);

        Ok(())
    }

    pub fn erase(&self, mesh_id: ResourceId) -> Result<()> {
        self.resource_transfer.load_buffer_at(
            self.mesh.at(mesh_id.inner),
            &[MeshGPU::create(0, 0, 0)],
        )?;

        Ok(())
    }
}
