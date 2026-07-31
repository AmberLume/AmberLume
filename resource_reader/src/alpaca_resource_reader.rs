use crate::asset_bytes_source::AssetBytesSource;
use crate::io_provider::IOProvider;
use alpaca::alpaca::index_entry::IndexEntry;
use alpaca::unpacker::alpaca_reader::AlpacaReader;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;
use crate::resource_entry_index::ResourceEntryIndex;
use crate::resource_reader::ResourceReader;

pub struct AlpacaResourceReader {
    alpacas: Vec<AlpacaReader>,

    resource_indices: HashMap<String, ResourceEntryIndex>,
}

impl AlpacaResourceReader {
    pub fn new(io_provider: Arc<dyn IOProvider>) -> Result<Self> {
        let files = io_provider.list_files();

        let mut alpacas = Vec::with_capacity(files.len());
        let mut resource_indices = HashMap::new();

        for (index, file) in files.iter().enumerate() {
            info!("Indexing {}...", file.display());

            let asset = io_provider.open(file)?;
            let alpaca_reader = AlpacaReader::parse(Box::new(AssetBytesSource::new(asset)))?;

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

            let resource_index_entry = ResourceEntryIndex::new(alpaca_index, entry_index);

            indices.insert(entry.name.clone(), resource_index_entry);
        }
    }
}

impl ResourceReader for AlpacaResourceReader {
    fn get_resource(&self, name: &str) -> Result<&[u8]> {
        let resource_entry_index = self
            .resource_indices
            .get(name)
            .expect(&format!("Can't find requested resource index: '{}'", name));

        let alpaca = &self.alpacas[resource_entry_index.alpaca_index];
        let entry = &alpaca.entries[resource_entry_index.entry_index];

        alpaca.read_slice(entry)
    }
}
