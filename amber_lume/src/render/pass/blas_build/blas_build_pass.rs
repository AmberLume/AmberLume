use gpu::ResourceFactories;
use render_graph::FrameContext;
use ray_tracing::BLASRequest;
use ray_tracing::blas_build_geometry_info;
use ray_tracing::{align_up, RayTracing};
use render_graph::Pass;
use render_graph::PassResourceDeclaration;
use render_graph::VirtualAccelerationStructure;
use render_graph::HeapAllocator;
use render_graph::BufferResourceScope;
use render_graph::DataResourceScope;
use render_graph::ImageResourceScope;
use anyhow::{ensure, Result};
use ash::vk::{AccelerationStructureBuildRangeInfoKHR, AccelerationStructureBuildSizesInfoKHR, AccelerationStructureBuildTypeKHR, AccelerationStructureKHR, AccelerationStructureTypeKHR, AccessFlags, DeviceOrHostAddressKHR, DeviceSize, PipelineStageFlags};
use std::mem::size_of;
use std::sync::Arc;

struct BLASBuild {
    blas_request: BLASRequest,
    handle: AccelerationStructureKHR,
    scratch_offset: DeviceSize,
}

pub struct BLASBuildPassData {
    blas_builds: Vec<BLASBuild>,
}

pub struct BLASBuildPass {
    ray_tracing: Arc<RayTracing>,
    blas: VirtualAccelerationStructure,
}

impl BLASBuildPass {
    pub fn create(ray_tracing: Arc<RayTracing>, blas: VirtualAccelerationStructure) -> Self {
        Self { ray_tracing, blas }
    }
}

impl Pass for BLASBuildPass {
    type PassData = BLASBuildPassData;

    fn name(&self) -> String {
        String::from("blas_build")
    }

    fn is_enabled(&self, _data_scope: &DataResourceScope) -> bool {
        self.ray_tracing.blas.has_pending()
    }

    fn declare_resources(&self, declaration: &mut PassResourceDeclaration) {
        declaration.write_acceleration_structure(
            self.blas,
            AccessFlags::ACCELERATION_STRUCTURE_WRITE_KHR,
            PipelineStageFlags::ACCELERATION_STRUCTURE_BUILD_KHR,
        );
    }

    fn prepare_data(
        &self,
        _data_scope: &mut DataResourceScope,
        _buffer_scope: &mut BufferResourceScope,
        _allocator: &mut HeapAllocator,
    ) -> Result<Self::PassData> {
        let alignment = self.ray_tracing.rt_limits.min_scratch_offset_alignment as DeviceSize;
        let capacity = self.ray_tracing.blas.scratch_capacity;

        let mut blas_builds = Vec::new();
        let mut scratch_cursor: DeviceSize = 0;

        for blas_request in self.ray_tracing.blas.request_queue.drain() {
            if blas_request.submeshes.is_empty() {
                continue;
            }

            let geometries = vec![self.ray_tracing.blas.geometry; blas_request.submeshes.len()];
            let primitive_counts = blas_request
                .submeshes
                .iter()
                .map(|submesh| submesh.index_count / 3)
                .collect::<Vec<_>>();

            let size_geometry_info = blas_build_geometry_info(&geometries);

            let mut sizes = AccelerationStructureBuildSizesInfoKHR::default();
            unsafe {
                self.ray_tracing
                    .as_loader
                    .get_acceleration_structure_build_sizes(
                        AccelerationStructureBuildTypeKHR::DEVICE,
                        &size_geometry_info,
                        &primitive_counts,
                        &mut sizes,
                    );
            }

            let acceleration_structure = self.ray_tracing.factory.allocate(
                &self.ray_tracing.resource_factories.buffer_factory,
                &format!("blas_mesh_{}", blas_request.mesh_id.inner),
                sizes.acceleration_structure_size,
                AccelerationStructureTypeKHR::BOTTOM_LEVEL,
            )?;

            let handle = acceleration_structure.handle;
            let as_device_address = acceleration_structure.device_address;
            if let Some(displaced) = self
                .ray_tracing
                .blas
                .registry
                .insert(blas_request.mesh_id, acceleration_structure)
            {
                self.ray_tracing.blas.destroy_queue.push(displaced);
            }
            self.ray_tracing
                .blas
                .write_address(blas_request.mesh_id, as_device_address)?;

            let scratch_offset = align_up(scratch_cursor, alignment);
            scratch_cursor = scratch_offset + sizes.build_scratch_size;
            ensure!(
                scratch_cursor <= capacity,
                "BLAS scratch overflow: {scratch_cursor} > {capacity}"
            );

            blas_builds.push(BLASBuild {
                blas_request,
                handle,
                scratch_offset,
            });
        }

        Ok(BLASBuildPassData { blas_builds })
    }

    fn record_commands(
        &self,
        context: &FrameContext,
        _image_scope: &ImageResourceScope,
        _buffer_scope: &BufferResourceScope,
        data: Self::PassData,
    ) -> Result<()> {
        if data.blas_builds.is_empty() {
            return Ok(());
        }

        let index_stride = size_of::<u32>() as u32;
        let command_buffer = context.command_buffer();
        let scratch_address = self.ray_tracing.blas.scratch_address(context.frame_index);

        let mut geometries = Vec::new();
        let mut range_infos = Vec::new();

        for blas_build in &data.blas_builds {
            let geometry = vec![self.ray_tracing.blas.geometry; blas_build.blas_request.submeshes.len()];

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
            self.ray_tracing.as_loader.cmd_build_acceleration_structures(
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
