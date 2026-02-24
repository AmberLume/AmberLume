use crate::resources::descriptor_index_manager::IndexManager;
use crate::resources::dynamic::res_ref::ResRef;
use crate::resources::dynamic::resource_backend::{ResourceBackend, ResourceKey};
use crossbeam_channel::{Receiver, Sender};
use dashmap::DashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Weak};
use std::thread::spawn;

pub type ResourceId = u32;

pub struct ResourceReadyEvent<T> {
    pub id: ResourceId,
    pub resource: T,
}

pub struct ResourceProvider<B: ResourceBackend> {
    backend: Arc<B>,

    index_manager: Arc<IndexManager>,
    frame_counter: Arc<AtomicU64>,

    active_resources: DashMap<ResourceId, B::Output>,
    asset_cache: DashMap<ResourceKey, Weak<ResRef>>,

    ready_rx: Receiver<ResourceReadyEvent<B::Output>>,
    ready_tx: Sender<ResourceReadyEvent<B::Output>>,
}

impl<B: ResourceBackend> ResourceProvider<B> {
    pub fn from(
        backend: B,
        index_manager: Arc<IndexManager>,
        frame_counter: Arc<AtomicU64>,
    ) -> Arc<Self> {
        let (ready_tx, ready_rx) = crossbeam_channel::unbounded();

        Arc::new(Self {
            backend: Arc::new(backend),

            index_manager,
            frame_counter,

            active_resources: DashMap::new(),
            asset_cache: DashMap::new(),

            ready_rx,
            ready_tx,
        })
    }

    pub fn get_or_load(&self, config: B::Config) -> Arc<ResRef> {
        let key = B::key_from(&config);

        if let Some(weak) = self.asset_cache.get(&key) {
            if let Some(arc) = weak.upgrade() {
                return arc;
            }
        }

        let id = self.index_manager.acquire().expect("Out of resource indices!");

        let res_ref = Arc::new(ResRef {
            id,
            index_manager: self.index_manager.clone(),
            frame_counter: self.frame_counter.clone(),
        });

        self.asset_cache.insert(key, Arc::downgrade(&res_ref));

        let backend = self.backend.clone();
        let tx = self.ready_tx.clone();

        spawn(move || {
            if let Ok(resource) = backend.create(&id, config) {
                let _ = tx.send(ResourceReadyEvent {
                    id,
                    resource,
                });
            }
        });

        res_ref
    }

    pub fn acquire_sync(&self, config: B::Config) -> Arc<ResRef> {
        let key = B::key_from(&config);

        if let Some(weak) = self.asset_cache.get(&key) {
            if let Some(arc) = weak.upgrade() {
                return arc;
            }
        }

        let id = self.index_manager.acquire().expect("Out of resource indices!");

        let resource = self.backend.create(&id, config)
            .expect("Failed to create resource synchronously");

        self.active_resources.insert(id, resource);

        let resource_guard = self.active_resources.get(&id).unwrap();
        self.backend.set_resource(&id, resource_guard.value()).expect("Failed to set resource");

        let res_ref = Arc::new(ResRef {
            id,
            index_manager: self.index_manager.clone(),
            frame_counter: self.frame_counter.clone(),
        });

        self.asset_cache.insert(key, Arc::downgrade(&res_ref));

        res_ref
    }

    pub fn get_resource(&self, id: ResourceId) -> Option<B::Output>
    where
        B::Output: Clone,
    {
        self.active_resources.get(&id).map(|r| r.value().clone())
    }

    pub fn update(&self) {
        let freed_indices = self.index_manager.update();
        for id in freed_indices {
            if let Some((_, resource)) = self.active_resources.remove(&id) {
                let _ = self.backend.destroy_resource(resource);
            }

            let _ = self.backend.set_default(&id);
        }

        while let Ok(event) = self.ready_rx.try_recv() {
            let _ = self.backend.set_resource(&event.id, &event.resource);

            self.active_resources.insert(event.id, event.resource);
        }
    }

    pub fn destroy(&self) {
        self.asset_cache.clear();

        let ids: Vec<ResourceId> = self.active_resources.iter().map(|r| *r.key()).collect();

        for id in ids {
            if let Some((_, resource)) = self.active_resources.remove(&id) {
                let _ = self.backend.destroy_resource(resource);

                let _ = self.backend.set_default(&id);
            }
        }
    }
}
