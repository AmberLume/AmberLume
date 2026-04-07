use std::sync::Arc;
use shipyard::Unique;
use crate::resources::alpaca_resource_reader::AlpacaResourceReader;

#[derive(Unique)]
pub struct ResourceLoaderUnique {
    pub alpaca_resource_reader: Arc<AlpacaResourceReader>,
}

impl ResourceLoaderUnique {
    pub fn new(alpaca_resource_reader: Arc<AlpacaResourceReader>) -> Self {
        Self {
            alpaca_resource_reader,
        }
    }
}
