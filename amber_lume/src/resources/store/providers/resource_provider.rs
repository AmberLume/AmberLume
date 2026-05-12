use crate::resources::index::index_manager::IndexManager;
use crate::resources::store::providers::res_ref::ResRef;
use crate::resources::store::providers::resource_backend::{ResourceBackend, ResourceHash};
use crossbeam_channel::{unbounded, Receiver, Sender};
use dashmap::DashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::thread::spawn;
use crate::resources::store::providers::resource_usage_statistics::ResourceUsageStatistics;
use crate::utils::arc_utils::ArcUnwrapOrErr;
use anyhow::Result;
use crate::render::factories::resource_factories::ResourceFactories;

pub type ResourceId = u32;

pub struct ResourceReadyEvent<T> {
    pub id: ResourceId,
    pub resource: T,
}

pub struct ResourceProvider<B: ResourceBackend> {
    pub backend: Arc<B>,

    index_manager: IndexManager,

    active_resources: DashMap<ResourceId, Arc<B::Output>>,
    asset_cache: DashMap<ResourceHash, Arc<ResRef>>,

    ready_rx: Receiver<ResourceReadyEvent<B::Output>>,
    ready_tx: Sender<ResourceReadyEvent<B::Output>>,

    drop_rx: Receiver<ResourceId>,
    drop_tx: Sender<ResourceId>,
}

impl<B: ResourceBackend> ResourceProvider<B> {
    pub fn from(
        backend: B,
        capacity: u32,
        delay: u32,
        frame_counter: Arc<AtomicU64>,
    ) -> Arc<Self> {
        let (ready_tx, ready_rx) = unbounded();
        let (drop_tx, drop_rx) = unbounded();

        let index_manager = IndexManager::new(capacity, delay, frame_counter.clone());

        Arc::new(Self {
            backend: Arc::new(backend),

            index_manager,

            active_resources: DashMap::new(),
            asset_cache: DashMap::new(),

            ready_rx,
            ready_tx,

            drop_tx,
            drop_rx,
        })
    }

    pub fn get_or_load(&self, config: B::Config) -> Arc<ResRef> {
        let key = B::resource_hash_from(&config);

        if let Some(cached) = self.asset_cache.get(&key) {
            return cached.clone();
        }

        let id = self.index_manager.acquire().expect("Out of resource indices!");

        let res_ref = Arc::new(ResRef::new(id, self.drop_tx.clone()));

        self.asset_cache.insert(key, res_ref.clone());

        let backend = self.backend.clone();
        let tx = self.ready_tx.clone();

        spawn(move || {
            if let Ok(resource) = backend.create(&id, config) {
                let _ = tx.send(ResourceReadyEvent { id, resource });
            }
        });

        res_ref
    }

    pub fn acquire_sync(&self, config: B::Config) -> Arc<ResRef> {
        let key = B::resource_hash_from(&config);

        if let Some(cached) = self.asset_cache.get(&key) {
            return cached.clone();
        }

        let id = self.index_manager.acquire().expect("Out of resource indices!");

        let resource = self.backend.create(&id, config)
            .expect("Failed to create resource synchronously");

        self.active_resources.insert(id, Arc::new(resource));

        let res_ref = Arc::new(ResRef::new(id, self.drop_tx.clone()));

        self.asset_cache.insert(key, res_ref.clone());

        res_ref
    }

    pub fn get_resource(&self, id: ResourceId) -> Option<Arc<B::Output>> {
        self.active_resources
            .get(&id)
            .map(|r| r.value().clone())
    }

    pub fn update(&self) {
        self.asset_cache.retain(|_, res_ref| Arc::strong_count(res_ref) > 1);

        while let Ok(id) = self.drop_rx.try_recv() {
            self.index_manager.release(id);
        }

        let freed_indices = self.index_manager.update();
        for id in freed_indices {
            if let Some((_, resource)) = self.active_resources.remove(&id) {
                let resource = resource
                    .try_unwrap()
                    .expect("Failed to unwrap resource");

                let _ = self.backend.destroy_resource(resource);
            }

            let _ = self.backend.erase(&id);
        }

        while let Ok(event) = self.ready_rx.try_recv() {
            self.active_resources.insert(event.id, Arc::new(event.resource));
        }
    }

    pub fn statistics(&self) -> ResourceUsageStatistics<B::Statistics> {
        ResourceUsageStatistics {
            index: self.index_manager.statistics(),

            backend: self.backend.statistics(),
        }
    }

    pub fn destroy(self, resource_factories: &ResourceFactories) -> Result<()> {
        self.asset_cache.clear();

        let ids: Vec<ResourceId> = self.active_resources.iter().map(|r| *r.key()).collect();

        for id in ids {
            if let Some((_, resource)) = self.active_resources.remove(&id) {
                let resource = resource
                    .try_unwrap()
                    .expect("Failed to unwrap resource");

                let _ = self.backend.destroy_resource(resource);
            }
        }

        self.backend.try_unwrap()?.destroy(&resource_factories.buffer_factory)?;

        Ok(())
    }
}
