use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use dashmap::{DashMap, Entry};
use tracing::error;
use crate::processors::processor::Processor;
use crate::processors::shader_processor::ShaderProcessor;
use crate::processors::write_file_processor::WriteFileProcessor;
use crate::build_task::{BuildTask, BuildTaskKey, BuildTaskStatis};
use crate::processors::convert_ktx2_processor::ConvertKTX2Processor;
use crate::processors::assets::extract_assets_processor::ExtractAssetsProcessor;
use crate::processors::route_target_processor::RouteTargetProcessor;

pub struct Dispatcher {
    route_target_processor: Arc<RouteTargetProcessor>,
    shader_processor: Arc<ShaderProcessor>,
    parse_gltf_processor: Arc<ExtractAssetsProcessor>,
    convert_ktx2_processor: Arc<ConvertKTX2Processor>,
    write_processor: Arc<WriteFileProcessor>,

    registry: DashMap<BuildTaskKey, BuildTaskStatis>,

    active_tasks: AtomicUsize,
    completed: Arc<Condvar>,
    guard: Arc<Mutex<bool>>,
}

impl Dispatcher {
    pub fn create() -> Self {
        Self {
            route_target_processor: Arc::new(RouteTargetProcessor::create()),
            shader_processor: Arc::new(ShaderProcessor::create()),
            parse_gltf_processor: Arc::new(ExtractAssetsProcessor::create()),
            convert_ktx2_processor: Arc::new(ConvertKTX2Processor::create()),
            write_processor: Arc::new(WriteFileProcessor::create()),

            registry: DashMap::new(),

            active_tasks: AtomicUsize::new(0),
            completed: Arc::new(Condvar::new()),
            guard: Arc::new(Mutex::new(false)),
        }
    }

    pub fn dispatch(self: Arc<Self>, task: BuildTask) {
        let dispatcher = self.clone();

        let key = BuildTaskKey::from_task(&task);

        match self.registry.entry(key.clone()) {
            Entry::Occupied(_) => {
                return;
            },
            Entry::Vacant(entry) => {
                entry.insert(BuildTaskStatis::Started);
                self.active_tasks.fetch_add(1, Ordering::SeqCst);
            }
        }

        rayon::spawn(move || {
            let result = match task {
                BuildTask::RouteTarget(task) => self.route_target_processor.process(dispatcher, &task),
                BuildTask::CompileShader(task) => self.shader_processor.process(dispatcher, &task),
                BuildTask::ExtractScenes(task) => self.parse_gltf_processor.process(dispatcher, &task),
                BuildTask::ConvertKTX2(task) => { self.convert_ktx2_processor.process(dispatcher, &task) },
                BuildTask::WriteFile(task) => self.write_processor.process(dispatcher, &task),
            };

            if let Err(error) = result {
                self.registry.insert(key.clone(), BuildTaskStatis::Failed);

                error!("Error while dispatching task! {}", error);
            } else {
                self.registry.insert(key.clone(), BuildTaskStatis::Completed);
            }

            let remaining = self.active_tasks.fetch_sub(1, Ordering::SeqCst);

            if remaining == 1 {
                let completed = &*self.completed;
                let mut completed_guard = self.guard.lock().unwrap();

                *completed_guard = true;

                completed.notify_all();
            }
        })
    }

    pub fn wait_all(&self) {
        let completed = &*self.completed;
        let mut completed_guard = self.guard.lock().unwrap();

        while !*completed_guard && self.active_tasks.load(Ordering::SeqCst) > 0 {
            completed_guard = completed.wait(completed_guard).unwrap();
        }
    }
}
