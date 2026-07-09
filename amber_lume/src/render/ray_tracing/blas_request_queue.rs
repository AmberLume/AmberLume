use std::collections::HashMap;
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
    loaded: Mutex<HashMap<ResourceId, Vec<SubmeshGPU>>>,
}

impl BLASRequestQueue {
    pub fn new() -> Self {
        Self {
            blas_requests: Mutex::new(Vec::new()),
            loaded: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.blas_requests.lock().is_empty()
    }

    pub fn drain(&self) -> Vec<BLASRequest> {
        take(&mut *self.blas_requests.lock())
    }

    pub fn requeue_all(&self) {
        let loaded = self.loaded.lock();

        *self.blas_requests.lock() = loaded
            .iter()
            .map(|(mesh_id, submeshes)| BLASRequest {
                mesh_id: *mesh_id,
                submeshes: submeshes.clone(),
            })
            .collect();
    }
}

impl MeshLoadObserver for BLASRequestQueue {
    fn on_load(&self, mesh_id: ResourceId, _mesh: &MeshGPU, submeshes: &[SubmeshGPU]) {
        self.loaded.lock().insert(mesh_id, submeshes.to_vec());
        self.blas_requests.lock().push(BLASRequest { mesh_id, submeshes: submeshes.to_vec() });
    }
}
