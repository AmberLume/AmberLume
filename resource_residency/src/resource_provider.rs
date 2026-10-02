use crate::res_ref::ResRef;
use crate::resource_backend::ResourceBackend;
use crate::resource_hash::ResourceHash;
use crate::resource_usage_statistics::ResourceUsageStatistics;
use anyhow::{bail, Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use dashmap::{DashMap, DashSet};
use index_allocator::ArcUnwrapOrErr;
use index_allocator::DeferredDestroy;
use index_allocator::{IndexManager, ResourceId};
use crate::task_scheduler::TaskScheduler;
use crate::thread_task_scheduler::ThreadTaskScheduler;
use std::hash::Hash;
use std::sync::Arc;
use tracing::error;

struct ResourceReadyEvent<T> {
    id: ResourceId,
    key: ResourceHash,
    resource: Option<T>,
}

struct ResourceWrite<C> {
    id: ResourceId,
    config: C,
}

pub struct ResourceProvider<B: ResourceBackend> {
    pub backend: Arc<B>,

    index_manager: Arc<IndexManager>,

    active_resources: Arc<DashMap<ResourceId, B::Output>>,
    asset_cache: DashMap<ResourceHash, Arc<ResRef>>,

    pending_creations: DashSet<ResourceId>,
    deferred_releases: DashSet<ResourceId>,
    deferred_destroy: Arc<DeferredDestroy>,

    scheduler: Arc<dyn TaskScheduler>,

    ready_rx: Receiver<ResourceReadyEvent<B::Output>>,
    ready_tx: Sender<ResourceReadyEvent<B::Output>>,

    write_rx: Receiver<ResourceWrite<B::Config>>,
    write_tx: Sender<ResourceWrite<B::Config>>,

    drop_rx: Receiver<ResourceId>,
    drop_tx: Sender<ResourceId>,
}

impl<B: ResourceBackend> ResourceProvider<B> {
    pub fn from(
        backend: B,
        index_manager: Arc<IndexManager>,
        deferred_destroy: Arc<DeferredDestroy>,
    ) -> Arc<Self> {
        Self::with_scheduler(
            backend,
            index_manager,
            deferred_destroy,
            Arc::new(ThreadTaskScheduler::create()),
        )
    }

    pub fn with_scheduler(
        backend: B,
        index_manager: Arc<IndexManager>,
        deferred_destroy: Arc<DeferredDestroy>,
        scheduler: Arc<dyn TaskScheduler>,
    ) -> Arc<Self> {
        let (ready_tx, ready_rx) = unbounded();
        let (write_tx, write_rx) = unbounded();
        let (drop_tx, drop_rx) = unbounded();

        Arc::new(Self {
            backend: Arc::new(backend),

            index_manager,

            active_resources: Arc::new(DashMap::new()),
            asset_cache: DashMap::new(),

            pending_creations: DashSet::new(),
            deferred_releases: DashSet::new(),
            deferred_destroy,

            scheduler,

            ready_rx,
            ready_tx,

            write_rx,
            write_tx,

            drop_tx,
            drop_rx,
        })
    }

    pub fn get_or_load(&self, config: B::Config) -> Result<Arc<ResRef>>
    where
        B::Config: Hash,
    {
        let key = ResourceHash::of(&config);

        if let Some(cached) = self.asset_cache.get(&key) {
            return Ok(cached.clone());
        }

        let id = self
            .index_manager
            .acquire()
            .context("Out of resource indices")?;

        let res_ref = Arc::new(ResRef::new(id, self.drop_tx.clone()));

        self.asset_cache.insert(key, res_ref.clone());

        let backend = self.backend.clone();
        let tx = self.ready_tx.clone();

        self.pending_creations.insert(id);

        self.scheduler.schedule(Box::new(move || {
            let resource = match backend.create(&id, config) {
                Ok(resource) => Some(resource),
                Err(error) => {
                    error!("Failed to create resource {}: {:#}", id.inner, error);

                    None
                }
            };

            let _ = tx.send(ResourceReadyEvent { id, key, resource });
        }));

        Ok(res_ref)
    }

    pub fn acquire_sync(&self, config: B::Config) -> Result<Arc<ResRef>>
    where
        B::Config: Hash,
    {
        let key = ResourceHash::of(&config);

        if let Some(cached) = self.asset_cache.get(&key) {
            return Ok(cached.clone());
        }

        let id = self
            .index_manager
            .acquire()
            .context("Out of resource indices")?;

        let resource = match self.backend.create(&id, config) {
            Ok(resource) => resource,
            Err(error) => {
                self.index_manager.release(id);

                return Err(error);
            }
        };

        self.active_resources.insert(id, resource);

        let res_ref = Arc::new(ResRef::new(id, self.drop_tx.clone()));

        self.asset_cache.insert(key, res_ref.clone());

        Ok(res_ref)
    }

    pub fn reserve(&self) -> Result<Arc<ResRef>> {
        let id = self
            .index_manager
            .acquire()
            .context("Out of resource indices")?;

        if let Err(error) = self.backend.reserve(&id) {
            self.index_manager.release(id);

            return Err(error);
        }

        Ok(Arc::new(ResRef::new(id, self.drop_tx.clone())))
    }

    pub fn write(&self, res_ref: &ResRef, config: B::Config) -> Result<()> {
        if self.active_resources.contains_key(&res_ref.id) || !self.pending_creations.insert(res_ref.id) {
            bail!("Resource {} is already written", res_ref.id.inner);
        }

        self.write_tx.send(ResourceWrite {
            id: res_ref.id,
            config,
        })?;

        Ok(())
    }

    pub fn with_resource<R>(&self, id: ResourceId, action: impl FnOnce(&B::Output) -> R) -> Option<R> {
        self.active_resources.get(&id).map(|entry| action(entry.value()))
    }

    pub fn update(&self) -> Vec<ResourceId> {
        let mut retired = Vec::new();

        while let Ok(write) = self.write_rx.try_recv() {
            self.pending_creations.remove(&write.id);

            match self.backend.create(&write.id, write.config) {
                Ok(resource) => {
                    self.active_resources.insert(write.id, resource);
                }
                Err(error) => {
                    error!("Failed to write resource {}: {:#}", write.id.inner, error);
                }
            }

            if self.deferred_releases.remove(&write.id).is_some() {
                self.retire(write.id);

                retired.push(write.id);
            }
        }

        while let Ok(event) = self.ready_rx.try_recv() {
            self.pending_creations.remove(&event.id);

            match event.resource {
                Some(resource) => {
                    self.active_resources.insert(event.id, resource);
                }
                None => {
                    self.asset_cache.remove(&event.key);
                }
            }

            if self.deferred_releases.remove(&event.id).is_some() {
                self.retire(event.id);

                retired.push(event.id);
            }
        }

        self.asset_cache
            .retain(|_, res_ref| Arc::strong_count(res_ref) > 1);

        while let Ok(id) = self.drop_rx.try_recv() {
            if self.pending_creations.contains(&id) {
                self.deferred_releases.insert(id);
            } else {
                self.retire(id);

                retired.push(id);
            }
        }

        retired
    }

    fn retire(&self, id: ResourceId) {
        let backend = self.backend.clone();
        let active_resources = self.active_resources.clone();
        let index_manager = self.index_manager.clone();

        self.deferred_destroy.push(move || {
            let destroyed = match active_resources.remove(&id) {
                Some((_, resource)) => backend.destroy_resource(resource),
                None => Ok(()),
            };
            let erased = backend.erase(&id);

            index_manager.release(id);

            destroyed.and(erased)
        });
    }

    pub fn statistics(&self) -> ResourceUsageStatistics<B::Statistics> {
        ResourceUsageStatistics {
            index: self.index_manager.statistics(),

            backend: self.backend.statistics(),
        }
    }

    pub fn destroy(self) -> Result<()> {
        let Self { backend, active_resources, asset_cache, .. } = self;

        asset_cache.clear();

        let ids: Vec<ResourceId> = active_resources.iter().map(|r| *r.key()).collect();

        for id in ids {
            if let Some((_, resource)) = active_resources.remove(&id) {
                let _ = backend.destroy_resource(resource);
            }
        }

        backend.try_unwrap()?.destroy()?;

        Ok(())
    }
}
