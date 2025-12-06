// use crate::resources::common::resource_provider::ResourceProvider;
// use crate::resources::index::resource_index::ResourceIndex;
// use crate::resources::io_provider::IOProvider;
// use crate::resources::shader::shader_backend::ShaderBackend;
// use ash::Device;
// use std::sync::Arc;
//
// pub struct ResourceHub {
//     shader_provider: Arc<ResourceProvider<ShaderBackend>>,
// }
//
// impl ResourceHub {
//     pub fn new(device: Device, io_provider: Arc<dyn IOProvider>) -> Self {
//         let resource_index = Arc::new(ResourceIndex::new(io_provider.clone()));
//
//         let shader_provider = ResourceProvider::from(ShaderBackend::new(
//             device,
//             io_provider.clone(),
//             resource_index,
//         ));
//         // let pipeline_provider = ResourceProvider::from(
//         //     PipelineBackend::new(gpu_context.device.clone(), shader_provider.clone())
//         // );
//         // let material_provider =  ResourceProvider::from(
//         //     MaterialBackend::new(gpu_context.clone(), resource_passport.clone(), asset_manager.clone())
//         // );
//         // let model_provider =  ResourceProvider::from(
//         //     ModelBackend::new(gpu_context.clone(), resource_passport.clone(), asset_manager.clone(), material_provider.clone())
//         // );
//
//         // shader_provider.touch(&ShaderConfig {
//         //     name: String::from("basic.vert"),
//         // });
//
//         Self {
//             shader_provider,
//             // pipeline_provider,
//             // model_provider,
//             // material_provider,
//         }
//     }
//
//     // pub fn get_providers(&self) -> (Arc<ResourceProvider<PipelineBackend>>, Arc<ResourceProvider<MaterialBackend>>) {
//     //     (self.pipeline_provider.clone(), self.material_provider.clone(),)
//     // }
//
//     // pub fn ensure_pipeline(&self, resref: &mut ResRef<PipelineConfig>) {
//     //     self.pipeline_provider.ensure(resref)
//     // }
//
//     // pub fn ensure_model(&self, resref: &mut ResRef<String>) {
//     //     self.model_provider.ensure(resref)
//     // }
//
//     // pub fn ensure_material(&self, resref:  &mut ResRef<String>) {
//     //     self.material_provider.ensure(resref)
//     // }
//
//     // pub fn get_ready_pipeline(&self, id: &ResourceId) -> Option<Arc<RenderPipeline>> {
//     //     self.pipeline_provider.get_ready(id, false)
//     // }
//
//     // pub fn get_ready_primitives(&self, id: &ResourceId) -> Option<Arc<Vec<PrimitiveData>>> {
//     //     self.model_provider.get_ready(id, false)
//     // }
//
//     // pub fn get_primitives_buffer_states(&self) -> (BufferState, BufferState) {
//     //     self.model_provider.get_storage()
//     // }
// }
