use crate::blas_registry::BLASRegistry;
use crate::skinned_blas_entry::SkinnedBlasEntry;
use crate::skinned_blas_plan::SkinnedBlasPlan;
use gpu::ManagedAccelerationStructure;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use anyhow::bail;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR,
    AccelerationStructureBuildSizesInfoKHR, AccelerationStructureKHR, AccelerationStructureTypeKHR,
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
use resource_store::SkinGeometry;
use std::sync::Arc;

pub struct BLAS {
    registry: BLASRegistry,
    skins: Mutex<HashMap<ResourceId, SkinnedBlasEntry>>,

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
            skins: Mutex::new(HashMap::new()),

            blas_queue,

            deferred_destroy,

            resource_factories,
        }
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

    pub fn consume_events(&self) -> Vec<ResourceId> {
        let mut pending = Vec::new();
        let mut pending_ids = HashSet::new();

        for event in self.blas_queue.drain() {
            match event {
                BlasEvent::Loaded { mesh_id, geometry_ranges } => {
                    self.record_geometry(mesh_id, geometry_ranges);

                    if pending_ids.insert(mesh_id) {
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
                BlasEvent::SkinLoaded { skin_id, geometry } => {
                    let displaced = self.skins.lock().insert(skin_id, SkinnedBlasEntry::create(geometry));

                    if let Some(acceleration_structure) = displaced.and_then(|entry| entry.acceleration_structure) {
                        self.retire(acceleration_structure);
                    }
                }
                BlasEvent::SkinUnloaded { skin_id } => {
                    let removed = self.skins.lock().remove(&skin_id);

                    if let Some(acceleration_structure) = removed.and_then(|entry| entry.acceleration_structure) {
                        self.retire(acceleration_structure);
                    }
                }
            }
        }

        pending
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

    pub fn skin_geometry(&self, skin_id: ResourceId) -> Option<SkinGeometry> {
        self.skins
            .lock()
            .get(&skin_id)
            .map(|entry| entry.geometry.clone())
    }

    pub fn plan_skin(
        &self,
        skin_id: ResourceId,
        build_sizes: impl FnOnce() -> Result<AccelerationStructureBuildSizesInfoKHR<'static>>,
    ) -> Result<SkinnedBlasPlan> {
        let mut skins = self.skins.lock();

        let Some(entry) = skins.get_mut(&skin_id) else {
            bail!("Skin {} has no BLAS entry", skin_id.inner);
        };

        if let Some(acceleration_structure) = &entry.acceleration_structure {
            let rebuild = entry.updates_since_rebuild >= SkinnedBlasEntry::REBUILD_INTERVAL;

            entry.updates_since_rebuild = if rebuild { 0 } else { entry.updates_since_rebuild + 1 };

            return Ok(SkinnedBlasPlan {
                handle: acceleration_structure.handle,
                device_address: acceleration_structure.device_address,
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

        let sizes = build_sizes()?;

        let acceleration_structure = self.allocate(
            &format!("blas_skin_{}", skin_id.inner),
            sizes.acceleration_structure_size,
        )?;

        let plan = SkinnedBlasPlan {
            handle: acceleration_structure.handle,
            device_address: acceleration_structure.device_address,
            mode: BuildAccelerationStructureModeKHR::BUILD,
            scratch_size: sizes.build_scratch_size,
        };

        entry.acceleration_structure = Some(acceleration_structure);
        entry.build_scratch_size = sizes.build_scratch_size;
        entry.update_scratch_size = sizes.update_scratch_size;

        Ok(plan)
    }

    fn retire(&self, acceleration_structure: ManagedAccelerationStructure) {
        let resource_factories = self.resource_factories.clone();

        self.deferred_destroy.push(move || destroy_acceleration_structure(&resource_factories, acceleration_structure));
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        for acceleration_structure in self.registry.drain() {
            destroy_acceleration_structure(resource_factories, acceleration_structure)?;
        }

        for (_, entry) in self.skins.into_inner() {
            if let Some(acceleration_structure) = entry.acceleration_structure {
                destroy_acceleration_structure(resource_factories, acceleration_structure)?;
            }
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
