use crate::profiler::meta_value::MetaValue;

pub struct CpuMetaEntry {
    pub name: &'static str,
    pub value: MetaValue,
}
