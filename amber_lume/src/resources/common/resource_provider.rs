use crate::resources::common::internal_state::InternalState;
use crate::resources::common::res_ref::{ResRef, ResState};
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use anyhow::{Result, bail};
use arc_swap::ArcSwap;
use dashmap::{DashMap, Entry};
use parking_lot::RwLock;
use std::mem::replace;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread::{Builder, JoinHandle};
use tracing::{info, instrument, trace, warn};

pub type ResourceId = u32;

pub struct ResourceProvider<B: ResourceBackend> {
    backend: Arc<B>,

    key_to_id: DashMap<ResourceKey, ResourceId>,
    states: ArcSwap<Vec<Arc<RwLock<InternalState<B::Output>>>>>,

    tasks: DashMap<u64, JoinHandle<()>>,
    next_task_id: AtomicU64,
    next_id: AtomicUsize,
}

impl<B: ResourceBackend> ResourceProvider<B> {
    pub fn from(backend: B) -> Arc<Self> {
        let provider = Self {
            backend: Arc::new(backend),

            key_to_id: DashMap::new(),
            states: ArcSwap::new(Arc::new(Vec::new())),

            tasks: DashMap::new(),
            next_task_id: AtomicU64::new(0),
            next_id: AtomicUsize::new(0),
        };

        Arc::new(provider)
    }

    #[instrument(skip(self), level = "trace")]
    pub fn get_now(&self, config: &B::Config) -> Option<Arc<B::Output>> {
        let id = self.get_id(&config);
        self.touch(config);

        self.get_ready(&id, true)
    }

    #[instrument(skip(self), level = "trace")]
    pub fn get_id(&self, config: &B::Config) -> ResourceId {
        let key = B::key_from(&config);

        if let Some(id) = self.key_to_id.get(&key) {
            return *id;
        }

        match self.key_to_id.entry(key) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = self.next_id.fetch_add(1, Ordering::Relaxed) as ResourceId;
                entry.insert(id);

                info!("Added new resource. ID: {}, Value: {:?}", id, config);

                id
            }
        }
    }

    #[instrument(skip(self, config), level = "trace")]
    pub fn touch(&self, config: &B::Config) {
        let mut resref = B::create_ref(config);

        self.ensure(&mut resref)
    }

    #[instrument(skip(self, resref), level = "trace")]
    pub fn ensure(&self, resref: &mut ResRef<B::Config>) {
        match &resref.state {
            ResState::New => {
                let id = self.handle_new(&resref);

                resref.state = ResState::Pending { id };
            }
            ResState::Pending { id } => {
                resref.state = self.get_res_state(*id);
            }
            ResState::Evicted { .. } => {
                let id = self.handle_new(&resref);

                resref.state = ResState::Pending { id };
            }
            ResState::Resolved { .. } => {}
            ResState::Broken { .. } => {
                trace!("ResRef is broken, cannot be resolved");
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    pub fn get_ready(&self, id: &ResourceId, sync: bool) -> Option<Arc<B::Output>> {
        let slot = {
            let slots = self.states.load();

            slots.get(*id as usize)?.clone()
        };

        {
            if let InternalState::Ready { resource } = &*slot.read() {
                return Some(resource.clone());
            }
        }

        if !sync {
            return None;
        }

        let task_id = {
            match &*slot.read() {
                InternalState::Creating { task_id } => Some(*task_id),
                _ => None,
            }
        };

        if let Some(task_id) = task_id {
            if let Some((_, task)) = self.tasks.remove(&task_id) {
                let _ = task.join();
            }
        }

        {
            match &*slot.read() {
                InternalState::Ready { resource } => Some(resource.clone()),
                _ => None,
            }
        }
    }

    #[instrument(skip(self), level = "trace")]
    fn handle_new(&self, resref: &ResRef<B::Config>) -> ResourceId {
        let id = self.get_id(&resref.meta.config);
        let slot = self.get_slot(id);

        let task_id_optional = {
            let mut state = slot.write();

            match &*state {
                InternalState::Creating { .. } | InternalState::Ready { .. } => None,
                InternalState::Failed { .. } | InternalState::Evicted => {
                    let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
                    *state = InternalState::Creating { task_id };
                    Some(task_id)
                }
            }
        };

        if let Some(task_id) = task_id_optional {
            let backend = self.backend.clone();
            let config = resref.meta.config.clone();
            let slot = slot.clone();

            let handle = Builder::new()
                .name(format!("resources-{}-{}", id, task_id))
                .spawn(move || {
                    let dependencies = backend.collect_dependencies(&config);

                    let result = backend.create(config, dependencies);

                    let mut slot = slot.write();

                    match result {
                        Ok(resource) => {
                            *slot = InternalState::Ready {
                                resource: Arc::new(resource),
                            }
                        }
                        Err(error) => {
                            *slot = InternalState::Failed {
                                error: Arc::new(error),
                            };
                        }
                    }
                })
                .expect("Can't spawn new resources thread");

            self.tasks.insert(task_id, handle);
        }

        id
    }

    #[instrument(skip(self), level = "trace")]
    fn expand_states_to_include(&self, id: ResourceId) {
        loop {
            let current = self.states.load_full();
            if (id as usize) < current.len() {
                return;
            }

            let mut next = current.as_ref().clone();

            while next.len() <= (id as usize) {
                next.push(Arc::new(RwLock::new(InternalState::Evicted)));
            }

            let previous = self.states.compare_and_swap(&current, Arc::new(next));

            if Arc::ptr_eq(&previous, &current) {
                return;
            }
        }
    }

    fn get_slot(&self, id: ResourceId) -> Arc<RwLock<InternalState<B::Output>>> {
        let vector = self.states.load();

        if (id as usize) < vector.len() {
            vector[id as usize].clone()
        } else {
            drop(vector);
            self.expand_states_to_include(id);
            self.states.load()[id as usize].clone()
        }
    }

    fn get_res_state(&self, id: ResourceId) -> ResState {
        match &*self.states.load()[id as usize].read() {
            InternalState::Creating { .. } => ResState::Pending { id },
            InternalState::Ready { .. } => ResState::Resolved { id },
            InternalState::Failed { error } => ResState::Broken {
                id,
                error: error.clone(),
            },
            InternalState::Evicted => ResState::Evicted { id },
        }
    }

    pub fn destroy(&self) -> Result<()> {
        let slots = self.states.load();

        for (index, slot) in slots.iter().enumerate() {
            let mut state = slot.write();

            let old_state = replace(&mut *state, InternalState::Evicted);

            if let InternalState::Ready { resource } = old_state {
                match Arc::try_unwrap(resource) {
                    Ok(resource) => self.backend.destroy_resource(resource)?,
                    Err(resource) => {
                        let strong_count = Arc::strong_count(&resource);
                        *state = InternalState::Ready { resource };

                        bail!(
                            "Can't destroy resource at index {}. Still in use: strong_count = {}",
                            index,
                            strong_count
                        );
                    }
                }
            }
        }

        match Arc::try_unwrap(self.backend.clone()) {
            Ok(mut backend) => backend.destroy()?,
            Err(backend) => {
                let strong_count = Arc::strong_count(&backend);

                bail!(
                    "Can't destroy backend. Still in use: strong_count = {}",
                    strong_count
                );
            }
        }

        Ok(())
    }
}
