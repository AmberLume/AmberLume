use ash::vk::{
    AccelerationStructureBuildGeometryInfoKHR, AccelerationStructureGeometryDataKHR,
    AccelerationStructureGeometryKHR, AccelerationStructureGeometryTrianglesDataKHR,
    AccelerationStructureTypeKHR, BuildAccelerationStructureFlagsKHR,
    BuildAccelerationStructureModeKHR, DeviceAddress, DeviceOrHostAddressConstKHR, Format,
    GeometryFlagsKHR, GeometryTypeKHR, IndexType,
};
use gpu::GpuSize;
use gpu_data::VertexPositionGPU;
use resource_store::GeometryRange;

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
