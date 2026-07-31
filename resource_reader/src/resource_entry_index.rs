pub struct ResourceEntryIndex {
    pub alpaca_index: usize,
    pub entry_index: usize,
}

impl ResourceEntryIndex {
    pub fn new(alpaca_index: usize, entry_index: usize) -> Self {
        Self {
            alpaca_index,
            entry_index,
        }
    }
}
