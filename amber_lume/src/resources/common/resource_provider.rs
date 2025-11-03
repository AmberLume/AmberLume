use crate::resources::common::internal_state::InternalState;
use crate::resources::common::res_ref::{ResRef, ResState};
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use arc_swap::ArcSwap;
use dashmap::{DashMap, Entry};
use parking_lot::RwLock;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread::{Builder, JoinHandle};
use tracing::{instrument, trace, warn};

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
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),

            key_to_id: DashMap::new(),
            states: ArcSwap::new(Arc::new(Vec::new())),

            tasks: DashMap::new(),
            next_task_id: AtomicU64::new(0),
            next_id: AtomicUsize::new(0),
        }
    }

    pub fn from(backend: B) -> Arc<Self> {
        Arc::new(Self::new(backend))
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

    pub fn get_ready(&self, id: &ResourceId, sync: bool) -> Option<Arc<B::Output>> {
        let slots = self.states.load();
        let slot = slots.get(*id as usize)?;

        if let InternalState::Ready { resource } = &*slot.read() {
            return Some(resource.clone());
        }

        if !sync {
            trace!("Slot {:?} is not ready", id);
            return None;
        }

        if let InternalState::Creating { task_id } = *slot.read() {
            if let Some((_, task)) = self.tasks.remove(&task_id) {
                let _ = task.join();
            }
        }

        match &*slot.read() {
            InternalState::Ready { resource } => Some(resource.clone()),
            _ => None,
        }
    }

    fn handle_new(&self, resref: &ResRef<B::Config>) -> ResourceId {
        let id = self.get_id(&resref.meta.config);
        let slot = self.get_slot(id);

        let task_id_optional = {
            let mut slot = slot.write();

            match &*slot {
                InternalState::Creating { .. } | InternalState::Ready { .. } => None,
                InternalState::Failed { .. } | InternalState::Evicted => {
                    let task_id = self.next_task_id.fetch_add(1, Ordering::Relaxed);
                    *slot = InternalState::Creating { task_id };
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
                    let mut dependencies_refs = backend.build_dependencies_refs(&config);
                    let dependencies = backend.collect_dependencies(&mut dependencies_refs);

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
                id
            }
        }
    }

    pub fn get_storage(&self) -> B::Storage {
        self.backend.get_storage()
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
}
