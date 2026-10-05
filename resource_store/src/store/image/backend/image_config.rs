use gpu::ImageDescription;
use gpu::ImageViewDescription;

#[derive(Clone)]
pub struct ImageConfig {
    pub label: String,

    pub image_description: ImageDescription,
    pub image_view_description: ImageViewDescription,

    pub levels: Vec<Vec<u8>>,
}
