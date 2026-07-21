use std::collections::{HashMap, HashSet};
use std::fs::{create_dir_all, read, write};
use std::path::{Path, PathBuf};
use anyhow::Result;
use blake3::hash;
use parking_lot::Mutex;
use rkyv::{from_bytes, to_bytes, Archive, Deserialize, Serialize};
use rkyv::rancor::Error;

const CACHE_VERSION: u32 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeState {
    Untouched,
    Touched,
    New,
}

#[derive(Archive, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct DependencyRecord {
    pub path: String,
    pub hash: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct CacheNode {
    pub hash: [u8; 32],
    pub dependencies: Vec<DependencyRecord>,
    pub outputs: Vec<String>,
    pub state: NodeState,
}

#[derive(Archive, Deserialize, Serialize, Debug)]
struct PersistedNode {
    hash: [u8; 32],
    dependencies: Vec<DependencyRecord>,
    outputs: Vec<String>,
}

#[derive(Archive, Deserialize, Serialize, Debug)]
struct CacheFile {
    version: u32,
    entries: Vec<(String, PersistedNode)>,
}

pub struct Cache {
    nodes: Mutex<HashMap<String, CacheNode>>,
}

impl Cache {
    pub fn new() -> Self {
        Self { nodes: Mutex::new(HashMap::new()) }
    }

    pub fn load(path: &Path) -> Result<Self> {
        let bytes = read(path)?;
        let file = from_bytes::<CacheFile, Error>(&bytes)?;

        let nodes = if file.version == CACHE_VERSION {
            file.entries.into_iter()
                .map(|(path, node)| (path, CacheNode {
                    hash: node.hash,
                    dependencies: node.dependencies,
                    outputs: node.outputs,
                    state: NodeState::Untouched,
                }))
                .collect::<HashMap<_, _>>()
        } else {
            HashMap::new()
        };

        Ok(Self {
            nodes: Mutex::new(nodes),
        })
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }
        let entries: Vec<(String, PersistedNode)> = self.nodes.lock().iter()
            .filter(|(_, node)| node.state != NodeState::Untouched)
            .map(|(k, node)| (k.clone(), PersistedNode {
                hash: node.hash,
                dependencies: node.dependencies.clone(),
                outputs: node.outputs.clone(),
            }))
            .collect();
        let file = CacheFile {
            version: CACHE_VERSION,
            entries,
        };
        let bytes = to_bytes::<Error>(&file)?;

        write(path, bytes.as_ref())?;

        Ok(())
    }

    pub fn lookup(&self, key: &str) -> Option<CacheNode> {
        self.nodes.lock().get(key).cloned()
    }

    pub fn touch(&self, key: &str, hash: [u8; 32], dependencies: Vec<DependencyRecord>) -> NodeState {
        let mut guard = self.nodes.lock();

        match guard.get_mut(key) {
            Some(node) if node.hash == hash && dependencies_equal(&node.dependencies, &dependencies) => {
                node.state = NodeState::Touched;

                NodeState::Touched
            }
            _ => {
                guard.insert(key.to_string(), CacheNode {
                    hash,
                    dependencies,
                    outputs: Vec::new(),
                    state: NodeState::New,
                });

                NodeState::New
            }
        }
    }

    pub fn invalidate(&self, key: &str) {
        self.nodes.lock().remove(key);
    }

    pub fn record_output(&self, key: &str, output: String) {
        if let Some(node) = self.nodes.lock().get_mut(key) {
            node.outputs.push(output);
        }
    }

    pub fn clear_outputs(&self, key: &str) {
        if let Some(node) = self.nodes.lock().get_mut(key) {
            node.outputs.clear();
        }
    }

    pub fn live_outputs(&self) -> HashSet<PathBuf> {
        self.nodes.lock().iter()
            .filter(|(_, node)| node.state != NodeState::Untouched)
            .flat_map(|(_, node)| node.outputs.iter().map(PathBuf::from))
            .collect()
    }

    pub fn summary(&self) -> (Vec<String>, Vec<String>, Vec<String>) {
        let mut untouched = Vec::new();
        let mut touched = Vec::new();
        let mut new = Vec::new();

        for (key, node) in self.nodes.lock().iter() {
            match node.state {
                NodeState::Untouched => untouched.push(key.clone()),
                NodeState::Touched => touched.push(key.clone()),
                NodeState::New => new.push(key.clone()),
            }
        }

        (untouched, touched, new)
    }
}

pub fn is_dependency_valid(dependency: &DependencyRecord) -> bool {
    match read(&dependency.path) {
        Ok(bytes) => {
            let actual: [u8; 32] = hash(&bytes).into();

            actual == dependency.hash
        }
        Err(_) => false,
    }
}

fn dependencies_equal(left: &[DependencyRecord], right: &[DependencyRecord]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut left_sorted: Vec<&DependencyRecord> = left.iter().collect();
    let mut right_sorted: Vec<&DependencyRecord> = right.iter().collect();

    left_sorted.sort_by(|x, y| x.path.cmp(&y.path));
    right_sorted.sort_by(|x, y| x.path.cmp(&y.path));

    left_sorted == right_sorted
}
