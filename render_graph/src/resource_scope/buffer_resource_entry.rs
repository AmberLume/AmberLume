use crate::DynamicBufferMemory;
use ash::vk::DeviceSize;
use gpu::BufferRange;

pub enum BufferResourceEntry {
    Imported {
        range: BufferRange,
    },
    Dynamic {
        label: &'static str,

        memory: DynamicBufferMemory,
        alignment: DeviceSize,

        clear: bool,
    },
}

impl BufferResourceEntry {
    pub fn imported(range: BufferRange) -> Self {
        Self::Imported { range }
    }

    pub fn dynamic(
        label: &'static str,
        memory: DynamicBufferMemory,
        alignment: DeviceSize,
        clear: bool,
    ) -> Self {
        Self::Dynamic {
            label,

            memory,
            alignment,

            clear,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Imported { range } => range.label,
            Self::Dynamic { label, .. } => label,
        }
    }
}
