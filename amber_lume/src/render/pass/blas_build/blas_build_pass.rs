use render_graph::ReadbackScope;
use anyhow::Result;
use ash::vk::{
    AccelerationStructureBuildRangeInfoKHR, AccelerationStructureBuildSizesInfoKHR,
    AccelerationStructureBuildTypeKHR, AccelerationStructureKHR, AccelerationStructureTypeKHR,
    AccessFlags, DeviceOrHostAddressKHR, DeviceSize, PipelineStageFlags,
};
use gpu::ResourceFactories;
use gpu_data::SubmeshGPU;
use index_allocator::ResourceId;
use render_snapshot::RenderSnapshot;
use resource_residency::ResourceProvider;
use resource_store::MeshBackend;
use std::collections::HashSet;
use ray_tracing::blas_build_geometry_info;
use ray_tracing::BLASRequest;
use ray_tracing::{align_up, RayTracing};
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::FrameContext;
use render_graph::HeapAllocator;
use render_graph::ImageResourceScope;
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualAccelerationStructure;
use render_graph::VirtualBuffer;
use render_graph::VirtualData;
use std::mem::size_of;
use std::sync::Arc;
use tracing::warn;

struct BLASBuild {
    blas_request: BLASRequest,
    handle: AccelerationStructureKHR,
    scratch_offset: DeviceSize,
}

pub struct BLASBuildPassData {
    ray_tracing: Arc<RayTracing>,
    blas_builds: Vec<BLASBuild>,
}

pub struct BLASBuildPass {
    ray_tracing: VirtualData<Arc<RayTracing>>,
    render_snapshot: VirtualData<RenderSnapshot>,
    touched_meshes: VirtualData<Vec<ResourceId>>,

    blas: VirtualAccelerationStructure,
    addresses: VirtualBuffer,
    mesh_vertex_buffer: VirtualBuffer,
    index_buffer: VirtualBuffer,

    mesh_provider: Arc<ResourceProvider<MeshBackend>>,
}

impl BLASBuildPass {
    pub fn create(
        ray_tracing: VirtualData<Arc<RayTracing>>,
        render_snapshot: VirtualData<RenderSnapshot>,
        touched_meshes: VirtualData<Vec<ResourceId>>,
        blas: VirtualAccelerationStructure,
        addresses: VirtualBuffer,
        mesh_vertex_buffer: VirtualBuffer,
        index_buffer: VirtualBuffer,
        mesh_provider: Arc<ResourceProvider<MeshBackend>>,
    ) -> Self {
        Self {
            ray_tracing,
            render_snapshot,
            touched_meshes,

            blas,
            addresses,
            mesh_vertex_buffer,
            index_buffer,

            mesh_provider,
        }
    }

    fn mesh_request(&self, mesh_id: ResourceId) -> Option<BLASRequest> {
        self.mesh_provider
            .with_resource(mesh_id, |mesh| {
                if mesh.submeshes_allocation.size != 1 {
                    return None;
                }

                Some(BLASRequest {
                    mesh_id,
                    submeshes: vec![SubmeshGPU::create(
                        mesh.indices_allocation.size,
                        mesh.indices_allocation.offset,
                        mesh.vertices_allocation.offset,
                        mesh.vertex_attributes_allocation.offset,
                        mesh.vertex_skins_allocation.map_or(0, |allocation| allocation.offset),
                        mesh.materials.first().map_or(0, |material| material.id.inner),
                        [0.0; 6],
                    )],
                })
            })
            .flatten()
    }
}

impl Pass for BLASBuildPass {
    type PassData = BLASBuildPassData;

    fn name(&self) -> String {
        String::from("blas_build")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        true
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration
            .consume(self.ray_tracing)
            .consume(self.render_snapshot)
            .consume(self.touched_meshes)
            .write_acceleration_structure(
                self.blas,
                AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_buffer(
                self.mesh_vertex_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .read_buffer(
                self.index_buffer,
                AccessFlags::SHADER_READ,
                PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
            )
            .write_buffer(
                self.addresses,
                AccessFlags::HOST_WRITE,
                PipelineStageFlags::HOST,
            );
    }

    fn prepare_data(
        &self,
        data_scope: &mut DataResourceScope,
        buffer_scope: &mut BufferResourceScope,
        allocator: &mut HeapAllocator,
        _frame_context: &FrameContext,
    ) -> Result<Self::PassData> {
        let ray_tracing = data_scope.get(self.ray_tracing).clone();
        let render_snapshot = data_scope.get(self.render_snapshot);
        let touched_meshes = data_scope.get(self.touched_meshes).clone();

        let alignment = ray_tracing.context.properties.min_scratch_offset_alignment as DeviceSize;
        let capacity = ray_tracing.blas.scratch_capacity;

        for mesh_id in ray_tracing.blas.request_queue.drain_unloaded() {
            if let Some(acceleration_structure) = ray_tracing.blas.registry.remove(mesh_id) {
                ray_tracing.blas.destroy_queue.push(acceleration_structure);
            }
        }

        let mut blas_builds = Vec::new();
        let mut scratch_cursor: DeviceSize = 0;
        let mut pending = ray_tracing.blas.request_queue.drain();

        let mut queued = pending
            .iter()
            .map(|blas_request| blas_request.mesh_id)
            .collect::<HashSet<_>>();

        for mesh_id in touched_meshes.iter() {
            if !queued.insert(*mesh_id) {
                continue;
            }

            pending.extend(self.mesh_request(*mesh_id));
        }

        for entity in render_snapshot.entities.iter() {
            let mesh_id = ResourceId::from(entity.mesh_id);

            if ray_tracing.blas.registry.contains(mesh_id) {
                continue;
            }

            if !queued.insert(mesh_id) {
                continue;
            }

            pending.extend(self.mesh_request(mesh_id));
        }

        let mut requests = pending.into_iter();

        while let Some(blas_request) = requests.next() {
            if blas_request.submeshes.is_empty() {
                continue;
            }

            let geometries = vec![ray_tracing.blas.geometry; blas_request.submeshes.len()];
            let primitive_counts = blas_request
                .submeshes
                .iter()
                .map(|submesh| submesh.index_count / 3)
                .collect::<Vec<_>>();

            let size_geometry_info = blas_build_geometry_info(&geometries);

            let mut sizes = AccelerationStructureBuildSizesInfoKHR::default();
            unsafe {
                ray_tracing
                    .context
                    .device
                    .get_acceleration_structure_build_sizes(
                        AccelerationStructureBuildTypeKHR::DEVICE,
                        &size_geometry_info,
                        &primitive_counts,
                        &mut sizes,
                    );
            }

            let scratch_offset = align_up(scratch_cursor, alignment);
            let next_scratch_cursor = scratch_offset + sizes.build_scratch_size;

            if next_scratch_cursor > capacity {
                let mut deferred = vec![blas_request];
                deferred.extend(requests);

                warn!("BLAS scratch budget reached, {} builds deferred", deferred.len());

                ray_tracing.blas.request_queue.requeue(deferred);

                break;
            }

            scratch_cursor = next_scratch_cursor;

            let acceleration_structure = ray_tracing.factory.allocate(
                &ray_tracing.resource_factories.buffer_factory,
                &format!("blas_mesh_{}", blas_request.mesh_id.inner),
                sizes.acceleration_structure_size,
                AccelerationStructureTypeKHR::BOTTOM_LEVEL,
            )?;

            let handle = acceleration_structure.handle;
            if let Some(displaced) = ray_tracing
                .blas
                .registry
                .insert(blas_request.mesh_id, acceleration_structure)
            {
                ray_tracing.blas.destroy_queue.push(displaced);
            }

            blas_builds.push(BLASBuild {
                blas_request,
                handle,
                scratch_offset,
            });
        }


        self.addresses.stage_slice(buffer_scope, allocator, &ray_tracing.blas.addresses())?;

        Ok(BLASBuildPassData {
            ray_tracing,
            blas_builds,
        })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        _readback_scope: &ReadbackScope,
        data: Self::PassData,
    ) -> Result<()> {
        if data.blas_builds.is_empty() {
            return Ok(());
        }

        let index_stride = size_of::<u32>() as u32;
        let command_buffer = context.command_buffer();
        let scratch_address = data.ray_tracing.blas.scratch_address(context.frame_index);

        let mut geometries = Vec::new();
        let mut range_infos = Vec::new();

        for blas_build in &data.blas_builds {
            let geometry = vec![data.ray_tracing.blas.geometry; blas_build.blas_request.submeshes.len()];

            let mut build_range_infos = Vec::new();

            for submesh in &blas_build.blas_request.submeshes {
                let range_info = AccelerationStructureBuildRangeInfoKHR::default()
                    .primitive_count(submesh.index_count / 3)
                    .primitive_offset(submesh.index_offset * index_stride)
                    .first_vertex(submesh.vertex_offset)
                    .transform_offset(0);

                build_range_infos.push(range_info);
            }

            geometries.push(geometry);
            range_infos.push(build_range_infos);
        }

        let mut build_geometry_infos = Vec::new();
        let mut range_slices = Vec::new();

        for ((blas_build, geometry), build_range_infos) in data
            .blas_builds
            .iter()
            .zip(&geometries)
            .zip(&range_infos)
        {
            let build_geometry_info = blas_build_geometry_info(geometry)
                .dst_acceleration_structure(blas_build.handle)
                .scratch_data(DeviceOrHostAddressKHR {
                    device_address: scratch_address + blas_build.scratch_offset,
                });

            build_geometry_infos.push(build_geometry_info);
            range_slices.push(build_range_infos.as_slice());
        }

        unsafe {
            data.ray_tracing.context.device.cmd_build_acceleration_structures(
                command_buffer,
                &build_geometry_infos,
                &range_slices,
            );
        }

        Ok(())
    }

    fn destroy(self, _resource_factories: &ResourceFactories) -> Result<()> {
        Ok(())
    }
}
