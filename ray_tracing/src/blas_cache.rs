use crate::blas::destroy_acceleration_structure;
use crate::blas_entry::BlasEntry;
use anyhow::bail;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildSizesInfoKHR, AccelerationStructureTypeKHR,
    BuildAccelerationStructureFlagsKHR, DeviceSize,
};
use gpu::ManagedAccelerationStructure;
use gpu::ResourceFactories;
use index_allocator::DeferredDestroy;
use index_allocator::ResourceId;
use index_allocator::ResourceLimits;
use parking_lot::Mutex;
use resource_store::GeometryRange;
use std::sync::Arc;

pub struct BlasCache {
    entries: Mutex<Vec<Option<BlasEntry>>>,

    deferred_destroy: Arc<DeferredDestroy>,

    resource_factories: Arc<ResourceFactories>,
}

impl BlasCache {
    pub const STATIC_FLAGS: BuildAccelerationStructureFlagsKHR = BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE;
    pub const SKINNED_FLAGS: BuildAccelerationStructureFlagsKHR = BuildAccelerationStructureFlagsKHR::from_raw(
        BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD.as_raw()
            | BuildAccelerationStructureFlagsKHR::ALLOW_UPDATE.as_raw(),
    );

    pub fn new(
        resource_limits: ResourceLimits,
        resource_factories: Arc<ResourceFactories>,
        deferred_destroy: Arc<DeferredDestroy>,
    ) -> Self {
        Self {
            entries: Mutex::new((0..resource_limits.max_meshes).map(|_| None).collect()),

            deferred_destroy,

            resource_factories,
        }
    }

    fn allocate(
        &self,
        name: &str,
        size: DeviceSize,
    ) -> Result<ManagedAccelerationStructure> {
        let Some(factory) = &self.resource_factories.acceleration_structure_factory else {
            bail!("Acceleration structure factory is missing")
        };

        factory.allocate(
            &self.resource_factories.buffer_factory,
            name,
            size,
            AccelerationStructureTypeKHR::BOTTOM_LEVEL,
        )
    }

    pub fn contains(&self, mesh_id: ResourceId) -> bool {
        self.entries.lock()[mesh_id.inner as usize].is_some()
    }

    pub fn create(
        &self,
        mesh_id: ResourceId,
        geometry_ranges: Vec<GeometryRange>,
        vertex_slice_stride: u32,
        vertex_slice_count: u32,
        build_sizes: impl Fn(&[GeometryRange], BuildAccelerationStructureFlagsKHR) -> Result<AccelerationStructureBuildSizesInfoKHR<'static>>,
    ) -> Result<()> {
        let flags = if vertex_slice_count == 1 {
            Self::STATIC_FLAGS
        } else {
            Self::SKINNED_FLAGS
        };

        let sizes = build_sizes(&geometry_ranges, flags)?;

        let acceleration_structure = self.allocate(
            &format!("blas_mesh_{}", mesh_id.inner),
            sizes.acceleration_structure_size,
        )?;

        let displaced = self.entries.lock()[mesh_id.inner as usize].replace(BlasEntry {
            geometry_ranges,
            vertex_slice_stride,

            acceleration_structure,

            build_scratch_size: sizes.build_scratch_size,
            update_scratch_size: sizes.update_scratch_size,

            refits_until_build: 0,
        });

        if let Some(displaced) = displaced {
            self.retire(displaced.acceleration_structure);
        }

        Ok(())
    }

    pub fn with_entry<R>(&self, mesh_id: ResourceId, action: impl FnOnce(&mut BlasEntry) -> R) -> Option<R> {
        self.entries.lock()[mesh_id.inner as usize].as_mut().map(action)
    }

    pub fn evict(&self, mesh_ids: &[ResourceId]) {
        let mut entries = self.entries.lock();

        for mesh_id in mesh_ids {
            if let Some(entry) = entries[mesh_id.inner as usize].take() {
                self.retire(entry.acceleration_structure);
            }
        }
    }

    fn retire(&self, acceleration_structure: ManagedAccelerationStructure) {
        let resource_factories = self.resource_factories.clone();

        self.deferred_destroy.push(move || destroy_acceleration_structure(&resource_factories, acceleration_structure));
    }

    pub fn retire_all(self) {
        for entry in self.entries.lock().drain(..).flatten() {
            self.retire(entry.acceleration_structure);
        }
    }
}
