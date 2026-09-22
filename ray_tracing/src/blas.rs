use crate::blas_registry::BLASRegistry;
use crate::skinned_blas_entry::SkinnedBlasEntry;
use crate::skinned_blas_plan::SkinnedBlasPlan;
use gpu::ManagedAccelerationStructure;
use parking_lot::Mutex;
use render_snapshot::RenderEntityId;
use std::collections::{HashMap, HashSet};
use anyhow::bail;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR,
    AccelerationStructureKHR, AccelerationStructureTypeKHR, BuildAccelerationStructureFlagsKHR,
    BuildAccelerationStructureModeKHR, DeviceAddress, DeviceOrHostAddressConstKHR, DeviceSize,
    Format, GeometryFlagsKHR, GeometryTypeKHR, IndexType,
};
use gpu::ResourceFactories;
use gpu::GpuSize;
use gpu_data::MeshVertexGPU;
use index_allocator::DeferredDestroy;
use index_allocator::ResourceId;
use index_allocator::ResourceLimits;
use resource_store::GeometryRange;
use resource_store::ResourceBuffers;
use std::sync::Arc;

pub struct BLAS {
    pub mesh_vertex_address: DeviceAddress,
    index_address: DeviceAddress,

    registry: BLASRegistry,
    skinned: Mutex<HashMap<RenderEntityId, SkinnedBlasEntry>>,

    deferred_destroy: Arc<DeferredDestroy>,

    resource_factories: Arc<ResourceFactories>,
}

impl BLAS {
    pub(crate) fn new(
        resource_limits: ResourceLimits,
        resource_factories: Arc<ResourceFactories>,
        deferred_destroy: Arc<DeferredDestroy>,
        resource_buffers: &ResourceBuffers,
    ) -> Result<Self> {
        Ok(Self {
            mesh_vertex_address: resource_buffers.mesh_vertex.allocation.device_address,
            index_address: resource_buffers.index.allocation.device_address,

            registry: BLASRegistry::new(resource_limits.max_meshes),
            skinned: Mutex::new(HashMap::new()),

            deferred_destroy,

            resource_factories,
        })
    }

    pub fn triangle_geometry(
        &self,
        vertex_address: DeviceAddress,
        geometry_range: &GeometryRange,
    ) -> AccelerationStructureGeometryKHR<'static> {
        let triangles = AccelerationStructureGeometryTrianglesDataKHR::default()
            .vertex_format(Format::R32G32B32_SFLOAT)
            .vertex_data(DeviceOrHostAddressConstKHR {
                device_address: vertex_address,
            })
            .vertex_stride(MeshVertexGPU::SIZE)
            .max_vertex(geometry_range.vertex_offset + geometry_range.vertex_count - 1)
            .index_type(IndexType::UINT32)
            .index_data(DeviceOrHostAddressConstKHR {
                device_address: self.index_address,
            });

        AccelerationStructureGeometryKHR::default()
            .geometry_type(GeometryTypeKHR::TRIANGLES)
            .geometry(AccelerationStructureGeometryDataKHR { triangles })
            .flags(GeometryFlagsKHR::OPAQUE)
    }

    pub fn allocate(
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

    pub fn record_geometry(&self, mesh_id: ResourceId, geometry_ranges: Vec<GeometryRange>) {
        if let Some(displaced) = self.registry.record_geometry(mesh_id, geometry_ranges) {
            self.retire(displaced);
        }
    }

    pub fn geometry_ranges(&self, mesh_id: ResourceId) -> Option<Vec<GeometryRange>> {
        self.registry.geometry_ranges(mesh_id)
    }

    pub fn register(
        &self,
        mesh_id: ResourceId,
        acceleration_structure: ManagedAccelerationStructure,
    ) -> AccelerationStructureKHR {
        let handle = acceleration_structure.handle;

        if let Some(displaced) = self.registry.set_acceleration_structure(mesh_id, acceleration_structure) {
            self.retire(displaced);
        }

        handle
    }

    pub fn unregister(&self, mesh_id: ResourceId) {
        if let Some(acceleration_structure) = self.registry.remove(mesh_id) {
            self.retire(acceleration_structure);
        }
    }

    pub fn addresses(&self) -> Vec<DeviceAddress> {
        self.registry.addresses()
    }

    pub fn plan_skinned(
        &self,
        entity_id: RenderEntityId,
        primitive_counts: &[u32],
        create: impl FnOnce() -> Result<SkinnedBlasEntry>,
    ) -> Result<SkinnedBlasPlan> {
        let mut skinned = self.skinned.lock();

        if let Some(entry) = skinned.get_mut(&entity_id) {
            if entry.primitive_counts == primitive_counts {
                let rebuild = entry.updates_since_rebuild >= SkinnedBlasEntry::REBUILD_INTERVAL;

                entry.updates_since_rebuild = if rebuild { 0 } else { entry.updates_since_rebuild + 1 };

                return Ok(SkinnedBlasPlan {
                    handle: entry.acceleration_structure.handle,
                    device_address: entry.acceleration_structure.device_address,
                    mode: if rebuild {
                        BuildAccelerationStructureModeKHR::BUILD
                    } else {
                        BuildAccelerationStructureModeKHR::UPDATE
                    },
                    scratch_size: if rebuild {
                        entry.build_scratch_size
                    } else {
                        entry.update_scratch_size
                    },
                });
            }
        }

        let entry = create()?;

        let plan = SkinnedBlasPlan {
            handle: entry.acceleration_structure.handle,
            device_address: entry.acceleration_structure.device_address,
            mode: BuildAccelerationStructureModeKHR::BUILD,
            scratch_size: entry.build_scratch_size,
        };

        if let Some(displaced) = skinned.insert(entity_id, entry) {
            self.retire(displaced.acceleration_structure);
        }

        Ok(plan)
    }

    pub fn retain_skinned(&self, entity_ids: &HashSet<RenderEntityId>) {
        let mut skinned = self.skinned.lock();

        let stale = skinned.keys()
            .filter(|entity_id| !entity_ids.contains(entity_id))
            .copied()
            .collect::<Vec<_>>();

        for entity_id in stale {
            if let Some(entry) = skinned.remove(&entity_id) {
                self.retire(entry.acceleration_structure);
            }
        }
    }

    fn retire(&self, acceleration_structure: ManagedAccelerationStructure) {
        let resource_factories = self.resource_factories.clone();

        self.deferred_destroy.push(move || destroy_acceleration_structure(&resource_factories, acceleration_structure));
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for acceleration_structure in self.registry.drain() {
            destroy_acceleration_structure(resource_factories, acceleration_structure)?;
        }

        for (_, entry) in self.skinned.into_inner() {
            destroy_acceleration_structure(resource_factories, entry.acceleration_structure)?;
        }

        Ok(())
    }
}

pub fn blas_build_geometry_info<'a>(
    geometries: &'a [AccelerationStructureGeometryKHR<'a>],
) -> AccelerationStructureBuildGeometryInfoKHR<'a> {
    AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .flags(BuildAccelerationStructureFlagsKHR::PREFER_FAST_TRACE)
        .mode(BuildAccelerationStructureModeKHR::BUILD)
        .geometries(geometries)
}

pub fn skinned_blas_build_geometry_info<'a>(
    geometries: &'a [AccelerationStructureGeometryKHR<'a>],
) -> AccelerationStructureBuildGeometryInfoKHR<'a> {
    AccelerationStructureBuildGeometryInfoKHR::default()
        .ty(AccelerationStructureTypeKHR::BOTTOM_LEVEL)
        .flags(
            BuildAccelerationStructureFlagsKHR::PREFER_FAST_BUILD
                | BuildAccelerationStructureFlagsKHR::ALLOW_UPDATE,
        )
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
