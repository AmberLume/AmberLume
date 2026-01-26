use crate::resources::common::internal_state::InternalState;
use crate::resources::common::resource_backend::{ResourceBackend, ResourceKey};
use crate::resources::res_ref::{ResRef, ResState};
use anyhow::{Result, bail, anyhow};
use arc_swap::ArcSwap;
use dashmap::{DashMap, Entry};
use parking_lot::RwLock;
use std::mem::replace;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::thread::{Builder, JoinHandle};
use tracing::{error, info};

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

    pub fn get_now(&self, config: &B::Config) -> Arc<B::Output> {
        let id = self.get_id(&config);
        self.touch(config);

        self.get_ready(&id)
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

                info!("Added new resource. ID: {}, Value: {:?}", id, config);

                id
            }
        }
    }

    pub fn touch(&self, config: &B::Config) {
        let mut resref = ResRef::from(config.clone());

        self.ensure(&mut resref)
    }

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
            ResState::Broken { id, error } => {
                error!("ResRef is broken, cannot be resolved. ID: {}, error: {}", id, error);
            }
        }
    }

    pub fn evict(&self, id: ResourceId) {
        let slots = self.states.load();
        let resource = slots.get(id as usize);

        if let Some(resource) = resource {
            let mut state = resource.write();

            if let InternalState::Ready { resource } = replace(&mut *state, InternalState::Evicted) {
                match Arc::try_unwrap(resource) {
                    Ok(res) => self.backend.destroy_resource(res).unwrap(),
                    Err(_) => unreachable!(),
                }
            }
        }
    }

    pub fn get_ready(&self, id: &ResourceId) -> Arc<B::Output> {
        let slots = self.states.load();
        let slot = slots.get(*id as usize)
            .expect("Resource ID out of bounds")
            .clone();

        {
            let state = slot.read();
            if let InternalState::Ready { resource } = &*state {
                return resource.clone();
            }
        }

        let task_id = {
            let state = slot.read();
            if let InternalState::Creating { task_id } = &*state {
                Some(*task_id)
            } else {
                None
            }
        };

        if let Some(tid) = task_id {
            if let Some((_, handle)) = self.tasks.remove(&tid) {
                let _ = handle.join();
            }

            loop {
                {
                    let state = slot.read();
                    if let InternalState::Ready { resource } = &*state {
                        return resource.clone();
                    }
                }
                thread::yield_now();
            }
        }

        let state = slot.read();
        if let InternalState::Ready { resource } = &*state {
            resource.clone()
        } else {
            unreachable!()
        }
    }

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

            let thread_name = format!("resources-{}", id);

            let handle = Builder::new()
                .name(thread_name.clone())
                .spawn(move || {
                    info!("Spawn resource thread(name: {}, config: {:?})", thread_name, config);
                    let dependencies = backend.collect_dependencies(&config);

                    let result = backend.create(&id, config, dependencies);

                    let mut slot = slot.write();

                    match result {
                        Ok(resource) => {
                            *slot = InternalState::Ready {
                                resource: Arc::new(resource),
                            }
                        }
                        Err(error) => {
                            error!("Failed to create resource {}: {}", id, error);

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

    pub fn destroy(self: Arc<Self>) -> Result<()> {
        let provider = Arc::try_unwrap(self)
            .map_err(|arc| {
                anyhow!("Provider still in use: {} references", Arc::strong_count(&arc))
            })?;

        for (_, task) in provider.tasks.into_iter() {
            let _ = task.join();
        }

        let slots = provider.states.into_inner();
        for (index, slot) in slots.iter().enumerate() {
            let state = slot.write();
            if let InternalState::Ready { resource } = &*state {
                if Arc::strong_count(resource) > 1 {
                    bail!("Resource {} still in use: {} refs",index, Arc::strong_count(resource));
                }
            }
        }

        for slot in slots.iter() {
            let mut state = slot.write();
            if let InternalState::Ready { resource } =
                replace(&mut *state, InternalState::Evicted)
            {
                match Arc::try_unwrap(resource) {
                    Ok(res) => provider.backend.destroy_resource(res)?,
                    Err(_) => unreachable!(),
                }
            }
        }

        match Arc::try_unwrap(provider.backend) {
            Ok(mut backend) => {
                backend.destroy()?;

                drop(backend);
            },
            Err(_) => unreachable!(),
        }

        Ok(())
    }
}
