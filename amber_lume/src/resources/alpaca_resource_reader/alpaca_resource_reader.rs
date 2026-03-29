use crate::platform_providers::io_provider::IOProvider;
use alpaca::alpaca::index_entry::IndexEntry;
use alpaca::unpacker::alpaca_reader::AlpacaReader;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

pub struct AlpacaResourceReader {
    alpacas: Vec<AlpacaReader>,

    resource_indices: HashMap<String, ResourceEntryIndex>,
}

pub struct ResourceEntryIndex {
    pub alpaca_index: usize,
    pub entry_index: usize,
}

impl AlpacaResourceReader {
    pub fn new(io_provider: Arc<dyn IOProvider>) -> Result<Self> {
        let files = io_provider.list_files();

        let mut alpacas = Vec::with_capacity(files.len());
        let mut resource_indices = HashMap::new();

        for (index, file) in files.iter().enumerate() {
            info!("Indexing {}...", file.display());

            let alpaca_reader = AlpacaReader::parse(&file)?;

            Self::index_resource_entries(index, &mut resource_indices, &alpaca_reader.entries);

            alpacas.push(alpaca_reader);
        }

        Ok(Self {
            alpacas,

            resource_indices,
        })
    }

    fn index_resource_entries(
        alpaca_index: usize,
        indices: &mut HashMap<String, ResourceEntryIndex>,
        entries: &[IndexEntry],
    ) {
        for (entry_index, entry) in entries.iter().enumerate() {
            info!("Found: {:#?}", entry);

            let resource_index_entry = ResourceEntryIndex {
                alpaca_index,
                entry_index,
            };

            indices.insert(entry.name.clone(), resource_index_entry);
        }
    }

    pub fn get_resource(&self, name: &str) -> Result<&[u8]> {
        let resource_entry_index = self.resource_indices
            .get(name)
            .expect(&format!("Can't find requested resource index: '{}'", name));

        let alpaca = &self.alpacas[resource_entry_index.alpaca_index];
        let entry = &alpaca.entries[resource_entry_index.entry_index];

        alpaca.read_slice(entry)
    }
}
