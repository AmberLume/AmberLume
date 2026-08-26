use crate::resource_scope::buffer_resource_scope::BufferResourceScope;
use crate::resource_scope::data_resource_scope::DataResourceScope;

pub struct PrepareScopes<'scope> {
    pub data: &'scope mut DataResourceScope,
    pub buffer: &'scope mut BufferResourceScope,
}
