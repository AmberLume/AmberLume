use crate::store::mesh_table::geometry_changes::GeometryChanges;
use crate::store::mesh_table::geometry_range::GeometryRange;
use crate::store::mesh_table::loaded_geometry::LoadedGeometry;
use anyhow::Result;
use gpu::RangeAllocation;
use gpu::ResourceTransfer;
use gpu::SingleAllocation;
use gpu_data::MeshGPU;
use gpu_data::SubmeshGPU;
use index_allocator::Allocation;
use index_allocator::ResourceId;
use parking_lot::Mutex;
use std::mem::take;
use std::sync::Arc;
use tracing::info;

pub struct MeshTable {
    pub mesh: Arc<SingleAllocation<MeshGPU>>,
    pub submesh: Arc<RangeAllocation<SubmeshGPU>>,

    resource_transfer: Arc<ResourceTransfer>,

    geometry_changes: Mutex<GeometryChanges>,
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

            geometry_changes: Mutex::new(GeometryChanges::default()),
        }
    }

    pub fn write(
        &self,
        mesh_id: ResourceId,
        submeshes_allocation: Allocation,
        submeshes: &[SubmeshGPU],
        bone_offset: u32,
        geometry_ranges: Vec<GeometryRange>,
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

        self.geometry_changes.lock().loaded.push(LoadedGeometry {
            mesh_id,

            ranges: geometry_ranges,
        });

        Ok(())
    }

    pub fn erase(&self, mesh_id: ResourceId) -> Result<()> {
        self.geometry_changes.lock().unloaded.push(mesh_id);

        self.resource_transfer.load_buffer_at(
            self.mesh.at(mesh_id.inner),
            &[MeshGPU::create(0, 0, 0)],
        )?;

        Ok(())
    }

    pub fn record_changed(&self, mesh_id: ResourceId) {
        self.geometry_changes.lock().changed.push(mesh_id);
    }

    pub fn take_geometry_changes(&self) -> GeometryChanges {
        take(&mut *self.geometry_changes.lock())
    }
}
