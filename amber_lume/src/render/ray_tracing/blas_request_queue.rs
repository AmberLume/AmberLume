use std::mem::take;
use parking_lot::Mutex;
use crate::resources::store::providers::mesh::buffer::mesh_buffer::MeshGPU;
use crate::resources::store::providers::mesh::buffer::submesh_buffer::SubmeshGPU;
use crate::resources::store::providers::mesh::mesh_load_observer::MeshLoadObserver;
use crate::resources::store::providers::resource_provider::ResourceId;

pub struct BLASRequest {
    pub mesh_id: ResourceId,
    pub submeshes: Vec<SubmeshGPU>,
}

pub struct BLASRequestQueue {
    blas_requests: Mutex<Vec<BLASRequest>>,
}

impl BLASRequestQueue {
    pub fn new() -> Self {
        Self {
            blas_requests: Mutex::new(Vec::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.blas_requests.lock().is_empty()
    }

    pub fn drain(&self) -> Vec<BLASRequest> {
        take(&mut *self.blas_requests.lock())
    }
}

impl MeshLoadObserver for BLASRequestQueue {
    fn on_load(&self, mesh_id: ResourceId, _mesh: &MeshGPU, submeshes: &[SubmeshGPU]) {
        self.blas_requests.lock().push(BLASRequest { mesh_id, submeshes: submeshes.to_vec() });
    }
}
