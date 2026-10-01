use crate::blas_entry::BlasEntry;
use crate::blas_registry::BLASRegistry;
use gpu::ManagedAccelerationStructure;
use std::collections::HashSet;
use anyhow::bail;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR,
    AccelerationStructureBuildSizesInfoKHR, AccelerationStructureTypeKHR,
    BuildAccelerationStructureFlagsKHR, BuildAccelerationStructureModeKHR, DeviceAddress,
    DeviceOrHostAddressConstKHR, DeviceSize, Format, GeometryFlagsKHR, GeometryTypeKHR, IndexType,
};
use gpu::ResourceFactories;
use gpu::GpuSize;
use gpu_data::VertexPositionGPU;
use index_allocator::DeferredDestroy;
use index_allocator::ResourceId;
use index_allocator::ResourceLimits;
use resource_store::BlasEvent;
use resource_store::BlasQueue;
use resource_store::GeometryRange;
use std::sync::Arc;

pub struct BLAS {
    registry: BLASRegistry,

    blas_queue: Arc<BlasQueue>,

    deferred_destroy: Arc<DeferredDestroy>,

    resource_factories: Arc<ResourceFactories>,
}

impl BLAS {
    pub const STATIC_FLAGS: BuildAccelerationStructureFlagsKHR = BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE;
    pub const SKINNED_FLAGS: BuildAccelerationStructureFlagsKHR = BuildAccelerationStructureFlagsKHR::from_raw(
        BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD.as_raw()
            | BuildAccelerationStructureFlagsKHR::ALLOW_UPDATE.as_raw(),
    );

    pub(crate) fn new(
        resource_limits: ResourceLimits,
        resource_factories: Arc<ResourceFactories>,
        deferred_destroy: Arc<DeferredDestroy>,
        blas_queue: Arc<BlasQueue>,
    ) -> Self {
        Self {
            registry: BLASRegistry::new(resource_limits.max_meshes),

            blas_queue,

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

    pub fn consume_events(
        &self,
        build_sizes: impl Fn(&[GeometryRange], BuildAccelerationStructureFlagsKHR) -> Result<AccelerationStructureBuildSizesInfoKHR<'static>>,
    ) -> Result<Vec<ResourceId>> {
        let mut pending = Vec::new();
        let mut pending_ids = HashSet::new();

        for event in self.blas_queue.drain() {
            match event {
                BlasEvent::Loaded { mesh_id, geometry_ranges, vertex_slice_stride, vertex_slice_count } => {
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

                    let displaced = self.registry.insert(mesh_id, BlasEntry {
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

                    if vertex_slice_count == 1 && pending_ids.insert(mesh_id) {
                        pending.push(mesh_id);
                    }
                }
                BlasEvent::Changed { mesh_id } => {
                    if pending_ids.insert(mesh_id) {
                        pending.push(mesh_id);
                    }
                }
                BlasEvent::Unloaded { mesh_id } => {
                    self.unregister(mesh_id);

                    if pending_ids.remove(&mesh_id) {
                        pending.retain(|pending_id| *pending_id != mesh_id);
                    }
                }
            }
        }

        Ok(pending)
    }

    pub fn unregister(&self, mesh_id: ResourceId) {
        if let Some(entry) = self.registry.remove(mesh_id) {
            self.retire(entry.acceleration_structure);
        }
    }

    pub fn with_entry<R>(&self, mesh_id: ResourceId, action: impl FnOnce(&mut BlasEntry) -> R) -> Option<R> {
        self.registry.with_entry(mesh_id, action)
    }

    fn retire(&self, acceleration_structure: ManagedAccelerationStructure) {
        let resource_factories = self.resource_factories.clone();

        self.deferred_destroy.push(move || destroy_acceleration_structure(&resource_factories, acceleration_structure));
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for entry in self.registry.drain() {
            destroy_acceleration_structure(resource_factories, entry.acceleration_structure)?;
        }

        Ok(())
    }
}

pub fn triangle_geometry(
    vertex_address: DeviceAddress,
    index_address: DeviceAddress,
    geometry_range: &GeometryRange,
) -> AccelerationStructureGeometryKHR<'static> {
    let triangles = AccelerationStructureGeometryTrianglesDataKHR::default()
        .vertex_format(Format::R32G32B32_SFLOAT)
        .vertex_data(DeviceOrHostAddressConstKHR {
            device_address: vertex_address,
        })
        .vertex_stride(VertexPositionGPU::SIZE)
        .max_vertex(geometry_range.vertex_offset + geometry_range.vertex_count - 1)
        .index_type(IndexType::UINT32)
        .index_data(DeviceOrHostAddressConstKHR {
            device_address: index_address,
        });

    AccelerationStructureGeometryKHR::default()
        .geometry_type(GeometryTypeKHR::TRIANGLES)
        .geometry(AccelerationStructureGeometryDataKHR { triangles })
        .flags(GeometryFlagsKHR::OPAQUE)
}

pub fn blas_build_geometry_info<'a>(
    geometries: &'a [AccelerationStructureGeometryKHR<'a>],
    flags: BuildAccelerationStructureFlagsKHR,
) -> AccelerationStructureBuildGeometryInfoKHR<'a> {
    AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .flags(flags)
        .mode(BuildAccelerationStructureModeKHR::BUILD)
        .geometries(geometries)
}

fn destroy_acceleration_structure(
    resource_factories: &ResourceFactories,
    acceleration_structure: ManagedAccelerationStructure,
) -> Result<()> {
    let Some(factory) = &resource_factories.acceleration_structure_factory else {
        bail!("Acceleration structure factory is missing")
    };

    factory.destroy(&resource_factories.buffer_factory, acceleration_structure)
}
