// use std::sync::Arc;
// use ash::vk::{BorderColor, Format, SamplerAddressMode};
// use crate::render::vulkan::factories::image::managed_image::ManagedImage;
// use crate::resources::common::resource_provider::ResourceProvider;
// use crate::resources::image::image_backend::{ImageBackend};
// use crate::resources::image::image_config::{ImageConfig, ImageSource};
// use crate::resources::sampler::sampler_config::SamplerConfig;
// 
// pub struct ShadowImageHolder {
//     pub shadow_count: u32,
//     pub shadow_images: Vec<ManagedImage>,
// }
// 
// impl ShadowImageHolder {
//     pub fn new(
//         image_provider: Arc<ResourceProvider<ImageBackend>>,
//         shadow_count: u32,
//         width: u32,
//         height: u32,
//     ) -> Self {
//         let shadow_images = (0..shadow_count).map(|index| {
//             let sampler_config = SamplerConfig {
//                 address_mode_u: SamplerAddressMode::CLAMP_TO_EDGE,
//                 address_mode_v: SamplerAddressMode::CLAMP_TO_EDGE,
//                 address_mode_w: SamplerAddressMode::CLAMP_TO_EDGE,
// 
//                 border_color: BorderColor::FLOAT_OPAQUE_WHITE,
// 
//                 ..SamplerConfig::default()
//             };
// 
//             let image_config = ImageConfig {
//                 name: format!("shadow_image_{}", index),
//                 source: ImageSource::Virtual {
//                     data: Vec::new(),
// 
//                     width,
//                     height,
// 
//                     format: Format::D32_SFLOAT,
//                 },
//                 sampler_config,
//             };
// 
// 
//         });
// 
//         Self {
//             shadow_count,
//             shadow_images,
//         }
//     }
// }