use crate::resource::common::resource_provider::ResourceId;
use anyhow::Error;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ResRef<CT> {
    pub meta: ResMeta<CT>,
    pub state: ResState,
}

impl<CT> ResRef<CT> {
    pub fn from(value: CT) -> Self {
        Self {
            meta: ResMeta { config: value },
            state: ResState::New,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResMeta<CT> {
    pub config: CT,
}

#[derive(Debug, Clone)]
pub enum ResState {
    New,
    Pending { id: ResourceId },
    Resolved { id: ResourceId },
    Broken { id: ResourceId, error: Arc<Error> },
    Evicted { id: ResourceId },
}

impl ResState {
    pub fn get_id(&self) -> Option<ResourceId> {
        match self {
            ResState::New => None,
            ResState::Pending { id, .. } => Some(*id),
            ResState::Resolved { id, .. } => Some(*id),
            ResState::Broken { id, .. } => Some(*id),
            ResState::Evicted { id } => Some(*id),
        }
    }
}
